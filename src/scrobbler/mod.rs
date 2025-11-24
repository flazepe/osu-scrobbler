mod last_fm;
mod listenbrainz;

use crate::{
    config::{Config, ScrobblerConfig},
    logger::Logger,
    scores::Score,
    scrobbler::{last_fm::LastfmScrobbler, listenbrainz::ListenBrainzScrobbler},
    utils::{exit, get_osu_pid, handle_redirects, validate_scrobble},
};
use colored::Colorize;
use reqwest::blocking::Client;
use std::{sync::LazyLock, thread::sleep, time::Duration};

static REQWEST: LazyLock<Client> = LazyLock::new(Client::new);

#[derive(Debug)]
pub struct Scrobbler {
    config: ScrobblerConfig,
    last_fm: Option<LastfmScrobbler>,
    listenbrainz: Option<ListenBrainzScrobbler>,
    recent_score: Option<Score>,
    cooldown_secs: u64,
}

impl Scrobbler {
    pub fn new() -> Self {
        let config_path = Config::get_canonicalized_path().unwrap_or_else(|error| exit("Config", format!("{error:?}")));
        let config = Config::read(&config_path).unwrap_or_else(|error| exit("Config", format!("{error:?}")));
        Logger::success("Config", format!("Successfully loaded from {}: {config:#?}", config_path.to_string_lossy().bright_blue()));

        if config.last_fm.is_none() && config.listenbrainz.is_none() {
            exit("Scrobbler", "Please provide configuration for either Last.fm or ListenBrainz.");
        }

        Self {
            config: config.scrobbler,
            last_fm: config.last_fm.map(LastfmScrobbler::new),
            listenbrainz: config.listenbrainz.map(ListenBrainzScrobbler::new),
            recent_score: None,
            cooldown_secs: 0,
        }
    }

    pub fn start(&mut self) {
        Logger::success("Scrobbler", "Started!");

        self.recent_score = Score::get_user_recent(&self.config).unwrap_or_default();

        loop {
            self.cooldown_secs = 0;
            self.poll();
            self.cooldown_secs += 5;
            sleep(Duration::from_secs(self.cooldown_secs));
        }
    }

    fn poll(&mut self) {
        self.config.sync();

        if get_osu_pid().is_none() {
            return;
        }

        match Score::get_user_recent(&self.config) {
            Ok(score) => {
                let Some(score) = score else { return };
                self.scrobble(&score);
                self.recent_score = Some(score);
            },
            Err(error) => {
                if error.to_string().contains("404") {
                    exit("Scrobbler", "Invalid osu! user ID given.");
                }
                Logger::error("Scrobbler", error);
                self.cooldown_secs += 10;
            },
        }
    }

    fn scrobble(&mut self, score: &Score) {
        if self.recent_score.as_ref().is_some_and(|recent_score| recent_score.ended_at == score.ended_at) {
            return;
        }

        let (artist_romanized, artist_original) = (&score.beatmapset.artist, &score.beatmapset.artist_unicode);
        let (title_romanized, title_original) = (&score.beatmapset.title, &score.beatmapset.title_unicode);

        let mut artist = artist_romanized;
        let mut title = title_romanized;

        if self.config.use_original_metadata {
            artist = artist_original;
            title = title_original;
        }

        if let Err(error) = validate_scrobble(score, &self.config) {
            Logger::warn(
                "Scrobbler",
                format!(
                    "Skipping score {} - {} [{}]: {error}",
                    artist.bright_blue(),
                    title.bright_blue(),
                    score.beatmap.version.bright_blue(),
                ),
            );

            return;
        }

        let (new_artist, new_title) = handle_redirects(score, artist, title, &self.config);

        let artist_redirected_text = new_artist.as_ref().map(|new_artist| {
            let old_artist = artist;
            artist = new_artist;
            format!(" (redirected from {})", old_artist.bright_blue())
        });

        let title_redirected_text = new_title.as_ref().map(|new_title| {
            let old_title = title;
            title = new_title;
            format!(" (redirected from {})", old_title.bright_blue())
        });

        let album = score.get_album_name();

        Logger::success(
            "Scrobbler",
            format!(
                "New score found: {}{} - {}{} ({})",
                artist.bright_blue(),
                artist_redirected_text.as_deref().unwrap_or_default(),
                title.bright_blue(),
                title_redirected_text.as_deref().unwrap_or_default(),
                album.as_deref().unwrap_or("Unknown Album").bright_blue(),
            ),
        );

        if self.config.log_scrobbles {
            Logger::file(format!("[{}] {artist} - {title}", score.ended_at));
        }

        if let Some(last_fm) = self.last_fm.as_ref() {
            match last_fm.scrobble(artist, title, album.as_deref(), score.beatmap.total_length) {
                Ok(_) => Logger::success("\tLast.fm", "Successfully scrobbled score."),
                Err(error) => Logger::error("\tLast.fm", error),
            };
        }

        if let Some(listenbrainz) = self.listenbrainz.as_ref() {
            match listenbrainz.scrobble(artist, title, album.as_deref(), score.beatmap.total_length) {
                Ok(_) => Logger::success("\tListenBrainz", "Successfully scrobbled score."),
                Err(error) => Logger::error("\tListenBrainz", error),
            };
        }
    }
}
