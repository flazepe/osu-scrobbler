use crate::scrobbler::ScrobblerError;
use md5::compute;
use reqwest::{blocking::Client, StatusCode};
use serde::Deserialize;
use std::time::{SystemTime, UNIX_EPOCH};

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
    pub fn new(username: &str, password: &str, api_key: &str, api_secret: &str) -> Self {
        let client = Client::new();

        let format = "json";
        let method = "auth.getMobileSession";

        let session_key = match client
            .post(API_BASE_URL)
            .header("content-length", "0")
            .query(&[
                ("format", format),
                ("api_key", api_key),
                ("method", method),
                ("password", password),
                ("username", username),
                (
                    "api_sig",
                    format!(
                        "{:x}",
                        compute(format!(
                            "api_key{api_key}method{method}password{password}username{username}{api_secret}",
                        )),
                    )
                    .as_str(),
                ),
            ])
            .send()
            .unwrap()
            .json::<LastfmSession>()
        {
            Ok(session) => {
                println!("Authenticated with Last.fm (username: {}).", session.session.name);
                session.session.key
            },
            Err(_) => panic!("An error occurred while authenticating to Last.fm: Invalid credentials provided"),
        };

        Self {
            client,
            api_key: api_key.to_string(),
            api_secret: api_secret.to_string(),
            session_key,
        }
    }

    pub fn scrobble(
        &self,
        title: &str,
        artist: &str,
        total_length: u32,
    ) -> Result<(), ScrobblerError> {
        let method = "track.scrobble";

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        match self.client
            .post(API_BASE_URL)
            .header("content-length", "0")
            .query(&[
                ("api_key", self.api_key.as_str()),
                ("artist[0]", artist),
                ("duration[0]", &total_length.to_string()),
                ("method", method),
                ("sk", &self.session_key),
                ("timestamp[0]", timestamp.to_string().as_str()),
                ("track[0]", title),
                (
                    "api_sig",
                    format!(
                        "{:x}",
                        compute(format!(
                            "api_key{}artist[0]{artist}duration[0]{total_length}method{method}sk{}timestamp[0]{timestamp}track[0]{title}{}",
                            self.api_key,
                            self.session_key,
                            self.api_secret,
                        )),
                    )
                    .as_str(),
                ),
            ])
            .send()
        {
            Ok(response) => match response.status() {
                StatusCode::OK => Ok(()),
                status_code => Err(ScrobblerError {
                    message: format!("Scrobble request failed: Received status code {status_code}"),
                }),
            },
            Err(error) => Err(ScrobblerError {
                message: error.to_string(),
            }),
        }
    }
}
