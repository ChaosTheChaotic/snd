use crate::types::Config;
use dirs::{config_dir, download_dir};
use flate2::{write::GzEncoder, Compression};
use gethostname::gethostname;
use std::{
    env::{self, temp_dir}, ffi::OsStr, fs::{create_dir_all, read_to_string, write, File}, path::{Path, PathBuf}
};
use tar::Builder;

pub fn get_config_path() -> PathBuf {
    let mut path = config_dir().expect("Could not find config directory");
    path.push("snd");
    path.push("config.conf");
    path
}

pub fn read_config() -> Config {
    let path = get_config_path();
    let mut follow_symlinks = false;
    let mut send_method = "semi-reliable".to_string();

    if path.exists() {
        if let Ok(contents) = read_to_string(&path) {
            for line in contents.lines() {
                if let Some(value) = line.strip_prefix("send_method = ") {
                    send_method = value.trim().to_string();
                }
                if let Some(value) = line.strip_prefix("follow_symlinks = ") {
                    follow_symlinks = value.trim() == "true";
                }
            }
        }
    }
    Config {
        send_method,
        follow_symlinks,
    }
}

pub fn write_config(config: &Config) -> std::io::Result<()> {
    let path = get_config_path();
    if let Some(parent) = path.parent() {
        create_dir_all(parent)?;
    }
    write(
        path,
        format!(
            "send_method = {}\nfollow_symlinks = {}",
            config.send_method, config.follow_symlinks
        ),
    )
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
        "tar" => "tarball",
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

pub fn downloadfc(full_path: &Path) -> (File, PathBuf) {
    let fname: &OsStr = full_path.file_name().unwrap_or_default();
    let dld = download_dir().unwrap_or_default();
    let nname = dld.join(fname);
    let fp: File = File::create(&nname).expect("Failed to create file");
    (fp, nname)
}

// This function creates a tar file but does not remove it. Removing it should be handled by any
// code that calls this
pub fn tarify(fpath: String) -> PathBuf {
    let dir_name = Path::new(&fpath)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("temp_dir");
    
    let tarfpth = temp_dir().join(format!("{}.tar.gz", dir_name));
    let tarfp = File::create(&tarfpth).expect("Failed to create temp file");
    let enc = GzEncoder::new(tarfp, Compression::default());
    let mut tar = Builder::new(enc);
    tar.append_dir_all("", &fpath)
        .expect("Failed to add directory to archive");
    tar.finish().expect("Failed to finish writing to the archive");
    tarfpth
}

// Stolen from rust path source code since its a nightly only feature and im not bothered.
fn split_file_at_dot(file: &OsStr) -> (&OsStr, Option<&OsStr>) {
    let slice = file.as_encoded_bytes();
    if slice == b".." {
        return (file, None);
    }
    // The unsafety here stems from converting between &OsStr and &[u8]
    // and back. This is safe to do because (1) we only look at ASCII
    // contents of the encoding and (2) new &OsStr values are produced
    // only from ASCII-bounded slices of existing &OsStr values.
    let i = match slice[1..].iter().position(|b| *b == b'.') {
        Some(i) => i + 1,
        None => return (file, None),
    };
    let before = &slice[..i];
    let after = &slice[i + 1..];
    unsafe {
        (
            OsStr::from_encoded_bytes_unchecked(before),
            Some(OsStr::from_encoded_bytes_unchecked(after)),
        )
    }
}

pub fn fpre(fpath: &Path) -> Option<&OsStr> {
    fpath.file_name().map(split_file_at_dot).and_then(|(before, _after)| Some(before))
}
