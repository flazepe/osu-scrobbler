use crate::exit;
use serde::{Deserialize, Serialize};
use std::{env::var, fs::read_to_string};
use toml::from_str;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub scrobbler: ScrobblerConfig,
    pub last_fm: Option<LastfmConfig>,
    pub listenbrainz: Option<ListenBrainzConfig>,
}

#[derive(Deserialize, Debug)]
pub struct ScrobblerConfig {
    pub user_id: u64,
    pub mode: Option<Mode>,

    #[serde(default = "ScrobblerConfig::use_original_metadata_default")]
    pub use_original_metadata: bool,

    #[serde(default = "ScrobblerConfig::min_beatmap_length_secs_default")]
    pub min_beatmap_length_secs: u32,

    #[serde(default)]
    pub scrobble_fails: bool,

    #[serde(default)]
    pub log_scrobbles: bool,

    #[serde(default)]
    pub artist_redirects: Vec<(String, String)>,
}

impl ScrobblerConfig {
    fn use_original_metadata_default() -> bool {
        true
    }

    fn min_beatmap_length_secs_default() -> u32 {
        60
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    Osu,
    Taiko,
    Fruits,
    Mania,
}

#[derive(Deserialize, Debug)]
pub struct LastfmConfig {
    pub username: String,
    pub password: String,
    pub api_key: String,
    pub api_secret: String,
}

#[derive(Deserialize, Debug)]
pub struct ListenBrainzConfig {
    pub user_token: String,
}

impl Config {
    pub fn get() -> Self {
        let env_config_path = var("OSU_SCROBBLER_CONFIG_PATH");
        let config_path = env_config_path.as_deref().unwrap_or("config.toml");
        let config_string = read_to_string(config_path).unwrap_or_else(|_| exit!("Config", "No config file found."));

        from_str(&config_string).unwrap_or_else(|error| exit!("Config", format!("Error parsing config file: {error}")))
    }
}
