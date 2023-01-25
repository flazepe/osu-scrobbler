use crate::osu::window::OsuWindowDetails;
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

pub fn get_beatmapset(details: &OsuWindowDetails) -> Option<Beatmapset> {
    if let Ok(json) = Client::new()
        .get(
            "https://api.nerinyan.moe/search?q=".to_string()
                + &encode((details.artist.to_owned() + " - " + &details.title).as_str()),
        )
        .send()
        .unwrap()
        .json::<Vec<Value>>()
    {
        for beatmapset in json {
            if (beatmapset["artist"].as_str().unwrap() == details.artist.as_str()
                || beatmapset["artist_unicode"].as_str().unwrap() == details.artist.as_str())
                && (beatmapset["title"].as_str().unwrap() == details.title.as_str()
                    || beatmapset["title_unicode"].as_str().unwrap() == details.title.as_str())
            {
                let mut target_beatmap: Option<Value> = None;

                for beatmap in beatmapset["beatmaps"].as_array().unwrap() {
                    if beatmap["version"].as_str().unwrap() == details.beatmap.as_str() {
                        target_beatmap = Some(beatmap.to_owned());
                        break;
                    }
                }

                if target_beatmap.is_some() {
                    return Some(Beatmapset {
                        artist: beatmapset["artist"].as_str().unwrap().to_owned(),
                        artist_unicode: beatmapset["artist_unicode"].as_str().unwrap().to_owned(),
                        title: beatmapset["title"].as_str().unwrap().to_owned(),
                        title_unicode: beatmapset["title_unicode"].as_str().unwrap().to_owned(),
                        length: target_beatmap.unwrap()["total_length"].as_u64().unwrap(),
                    });
                }
            }
        }
    }

    None
}
