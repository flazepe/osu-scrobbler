mod payloads;

use crate::scores::Beatmapset;
use anyhow::{bail, Context, Result};
use payloads::{SpotifyArtist, SpotifyData, SpotifySearchResult, SpotifyToken, SpotifyTrack};
use reqwest::blocking::Client;

use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Spotify {
    client: Client,
    token: Option<SpotifyToken>,
}

impl Spotify {
    pub fn new() -> Self {
        Self { client: Client::new(), token: None }
    }

    fn ensure_token(&mut self) -> Result<()> {
        if let Some(token) = &self.token {
            if SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() < token.access_token_expiration_timestamp_ms {
                return Ok(());
            }
        }

        let token = self
            .client
            .get("https://open.spotify.com/get_access_token")
            .query(&[("reason", "transport"), ("productType", "web_player")])
            .header("user-agent", "yes")
            .send()?
            .json::<SpotifyToken>()
            .context("Could not get user token.")?;

        self.token = Some(token);

        Ok(())
    }

    pub fn search_track(&mut self, beatmapset: Beatmapset) -> Result<SpotifyTrack> {
        self.ensure_token()?;

        let Some(token) = &self.token else { bail!("Token not found.") };

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
            .bearer_auth(&token.access_token)
            .query(&[("operationName", "searchTracks"), ("variables", &variables.to_string()), ("extensions", &extensions.to_string())])
            .send()?
            .json::<SpotifyData<SpotifySearchResult>>()?;

        for track in json.data.search_v2.tracks_v2.items {
            let Some(artist) = track.item.data.artists.items.into_iter().find(|artist: &SpotifyArtist| {
                [beatmapset.artist.to_lowercase(), beatmapset.artist_unicode.to_lowercase()]
                    .map(|artist| artist.split(" feat").next().unwrap_or("").to_string())
                    .contains(&artist.profile.name.to_lowercase())
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
