use crate::cli::colored_rec_h;
use crate::network::{begin_broadcast_with_socket, send_file, send_to_ip, PORT};
use crate::tui::*;
use crate::types::*;
use crate::utils::{downloadfc, expand_path, extract_hostname, gen_cname, get_file_type};
use colored::Colorize;
use std::{
    fs::File,
    ffi::{c_char, CStr},
    io::{self, Write},
    net::{SocketAddr, UdpSocket},
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

pub fn prompt(shtyp: ShModes, cname: String) {
    if matches!(shtyp, ShModes::REC) {
        let direct_messages: Arc<Mutex<Vec<DM>>> = Arc::new(Mutex::new(Vec::new()));
        let direct_clone = Arc::clone(&direct_messages);

        // Create main socket and set broadcast
        let socket = UdpSocket::bind(("0.0.0.0", PORT)).expect("Failed to bind to port");
        socket.set_broadcast(true).expect("Failed to set broadcast");

        // Clone socket for broadcast thread
        let broadcast_socket = socket.try_clone().expect("Failed to clone socket");

        // Start broadcast thread with cloned socket
        thread::spawn(move || {
            begin_broadcast_with_socket(&broadcast_socket);
            loop {
                thread::sleep(Duration::from_secs(2));
                begin_broadcast_with_socket(&broadcast_socket);
            }
        });

        // Start listener thread with original socket
        let recv_socket = socket; // Use original socket for receiving
        thread::spawn(move || {
            let mut buf = [0; 1024];
            loop {
                match recv_socket.recv_from(&mut buf) {
                    Ok((size, source)) => {
                        let message = String::from_utf8_lossy(&buf[..size]).to_string();
                        if message.starts_with("DIRECTH: HMCHNE; ") {
                            const PREFIX: &str = "DIRECTH: HMCHNE; ";
                            let rest = &message[PREFIX.len()..].trim();
                            let tokens: Vec<&str> = rest.split("; ").collect();

                            // Find WFILE marker position
                            if let Some(wfile_idx) = tokens.iter().position(|&t| t == "WFILE") {
                                // Ensure enough tokens after WFILE
                                if tokens.len() < wfile_idx + 5 {
                                    eprintln!("Invalid message: not enough tokens after WFILE");
                                    return;
                                }

                                // Extract fields based on marker positions
                                let hostname = tokens[0..wfile_idx].join("; ");
                                let file_path = tokens[wfile_idx + 1];

                                // Verify WTYP marker
                                if tokens[wfile_idx + 2] != "WTYP" {
                                    eprintln!("Expected WTYP marker after file path");
                                    return;
                                }
                                let file_type = tokens[wfile_idx + 3];

                                // Verify WSZ marker
                                if tokens[wfile_idx + 4] != "WSZ" {
                                    eprintln!("Expected WSZ marker after file type");
                                    return;
                                }
                                let file_size = tokens[wfile_idx + 5].parse::<u64>().unwrap_or(0);

                                // Add DM to list
                                let mut guard = direct_clone.lock().unwrap();
                                guard.push(DM {
                                    host_info: HostInfo {
                                        name: hostname,
                                        ip: source.ip(),
                                    },
                                    file_path: file_path.to_string(),
                                    file_type: file_type.to_string(),
                                    file_size,
                                });
                            } else {
                                eprintln!("WFILE marker not found in message");
                            }
                        }
                    }
                    Err(e) => eprintln!("Error receiving message: {}", e),
                }
            }
        });

        println!("\n\n\n");
        print_prompt(&shtyp, &cname);
        let mut res = String::new();
        loop {
            res.clear();
            io::stdin()
                .read_line(&mut res)
                .expect("Failed to read line");
            match res.trim() {
                "exit" => break,
                "help" => println!("{}", colored_rec_h()),
                "vdms" => {
                    let guard = direct_messages.lock().unwrap();
                    if guard.is_empty() {
                        println!("No direct messages received yet.");
                    } else {
                        println!("Direct Messages Received:");
                        for (i, msg) in guard.iter().enumerate() {
                            println!("{}. {}", i + 1, msg);
                        }
                    }
                }
                "rec" => rec(direct_messages.clone()),
                _ => println!("{}", "Not a recognised command".red()),
            }
            print_prompt(&shtyp, &cname);
        }
    }
}

fn print_prompt(shtyp: &ShModes, cname: &str) {
    let mode_str = shtyp.to_string();
    let colored_mode = match shtyp {
        ShModes::REC => mode_str.red().bold(),
        ShModes::SND => mode_str.green().bold(),
    };
    let colored_cname = cname.blue().bold();
    let prompt = format!("[{}@{}]# ", colored_mode, colored_cname).bold();
    print!("{}", prompt);
    let _ = io::stdout().flush();
}

fn snd_mode_tui() {
    let mut res: String = String::new();
    let mut valid: bool = false;
    let mut exp: PathBuf = PathBuf::new();

    while !valid {
        res.clear();
        print_prompt(&ShModes::SND, &gen_cname());
        print!(" Input a valid file path to send: ");
        let _ = io::stdout().flush();
        io::stdin()
            .read_line(&mut res)
            .expect("Failed to read line");

        exp = expand_path(res.trim());
        if exp.exists() {
            valid = true;
        } else {
            println!(
                "{}",
                "Provided full path does not exist. Please put in an existing file".red()
            )
        }
    }

    let abspath: String = exp.to_string_lossy().to_string();
    println!("{} is a valid file at {}!", res.trim(), abspath);

    unsafe {
        initTUI();
    }

    let stop_flag = Arc::new(Mutex::new(false));
    let hostnames: Arc<Mutex<Vec<HostInfo>>> = Arc::new(Mutex::new(Vec::new()));
    let hostnames_clone = Arc::clone(&hostnames);

    // Start UDP listener thread with stop flag
    let stop_flag_clone = Arc::clone(&stop_flag);
    let handle = thread::spawn(move || {
        let socket = UdpSocket::bind(("0.0.0.0", PORT)).expect("Failed to bind to port");
        socket
            .set_read_timeout(Some(Duration::from_millis(100)))
            .expect("Failed to set read timeout");
        let mut buf = [0; 1024];
        loop {
            // Check stop flag
            if *stop_flag_clone.lock().unwrap() {
                break;
            }

            match socket.recv_from(&mut buf) {
                Ok((size, source)) => {
                    let message = String::from_utf8_lossy(&buf[..size]);
                    let hostname = extract_hostname(&message);
                    let mut guard = hostnames_clone.lock().unwrap();

                    if !guard.iter().any(|h| h.name == hostname) {
                        guard.push(HostInfo {
                            name: hostname.clone(),
                            ip: source.ip(),
                        });

                        let names: Vec<String> = guard.iter().map(|h| h.name.clone()).collect();
                        update_tui_hostnames(&names);
                    }
                }
                Err(e)
                    if e.kind() == io::ErrorKind::WouldBlock
                        || e.kind() == io::ErrorKind::TimedOut =>
                {
                    continue;
                }
                Err(e) => eprintln!("Receive error: {}", e),
            }
        }
    });

    let thostnme: *const c_char = unsafe { runTUI() };
    let thnms = unsafe { CStr::from_ptr(thostnme) }
        .to_str()
        .unwrap_or("Failed to get result from runTUI")
        .to_string();

    // Stop listener thread
    *stop_flag.lock().unwrap() = true;
    handle.join().expect("Failed to join listener thread");

    let target_ip = {
        let guard = hostnames.lock().unwrap();
        guard
            .iter()
            .find(|h| h.name == thnms)
            .map(|h| h.ip)
            .expect("Host not found")
    };

    println!("{}", target_ip);
    send_to_ip(
        target_ip,
        format!(
            "DIRECTH: HMCHNE; {}; WFILE; {}; WTYP; {}; WSZ; {}",
            gen_cname(),
            abspath,
            get_file_type(Path::new(&abspath)),
            std::fs::metadata(Path::new(&abspath)).unwrap().size()
        ),
    );

    unsafe {
        termTUI();
    }
    println!("{}", "Waiting for receiver to accept...".yellow());
    let socket = UdpSocket::bind(("0.0.0.0", PORT)).expect("Failed to bind");
    socket
        .set_read_timeout(Some(Duration::from_secs(30)))
        .expect("Failed to set timeout");

    let mut buf = [0; 1024];
    let mut accepted = false;

    while !accepted {
        match socket.recv_from(&mut buf) {
            Ok((size, source)) => {
                let message = String::from_utf8_lossy(&buf[..size]);
                if message.starts_with("ACCEPT: ") {
                    let file_path = message["ACCEPT: ".len()..]
                        .split(';')
                        .next()
                        .unwrap_or("")
                        .trim();
                    let from_host = message.split("FROM: ").nth(1).unwrap_or("").trim();

                    if file_path == abspath {
                        println!(
                            "{} {} {} {} {}",
                            "Receiver".green(),
                            from_host.blue().bold(),
                            "accepted".green(),
                            "file:".green(),
                            file_path.blue().bold()
                        );

                        // Prompt for confirmation
                        print!("Send file? (y/N): ");
                        io::stdout().flush().unwrap();

                        let mut response = String::new();
                        io::stdin().read_line(&mut response).unwrap();

                        if response.trim().eq_ignore_ascii_case("y") {
                            if let Err(e) = socket.send_to("FSNT;".as_bytes(), source) {
                                eprintln!("Failed to send FSNT;: {}", e);
                            } else {
                                println!("Sent FSNT; to {}", source);
                            }
                            send_file(File::open(file_path).expect("Failed to open file"), source);
                        } else {
                            println!("{}", "Transfer canceled".yellow());
                        }

                        accepted = true;
                    }
                }
            }
            Err(e)
                if e.kind() == io::ErrorKind::WouldBlock || e.kind() == io::ErrorKind::TimedOut =>
            {
                println!("{}", "No acceptance received. Timing out...".yellow());
                break;
            }
            Err(e) => {
                eprintln!("Receive error: {}", e);
                break;
            }
        }
    }
}

pub fn sh_init(shtyp: ShModes) {
    match shtyp {
        ShModes::REC => prompt(shtyp, gen_cname()),
        ShModes::SND => snd_mode_tui(),
    }
}

fn rec(dms: Arc<Mutex<Vec<DM>>>) {
    let guard = dms.lock().unwrap();
    if guard.is_empty() {
        println!("No direct messages received yet.");
        return;
    }

    println!("Direct Messages Received:");
    for (i, msg) in guard.iter().enumerate() {
        println!("{}. {}", i + 1, msg);
    }

    println!("Type the index of the message you would like to accept (or 'cancel' to cancel)");
    let _ = io::stdout().flush();

    let mut res = String::new();
    io::stdin()
        .read_line(&mut res)
        .expect("Failed to read line");
    let res = res.trim();

    if res.eq_ignore_ascii_case("cancel") {
        println!("{}", "Acceptance canceled".yellow());
        return;
    }

    let idx = match res.parse::<usize>() {
        Ok(num) if num > 0 && num <= guard.len() => num - 1,
        _ => {
            println!("{}", "Invalid index. Please enter a valid number".red());
            return;
        }
    };

    let dm = &guard[idx];
    println!("{}: {}", "You selected".green(), dm);

    // Send acceptance message to the sender
    //let socket = UdpSocket::bind("0.0.0.0:0").expect("Failed to bind");
    let socket = UdpSocket::bind(("0.0.0.0", 0)).expect("Failed to bind");
    let target = SocketAddr::new(dm.host_info.ip, PORT);
    let msg = format!("ACCEPT: {}; FROM: {}", dm.file_path, gen_cname());

    if let Err(e) = socket.send_to(msg.as_bytes(), target) {
        eprintln!("Failed to send acceptance: {}", e);
    } else {
        println!(
            "{} {} {} {}",
            "Request sent to".green(),
            dm.host_info.name.blue().bold(),
            "to send".green(),
            dm.file_path.blue().bold()
        );
    }
    let mut buf = [0; 1024];
    match socket.recv_from(&mut buf) {
        Ok((size, source)) => {
            let msg = String::from_utf8_lossy(&buf[..size]).to_string();
            if msg.trim() == "FSNT;" {
                println!("{}", "File being sent!".green());
                let mut fp: File = downloadfc(&Path::new(&dm.file_path));
                let mut size_buf = [0u8; 8];
                socket.recv_from(&mut size_buf).expect("Failed to receive file size");
                let file_size = u64::from_be_bytes(size_buf);
                let mut remaining = file_size;
                let mut chunk_buf = [0u8; 1024];
                
                while remaining > 0 {
                    let (count, _) = socket.recv_from(&mut chunk_buf)
                        .expect("Failed to receive chunk");
                    fp.write_all(&chunk_buf[..count])
                        .expect("Failed to write chunk");
                    remaining -= count as u64;
                }
            }
        },
        Err(e) => eprintln!("Receive error: {}", e)
    }
}
