use gethostname::gethostname;
use std::{
    env, ffi::OsStr, fs::File, path::{Path, PathBuf}
};
use dirs::download_dir;

pub fn human_readable_size(size: u64) -> String {
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

pub fn expand_path(input: &str) -> PathBuf {
    let expanded = if input.starts_with('~') {
        if let Some(home) = dirs::home_dir() {
            if input == "~" {
                home
            } else if input.starts_with("~/") || input.starts_with("~\\") {
                home.join(&input[2..])
            } else {
                PathBuf::from(input)
            }
        } else {
            PathBuf::from(input)
        }
    } else if input.contains('$') {
        let mut result = input.to_string();
        if let Ok(home) = env::var("HOME") {
            result = result.replace("$HOME", &home);
        }
        PathBuf::from(result)
    } else {
        PathBuf::from(input)
    };

    if expanded.is_relative() {
        env::current_dir().unwrap().join(expanded)
    } else {
        expanded
    }
}

pub fn get_file_type(path: &Path) -> &'static str {
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
        // ... rest of the file type matches ...
        _ => "file",
    }
}

pub fn is_vpn(name: &str) -> bool {
    if cfg!(windows) {
        let patterns = ["TAP", "OpenVPN", "WireGuard", "ZeroTier", "Tailscale"];
        patterns.iter().any(|p| name.to_uppercase().contains(p))
    } else if cfg!(unix) {
        let patterns = ["tun", "tap", "ppp", "zt", "tailscale", "utun", "vpn"];
        patterns.iter().any(|p| name.starts_with(p))
    } else {
        false
    }
}

pub fn gen_cname() -> String {
    gethostname()
        .to_str()
        .unwrap_or("nohostnameerror")
        .to_string()
}

pub fn extract_hostname(message: &str) -> String {
    if let Some(start) = message.find("from ") {
        if let Some(end) = message.find('!') {
            if start + 5 < end {
                return message[start + 5..end].to_string();
            }
        }
    }
    message.trim_end_matches('!').to_string()
}

pub fn downloadfc(full_path: &Path) -> File {
    let fname: &OsStr = full_path.file_name().unwrap_or_default();
    let dld = download_dir().unwrap_or_default();
    let nname = dld.join(fname);
    let fp: File = File::create(nname).expect("Failed to create file");
    return fp;
}
