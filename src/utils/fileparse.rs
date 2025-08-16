use std::{
    env,
    ffi::OsStr,
    path::{Path, PathBuf},
};

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

// Stolen from rust path source code and slightly refactored since (as of writing this) its a nightly only feature and im not bothered.
pub fn fpre(fpath: &Path) -> Option<&OsStr> {
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
    fpath
        .file_name()
        .map(split_file_at_dot)
        .and_then(|(before, _after)| Some(before))
}

pub fn sanitize_file_name(name: &str) -> String {
    name.replace(std::path::MAIN_SEPARATOR, "_")
        .replace('/', "_")
        .replace('\\', "_")
}
