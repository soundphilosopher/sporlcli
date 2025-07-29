use std::time::Duration;

use chrono::Datelike;
use reqwest::{Client, StatusCode};
use tokio::time::sleep;

use crate::{
    common, error, info,
    management::TokenManager,
    success,
    types::{
        AddTrackToPlaylistRequest, AddTrackToPlaylistResponse, Album, CreatePlaylistRequest,
        CreatePlaylistResponse, GetSeveralAlbumsResponse, GetUserPlaylistsResponse, Track,
    },
    utils, warning,
};

pub async fn playlist(previous_weeks: Option<u32>, release_date: Option<String>) {
    let curr_date = utils::get_date_from_string(release_date);
    let curr_year = curr_date.year();
    let release_weeks = utils::get_custom_week_range(curr_date, previous_weeks.unwrap_or(0));

    for release_week in release_weeks {
        let playlist_name = format!(
            "Weekly Picks {}/{}",
            release_week.week.clone(),
            curr_year.clone()
        );

        let playlist_exists = match playlist_already_exists(&playlist_name).await {
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
            "Gather album information for release week {}/{}",
            release_week.week.clone(),
            curr_year.clone()
        );

        let mut all_albums: Vec<GetSeveralAlbumsResponse> = Vec::new();

        let releases: Vec<Album> =
            match utils::get_weekly_releases(release_week.week, curr_year).await {
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
            let handle = tokio::spawn(async move { get_several_albums(&chunk).await });
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
            "Album information gathered for release week {}/{}",
            release_week.week.clone(),
            curr_year.clone()
        );

        info!(
            "Create playlist for release week {}/{}",
            release_week.week.clone(),
            curr_year.clone()
        );

        let playlist_id: Option<String> = match create_playlist(playlist_name).await {
            Ok(resp) => {
                success!(
                    "Playlist for release week {}/{} created.",
                    release_week.week.clone(),
                    curr_year.clone()
                );
                Some(resp.id.clone())
            }
            Err(e) => {
                warning!("Failed to create playlist: {}", e);
                None
            }
        };

        if let Some(playlist_id) = playlist_id {
            info!(
                "Add tracks to playlist for release week {}/{}",
                release_week.week.clone(),
                curr_year.clone()
            );
            let tracks: Vec<Track> = all_albums
                .iter()
                .flat_map(|ar| {
                    ar.albums
                        .iter()
                        .flat_map(|album| album.tracks.items.first())
                })
                .cloned()
                .collect();
            match add_tracks_to_playlist(playlist_id, tracks).await {
                Ok(_) => success!(
                    "Tracks added to playlist for release week {}/{}",
                    release_week.week.clone(),
                    curr_year.clone()
                ),
                Err(e) => warning!("Failed to add tracks to playlist: {}", e),
            };
        }
    }
}

async fn get_several_albums(
    albums: &Vec<Album>,
) -> Result<GetSeveralAlbumsResponse, reqwest::Error> {
    let album_ids = albums
        .iter()
        .map(|a| a.id.as_str())
        .collect::<Vec<_>>()
        .join(",");

    let api_url = format!(
        "{url}/albums?ids={album_ids}",
        url = common::SPOTIFY_API_URL,
        album_ids = album_ids
    );

    let mut token_mgr = match TokenManager::load_from_cache().await {
        Ok(manager) => manager,
        Err(e) => {
            error!(
                "Failed to load token. Please run sporlcli auth\n Error: {}",
                e
            );
        }
    };

    loop {
        let client = Client::new();
        let token = token_mgr.get_valid_token().await;
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

        let json = response.json::<GetSeveralAlbumsResponse>().await?;
        return Ok(json);
    }
}

async fn create_playlist(name: String) -> Result<CreatePlaylistResponse, reqwest::Error> {
    let mut token_mgr = match TokenManager::load_from_cache().await {
        Ok(manager) => manager,
        Err(e) => {
            error!(
                "Failed to load token. Please run sporlcli auth\n Error: {}",
                e
            );
        }
    };

    let api_url = format!(
        "{url}/users/{user_id}/playlists",
        url = common::SPOTIFY_API_URL,
        user_id = common::SPOTIFY_USER_ID,
    );

    loop {
        let client = Client::new();
        let token = token_mgr.get_valid_token().await;
        let response = client
            .post(&api_url)
            .bearer_auth(token)
            .json(&serde_json::json!(CreatePlaylistRequest {
                name: name.clone(),
                description: "[auto] Generated by SporlCLI".to_string(),
                public: false,
                collaborative: false
            }))
            .send()
            .await;

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

        let json = response.json::<CreatePlaylistResponse>().await?;
        return Ok(json);
    }
}

async fn add_tracks_to_playlist(
    playlist_id: String,
    tracks: Vec<Track>,
) -> Result<AddTrackToPlaylistResponse, reqwest::Error> {
    let mut token_mgr = match TokenManager::load_from_cache().await {
        Ok(manager) => manager,
        Err(e) => {
            error!(
                "Failed to load token. Please run sporlcli auth\n Error: {}",
                e
            );
        }
    };

    let api_url = format!(
        "{url}/playlists/{playlist_id}/tracks",
        url = common::SPOTIFY_API_URL,
        playlist_id = playlist_id,
    );

    loop {
        let client = Client::new();
        let token = token_mgr.get_valid_token().await;
        let response = client
            .post(&api_url)
            .bearer_auth(token)
            .json(&serde_json::json!(AddTrackToPlaylistRequest {
                uris: tracks.iter().map(|track| track.uri.clone()).collect()
            }))
            .send()
            .await;

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

        let json = response.json::<AddTrackToPlaylistResponse>().await?;
        return Ok(json);
    }
}

async fn playlist_already_exists(playlist_name: &str) -> Result<bool, reqwest::Error> {
    let mut token_mgr = match TokenManager::load_from_cache().await {
        Ok(manager) => manager,
        Err(e) => {
            error!(
                "Failed to load token. Please run sporlcli auth\n Error: {}",
                e
            );
        }
    };

    let api_url = format!("{url}/me/playlists", url = common::SPOTIFY_API_URL,);

    loop {
        let client = Client::new();
        let token = token_mgr.get_valid_token().await;
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

        let json = response.json::<GetUserPlaylistsResponse>().await?;
        for playlist in json.items {
            if playlist.name == playlist_name {
                return Ok(true);
            }
        }

        return Ok(false);
    }
}
