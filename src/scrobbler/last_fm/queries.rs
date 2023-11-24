use md5::compute;
use std::collections::BTreeMap;

pub struct LastfmQuery(BTreeMap<String, String>);

impl LastfmQuery {
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }

    pub fn insert<T: ToString, U: ToString>(mut self, key: T, value: U) -> Self {
        self.0.insert(key.to_string(), value.to_string());
        self
    }

    pub fn sign<T: ToString>(self, api_secret: T) -> BTreeMap<String, String> {
        let api_sig = format!(
            "{:x}",
            compute(self.0.iter().map(|(key, value)| format!("{key}{value}")).collect::<String>() + &api_secret.to_string()),
        );

        self.insert("api_sig", api_sig).insert("format", "json").0
    }
}
