mod queries;

use crate::{exit, logger::log_success};
use anyhow::{bail, Result};
use colored::Colorize;
use queries::LastfmQuery;
use reqwest::{blocking::Client, StatusCode};
use serde::Deserialize;
use std::time::{SystemTime, UNIX_EPOCH};

const API_BASE_URL: &str = "https://ws.audioscrobbler.com/2.0/";

pub struct LastfmScrobbler {
    pub client: Client,
    pub api_key: String,
    pub api_secret: String,
    pub session_key: String,
}

#[derive(Deserialize)]
struct LastfmSession {
    session: LastfmSessionData,
}

#[derive(Deserialize)]
struct LastfmSessionData {
    key: String,
    name: String,
}

impl LastfmScrobbler {
    pub fn new(username: String, password: String, api_key: String, api_secret: String) -> Self {
        let client = Client::new();

        let response = client
            .post(API_BASE_URL)
            .header("content-length", "0")
            .query(
                &LastfmQuery::new()
                    .insert("api_key", &api_key)
                    .insert("method", "auth.getMobileSession")
                    .insert("username", username)
                    .insert("password", password)
                    .sign(&api_secret),
            )
            .send()
            .and_then(|response| response.json::<LastfmSession>());

        let Ok(session) = response else { exit!("Last.fm", "Invalid credentials provided.") };
        log_success("Last.fm", format!("Successfully authenticated with username {}.", session.session.name.bright_blue()));

        Self { client, api_key, api_secret, session_key: session.session.key }
    }

    pub fn scrobble(&self, artist: &str, title: &str, album: Option<&str>, total_length: u32) -> Result<()> {
        let status = self
            .client
            .post(API_BASE_URL)
            .header("content-length", "0")
            .query(
                &LastfmQuery::new()
                    .insert("api_key", &self.api_key)
                    .insert("sk", &self.session_key)
                    .insert("method", "track.scrobble")
                    .insert("artist[0]", artist)
                    .insert("track[0]", title)
                    .insert("album[0]", album.unwrap_or(""))
                    .insert("duration[0]", total_length)
                    .insert("timestamp[0]", SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs())
                    .sign(&self.api_secret),
            )
            .send()?
            .status();

        if status != StatusCode::OK {
            bail!("Received status code {status}.");
        }

        Ok(())
    }
}
