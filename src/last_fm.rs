use rustfm_scrobble::{
    responses::{NowPlayingResponse, ScrobbleResponse},
    Scrobble, Scrobbler as RustfmScrobbler, ScrobblerError,
};

pub struct Scrobbler {
    scrobbler: RustfmScrobbler,
}

impl Scrobbler {
    pub fn new(username: &str, password: &str, api_key: &str, api_secret: &str) -> Self {
        let mut scrobbler = RustfmScrobbler::new(api_key, api_secret);

        match scrobbler.authenticate_with_password(username, password) {
            Ok(_) => println!("Authenticated with Last.fm (username {username})."),
            Err(err) => panic!("An error occurred while authenticating to Last.fm: {}", err),
        };

        Self { scrobbler }
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
