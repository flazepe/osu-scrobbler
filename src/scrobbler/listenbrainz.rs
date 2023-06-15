use colored::Colorize;
use reqwest::{blocking::Client, StatusCode};
use serde::Deserialize;
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};

const API_BASE_URL: &str = "https://api.listenbrainz.org/1";

pub struct ListenBrainzScrobbler {
    client: Client,
    user_token: String,
}

#[derive(Deserialize)]
struct ListenBrainzToken {
    user_name: String,
}

impl ListenBrainzScrobbler {
    pub fn new(user_token: &str) -> Self {
        let client = Client::new();

        match client
            .get(format!("{API_BASE_URL}/validate-token"))
            .header("authorization", format!("Token {user_token}"))
            .send()
            .unwrap()
            .json::<ListenBrainzToken>()
        {
            Ok(token) => {
                println!("{} Successfully authenticated with username {}.", "[ListenBrainz]".bright_green(), token.user_name.bright_blue())
            },
            Err(_) => panic!("{} Invalid user token provided.", "[ListenBrainz]".bright_red()),
        };

        Self { client, user_token: user_token.to_string() }
    }

    pub fn scrobble(&self, title: &str, artist: &str, total_length: u32) -> Result<(), String> {
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
                            "submission_client": "osu-scrobbler (github.com/flazepe/osu-scrobbler)",
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
                status_code => Err(format!("Received status code {status_code}")),
            },
            Err(error) => Err(error.to_string()),
        }
    }
}
