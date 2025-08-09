use std::time::Duration;

use chrono::{Datelike, NaiveDate};
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::{Client, StatusCode};
use tabled::Table;
use tokio::time::sleep;

use crate::{
    config, error,
    management::{
        ArtistReleaseManager, ReleaseWeekManager, STATE_TYPE_RELEASES, StateManager, TokenManager,
    },
    success,
    types::{Album, AlbumResponse, ArtistReleases, ReleaseTableRow, ReleaseWeek},
    utils, warning,
};

pub async fn releases(
    update: bool,
    force_update: bool,
    release_types: utils::ReleaseKinds,
    weeks_include: Option<u32>,
    release_date: Option<String>,
) {
    if update {
        match call_update(force_update, &release_types).await {
            Ok(message) => success!("{}", message),
            Err(_) => error!("Cannot update from remote"),
        }

        return;
    }

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
                        "Failed to load releases for week {}/{}: {}\nRun sporlcli releases --update.",
                        release_week.week.clone(),
                        cur_year,
                        e
                    );
                    continue;
                }
            },
            Err(e) => {
                warning!(
                    "Failed to load releases for week {}/{}: {:?}\nRun sporlcli releases --update.",
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

async fn call_update(force: bool, release_types: &utils::ReleaseKinds) -> Result<String, String> {
    let pb = ProgressBar::new_spinner();
    pb.set_message("Fetching releases for followed artists...");
    pb.enable_steady_tick(Duration::from_secs(100));
    pb.set_style(
        ProgressStyle::with_template("{spinner:.blue} {msg}")
            .unwrap()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏"),
    );

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

    let artist_chunks = artist_releases.chunks(20);
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

            match load_releases_from_remote(artist.artist.id.clone(), &token, 50, release_types)
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
    Ok("Release cache updated.".to_string())
}

async fn load_releases_from_remote(
    artist_id: String,
    token: &str,
    limit: u32,
    release_types: &utils::ReleaseKinds,
) -> Result<Vec<Album>, reqwest::Error> {
    let client = Client::new();
    let api_url = format!(
        "{uri}/artists/{id}/albums?include_groups={include_groups}&limit={limit}",
        uri = &config::spotify_apiurl(),
        id = artist_id,
        include_groups = format!("{}", release_types),
        limit = limit
    );

    let response = client.get(&api_url).bearer_auth(token).send().await?;
    // check for retry-after header
    if response.status() == StatusCode::TOO_MANY_REQUESTS {
        if let Some(retry_after) = response.headers().get("retry-after") {
            let retry_after = retry_after
                .to_str()
                .unwrap_or("0")
                .parse::<u64>()
                .unwrap_or(0);
            if retry_after <= 120 {
                sleep(Duration::from_secs(retry_after)).await;
            } else {
                warning!(
                    "Retry after has reached a abnormal high of {} seconds. Try your best tommorrow again.",
                    retry_after
                );
            }
        }
    }

    let json = response.json::<AlbumResponse>().await?;

    Ok(json.items)
}

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
