use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use tabled::Tabled;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    pub access_token: String,
    pub refresh_token: String,
    pub scope: String,
    pub expires_in: u64,
    pub obtained_at: u64,
}

#[derive(Debug, Clone)]
pub struct PkceToken {
    pub code_verifier: String,
    pub token: Option<Token>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artist {
    pub id: String,
    pub name: String,
    pub genres: Vec<String>,
}

#[derive(Tabled)]
pub struct ArtistTableRow {
    pub name: String,
    pub genres: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FollowedArtistsResponse {
    pub artists: ArtistsContainer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtistsContainer {
    pub items: Vec<Artist>,
    pub next: Option<String>,
    pub cursors: Option<Cursors>,
    pub total: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cursors {
    pub after: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlbumResponse {
    pub items: Vec<Album>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Album {
    pub id: String,
    pub name: String,
    pub release_date: String,
    pub release_date_precision: String,
    pub album_type: String,
    pub artists: Vec<AlbumArtist>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtistReleases {
    pub artist: Artist,
    pub releases: Vec<Album>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlbumArtist {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct WeekOfTheYear {
    pub week: u32,
    pub dates: Vec<NaiveDate>,
}

#[derive(Debug, Clone)]
pub struct ReleaseWeek {
    pub week: WeekOfTheYear,
    pub year: i32,
    pub releases: Vec<Album>,
}

#[derive(Tabled)]
pub struct ReleaseTableRow {
    pub date: String,
    pub name: String,
    pub artists: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePlaylistRequest {
    pub name: String,
    pub description: String,
    pub public: bool,
    pub collaborative: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePlaylistResponse {
    pub id: String,
    pub name: String,
    pub description: String,
    pub public: bool,
    pub collaborative: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetSeveralAlbumsResponse {
    pub albums: Vec<GetAlbumResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetAlbumResponse {
    pub id: String,
    pub name: String,
    pub release_date: String,
    pub tracks: Tracks,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tracks {
    pub items: Vec<Track>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    pub id: String,
    pub name: String,
    pub uri: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddTrackToPlaylistRequest {
    pub uris: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddTrackToPlaylistResponse {
    pub snapshot_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetUserPlaylistsResponse {
    pub items: Vec<Playlist>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Playlist {
    pub id: String,
    pub name: String,
    pub description: String,
    pub public: bool,
    pub collaborative: bool,
    pub snapshot_id: String,
}
