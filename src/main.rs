mod config;
mod last_fm;
mod osu;

use osu::scrobbler::Scrobbler;

fn main() {
    Scrobbler::new().start();
}
