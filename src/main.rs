use std::{env, fmt, io::{self, Write}};
use colored::Colorize;

#[derive(Debug)]
enum ShModes {
    REC,
    SND,
}

impl fmt::Display for ShModes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

fn print_prompt(shtyp: &ShModes, cname: &str) {
    let mode_str = shtyp.to_string();
    let colored_mode = match shtyp {
        ShModes::REC => mode_str.red().bold(),
        ShModes::SND => mode_str.green().bold(),
    };
    let colored_cname = cname.blue().bold();
    let prompt = format!("[{}@{}]# ", colored_mode, colored_cname).bold();
    print!("{}", prompt);
    let _ = io::stdout().flush();
}

// This should later have some logic to create a cname
fn gen_cname() -> String {
    return "placeholder".to_string();
}

fn prompt(shtyp: ShModes, cname: String) {
    println!("\n\n\n");
    print_prompt(&shtyp, &cname);
    let mut res = String::new();
    loop {
        res.clear();
        io::stdin().read_line(&mut res).expect("Failed to read line");
        match res.trim() {
            "exit" => break,
            "help" => println!("placeholder"),
            _ => {}
        }
        print_prompt(&shtyp, &cname);
    }
}

fn sh_init(shtyp: ShModes) {
    prompt(shtyp, gen_cname());
}

fn colorize_help() -> String {
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

// Main parser for the cmd line flags
fn parse(args: &[String]) -> String {
    for arg in args {
        match arg.as_str() {
            "--help" | "-h" => return colorize_help(),
            "--version" | "-V" => return env!("CARGO_PKG_VERSION").bright_cyan().bold().to_string(),
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
    // No valid flags found
    format!(
        "{}\n{}",
        "Command option not found".red().bold(),
        colorize_help()
    )
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let parsed: String = parse(&args[1..]); // 1.. Ignores the first arg which is the binary path
    println!("{}", parsed);
}
