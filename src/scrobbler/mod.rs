mod last_fm;
mod listenbrainz;

use crate::{
    config::{ScrobblerConfig, get_config},
    exit,
    logger::{log_error, log_file, log_success},
    scores::Score,
    scrobbler::{last_fm::LastfmScrobbler, listenbrainz::ListenBrainzScrobbler},
};
use colored::Colorize;
use reqwest::blocking::Client;
use std::{sync::LazyLock, thread::sleep, time::Duration};

static REQWEST: LazyLock<Client> = LazyLock::new(Client::new);

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
            exit!("Scrobbler", "Please provide configuration for either Last.fm or ListenBrainz.");
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

        self.recent_score = Score::get_user_recent(self.config.user_id, &self.config.mode).unwrap_or(None);

        loop {
            self.cooldown_secs = 0;
            self.poll();
            self.cooldown_secs += 5;
            sleep(Duration::from_secs(self.cooldown_secs));
        }
    }

    fn poll(&mut self) {
        match Score::get_user_recent(self.config.user_id, &self.config.mode) {
            Ok(score) => {
                let Some(score) = score else { return };

                if self.recent_score.as_ref().is_some_and(|recent_score| recent_score.ended_at != score.ended_at) {
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

        let (artist_romanized, artist_original) = (&score.beatmapset.artist, &score.beatmapset.artist_unicode);
        let (title_romanized, title_original) = (&score.beatmapset.title, &score.beatmapset.title_unicode);

        let mut artist = artist_romanized;
        let mut title = title_romanized;
        let album = score.get_album_name();

        if self.config.use_original_metadata.unwrap_or(true) {
            artist = artist_original;
            title = title_original;
        }

        let redirected_text = self
            .config
            .artist_redirects
            .as_ref()
            .and_then(|artist_redirects| {
                artist_redirects.iter().find(|(old, new)| {
                    [artist_original.to_lowercase(), artist_romanized.to_lowercase()].contains(&old.to_lowercase())
                        && new.to_lowercase() != artist.to_lowercase()
                })
            })
            .map(|(old, new)| {
                artist = new;
                format!(" (redirected from {})", old.bright_blue())
            });

        log_success(
            "Scrobbler",
            format!(
                "New score found: {}{} - {} ({})",
                artist.bright_blue(),
                redirected_text.as_deref().unwrap_or(""),
                title.bright_blue(),
                album.as_deref().unwrap_or("Unknown Album").bright_blue(),
            ),
        );

        if self.config.log_scrobbles.unwrap_or(false) {
            log_file(format!("[{}] {artist} - {title}", score.ended_at));
        }

        if let Some(last_fm) = self.last_fm.as_ref() {
            match last_fm.scrobble(artist, title, album.as_deref(), score.beatmap.total_length) {
                Ok(_) => log_success("\tLast.fm", "Successfully scrobbled score."),
                Err(error) => log_error("\tLast.fm", error),
            };
        }

        if let Some(listenbrainz) = self.listenbrainz.as_ref() {
            match listenbrainz.scrobble(artist, title, album.as_deref(), score.beatmap.total_length) {
                Ok(_) => log_success("\tListenBrainz", "Successfully scrobbled score."),
                Err(error) => log_error("\tListenBrainz", error),
            };
        }

        self.recent_score = Some(score);
    }
}
