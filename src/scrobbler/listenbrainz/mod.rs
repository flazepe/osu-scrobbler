mod payloads;

use crate::{
    config::ListenBrainzConfig,
    logger::Logger,
    scrobbler::{REQWEST, Scrobbler},
};
use anyhow::{Result, bail};
use colored::Colorize;
use payloads::{Listen, ListenType, Listens};
use reqwest::StatusCode;
use serde::Deserialize;

const API_BASE_URL: &str = "https://api.listenbrainz.org/1";

#[derive(Debug)]
pub struct ListenBrainzScrobbler<'a> {
    config: &'a ListenBrainzConfig,
}

#[derive(Deserialize)]
struct ListenBrainzToken {
    user_name: String,
}

impl<'a> ListenBrainzScrobbler<'a> {
    pub fn new(config: &'a ListenBrainzConfig) -> Self {
        let user_token = &config.user_token;
        let response = REQWEST
            .get(format!("{API_BASE_URL}/validate-token"))
            .header("authorization", format!("Token {user_token}"))
            .send()
            .and_then(|response| response.json::<ListenBrainzToken>());
        let Ok(token) = response else { Scrobbler::exit("ListenBrainz", "Invalid user token provided.") };
        Logger::success("ListenBrainz", format!("Successfully authenticated with username {}.", token.user_name.bright_blue()));

        Self { config }
    }

    pub fn scrobble(&self, artist: &str, title: &str, album: Option<&str>, total_length: u32) -> Result<()> {
        let user_token = &self.config.user_token;
        let status = REQWEST
            .post(format!("{API_BASE_URL}/submit-listens"))
            .header("authorization", format!("Token {user_token}"))
            .json(&Listens::new(ListenType::Single, vec![Listen::new(artist, title, album, total_length)]))
            .send()?
            .status();

        if status != StatusCode::OK {
            bail!("Received status code {status}.");
        }

        Ok(())
    }
}
