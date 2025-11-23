use crate::{logger::Logger, utils::exit};
use anyhow::Context;
use colored::Colorize;
use regex::{Regex, RegexBuilder};
use serde::{
    Deserialize, Deserializer, Serialize,
    de::{SeqAccess, Visitor},
};
use std::{
    env::var,
    fmt::{Debug, Display, Formatter, Result as FmtResult},
    fs::{canonicalize, read_to_string},
};
use toml::from_str;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub scrobbler: ScrobblerConfig,
    pub last_fm: Option<LastfmConfig>,
    pub listenbrainz: Option<ListenBrainzConfig>,
}

impl Config {
    pub fn get() -> Self {
        let env_config_path = var("OSU_SCROBBLER_CONFIG_PATH");
        let config_path = canonicalize(env_config_path.as_deref().unwrap_or("config.toml"))
            .context("Could not resolve the path to config file.")
            .unwrap_or_else(|error| exit("Config", format!("{error:?}")));

        let config_string = read_to_string(&config_path)
            .context("An error occurred while trying to read config file.")
            .unwrap_or_else(|error| exit("Config", format!("{error:?}")));
        let config = from_str(&config_string)
            .context("An error occurred while parsing config file.")
            .unwrap_or_else(|error| exit("Config", format!("{error:?}")));

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

#[derive(Deserialize, Default, Debug)]
pub struct ScrobblerRedirectsConfig {
    #[serde(default)]
    pub artists: ScrobblerRedirectsTypeConfig,

    #[serde(default)]
    pub titles: ScrobblerRedirectsTypeConfig,
}

#[derive(Deserialize, Default, Debug)]
pub struct ScrobblerRedirectsTypeConfig {
    #[serde(deserialize_with = "deserialize_case_insensitive_redirects_vec", default)]
    pub equal_matches: Vec<(String, String)>,

    #[serde(deserialize_with = "deserialize_regex_redirects_vec", default)]
    pub regex_matches: Vec<(Regex, String)>,
}

#[derive(Deserialize, Default, Debug)]
pub struct ScrobblerBlacklistConfig {
    pub artists: ScrobblerBlacklistTypeConfig,
    pub titles: ScrobblerBlacklistTypeConfig,
    pub difficulties: ScrobblerBlacklistTypeConfig,
}

#[derive(Deserialize, Default, Debug)]
pub struct ScrobblerBlacklistTypeConfig {
    #[serde(deserialize_with = "deserialize_case_insensitive_vec", default)]
    pub equal_matches: Vec<String>,

    #[serde(deserialize_with = "deserialize_regex_vec", default)]
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

            while let Some((regex_pattern_string, regex_replacer_string)) = seq.next_element::<(String, String)>()? {
                let regex = RegexBuilder::new(&regex_pattern_string)
                    .case_insensitive(true)
                    .build()
                    .context(format!("Invalid regex pattern: {}", regex_pattern_string.bright_red()))
                    .unwrap_or_else(|error| exit("Config", format!("{error:?}")));

                vec.push((regex, regex_replacer_string));
            }

            Ok(vec)
        }
    }

    deserializer.deserialize_seq(RegexRedirectsVecVisitor)
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

            while let Some(regex_pattern_string) = seq.next_element::<String>()? {
                let regex = RegexBuilder::new(&regex_pattern_string)
                    .case_insensitive(true)
                    .build()
                    .context(format!("Invalid regex pattern: {}", regex_pattern_string.bright_red()))
                    .unwrap_or_else(|error| exit("Config", format!("{error:?}")));

                vec.push(regex);
            }

            Ok(vec)
        }
    }

    deserializer.deserialize_seq(RegexVecVisitor)
}
