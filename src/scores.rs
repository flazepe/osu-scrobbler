use crate::config::Mode;
use anyhow::{Result, bail};
use musicbrainz_rs::{
    Search,
    entity::{
        recording::{Recording, RecordingSearchQuery},
        release_group::ReleaseGroupPrimaryType,
    },
};
use reqwest::{StatusCode, blocking::Client};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Score {
    pub passed: bool,
    pub started_at: String,
    pub ended_at: String,
    pub beatmap: Beatmap,
    pub beatmapset: Beatmapset,
}

impl Score {
    pub fn get_user_recent(user_id: u64, mode: &Option<Mode>, include_fails: bool) -> Result<Option<Self>> {
        let mut request = Client::new().get(format!("https://osu.ppy.sh/users/{user_id}/scores/recent?include_fails={include_fails}"));

        if let Some(mode) = mode {
            request = request.query(&[("mode", mode)]);
        }

        let response = match request.send() {
            Ok(response) => response,
            Err(error) => bail!("Could not send request to get user's recent score: {error}"),
        };

        let status_code = response.status();

        if status_code != StatusCode::OK {
            bail!("Could not get user's recent score. Received status code: {status_code}");
        }

        let Ok(mut scores) = response.json::<Vec<Self>>() else { return Ok(None) };

        if scores.is_empty() { Ok(None) } else { Ok(Some(scores.remove(0))) }
    }

    pub fn get_musicbrainz_recordings(&self) -> Vec<Recording> {
        let mut query_artist = RecordingSearchQuery::query_builder();
        query_artist
            .artist(&self.beatmapset.artist)
            .or()
            .artist(&self.beatmapset.artist_unicode)
            .or()
            .artist_name(&self.beatmapset.artist)
            .or()
            .artist_name(&self.beatmapset.artist_unicode);

        let mut query_title = RecordingSearchQuery::query_builder();
        query_title
            .recording(&self.beatmapset.title)
            .or()
            .recording(&self.beatmapset.title_unicode)
            .or()
            .recording_accent(&self.beatmapset.title)
            .or()
            .recording_accent(&self.beatmapset.title_unicode)
            .or()
            .alias(&self.beatmapset.title)
            .or()
            .alias(&self.beatmapset.title_unicode);

        let query = RecordingSearchQuery::query_builder().expr(&mut query_artist).and().expr(&mut query_title).build();
        let results = Recording::search(query).limit(100).execute();

        results.map(|results| results.entities).unwrap_or_default()
    }

    pub fn get_album_name(&self) -> Option<String> {
        let find_by_group_primary_type = |primary_type: Option<ReleaseGroupPrimaryType>| {
            for recording in self.get_musicbrainz_recordings() {
                let Some(releases) = recording.releases else { continue };

                for release in releases {
                    let Some(primary_type) = &primary_type else { return Some(release.title) };
                    let Some(release_group) = release.release_group else { continue };

                    if release_group.primary_type == Some(primary_type.clone()) && release_group.secondary_types.is_empty() {
                        return Some(release.title);
                    }
                }
            }

            None
        };

        find_by_group_primary_type(Some(ReleaseGroupPrimaryType::Album))
            .or_else(|| find_by_group_primary_type(Some(ReleaseGroupPrimaryType::Ep)))
            .or_else(|| find_by_group_primary_type(Some(ReleaseGroupPrimaryType::Single)))
            .or_else(|| find_by_group_primary_type(Some(ReleaseGroupPrimaryType::Other)))
            .or_else(|| find_by_group_primary_type(Some(ReleaseGroupPrimaryType::UnrecognizedReleaseGroupPrimaryType)))
            .or_else(|| find_by_group_primary_type(None))
    }
}

#[derive(Deserialize)]
pub struct Beatmap {
    pub total_length: u32,
    pub hit_length: u32,
}

#[derive(Deserialize)]
pub struct Beatmapset {
    pub artist: String,
    pub artist_unicode: String,
    pub title: String,
    pub title_unicode: String,
}
