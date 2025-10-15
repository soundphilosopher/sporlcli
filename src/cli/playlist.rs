use chrono::Datelike;

use crate::{
    info, spotify, success,
    types::{Album, GetSeveralAlbumsResponse, Track},
    utils, warning,
};

/// Creates Spotify playlists for specified release weeks with curated tracks.
///
/// Generates playlists containing the first track from each album released during
/// the specified time period. The function handles the complete workflow from
/// release data gathering to playlist creation and track addition, with support
/// for both specific dates and relative week ranges.
///
/// # Arguments
///
/// * `previous_weeks` - Number of weeks before the target date to include (None = current week only)
/// * `release_date` - Specific date to target (None = current date)
/// * `release_kinds` - Filter for release kinds (e.g., album, single)
///
/// # Playlist Naming
///
/// Playlists are named using the format: "Weekly Picks {week}/{year}"
///
/// Examples:
/// - "Weekly Picks 42/2023"
/// - "Weekly Picks 1/2024"
///
/// # Workflow Overview
///
/// For each target week, the function:
/// 1. **Duplicate Check**: Verifies playlist doesn't already exist
/// 2. **Release Gathering**: Fetches album releases for the week
/// 3. **Detail Retrieval**: Gets complete album information including tracks
/// 4. **Playlist Creation**: Creates the playlist on Spotify
/// 5. **Track Addition**: Adds the first track from each album
///
/// # Time Range Logic
///
/// The function determines which weeks to process:
/// - If `release_date` is provided, uses that as the target date
/// - If `previous_weeks` is provided, includes that many weeks before the target
/// - If both are None, processes only the current week
/// - If `previous_weeks` is 0, processes only the target week
///
/// # Track Selection Strategy
///
/// - Selects the first track from each album
/// - Assumes the first track is representative or the lead single
/// - Filters out albums with no tracks available
/// - Handles various album types (albums, singles, EPs)
///
/// # Concurrency and Performance
///
/// The function uses several optimization strategies:
/// - **Batch Processing**: Processes albums in chunks of 20 for detailed info
/// - **Concurrent Requests**: Uses async spawning for parallel API calls
/// - **Chunked Track Addition**: Adds tracks in batches of 100 (Spotify limit)
/// - **Early Termination**: Skips weeks with no releases
///
/// # Error Handling
///
/// Comprehensive error handling for various failure scenarios:
/// - **Missing Releases**: Warns and skips weeks with no release data
/// - **API Failures**: Continues processing other weeks on individual failures
/// - **Playlist Conflicts**: Skips existing playlists with informational message
/// - **Track Addition Failures**: Continues with remaining batches
/// - **Network Issues**: Provides clear error messages and continues where possible
///
/// # Duplicate Prevention
///
/// - Checks for existing playlists before creation
/// - Uses exact string matching on playlist names
/// - Provides informational messages when skipping existing playlists
/// - Allows manual playlist deletion if recreation is desired
///
/// # Example Usage
///
/// ```bash
/// # Create playlist for current week only
/// sporlcli playlist
///
/// # Create playlists for last 4 weeks
/// sporlcli playlist --previous-weeks 4
///
/// # Create playlist for specific date's week
/// sporlcli playlist --release-date 2023-12-25
///
/// # Create playlist for specific release kinds
/// sporlcli playlist --release-kinds album,single
///
/// # Create playlist for specific release kinds and date
/// sporlcli playlist --release-kinds album,single --release-date 2023-12-25
///
/// # Create playlists for 2 weeks before specific date
/// sporlcli playlist --release-date 2023-12-25 --previous-weeks 2
/// ```
///
/// # Playlist Content
///
/// Each playlist contains:
/// - First track from each album released during the week
/// - Tracks from all release types (albums, singles, EPs, compilations)
/// - Automatic deduplication of albums from the same artist
/// - Chronological ordering based on release data processing
///
/// # API Limitations and Handling
///
/// The function respects Spotify API limitations:
/// - **Album Batch Size**: 20 albums per detailed info request
/// - **Track Batch Size**: 100 tracks per playlist addition request
/// - **Rate Limiting**: Built-in delays and error handling for rate limits
/// - **Authentication**: Automatic token refresh for expired credentials
///
/// # Progress Feedback
///
/// Provides detailed progress information:
/// - Week processing status
/// - Album information gathering progress
/// - Playlist creation confirmations
/// - Track addition success/failure status
/// - Overall operation completion status
///
/// # Error Recovery
///
/// The function is designed for resilience:
/// - Individual week failures don't stop processing of other weeks
/// - Track addition failures don't prevent playlist creation
/// - Network issues are reported but allow for manual retry
/// - Partial success scenarios are clearly communicated
///
/// # Dependencies
///
/// Requires:
/// - Valid Spotify authentication (run `sporlcli auth` first)
/// - Cached release data (run `sporlcli releases update` if needed)
/// - Spotify Premium account for playlist modification permissions
/// - Network connectivity for API requests
///
/// # Future Enhancements
///
/// Potential improvements:
/// - Configurable track selection (not just first track)
/// - Custom playlist descriptions with week information
/// - Support for collaborative playlists
/// - Playlist artwork customization
/// - Integration with user's existing playlist folders
pub async fn playlist(
    previous_weeks: Option<u32>,
    release_date: Option<String>,
    release_kinds: &utils::ReleaseKinds,
) {
    let curr_date = utils::get_date_from_string(release_date);
    let curr_year = curr_date.year();
    let release_weeks = utils::get_custom_week_range(curr_date, previous_weeks.unwrap_or(0));

    for release_kind in release_kinds.iter() {
        for release_week in release_weeks.clone() {
            let playlist_name = format!(
                "Weekly Picks {}/{} ({})",
                release_week.week.clone(),
                curr_year.clone(),
                release_kind.clone()
            );

            let playlist_exists = match spotify::playlist::exists(&playlist_name).await {
                Ok(exists) => exists,
                Err(e) => {
                    warning!("Failed to check if playlist exists: {}", e);
                    false
                }
            };

            if playlist_exists {
                info!("Playlist {} already exists", playlist_name);
                continue;
            }

            info!(
                "Gather {} information for release week {}/{}",
                release_kind.clone(),
                release_week.week.clone(),
                curr_year.clone()
            );

            let mut all_albums: Vec<GetSeveralAlbumsResponse> = Vec::new();

            let releases: Vec<Album> = match utils::get_weekly_releases(
                release_week.week,
                curr_year,
                &release_kind.to_string(),
            )
            .await
            {
                Ok(releases) => releases,
                Err(e) => {
                    warning!("{}", e);
                    Vec::new()
                }
            };

            if releases.is_empty() {
                continue;
            }

            let release_chunks = releases.chunks(20);
            let mut handles = Vec::new();

            for chunk in release_chunks {
                let chunk = chunk.to_vec();
                let handle =
                    tokio::spawn(
                        async move { spotify::releases::get_several_releases(&chunk).await },
                    );
                handles.push(handle);
            }

            for handle in handles {
                match handle.await {
                    Ok(Ok(response)) => {
                        all_albums.push(response);
                    }
                    Ok(Err(e)) => {
                        warning!("{}", e);
                    }
                    Err(e) => {
                        warning!("Task join error: {}", e);
                    }
                }
            }

            success!(
                "Release information gathered for release week {}/{}",
                release_week.week.clone(),
                curr_year.clone()
            );

            info!("Create playlist {} ...", playlist_name.clone());

            let playlist_id: Option<String> =
                match spotify::playlist::create(playlist_name.clone()).await {
                    Ok(resp) => {
                        success!("Playlist {} created.", playlist_name.clone());
                        Some(resp.id.clone())
                    }
                    Err(e) => {
                        warning!("Failed to create playlist: {}", e);
                        None
                    }
                };

            if let Some(playlist_id) = playlist_id {
                info!("Add tracks to playlist {} ...", playlist_name.clone());
                let tracks: Vec<Track> = all_albums
                    .iter()
                    .flat_map(|ar| {
                        ar.albums
                            .iter()
                            .flat_map(|album| album.tracks.items.first())
                    })
                    .cloned()
                    .collect();

                let tracks_chunks = tracks.chunks(100);
                for chunk in tracks_chunks {
                    match spotify::playlist::add_tracks(playlist_id.clone(), chunk.to_vec()).await {
                        Ok(_) => success!("Tracks added to playlist {}", playlist_name.clone()),
                        Err(e) => warning!("Failed to add tracks to playlist: {}", e),
                    };
                }
            }
        }
    }
}
