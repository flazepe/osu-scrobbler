use reqwest::blocking::Client;
use serde_json::Value;
use urlencoding::encode;

#[derive(Clone)]
pub struct Beatmapset {
    pub artist: String,
    pub artist_unicode: String,
    pub title: String,
    pub title_unicode: String,
    pub length: u32,
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

                let mut difficulty = beatmap["version"].as_str().unwrap().to_owned();

                // Mania difficulty names are prefixed with [nK] on the mirror
                if difficulty.starts_with("[") && difficulty.contains("K] ") {
                    difficulty = difficulty
                        .to_owned()
                        .chars()
                        .skip(if difficulty.starts_with("[10K]") {
                            6
                        } else {
                            5
                        })
                        .collect();
                }

                if format!("{} - {} [{}]", artist, title, difficulty) == window_title
                    || format!("{} - {} [{}]", artist_unicode, title_unicode, difficulty)
                        == window_title
                {
                    return Some(Beatmapset {
                        artist: artist_unicode.to_owned(),
                        artist_unicode: artist_unicode.to_owned(),
                        title: title.to_owned(),
                        title_unicode: title_unicode.to_owned(),
                        length: beatmap["total_length"].to_string().parse::<u32>().unwrap(),
                    });
                }
            }
        }
    }

    None
}
