use crate::{modes::sh_init, types::ShModes, utils::config::handle_config_subcommand};
use colored::Colorize;

pub fn colored_rec_h() -> String {
    format!(
        "{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}",
        "exit:".yellow().bold(),
        "Exits the program".cyan(),
        "help:".yellow().bold(),
        "Prints this message".cyan(),
        "vdms:".yellow().bold(),
        "View all received direct messages, deleted after 300s to prevent large buildup".cyan(),
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
        "The file size is approx and can be off by a bit though is mostly accurate (this issue is mostly with folders)",
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
