use crate::OsuScrobbler;
use reqwest::{blocking::Client, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fmt::{Display, Formatter, Result as FmtResult};

const API_BASE_URL: &str = "https://api.listenbrainz.org/1";

pub struct ListenBrainzScrobbler {
    client: Client,
    user_token: String,
}

#[derive(Deserialize)]
struct ValidToken {
    user_name: String,
}

pub struct ScrobblerError {
    message: String,
}

impl Display for ScrobblerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.message)
    }
}

impl ListenBrainzScrobbler {
    pub fn new(user_token: &str) -> Self {
        let client = Client::new();

        match client
            .get(format!("{API_BASE_URL}/validate-token"))
            .header("authorization", format!("Token {user_token}"))
            .send()
            .unwrap()
            .json::<ValidToken>() {
				Ok(token) => println!("Authenticated with ListenBrainz (username: {}).", token.user_name),
				Err(_) => panic!("An error occurred while authenticating to ListenBrainz: Invalid user token provided"),
			};

        Self {
            client,
            user_token: user_token.to_string(),
        }
    }

    fn _scrobble<T: Serialize>(&self, endpoint: &str, json: T) -> Result<(), ScrobblerError> {
        match self
            .client
            .post(format!("{API_BASE_URL}/{endpoint}"))
            .header("authorization", format!("Token {}", self.user_token))
            .json(&json)
            .send()
        {
            Ok(response) => match response.status() {
                StatusCode::OK => Ok(()),
                status_code @ _ => Err(ScrobblerError {
                    message: format!("Scrobble request failed: Received status code {status_code}"),
                }),
            },
            Err(error) => Err(ScrobblerError {
                message: error.to_string(),
            }),
        }
    }

    pub fn set_now_playing(&self, title: &str, artist: &str) -> Result<(), ScrobblerError> {
        self._scrobble(
            "now-playing",
            json!({
                "listen_type": "playing_now",
                "payload": [{
                    "track_metadata": {
                        "artist_name": artist,
                        "track_name": title,
                    },
                }],
            }),
        )
    }

    pub fn scrobble(&self, title: &str, artist: &str) -> Result<(), ScrobblerError> {
        self._scrobble(
            "submit-listens",
            json!({
                "listen_type": "single",
                "payload": [{
                    "listened_at": OsuScrobbler::get_current_timestamp(),
                    "track_metadata": {
                        "artist_name": artist,
                        "track_name": title,
                    },
                }],
            }),
        )
    }
}
