use reqwest::blocking::Client;
use serde::Deserialize;

#[derive(Clone)]
pub struct CompactBeatmapset {
    pub artist: String,
    pub artist_unicode: String,
    pub title: String,
    pub title_unicode: String,
    pub total_length: u32,
}

#[derive(Deserialize)]
struct Beatmapset {
    artist: String,
    artist_unicode: String,
    title: String,
    title_unicode: String,
    beatmaps: Vec<Beatmap>,
}

#[derive(Deserialize)]
struct Beatmap {
    version: String,
    total_length: u32,
}

pub fn get_beatmapset(window_title: &str) -> Option<CompactBeatmapset> {
    let beatmapsets = Client::new()
        .get("https://api.nerinyan.moe/search")
        .query(&[("q", window_title)])
        .send()
        .and_then(|response| response.json::<Vec<Beatmapset>>())
        .unwrap_or(vec![]);

    for beatmapset in beatmapsets {
        for beatmap in beatmapset.beatmaps {
            let mut difficulty = beatmap.version;

            // Mania difficulty names are prefixed with [nK] on the mirror
            if difficulty.starts_with("[") && difficulty.contains("K] ") {
                difficulty = difficulty
                    .chars()
                    .skip(if difficulty.starts_with("[10K]") {
                        6
                    } else {
                        5
                    })
                    .collect();
            }

            if format!(
                "{} - {} [{}]",
                beatmapset.artist, beatmapset.title, difficulty
            ) == window_title
                || format!(
                    "{} - {} [{}]",
                    beatmapset.artist_unicode, beatmapset.title_unicode, difficulty
                ) == window_title
            {
                return Some(CompactBeatmapset {
                    artist: beatmapset.artist,
                    artist_unicode: beatmapset.artist_unicode,
                    title: beatmapset.title,
                    title_unicode: beatmapset.title_unicode,
                    total_length: beatmap.total_length,
                });
            }
        }
    }

    None
}
