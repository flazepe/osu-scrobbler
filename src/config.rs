use crate::logger::log_error;
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
    pub use_original_metadata: Option<bool>,
    pub min_beatmap_length_secs: Option<u32>,
    pub log_scrobbles: Option<bool>,
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
    from_str(&read_to_string("config.toml").unwrap_or_else(|_| {
        log_error("Config", "No config file found.");
        panic!();
    }))
    .unwrap_or_else(|error| {
        log_error("Config", format!("Error parsing config file: {error}"));
        panic!();
    })
}
