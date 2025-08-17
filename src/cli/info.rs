use chrono::Utc;

use crate::{error, info, management::ArtistReleaseManager, spotify, utils, warning};

/// Internal structure for holding release week information.
///
/// Contains essential information about a release week including the week number
/// and a formatted string representation of the date range. This structure is
/// used internally to pass week information between functions.
struct ReleaseWeekInfo {
    /// Week number within the year (1-52/53)
    week: u32,
    /// Formatted date range string (e.g., "2023-10-14 - 2023-10-20")
    dates: String,
}

/// Displays various types of information about the application state and data.
///
/// Provides a unified CLI interface for querying different types of information
/// including release weeks, artist statistics, historical week data, and date
/// lookups. The function accepts multiple boolean flags to determine what
/// information to display.
///
/// # Arguments
///
/// * `release_week` - Display current release week information
/// * `artists` - Display artist count statistics (cache vs remote)
/// * `previous_weeks` - Number of previous weeks to display information for
/// * `release_date` - Specific date to lookup release week information
///
/// # Information Types
///
/// ## Release Week (`--release-week`)
/// Shows information about the current release week:
/// - Week number within the current year
/// - Date range covered by the week (Saturday to Friday)
///
/// ## Artist Statistics (`--artists`)
/// Compares local cache with remote Spotify data:
/// - Number of artists in local cache
/// - Number of artists currently followed on Spotify
/// - Warning if cache is outdated
///
/// ## Previous Weeks (`--previous-weeks N`)
/// Displays week information for the last N weeks:
/// - Week numbers and date ranges
/// - Useful for understanding recent release periods
///
/// ## Date Lookup (`--release-date YYYY-MM-DD`)
/// Shows which release week a specific date falls into:
/// - Converts any date to its corresponding release week number
///
/// # Execution Priority
///
/// The function executes in priority order and returns after the first match:
/// 1. Release week information (if `release_week` is true)
/// 2. Artist statistics (if `artists` is true)
/// 3. Previous weeks information (if `previous_weeks` is provided)
/// 4. Date lookup (if `release_date` is provided)
///
/// # Error Handling
///
/// Different information types have different error handling:
/// - **Release week errors**: Terminate with error message
/// - **Artist cache failures**: Default to 0 count with warning
/// - **API failures**: Default to 0 count, may show warning
/// - **Date parsing errors**: Use current date as fallback
///
/// # Example Usage
///
/// ```bash
/// # Show current release week
/// sporlcli info --release-week
///
/// # Show artist statistics
/// sporlcli info --artists
///
/// # Show last 4 weeks
/// sporlcli info --previous-weeks 4
///
/// # Look up specific date
/// sporlcli info --release-date 2023-12-25
/// ```
///
/// # Output Examples
///
/// **Release Week:**
/// ```
/// [o] Current release week: 42
/// [o] Current release week dates: 2023-10-14 - 2023-10-20
/// ```
///
/// **Artist Statistics:**
/// ```
/// [o] Artist count cache: 150
/// [o] Artist count remote: 152
/// [!] Artist count cache is outdated by 2.
/// ```
///
/// **Previous Weeks:**
/// ```
/// [o] Release week: 40
/// [o] Release week dates: 2023-09-30 - 2023-10-06
/// [o] Release week: 41
/// [o] Release week dates: 2023-10-07 - 2023-10-13
/// ```
///
/// **Date Lookup:**
/// ```
/// [o] 2023-12-25 is in release week 52.
/// ```
///
/// # Use Cases
///
/// - **Release Planning**: Understanding current and upcoming release periods
/// - **Cache Monitoring**: Checking if artist data needs updating
/// - **Historical Analysis**: Reviewing past release weeks
/// - **Date Conversion**: Converting calendar dates to release week numbers
/// - **Debugging**: Verifying week calculations and date ranges
pub async fn info(
    release_week: bool,
    artists: bool,
    previous_weeks: Option<u32>,
    release_date: Option<String>,
) {
    if release_week {
        let info: ReleaseWeekInfo = match current_release_week().await {
            Ok(info) => info,
            Err(err) => error!("Error fetching release week info: {}", err),
        };

        info!("Current release week: {}", info.week);
        info!("Current release week dates: {}", info.dates);
        return;
    }

    if artists {
        let artist_cache_count = match ArtistReleaseManager::load().await {
            Ok(arm) => arm.count_artists().clone() as u64,
            Err(_) => 0,
        };

        let artist_remote_count = match spotify::artists::get_total_artist_count().await {
            Ok(c) => c,
            Err(_) => 0,
        };

        info!("Artist count cache: {}", artist_cache_count);
        info!("Artist count remote: {}", artist_remote_count);
        if artist_cache_count < artist_remote_count {
            warning!(
                "Artist count cache is outdated by {}.",
                artist_remote_count - artist_cache_count
            );
        }

        return;
    }

    if let Some(previous_weeks) = previous_weeks {
        let curr_date = Utc::now().date_naive();
        let release_weeks = utils::get_custom_week_range(curr_date, previous_weeks);

        for release_week in release_weeks {
            info!("Release week: {}", release_week.week);
            info!(
                "Release week dates: {} - {}",
                release_week
                    .dates
                    .first()
                    .unwrap_or(&Utc::now().date_naive()),
                release_week
                    .dates
                    .last()
                    .unwrap_or(&Utc::now().date_naive())
            );
        }
        return;
    }

    if let Some(release_date_str) = release_date {
        let release_date = utils::get_date_from_string(Some(release_date_str));
        let release_week = utils::get_release_week_number(release_date);
        info!("{} is in release week {}.", release_date, release_week);
    }
}

/// Retrieves information about the current release week.
///
/// Internal helper function that calculates the current release week based on
/// today's date and formats the information for display. Uses the application's
/// week calculation logic to determine week numbers and date ranges.
///
/// # Returns
///
/// Returns a `Result` containing:
/// - `Ok(ReleaseWeekInfo)` - Current week information with number and date range
/// - `Err(String)` - Error message if week calculation fails
///
/// # Week Calculation
///
/// The function uses the application's standard week calculation:
/// - Weeks start on Saturday and end on Friday
/// - Week 1 begins on the Saturday before or on January 1st
/// - Week numbers are 1-based and typically range from 1-52 (sometimes 53)
///
/// # Date Range Formatting
///
/// The date range is formatted as "YYYY-MM-DD - YYYY-MM-DD" showing the
/// full week span from Saturday through Friday. This provides clear
/// visibility into which dates fall within the current release week.
///
/// # Error Conditions
///
/// While this function currently always succeeds, it returns a Result to
/// allow for future error conditions such as:
/// - Invalid system date/time
/// - Date calculation overflow
/// - Calendar system issues
///
/// # Example
///
/// ```
/// let week_info = current_release_week().await?;
/// println!("Week {}: {}", week_info.week, week_info.dates);
/// // Output: Week 42: 2023-10-14 - 2023-10-20
/// ```
///
/// # Internal Usage
///
/// This function is called internally by the `info()` function when the
/// `--release-week` flag is used. It encapsulates the logic for current
/// week calculation and formatting.
///
/// # Date Handling
///
/// Uses UTC time for consistency and to avoid timezone-related issues
/// when determining the current week. The week calculation is based on
/// the current UTC date, ensuring consistent behavior regardless of
/// the user's local timezone.
async fn current_release_week() -> Result<ReleaseWeekInfo, String> {
    let curr_date = Utc::now().date_naive();
    let release_week = utils::build_week(curr_date);
    Ok(ReleaseWeekInfo {
        week: release_week.week.clone(),
        dates: format!(
            "{} - {}",
            release_week
                .dates
                .first()
                .unwrap_or(&Utc::now().date_naive()),
            release_week
                .dates
                .last()
                .unwrap_or(&Utc::now().date_naive())
        ),
    })
}
