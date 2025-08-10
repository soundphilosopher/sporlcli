use std::time::Duration;

use reqwest::{Client, StatusCode};
use tokio::time::sleep;

use crate::{
    config, error,
    management::TokenManager,
    types::{Album, AlbumResponse, GetSeveralAlbumsResponse},
    utils, warning,
};

pub async fn get_release_for_artist(
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

pub async fn get_several_releases(
    albums: &Vec<Album>,
) -> Result<GetSeveralAlbumsResponse, reqwest::Error> {
    let album_ids = albums
        .iter()
        .map(|a| a.id.as_str())
        .collect::<Vec<_>>()
        .join(",");

    let api_url = format!(
        "{url}/albums?ids={album_ids}",
        url = &config::spotify_apiurl(),
        album_ids = album_ids
    );

    let mut token_mgr = match TokenManager::load().await {
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
