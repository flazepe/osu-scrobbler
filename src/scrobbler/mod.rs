mod last_fm;
mod listenbrainz;

use crate::{
    config::{get_config, ScrobblerConfig},
    scrobbler::{last_fm::LastfmScrobbler, listenbrainz::ListenBrainzScrobbler},
};
use colored::Colorize;
use reqwest::{blocking::Client, StatusCode};
use serde::Deserialize;
use serde_json::from_str;
use std::{thread::sleep, time::Duration};

pub struct Scrobbler {
    config: ScrobblerConfig,
    last_fm: Option<LastfmScrobbler>,
    listenbrainz: Option<ListenBrainzScrobbler>,
    last_score: Option<Score>,
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
            last_fm: config
                .last_fm
                .map(|last_fm| LastfmScrobbler::new(last_fm.username, last_fm.password, last_fm.api_key, last_fm.api_secret)),
            listenbrainz: config.listenbrainz.map(|listenbrainz| ListenBrainzScrobbler::new(listenbrainz.user_token)),
            last_score: None,
        }
    }

    pub fn start(&mut self) {
        println!("{} Started!", "[Scrobbler]".bright_green());

        // Set the initial last score
        self.last_score = self.get_last_score();

        loop {
            self.poll();
            sleep(Duration::from_secs(5));
        }
    }

    fn poll(&mut self) {
        let Some(score) = self.get_last_score() else { return; };

        if self.last_score.as_ref().map_or(true, |last_score| last_score.ended_at != score.ended_at) {
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

        println!("{} New score found: {}", "[Scrobbler]".bright_green(), format!("{artist} - {title}").bright_blue());

        if let Some(last_fm) = self.last_fm.as_ref() {
            match last_fm.scrobble(title, artist, score.beatmap.total_length) {
                Ok(_) => println!("\t{} Successfully scrobbled score.", "[Last.fm]".bright_green()),
                Err(error) => println!("\t{} {error}", "[Last.fm]".bright_red()),
            };
        }

        if let Some(listenbrainz) = self.listenbrainz.as_ref() {
            match listenbrainz.scrobble(title, artist, score.beatmap.total_length) {
                Ok(_) => println!("\t{} Successfully scrobbled score.", "[ListenBrainz]".bright_green()),
                Err(error) => println!("\t{} {error}", "[ListenBrainz]".bright_red()),
            };
        }

        self.last_score = Some(score);
    }

    fn get_last_score(&self) -> Option<Score> {
        let mut request = Client::new().get(format!("https://osu.ppy.sh/users/{}/scores/recent", self.config.user_id));

        if let Some(mode) = self.config.mode.as_ref() {
            request = request.query(&[("mode", mode)]);
        }

        let Ok(response) = request.send() else { return None; };

        if response.status() == StatusCode::NOT_FOUND {
            panic!("{} Invalid osu! user ID given.", "[Scrobbler]".bright_red());
        }

        let Ok(text) = response.text() else { return None; };

        match from_str::<Vec<Score>>(text.as_str()) {
            Ok(mut scores) => match scores.is_empty() {
                true => None,
                false => Some(scores.remove(0)),
            },
            Err(_) => {
                println!("{} Could not parse response from osu! API: {text}", "[Scrobbler]".bright_red());
                None
            },
        }
    }
}
