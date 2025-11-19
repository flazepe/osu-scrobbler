use crate::exit;
use serde::{
    Deserialize, Deserializer, Serialize,
    de::{SeqAccess, Visitor},
};
use std::{
    env::var,
    fmt::{Formatter, Result as FmtResult},
    fs::read_to_string,
};
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

    #[serde(default)]
    pub blacklist: ScrobblerBlacklistConfig,
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

#[derive(Deserialize, Default, Debug)]
pub struct ScrobblerBlacklistConfig {
    pub artists: ScrobblerBlacklistTypeConfig,
    pub titles: ScrobblerBlacklistTypeConfig,
    pub difficulties: ScrobblerBlacklistTypeConfig,
}

#[derive(Deserialize, Default, Debug)]
pub struct ScrobblerBlacklistTypeConfig {
    #[serde(deserialize_with = "deserialize_case_insensitive_vec")]
    pub equals: Vec<String>,

    #[serde(deserialize_with = "deserialize_case_insensitive_vec")]
    pub contains_words: Vec<String>,
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

fn deserialize_case_insensitive_vec<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    struct CaseInsensitiveVecVisitor;

    impl<'de> Visitor<'de> for CaseInsensitiveVecVisitor {
        type Value = Vec<String>;

        fn expecting(&self, formatter: &mut Formatter) -> FmtResult {
            formatter.write_str("an array of strings")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let mut vec = Vec::with_capacity(seq.size_hint().unwrap_or_default());

            while let Some(element) = seq.next_element::<String>()? {
                vec.push(element.to_lowercase());
            }

            Ok(vec)
        }
    }

    deserializer.deserialize_seq(CaseInsensitiveVecVisitor)
}
