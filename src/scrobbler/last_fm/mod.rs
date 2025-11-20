mod queries;

use crate::{config::LastfmConfig, logger::Logger, scrobbler::REQWEST, utils::exit};
use anyhow::{Result, bail};
use chrono::Utc;
use colored::Colorize;
use queries::LastfmQuery;
use reqwest::StatusCode;
use serde::Deserialize;

const API_BASE_URL: &str = "https://ws.audioscrobbler.com/2.0/";

#[derive(Debug)]
pub struct LastfmScrobbler<'a> {
    config: &'a LastfmConfig,
    session_key: String,
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

impl<'a> LastfmScrobbler<'a> {
    pub fn new(config: &'a LastfmConfig) -> Self {
        let response = REQWEST
            .post(API_BASE_URL)
            .header("content-length", "0")
            .query(
                &LastfmQuery::new()
                    .insert("api_key", &config.api_key)
                    .insert("method", "auth.getMobileSession")
                    .insert("username", &config.username)
                    .insert("password", &config.password)
                    .sign(&config.api_secret),
            )
            .send()
            .and_then(|response| response.json::<LastfmSession>());

        let Ok(session) = response else { exit("Last.fm", "Invalid credentials provided.") };
        Logger::success("Last.fm", format!("Successfully authenticated with username {}.", session.session.name.bright_blue()));

        Self { config, session_key: session.session.key }
    }

    pub fn scrobble(&self, artist: &str, title: &str, album: Option<&str>, total_length: u32) -> Result<()> {
        let status = REQWEST
            .post(API_BASE_URL)
            .header("content-length", "0")
            .query(
                &LastfmQuery::new()
                    .insert("api_key", &self.config.api_key)
                    .insert("sk", &self.session_key)
                    .insert("method", "track.scrobble")
                    .insert("artist[0]", artist)
                    .insert("track[0]", title)
                    .insert("album[0]", album.unwrap_or_default())
                    .insert("duration[0]", total_length)
                    .insert("timestamp[0]", Utc::now().timestamp())
                    .sign(&self.config.api_secret),
            )
            .send()?
            .status();

        if status != StatusCode::OK {
            bail!("Received status code {status}.");
        }

        Ok(())
    }
}
