use std::process::exit;

use clap::Parser;
use moodly::Cli;

fn main() {
    ctrlc::set_handler(|| exit(0)).expect("Error setting Ctrl-C handler");

    if let Err(e) = Cli::parse().run() {
        println!("Encountered an error: {e}");
        exit(1);
    }
}
