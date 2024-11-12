mod last_fm;
mod listenbrainz;

use crate::{
    config::{get_config, ScrobblerConfig},
    exit,
    logger::{log_error, log_file, log_success},
    scores::{get_recent_score, Score},
    scrobbler::{last_fm::LastfmScrobbler, listenbrainz::ListenBrainzScrobbler},
};
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
    pub fn new() -> Self {
        let config = get_config();

        if config.last_fm.is_none() && config.listenbrainz.is_none() {
            exit!("Scrobbler", "Please provide config for either Last.fm or ListenBrainz.");
        }

        Self {
            config: config.scrobbler,
            last_fm: config.last_fm.map(|config| LastfmScrobbler::new(config.username, config.password, config.api_key, config.api_secret)),
            listenbrainz: config.listenbrainz.map(|config| ListenBrainzScrobbler::new(config.user_token)),
            recent_score: None,
            cooldown_secs: 0,
        }
    }

    pub fn start(&mut self) {
        log_success("Scrobbler", "Started!");

        self.recent_score = get_recent_score(self.config.user_id, &self.config.mode).unwrap_or(None);

        loop {
            self.cooldown_secs = 0;
            self.poll();
            self.cooldown_secs += 5;
            sleep(Duration::from_secs(self.cooldown_secs));
        }
    }

    fn poll(&mut self) {
        match get_recent_score(self.config.user_id, &self.config.mode) {
            Ok(score) => {
                let Some(score) = score else { return };

                if self.recent_score.as_ref().map_or(true, |recent_score| recent_score.ended_at != score.ended_at) {
                    self.scrobble(score);
                }
            },
            Err(error) => {
                if error.to_string().contains("404") {
                    exit!("Scrobbler", "Invalid osu! user ID given.");
                }

                log_error("Scrobbler", error);
                self.cooldown_secs += 10;
            },
        }
    }

    fn scrobble(&mut self, score: Score) {
        if score.beatmap.total_length < self.config.min_beatmap_length_secs.unwrap_or(60) {
            return;
        }

        let (artist_original, artist_romanized) = (&score.beatmapset.artist_unicode, &score.beatmapset.artist);
        let (title_original, title_romanized) = (&score.beatmapset.title_unicode, &score.beatmapset.title);

        let (mut artist, title) = if self.config.use_original_metadata.unwrap_or(true) {
            (artist_original, title_original)
        } else {
            (artist_romanized, title_romanized)
        };

        let mut redirected_text = "".into();

        if let Some(artist_redirects) = &self.config.artist_redirects {
            if let Some((old_artist, new_artist)) = artist_redirects.iter().find(|(old_artist, new_artist)| {
                artist != new_artist
                    && [artist_original.to_lowercase(), artist_romanized.to_lowercase()].contains(&old_artist.to_lowercase())
            }) {
                artist = new_artist;
                redirected_text = format!(" (redirected from {})", old_artist.bright_blue());
            }
        }

        log_success("Scrobbler", format!("New score found: {}{redirected_text} - {}", artist.bright_blue(), title.bright_blue()));

        if self.config.log_scrobbles.unwrap_or(false) {
            log_file(format!("[{}] {artist} - {title}", score.ended_at));
        }

        if let Some(last_fm) = self.last_fm.as_ref() {
            match last_fm.scrobble(artist, title, score.beatmap.total_length) {
                Ok(_) => log_success("\tLast.fm", "Successfully scrobbled score."),
                Err(error) => log_error("\tLast.fm", error),
            };
        }

        if let Some(listenbrainz) = self.listenbrainz.as_ref() {
            match listenbrainz.scrobble(artist, title, score.beatmap.total_length) {
                Ok(_) => log_success("\tListenBrainz", "Successfully scrobbled score."),
                Err(error) => log_error("\tListenBrainz", error),
            };
        }

        self.recent_score = Some(score);
    }
}
