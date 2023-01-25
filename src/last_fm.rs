use crate::config::get_config;
use rustfm_scrobble::{Scrobble, Scrobbler};

pub struct LastfmScrobbler {
    scrobbler: Scrobbler,
}

impl LastfmScrobbler {
    pub fn new() -> Self {
        let config = get_config();

        let mut scrobbler = Scrobbler::new(&config.last_fm.api_key, &config.last_fm.api_secret);

        if let Err(err) =
            scrobbler.authenticate_with_password(&config.last_fm.username, &config.last_fm.password)
        {
            panic!(
                "An error occurred while trying to authenticate to Last.fm: {}",
                err
            );
        };

        LastfmScrobbler { scrobbler }
    }

    pub fn set_now_playing(&self, title: &str, artist: &str, album: &str) {
        let song = Scrobble::new(artist, title, album);

        if let Err(err) = self.scrobbler.now_playing(&song) {
            panic!(
                "An error occurred while trying to set Last.fm now playing: {}",
                err
            );
        };
    }

    pub fn scrobble(&self, title: &str, artist: &str, album: &str) {
        let song = Scrobble::new(artist, title, album);

        if let Err(err) = self.scrobbler.scrobble(&song) {
            panic!(
                "An error occurred while trying to scrobble in Last.fm: {}",
                err
            );
        };
    }
}
