use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpotifyToken {
    pub access_token: String,
    pub access_token_expiration_timestamp_ms: u128,
}

#[derive(Deserialize)]
pub struct SpotifyTrack {
    pub artist: String,
    pub title: String,
    pub album: String,
}

#[derive(Deserialize)]
pub struct SpotifyData<T> {
    pub data: T,
}

#[derive(Deserialize)]
pub struct SpotifyItems<T> {
    pub items: Vec<T>,
}

#[derive(Deserialize)]
pub struct SpotifyItem<T> {
    pub item: T,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpotifySearchResult {
    pub search_v2: SpotifySearchV2,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpotifySearchV2 {
    pub tracks_v2: SpotifyItems<SpotifyItem<SpotifyData<SpotifyTrackItem>>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpotifyTrackItem {
    pub name: String,
    pub album_of_track: SpotifyAlbum,
    pub artists: SpotifyItems<SpotifyArtist>,
}

#[derive(Deserialize)]
pub struct SpotifyAlbum {
    pub name: String,
}

#[derive(Deserialize)]
pub struct SpotifyArtist {
    pub profile: SpotifyArtistProfile,
}

#[derive(Deserialize)]
pub struct SpotifyArtistProfile {
    pub name: String,
}
