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
    pub fn new(username: String, password: String, api_key: String, api_secret: String) -> Self {
        let client = Client::new();

        let session_key = match client
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
            .unwrap()
            .json::<LastfmSession>()
        {
            Ok(session) => {
                println!("{} Successfully authenticated with username {}.", "[Last.fm]".bright_green(), session.session.name.bright_blue());
                session.session.key
            },
            Err(_) => panic!("{} Invalid credentials provided.", "[Last.fm]".bright_red()),
        };

        Self { client, api_key, api_secret, session_key }
    }

    pub fn scrobble(&self, title: &str, artist: &str, total_length: u32) -> Result<(), String> {
        match self
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
                    .insert("timestamp[0]", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs())
                    .insert("track[0]", title)
                    .sign(&self.api_secret),
            )
            .send()
        {
            Ok(response) => match response.status() {
                StatusCode::OK => Ok(()),
                status_code => Err(format!("Received status code {status_code}.")),
            },
            Err(error) => Err(error.to_string()),
        }
    }
}

struct LastfmQuery {
    query: BTreeMap<String, String>,
}

impl LastfmQuery {
    pub fn new() -> Self {
        Self { query: BTreeMap::new() }
    }

    pub fn insert<T: ToString, U: ToString>(mut self, key: T, value: U) -> Self {
        self.query.insert(key.to_string(), value.to_string());
        self
    }

    pub fn sign<T: ToString>(self, api_secret: T) -> BTreeMap<String, String> {
        let api_sig = format!(
            "{:x}",
            compute(self.query.iter().map(|(key, value)| format!("{key}{value}")).collect::<String>() + &api_secret.to_string()),
        );

        self.insert("api_sig", api_sig).insert("format", "json").query
    }
}
