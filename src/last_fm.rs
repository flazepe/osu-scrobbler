#[path = "config.rs"]
mod config;

use rustfm_scrobble::{Scrobble, Scrobbler};

pub fn get_scrobbler() -> Scrobbler {
    let config = config::get_config();

    let mut scrobbler = Scrobbler::new(&config.last_fm_api_key, &config.last_fm_api_secret);

    if let Err(err) =
        scrobbler.authenticate_with_password(&config.last_fm_username, &config.last_fm_password)
    {
        panic!("An error occurred: {:?}", err);
    };

    scrobbler
}

pub fn set_now_playing(scrobbler: &Scrobbler, title: &str, artist: &str, album: &str) {
    let song = Scrobble::new(artist, title, album);

    if let Err(err) = scrobbler.now_playing(&song) {
        panic!("An error occurred: {:?}", err);
    };
}

pub fn scrobble(scrobbler: &Scrobbler, title: &str, artist: &str, album: &str) {
    let song = Scrobble::new(artist, title, album);

    if let Err(err) = scrobbler.scrobble(&song) {
        panic!("An error occurred: {:?}", err);
    };
}
