use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use tabled::Tabled;

/// Represents an OAuth access token with refresh capabilities.
///
/// Contains all the necessary information for authenticating with the Spotify API,
/// including the access token, refresh token, scope permissions, and expiration details.
/// This structure is used to manage the authentication state throughout the application.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    /// The access token used for API authentication
    pub access_token: String,
    /// The refresh token used to obtain new access tokens
    pub refresh_token: String,
    /// The scope of permissions granted to the token
    pub scope: String,
    /// The lifetime of the access token in seconds
    pub expires_in: u64,
    /// Unix timestamp when the token was obtained
    pub obtained_at: u64,
}

/// Represents a PKCE (Proof Key for Code Exchange) token pair.
///
/// Used in the OAuth PKCE flow for secure authentication. Contains the code verifier
/// that was generated locally and the optional token that may be obtained after
/// the authentication flow completes.
#[derive(Debug, Clone)]
pub struct PkceToken {
    /// The code verifier used in the PKCE flow
    pub code_verifier: String,
    /// The optional token obtained after successful authentication
    pub token: Option<Token>,
}

/// Represents a Spotify artist with basic information and genre classification.
///
/// Contains the essential information about an artist including their unique identifier,
/// display name, and associated genres. This is used throughout the application
/// when working with artist data from the Spotify API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artist {
    /// Unique Spotify identifier for the artist
    pub id: String,
    /// Display name of the artist
    pub name: String,
    /// List of genres associated with the artist
    pub genres: Vec<String>,
}

/// Represents an artist row for table display purposes.
///
/// A simplified representation of artist data optimized for display in tabular format.
/// The genres are stored as a single formatted string rather than a vector for
/// easier table rendering.
#[derive(Tabled)]
pub struct ArtistTableRow {
    /// Artist's display name
    pub name: String,
    /// Comma-separated string of artist's genres
    pub genres: String,
}

/// Response structure for Spotify's followed artists API endpoint.
///
/// Represents the top-level response when fetching followed artists from Spotify.
/// Contains the artists data wrapped in the expected response structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FollowedArtistsResponse {
    /// Container holding the actual artist data and pagination information
    pub artists: ArtistsContainer,
}

/// Container for artist data with pagination support.
///
/// Wraps a list of artists along with pagination metadata. Used by Spotify's API
/// to provide paginated results when fetching large lists of artists, allowing
/// for efficient data retrieval and navigation through result sets.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtistsContainer {
    /// List of artist objects in the current page
    pub items: Vec<Artist>,
    /// URL for the next page of results, if available
    pub next: Option<String>,
    /// Cursor-based pagination information
    pub cursors: Option<Cursors>,
    /// Total number of items available (may not always be provided)
    pub total: Option<u64>,
}

/// Cursor-based pagination information for API responses.
///
/// Provides cursor information for navigating through paginated API responses.
/// The cursor system allows for more efficient pagination compared to offset-based
/// pagination, especially for large datasets.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cursors {
    /// Cursor pointing to the position after the current page
    pub after: Option<String>,
}

/// Response structure for album-related API endpoints.
///
/// A simple wrapper around a list of albums, used when the API returns
/// multiple albums without additional pagination or metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlbumResponse {
    /// List of album objects
    pub items: Vec<Album>,
}

/// Represents a Spotify album with release information and artist details.
///
/// Contains comprehensive information about an album including its metadata,
/// release information, type classification, and associated artists. This is
/// a core data structure used throughout the application for album management.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Album {
    /// Unique Spotify identifier for the album
    pub id: String,
    /// Album title/name
    pub name: String,
    /// Release date in string format (precision varies)
    pub release_date: String,
    /// Precision of the release date (year, month, or day)
    pub release_date_precision: String,
    /// Type of album (album, single, compilation, etc.)
    pub album_type: String,
    /// List of artists associated with the album
    pub artists: Vec<AlbumArtist>,
}

/// Represents an artist and their associated releases.
///
/// Groups an artist with their collection of releases, providing a convenient
/// way to organize and display artist-specific release information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtistReleases {
    /// The artist information
    pub artist: Artist,
    /// List of releases by this artist
    pub releases: Vec<Album>,
}

/// Represents an artist in the context of an album.
///
/// A simplified artist representation used specifically within album contexts.
/// Contains only the essential identifying information needed when an artist
/// is referenced as part of an album's metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlbumArtist {
    /// Unique Spotify identifier for the artist
    pub id: String,
    /// Artist's display name
    pub name: String,
}

/// Represents a week within a year with all its dates.
///
/// Defines a specific week by its number and contains all the individual dates
/// that fall within that week. Used for organizing releases by weekly periods
/// in the music release tracking system.
#[derive(Debug, Clone)]
pub struct WeekOfTheYear {
    /// Week number within the year (1-based)
    pub week: u32,
    /// All dates that fall within this week
    pub dates: Vec<NaiveDate>,
}

/// Represents a complete release week with associated albums.
///
/// Combines week information with the actual music releases that occurred
/// during that week. This provides a complete picture of musical activity
/// for a specific time period.
#[derive(Debug, Clone)]
pub struct ReleaseWeek {
    /// The week information (number and dates)
    pub week: WeekOfTheYear,
    /// The year this week belongs to
    pub year: i32,
    /// All album releases during this week
    pub releases: Vec<Album>,
}

/// Represents a release entry for table display purposes.
///
/// A flattened representation of release data optimized for tabular display.
/// Converts complex album and artist data into simple strings suitable for
/// rendering in tables and reports.
#[derive(Tabled)]
pub struct ReleaseTableRow {
    /// Release date formatted as a string
    pub date: String,
    /// Album/release name
    pub name: String,
    /// Formatted string of artist names
    pub artists: String,
}

/// Request payload for creating a new Spotify playlist.
///
/// Contains all the necessary information to create a playlist via the Spotify API,
/// including metadata and privacy settings. Used when programmatically creating
/// playlists for organizing music releases.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePlaylistRequest {
    /// Name/title for the new playlist
    pub name: String,
    /// Description text for the playlist
    pub description: String,
    /// Whether the playlist should be public
    pub public: bool,
    /// Whether the playlist should allow collaborative editing
    pub collaborative: bool,
}

/// Response from Spotify after successfully creating a playlist.
///
/// Contains the information about the newly created playlist, including its
/// assigned unique identifier and confirmed settings. Used to verify playlist
/// creation and obtain the playlist ID for subsequent operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePlaylistResponse {
    /// Unique Spotify identifier for the created playlist
    pub id: String,
    /// Confirmed name of the playlist
    pub name: String,
    /// Confirmed description of the playlist
    pub description: String,
    /// Confirmed public/private status
    pub public: bool,
    /// Confirmed collaborative status
    pub collaborative: bool,
}

/// Response structure for fetching multiple albums from Spotify.
///
/// Used when requesting detailed information about multiple albums simultaneously.
/// Provides a wrapper around the list of detailed album responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetSeveralAlbumsResponse {
    /// List of detailed album information
    pub albums: Vec<GetAlbumResponse>,
}

/// Detailed album information including track listings.
///
/// An extended album representation that includes the complete track listing
/// in addition to basic album metadata. Used when full album details are
/// needed, particularly for playlist creation or detailed analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetAlbumResponse {
    /// Unique Spotify identifier for the album
    pub id: String,
    /// Album title/name
    pub name: String,
    /// Release date of the album
    pub release_date: String,
    /// Complete track listing for the album
    pub tracks: Tracks,
}

/// Container for track information within an album.
///
/// Wraps a list of tracks, providing a structured way to represent
/// the track listing of an album. Used as part of detailed album
/// information when track-level data is required.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tracks {
    /// List of tracks in the album
    pub items: Vec<Track>,
}

/// Represents an individual track with essential playback information.
///
/// Contains the core information needed to identify and play a track,
/// including its Spotify URI which is used for adding tracks to playlists
/// and other playback operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    /// Unique Spotify identifier for the track
    pub id: String,
    /// Track title/name
    pub name: String,
    /// Spotify URI for the track (used for playback and playlist operations)
    pub uri: String,
}

/// Request payload for adding tracks to a Spotify playlist.
///
/// Contains the list of track URIs to be added to a playlist. Used when
/// programmatically populating playlists with specific tracks, such as
/// when creating release-based playlists.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddTrackToPlaylistRequest {
    /// List of Spotify track URIs to add to the playlist
    pub uris: Vec<String>,
}

/// Response from Spotify after adding tracks to a playlist.
///
/// Provides confirmation that tracks were successfully added and includes
/// a snapshot ID that represents the current state of the playlist. The
/// snapshot ID can be used for version tracking and conflict resolution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddTrackToPlaylistResponse {
    /// Snapshot identifier representing the current playlist state
    pub snapshot_id: String,
}

/// Response structure for fetching user playlists from Spotify.
///
/// Contains a list of playlists belonging to the authenticated user.
/// Used when retrieving existing playlists for display, management,
/// or playlist selection operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetUserPlaylistsResponse {
    /// List of user's playlists
    pub items: Vec<Playlist>,
}

/// Represents a Spotify playlist with metadata and settings.
///
/// Contains comprehensive information about a playlist including its
/// identification, metadata, privacy settings, and current state.
/// Used for playlist management and display operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Playlist {
    /// Unique Spotify identifier for the playlist
    pub id: String,
    /// Playlist name/title
    pub name: String,
    /// Playlist description text
    pub description: String,
    /// Whether the playlist is publicly visible
    pub public: bool,
    /// Whether the playlist allows collaborative editing
    pub collaborative: bool,
    /// Current snapshot identifier for the playlist state
    pub snapshot_id: String,
}
