mod config;
mod last_fm;
mod osu;
mod scrobble_loop;

use scrobble_loop::start;

fn main() {
    println!("Started scrobbler!");
    start();
}
