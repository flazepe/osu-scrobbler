mod last_fm;
mod listenbrainz;

use crate::{
    config::{Config, ScrobblerConfig},
    exit,
    logger::{log_error, log_file, log_success, log_warn},
    scores::Score,
    scrobbler::{last_fm::LastfmScrobbler, listenbrainz::ListenBrainzScrobbler},
};
use anyhow::{Result, bail};
use chrono::DateTime;
use colored::Colorize;
use regex::{Regex, escape};
use reqwest::blocking::Client;
use std::{sync::LazyLock, thread::sleep, time::Duration};
use sysinfo::{ProcessRefreshKind, RefreshKind, System};

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
        let config = Config::get();

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

        self.recent_score = Score::get_user_recent(self.config.user_id, &self.config.mode, self.config.scrobble_fails).unwrap_or_default();

        loop {
            self.cooldown_secs = 0;
            self.poll();
            self.cooldown_secs += 5;
            sleep(Duration::from_secs(self.cooldown_secs));
        }
    }

    fn poll(&mut self) {
        if !Self::osu_is_running() {
            return;
        }

        match Score::get_user_recent(self.config.user_id, &self.config.mode, self.config.scrobble_fails) {
            Ok(score) => {
                let Some(score) = score else { return };
                self.scrobble(&score);
                self.recent_score = Some(score);
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

        if let Err(error) = self.validate_scrobble(score) {
            log_warn(
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

        let (new_artist, new_title) = self.handle_redirects(score, artist, title);

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

        log_success(
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
    }

    fn handle_redirects(&self, score: &Score, artist: &str, title: &str) -> (Option<String>, Option<String>) {
        let artists = [score.beatmapset.artist.to_lowercase(), score.beatmapset.artist_unicode.to_lowercase()];

        if let Some((_, new_artist)) = self.config.artist_redirects.iter().find(|(old, _)| artists.contains(&old.to_lowercase())) {
            return (Some(new_artist.clone()), None);
        }

        let mut new_artist = None;

        for (regex, replacer) in &self.config.artist_regex_redirects {
            if regex.is_match(artist) {
                new_artist = Some(regex.replace(artist, replacer).to_string());
                break;
            }
        }

        let mut new_title = None;

        for (regex, replacer) in &self.config.title_regex_redirects {
            if regex.is_match(title) {
                new_title = Some(regex.replace(title, replacer).to_string());
                break;
            }
        }

        (new_artist, new_title)
    }

    fn validate_scrobble(&self, score: &Score) -> Result<()> {
        let boundary_match = |haystack, word| Regex::new(&format!("\\b{}\\b", escape(word))).unwrap().is_match(haystack);
        let (artist_romanized, artist_original) = (score.beatmapset.artist.to_lowercase(), score.beatmapset.artist_unicode.to_lowercase());

        if self.config.blacklist.artists.equals.contains(&artist_romanized)
            || self.config.blacklist.artists.equals.contains(&artist_original)
        {
            bail!("Beatmapset artist is blacklisted.");
        }

        if let Some(word) = self
            .config
            .blacklist
            .artists
            .contains_words
            .iter()
            .find(|word| boundary_match(&artist_romanized, word) || boundary_match(&artist_original, word))
        {
            bail!("Beatmapset artist contains a blacklisted word ({}).", word.bright_red());
        }

        if let Some(regex) = self
            .config
            .blacklist
            .artists
            .matches_regex
            .iter()
            .find(|regex| regex.is_match(&score.beatmapset.artist) || regex.is_match(&score.beatmapset.artist_unicode))
        {
            bail!("Beatmapset artist contains a blacklisted regex ({}).", regex.as_str().bright_red());
        }

        let (title_romanized, title_original) = (score.beatmapset.title.to_lowercase(), score.beatmapset.title_unicode.to_lowercase());

        if self.config.blacklist.titles.equals.contains(&title_romanized) || self.config.blacklist.titles.equals.contains(&title_original) {
            bail!("Beatmapset title is blacklisted.");
        }

        if let Some(word) = self
            .config
            .blacklist
            .titles
            .contains_words
            .iter()
            .find(|word| boundary_match(&title_romanized, word) || boundary_match(&title_original, word))
        {
            bail!("Beatmapset title contains a blacklisted word ({}).", word.bright_red());
        }

        if let Some(regex) = self
            .config
            .blacklist
            .titles
            .matches_regex
            .iter()
            .find(|regex| regex.is_match(&score.beatmapset.title) || regex.is_match(&score.beatmapset.title_unicode))
        {
            bail!("Beatmapset title contains a blacklisted regex ({}).", regex.as_str().bright_red());
        }

        let difficulty = score.beatmap.version.to_lowercase();

        if self.config.blacklist.difficulties.equals.contains(&difficulty) {
            bail!("Beatmap difficulty is blacklisted.");
        }

        if let Some(word) = self.config.blacklist.difficulties.contains_words.iter().find(|word| boundary_match(&difficulty, word)) {
            bail!("Beatmapset difficulty contains a blacklisted word ({}).", word.bright_red());
        }

        if let Some(regex) = self.config.blacklist.difficulties.matches_regex.iter().find(|regex| regex.is_match(&score.beatmap.version)) {
            bail!("Beatmapset difficulty contains a blacklisted regex ({}).", regex.as_str().bright_red());
        }

        if score.beatmap.total_length < self.config.min_beatmap_length_secs {
            bail!(
                "Beatmap's total length ({}) is less than the configured minimum length ({}).",
                format!("{}s", score.beatmap.total_length).bright_blue(),
                format!("{}s", self.config.min_beatmap_length_secs).bright_blue(),
            );
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
                    .and_then(|dt_or_nc_mod| dt_or_nc_mod.settings.as_ref().map(|settings| settings.speed_change.unwrap_or(1.5)))
                    .unwrap_or(1.);
                let hit_length = score.beatmap.hit_length as f64 / rate;

                // A valid scrobble should be half of the beatmap's hit length or 4 minutes, whichever occurs earlier
                // This might go through if the user paused, took a long break, and continued (just to fail some time after)
                let is_valid_scrobble = delta >= hit_length / 2. || delta >= 60. * 4.;

                if !is_valid_scrobble {
                    bail!("Play's progress is less than half of the drain time.");
                }
            }
        }

        Ok(())
    }

    fn osu_is_running() -> bool {
        let system = System::new_with_specifics(RefreshKind::nothing().with_processes(ProcessRefreshKind::everything()));
        system.processes().iter().any(|(_, process)| process.name() == "osu!" || process.name() == "osu!.exe")
    }
}
