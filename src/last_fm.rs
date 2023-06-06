use rustfm_scrobble::{
    responses::{NowPlayingResponse, ScrobbleResponse},
    Scrobble, Scrobbler, ScrobblerError,
};

pub struct LastfmScrobbler {
    scrobbler: Scrobbler,
}

impl LastfmScrobbler {
    pub fn new(username: &str, password: &str, api_key: &str, api_secret: &str) -> Self {
        let mut scrobbler = Scrobbler::new(api_key, api_secret);

        match scrobbler.authenticate_with_password(username, password) {
            Ok(_) => println!("Authenticated with Last.fm (username: {username})."),
            Err(error) => panic!("An error occurred while authenticating to Last.fm: {error}"),
        };

        Self { scrobbler }
    }

    pub fn set_now_playing(
        &self,
        title: &str,
        artist: &str,
    ) -> Result<NowPlayingResponse, ScrobblerError> {
        self.scrobbler
            .now_playing(&Scrobble::new(artist, title, title))
    }

    pub fn scrobble(&self, title: &str, artist: &str) -> Result<ScrobbleResponse, ScrobblerError> {
        self.scrobbler
            .scrobble(&Scrobble::new(artist, title, title))
    }
}
