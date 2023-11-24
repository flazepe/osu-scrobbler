mod last_fm;
mod listenbrainz;

use crate::{
    config::{get_config, ScrobblerConfig},
    logger::{log_error, log_file, log_success},
    scores::{get_recent_score, Score},
    scrobbler::{last_fm::LastfmScrobbler, listenbrainz::ListenBrainzScrobbler},
};
use anyhow::Result;
use colored::Colorize;
use std::{thread::sleep, time::Duration};

pub struct Scrobbler {
    config: ScrobblerConfig,
    last_fm: Option<LastfmScrobbler>,
    listenbrainz: Option<ListenBrainzScrobbler>,
    recent_score: Option<Score>,
    cooldown_secs: u64,
}

impl Scrobbler {
    pub fn new() -> Result<Self> {
        let config = get_config();

        Ok(Self {
            config: config.scrobbler,
            last_fm: config.last_fm.and_then(|config| {
                LastfmScrobbler::new(config.username, config.password, config.api_key, config.api_secret).map_or_else(
                    |_| {
                        log_error("Last.fm", "Invalid credentials provided.");
                        None
                    },
                    |scrobbler| {
                        log_success("Last.fm", format!("Successfully authenticated with username {}.", scrobbler.username.bright_blue()));
                        Some(scrobbler)
                    },
                )
            }),
            listenbrainz: config.listenbrainz.and_then(|config| {
                ListenBrainzScrobbler::new(config.user_token).map_or_else(
                    |_| {
                        log_error("ListenBrainz", "Invalid user token provided.");
                        None
                    },
                    |scrobbler| {
                        log_success(
                            "ListenBrainz",
                            format!("Successfully authenticated with username {}.", scrobbler.username.bright_blue()),
                        );
                        Some(scrobbler)
                    },
                )
            }),
            recent_score: None,
            cooldown_secs: 0,
        })
    }

    pub fn start(&mut self) {
        log_success("Scrobbler", "Started!");

        // Set the initial last score
        self.recent_score = get_recent_score(self.config.user_id, &self.config.mode).unwrap_or(None);

        loop {
            self.cooldown_secs = 0;
            self.poll();
            self.cooldown_secs += 5;
            sleep(Duration::from_secs(self.cooldown_secs));
        }
    }

    fn poll(&mut self) {
        let Some(score) = get_recent_score(self.config.user_id, &self.config.mode).unwrap_or_else(|error| {
            // Exit on invalid user ID
            if error.to_string().contains("404") {
                log_error("Scrobbler", "Invalid osu! user ID given.");
                panic!();
            }

            log_error("Scrobbler", error);

            // Increase cooldown by 10s since API returned an error
            self.cooldown_secs += 10;

            None
        }) else {
            return;
        };

        if self.recent_score.as_ref().map_or(true, |recent_score| recent_score.ended_at != score.ended_at) {
            self.scrobble(score);
        }
    }

    fn scrobble(&mut self, score: Score) {
        if score.beatmap.total_length < self.config.min_beatmap_length_secs.unwrap_or(60) {
            return;
        }

        let title = match self.config.use_original_metadata.unwrap_or(true) {
            true => &score.beatmapset.title_unicode,
            false => &score.beatmapset.title,
        };

        let artist = match self.config.use_original_metadata.unwrap_or(true) {
            true => &score.beatmapset.artist_unicode,
            false => &score.beatmapset.artist,
        };

        log_success("Scrobbler", format!("New score found: {}", format!("{artist} - {title}").bright_blue()));

        if self.config.log_scrobbles.unwrap_or(false) {
            log_file(format!("[{} UTC] {artist} - {title}", score.ended_at));
        }

        if let Some(last_fm) = self.last_fm.as_ref() {
            match last_fm.scrobble(title, artist, score.beatmap.total_length) {
                Ok(_) => log_success("\tLast.fm", "Successfully scrobbled score."),
                Err(error) => log_error("\tLast.fm", error),
            };
        }

        if let Some(listenbrainz) = self.listenbrainz.as_ref() {
            match listenbrainz.scrobble(title, artist, score.beatmap.total_length) {
                Ok(_) => log_success("\tListenBrainz", "Successfully scrobbled score."),
                Err(error) => log_error("\tListenBrainz", error),
            };
        }

        self.recent_score = Some(score);
    }
}
