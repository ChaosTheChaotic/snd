use crate::utils::*;
use colored::Colorize;
use if_addrs::IfAddr;
use std::{net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket}, fs::File, io::Read};

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

pub fn send_file(file: File, target: SocketAddr) {
    let mut buf = [0; 1024];
    let socket = UdpSocket::bind("0.0.0.0:0").expect("Failed to bind to a port");
    
    // Get file size and convert to big-endian bytes
    let file_size = file.metadata().expect("Failed to get metadata").len();
    let size_bytes = file_size.to_be_bytes();
    
    // Send file size first
    socket.send_to(&size_bytes, &target)
        .expect("Failed to send file size");

    // Process file in chunks
    let mut file = file;
    loop {
        let bytes_read = file.read(&mut buf).expect("Failed to read file");
        if bytes_read == 0 {
            break;
        }
        socket.send_to(&buf[..bytes_read], &target)
            .expect("Failed to send data chunk");
    }
    
    println!("{}", "File transfer complete!".green());
}
