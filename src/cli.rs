use crate::modes::sh_init;
use crate::types::ShModes;
use colored::Colorize;

pub fn colorize_help() -> String {
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

pub fn parse(args: &[String]) -> String {
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
    format!(
        "{}\n{}",
        "Command option not found".red().bold(),
        colorize_help()
    )
}
