use crate::config::Mode;
use anyhow::{Result, bail};
use musicbrainz_rs::{
    Search,
    entity::recording::{Recording, RecordingSearchQuery},
};
use reqwest::{StatusCode, blocking::Client};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Score {
    pub ended_at: String,
    pub beatmap: Beatmap,
    pub beatmapset: Beatmapset,
}

impl Score {
    pub fn get_user_recent(user_id: u64, mode: &Option<Mode>) -> Result<Option<Self>> {
        let mut request = Client::new().get(format!("https://osu.ppy.sh/users/{user_id}/scores/recent"));

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

    pub fn get_musicbrainz_recording(&self) -> Option<Recording> {
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
        let results = Recording::search(query).with_aliases().execute();

        results.ok().and_then(|results| results.entities.into_iter().next())
    }
}

#[derive(Deserialize)]
pub struct Beatmap {
    pub total_length: u32,
}

#[derive(Deserialize)]
pub struct Beatmapset {
    pub artist: String,
    pub artist_unicode: String,
    pub title: String,
    pub title_unicode: String,
}
