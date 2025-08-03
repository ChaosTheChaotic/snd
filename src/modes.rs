use crate::{
    c::{
        diskman::du,
        tui::{initTUI, runTUI, termTUI, update_tui_hostnames},
    },
    cli::colored_rec_h,
    network::{begin_broadcast_with_socket, recv_file, send_file, send_to_ip, PORT},
    types::{HostInfo, ShModes, DM},
    utils::{
        config::read_config,
        file::tarify,
        fileparse::{expand_path, get_file_type},
        net::{extract_hostname, gen_cname},
    },
};
use colored::Colorize;
use std::{
    ffi::{c_char, CStr, CString},
    fs::{remove_file, File},
    io::{self, Write},
    net::{SocketAddr, UdpSocket},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

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

fn prompt(shtyp: ShModes, cname: String) {
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

        let recv_socket = socket; // Use original socket for receiving due to errors with binding
                                  // to the same addr multiple times
        thread::spawn(move || {
            let mut buf = [0; 1024];
            loop {
                match recv_socket.recv_from(&mut buf) {
                    Ok((size, source)) => {
                        let message = String::from_utf8_lossy(&buf[..size]).to_string();
                        if message.starts_with("DIRECTH: HMCHNE; ") {
                            const PFX: &str = "DIRECTH: HMCHNE; ";
                            let rest = &message[PFX.len()..].trim();
                            let tokens: Vec<&str> = rest.split(';').map(|s| s.trim()).collect();
                            let wfile_idx = tokens.iter().position(|&t| t == "WFILE");
                            let wtyp_idx = tokens.iter().position(|&t| t == "WTYP");
                            let wsz_idx = tokens.iter().position(|&t| t == "WSZ");
                            let sndm_idx = tokens.iter().position(|&t| t == "SNDM");

                            if let (
                                Some(wfile_idx),
                                Some(wtyp_idx),
                                Some(wsz_idx),
                                Some(sndm_idx),
                            ) = (wfile_idx, wtyp_idx, wsz_idx, sndm_idx)
                            {
                                if wtyp_idx > wfile_idx
                                    && wsz_idx > wtyp_idx
                                    && sndm_idx > wsz_idx
                                    && wtyp_idx < tokens.len()
                                    && wsz_idx < tokens.len()
                                    && sndm_idx < tokens.len()
                                {
                                    let hostname = tokens[0..wfile_idx].join("; ");
                                    let file_path = tokens[wfile_idx + 1..wtyp_idx].join("; ");
                                    let file_type = tokens[wtyp_idx + 1];
                                    let file_size = tokens[wsz_idx + 1].parse::<u64>().unwrap_or(0);
                                    let send_method = tokens[sndm_idx + 1].to_string();

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
                                        send_method,
                                        recv_time: Instant::now(),
                                    });
                                }
                            } else {
                                eprintln!("WFILE marker not found in message");
                            }
                        }
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
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
                    let mut guard = direct_messages.lock().unwrap();
                    guard.retain(|dm| {
                        dm.recv_time.elapsed() < Duration::from_secs(read_config().dm_timeout_s)
                    });
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

    let ftyp = if exp.is_dir() {
        let tar_path = tarify(exp.to_string_lossy().to_string());
        exp = tar_path;
        "directory".to_string()
    } else {
        get_file_type(&exp).to_string()
    };

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
                Err(e) if e.kind() == io::ErrorKind::Interrupted => continue, // Handle EINTR
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
    unsafe {
        libc::free(thostnme as *mut libc::c_void);
    }

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
            "DIRECTH: HMCHNE; {}; WFILE; {}; WTYP; {}; WSZ; {}; SNDM; {}",
            gen_cname(),
            abspath,
            ftyp,
            //std::fs::metadata(Path::new(&abspath)).unwrap().size(),
            unsafe {
                du(
                    CString::new(abspath.as_str())
                        .expect("Failed to convert to CString")
                        .as_ptr(),
                    read_config().follow_symlinks,
                )
            }, // Jesus christ this took forever to work out
            read_config().send_method,
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
                            send_file(
                                File::open(file_path).expect("Failed to open file"),
                                source,
                                read_config().send_method,
                            );
                            if get_file_type(Path::new(&abspath)) == "directory" {
                                remove_file(&exp).expect("Failed to remove temporary tarball")
                            }
                        } else {
                            println!("{}", "Transfer canceled".yellow());
                        }

                        accepted = true;
                    }
                }
            }
            Err(e) if e.kind() == io::ErrorKind::Interrupted => continue, // Handle EINTR
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

pub fn rec(dms: Arc<Mutex<Vec<DM>>>) {
    let mut guard = dms.lock().unwrap();
    guard.retain(|dm| dm.recv_time.elapsed() < Duration::from_secs(read_config().dm_timeout_s));
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

    let socket = UdpSocket::bind(("0.0.0.0", 0)).expect("Failed to bind");
    let target = SocketAddr::new(dm.host_info.ip, PORT);
    let msg = format!("ACCEPT: {}; FROM: {}", dm.file_path, gen_cname());

    if let Err(e) = socket.send_to(msg.as_bytes(), target) {
        eprintln!("Failed to send acceptance: {}", e);
    } else {
        println!(
            "{} {} {} {} {} {}",
            "Request sent to".green(),
            dm.host_info.name.blue().bold(),
            "to send".green(),
            dm.file_path.blue().bold(),
            "using mode".green(),
            dm.send_method.blue().bold(),
        );
    }
    recv_file(&socket, dm);
}
