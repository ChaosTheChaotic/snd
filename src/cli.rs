use crate::{
    modes::sh_init,
    types::ShModes,
    utils::{get_config_path, read_config, write_config},
};
use colored::Colorize;

pub fn colorize_help() -> String {
    format!(
        "{}\n{}{}{}\n{}{}{}\n{}{}{}\n{}{}{}\n{}{}{}\n{}",
        "snd:".yellow().bold(),
        "--[(h)elp|(V)ersion|(r)ec|(s)nd|(c)onfig]".green(),
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
        "Puts the program into sending mode".cyan(),
        "\n",
        "config:".yellow().bold(),
        "View or change settings".cyan(),
    )
}

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

fn handle_config_subcommand(args: &[String]) -> String {
    if args.is_empty() {
        let config = read_config();
        let path = get_config_path();

        format!(
            "{}\n{}\n\n  {}: {}\n    {}\n\n  {}: {}\n    {}\n\n{}\n{}",
            "Available settings:".yellow().bold(),
            format!("(Stored at: {})", path.display()).dimmed(),
            "1. legacy".green().bold(),
            config.send_method == "legacy",
            "Reliable for small files but looses info for larger files".cyan(),
            "2. semi-reliable".green().bold(),
            config.send_method == "semi-reliable",
            "Uses the same method as legacy but with extra protections to ensure most of the file gets through, generally much slower than legacy due to the wait times for ACK response".cyan(),
            "Send methods will always be decided based on who is sending the file, this means even with conflicting send methods, the method of the sender takes priority".yellow(),
            "To change: --config set <1|2>".yellow(),
        )
    } else if args.len() >= 2 && args[0] == "set" {
        let choice = &args[1];
        let new_value = match choice.as_str() {
            "1" => "legacy",
            "2" => "semi-reliable",
            _ => {
                return format!(
                    "{}\n{}",
                    "Invalid choice! Use:".red(),
                    "--config set <1|2>\n  1 = legacy\n  2 = semi-reliable".yellow()
                );
            }
        };

        let mut config = read_config();
        config.send_method = new_value.to_string();
        if let Err(e) = write_config(&config) {
            return format!("{}: {}", "Failed to save config".red(), e);
        }

        format!(
            "{} {} {}\n{}",
            "Send method set to".green(),
            new_value.bright_cyan().bold(),
            "successfully!".green(),
            format!("(Updated at: {})", get_config_path().display()).dimmed()
        )
    } else {
        format!(
            "{}\n{}",
            "Invalid config command".red(),
            "Usage:\n  --config: View settings\n  --config set <1|2>: Change send method".yellow()
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
