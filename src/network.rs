use crate::utils::{gen_cname, is_vpn};
use colored::Colorize;
use if_addrs::IfAddr;
use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
    net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket},
    time::Duration,
};

pub const PORT: u16 = 58422;

pub fn send_to_ip(ip: IpAddr, msg: String) {
    let socket = UdpSocket::bind("0.0.0.0:0").expect("Failed to bind to a port");
    let target = SocketAddr::new(ip, PORT);
    match socket.send_to(msg.as_bytes(), target) {
        Ok(_) => println!("Sent to {}", target),
        Err(e) => eprintln!("Failed to send to {}: {}", target, e),
    }
}

pub fn begin_broadcast_with_socket(socket: &UdpSocket) {
    let mut sent = false;
    let interfaces = if_addrs::get_if_addrs().expect("Failed to get if addrs");

    for interface in interfaces {
        if interface.is_loopback() || is_vpn(&interface.name) {
            continue;
        }
        if let IfAddr::V4(addr) = interface.addr {
            if let Some(broadcast) = addr.broadcast {
                let target = SocketAddr::new(IpAddr::V4(broadcast), PORT);
                let msg = "Hello from ".to_string() + &gen_cname() + "!";
                match socket.send_to(msg.as_bytes(), &target) {
                    Ok(_) => sent = true,
                    Err(e) => eprintln!("Failed to send via {}: {}", interface.name, e),
                }
            }
        }
    }

    if !sent {
        eprintln!(
            "{}",
            "No valid interfaces found. Trying fallback broadcast...".red()
        );
        let fallback = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(255, 255, 255, 255)), PORT);
        if let Err(e) = socket.send_to("Hello, world!".as_bytes(), fallback) {
            eprintln!("Fallback broadcast failed: {}", e);
        }
    }
}

pub fn send_file(file: File, target: SocketAddr, mode: String) {
    if mode == "semi-reliable" {
        send_file_semi_reliable(file, target);
    } else {
        send_file_legacy(file, target);
    }
}

fn send_file_legacy(mut file: File, target: SocketAddr) {
    let mut buf = [0; 1400];
    let socket = UdpSocket::bind("0.0.0.0:0").expect("Failed to bind to a port");

    let file_size = file.metadata().expect("Failed to get metadata").len();
    let size_bytes = file_size.to_be_bytes();

    // Send file size first
    socket
        .send_to(&size_bytes, &target)
        .expect("Failed to send file size");

    // Process file in chunks
    loop {
        let bytes_read = file.read(&mut buf).expect("Failed to read file");
        if bytes_read == 0 {
            break;
        }
        socket
            .send_to(&buf[..bytes_read], &target)
            .expect("Failed to send data chunk");
    }

    println!("{}", "File transfer complete!".green());
}

fn send_file_semi_reliable(mut file: File, target: SocketAddr) {
    const CHUNK_SIZE: usize = 1392;
    const INITIAL_TIMEOUT: u64 = 100;
    const MAX_TIMEOUT: u64 = 2000;

    let socket = UdpSocket::bind("0.0.0.0:0").expect("Failed to bind to a port");
    socket
        .set_nonblocking(false)
        .expect("Failed to set blocking");
    // Removed initial timeout setting here

    let file_size = file.metadata().expect("Failed to get metadata").len();
    let size_bytes = file_size.to_be_bytes();

    // Send file size first
    socket
        .send_to(&size_bytes, &target)
        .expect("Failed to send file size");

    let mut sent_bytes: u64 = 0;
    let mut sequence_number = 0u64;
    let mut buffer = [0u8; CHUNK_SIZE + 8]; // Extra space for sequence number

    while sent_bytes < file_size {
        // Reset timeout to initial value for each new packet
        let mut current_timeout = INITIAL_TIMEOUT;
        socket
            .set_read_timeout(Some(Duration::from_millis(current_timeout)))
            .expect("Failed to set read timeout");

        // Prepare chunk with sequence number
        let seq_bytes = sequence_number.to_be_bytes();
        buffer[0..8].copy_from_slice(&seq_bytes);

        file.seek(SeekFrom::Start(sent_bytes))
            .expect("Failed to seek file");

        let read_size = file
            .read(&mut buffer[8..])
            .expect("Failed to read file chunk");

        let chunk_end = 8 + read_size;
        let packet = &buffer[..chunk_end];

        let mut ack_received = false;

        while !ack_received {
            // Send the chunk
            if let Err(e) = socket.send_to(packet, &target) {
                eprintln!("Failed to send chunk: {}", e);
            }

            // Wait for ACK
            let mut ack_buffer = [0u8; 8];
            match socket.recv_from(&mut ack_buffer) {
                Ok((_, src)) => {
                    if src == target {
                        let received_seq = u64::from_be_bytes(ack_buffer);
                        if received_seq == sequence_number {
                            ack_received = true;
                        }
                    }
                }
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::WouldBlock
                        || e.kind() == std::io::ErrorKind::TimedOut
                    {
                        // Timeout occurred, will retry
                    } else {
                        eprintln!("Receive error: {}", e);
                    }
                }
            }

            // Handle retry
            if !ack_received {
                current_timeout = (current_timeout * 2).min(MAX_TIMEOUT);
                socket
                    .set_read_timeout(Some(Duration::from_millis(current_timeout)))
                    .expect("Failed to set read timeout");
            }
        }

        sent_bytes += read_size as u64;
        sequence_number += 1;
    }

    println!("{}", "File transfer complete!".green());
}
