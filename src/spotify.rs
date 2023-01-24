use serde::{Deserialize, Serialize};
use serde_json::Value;
use urlencoding::encode;

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct SpotifyToken {
    client_id: String,
    access_token: String,
    access_token_expiration_timestamp_ms: usize,
    is_anonymous: bool,
}

#[derive(Deserialize, Serialize)]
pub struct SpotifyTrack {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub length: usize,
}

// (String, usize)
async fn get_token() -> String {
    let json: SpotifyToken = reqwest::Client::new()
        .get("https://open.spotify.com/get_access_token?reason=transport&productType=web_player")
        .header("user-agent", "hello world")
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    json.access_token // (json.access_token, json.access_token_expiration_timestamp_ms)
}

pub async fn get_track(query: &str) -> Option<SpotifyTrack> {
    let token = get_token().await;

    let json: Value = reqwest::Client::new()
        .get("https://api-partner.spotify.com/pathfinder/v1/query?operationName=searchTracks&variables=%7B%22searchTerm%22%3A%22".to_string() + &encode(query) + "%22%2C%22offset%22%3A0%2C%22limit%22%3A100%2C%22numberOfTopResults%22%3A20%2C%22includeAudiobooks%22%3Afalse%7D&extensions=%7B%22persistedQuery%22%3A%7B%22version%22%3A1%2C%22sha256Hash%22%3A%221d021289df50166c61630e02f002ec91182b518e56bcd681ac6b0640390c0245%22%7D%7D")
        .header("user-agent", "hello world")
        .header("authorization", "Bearer ".to_owned() + &token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    let track = &json["data"]["searchV2"]["tracksV2"]["items"][0]["item"]["data"];

    if track == &serde_json::Value::Null {
        return None;
    }

    Some(SpotifyTrack {
        title: track["name"].as_str().unwrap().to_string(),
        artist: track["artists"]["items"][0]["profile"]["name"]
            .as_str()
            .unwrap()
            .to_string(),
        album: track["albumOfTrack"]["name"].as_str().unwrap().to_string(),
        length: track["duration"]["totalMilliseconds"]
            .to_string()
            .parse::<usize>()
            .unwrap(),
    })
}
