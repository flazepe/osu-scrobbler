use crate::config::get_config;

use rustfm_scrobble::{Scrobble, Scrobbler};

pub fn get_scrobbler() -> Scrobbler {
    let config = get_config();

    let mut scrobbler = Scrobbler::new(&config.last_fm.api_key, &config.last_fm.api_secret);

    if let Err(err) =
        scrobbler.authenticate_with_password(&config.last_fm.username, &config.last_fm.password)
    {
        panic!(
            "An error occurred while trying to authenticate to Last.fm: {:?}",
            err
        );
    };

    scrobbler
}

pub fn set_now_playing(scrobbler: &Scrobbler, title: &str, artist: &str, album: &str) {
    let song = Scrobble::new(artist, title, album);

    if let Err(err) = scrobbler.now_playing(&song) {
        panic!(
            "An error occurred while trying to set Last.fm now playing: {:?}",
            err
        );
    };
}

pub fn scrobble(scrobbler: &Scrobbler, title: &str, artist: &str, album: &str) {
    let song = Scrobble::new(artist, title, album);

    if let Err(err) = scrobbler.scrobble(&song) {
        panic!(
            "An error occurred while trying to scrobble in Last.fm: {:?}",
            err
        );
    };
}
