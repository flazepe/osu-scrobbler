use reqwest::blocking::Client;
use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub struct OsuScore {
    pub ended_at: String,
    pub beatmap: OsuBeatmap,
    pub beatmapset: OsuBeatmapset,
}

#[derive(Clone, Deserialize)]
pub struct OsuBeatmap {
    pub total_length: u32,
}

#[derive(Clone, Deserialize)]
pub struct OsuBeatmapset {
    pub artist: String,
    pub artist_unicode: String,
    pub title: String,
    pub title_unicode: String,
}

pub fn get_last_score(id: u64, mode: Option<String>) -> Option<OsuScore> {
    let mode = mode.unwrap_or("".into());

    match Client::new()
        .get(format!(
            "https://osu.ppy.sh/users/{id}/scores/recent{}",
            match ["osu", "taiko", "fruits", "mania"].contains(&mode.as_str()) {
                true => format!("?mode={mode}"),
                false => "".into(),
            },
        ))
        .send()
        .and_then(|response| response.json::<Vec<OsuScore>>())
    {
        Ok(mut scores) => match scores.is_empty() {
            true => None,
            false => Some(scores.remove(0)),
        },
        Err(_) => panic!("Invalid osu! user ID given."),
    }
}
