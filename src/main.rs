mod config;
mod last_fm;
mod nerinyan;
mod osu;
mod osu_scrobble;
mod scrobble_loop;

use tokio::main;

#[main]
async fn main() {
    println!("Looping...");
    scrobble_loop::main(&last_fm::get_scrobbler(), None).await;
}
