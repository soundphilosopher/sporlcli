//! Utility functions and types for the Sporl music release tracking application.
//!
//! This module provides essential utilities for OAuth authentication, date/time handling,
//! release data processing, and release type management. It serves as the core utility
//! layer for the Sporl application, offering both public APIs for application features
//! and internal helper functions.
//!
//! # Key Features
//!
//! ## OAuth PKCE Support
//! - Code verifier and challenge generation for secure OAuth flows
//! - Compliant with RFC 7636 (Proof Key for Code Exchange)
//!
//! ## Release Week Management
//! - Custom week numbering system (Saturday to Friday)
//! - Week range calculations and date utilities
//! - Consistent week-based release tracking
//!
//! ## Release Data Processing
//! - Album deduplication and sorting utilities
//! - Release table row management
//! - Integration with cached release data
//!
//! ## Release Type System
//! - Comprehensive release kind enumeration and validation
//! - Command-line argument parsing for release types
//! - Flexible filtering system for different music release categories
//!
//! # Usage Examples
//!
//! ## OAuth Authentication
//! ```rust,no_run
//! use sporl::utils::{generate_code_verifier, generate_code_challenge};
//!
//! // Generate PKCE code verifier and challenge
//! let verifier = generate_code_verifier();
//! let challenge = generate_code_challenge(&verifier);
//! ```
//!
//! ## Working with Release Weeks
//! ```rust,no_run
//! use sporl::utils::{build_week, get_release_week_number};
//! use chrono::NaiveDate;
//!
//! // Get the week structure for a specific date
//! let date = NaiveDate::from_ymd_opt(2023, 10, 17).unwrap();
//! let week = build_week(date);
//! let week_number = get_release_week_number(date);
//! ```
//!
//! ## Processing Release Data
//! ```rust,no_run
//! use sporl::utils::{get_weekly_releases, remove_duplicate_albums};
//!
//! // Get and process weekly releases
//! let mut albums = get_weekly_releases(42, 2023).await?;
//! remove_duplicate_albums(&mut albums);
//! ```
//!
//! ## Release Type Filtering
//! ```rust,no_run
//! use sporl::utils::{parse_release_kinds, ReleaseKind};
//!
//! // Parse release kinds from user input
//! let kinds = parse_release_kinds("album,single")?;
//! for kind in kinds.iter() {
//!     println!("Processing: {}", kind);
//! }
//! ```

use std::{
    cmp::Ordering,
    collections::{BTreeSet, HashSet},
    fmt,
};

use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::{Datelike, Duration, NaiveDate, Utc, Weekday};
use rand::{Rng, distr::Alphanumeric};
use sha2::{Digest, Sha256};

use clap::ValueEnum;

use crate::{
    management::ReleaseWeekManager,
    types::{Album, ReleaseTableRow, WeekOfTheYear},
};

/// Generates a random code verifier for OAuth PKCE (Proof Key for Code Exchange).
///
/// Creates a 128-character random string using alphanumeric characters.
/// This is used as the code verifier in the OAuth PKCE flow for secure authentication.
///
/// # Returns
///
/// A `String` containing 128 random alphanumeric characters.
///
/// # Example
///
/// ```
/// let verifier = generate_code_verifier();
/// assert_eq!(verifier.len(), 128);
/// ```
pub fn generate_code_verifier() -> String {
    rand::rng()
        .sample_iter(&Alphanumeric)
        .take(128)
        .map(char::from)
        .collect()
}

/// Generates a code challenge from a code verifier for OAuth PKCE.
///
/// Takes a code verifier string, computes its SHA256 hash, and encodes it using
/// URL-safe base64 encoding without padding. This challenge is sent to the
/// authorization server during the OAuth flow.
///
/// # Arguments
///
/// * `verifier` - The code verifier string to hash and encode
///
/// # Returns
///
/// A `String` containing the base64-encoded SHA256 hash of the verifier.
///
/// # Example
///
/// ```
/// let verifier = generate_code_verifier();
/// let challenge = generate_code_challenge(&verifier);
/// ```
pub fn generate_code_challenge(verifier: &str) -> String {
    let hash = Sha256::digest(verifier.as_bytes());
    URL_SAFE_NO_PAD.encode(hash)
}

/// Finds the Saturday that occurs before or on the given date.
///
/// This function is used to determine the start of a week, where weeks are
/// defined as starting on Saturday. If the given date is already a Saturday,
/// it returns that date unchanged.
///
/// # Arguments
///
/// * `date` - The date to find the preceding/current Saturday for
///
/// # Returns
///
/// A `NaiveDate` representing the Saturday before or on the given date.
///
/// # Example
///
/// ```
/// let tuesday = NaiveDate::from_ymd_opt(2023, 10, 17).unwrap(); // Tuesday
/// let saturday = get_saturday_before_or_on(tuesday); // Returns Oct 14, 2023 (Saturday)
/// ```
pub(crate) fn get_saturday_before_or_on(date: NaiveDate) -> NaiveDate {
    let weekday = date.weekday().num_days_from_sunday(); // Sunday=0, Saturday=6
    let days_to_subtract = (weekday + 1) % 7; // How many days to go back to Saturday
    date - Duration::days(days_to_subtract as i64)
}

/// Calculates the release week number for a given date within its year.
///
/// Weeks are numbered starting from 1, where week 1 begins on the Saturday
/// before or on January 1st. This creates a consistent weekly numbering system
/// for tracking music releases throughout the year.
///
/// # Arguments
///
/// * `date` - The date to calculate the week number for
///
/// # Returns
///
/// A `u32` representing the week number (1-based) within the year.
///
/// # Example
///
/// ```
/// let date = NaiveDate::from_ymd_opt(2023, 1, 15).unwrap();
/// let week_num = get_release_week_number(date); // Returns the week number for Jan 15, 2023
/// ```
pub fn get_release_week_number(date: NaiveDate) -> u32 {
    let current_week_start = get_saturday_before_or_on(date);

    // Determine which year's scheme applies (previous year if the week-start Saturday is in the prev year)
    let anchor_year = if current_week_start.year() < date.year() {
        current_week_start.year()
    } else {
        date.year()
    };

    let jan1 = NaiveDate::from_ymd_opt(anchor_year, 1, 1).unwrap();
    let first_week_start = get_saturday_before_or_on(jan1);

    let diff_weeks = ((current_week_start - first_week_start).num_days() / 7) as u32;

    // If Jan 1 is Saturday *for the anchor year*, shift to 1-based (avoids week 0 in those years)
    let jan1_is_sat = jan1.weekday() == Weekday::Sat;
    diff_weeks + if jan1_is_sat { 1 } else { 0 }
}

/// Builds a complete week structure starting from the Saturday before or on the given date.
///
/// Creates a `WeekOfTheYear` struct containing the week number and all seven dates
/// in that week (Saturday through Friday). This provides a complete representation
/// of a release week.
///
/// # Arguments
///
/// * `date` - Any date within the week to build
///
/// # Returns
///
/// A `WeekOfTheYear` struct containing the week number and all dates in that week.
///
/// # Example
///
/// ```
/// let date = NaiveDate::from_ymd_opt(2023, 10, 17).unwrap(); // Tuesday
/// let week = build_week(date); // Returns week containing Oct 14-20, 2023
/// ```
pub fn build_week(date: NaiveDate) -> WeekOfTheYear {
    let saturday = get_saturday_before_or_on(date);
    let dates: Vec<NaiveDate> = (0..7).map(|i| saturday + Duration::days(i)).collect();

    let week_number = get_release_week_number(saturday);

    WeekOfTheYear {
        week: week_number,
        dates,
    }
}

/// Generates a range of weeks going back from a given date.
///
/// Creates a vector of `WeekOfTheYear` structures representing consecutive weeks
/// leading up to (and optionally including) the week containing the given date.
/// If the date falls on a Friday, the current week is included; otherwise it's skipped.
///
/// # Arguments
///
/// * `date` - The reference date to work backwards from
/// * `weeks_before` - The number of weeks to include in the range
///
/// # Returns
///
/// A `Vec<WeekOfTheYear>` containing the requested weeks in chronological order.
///
/// # Example
///
/// ```
/// let date = NaiveDate::from_ymd_opt(2023, 10, 17).unwrap();
/// let weeks = get_custom_week_range(date, 3); // Gets 3 weeks before current week
/// ```
pub fn get_custom_week_range(date: NaiveDate, weeks_before: u32) -> Vec<WeekOfTheYear> {
    let skip_current = date.weekday() != Weekday::Fri;

    let start_offset = if skip_current { 1 } else { 0 };

    (start_offset..=weeks_before + start_offset)
        .map(|i| {
            let target_date = date - Duration::days((i * 7) as i64);
            build_week(target_date)
        })
        .collect()
}

/// Removes duplicate albums from a vector based on their ID.
///
/// Modifies the input vector in-place, retaining only the first occurrence of each
/// unique album ID. This is useful for deduplicating album lists that might contain
/// the same album multiple times.
///
/// # Arguments
///
/// * `albums` - A mutable reference to a vector of albums to deduplicate
///
/// # Example
///
/// ```
/// let mut albums = vec![album1, album2, album1]; // album1 appears twice
/// remove_duplicate_albums(&mut albums); // Now contains only album1, album2
/// ```
pub fn remove_duplicate_albums(albums: &mut Vec<Album>) {
    let mut seen_ids = HashSet::new();
    albums.retain(|album| seen_ids.insert(album.id.clone()));
}

/// Sorts release table rows by date (descending) and then by artist name (ascending).
///
/// Modifies the input vector in-place to sort entries with the most recent releases first.
/// When multiple releases have the same date, they are sorted alphabetically by artist name.
/// This provides a consistent and useful ordering for displaying release information.
///
/// # Arguments
///
/// * `rows` - A mutable reference to a vector of release table rows to sort
///
/// # Example
///
/// ```
/// let mut rows = vec![row1, row2, row3];
/// sort_release_table_rows(&mut rows); // Sorted by date desc, then artist asc
/// ```
pub fn sort_release_table_rows(rows: &mut Vec<ReleaseTableRow>) {
    rows.sort_by(|a, b| {
        match b.date.cmp(&a.date) {
            Ordering::Equal => a.artists.cmp(&b.artists), // secondary sort: name ascending
            other => other,
        }
    });
}

/// Parses a date string or returns the current date if parsing fails or input is None.
///
/// Attempts to parse the input string using the format "%Y-%m-%d" (e.g., "2023-10-17").
/// If the input is None or parsing fails, returns the current UTC date. This provides
/// a robust way to handle date inputs with a sensible fallback.
///
/// # Arguments
///
/// * `date` - An optional string containing a date in YYYY-MM-DD format
///
/// # Returns
///
/// A `NaiveDate` representing either the parsed date or the current date.
///
/// # Example
///
/// ```
/// let date1 = get_date_from_string(Some("2023-10-17".to_string())); // Oct 17, 2023
/// let date2 = get_date_from_string(None); // Current date
/// let date3 = get_date_from_string(Some("invalid".to_string())); // Current date
/// ```
pub fn get_date_from_string(date: Option<String>) -> NaiveDate {
    match date {
        Some(date_str) => NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
            .unwrap_or_else(|_| Utc::now().date_naive()),
        None => Utc::now().date_naive(),
    }
}

/// Retrieves and processes weekly music releases for a specific week and year.
///
/// Loads release data from cache using a `ReleaseWeekManager`, then processes the results
/// by removing duplicates and sorting by date and artist. Returns a clean, sorted list
/// of albums released during the specified week.
///
/// # Arguments
///
/// * `week` - The week number within the year (1-based)
/// * `year` - The year to get releases for
///
/// # Returns
///
/// A `Result<Vec<Album>, String>` containing either the processed album list or an error message.
///
/// # Errors
///
/// Returns an error string if:
/// - Failed to load the release manager from cache
/// - Failed to retrieve releases from the manager
///
/// # Example
///
/// ```
/// let releases = get_weekly_releases(42, 2023).await?; // Week 42 of 2023
/// ```
pub async fn get_weekly_releases(week: u32, year: i32) -> Result<Vec<Album>, String> {
    let mut releases: Vec<Album> = match ReleaseWeekManager::new(week, year, None)
        .load_from_cache()
        .await
    {
        Ok(manager) => match manager.get_releases().await {
            Ok(releases) => releases,
            Err(e) => {
                return Err(format!(
                    "Failed to load releases for week {}/{}: {}\nRun sporlcli releases --update.",
                    week, year, e
                ));
            }
        },
        Err(e) => {
            return Err(format!(
                "Failed to load releases for week {}/{}: {:?}\nRun sporlcli releases --update.",
                week, year, e
            ));
        }
    };

    remove_duplicate_albums(&mut releases);
    sort_albums_by_date_and_artist(&mut releases);
    Ok(releases)
}

/// Sorts albums by release date (descending) and then by first artist name (ascending).
///
/// Modifies the input vector in-place to sort albums with the most recent releases first.
/// When multiple albums have the same release date, they are sorted alphabetically by
/// the name of the first artist. This provides consistent ordering for album displays.
///
/// # Arguments
///
/// * `albums` - A mutable reference to a vector of albums to sort
///
/// # Example
///
/// ```
/// let mut albums = vec![album1, album2, album3];
/// sort_albums_by_date_and_artist(&mut albums); // Sorted by date desc, then artist asc
/// ```
pub(crate) fn sort_albums_by_date_and_artist(albums: &mut Vec<Album>) {
    albums.sort_by(|a, b| {
        let date_cmp = b.release_date.cmp(&a.release_date);
        if date_cmp != Ordering::Equal {
            return date_cmp;
        }

        let a_artist = a.artists.get(0).map(|artist| artist.name.to_lowercase());
        let b_artist = b.artists.get(0).map(|artist| artist.name.to_lowercase());

        a_artist.cmp(&b_artist)
    });
}

/// The normalized set of release types supported by the application.
///
/// Represents the different categories of music releases that can be filtered
/// and processed. Each variant corresponds to a specific type of musical release
/// as defined by music platforms and industry standards.
///
/// # Variants
///
/// * `Album` - Full-length studio albums
/// * `Single` - Individual tracks or short releases
/// * `AppearsOn` - Tracks appearing on compilations or other artists' releases
/// * `Compilation` - Collection albums, greatest hits, etc.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, ValueEnum, Hash)]
#[value(rename_all = "snake_case")]
pub enum ReleaseKind {
    Album,
    Single,
    AppearsOn,
    Compilation,
}

impl ReleaseKind {
    pub const ALL: [ReleaseKind; 4] = [
        ReleaseKind::Album,
        ReleaseKind::Single,
        ReleaseKind::AppearsOn,
        ReleaseKind::Compilation,
    ];
}

/// A validated, deduplicated set of release kinds parsed from command-line input.
///
/// Wraps a `BTreeSet<ReleaseKind>` to ensure that release kinds are unique and
/// ordered consistently. Used to represent the user's selection of which types
/// of releases to include in operations.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReleaseKinds(pub BTreeSet<ReleaseKind>);

impl ReleaseKinds {
    /// Returns an iterator over the release kinds in this set.
    ///
    /// The iterator yields `ReleaseKind` values in sorted order due to the
    /// underlying `BTreeSet` structure.
    ///
    /// # Returns
    ///
    /// An iterator that yields `ReleaseKind` values.
    ///
    /// # Example
    ///
    /// ```
    /// for kind in release_kinds.iter() {
    ///     println!("Processing: {}", kind);
    /// }
    /// ```
    pub fn iter(&self) -> impl Iterator<Item = ReleaseKind> + '_ {
        self.0.iter().copied()
    }
}

impl Default for ReleaseKinds {
    /// Creates a default `ReleaseKinds` containing only the `Album` type.
    ///
    /// This provides a sensible default when no specific release types are specified,
    /// focusing on the most common type of music release.
    ///
    /// # Returns
    ///
    /// A `ReleaseKinds` instance containing only `ReleaseKind::Album`.
    fn default() -> Self {
        // Default value is "album"
        let mut set = BTreeSet::new();
        set.insert(ReleaseKind::Album);
        ReleaseKinds(set)
    }
}

impl fmt::Display for ReleaseKind {
    /// Formats a `ReleaseKind` using its canonical command-line representation.
    ///
    /// Uses the value defined in the `ValueEnum` derive to ensure consistency
    /// with command-line argument parsing. Returns the snake_case version of
    /// the variant name (e.g., "appears_on" for `AppearsOn`).
    ///
    /// # Arguments
    ///
    /// * `f` - The formatter to write to
    ///
    /// # Returns
    ///
    /// A `fmt::Result` indicating success or failure of the formatting operation.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = self
            .to_possible_value()
            .expect("ValueEnum should have a possible value")
            .get_name()
            .to_owned();
        f.write_str(&name)
    }
}

impl fmt::Display for ReleaseKinds {
    /// Formats a `ReleaseKinds` set as a comma-separated list of canonical names.
    ///
    /// Creates a string representation suitable for display or logging, showing
    /// all selected release kinds separated by commas (e.g., "album,single").
    ///
    /// # Arguments
    ///
    /// * `f` - The formatter to write to
    ///
    /// # Returns
    ///
    /// A `fmt::Result` indicating success or failure of the formatting operation.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut it = self.0.iter();
        if let Some(first) = it.next() {
            write!(f, "{}", first)?;
            for k in it {
                write!(f, ",{}", k)?;
            }
        }
        Ok(())
    }
}

/// Custom parser for release kinds from command-line input.
///
/// Parses a comma-separated string of release types, with support for special values
/// like "all". Validates each entry, normalizes formatting (handles hyphens/underscores),
/// and returns a deduplicated set of release kinds.
///
/// # Arguments
///
/// * `input` - A string containing comma-separated release kinds (e.g., "album,single" or "all")
///
/// # Returns
///
/// A `Result<ReleaseKinds, String>` containing either the parsed set or an error message.
///
/// # Errors
///
/// Returns an error string if:
/// - The input is empty or contains only whitespace
/// - Any segment contains invalid release kind names
/// - Malformed input (e.g., empty segments between commas)
///
/// # Special Values
///
/// * `"all"` - Expands to include all available release kinds
///
/// # Example
///
/// ```
/// let kinds1 = parse_release_kinds("album,single")?;
/// let kinds2 = parse_release_kinds("all")?;
/// let kinds3 = parse_release_kinds("album, appears-on")?; // Handles spaces and hyphens
/// ```
pub fn parse_release_kinds(input: &str) -> Result<ReleaseKinds, String> {
    let mut set: BTreeSet<ReleaseKind> = BTreeSet::new();

    // Empty string should be rejected (unless using default value)
    if input.trim().is_empty() {
        return Err("value for --type cannot be empty".into());
    }

    // Split by comma (user can also repeat the flag; we handle that at the clap layer if desired)
    for raw in input.split(',') {
        let part = raw.trim();
        if part.is_empty() {
            return Err("malformed --type: empty segment between commas".into());
        }

        // Allow lowercase inputs and hyphens/underscores (robust UX)
        let normalized = part.to_ascii_lowercase().replace('-', "_");

        if normalized == "all" {
            set.extend(ReleaseKind::ALL);
            continue;
        }

        // Let ValueEnum do the strict mapping so it's always in sync with the enum.
        // (ValueEnum matches case-insensitively already, but we normalized above.)
        match ReleaseKind::from_str(&normalized, true) {
            Ok(kind) => {
                set.insert(kind);
            }
            Err(_) => {
                // Build a helpful error with allowed values
                let allowed = ["album", "single", "appears_on", "compilation", "all"].join(", ");
                return Err(format!(
                    "invalid value '{part}' for --type (allowed: {allowed})"
                ));
            }
        }
    }

    if set.is_empty() {
        // This would only happen if input was something like just commas, but we guard above anyway.
        return Err("no valid kinds provided to --type".into());
    }

    Ok(ReleaseKinds(set))
}
