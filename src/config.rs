use crate::{logger::Logger, utils::exit};
use anyhow::{Context, Result};
use colored::Colorize;
use regex::{Regex, RegexBuilder};
use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{SeqAccess, Visitor},
    ser::SerializeSeq,
};
use serde_json::to_string;
use serde_regex::Serde as SerdeRegex;
use std::{
    env::var,
    fmt::{Debug, Display, Formatter, Result as FmtResult},
    fs::{canonicalize, read_to_string},
    path::PathBuf,
};
use toml::from_str;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub scrobbler: ScrobblerConfig,
    pub last_fm: Option<LastfmConfig>,
    pub listenbrainz: Option<ListenBrainzConfig>,
}

impl Config {
    fn get_canonicalized_path() -> Result<PathBuf> {
        let env_config_path = var("OSU_SCROBBLER_CONFIG_PATH");
        let config_path = env_config_path.as_deref().unwrap_or("config.toml");
        canonicalize(config_path).context("Could not resolve the path to config file.")
    }

    fn read(path: &PathBuf) -> Result<Self> {
        let config_string = read_to_string(path).context("An error occurred while trying to read config file.")?;
        from_str(&config_string).context("An error occurred while parsing config file.")
    }

    pub fn init() -> Self {
        let config_path = Config::get_canonicalized_path().unwrap_or_else(|error| exit("Config", format!("{error:?}")));
        let config = Config::read(&config_path).unwrap_or_else(|error| exit("Config", format!("{error:?}")));
        Logger::success("Config", format!("Successfully loaded from {}: {config:#?}", config_path.to_string_lossy().bright_blue()));
        config
    }
}

#[derive(Deserialize, Debug)]
pub struct ScrobblerConfig {
    pub user_id: u64,

    #[serde(default)]
    pub mode: Mode,

    #[serde(default = "ScrobblerConfig::use_original_metadata_default")]
    pub use_original_metadata: bool,

    #[serde(default = "ScrobblerConfig::min_beatmap_length_secs_default")]
    pub min_beatmap_length_secs: u32,

    #[serde(default)]
    pub scrobble_fails: bool,

    #[serde(default)]
    pub log_scrobbles: bool,

    #[serde(default)]
    pub redirects: ScrobblerRedirectsConfig,

    #[serde(default)]
    pub blacklist: ScrobblerBlacklistConfig,
}

impl ScrobblerConfig {
    pub fn sync(&mut self) {
        let Ok(config_path) = Config::get_canonicalized_path() else { return };
        let Ok(new_config) = Config::read(&config_path) else { return };

        if to_string(&new_config.scrobbler.redirects).unwrap_or_default() != to_string(&self.redirects).unwrap_or_default() {
            self.redirects = new_config.scrobbler.redirects;
            Logger::success("Config", format!("Successfully reloaded redirects from {}.", config_path.to_string_lossy().bright_blue()));
        }

        if to_string(&new_config.scrobbler.blacklist).unwrap_or_default() != to_string(&self.blacklist).unwrap_or_default() {
            self.blacklist = new_config.scrobbler.blacklist;
            Logger::success("Config", format!("Successfully reloaded blacklist from {}.", config_path.to_string_lossy().bright_blue()));
        }
    }

    fn use_original_metadata_default() -> bool {
        true
    }

    fn min_beatmap_length_secs_default() -> u32 {
        60
    }
}

#[derive(Deserialize, Serialize, Default, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    #[default]
    Default,

    Osu,
    Taiko,
    Fruits,
    Mania,
}

#[derive(Deserialize, Serialize, Default, Debug)]
pub struct ScrobblerRedirectsConfig {
    #[serde(default)]
    pub artists: ScrobblerRedirectsTypeConfig,

    #[serde(default)]
    pub titles: ScrobblerRedirectsTypeConfig,
}

#[derive(Deserialize, Serialize, Default, Debug)]
pub struct ScrobblerRedirectsTypeConfig {
    #[serde(deserialize_with = "deserialize_case_insensitive_redirects_vec", default)]
    pub equal_matches: Vec<(String, String)>,

    #[serde(deserialize_with = "deserialize_regex_redirects_vec", serialize_with = "serialize_regex_redirects_vec", default)]
    pub regex_matches: Vec<(Regex, String)>,
}

#[derive(Deserialize, Serialize, Default, Debug)]
pub struct ScrobblerBlacklistConfig {
    pub artists: ScrobblerBlacklistTypeConfig,
    pub titles: ScrobblerBlacklistTypeConfig,
    pub difficulties: ScrobblerBlacklistTypeConfig,
}

#[derive(Deserialize, Serialize, Default, Debug)]
pub struct ScrobblerBlacklistTypeConfig {
    #[serde(deserialize_with = "deserialize_case_insensitive_vec", default)]
    pub equal_matches: Vec<String>,

    #[serde(deserialize_with = "deserialize_regex_vec", serialize_with = "serde_regex::serialize", default)]
    pub regex_matches: Vec<Regex>,
}

#[derive(Deserialize, Debug)]
pub struct LastfmConfig {
    pub username: String,
    pub password: SensitiveString,
    pub api_key: SensitiveString,
    pub api_secret: SensitiveString,
}

#[derive(Deserialize, Debug)]
pub struct ListenBrainzConfig {
    pub user_token: SensitiveString,
}

#[derive(Deserialize)]
pub struct SensitiveString(String);

impl Display for SensitiveString {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.0)
    }
}

impl Debug for SensitiveString {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "<redacted>")
    }
}

fn deserialize_case_insensitive_redirects_vec<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<(String, String)>, D::Error> {
    struct RegexCaseInsensitiveRedirectsVecVisitor;

    impl<'de> Visitor<'de> for RegexCaseInsensitiveRedirectsVecVisitor {
        type Value = Vec<(String, String)>;

        fn expecting(&self, formatter: &mut Formatter) -> FmtResult {
            formatter.write_str("an array of tuples containing a regex pattern and replacer string")
        }

        fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
            let mut vec = Vec::with_capacity(seq.size_hint().unwrap_or_default());

            while let Some((old, new)) = seq.next_element::<(String, String)>()? {
                vec.push((old.to_lowercase(), new));
            }

            Ok(vec)
        }
    }

    deserializer.deserialize_seq(RegexCaseInsensitiveRedirectsVecVisitor)
}

fn deserialize_case_insensitive_vec<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<String>, D::Error> {
    struct CaseInsensitiveVecVisitor;

    impl<'de> Visitor<'de> for CaseInsensitiveVecVisitor {
        type Value = Vec<String>;

        fn expecting(&self, formatter: &mut Formatter) -> FmtResult {
            formatter.write_str("an array of strings")
        }

        fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
            let mut vec = Vec::with_capacity(seq.size_hint().unwrap_or_default());

            while let Some(element) = seq.next_element::<String>()? {
                vec.push(element.to_lowercase());
            }

            Ok(vec)
        }
    }

    deserializer.deserialize_seq(CaseInsensitiveVecVisitor)
}

fn deserialize_regex_redirects_vec<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<(Regex, String)>, D::Error> {
    struct RegexRedirectsVecVisitor;

    impl<'de> Visitor<'de> for RegexRedirectsVecVisitor {
        type Value = Vec<(Regex, String)>;

        fn expecting(&self, formatter: &mut Formatter) -> FmtResult {
            formatter.write_str("an array of tuples containing a regex pattern and replacer string")
        }

        fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
            let mut vec = Vec::with_capacity(seq.size_hint().unwrap_or_default());

            while let Some((SerdeRegex(regex), regex_replacer_string)) = seq.next_element::<(SerdeRegex<Regex>, String)>()? {
                let case_insensitive_regex = RegexBuilder::new(regex.as_str()).case_insensitive(true).build().unwrap();
                vec.push((case_insensitive_regex, regex_replacer_string));
            }

            Ok(vec)
        }
    }

    deserializer.deserialize_seq(RegexRedirectsVecVisitor)
}

fn serialize_regex_redirects_vec<S: Serializer>(value: &[(Regex, String)], serializer: S) -> Result<S::Ok, S::Error> {
    let mut seq = serializer.serialize_seq(Some(value.len()))?;

    for (regex, replacer) in value {
        seq.serialize_element(&(regex.as_str(), replacer))?;
    }

    seq.end()
}

fn deserialize_regex_vec<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<Regex>, D::Error> {
    struct RegexVecVisitor;

    impl<'de> Visitor<'de> for RegexVecVisitor {
        type Value = Vec<Regex>;

        fn expecting(&self, formatter: &mut Formatter) -> FmtResult {
            formatter.write_str("an array of regex pattern strings")
        }

        fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
            let mut vec = Vec::with_capacity(seq.size_hint().unwrap_or_default());

            while let Some(SerdeRegex(regex)) = seq.next_element::<SerdeRegex<Regex>>()? {
                let case_insensitive_regex = RegexBuilder::new(regex.as_str()).case_insensitive(true).build().unwrap();
                vec.push(case_insensitive_regex);
            }

            Ok(vec)
        }
    }

    deserializer.deserialize_seq(RegexVecVisitor)
}
