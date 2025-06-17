// use std::path::PathBuf;

use std::process::exit;

use clap::Parser;
use moodly::{Cli, Commands};

fn main() {
    ctrlc::set_handler(|| exit(0)).expect("Error setting Ctrl-C handler");

    let cli = Cli::parse();

    if let Err(e) = moodly::record() {
        println!("Encountered an error: {e}");
    }

    // match &cli.command {
    //     Some(Commands::Clean { force: _ }) => {
    //         todo!()
    //     }
    //     None => {
    //         if let Err(e) = moodly::record() {
    //             println!("Encountered an error: {e}");
    //         }
    //     }
    // }
}
