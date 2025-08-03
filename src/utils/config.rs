use crate::types::Config;
use dirs::config_dir;
use std::{
    fs::{create_dir_all, read_to_string, write},
    path::PathBuf,
};

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
