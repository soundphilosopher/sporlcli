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

pub async fn update_artists(force: bool) {
    let artist_cache_count = match ArtistReleaseManager::load().await {
        Ok(arm) => arm.count_artists(),
        Err(_) => 0,
    };

    let artist_remote_count = match spotify::artists::get_total_artist_count().await {
        Ok(c) => c,
        Err(_) => 0,
    };

    let max_new: u64 = if force {
        artist_remote_count
    } else {
        if artist_remote_count > artist_cache_count as u64 {
            artist_remote_count - artist_cache_count as u64
        } else {
            0
        }
    };

    if let Err(e) = load_remote_artists(max_new).await {
        error!("Cannot update artists. Err: {}", e)
    }
}

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

async fn load_cached_artists() -> Result<Vec<Artist>, String> {
    match ArtistReleaseManager::load().await {
        Ok(arm) => Ok(arm.get_all_artists().unwrap_or(Vec::new())),
        Err(e) => Err(format!("Failed to load artists. Err: {}", e)),
    }
}

async fn load_remote_artists(max_new: u64) -> Result<Vec<ArtistReleases>, reqwest::Error> {
    let mut arm: ArtistReleaseManager = match ArtistReleaseManager::load().await {
        Ok(arm) => arm,
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
