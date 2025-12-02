mod config;
mod logger;
mod scores;
mod scrobbler;
mod utils;

use anyhow::Result;
use logger::Logger;
use scrobbler::Scrobbler;
use std::{io::stdin, process::exit};

fn main() -> Result<()> {
    if let Err(error) = Scrobbler::new().and_then(|mut scrobbler| scrobbler.start()) {
        Logger::error("Scrobbler", format!("{error:?}"), false);
        println!("\nPress enter to exit.");
        let _ = stdin().read_line(&mut String::new());
        exit(1);
    }

    Ok(())
}
