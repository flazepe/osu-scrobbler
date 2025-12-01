mod last_fm;
mod listenbrainz;

use crate::{
    config::{Config, ScrobblerConfig},
    logger::Logger,
    scores::Score,
    scrobbler::{last_fm::LastfmScrobbler, listenbrainz::ListenBrainzScrobbler},
    utils::{exit, get_osu_pid, handle_redirects, validate_scrobble},
};
use anyhow::{Context, Result};
use chrono::DateTime;
use colored::Colorize;
use reqwest::blocking::Client;
use std::{
    sync::LazyLock,
    thread::sleep,
    time::{Duration, SystemTime},
};

static REQWEST: LazyLock<Client> = LazyLock::new(Client::new);

#[derive(Debug)]
pub struct Scrobbler {
    config: ScrobblerConfig,
    config_modified: SystemTime,
    config_reload_result: Result<()>,
    last_fm: Option<LastfmScrobbler>,
    listenbrainz: Option<ListenBrainzScrobbler>,
    recent_score: Option<Score>,
    cooldown_secs: u64,
}

impl Scrobbler {
    pub fn new() -> Self {
        let (config, config_modified) = Config::init();

        Self {
            config: config.scrobbler,
            config_modified,
            config_reload_result: Ok(()),
            last_fm: config.last_fm.map(LastfmScrobbler::new),
            listenbrainz: config.listenbrainz.map(ListenBrainzScrobbler::new),
            recent_score: None,
            cooldown_secs: 0,
        }
    }

    pub fn start(&mut self) {
        self.recent_score = Score::get_user_recent(&self.config).unwrap_or_else(|error| exit("Scrobbler", format!("{error:?}")));

        Logger::success("Scrobbler", "Started!", false);

        loop {
            self.cooldown_secs = 0;

            self.reload_config();

            if get_osu_pid().is_some() {
                self.poll();
            }

            self.cooldown_secs += 5;

            sleep(Duration::from_secs(self.cooldown_secs));
        }
    }

    fn reload_config(&mut self) {
        match self.config.reload(&mut self.config_modified).context("Could not reload config file.") {
            Ok(new_recent_score) => {
                if let Some(new_recent_score) = new_recent_score {
                    self.recent_score = Some(new_recent_score);
                }

                self.config_reload_result = Ok(());
            },
            Err(new_error) => {
                let new_error_string = format!("{new_error:?}");

                if self.config_reload_result.as_ref().err().is_none_or(|error| format!("{error:?}") != new_error_string) {
                    Logger::error("Config", new_error_string, false);
                }

                self.config_reload_result = Err(new_error);
            },
        }
    }

    fn poll(&mut self) {
        match Score::get_user_recent(&self.config) {
            Ok(score) => {
                let Some(score) = score else { return };
                self.scrobble(&score);
                self.recent_score = Some(score);
            },
            Err(error) => {
                Logger::error("Scrobbler", error, false);
                self.cooldown_secs += 10;
            },
        }
    }

    fn scrobble(&mut self, score: &Score) {
        if self.recent_score.as_ref().is_some_and(|recent_score| recent_score.ended_at == score.ended_at) {
            return;
        }

        if !score.passed {
            let started_at = score.started_at.as_ref().and_then(|started_at| DateTime::parse_from_rfc3339(started_at).ok());
            let ended_at = DateTime::parse_from_rfc3339(&score.ended_at).ok();
            let delta =
                ended_at.and_then(|ended_at| started_at.map(|started_at| (ended_at - started_at).as_seconds_f64())).unwrap_or_default();

            if delta > 0. {
                let rate = score
                    .mods
                    .iter()
                    .find(|score_mod| score_mod.acronym == "DT" || score_mod.acronym == "NC")
                    .and_then(|score_mod| score_mod.settings.as_ref().map(|settings| settings.speed_change.unwrap_or(1.5)))
                    .unwrap_or(1.);
                let hit_length = score.beatmap.hit_length as f64 / rate;

                // A valid scrobble should be half of the beatmap's hit length or 4 minutes, whichever occurs earlier
                // This might go through if the user paused, took a long break, and continued (just to fail some time after)
                let is_valid_scrobble = delta >= hit_length / 2. || delta >= 60. * 4.;

                if !is_valid_scrobble {
                    return;
                }
            }
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
                false,
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

        let album = if self.config.fetch_album_names { score.get_album_name(artist, title) } else { None };

        Logger::success(
            "Scrobbler",
            format!(
                "New score by {} ({}) found: {}{} - {}{} ({})",
                score.user.username.bright_blue(),
                score.user.id.to_string().bright_blue(),
                artist.bright_blue(),
                artist_redirected_text.as_deref().unwrap_or_default(),
                title.bright_blue(),
                title_redirected_text.as_deref().unwrap_or_default(),
                album.as_deref().unwrap_or("Unknown Album").bright_blue(),
            ),
            false,
        );

        if self.config.log_scrobbles {
            Logger::file(format!("{} | {artist} - {title}", score.ended_at));
        }

        if let Some(last_fm) = self.last_fm.as_ref() {
            match last_fm.scrobble(artist, title, album.as_deref(), score.beatmap.total_length) {
                Ok(_) => Logger::success("Last.fm", "Successfully scrobbled score.", true),
                Err(error) => Logger::error("Last.fm", error, true),
            };
        }

        if let Some(listenbrainz) = self.listenbrainz.as_ref() {
            match listenbrainz.scrobble(artist, title, album.as_deref(), score.beatmap.total_length) {
                Ok(_) => Logger::success("ListenBrainz", "Successfully scrobbled score.", true),
                Err(error) => Logger::error("ListenBrainz", error, true),
            };
        }
    }
}
