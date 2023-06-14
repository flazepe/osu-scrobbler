mod config;
mod scrobbler;

use scrobbler::Scrobbler;

fn main() {
    Scrobbler::new().start();
}
