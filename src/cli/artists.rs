use std::time::Duration;

use indicatif::{ProgressBar, ProgressStyle};
use reqwest::{Client, StatusCode};
use tabled::Table;
use tokio::time::sleep;

use crate::{
    common, error,
    management::{ArtistsManager, TokenManager},
    success,
    types::{Artist, ArtistTableRow, FollowedArtistsResponse},
    warning,
};

pub async fn artists(update: bool, search: Option<String>) {
    if update {
        let artist_cache_count = match ArtistsManager::load_from_cache().await {
            Ok(am) => am.count(),
            Err(_) => 0,
        };

        let artist_remote_count = match get_remote_artist_count().await {
            Ok(c) => c,
            Err(_) => 0,
        };

        let max_new: u64 = if artist_remote_count > artist_cache_count {
            artist_remote_count - artist_cache_count
        } else {
            0
        };

        if let Err(e) = load_remote_artists(max_new).await {
            error!("Cannot update artists. Err: {}", e)
        }
        return;
    }

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
    let artists_mgr = ArtistsManager::load_from_cache().await?;
    Ok(artists_mgr.get_artists())
}

async fn load_remote_artists(max_new: u64) -> Result<Vec<Artist>, reqwest::Error> {
    let mut token_mgr = match TokenManager::load_from_cache().await {
        Ok(t) => t,
        Err(e) => {
            error!(
                "Failed to load token. Please run sporlcli auth\n Error: {}",
                e
            );
        }
    };

    let mut all_artists: Vec<Artist> = match ArtistsManager::load_from_cache().await {
        Ok(mgr) => mgr.get_artists(),
        Err(_) => Vec::new(),
    };

    let mut after: Option<String> = None;
    let mut new_once = max_new;
    if new_once == 0 {
        success!("Nothing to update here.");
        return Ok(all_artists);
    }

    let pb = ProgressBar::new_spinner();
    pb.set_message("Fetching followed artists...");
    pb.enable_steady_tick(Duration::from_millis(100));
    pb.set_style(
        ProgressStyle::with_template("{spinner:.blue} {msg}")
            .unwrap()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏"),
    );

    let mut total_fetched = 0;
    let mut limit = 50;

    loop {
        if new_once < 50 {
            limit = new_once;
        }

        let token = token_mgr.get_valid_token().await;
        let result = get_artists_from_remote(&token, limit, after.clone()).await;

        match result {
            Ok((artists, next_after)) => {
                if artists.is_empty() {
                    break;
                }

                total_fetched += artists.len();
                new_once -= total_fetched as u64;
                pb.set_message(format!("Fetched {} artists...", total_fetched));

                all_artists.extend(artists);
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
    success!("Fetched {} artists!", all_artists.len());

    let artists_mgr = ArtistsManager::new(all_artists.clone());
    if let Err(e) = artists_mgr.save_to_cache().await {
        error!("Failed to cache artists. Err: {}", e);
    }

    Ok(all_artists)
}

pub async fn get_remote_artist_count() -> Result<u64, reqwest::Error> {
    let mut token_mgr = match TokenManager::load_from_cache().await {
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
        let api_url = format!("{uri}/me/following?type={type}&limit={limit}", uri = common::SPOTIFY_API_URL, type = "artist", limit = "1");

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

async fn get_artists_from_remote(
    token: &str,
    limit: u64,
    after: Option<String>,
) -> Result<(Vec<Artist>, Option<String>), reqwest::Error> {
    let attempt_after = after.clone();

    loop {
        let mut api_url = format!(
            "{uri}/me/following?type=artist&limit={limit}",
            uri = common::SPOTIFY_API_URL,
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
