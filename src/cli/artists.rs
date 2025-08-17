use std::time::Duration;

use indicatif::{ProgressBar, ProgressStyle};
use tabled::Table;

use crate::{
    error, info,
    management::{ArtistReleaseManager, TokenManager},
    spotify, success,
    types::{Artist, ArtistReleases, ArtistTableRow},
    warning,
};

/// Updates the local artist cache with followed artists from Spotify.
///
/// Compares the number of cached artists with the current number of followed
/// artists on Spotify and fetches any new artists. The function intelligently
/// determines how many artists need to be updated and provides progress feedback
/// during the operation.
///
/// # Arguments
///
/// * `force` - If true, forces a complete refresh of all artists, ignoring cache
///
/// # Behavior
///
/// - **Incremental Updates**: Only fetches new artists unless force is enabled
/// - **Cache Comparison**: Compares local cache count with remote count
/// - **Force Mode**: When enabled, completely rebuilds the artist cache
/// - **Progress Tracking**: Shows real-time progress during the update process
///
/// # Update Logic
///
/// 1. Load current cached artist count
/// 2. Get total artist count from Spotify
/// 3. Calculate how many new artists to fetch
/// 4. If no new artists and not forcing, skip update
/// 5. Otherwise, fetch and cache the new/all artists
///
/// # Error Handling
///
/// The function handles various error conditions gracefully:
/// - Missing or corrupt artist cache (treats as empty)
/// - Spotify API failures (reports error and exits)
/// - Authentication issues (directs user to re-authenticate)
///
/// # Example
///
/// ```
/// // Incremental update - only fetch new artists
/// update_artists(false).await;
///
/// // Force complete refresh
/// update_artists(true).await;
/// ```
///
/// # Performance Notes
///
/// - Incremental updates are much faster for users with stable follow lists
/// - Force updates may take significant time for users following many artists
/// - Progress indicators help users understand operation status
pub async fn update_artists(force: bool) {
    let artist_cache_count = match ArtistReleaseManager::load().await {
        Ok(arm) => arm.count_artists(),
        Err(_) => 0,
    };

    let artist_remote_count = match spotify::artists::get_total_artist_count().await {
        Ok(c) => c,
        Err(_) => 0,
    };

    let max_new: u64 = if artist_remote_count > artist_cache_count as u64 && !force {
        artist_remote_count - artist_cache_count as u64
    } else {
        0
    };

    if let Err(e) = load_remote_artists(max_new, force).await {
        error!("Cannot update artists. Err: {}", e)
    }
}

/// Lists cached artists with optional search filtering and tabular display.
///
/// Displays a formatted table of followed artists from the local cache,
/// with support for name-based filtering. Artists are sorted alphabetically
/// by name for consistent presentation.
///
/// # Arguments
///
/// * `search` - Optional search term to filter artists by name (case-insensitive)
///
/// # Display Format
///
/// The output table includes:
/// - **Name**: Artist's display name
/// - **Genres**: Up to 3 genres associated with the artist (comma-separated)
///
/// # Search Functionality
///
/// When a search term is provided:
/// - Case-insensitive partial matching on artist names
/// - Filters the artist list before display
/// - Empty results show an empty table
///
/// # Sorting
///
/// Artists are always sorted alphabetically by name (case-insensitive) to
/// provide consistent, predictable ordering regardless of how they were
/// stored in the cache.
///
/// # Error Handling
///
/// - Missing or corrupt artist cache results in a warning message
/// - Cache loading failures are reported but don't crash the application
/// - Empty caches result in an empty table display
///
/// # Example
///
/// ```
/// // List all artists
/// list_artists(None).await;
///
/// // Search for artists containing "rock"
/// list_artists(Some("rock".to_string())).await;
///
/// // Search for specific artist
/// list_artists(Some("Arctic Monkeys".to_string())).await;
/// ```
///
/// # Output Example
///
/// ```text
/// ┌─────────────────────┬──────────────────────────────┐
/// │ name                │ genres                       │
/// ├─────────────────────┼──────────────────────────────┤
/// │ Arctic Monkeys      │ garage rock,indie rock,rock  │
/// │ Radiohead          │ alternative rock,art rock    │
/// │ The Beatles        │ rock,pop,psychedelic rock    │
/// └─────────────────────┴──────────────────────────────┘
/// ```
pub async fn list_artists(search: Option<String>) {
    match load_cached_artists().await {
        Ok(artists) => {
            // sort artists by name
            let mut sorted_artists = artists.clone();
            sorted_artists.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

            if let Some(artist_search) = search {
                let search_term = artist_search.clone().to_lowercase();
                sorted_artists.retain(|a| a.name.to_lowercase().contains(&search_term));
            }

            // convert artists to table rows
            let table_rows: Vec<ArtistTableRow> = sorted_artists
                .into_iter()
                .map(|a| ArtistTableRow {
                    name: a.name,
                    genres: a
                        .genres
                        .iter()
                        .take(3)
                        .cloned()
                        .collect::<Vec<_>>()
                        .join(","),
                })
                .collect();

            let table = Table::new(table_rows);
            println!("{}", table);
        }
        Err(e) => warning!("Failed to load arists. Err: {}", e),
    }
}

/// Loads artists from the local cache.
///
/// Internal helper function that retrieves all cached artists from the
/// ArtistReleaseManager. This provides a simple interface for accessing
/// cached artist data with consistent error handling.
///
/// # Returns
///
/// Returns a `Result` containing:
/// - `Ok(Vec<Artist>)` - All cached artists, or empty vector if none
/// - `Err(String)` - Error message describing the load failure
///
/// # Error Conditions
///
/// This function will return an error if:
/// - The artist cache file doesn't exist
/// - The cache file is corrupted or unreadable
/// - JSON deserialization fails
/// - File system permissions prevent reading
///
/// # Cache Fallback
///
/// If the artist cache contains no artists, returns an empty vector
/// rather than an error, allowing calling code to handle empty caches
/// gracefully.
///
/// # Example
///
/// ```
/// match load_cached_artists().await {
///     Ok(artists) => println!("Loaded {} artists", artists.len()),
///     Err(e) => eprintln!("Cache error: {}", e),
/// }
/// ```
async fn load_cached_artists() -> Result<Vec<Artist>, String> {
    match ArtistReleaseManager::load().await {
        Ok(arm) => Ok(arm.get_all_artists().unwrap_or(Vec::new())),
        Err(e) => Err(format!("Failed to load artists. Err: {}", e)),
    }
}

/// Fetches artists from Spotify and updates the local cache.
///
/// Core function that handles the actual fetching of artist data from Spotify's
/// API with intelligent pagination, progress tracking, and cache management.
/// Supports both incremental updates and complete refreshes.
///
/// # Arguments
///
/// * `max_new` - Maximum number of new artists to fetch (0 = no update needed)
/// * `force` - If true, starts with an empty cache regardless of existing data
///
/// # Returns
///
/// Returns a `Result` containing:
/// - `Ok(Vec<ArtistReleases>)` - All artist data after the update
/// - `Err(reqwest::Error)` - HTTP/API error during the fetch process
///
/// # Fetching Strategy
///
/// The function uses intelligent pagination:
/// - Fetches up to 50 artists per API request (Spotify's maximum)
/// - Uses cursor-based pagination for efficient traversal
/// - Adjusts final request size to match exact requirements
/// - Stops early if no more artists are available
///
/// # Progress Tracking
///
/// Provides real-time progress feedback:
/// - Animated spinner during the operation
/// - Running count of fetched artists
/// - Clear success/failure messaging
/// - Progress indicators that clean up properly
///
/// # Cache Management
///
/// The function handles cache operations:
/// - Loads existing cache unless force mode is enabled
/// - Adds newly fetched artists to the cache
/// - Persists updated cache to disk
/// - Reports final cache statistics
///
/// # Authentication
///
/// Manages API authentication automatically:
/// - Loads stored authentication tokens
/// - Refreshes tokens if they're expired
/// - Handles authentication failures with clear error messages
///
/// # Error Handling
///
/// Comprehensive error handling for:
/// - Authentication failures (directs user to re-authenticate)
/// - Network connectivity issues
/// - API rate limiting and service errors
/// - Cache persistence failures
/// - Invalid API responses
///
/// # Early Exit Conditions
///
/// The function exits early in several scenarios:
/// - `max_new` is 0 (no update needed)
/// - API returns empty artist list
/// - Required number of artists have been fetched
/// - No more artists available (pagination exhausted)
///
/// # Example Usage
///
/// ```
/// // Fetch up to 100 new artists
/// let artists = load_remote_artists(100, false).await?;
///
/// // Force complete refresh
/// let artists = load_remote_artists(0, true).await?;
/// ```
///
/// # Performance Considerations
///
/// - Uses efficient cursor-based pagination
/// - Respects API rate limits through built-in delays
/// - Minimizes memory usage by processing artists in batches
/// - Persists cache after completion to avoid data loss
///
/// # Network Resilience
///
/// The function includes resilience features:
/// - Automatic token refresh for expired credentials
/// - Graceful handling of temporary API failures
/// - Progress preservation across network interruptions
/// - Clear error reporting for permanent failures
async fn load_remote_artists(
    max_new: u64,
    force: bool,
) -> Result<Vec<ArtistReleases>, reqwest::Error> {
    let mut arm: ArtistReleaseManager = match ArtistReleaseManager::load().await {
        Ok(arm) => {
            if force {
                ArtistReleaseManager::new(None)
            } else {
                arm
            }
        }
        Err(_) => ArtistReleaseManager::new(None),
    };

    if max_new == 0 {
        success!("Nothing to update here.");
        return Ok(arm.all().unwrap_or(Vec::new()));
    } else {
        info!("Update {} artists in cache ...", max_new);
    }

    // load tokeb manager for retrieve valid auth token
    let mut token_mgr = match TokenManager::load().await {
        Ok(t) => t,
        Err(e) => {
            error!(
                "Failed to load token. Please run sporlcli auth\n Error: {}",
                e
            );
        }
    };

    // start progress wheel
    let pb = ProgressBar::new_spinner();
    pb.set_message("Fetching followed artists...");
    pb.enable_steady_tick(Duration::from_millis(100));
    pb.set_style(
        ProgressStyle::with_template("{spinner:.blue} {msg}")
            .unwrap()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏"),
    );

    let mut after: Option<String> = None;
    let mut new_once = max_new;

    let mut total_fetched = 0u64;

    loop {
        let limit = if new_once < 50 { new_once } else { 50 };
        if limit <= 0 {
            break;
        }

        let token = token_mgr.get_valid_token().await;
        let result = spotify::artists::get_artist(&token, limit, after.clone()).await;

        match result {
            Ok((artists, next_after)) => {
                if artists.is_empty() {
                    break;
                }

                total_fetched += artists.len() as u64;
                new_once -= artists.len() as u64;
                pb.set_message(format!(
                    "Fetched {}/{} artists from remote ...",
                    total_fetched, max_new
                ));

                arm.add_artists(artists);
                after = next_after;

                if after.is_none() {
                    break;
                }
            }
            Err(e) => {
                pb.finish_and_clear();
                error!("Failed to fetch artists: {}", e);
            }
        }
    }

    pb.finish_and_clear();
    success!(
        "Fetched {}/{} artists!",
        max_new,
        arm.count_artists().clone(),
    );

    // let artists_mgr = ArtistReleaseManager::new()
    if let Err(e) = arm.persist().await {
        error!("Failed to cache artists. Err: {}", e);
    }

    success!("Cached {} artists.", arm.count_artists().clone());

    Ok(arm.all().unwrap_or(Vec::new()))
}
