use colored::Colorize;
use md5::compute;
use reqwest::{blocking::Client, StatusCode};
use serde::Deserialize;
use std::{
    collections::BTreeMap,
    time::{SystemTime, UNIX_EPOCH},
};

const API_BASE_URL: &str = "https://ws.audioscrobbler.com/2.0/";

pub struct LastfmScrobbler {
    client: Client,
    api_key: String,
    api_secret: String,
    session_key: String,
}

#[derive(Deserialize)]
struct LastfmSession {
    session: LastfmSessionData,
}

#[derive(Deserialize)]
struct LastfmSessionData {
    name: String,
    key: String,
}

impl LastfmScrobbler {
    /// create a new LastfmScrobbler instance.
    pub fn new(username: String, password: String, api_key: String, api_secret: String) -> Result<Self, String> {
        let client = Client::new();

        let session_key = client
            .post(API_BASE_URL)
            .header("content-length", "0")
            .query(
                &LastfmQuery::new()
                    .insert("api_key", &api_key)
                    .insert("method", "auth.getMobileSession")
                    .insert("password", password)
                    .insert("username", username)
                    .sign(&api_secret),
            )
            .send()
            .map_err(|err| format!("Failed to authenticate with Last.fm: {}", err))?
            .json::<LastfmSession>()
            .map_err(|err| format!("Failed to parse Last.fm session: {}", err))
            .map(|session| {
                println!(
                    "{} Successfully authenticated with username {}.",
                    "[Last.fm]".bright_green(),
                    session.session.name.bright_blue()
                );
                session.session.key
            })?;

        Ok(Self { client, api_key, api_secret, session_key })
    }

    /// scrobble a track to Last.fm.
    pub fn scrobble(&self, title: &str, artist: &str, total_length: u32) -> Result<(), String> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|err| format!("Failed to get timestamp: {}", err))?
            .as_secs();

        let response = self
            .client
            .post(API_BASE_URL)
            .header("content-length", "0")
            .query(
                &LastfmQuery::new()
                    .insert("api_key", &self.api_key)
                    .insert("artist[0]", artist)
                    .insert("duration[0]", total_length)
                    .insert("method", "track.scrobble")
                    .insert("sk", &self.session_key)
                    .insert("timestamp[0]", timestamp)
                    .insert("track[0]", title)
                    .sign(&self.api_secret),
            )
            .send()
            .map_err(|err| format!("Failed to send scrobble request: {}", err))?;

        match response.status() {
            StatusCode::OK => Ok(()),
            status_code => Err(format!("Received status code {}.", status_code)),
        }
    }
}

struct LastfmQuery {
    query: BTreeMap<String, String>,
}

impl LastfmQuery {
    /// create a new LastfmQuery instance.
    pub fn new() -> Self {
        Self { query: BTreeMap::new() }
    }

    /// insert a key-value pair into the query parameters.
    pub fn insert<T: ToString, U: ToString>(mut self, key: T, value: U) -> Self {
        self.query.insert(key.to_string(), value.to_string());
        self
    }

    /// sign the query parameters using the API secret.
    pub fn sign<T: ToString>(self, api_secret: T) -> BTreeMap<String, String> {
        let api_sig = format!("{:x}", compute(
            self.query
                .iter()
                .map(|(key, value)| format!("{}{}", key, value))
                .collect::<String>()
                + &api_secret.to_string(),
        ));

        self.insert("api_sig", api_sig).insert("format", "json").query
    }
}
