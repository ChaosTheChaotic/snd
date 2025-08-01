use crate::{
    modes::sh_init,
    types::ShModes,
    utils::{get_config_path, read_config, write_config},
};
use colored::Colorize;

pub fn colored_rec_h() -> String {
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

pub fn colorize_help() -> String {
    format!(
        "{}\n{}{}{}{}\n{}{}{}\n{}{}{}\n{}{}{}\n{}{}{}\n{}{}{}\n",
        "snd:".yellow().bold(),
        "--[(h)elp|(V)ersion|(r)ec|(s)nd|(c)onfig]".green(),
        "\n\nCommands parsed in the order listed, first recognised flag will be run\n\n",
        "The file size is approx and can be off by a bit (this issue is mostly with folders)",
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
        "Puts the program into sending mode".cyan(),
        "\n",
        "config:".yellow().bold(),
        "View or change settings".cyan(),
        "\n  To change: --config set <key> <value>".yellow(),
        "\n  To reset: --config reset".yellow(),
    )
}

fn handle_config_subcommand(args: &[String]) -> String {
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
            _ => {
                return format!(
                    "{}\n{}",
                    "Invalid config key!".red(),
                    "Valid keys: send_method, follow_symlinks".yellow()
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

pub fn parse(args: &[String]) -> String {
    for (index, arg) in args.iter().enumerate() {
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
            "--config" | "-c" => {
                let rest_args = args.get(index + 1..).unwrap_or_default();
                return handle_config_subcommand(rest_args);
            }
            _ => {}
        }
    }
    format!(
        "{}\n{}",
        "Command option not found".red().bold(),
        colorize_help()
    )
}
