mod cli;
mod modes;
mod network;
mod tui;
mod types;
mod utils;

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let parsed: String = cli::parse(&args[1..]);
    println!("{}", parsed);
}
