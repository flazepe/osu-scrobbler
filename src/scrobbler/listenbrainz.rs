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
    pub fn new(user_token: String) -> Result<Self, String> {
        let client = Client::new();
        let response = client
            .get(format!("{}/validate-token", API_BASE_URL))
            .header("authorization", format!("Token {}", user_token))
            .send()
            .map_err(|error| format!("Error validating user token: {}", error))?;

        if response.status().is_success() {
            let token = response
                .json::<ListenBrainzToken>()
                .map_err(|error| format!("Error parsing token response: {}", error))?;
            println!(
                "{} Successfully authenticated with username {}.",
                "[ListenBrainz]".bright_green(),
                token.user_name.bright_blue()
            );
            Ok(Self { client, user_token })
        } else {
            Err(format!("Invalid user token provided."))
        }
    }

    pub fn scrobble(&self, title: &str, artist: &str, total_length: u32) -> Result<(), String> {
        let listen_data = json!({
            "listen_type": "single",
            "payload": [{
                "listened_at": SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map_err(|error| format!("Error getting current time: {}", error))?
                    .as_secs(),
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
        });

        let response = self
            .client
            .post(format!("{}/submit-listens", API_BASE_URL))
            .header("authorization", format!("Token {}", self.user_token))
            .json(&listen_data)
            .send()
            .map_err(|error| format!("Error sending scrobble request: {}", error))?;

        match response.status() {
            StatusCode::OK => Ok(()),
            status_code => Err(format!("Received status code {}.", status_code)),
        }
    }
}
