mod payloads;

use crate::{exit, logger::log_success};
use anyhow::{bail, Result};
use colored::Colorize;
use payloads::{Listen, Listens};
use reqwest::{blocking::Client, StatusCode};
use serde::Deserialize;

const API_BASE_URL: &str = "https://api.listenbrainz.org/1";

pub struct ListenBrainzScrobbler {
    pub client: Client,
    pub user_token: String,
}

#[derive(Deserialize)]
struct ListenBrainzToken {
    user_name: String,
}

impl ListenBrainzScrobbler {
    pub fn new(user_token: String) -> Self {
        let user_token = format!("Token {user_token}");

        let client = Client::new();

        let response = client
            .get(format!("{API_BASE_URL}/validate-token"))
            .header("authorization", &user_token)
            .send()
            .and_then(|response| response.json::<ListenBrainzToken>());

        let Ok(token) = response else { exit!("ListenBrainz", "Invalid user token provided.") };
        log_success("ListenBrainz", format!("Successfully authenticated with username {}.", token.user_name.bright_blue()));

        Self { client, user_token }
    }

    pub fn scrobble(&self, artist: &str, title: &str, album: Option<&str>, total_length: u32) -> Result<()> {
        let status = self
            .client
            .post(format!("{API_BASE_URL}/submit-listens"))
            .header("authorization", &self.user_token)
            .json(&Listens::new("single", vec![Listen::new(artist, title, album, total_length)]))
            .send()?
            .status();

        if status != StatusCode::OK {
            bail!("Received status code {status}.");
        }

        Ok(())
    }
}
