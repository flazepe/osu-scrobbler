mod queries;

use anyhow::{bail, Result};
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
    pub username: String,
}

#[derive(Deserialize)]
struct LastfmSession {
    session: LastfmSessionData,
}

#[derive(Deserialize)]
struct LastfmSessionData {
    key: String,

    #[serde(rename = "name")]
    username: String,
}

impl LastfmScrobbler {
    pub fn new(username: String, password: String, api_key: String, api_secret: String) -> Result<Self> {
        let client = Client::new();
        let session = client
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
            .send()?
            .json::<LastfmSession>()?
            .session;

        Ok(Self { client, api_key, api_secret, session_key: session.key, username: session.username })
    }

    pub fn scrobble(&self, title: &str, artist: &str, total_length: u32) -> Result<()> {
        match self
            .client
            .post(API_BASE_URL)
            .header("content-length", "0")
            .query(
                &LastfmQuery::new()
                    .insert("api_key", &self.api_key)
                    .insert("sk", &self.session_key)
                    .insert("method", "track.scrobble")
                    .insert("track[0]", title)
                    .insert("artist[0]", artist)
                    .insert("duration[0]", total_length)
                    .insert("timestamp[0]", SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs())
                    .sign(&self.api_secret),
            )
            .send()?
            .status()
        {
            StatusCode::OK => Ok(()),
            status_code => bail!("Received status code {status_code}."),
        }
    }
}
