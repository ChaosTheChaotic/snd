use colored::Colorize;
use gethostname::gethostname;
use if_addrs::IfAddr;
use std::{
    env, ffi::{c_char, CStr, CString}, fmt, io::{self, Write}, net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket}, os::unix::fs::MetadataExt, path::Path, sync::{Arc, Mutex}, thread, time::Duration
};

const PORT: u16 = 58422;

// Since I couldnt find an easier way to get the addresses of something via the hostname alone
#[derive(Debug)]
struct HostInfo {
    name: String,
    ip: IpAddr,
}

#[derive(Debug)]
struct DM {
    host_info: HostInfo,
    file_path: String,
    file_type: String,
    file_size: u64,
}


fn human_readable_size(size: u64) -> String {
    const UNITS: [&str; 6] = ["bytes", "KB", "MB", "GB", "TB", "PB"];
    let mut size = size as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{:.0} {}", size, UNITS[unit_index])
    } else {
        format!("{:.2} {}", size, UNITS[unit_index])
    }
}

impl fmt::Display for DM {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let size_str = human_readable_size(self.file_size);
        write!(f, "From {} with ip {} and {}: {} with size {}", 
            self.host_info.name, 
            self.host_info.ip, 
            self.file_type, 
            self.file_path, 
            size_str)
    }
}

unsafe extern "C" {
    fn initTUI();
    fn termTUI();
    fn runTUI() -> *const c_char;
    fn setHostnames(hostnames: *mut *const c_char, count: i32);
}

#[derive(Debug)]
enum ShModes {
    REC,
    SND,
}

impl fmt::Display for ShModes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

fn get_file_type(path: &Path) -> &'static str {
    if path.is_dir() {
        return "directory";
    }
    let ext = match path.extension() {
        Some(e) => e,
        None => return "file",
    };
    let ext_str = match ext.to_str() {
        Some(s) => s,
        None => return "file",
    };
    match ext_str {
        "rs" => "Rust file",
        "py" => "Python file",
        "js" => "JavaScript file",
        "java" => "Java file",
        "c" => "C file",
        "cpp" | "cc" => "C++ file",
        "h" | "hpp" => "Header file",
        "kt" => "Kotlin file",
        "ts" => "Typescript",
        "sh" | "bash" | "zsh" => "Shell script",
        "bashrc" | "zshrc" | "profile" | "zprofile" | "bash_profile" => "Shell init script",
        "txt" => "Text file",
        "gitignore" => "gitignore file",
        "zip" => "zip file",
        "so" => "Shared object file",
        "dll" => "Data linked library",
        "exe" => "Windows executable",
        "mp3" => "MP3 Audio file",
        "m4a" => "m4a Audio file",
        "mp4" => "MP4 Video file",
        "m4v" => "m4v Video file",
        "mov" => "mov Video file",
        "desktop" => "Linux desktop meta file",
        "bin" => "Binary file",
        "png" => "PNG Image",
        "flac" => "flac Audio File",
        "jpeg" | "jpg" => "JPEG Image",
        "blob" => "blob file",
        "tsx" => "Typescript react file",
        "jsx" => "Javascript react file",
        "yaml" | "yml" => "yaml file",
        "toml" => "toml file",
        "cs" => "C# file",
        "html" => "html file",
        "lua" => "lua file",
        "dart" => "dart file",
        "go" => "go file",
        "conf" => "Config file",
        "css" => "css file",
        "json" => "json file",
        "asm" | "s" => "Assembly file",
        "m" => "Objective C file",
        "zig" => "zig file",
        "gradle" => "gradle file",
        "php" => "php file",
        "rb" => "ruby",
        "md" => "markdown",
        "AppImage" => "App image file",
        "ld" => "Linker script",
        "jkr" => "Balatro joker save file",
        "bepis" => "Ultrakill save file",
        "love" => "Love game",
        "qml" => "Qt markup language file",
        "svg" => "SVG image",
        "ttf" => "ttf font",
        "otf" => "otf font",
        "gif" => "gif file",
        "iso" => "Installation media file",
        "patch" | "diff" => "Diff file",
        "smali" => "Android smali file",
        "cmake" => "cmake source code file",
        _ => "file"
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

fn gen_cname() -> String {
    return gethostname()
        .to_str()
        .unwrap_or("nohostnameerror")
        .to_string();
}

fn send_to_ip(ip: IpAddr, msg: String) {
    let socket = UdpSocket::bind("0.0.0.0:0").expect("Failed to bind to a port");
    let target = SocketAddr::new(ip, PORT);
    match socket.send_to(msg.as_bytes(), target) {
        Ok(_) => println!("Sent to {}", target),
        Err(e) => eprintln!("Failed to send to {}: {}", target, e),
    }
}

fn begin_broadcast_with_socket(socket: &UdpSocket) {
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
        eprintln!("{}", "No valid interfaces found. Trying fallback broadcast...".red());
        let fallback = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(255, 255, 255, 255)), PORT);
        if let Err(e) = socket.send_to("Hello, world!".as_bytes(), fallback) {
            eprintln!("Fallback broadcast failed: {}", e);
        }
    }
}

// Cant think of a better way to do this icl
fn is_vpn(name: &str) -> bool {
    if cfg!(windows) {
        // Windows VPN interfaces
        let patterns = ["TAP", "OpenVPN", "WireGuard", "ZeroTier", "Tailscale"];
        patterns.iter().any(|p| name.to_uppercase().contains(p))
    } else if cfg!(unix) {
        // Linux/macOS VPN interfaces
        let patterns = ["tun", "tap", "ppp", "zt", "tailscale", "utun", "vpn"];
        patterns.iter().any(|p| name.starts_with(p))
    } else {
        false // Unknown platform
    }
}

fn extract_hostname(message: &str) -> String {
    // The message format: "Hello from <hostname>!"
    if let Some(start) = message.find("from ") {
        if let Some(end) = message.find('!') {
            if start + 5 < end {
                return message[start + 5..end].to_string();
            }
        }
    }
    // Fallback: use the whole message without the exclamation if any
    message.trim_end_matches('!').to_string()
}

fn update_tui_hostnames(hostnames: &Vec<String>) {
    let c_strings: Vec<CString> = hostnames
        .iter()
        .map(|s| CString::new(s.as_str()).expect("CString::new failed"))
        .collect();
    let mut pointers: Vec<*const c_char> = c_strings.iter().map(|cs| cs.as_ptr()).collect();
    unsafe {
        setHostnames(pointers.as_mut_ptr(), pointers.len() as i32);
    }
}

fn snd_mode_tui() {

    let mut res: String = String::new();
    let mut valid: bool = false;
    while !valid {
        res.clear();
        print_prompt(&ShModes::SND, &gen_cname());
        println!(" Input a valid full file path to send: ");
        io::stdin().read_line(&mut res).expect("Failed to read line");
        if std::fs::exists(&res.trim()).expect("File might not exist") {
            valid = true;
        } else {
            println!("{}", "Provided full file path does not exist. Please put in an existing file ".red())
        }
    }
    println!("{} is a valid file!", res);

    unsafe {
        initTUI();
    }

    let hostnames: Arc<Mutex<Vec<HostInfo>>> = Arc::new(Mutex::new(Vec::new()));
    let hostnames_clone = Arc::clone(&hostnames);

    // Start UDP listener thread
    thread::spawn(move || {
        let socket = UdpSocket::bind(("0.0.0.0", PORT)).expect("Failed to bind to port");
        let mut buf = [0; 1024];
        loop {
            match socket.recv_from(&mut buf) {
                Ok((size, source)) => {
                    let message = String::from_utf8_lossy(&buf[..size]);
                    let hostname = extract_hostname(&message);
                    let mut guard = hostnames_clone.lock().unwrap();

                    // Check if we already have this host
                    if !guard.iter().any(|h| h.name == hostname) {
                        guard.push(HostInfo {
                            name: hostname.clone(),
                            ip: source.ip(),
                        });

                        // Update TUI with just hostnames
                        let names: Vec<String> = guard.iter().map(|h| h.name.clone()).collect();
                        update_tui_hostnames(&names);
                    }
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

    print!("{:?}\n", thnms);

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
            res.trim(),
            get_file_type(Path::new(&res.trim())),
            std::fs::metadata(Path::new(&res.trim())).unwrap().size()
        )
    );

    unsafe {
        termTUI();
    }

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
                "rec" => println!("Placeholder"),
                _ => println!("{}", "Not a recognised command".red())
            }
            print_prompt(&shtyp, &cname);
        }
    }
}

fn sh_init(shtyp: ShModes) {
    match shtyp {
        ShModes::REC => prompt(shtyp, gen_cname()),
        ShModes::SND => snd_mode_tui(),
    }
}

fn colorize_help() -> String {
    format!(
        "{}\n{}{}{}\n{}{}{}\n{}{}{}\n{}{}{}\n{}",
        "snd:".yellow().bold(),
        "--[(h)elp|(V)ersion|(r)ec|(s)nd]".green(),
        "\n\nCommands parsed in the order listed, first recognised flag will be run\n\n",
        "help:".yellow().bold(),
        "Prints this help message".cyan(),
        "\n",
        "Version:".yellow().bold(),
        "Prints version number".cyan(),
        "\n",
        "rec:".yellow().bold(),
        "Puts the program into receving mode".cyan(),
        "\n",
        "snd:".yellow().bold(),
        "Puts the program into sending mode".cyan()
    )
}

fn colored_rec_h() -> String {
    format!(
        "{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}",
        "exit:".yellow().bold(),
        "Exits the program".cyan(),
        "help:".yellow().bold(),
        "Prints this message".cyan(),
        "vdms:".yellow().bold(),
        "View all received direct messages".cyan(),
        "rec".yellow().bold(),
        "Accepts a dm from the machine, takes in the index of the wanted message as a param".cyan(),
    )
}

// Main parser for the cmd line flags
fn parse(args: &[String]) -> String {
    for arg in args {
        match arg.as_str() {
            "--help" | "-h" => return colorize_help(),
            "--version" | "-V" => {
                return env!("CARGO_PKG_VERSION").bright_cyan().bold().to_string()
            }
            "--rec" | "-r" => {
                sh_init(ShModes::REC);
                return "Done.".bright_green().to_string();
            }
            "--snd" | "-s" => {
                sh_init(ShModes::SND);
                return "Done.".bright_green().to_string();
            }
            _ => {}
        }
    }
    // No valid flags found
    format!(
        "{}\n{}",
        "Command option not found".red().bold(),
        colorize_help()
    )
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let parsed: String = parse(&args[1..]); // 1.. Ignores the first arg which is the binary path
    println!("{}", parsed);
}
