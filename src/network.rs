use crate::{
    types::PendingPacket,
    utils::{gen_cname, is_vpn},
};
use colored::Colorize;
use if_addrs::IfAddr;
use std::{
    collections::VecDeque,
    fs::File,
    io::{Read, Seek, SeekFrom},
    net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket},
    time::{Duration, Instant},
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
    const WINDOW_SIZE: usize = 32;
    const INITIAL_TIMEOUT_MS: u64 = 100;
    const MAX_TIMEOUT_MS: u64 = 2000;

    let socket = UdpSocket::bind("0.0.0.0:0").expect("Failed to bind");
    socket
        .set_nonblocking(false)
        .expect("Failed to set blocking");
    socket
        .set_read_timeout(Some(Duration::from_millis(10)))
        .expect("Failed to set read timeout");

    let file_size = file.metadata().expect("Failed to get metadata").len();
    let size_bytes = file_size.to_be_bytes();
    socket
        .send_to(&size_bytes, &target)
        .expect("Failed to send file size");

    let mut sent_bytes: u64 = 0;
    let mut sequence_number = 0u64;
    let mut pending_packets = VecDeque::with_capacity(WINDOW_SIZE);
    let mut timeout = INITIAL_TIMEOUT_MS;

    while sent_bytes < file_size || !pending_packets.is_empty() {
        // Send new packets while window isn't full
        while pending_packets.len() < WINDOW_SIZE && sent_bytes < file_size {
            let mut buffer = vec![0u8; CHUNK_SIZE];
            file.seek(SeekFrom::Start(sent_bytes))
                .expect("Failed to seek file");

            let read_size = file.read(&mut buffer).expect("Failed to read chunk");
            buffer.truncate(read_size);

            let seq_bytes = sequence_number.to_be_bytes();
            let mut packet = Vec::with_capacity(8 + read_size);
            packet.extend_from_slice(&seq_bytes);
            packet.extend_from_slice(&buffer);

            if let Err(e) = socket.send_to(&packet, &target) {
                eprintln!("Failed to send chunk: {}", e);
            }

            pending_packets.push_back(PendingPacket {
                sequence_number,
                data: packet,
                last_sent: Instant::now(),
                timeout_duration: Duration::from_millis(timeout),
                retry_count: 0,
            });

            sent_bytes += read_size as u64;
            sequence_number += 1;
        }

        // Receive ACKs
        let mut ack_buffer = [0u8; 8];
        while let Ok((size, src)) = socket.recv_from(&mut ack_buffer) {
            if src != target {
                continue;
            }

            if size == 8 {
                let ack_seq = u64::from_be_bytes(ack_buffer);
                if let Some(pos) = pending_packets
                    .iter()
                    .position(|p| p.sequence_number == ack_seq)
                {
                    pending_packets.remove(pos);
                    // Reset timeout after successful ACK
                    timeout = INITIAL_TIMEOUT_MS;
                }
            }
        }

        // Handle timeouts and retransmissions
        let now = Instant::now();
        for packet in &mut pending_packets {
            if now.duration_since(packet.last_sent) >= packet.timeout_duration {
                if let Err(e) = socket.send_to(&packet.data, &target) {
                    eprintln!("Failed to resend chunk: {}", e);
                } else {
                    packet.retry_count += 1;
                    packet.last_sent = now;
                    // Exponential backoff
                    packet.timeout_duration = Duration::from_millis(
                        (packet.timeout_duration.as_millis() as u64 * 2).min(MAX_TIMEOUT_MS),
                    );
                }
            }
        }

        // Avoid busy waiting
        std::thread::sleep(Duration::from_millis(1));
    }

    println!("{}", "File transfer complete!".green());
}
