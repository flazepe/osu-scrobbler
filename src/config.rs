use serde::Deserialize;
use std::fs::read_to_string;
use toml::from_str;

#[derive(Deserialize)]
pub struct Config {
    pub scrobble: ScrobbleConfig,
    pub last_fm: LastfmConfig,
}

#[derive(Deserialize)]
pub struct ScrobbleConfig {
    pub use_original_metadata: bool,
    pub min_beatmap_length_secs: u32,
}

#[derive(Deserialize)]
pub struct LastfmConfig {
    pub username: String,
    pub password: String,
    pub api_key: String,
    pub api_secret: String,
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
