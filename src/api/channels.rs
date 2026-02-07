use crate::error::{KickApiError, Result};
use crate::models::Channel;
use reqwest;

/// Channels API - handles all channel-related endpoints
pub struct ChannelsApi<'a> {
    client: &'a reqwest::Client,
    token: &'a Option<String>,
    base_url: &'a str,
}

impl<'a> ChannelsApi<'a> {
    /// Create a new ChannelsApi instance
    pub(crate) fn new(
        client: &'a reqwest::Client,
        token: &'a Option<String>,
        base_url: &'a str,
    ) -> Self {
        Self {
            client,
            token,
            base_url,
        }
    }

    /// Get a channel by slug
    ///
    /// Requires OAuth token with `channel:read` scope
    ///
    /// # Example
    /// ```no_run
    /// let channel = client.channels().get("xqc").await?;
    /// println!("Channel: {}", channel.slug);
    /// ```
    pub async fn get(&self, channel_slug: &str) -> Result<Channel> {
        let url = format!("{}/channels", self.base_url);

        let mut request = self
            .client
            .get(&url)
            .header("Accept", "*/*")
            .query(&[("slug", channel_slug)]);

        if let Some(token) = self.token {
            request = request.bearer_auth(token);
        } else {
            return Err(KickApiError::ApiError(
                "OAuth token required for this endpoint".to_string(),
            ));
        }

        let response = crate::http::send_with_retry(self.client, request).await?;
        if response.status().is_success() {
            let body = response.text().await?;

            #[derive(serde::Deserialize)]
            struct ChannelsResponse {
                data: Vec<Channel>,
            }

            let resp: ChannelsResponse = serde_json::from_str(&body)
                .map_err(|e| KickApiError::ApiError(format!("JSON parse error: {}", e)))?;

            resp.data
                .into_iter()
                .next()
                .ok_or_else(|| KickApiError::ApiError("Channel not found".to_string()))
        } else {
            Err(KickApiError::ApiError(format!(
                "Failed to get channel: {}",
                response.status()
            )))
        }
    }

    /// Get your own channels (the authenticated user's channels)
    ///
    /// Requires OAuth token with `channel:read` scope
    ///
    /// # Example
    /// ```no_run
    /// let my_channels = client.channels().get_mine().await?;
    /// for channel in my_channels {
    ///     println!("My channel: {}", channel.slug);
    /// }
    /// ```
    pub async fn get_mine(&self) -> Result<Vec<Channel>> {
        let url = format!("{}/channels", self.base_url);

        let mut request = self.client.get(&url).header("Accept", "*/*");

        if let Some(token) = self.token {
            request = request.bearer_auth(token);
        } else {
            return Err(KickApiError::ApiError(
                "OAuth token required for this endpoint".to_string(),
            ));
        }

        let response = crate::http::send_with_retry(self.client, request).await?;
        if response.status().is_success() {
            let body = response.text().await?;

            #[derive(serde::Deserialize)]
            struct ChannelsResponse {
                data: Vec<Channel>,
            }

            let resp: ChannelsResponse = serde_json::from_str(&body)
                .map_err(|e| KickApiError::ApiError(format!("JSON parse error: {}", e)))?;

            Ok(resp.data)
        } else {
            Err(KickApiError::ApiError(format!(
                "Failed to get channels: {}",
                response.status()
            )))
        }
    }
}
