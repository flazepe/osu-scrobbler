mod last_fm;
mod listenbrainz;

use crate::{
    config::{get_config, ScrobblerConfig},
    scrobbler::{last_fm::LastfmScrobbler, listenbrainz::ListenBrainzScrobbler},
};
use reqwest::blocking::Client;
use serde::Deserialize;
use std::{
    fmt::{Display, Formatter, Result as FmtResult},
    thread::sleep,
    time::Duration,
};

pub struct Scrobbler {
    config: ScrobblerConfig,
    last_fm: Option<LastfmScrobbler>,
    listenbrainz: Option<ListenBrainzScrobbler>,
    last_score: Option<Score>,
}

pub struct ScrobblerError {
    message: String,
}

impl Display for ScrobblerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.message)
    }
}

#[derive(Clone, Deserialize)]
pub struct Score {
    pub ended_at: String,
    pub beatmap: Beatmap,
    pub beatmapset: Beatmapset,
}

#[derive(Clone, Deserialize)]
pub struct Beatmap {
    pub total_length: u32,
}

#[derive(Clone, Deserialize)]
pub struct Beatmapset {
    pub artist: String,
    pub artist_unicode: String,
    pub title: String,
    pub title_unicode: String,
}

impl Scrobbler {
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
        self.last_score = self.get_last_score();

        loop {
            self.poll();
            sleep(Duration::from_secs(5));
        }
    }

    fn poll(&mut self) {
        let Some(score) = self.get_last_score() else { return; };

        if self
            .last_score
            .as_ref()
            .map_or(true, |last_score| last_score.ended_at != score.ended_at)
        {
            self.scrobble(score);
        }
    }

    fn scrobble(&mut self, score: Score) {
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
            match last_fm.scrobble(title, artist, score.beatmap.total_length) {
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

    fn get_last_score(&self) -> Option<Score> {
        let mut request = Client::new().get(format!(
            "https://osu.ppy.sh/users/{}/scores/recent",
            self.config.user_id,
        ));

        if let Some(mode) = self.config.mode.as_ref() {
            request = request.query(&[("mode", mode)])
        }

        match request
            .send()
            .and_then(|response| response.json::<Vec<Score>>())
        {
            Ok(mut scores) => match scores.is_empty() {
                true => None,
                false => Some(scores.remove(0)),
            },
            Err(_) => panic!("Invalid osu! user ID given."),
        }
    }
}
