use crate::config::get_config;
use rustfm_scrobble::{
    responses::{NowPlayingResponse, ScrobbleResponse},
    Scrobble, Scrobbler, ScrobblerError,
};

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

    pub fn set_now_playing(
        &self,
        title: &str,
        artist: &str,
        album: &str,
    ) -> Result<NowPlayingResponse, ScrobblerError> {
        self.scrobbler
            .now_playing(&Scrobble::new(artist, title, album))
    }

    pub fn scrobble(
        &self,
        title: &str,
        artist: &str,
        album: &str,
    ) -> Result<ScrobbleResponse, ScrobblerError> {
        self.scrobbler
            .scrobble(&Scrobble::new(artist, title, album))
    }
}
