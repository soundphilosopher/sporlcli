use chrono::Datelike;

use crate::{
    info, spotify, success,
    types::{Album, GetSeveralAlbumsResponse, Track},
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

        let playlist_exists = match spotify::playlist::exists(&playlist_name).await {
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
            let handle =
                tokio::spawn(async move { spotify::releases::get_several_releases(&chunk).await });
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

        let playlist_id: Option<String> = match spotify::playlist::create(playlist_name).await {
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

            let tracks_chunks = tracks.chunks(100);
            for chunk in tracks_chunks {
                match spotify::playlist::add_tracks(playlist_id.clone(), chunk.to_vec()).await {
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
}
