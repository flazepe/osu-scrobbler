mod config;
mod last_fm;
mod listenbrainz;
mod osu;

use osu::scrobbler::OsuScrobbler;

fn main() {
    OsuScrobbler::new().start();
}
