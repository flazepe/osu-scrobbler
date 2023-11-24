mod config;
mod logger;
mod scores;
mod scrobbler;

use anyhow::Result;
use scrobbler::Scrobbler;

fn main() -> Result<()> {
    Scrobbler::new()?.start();
    Ok(())
}
