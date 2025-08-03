use crate::types::Config;
use colored::Colorize;
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
    let mut dm_timeout_s: u64 = 300;
    let mut encrypt: bool = true;

    if path.exists() {
        if let Ok(contents) = read_to_string(&path) {
            for line in contents.lines() {
                if let Some(value) = line.strip_prefix("send_method = ") {
                    send_method = value.trim().to_string();
                }
                if let Some(value) = line.strip_prefix("follow_symlinks = ") {
                    follow_symlinks = value.trim() == "true";
                }
                if let Some(value) = line.strip_prefix("dm_timeout_s = ") {
                    dm_timeout_s = value.trim().parse().unwrap_or(300);
                }
                if let Some(value) = line.strip_prefix("encrypt = ") {
                    encrypt = value.trim() == "false";
                }
            }
        }
    }
    Config {
        send_method,
        follow_symlinks,
        dm_timeout_s,
        encrypt,
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
            "send_method = {}\nfollow_symlinks = {}\ndm_timeout_s = {}",
            config.send_method, config.follow_symlinks, config.dm_timeout_s,
        ),
    )
}

pub fn handle_config_subcommand(args: &[String]) -> String {
    if args.is_empty() {
        let config = read_config();
        let path = get_config_path();

        format!(
            "{}\n{}\n\n  {}: {}\n    {}\n\n  {}: {}\n    {}\n\n{}\n{}\n{}",
            "Available settings:".yellow().bold(),
            format!("(Stored at: {})", path.display()).dimmed(),
            "1. send_method".green().bold(),
            config.send_method,
            "Legacy is faster at the cost of reliablity, semi-reliable is slower but more reliable"
                .cyan(),
            "2. follow_symlinks".green().bold(),
            config.follow_symlinks,
            "Follow symbolic links when calculating file sizes".cyan(),
            "Send methods will always be decided based on who is sending the file".yellow(),
            "To change: --config set <key> <value>".yellow(),
            "To reset: --config reset".yellow(),
        )
    } else if args[0] == "set" {
        if args.len() < 3 {
            return format!(
                "{}\n{}",
                "Invalid config command".red(),
                "Usage: --config set <key> <value>\n  Example: --config set send_method legacy"
                    .yellow()
            );
        }

        let key = &args[1].to_lowercase();
        let value = &args[2].to_lowercase();
        let mut config = read_config();

        match key.as_str() {
            "send_method" => {
                config.send_method = match value.as_str() {
                    "legacy" | "1" => "legacy".to_string(),
                    "semi-reliable" | "2" => "semi-reliable".to_string(),
                    _ => {
                        return format!(
                            "{}\n{}",
                            "Invalid value for send_method!".red(),
                            "Valid options: legacy (or 1), semi-reliable (or 2)".yellow()
                        );
                    }
                };
            }
            "follow_symlinks" => {
                config.follow_symlinks = match value.as_str() {
                    "true" | "1" | "yes" | "on" => true,
                    "false" | "0" | "no" | "off" => false,
                    _ => {
                        return format!(
                            "{}\n{}",
                            "Invalid value for follow_symlinks!".red(),
                            "Valid options: true/yes/1, false/no/0".yellow()
                        );
                    }
                };
            }
            "dm_timeout_secs" => {
                config.dm_timeout_s = match value.parse::<u64>() {
                    Ok(val) if val > 0 => val,
                    _ => {
                        return format!(
                            "{}\n{}",
                            "Invalid value for dm_timeout_secs!".red(),
                            "Must be a positive integer".yellow()
                        );
                    }
                };
            }
            "encrypt" => {
                config.encrypt = match value.as_str() {
                    "true" | "1" | "yes" | "on" => true,
                    "false" | "0" | "no" | "off" => false,
                    _ => {
                        return format!(
                            "{}\n{}",
                            "Invalid value for encrypt!".red(),
                            "Valid options: true/yes/1, false/no/0".yellow(),
                        )
                    }
                }
            }
            _ => {
                return format!(
                    "{}\n{}",
                    "Invalid config key!".red(),
                    "Valid keys: send_method, follow_symlinks, dm_timeout_s".yellow()
                );
            }
        }

        if let Err(e) = write_config(&config) {
            return format!("{}: {}", "Failed to save config".red(), e);
        }

        format!(
            "{} {} = {}\n{}",
            "Config setting".green(),
            key.bright_cyan().bold(),
            value.bright_cyan().bold(),
            format!("(Updated at: {})", get_config_path().display()).dimmed()
        )
    } else if args[0] == "reset" {
        let default_config = crate::types::Config {
            send_method: "semi-reliable".to_string(),
            follow_symlinks: false,
            dm_timeout_s: 300,
            encrypt: true,
        };

        if let Err(e) = write_config(&default_config) {
            return format!("{}: {}", "Failed to reset config".red(), e);
        }

        format!(
            "{}\n{}\n{}",
            "Config reset to default values:".green(),
            format!("  send_method = {}", default_config.send_method),
            format!("  follow_symlinks = {}", default_config.follow_symlinks)
        )
    } else {
        format!(
            "{}\n{}",
            "Invalid config command".red(),
            "Usage:\n  --config: View settings\n  --config set <key> <value>: Change setting\n  --config reset: Reset to default"
                .yellow()
        )
    }
}
