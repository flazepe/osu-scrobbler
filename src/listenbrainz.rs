use reqwest::{blocking::Client, StatusCode};
use serde::Deserialize;
use serde_json::json;
use std::{
    fmt::{Display, Formatter, Result as FmtResult},
    time::{SystemTime, UNIX_EPOCH},
};

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

    pub fn scrobble(
        &self,
        title: &str,
        artist: &str,
        total_length: u32,
    ) -> Result<(), ScrobblerError> {
        match self
            .client
            .post(format!("{API_BASE_URL}/submit-listens"))
            .header("authorization", format!("Token {}", self.user_token))
            .json(&json!({
                "listen_type": "single",
                "payload": [{
                    "listened_at": SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                    "track_metadata": {
                        "artist_name": artist,
                        "track_name": title,
                        "additional_info": {
                            "media_player": "osu!",
                            "submission_client": "osu!scrobbler (github.com/flazepe/osu-scrobbler)",
                            "submission_client_version": env!("CARGO_PKG_VERSION"),
                            "duration_ms": total_length * 1000,
                        },
                    },
                }],
            }))
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
