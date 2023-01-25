mod config;
mod last_fm;
mod osu;
mod scrobble_loop;

use last_fm::LastfmScrobbler;
use scrobble_loop::main as scrobble_loop;
use tokio::main;

#[main]
async fn main() {
    println!("Looping...");
    scrobble_loop(&LastfmScrobbler::new(), None).await;
}
