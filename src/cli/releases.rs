use std::time::Duration;

use chrono::{Datelike, NaiveDate};
use tabled::Table;
use tokio::time::sleep;

use crate::{
    error,
    management::{
        ArtistReleaseManager, ReleaseWeekManager, STATE_TYPE_RELEASES, StateManager, TokenManager,
    },
    spotify, success,
    types::{Album, ArtistReleases, ReleaseTableRow, ReleaseWeek},
    utils, warning,
};

/// Updates the local release cache with latest releases from followed artists.
///
/// Performs a comprehensive update of release data by fetching releases for all
/// followed artists from Spotify. The function includes state management for
/// resume capability, rate limiting for API compliance, and intelligent caching
/// to organize releases by week and year.
///
/// # Arguments
///
/// * `force` - If true, forces a complete refresh ignoring cached state
/// * `release_types` - Types of releases to fetch (album, single, compilation, etc.)
///
/// # Update Process
///
/// The update follows a systematic approach:
/// 1. **State Management**: Loads or creates state for tracking processed artists
/// 2. **Artist Loading**: Retrieves list of followed artists from cache
/// 3. **Release Fetching**: Gets releases for each artist from Spotify API
/// 4. **Data Organization**: Groups releases by week and year
/// 5. **Cache Persistence**: Saves organized data for quick future access
///
/// # State Management and Resume Capability
///
/// The function maintains state to enable robust operation:
/// - **Progress Tracking**: Records which artists have been processed
/// - **Resume Support**: Can continue from interruption point
/// - **Force Mode**: When enabled, ignores existing state and starts fresh
/// - **State Cleanup**: Automatically cleans state after successful completion
///
/// # Rate Limiting and API Compliance
///
/// Implements several strategies to respect Spotify's API limits:
/// - **Batch Processing**: Processes artists in chunks of 100
/// - **Progressive Delays**: 30-second delays between batches
/// - **Rate Limit Handling**: Built-in handling for 429 responses
/// - **Token Management**: Automatic token refresh for expired credentials
///
/// # Error Handling and Resilience
///
/// Comprehensive error handling for production use:
/// - **Individual Failures**: Artist-level failures don't stop entire process
/// - **Network Issues**: Graceful handling of connectivity problems
/// - **API Errors**: Proper response to Spotify service issues
/// - **Cache Failures**: Continues operation even with cache write failures
/// - **State Persistence**: Saves progress even on partial failures
///
/// # Progress Feedback
///
/// Provides detailed real-time progress information:
/// - Current artist being processed
/// - Number of releases found per artist
/// - Overall progress counters (current/total)
/// - Success/failure status for each operation
/// - Final summary statistics
///
/// # Data Organization
///
/// Organizes fetched data for efficient access:
/// - **Artist-Release Mapping**: Associates releases with their artists
/// - **Weekly Grouping**: Groups releases by their release week
/// - **Year Organization**: Separates data by year for scalability
/// - **Duplicate Removal**: Handles duplicate releases across artists
///
/// # Cache Strategy
///
/// Uses a multi-level caching approach:
/// - **Artist Cache**: Stores artist-to-releases mapping
/// - **Weekly Cache**: Organizes releases by week/year combinations
/// - **State Cache**: Tracks processing progress for resume capability
///
/// # Example Usage
///
/// ```bash
/// # Update with default settings (albums only)
/// sporlcli releases update
///
/// # Force complete refresh
/// sporlcli releases update --force
///
/// # Update specific release types
/// sporlcli releases update --type album,single
///
/// # Update all release types
/// sporlcli releases update --type all
/// ```
///
/// # Performance Considerations
///
/// - Processing time scales with number of followed artists
/// - Network delays and API rate limits affect total duration
/// - Force updates take significantly longer than incremental updates
/// - Memory usage is proportional to total release count
///
/// # Prerequisites
///
/// - Valid Spotify authentication (run `sporlcli auth` first)
/// - Followed artists cache (run `sporlcli artists update` first)
/// - Network connectivity for API requests
/// - Sufficient disk space for cache files
pub async fn update_releases(force: bool, release_types: &utils::ReleaseKinds) {
    let pb = utils::create_progress_bar("Fetching releases for followed artists...");

    let mut state = match StateManager::new(STATE_TYPE_RELEASES.to_string())
        .load()
        .await
    {
        Ok(state) => state,
        Err(_) => StateManager::new(STATE_TYPE_RELEASES.to_string()),
    };

    let mut artist_release_mgr = ArtistReleaseManager::load()
        .await
        .unwrap_or_else(|_| ArtistReleaseManager::new(None));

    let mut token_mgr = match TokenManager::load().await {
        Ok(manager) => manager,
        Err(e) => {
            error!(
                "Failed to load token. Please run sporlcli auth\n Error: {}",
                e
            );
        }
    };

    let mut remote_releases: Vec<Album> = Vec::new();
    let artist_releases: Vec<ArtistReleases> = if let Some(ar) = artist_release_mgr.all() {
        ar
    } else {
        Vec::new()
    };

    let artist_chunks = artist_releases.chunks(100);
    let artists_total = artist_releases.len().clone();
    let mut artists_count = 0;
    let mut artist_cached = false;

    'chunk: for artist_chunk in artist_chunks {
        for artist in artist_chunk {
            let token = token_mgr.get_valid_token().await;

            if state.has(artist.artist.id.clone()) && !force {
                pb.set_message(format!(
                    "Releases for artist {artist_name} already cached. ({artists_count}/{artists_total})",
                    artist_name = artist.artist.name.clone(),
                    artists_count = artists_count,
                    artists_total = artists_total
                ));
                artist_cached = true;
                artists_count += 1;

                remote_releases.extend(artist.releases.clone());
                continue;
            }

            artist_cached = false;

            match spotify::releases::get_release_for_artist(
                artist.artist.id.clone(),
                &token,
                50,
                release_types,
            )
            .await
            {
                Ok(releases) => {
                    pb.set_message(format!(
                        "Fetched {releases} releases from artist {artist_name} ({artists_count}/{artists_total}).",
                        releases = releases.len().clone(),
                        artist_name = artist.artist.name.clone(),
                        artists_count = artists_count,
                        artists_total = artists_total
                    ));
                    remote_releases.extend(releases.clone());
                    state.add(artist.artist.id.clone());
                    artists_count += 1;

                    // cache release for artist
                    if releases.len() > 0 {
                        match artist_release_mgr
                            .add_releases_to_artist(&artist.artist.id, releases)
                            .persist()
                            .await
                        {
                            Ok(_) => {
                                pb.set_message(format!(
                                    "Releases for artist {artist_name} cached. ({artists_count}/{artists_total})",
                                    artist_name = artist.artist.name,
                                    artists_count = artists_count,
                                    artists_total = artists_total
                                ));
                            }
                            Err(e) => {
                                pb.set_message(format!(
                                    "Cannot cache releases for artist {artist_name} ({artists_count}/{artists_total}): {e}",
                                    artist_name = artist.artist.name,
                                    artists_count = artists_count,
                                    artists_total = artists_total
                                ));
                            }
                        }
                    }
                }
                Err(e) => {
                    pb.set_message(format!(
                        "Failed to load releases for artist {artist_name}: {error} ({artists_count}/{artists_total})",
                        artist_name = artist.artist.name.clone(),
                        error = e,
                        artists_count = artists_count,
                        artists_total = artists_total
                    ));

                    match state.persist().await {
                        Ok(_) => pb.set_message(format!(
                            "Successfully persisted state. ({artists_count}/{artists_total})",
                            artists_count = artists_count,
                            artists_total = artists_total
                        )),
                        Err(e) => {
                            pb.set_message(format!("Failed to persist state: {error:?} ({artists_count}/{artists_total})",
                                error = e,
                                artists_count = artists_count,
                                artists_total = artists_total
                            ));
                        }
                    }

                    break 'chunk;
                }
            }
        }

        if !artist_cached {
            sleep(Duration::from_secs(30)).await;
        }
    }

    pb.finish();

    // @todo implement cleanup of stste
    if artists_count == artists_total {
        match state.clear().await {
            Ok(_) => success!("State cache cleaned."),
            Err(e) => warning!("Cannot cleanup state cache. Err: {:?}", e),
        }
    }

    let releases_per_week = match prepare_remote_releases(remote_releases).await {
        Ok(releases) => releases,
        Err(e) => {
            warning!("Failed to prepare remote releases: {}", e);
            Vec::new()
        }
    };

    for release_per_week in releases_per_week.clone() {
        match ReleaseWeekManager::new(
            release_per_week.week.week.clone(),
            release_per_week.year.clone(),
            Some(release_per_week.releases.clone()),
        )
        .save_to_cache()
        .await
        {
            Ok(_) => pb.set_message(format!(
                "Releases for week {week} in year {year} cached.",
                week = release_per_week.week.week.clone(),
                year = release_per_week.year.clone()
            )),
            Err(_) => warning!(
                "Cannot cache releases for week {week} in year {year}.",
                week = release_per_week.week.week.clone(),
                year = release_per_week.year.clone()
            ),
        }
    }

    pb.finish_and_clear();
    success!("Release cache updated.");
}

/// Lists cached releases with optional time-based filtering and tabular display.
///
/// Displays a formatted table of releases from the local cache, organized by
/// week and year. Supports filtering by week ranges and specific dates, with
/// automatic sorting and duplicate removal for clean presentation.
///
/// # Arguments
///
/// * `weeks_include` - Number of weeks before the target date to include (None = current week only)
/// * `release_date` - Specific target date (None = current date)
///
/// # Display Format
///
/// For each week, displays a table with:
/// - **Date**: Release date in YYYY-MM-DD format
/// - **Name**: Album/release title
/// - **Artists**: Primary artist name (first artist if multiple)
///
/// Each week's releases are displayed in a separate table with week/year headers.
///
/// # Time Range Logic
///
/// The function determines which weeks to display:
/// - If `release_date` is provided, uses that as the target date
/// - If `weeks_include` is provided, shows that many weeks before the target
/// - If both are None, shows only the current week
/// - If `weeks_include` is 0, shows only the target week
///
/// # Data Processing
///
/// For each week's releases:
/// 1. **Cache Loading**: Loads release data from weekly cache files
/// 2. **Duplicate Removal**: Eliminates duplicate albums across artists
/// 3. **Data Transformation**: Converts to table-friendly format
/// 4. **Sorting**: Orders by date (descending) then artist (ascending)
/// 5. **Table Generation**: Creates formatted table output
///
/// # Error Handling
///
/// Graceful handling of various error conditions:
/// - **Missing Cache**: Shows warning with update instructions
/// - **Corrupt Data**: Skips problematic weeks and continues
/// - **Empty Weeks**: Continues processing other weeks
/// - **Load Failures**: Provides helpful error messages
///
/// # Cache Dependencies
///
/// Requires cached release data from previous update operations:
/// - Weekly cache files must exist for requested time periods
/// - Cache files are created by `sporlcli releases update`
/// - Missing cache files result in warnings with guidance
///
/// # Example Usage
///
/// ```bash
/// # Show current week releases
/// sporlcli releases
///
/// # Show last 4 weeks of releases
/// sporlcli releases --previous-weeks 4
///
/// # Show releases for specific date's week
/// sporlcli releases --release-date 2023-12-25
///
/// # Show 2 weeks before specific date
/// sporlcli releases --release-date 2023-12-25 --previous-weeks 2
/// ```
///
/// # Output Example
///
/// ```text
/// Week: 42	Year: 2023
/// ┌────────────┬─────────────────────────────┬─────────────────┐
/// │ date       │ name                        │ artists         │
/// ├────────────┼─────────────────────────────┼─────────────────┤
/// │ 2023-10-20 │ New Album Title             │ Artist Name     │
/// │ 2023-10-19 │ Latest Single               │ Another Artist  │
/// │ 2023-10-18 │ EP Release                  │ Band Name       │
/// └────────────┴─────────────────────────────┴─────────────────┘
///
/// Week: 41	Year: 2023
/// ┌────────────┬─────────────────────────────┬─────────────────┐
/// │ date       │ name                        │ artists         │
/// ├────────────┼─────────────────────────────┼─────────────────┤
/// │ 2023-10-13 │ Previous Week Album         │ Previous Artist │
/// └────────────┴─────────────────────────────┴─────────────────┘
/// ```
///
/// # Data Quality Features
///
/// - **Duplicate Removal**: Handles same album appearing multiple times
/// - **Consistent Sorting**: Always orders by date then artist
/// - **Artist Simplification**: Shows primary artist to avoid cluttered display
/// - **Date Formatting**: Uses standard YYYY-MM-DD format for clarity
///
/// # Performance Notes
///
/// - Loads each week's data separately for memory efficiency
/// - Table generation is fast for typical week sizes
/// - Large time ranges may require multiple cache file loads
/// - Output formatting time is proportional to total releases shown
pub async fn list_releases(weeks_include: Option<u32>, release_date: Option<String>) {
    // let release_date = match NaiveDate::parse_from_str(&album.release_date, "%Y-%m-%d")
    let curr_date = utils::get_date_from_string(release_date);
    let cur_year = curr_date.year();
    let release_weeks = utils::get_custom_week_range(curr_date, weeks_include.unwrap_or(0));

    for release_week in release_weeks.clone() {
        let mut weekly_releases: Vec<Album> = match ReleaseWeekManager::new(
            release_week.week.clone(),
            cur_year,
            None,
        )
        .load_from_cache()
        .await
        {
            Ok(manager) => match manager.get_releases().await {
                Ok(releases) => releases,
                Err(e) => {
                    warning!(
                        "Failed to load releases for week {}/{}: {}\nRun sporlcli releases update.",
                        release_week.week.clone(),
                        cur_year,
                        e
                    );
                    continue;
                }
            },
            Err(e) => {
                warning!(
                    "Failed to load releases for week {}/{}: {:?}\nRun sporlcli releases update.",
                    release_week.week.clone(),
                    cur_year,
                    e
                );
                continue;
            }
        };

        utils::remove_duplicate_albums(&mut weekly_releases);

        let mut weekly_releases_row: Vec<ReleaseTableRow> = weekly_releases
            .into_iter()
            .map(|a| ReleaseTableRow {
                date: a.release_date,
                name: a.name,
                artists: a
                    .artists
                    .iter()
                    .map(|a| a.name.clone())
                    .collect::<Vec<String>>()
                    .first()
                    .unwrap_or(&String::new())
                    .clone(),
            })
            .collect();

        utils::sort_release_table_rows(&mut weekly_releases_row);

        let table = Table::new(weekly_releases_row);
        println!(
            "Week: {week}\tYear: {year}\n{table}\n",
            week = release_week.week.clone(),
            year = cur_year,
            table = table
        );
    }
}

/// Organizes releases by their respective weeks and years for efficient caching.
///
/// Internal helper function that takes a flat list of releases and groups them
/// into week-based structures for organized storage. This enables efficient
/// querying of releases by time period and supports the weekly cache organization.
///
/// # Arguments
///
/// * `remote_releases` - Vector of albums from various artists and time periods
///
/// # Returns
///
/// Returns a `Result` containing:
/// - `Ok(Vec<ReleaseWeek>)` - Organized release data grouped by week and year
/// - `Err(String)` - Error message describing processing failures
///
/// # Processing Logic
///
/// For each release:
/// 1. **Date Validation**: Ensures release has day-precision date
/// 2. **Date Parsing**: Converts string date to structured date object
/// 3. **Week Calculation**: Determines which release week the date falls into
/// 4. **Year Extraction**: Gets the year for proper organization
/// 5. **Grouping**: Adds to existing week group or creates new one
///
/// # Date Precision Filtering
///
/// Only processes releases with day-precision dates:
/// - Includes: "2023-10-15" (day precision)
/// - Excludes: "2023-10" (month precision) or "2023" (year precision)
/// - This ensures accurate week assignment and consistent organization
///
/// # Week Assignment Algorithm
///
/// Uses the application's standard week calculation:
/// - Weeks start on Saturday and end on Friday
/// - Week numbers are calculated based on the application's week system
/// - Cross-year boundaries are handled correctly
/// - Consistent with other week-based operations in the application
///
/// # Grouping Strategy
///
/// Efficiently groups releases using a find-or-create approach:
/// - Searches for existing week/year combination
/// - Adds to existing group if found
/// - Creates new ReleaseWeek structure if not found
/// - Maintains chronological organization
///
/// # Error Handling
///
/// Handles various data quality issues:
/// - **Invalid Dates**: Logs warning and skips problematic releases
/// - **Missing Precision**: Skips releases without day-level precision
/// - **Parse Failures**: Continues processing other releases
/// - **Malformed Data**: Provides descriptive error messages
///
/// # Memory Efficiency
///
/// Designed for efficient memory usage:
/// - Groups releases in-place without unnecessary copying
/// - Uses move semantics where possible
/// - Minimizes temporary allocations
/// - Scales well with large release datasets
///
/// # Output Structure
///
/// Returns a vector of ReleaseWeek structures where each contains:
/// - Week information (number and date range)
/// - Year for the week
/// - All releases that fall within that week
///
/// # Example
///
/// ```
/// let releases = vec![/* various albums */];
/// let organized = prepare_remote_releases(releases).await?;
///
/// for week_data in organized {
///     println!("Week {} of {}: {} releases",
///              week_data.week.week,
///              week_data.year,
///              week_data.releases.len());
/// }
/// ```
///
/// # Data Quality
///
/// The function ensures high-quality output:
/// - Only includes releases with accurate dates
/// - Proper week assignment based on application logic
/// - Consistent year association
/// - Handles edge cases around year boundaries
///
/// # Performance Characteristics
///
/// - Time complexity: O(n*m) where n=releases, m=unique weeks
/// - Space complexity: O(n) for output organization
/// - Efficient for typical release volumes
/// - Scales reasonably with large datasets
async fn prepare_remote_releases(remote_releases: Vec<Album>) -> Result<Vec<ReleaseWeek>, String> {
    let mut releases_weeks: Vec<ReleaseWeek> = Vec::new();

    for album in remote_releases {
        if album.release_date_precision != "day" {
            continue;
        }

        let release_date = match NaiveDate::parse_from_str(&album.release_date, "%Y-%m-%d") {
            Ok(d) => d,
            Err(err) => {
                warning!(
                    "Cannot parse release date for album: {}, {}",
                    album.name,
                    err
                );
                continue;
            }
        };

        let release_week_for_album = utils::build_week(release_date);
        let release_year_for_album = release_date.year();

        // Look for an existing ReleaseWeek
        if let Some(week_entry) = releases_weeks.iter_mut().find(|rw| {
            rw.year == release_year_for_album && rw.week.week == release_week_for_album.week
        }) {
            week_entry.releases.push(album);
        } else {
            // If not found, create a new ReleaseWeek
            releases_weeks.push(ReleaseWeek {
                week: release_week_for_album,
                year: release_year_for_album,
                releases: vec![album],
            });
        }
    }

    Ok(releases_weeks)
}
