mod config;
mod last_fm;
mod osu;
mod scrobble_loop;

use scrobble_loop::main as scrobble_loop;
use tokio::main;

#[main]
async fn main() {
    println!("Looping...");
    scrobble_loop(&last_fm::get_scrobbler(), None).await;
}
