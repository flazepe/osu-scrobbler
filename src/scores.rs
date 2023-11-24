use crate::{config::Mode, logger::log_error};
use reqwest::{blocking::Client, StatusCode};
use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub struct Score {
    pub ended_at: String,
    pub beatmap: Beatmap,
    pub beatmapset: Beatmapset,
}

#[derive(Clone, Deserialize)]
pub struct Beatmap {
    pub total_length: u32,
}

#[derive(Clone, Deserialize)]
pub struct Beatmapset {
    pub artist: String,
    pub artist_unicode: String,
    pub title: String,
    pub title_unicode: String,
}

pub fn get_recent_score(user_id: u64, mode: &Option<Mode>) -> Option<Score> {
    let mut request = Client::new().get(format!("https://osu.ppy.sh/users/{user_id}/scores/recent"));

    if let Some(mode) = mode {
        request = request.query(&[("mode", mode)]);
    }

    let response = match request.send() {
        Ok(response) => response,
        Err(error) => {
            log_error("Scores", format!("Could not send request to get user's recent score: {error}"));
            return None;
        },
    };

    let status_code = response.status();

    if status_code != StatusCode::OK {
        match status_code {
            StatusCode::NOT_FOUND => {
                log_error("Scores", "Invalid osu! user ID given.");
                panic!();
            },
            _ => {
                log_error("Scores", format!("Could not get user's recent score: Received status code {status_code}."));
                return None;
            },
        }
    }

    let Ok(mut scores) = response.json::<Vec<Score>>() else { return None };

    match scores.is_empty() {
        true => None,
        false => Some(scores.remove(0)),
    }
}
