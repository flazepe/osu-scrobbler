mod config;
mod logger;
mod scores;
mod scrobbler;
mod utils;

use scrobbler::Scrobbler;

fn main() {
    Scrobbler::new().start()
}
