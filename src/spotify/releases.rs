use std::time::Duration;

use reqwest::{Client, StatusCode};
use tokio::time::sleep;

use crate::{
    config, error,
    management::TokenManager,
    types::{Album, AlbumResponse, GetSeveralAlbumsResponse},
    utils, warning,
};

/// Retrieves albums/releases for a specific artist from the Spotify Web API.
///
/// Fetches a list of albums by an artist, filtered by release types (album, single,
/// compilation, appears_on). The function handles rate limiting gracefully by
/// respecting the `Retry-After` header when encountering 429 Too Many Requests responses.
///
/// # Arguments
///
/// * `artist_id` - Spotify ID of the artist to fetch releases for
/// * `token` - Valid access token for Spotify API authentication
/// * `limit` - Maximum number of albums to return (1-50, default 20)
/// * `release_types` - Specifies which types of releases to include (album, single, etc.)
///
/// # Returns
///
/// Returns a `Result` containing:
/// - `Ok(Vec<Album>)` - List of albums matching the criteria
/// - `Err(reqwest::Error)` - Network error, API error, or HTTP error
///
/// # Rate Limiting
///
/// The function implements intelligent rate limit handling:
/// - Detects 429 Too Many Requests responses
/// - Reads the `Retry-After` header for the recommended delay
/// - Automatically waits and retries for delays ≤ 120 seconds
/// - Issues a warning for excessive delays (> 120 seconds)
///
/// # Release Type Filtering
///
/// The `release_types` parameter determines which categories of releases to include:
/// - `album` - Full-length studio albums
/// - `single` - Singles and EPs
/// - `appears_on` - Albums the artist appears on but doesn't own
/// - `compilation` - Compilation albums and greatest hits
/// - `all` - All of the above types
///
/// # API Endpoint
///
/// Uses Spotify's `/artists/{id}/albums` endpoint with the following parameters:
/// - `include_groups` - Comma-separated list of release types
/// - `limit` - Number of results to return
///
/// # Error Handling
///
/// - Rate limit responses are handled automatically with retry logic
/// - Network errors are propagated to the caller
/// - Invalid artist IDs result in API errors that are propagated
/// - Malformed responses are handled by reqwest's JSON parsing
///
/// # Example
///
/// ```
/// let artist_id = "4NHQUGzhtTLFvgF5SZesLK"; // Tove Lo
/// let token = "BQC..."; // Valid access token
/// let release_types = utils::parse_release_kinds("album,single")?;
///
/// let albums = get_release_for_artist(
///     artist_id.to_string(),
///     token,
///     20,
///     &release_types
/// ).await?;
///
/// println!("Found {} releases", albums.len());
/// ```
///
/// # Performance Notes
///
/// - Each request fetches up to `limit` albums
/// - For artists with many releases, multiple requests may be needed
/// - Rate limiting may introduce delays during high-frequency usage
/// - Consider caching results for frequently accessed artists
pub async fn get_release_for_artist(
    artist_id: String,
    token: &str,
    limit: u32,
    release_types: &utils::ReleaseKinds,
) -> Result<Vec<Album>, reqwest::Error> {
    let client = Client::new();
    let api_url = format!(
        "{uri}/artists/{id}/albums?include_groups={include_groups}&limit={limit}",
        uri = &config::spotify_apiurl(),
        id = artist_id,
        include_groups = format!("{}", release_types),
        limit = limit
    );

    let response = client.get(&api_url).bearer_auth(token).send().await?;
    // check for retry-after header
    if response.status() == StatusCode::TOO_MANY_REQUESTS {
        if let Some(retry_after) = response.headers().get("retry-after") {
            let retry_after = retry_after
                .to_str()
                .unwrap_or("0")
                .parse::<u64>()
                .unwrap_or(0);
            if retry_after <= 120 {
                sleep(Duration::from_secs(retry_after)).await;
            } else {
                warning!(
                    "Retry after has reached a abnormal high of {} seconds. Try your best tommorrow again.",
                    retry_after
                );
            }
        }
    }

    let json = response.json::<AlbumResponse>().await?;

    Ok(json.items)
}

/// Retrieves detailed information for multiple albums in a single API request.
///
/// Fetches comprehensive album data including track listings for a batch of albums
/// using their Spotify IDs. This is more efficient than making individual requests
/// for each album when detailed information is needed.
///
/// # Arguments
///
/// * `albums` - Vector of Album objects containing the IDs to fetch details for
///
/// # Returns
///
/// Returns a `Result` containing:
/// - `Ok(GetSeveralAlbumsResponse)` - Detailed album information with track listings
/// - `Err(reqwest::Error)` - HTTP error, network error, or API error
///
/// # Batch Processing
///
/// - Combines up to 20 album IDs in a single request (Spotify API limit)
/// - IDs are extracted from the input albums and joined with commas
/// - Single API call reduces network overhead and improves performance
///
/// # Authentication
///
/// Uses the stored token manager for authentication. If no valid token is found,
/// the function will terminate the program with an error message directing the
/// user to authenticate.
///
/// # Detailed Album Data
///
/// The response includes enhanced album information:
/// - Basic album metadata (name, release date, etc.)
/// - Complete track listings with track IDs and URIs
/// - Track metadata (name, duration, etc.)
/// - Additional album details not available in basic album responses
///
/// # Retry Logic
///
/// Implements automatic retry for 502 Bad Gateway errors with a 10-second delay.
/// Other HTTP errors are propagated immediately to the caller.
///
/// # Error Conditions
///
/// Common failure scenarios:
/// - Invalid album IDs (some albums may not be found)
/// - Albums unavailable in the user's market
/// - Network connectivity issues
/// - Authentication token expired or invalid
/// - Spotify API service errors
///
/// # API Limitations
///
/// - Maximum 20 album IDs per request
/// - Total URL length limitations may apply for very long ID lists
/// - Some albums may be unavailable in certain geographic markets
/// - Private or unreleased albums may not be accessible
///
/// # Example
///
/// ```
/// let albums = vec![
///     Album { id: "abc123".to_string(), ..Default::default() },
///     Album { id: "def456".to_string(), ..Default::default() },
/// ];
///
/// let detailed_response = get_several_releases(&albums).await?;
/// for album in detailed_response.albums {
///     println!("Album: {} has {} tracks", album.name, album.tracks.items.len());
/// }
/// ```
///
/// # Use Cases
///
/// This function is particularly useful for:
/// - Creating playlists with specific tracks from albums
/// - Analyzing track-level data across multiple albums
/// - Bulk operations requiring detailed album information
/// - Reducing API call volume when working with album collections
pub async fn get_several_releases(
    albums: &Vec<Album>,
) -> Result<GetSeveralAlbumsResponse, reqwest::Error> {
    let album_ids = albums
        .iter()
        .map(|a| a.id.as_str())
        .collect::<Vec<_>>()
        .join(",");

    let api_url = format!(
        "{url}/albums?ids={album_ids}",
        url = &config::spotify_apiurl(),
        album_ids = album_ids
    );

    let mut token_mgr = match TokenManager::load().await {
        Ok(manager) => manager,
        Err(e) => {
            error!(
                "Failed to load token. Please run sporlcli auth\n Error: {}",
                e
            );
        }
    };

    loop {
        let client = Client::new();
        let token = token_mgr.get_valid_token().await;
        let response = client.get(&api_url).bearer_auth(token).send().await;

        let response = match response {
            Ok(resp) => match resp.error_for_status() {
                Ok(valid_response) => valid_response,
                Err(err) => {
                    if let Some(status) = err.status() {
                        if status == StatusCode::BAD_GATEWAY {
                            sleep(Duration::from_secs(10)).await;
                            continue; // retry
                        }
                    }

                    return Err(err); // propagate other errors
                }
            },
            Err(err) => {
                return Err(err);
            } // network or reqwest error
        };

        let json = response.json::<GetSeveralAlbumsResponse>().await?;
        return Ok(json);
    }
}
