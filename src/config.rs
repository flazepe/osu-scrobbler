use serde::Deserialize;
use std::fs;
use std::process::exit;
use toml;

#[derive(Deserialize)]
pub struct Config {
    pub last_fm_username: String,
    pub last_fm_password: String,
    pub last_fm_api_key: String,
    pub last_fm_api_secret: String,
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
