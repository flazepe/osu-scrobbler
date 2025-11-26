use md5::compute;
use std::{collections::BTreeMap, fmt::Display};

pub struct LastfmQuery<'a>(BTreeMap<&'a str, String>);

impl<'a> LastfmQuery<'a> {
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }

    pub fn insert<T: Display>(mut self, key: &'a str, value: T) -> Self {
        self.0.insert(key, value.to_string());
        self
    }

    pub fn sign<T: Display>(self, api_secret: T) -> BTreeMap<&'a str, String> {
        let api_sig = format!(
            "{:x}",
            compute(self.0.iter().fold("".into(), |acc, (key, value)| format!("{acc}{key}{value}")) + &api_secret.to_string()),
        );
        self.insert("api_sig", api_sig).insert("format", "json").0
    }
}
