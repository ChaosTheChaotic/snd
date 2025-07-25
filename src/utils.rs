use dirs::{config_dir, download_dir};
use gethostname::gethostname;
use std::{
    env,
    ffi::OsStr,
    fs::{create_dir_all, read_to_string, write, File},
    path::{Path, PathBuf},
};

pub struct Config {
    pub send_method: String,
}

pub fn get_config_path() -> PathBuf {
    let mut path = config_dir().expect("Could not find config directory");
    path.push("snd");
    path.push("config.conf");
    path
}

pub fn read_config() -> Config {
    let path = get_config_path();
    if path.exists() {
        if let Ok(contents) = read_to_string(&path) {
            for line in contents.lines() {
                if let Some(value) = line.strip_prefix("send_method = ") {
                    return Config {
                        send_method: value.trim().to_string(),
                    };
                }
            }
        }
    }
    Config {
        send_method: "semi-reliable".to_string(),
    }
}

pub fn write_config(config: &Config) -> std::io::Result<()> {
    let path = get_config_path();
    if let Some(parent) = path.parent() {
        create_dir_all(parent)?;
    }
    write(path, format!("send_method = {}", config.send_method))
}

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
        "sql" | "sqlite" | "sqlite3" => "SQL database",
        "db" => "Database file",
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
