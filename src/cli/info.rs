use chrono::Utc;

use crate::{cli::artists, error, info, management::ArtistsManager, utils, warning};

struct ReleaseWeekInfo {
    week: u32,
    dates: String,
}

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
        let artist_cache_count = match ArtistsManager::load_from_cache().await {
            Ok(am) => am.count(),
            Err(_) => 0,
        };

        let artist_remote_count = match artists::get_remote_artist_count().await {
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
