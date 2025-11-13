use chrono::Utc;
use serde::Serialize;
use std::fmt::Display;

#[derive(Serialize)]
pub struct Listens {
    pub listen_type: ListenType,
    pub payload: Vec<Listen>,
}

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ListenType {
    Single,
}

impl Listens {
    pub fn new(listen_type: ListenType, listens: Vec<Listen>) -> Self {
        Self { listen_type, payload: listens }
    }
}

#[derive(Serialize)]
pub struct Listen {
    pub listened_at: i64,
    pub track_metadata: TrackMetadata,
}

impl Listen {
    pub fn new<T: Display, U: Display, V: Display>(artist_name: T, track_name: U, release_name: Option<V>, duration: u32) -> Self {
        Self {
            listened_at: Utc::now().timestamp(),
            track_metadata: TrackMetadata::new(
                artist_name,
                track_name,
                release_name,
                TrackAdditionalInfo::new(
                    "osu!",
                    "osu-scrobbler (github.com/flazepe/osu-scrobbler)",
                    env!("CARGO_PKG_VERSION"),
                    duration * 1000,
                ),
            ),
        }
    }
}

#[derive(Serialize)]
pub struct TrackMetadata {
    pub artist_name: String,
    pub track_name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_name: Option<String>,

    pub additional_info: TrackAdditionalInfo,
}

impl TrackMetadata {
    pub fn new<T: Display, U: Display, V: Display>(
        artist_name: T,
        track_name: U,
        release_name: Option<V>,
        additional_info: TrackAdditionalInfo,
    ) -> Self {
        Self {
            artist_name: artist_name.to_string(),
            track_name: track_name.to_string(),
            release_name: release_name.map(|release_name| release_name.to_string()),
            additional_info,
        }
    }
}

#[derive(Serialize)]
pub struct TrackAdditionalInfo {
    pub media_player: String,
    pub submission_client: String,
    pub submission_client_version: String,
    pub duration_ms: u32,
}

impl TrackAdditionalInfo {
    pub fn new<T: Display, U: Display, V: Display>(
        media_player: T,
        submission_client: U,
        submission_client_version: V,
        duration_ms: u32,
    ) -> Self {
        Self {
            media_player: media_player.to_string(),
            submission_client: submission_client.to_string(),
            submission_client_version: submission_client_version.to_string(),
            duration_ms,
        }
    }
}
