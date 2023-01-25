use serde::Deserialize;
use std::fs;
use std::process::exit;
use toml;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub options: OptionsConfig,
    pub last_fm: LastfmConfig,
}

#[derive(Debug, Deserialize)]
pub struct OptionsConfig {
    pub use_original_metadata: bool,
    pub min_beatmap_length_seconds: u64,
}

#[derive(Debug, Deserialize)]
pub struct LastfmConfig {
    pub username: String,
    pub password: String,
    pub api_key: String,
    pub api_secret: String,
}

pub fn get_config() -> Config {
    let contents = match fs::read_to_string("config.toml") {
        Ok(c) => c,
        Err(_) => {
            eprintln!("No config file found!");
            exit(1);
        }
    };

    let config: Config = match toml::from_str(&contents) {
        Ok(c) => c,
        Err(err) => {
            eprintln!("An error occurred while reading config: {:?}", err);
            exit(1);
        }
    };

    config
}
