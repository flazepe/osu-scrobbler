mod last_fm;
mod listenbrainz;

use crate::{
    config::{get_config, ScrobblerConfig},
    scrobbler::{last_fm::LastfmScrobbler, listenbrainz::ListenBrainzScrobbler},
};
use colored::Colorize;
use reqwest::{blocking::Client, StatusCode};
use serde::Deserialize;
use std::{thread::sleep, time::Duration};

pub struct Scrobbler {
    config: ScrobblerConfig,
    last_fm: Option<LastfmScrobbler>,
    listenbrainz: Option<ListenBrainzScrobbler>,
    recent_score: Option<Score>,
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
            recent_score: None,
        }
    }

    pub fn start(&mut self) {
        println!("{} Started!", "[Scrobbler]".bright_green());

        // Set the initial last score
        self.recent_score = self.get_recent_score();

        loop {
            self.poll();
            sleep(Duration::from_secs(5));
        }
    }

    fn poll(&mut self) {
        if let Some(score) = self.get_recent_score() {
            if self.recent_score.as_ref().map_or(true, |recent_score| recent_score.ended_at != score.ended_at) {
                self.scrobble(score);
            }
        }
    }

    fn scrobble(&mut self, score: Score) {
        if score.beatmap.total_length < self.config.min_beatmap_length_secs {
            return;
        }

        let title = if self.config.use_original_metadata {
            &score.beatmapset.title_unicode
        } else {
            &score.beatmapset.title
        };

        let artist = if self.config.use_original_metadata {
            &score.beatmapset.artist_unicode
        } else {
            &score.beatmapset.artist
        };

        println!("{} New score found: {}", "[Scrobbler]".bright_green(), format!("{artist} - {title}").bright_blue());

        if let Some(last_fm) = self.last_fm.as_ref() {
            match last_fm.scrobble(title, artist, score.beatmap.total_length) {
                Ok(_) => println!("\t{} Successfully scrobbled score.", "[Last.fm]".bright_green()),
                Err(error) => println!("\t{} Error while scrobbling score: {}", "[Last.fm]".bright_red(), error),
            }
        }

        if let Some(listenbrainz) = self.listenbrainz.as_ref() {
            match listenbrainz.scrobble(title, artist, score.beatmap.total_length) {
                Ok(_) => println!("\t{} Successfully scrobbled score.", "[ListenBrainz]".bright_green()),
                Err(error) => println!("\t{} Error while scrobbling score: {}", "[ListenBrainz]".bright_red(), error),
            }
        }

        self.recent_score = Some(score);
    }

    fn get_recent_score(&self) -> Option<Score> {
        let mut request = Client::new().get(format!("https://osu.ppy.sh/users/{}/scores/recent", self.config.user_id));

        if let Some(mode) = self.config.mode.as_ref() {
            request = request.query(&[("mode", mode)]);
        }

        let response = match request.send() {
            Ok(response) => response,
            Err(error) => {
                println!("{} Error while getting user's recent score: {}", "[Scrobbler]".bright_red(), error);
                return None;
            }
        };

        let status_code = response.status();

        if status_code != StatusCode::OK {
            match status_code {
                StatusCode::NOT_FOUND => panic!("{} Invalid osu! user ID given.", "[Scrobbler]".bright_red()),
                _ => {
                    println!("{} Error while getting user's recent score: Received status code {}.", "[Scrobbler]".bright_red(), status_code);
                    return None;
                }
            }
        }

        let scores = match response.json::<Vec<Score>>() {
            Ok(scores) => scores,
            Err(error) => {
                println!("{} Error while parsing scores: {}", "[Scrobbler]".bright_red(), error);
                return None;
            }
        };

        scores.into_iter().next()
    }
}
