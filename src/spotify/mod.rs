mod payloads;

use crate::scores::Beatmapset;
use anyhow::{Context, Result, bail};
use payloads::{SpotifyAccessToken, SpotifyArtist, SpotifyData, SpotifySearchResult, SpotifyTrack};
use reqwest::blocking::Client;
use serde_json::json;
use std::{
    sync::LazyLock,
    time::{SystemTime, UNIX_EPOCH},
};
use totp_rs::{Algorithm, Secret, TOTP};

static SPOTIFY_TOTP: LazyLock<TOTP> = LazyLock::new(|| {
    let secret = generate_totp_secret([12, 56, 76, 33, 88, 44, 88, 33, 78, 78, 11, 66, 22, 22, 55, 69, 54]).unwrap();
    TOTP::new(Algorithm::SHA1, 6, 1, 30, secret).unwrap()
});

fn generate_totp_secret(secret: [usize; 17]) -> Result<Vec<u8>> {
    let transformed = secret.iter().enumerate().fold(String::new(), |acc, (index, entry)| acc + &(entry ^ ((index % 33) + 9)).to_string());
    Ok(Secret::Raw(transformed.as_bytes().to_vec()).to_bytes()?)
}

pub struct Spotify {
    client: Client,
    access_token: Option<SpotifyAccessToken>,
}

impl Spotify {
    pub fn new() -> Self {
        Self { client: Client::new(), access_token: None }
    }

    fn ensure_access_token(&mut self) -> Result<()> {
        if let Some(access_token) = &self.access_token {
            if SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() < access_token.access_token_expiration_timestamp_ms {
                return Ok(());
            }
        }

        let access_token = self
            .client
            .get("https://open.spotify.com/get_access_token")
            .query(&[("productType", "web-player"), ("totp", &SPOTIFY_TOTP.generate_current()?), ("totpVer", "5")])
            .header("user-agent", "yes")
            .send()?
            .json::<SpotifyAccessToken>()
            .context("Could not get user access token.")?;

        self.access_token = Some(access_token);

        Ok(())
    }

    pub fn search_track(&mut self, beatmapset: &Beatmapset) -> Result<SpotifyTrack> {
        self.ensure_access_token()?;

        let Some(access_token) = &self.access_token else { bail!("Access token not found.") };

        let variables = json!({
            "searchTerm": format!("{} - {}", beatmapset.artist_unicode, beatmapset.title_unicode),
            "offset": 0,
            "limit": 20,
        });
        let extensions = json!({
            "persistedQuery": {
                "version": 1,
                "sha256Hash": "220d098228a4eaf216b39e8c147865244959c4cc6fd82d394d88afda0b710929",
            },
        });
        let json = self
            .client
            .get("https://api-partner.spotify.com/pathfinder/v1/query")
            .bearer_auth(&access_token.access_token)
            .query(&[("operationName", "searchTracks"), ("variables", &variables.to_string()), ("extensions", &extensions.to_string())])
            .send()?
            .json::<SpotifyData<SpotifySearchResult>>()?;

        for track in json.data.search_v2.tracks_v2.items {
            let Some(artist) = track.item.data.artists.items.into_iter().find(|artist: &SpotifyArtist| {
                [beatmapset.artist.to_lowercase(), beatmapset.artist_unicode.to_lowercase()]
                    .iter()
                    .any(|entry| entry.split(" feat").next().unwrap_or("").contains(&artist.profile.name))
            }) else {
                continue;
            };

            if ![beatmapset.title_unicode.to_lowercase(), beatmapset.title.to_lowercase()].contains(&track.item.data.name.to_lowercase()) {
                continue;
            }

            return Ok(SpotifyTrack {
                artist: artist.profile.name,
                title: track.item.data.name,
                album: track.item.data.album_of_track.name,
            });
        }

        bail!("Could not find track.");
    }
}
