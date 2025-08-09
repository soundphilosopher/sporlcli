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

pub fn generate_code_verifier() -> String {
    rand::rng()
        .sample_iter(&Alphanumeric)
        .take(128)
        .map(char::from)
        .collect()
}

pub fn generate_code_challenge(verifier: &str) -> String {
    let hash = Sha256::digest(verifier.as_bytes());
    URL_SAFE_NO_PAD.encode(hash)
}

fn get_saturday_before_or_on(date: NaiveDate) -> NaiveDate {
    let weekday = date.weekday().num_days_from_sunday(); // Sunday=0, Saturday=6
    let days_to_subtract = (weekday + 1) % 7; // How many days to go back to Saturday
    date - Duration::days(days_to_subtract as i64)
}

pub fn get_release_week_number(date: NaiveDate) -> u32 {
    let jan1 = NaiveDate::from_ymd_opt(date.year(), 1, 1).unwrap();
    let first_week_start = get_saturday_before_or_on(jan1);
    let current_week_start = get_saturday_before_or_on(date);
    let diff = current_week_start - first_week_start;
    (diff.num_days() / 7 + 1) as u32
}

pub fn build_week(date: NaiveDate) -> WeekOfTheYear {
    let saturday = get_saturday_before_or_on(date);
    let dates: Vec<NaiveDate> = (0..7).map(|i| saturday + Duration::days(i)).collect();

    let week_number = get_release_week_number(saturday);

    WeekOfTheYear {
        week: week_number,
        dates,
    }
}

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

pub fn remove_duplicate_albums(albums: &mut Vec<Album>) {
    let mut seen_ids = HashSet::new();
    albums.retain(|album| seen_ids.insert(album.id.clone()));
}

pub fn sort_release_table_rows(rows: &mut Vec<ReleaseTableRow>) {
    rows.sort_by(|a, b| {
        match b.date.cmp(&a.date) {
            Ordering::Equal => a.artists.cmp(&b.artists), // secondary sort: name ascending
            other => other,
        }
    });
}

pub fn get_date_from_string(date: Option<String>) -> NaiveDate {
    match date {
        Some(date_str) => NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
            .unwrap_or_else(|_| Utc::now().date_naive()),
        None => Utc::now().date_naive(),
    }
}

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

fn sort_albums_by_date_and_artist(albums: &mut Vec<Album>) {
    albums.sort_by(|a, b| {
        let date_cmp = b.release_date.cmp(&a.release_date);
        if date_cmp != Ordering::Equal {
            return date_cmp;
        }

        let a_artist = a.artists.get(0).map(|artist| &artist.name);
        let b_artist = b.artists.get(0).map(|artist| &artist.name);

        a_artist.cmp(&b_artist)
    });
}

/// The normalized set of types your command will use.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, ValueEnum, Hash)]
#[value(rename_all = "snake_case")]
pub enum ReleaseKind {
    Album,
    Single,
    AppearsOn,
    Compilation,
}

impl ReleaseKind {
    const ALL: [ReleaseKind; 4] = [
        ReleaseKind::Album,
        ReleaseKind::Single,
        ReleaseKind::AppearsOn,
        ReleaseKind::Compilation,
    ];
}

/// A validated, deduplicated set of kinds parsed from `--type`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReleaseKinds(BTreeSet<ReleaseKind>);

impl ReleaseKinds {
    pub fn iter(&self) -> impl Iterator<Item = ReleaseKind> + '_ {
        self.0.iter().copied()
    }
}

impl Default for ReleaseKinds {
    fn default() -> Self {
        // Default value is "album"
        let mut set = BTreeSet::new();
        set.insert(ReleaseKind::Album);
        ReleaseKinds(set)
    }
}

// Print a single kind as clap's canonical value (lower_snake_case).
impl fmt::Display for ReleaseKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = self
            .to_possible_value()
            .expect("ValueEnum should have a possible value")
            .get_name()
            .to_owned();
        f.write_str(&name)
    }
}

// Print the set as a comma-separated list using those canonical names.
impl fmt::Display for ReleaseKinds {
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

/// Custom parser for `--type`. Accepts comma-separated values, expands `all`,
/// validates entries, trims whitespace, and deduplicates.
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
