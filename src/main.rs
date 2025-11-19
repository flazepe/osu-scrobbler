mod config;
mod logger;
mod scores;
mod scrobbler;

use scrobbler::Scrobbler;

fn main() {
    Scrobbler::new().start()
}
