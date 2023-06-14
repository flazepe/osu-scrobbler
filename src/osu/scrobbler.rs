use crate::{
    config::{get_config, ScrobblerConfig},
    last_fm::LastfmScrobbler,
    listenbrainz::ListenBrainzScrobbler,
    osu::api::{get_last_score, OsuScore},
};
use std::{thread::sleep, time::Duration};

pub struct OsuScrobbler {
    config: ScrobblerConfig,
    last_fm: Option<LastfmScrobbler>,
    listenbrainz: Option<ListenBrainzScrobbler>,
    last_score: Option<OsuScore>,
}

impl OsuScrobbler {
    pub fn new() -> Self {
        let config = get_config();

        Self {
            config: config.scrobbler,
            last_fm: config.last_fm.map(|last_fm| {
                LastfmScrobbler::new(
                    &last_fm.username,
                    &last_fm.password,
                    &last_fm.api_key,
                    &last_fm.api_secret,
                )
            }),
            listenbrainz: config
                .listenbrainz
                .map(|listenbrainz| ListenBrainzScrobbler::new(&listenbrainz.user_token)),
            last_score: None,
        }
    }

    pub fn start(&mut self) {
        println!("Started osu-scrobbler!");

        // Set the initial last score
        self.last_score = get_last_score(self.config.user_id, self.config.mode.as_ref().cloned());

        loop {
            self.poll();
            sleep(Duration::from_secs(5));
        }
    }

    fn poll(&mut self) {
        let Some(score) = get_last_score(self.config.user_id, self.config.mode.as_ref().cloned()) else { return; };

        if self
            .last_score
            .as_ref()
            .map_or(true, |last_score| last_score.ended_at != score.ended_at)
        {
            self.scrobble(score);
        }
    }

    fn scrobble(&mut self, score: OsuScore) {
        if score.beatmap.total_length < self.config.min_beatmap_length_secs {
            return;
        }

        let title = match self.config.use_original_metadata {
            true => &score.beatmapset.title_unicode,
            false => &score.beatmapset.title,
        };

        let artist = match self.config.use_original_metadata {
            true => &score.beatmapset.artist_unicode,
            false => &score.beatmapset.artist,
        };

        println!("New score found: {artist} - {title}");

        if let Some(last_fm) = self.last_fm.as_ref() {
            match last_fm.scrobble(title, artist) {
                Ok(_) => println!("Scrobbled to Last.fm ^"),
                Err(error) => {
                    println!("An error occurred while scrobbling ^ to Last.fm: {error}")
                }
            }
        }

        if let Some(listenbrainz) = self.listenbrainz.as_ref() {
            match listenbrainz.scrobble(title, artist, score.beatmap.total_length) {
                Ok(_) => println!("Scrobbled to ListenBrainz ^"),
                Err(error) => {
                    println!("An error occurred while scrobbling ^ to ListenBrainz: {error}")
                }
            }
        }

        self.last_score = Some(score);
    }
}
