use std::{cmp::Ordering, collections::HashSet};

use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::{Datelike, Duration, NaiveDate, Utc, Weekday};
use rand::{Rng, distr::Alphanumeric};
use sha2::{Digest, Sha256};

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
