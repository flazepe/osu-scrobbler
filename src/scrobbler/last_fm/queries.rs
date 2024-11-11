use md5::compute;
use std::{collections::BTreeMap, fmt::Display};

pub struct LastfmQuery(BTreeMap<String, String>);

impl LastfmQuery {
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }

    pub fn insert<T: Display, U: Display>(mut self, key: T, value: U) -> Self {
        self.0.insert(key.to_string(), value.to_string());
        self
    }

    pub fn sign<T: Display>(self, api_secret: T) -> BTreeMap<String, String> {
        let api_sig = format!(
            "{:x}",
            compute(self.0.iter().fold("".into(), |acc, (key, value)| format!("{acc}{key}{value}")) + &api_secret.to_string()),
        );
        self.insert("api_sig", api_sig).insert("format", "json").0
    }
}
