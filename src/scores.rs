use crate::config::Mode;
use anyhow::{Result, bail};
use reqwest::{StatusCode, blocking::Client};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Score {
    pub ended_at: String,
    pub beatmap: Beatmap,
    pub beatmapset: Beatmapset,
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

pub fn get_recent_score(user_id: u64, mode: &Option<Mode>) -> Result<Option<Score>> {
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

    let Ok(mut scores) = response.json::<Vec<Score>>() else { return Ok(None) };

    if scores.is_empty() { Ok(None) } else { Ok(Some(scores.remove(0))) }
}
