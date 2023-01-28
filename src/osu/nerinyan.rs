use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use urlencoding::encode;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Beatmapset {
    pub artist: String,
    pub artist_unicode: String,
    pub title: String,
    pub title_unicode: String,
    pub length: u64,
}

pub fn get_beatmapset(window_title: &str) -> Option<Beatmapset> {
    if let Ok(json) = Client::new()
        .get(format!(
            "https://api.nerinyan.moe/search?q={}",
            encode(window_title)
        ))
        .send()
        .unwrap()
        .json::<Vec<Value>>()
    {
        for beatmapset in json {
            for beatmap in beatmapset["beatmaps"].as_array().unwrap() {
                let title = beatmapset["title"].as_str().unwrap();
                let title_unicode = beatmapset["title_unicode"].as_str().unwrap();

                let artist = beatmapset["artist"].as_str().unwrap();
                let artist_unicode = beatmapset["artist_unicode"].as_str().unwrap();

                let difficulty = beatmap["version"].as_str().unwrap();

                if format!("{} - {} [{}]", artist, title, difficulty) == window_title
                    || format!("{} - {} [{}]", artist_unicode, title_unicode, difficulty)
                        == window_title
                {
                    return Some(Beatmapset {
                        artist: artist_unicode.to_owned(),
                        artist_unicode: artist_unicode.to_owned(),
                        title: title.to_owned(),
                        title_unicode: title_unicode.to_owned(),
                        length: beatmap["total_length"].as_u64().unwrap(),
                    });
                }
            }
        }
    }

    None
}
