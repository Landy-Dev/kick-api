use reqwest;
use crate::error::{KickApiError, Result};

const KICK_BASE_URL: &str = "https://kick.com/api/v2";

pub struct KickApiClient {
    base_url: String,
    client: reqwest::Client,
    oauth_token: Option<String>,
}

impl KickApiClient {
    pub fn new() -> Self {
        KickApiClient {
            base_url: KICK_BASE_URL.to_string(),
            client: reqwest::Client::new(),
            oauth_token: None,
        }
    }

    pub async fn get_channel(&self, channel_slug: &str) -> Result<String> {
        let url = format!("{}/channels/{}", self.base_url, channel_slug);

        let response = self.client.get(&url).send().await?;

        if response.status().is_success() {
            let body = response.text().await?;
            Ok(body)
        } else {
            Err(KickApiError::ApiError(format!(
                "Failed to get channel: {}",
                response.status()
            )))
        }
    }
}