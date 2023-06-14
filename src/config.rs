use serde::{Deserialize, Serialize};
use std::fs::read_to_string;
use toml::from_str;

#[derive(Deserialize)]
pub struct Config {
    pub scrobbler: ScrobblerConfig,
    pub last_fm: Option<LastfmConfig>,
    pub listenbrainz: Option<ListenBrainzConfig>,
}

#[derive(Deserialize)]
pub struct ScrobblerConfig {
    pub user_id: u64,
    pub mode: Option<Mode>,
    pub use_original_metadata: bool,
    pub min_beatmap_length_secs: u32,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    Osu,
    Taiko,
    Fruits,
    Mania,
}

#[derive(Deserialize)]
pub struct LastfmConfig {
    pub username: String,
    pub password: String,
    pub api_key: String,
    pub api_secret: String,
}

#[derive(Deserialize)]
pub struct ListenBrainzConfig {
    pub user_token: String,
}

pub fn get_config() -> Config {
    match from_str(&match read_to_string("config.toml") {
        Ok(contents) => contents,
        Err(_) => panic!("No config file found!"),
    }) {
        Ok(config) => config,
        Err(error) => panic!("An error occurred while reading config: {error}"),
    }
}
