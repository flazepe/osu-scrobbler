#[path = "config.rs"]
mod config;

use rustfm_scrobble::{Scrobble, Scrobbler};

pub fn get_scrobbler() -> Scrobbler {
    let config = config::get_config();

    let mut scrobbler = Scrobbler::new(&config.last_fm_api_key, &config.last_fm_api_secret);

    match scrobbler.authenticate_with_password(&config.last_fm_username, &config.last_fm_password) {
        Ok(_) => (),
        Err(err) => panic!("An error occurred: {:?}", err),
    };

    scrobbler
}

pub fn set_now_playing(scrobbler: &Scrobbler, title: &str, artist: &str, album: &str) {
    let song = Scrobble::new(artist, title, album);

    match scrobbler.now_playing(&song) {
        Ok(_) => (),
        Err(err) => panic!("An error occurred: {:?}", err),
    };
}

pub fn scrobble(scrobbler: &Scrobbler, title: &str, artist: &str, album: &str) {
    let song = Scrobble::new(artist, title, album);

    match scrobbler.scrobble(&song) {
        Ok(_) => (),
        Err(err) => panic!("An error occurred: {:?}", err),
    };
}
