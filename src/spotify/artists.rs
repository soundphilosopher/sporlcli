use std::time::Duration;

use indicatif::{ProgressBar, ProgressStyle};
use reqwest::{Client, StatusCode};
use tokio::time::sleep;

use crate::{
    config, error,
    management::TokenManager,
    types::{Artist, FollowedArtistsResponse},
};

pub async fn get_artist(
    token: &str,
    limit: u64,
    after: Option<String>,
) -> Result<(Vec<Artist>, Option<String>), reqwest::Error> {
    let attempt_after = after.clone();

    loop {
        let mut api_url = format!(
            "{uri}/me/following?type=artist&limit={limit}",
            uri = &config::spotify_apiurl(),
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

pub async fn get_total_artist_count() -> Result<u64, reqwest::Error> {
    let mut token_mgr = match TokenManager::load().await {
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
        let api_url = format!("{uri}/me/following?type={type}&limit={limit}", uri = &config::spotify_apiurl(), type = "artist", limit = "1");

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
