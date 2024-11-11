mod payloads;

use anyhow::{bail, Result};
use payloads::{Listen, Listens};
use reqwest::{blocking::Client, StatusCode};
use serde::Deserialize;

const API_BASE_URL: &str = "https://api.listenbrainz.org/1";

pub struct ListenBrainzScrobbler {
    pub client: Client,
    pub user_token: String,
    pub username: String,
}

#[derive(Deserialize)]
struct ListenBrainzToken {
    #[serde(rename = "user_name")]
    username: String,
}

impl ListenBrainzScrobbler {
    pub fn new(user_token: String) -> Result<Self> {
        let user_token = format!("Token {user_token}");
        let client = Client::new();
        let username = client
            .get(format!("{API_BASE_URL}/validate-token"))
            .header("authorization", &user_token)
            .send()?
            .json::<ListenBrainzToken>()?
            .username;

        Ok(Self { client, user_token, username })
    }

    pub fn scrobble(&self, artist: &str, title: &str, total_length: u32) -> Result<()> {
        match self
            .client
            .post(format!("{API_BASE_URL}/submit-listens"))
            .header("authorization", &self.user_token)
            .json(&Listens::new("single", vec![Listen::new(artist, title, total_length)]))
            .send()?
            .status()
        {
            StatusCode::OK => Ok(()),
            status_code => bail!("Received status code {status_code}."),
        }
    }
}
