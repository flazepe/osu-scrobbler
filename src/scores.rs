use crate::config::{Mode, ScrobblerConfig};
use anyhow::{Context, Result, bail};
use colored::Colorize;
use musicbrainz_rs::{
    Search,
    entity::{
        recording::{Recording, RecordingSearchQuery},
        release_group::ReleaseGroupPrimaryType,
    },
};
use reqwest::{StatusCode, blocking::Client};
use serde::Deserialize;
use std::fmt::Display;

#[derive(Deserialize, Debug)]
pub struct Score {
    pub passed: bool,
    pub mods: Vec<ScoreMods>,
    pub started_at: Option<String>,
    pub ended_at: String,
    pub beatmap: Beatmap,
    pub beatmapset: Beatmapset,
    pub user: User,
}

impl Score {
    pub fn get_user_recent(config: &ScrobblerConfig) -> Result<Option<Self>> {
        let user_id = config.user_id;
        let include_fails = config.scrobble_fails;
        let mut request = Client::new().get(format!("https://osu.ppy.sh/users/{user_id}/scores/recent?include_fails={include_fails}"));

        if config.mode != Mode::Default {
            request = request.query(&[("mode", &config.mode)]);
        }

        let response = request.send().context("Could not send request to get user's recent score.")?;
        let status_code = response.status();

        if status_code != StatusCode::OK {
            let cause = if status_code == StatusCode::NOT_FOUND {
                format!("Invalid user ID: {}", config.user_id.to_string().bright_blue())
            } else {
                format!("Received status code: {}", status_code.as_str().bright_blue())
            };

            bail!("Could not get user's recent score. {cause}");
        }

        let mut scores = response.json::<Vec<Self>>().context("Could not deserialize user's recent scores.")?;

        if scores.is_empty() { Ok(None) } else { Ok(Some(scores.remove(0))) }
    }

    pub fn get_album_name<T: Display, U: Display>(&self, artist: T, title: U) -> Option<String> {
        let mut title_album = None;
        let mut title_ep = None;
        let mut title_single = None;
        let mut title_other = None;
        let mut title_unrecognized = None;
        let mut title_first = None;

        for recording in self.get_musicbrainz_recordings(artist, title) {
            let Some(releases) = recording.releases else { continue };

            for release in releases {
                if title_first.is_none() {
                    title_first = Some(release.title.clone());
                }

                let release_group_primary_type = release
                    .release_group
                    .as_ref()
                    .and_then(|release_group| release_group.primary_type.as_ref())
                    .unwrap_or(&ReleaseGroupPrimaryType::UnrecognizedReleaseGroupPrimaryType);

                let release_group_secondary_types_is_empty =
                    release.release_group.as_ref().map(|release_group| release_group.secondary_types.is_empty()).unwrap_or_default();

                let option = match release_group_primary_type {
                    ReleaseGroupPrimaryType::Album => &mut title_album,
                    ReleaseGroupPrimaryType::Ep => &mut title_ep,
                    ReleaseGroupPrimaryType::Single => &mut title_single,
                    ReleaseGroupPrimaryType::Other => &mut title_other,
                    _ => &mut title_unrecognized,
                };

                if option.is_none() && release_group_secondary_types_is_empty {
                    _ = option.insert(release.title);
                }
            }
        }

        title_album.or(title_ep).or(title_single).or(title_other).or(title_unrecognized).or(title_first)
    }

    fn get_musicbrainz_recordings<T: Display, U: Display>(&self, artist: T, title: U) -> Vec<Recording> {
        let artist = artist.to_string();
        let title = title.to_string();

        let mut query_artist = RecordingSearchQuery::query_builder();
        query_artist
            .artist(&artist)
            .or()
            .artist(&self.beatmapset.artist)
            .or()
            .artist(&self.beatmapset.artist_unicode)
            .or()
            .artist_name(&artist)
            .or()
            .artist_name(&self.beatmapset.artist)
            .or()
            .artist_name(&self.beatmapset.artist_unicode);

        let mut query_title = RecordingSearchQuery::query_builder();
        query_title
            .recording(&title)
            .or()
            .recording(&self.beatmapset.title)
            .or()
            .recording(&self.beatmapset.title_unicode)
            .or()
            .recording_accent(&title)
            .or()
            .recording_accent(&self.beatmapset.title)
            .or()
            .recording_accent(&self.beatmapset.title_unicode)
            .or()
            .alias(&title)
            .or()
            .alias(&self.beatmapset.title)
            .or()
            .alias(&self.beatmapset.title_unicode);

        let query = RecordingSearchQuery::query_builder().expr(&mut query_artist).and().expr(&mut query_title).build();
        let results = Recording::search(query).limit(100).execute();

        results.map(|results| results.entities).unwrap_or_default()
    }
}

#[derive(Deserialize, Debug)]
pub struct ScoreMods {
    pub acronym: String,
    pub settings: Option<ScoreModSettings>,
}

#[derive(Deserialize, Debug)]
pub struct ScoreModSettings {
    pub speed_change: Option<f64>,
}

#[derive(Deserialize, Debug)]
pub struct Beatmap {
    pub version: String,
    pub total_length: u32,
    pub hit_length: u32,
}

#[derive(Deserialize, Debug)]
pub struct Beatmapset {
    pub artist: String,
    pub artist_unicode: String,
    pub title: String,
    pub title_unicode: String,
}

#[derive(Deserialize, Debug)]
pub struct User {
    pub id: u32,
    pub username: String,
}
