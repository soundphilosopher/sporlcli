use std::time::Duration;

use indicatif::{ProgressBar, ProgressStyle};
use reqwest::{Client, StatusCode};
use tokio::time::sleep;

use crate::{
    config, error,
    management::TokenManager,
    types::{Artist, FollowedArtistsResponse},
};

/// Retrieves a page of followed artists from the Spotify Web API.
///
/// Fetches artists that the authenticated user follows using pagination with cursor-based
/// navigation. The function handles rate limiting and retries automatically for certain
/// error conditions like 502 Bad Gateway responses.
///
/// # Arguments
///
/// * `token` - Valid access token for Spotify API authentication
/// * `limit` - Maximum number of artists to return in this request (1-50)
/// * `after` - Optional cursor for pagination, specifying where to start the next page
///
/// # Returns
///
/// Returns a `Result` containing:
/// - `Ok((Vec<Artist>, Option<String>))` - List of artists and optional next cursor
/// - `Err(reqwest::Error)` - Network error, API error, or other HTTP-related error
///
/// # Retry Logic
///
/// The function implements automatic retry logic for 502 Bad Gateway errors with a
/// 10-second delay between attempts. Other errors are propagated immediately.
///
/// # API Rate Limits
///
/// This endpoint is subject to Spotify's rate limiting. The caller should implement
/// appropriate delays between requests when fetching multiple pages.
///
/// # Example
///
/// ```
/// let token = "BQC..."; // Valid access token
/// let (artists, next_cursor) = get_artist(token, 20, None).await?;
///
/// // Fetch next page if available
/// if let Some(cursor) = next_cursor {
///     let (more_artists, _) = get_artist(token, 20, Some(cursor)).await?;
/// }
/// ```
pub async fn get_artist(
    token: &str,
    limit: u64,
    after: Option<String>,
) -> Result<(Vec<Artist>, Option<String>), reqwest::Error> {
    let attempt_after = after.clone();

    loop {
        let mut api_url = format!(
            "{uri}/me/following?type=artist&limit={limit}",
            uri = &config::spotify_apiurl(),
            limit = limit
        );
        if let Some(after_val) = &attempt_after {
            api_url.push_str(&format!("&after={}", after_val));
        }

        let client = Client::new();
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

        let res = response.json::<FollowedArtistsResponse>().await?;
        let next_after = res.artists.cursors.and_then(|c| c.after);

        return Ok((res.artists.items, next_after));
    }
}

/// Retrieves the total count of artists followed by the authenticated user.
///
/// Makes a minimal API request to get just the total count without fetching
/// all artist data. Uses the stored token manager for authentication and
/// displays a progress spinner during the operation.
///
/// # Returns
///
/// Returns a `Result` containing:
/// - `Ok(u64)` - Total number of followed artists
/// - `Err(reqwest::Error)` - Network error, API error, or authentication failure
///
/// # Authentication
///
/// Loads the token from the token manager. If no valid token is found,
/// the function will terminate the program with an error message directing
/// the user to run `sporlcli auth`.
///
/// # Progress Indication
///
/// Displays a spinner with the message "Fetching remote artists count..."
/// while the request is in progress. The spinner is automatically cleared
/// when the operation completes or fails.
///
/// # Retry Logic
///
/// Implements the same retry logic as `get_artist()` for 502 Bad Gateway
/// errors with a 10-second delay.
///
/// # Error Handling
///
/// - Token loading failures result in program termination with error message
/// - Network errors are propagated to the caller
/// - Progress indicator is properly cleaned up on all exit paths
///
/// # Example
///
/// ```
/// let total_count = get_total_artist_count().await?;
/// println!("You follow {} artists", total_count);
/// ```
///
/// # API Efficiency
///
/// This function uses `limit=1` to minimize data transfer while still getting
/// the total count from the API response metadata.
pub async fn get_total_artist_count() -> Result<u64, reqwest::Error> {
    let mut token_mgr = match TokenManager::load().await {
        Ok(t) => t,
        Err(e) => {
            error!(
                "Failed to load token. Please run sporlcli auth\n Error: {}",
                e
            );
        }
    };

    let pb = ProgressBar::new_spinner();
    pb.set_message("Fetching remote artists count...");
    pb.enable_steady_tick(Duration::from_millis(100));
    pb.set_style(
        ProgressStyle::with_template("{spinner:.blue} {msg}")
            .unwrap()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏"),
    );

    loop {
        let token = token_mgr.get_valid_token().await;
        let api_url = format!("{uri}/me/following?type={type}&limit={limit}", uri = &config::spotify_apiurl(), type = "artist", limit = "1");

        let client = Client::new();
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

                    pb.finish_and_clear();
                    return Err(err); // propagate other errors
                }
            },
            Err(err) => {
                pb.finish_and_clear();
                return Err(err);
            } // network or reqwest error
        };

        pb.finish_and_clear();
        let res = response.json::<FollowedArtistsResponse>().await?;

        return Ok(res.artists.total.unwrap_or_else(|| 0));
    }
}
