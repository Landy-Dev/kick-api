use crate::error::{KickApiError, Result};
use crate::models::{EventSubscription, SubscribeRequest, SubscribeResult};
use reqwest;

/// Events API - handles webhook/event subscription endpoints
///
/// Scopes required: `events:subscribe`
pub struct EventsApi<'a> {
    client: &'a reqwest::Client,
    token: &'a Option<String>,
    base_url: &'a str,
}

impl<'a> EventsApi<'a> {
    /// Create a new EventsApi instance
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

    /// List active event subscriptions
    ///
    /// Optionally filter by broadcaster user ID.
    ///
    /// Requires OAuth token with `events:subscribe` scope
    ///
    /// # Example
    /// ```no_run
    /// // List all subscriptions
    /// let subs = client.events().list(None).await?;
    ///
    /// // List subscriptions for a specific broadcaster
    /// let subs = client.events().list(Some(12345)).await?;
    /// ```
    pub async fn list(
        &self,
        broadcaster_user_id: Option<u64>,
    ) -> Result<Vec<EventSubscription>> {
        self.require_token()?;

        let url = format!("{}/events/subscriptions", self.base_url);
        let mut request = self
            .client
            .get(&url)
            .header("Accept", "*/*")
            .bearer_auth(self.token.as_ref().unwrap());

        if let Some(id) = broadcaster_user_id {
            request = request.query(&[("broadcaster_user_id", id)]);
        }

        let response = crate::http::send_with_retry(self.client, request).await?;

        if response.status().is_success() {
            let body = response.text().await?;

            #[derive(serde::Deserialize)]
            struct DataResponse {
                data: Vec<EventSubscription>,
            }

            let resp: DataResponse = serde_json::from_str(&body)
                .map_err(|e| KickApiError::ApiError(format!("JSON parse error: {}", e)))?;

            Ok(resp.data)
        } else {
            Err(KickApiError::ApiError(format!(
                "Failed to list event subscriptions: {}",
                response.status()
            )))
        }
    }

    /// Subscribe to events
    ///
    /// Requires OAuth token with `events:subscribe` scope
    ///
    /// # Example
    /// ```no_run
    /// use kick_api::{SubscribeRequest, SubscribeEvent};
    ///
    /// let request = SubscribeRequest {
    ///     broadcaster_user_id: Some(12345),
    ///     method: "webhook".to_string(),
    ///     events: vec![
    ///         SubscribeEvent { name: "chat.message.created".to_string(), version: 1 },
    ///     ],
    /// };
    /// let results = client.events().subscribe(request).await?;
    /// ```
    pub async fn subscribe(
        &self,
        request: SubscribeRequest,
    ) -> Result<Vec<SubscribeResult>> {
        self.require_token()?;

        let url = format!("{}/events/subscriptions", self.base_url);
        let request = self
            .client
            .post(&url)
            .header("Accept", "*/*")
            .bearer_auth(self.token.as_ref().unwrap())
            .json(&request);
        let response = crate::http::send_with_retry(self.client, request).await?;

        if response.status().is_success() {
            let body = response.text().await?;

            #[derive(serde::Deserialize)]
            struct DataResponse {
                data: Vec<SubscribeResult>,
            }

            let resp: DataResponse = serde_json::from_str(&body)
                .map_err(|e| KickApiError::ApiError(format!("JSON parse error: {}", e)))?;

            Ok(resp.data)
        } else {
            Err(KickApiError::ApiError(format!(
                "Failed to subscribe to events: {}",
                response.status()
            )))
        }
    }

    /// Unsubscribe from events by subscription IDs
    ///
    /// Requires OAuth token with `events:subscribe` scope
    ///
    /// # Example
    /// ```no_run
    /// client.events().unsubscribe(vec!["sub_id_1".to_string(), "sub_id_2".to_string()]).await?;
    /// ```
    pub async fn unsubscribe(&self, ids: Vec<String>) -> Result<()> {
        self.require_token()?;

        let url = format!("{}/events/subscriptions", self.base_url);
        let id_pairs: Vec<(&str, &str)> = ids.iter().map(|id| ("id", id.as_str())).collect();

        let request = self
            .client
            .delete(&url)
            .header("Accept", "*/*")
            .bearer_auth(self.token.as_ref().unwrap())
            .query(&id_pairs);
        let response = crate::http::send_with_retry(self.client, request).await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(KickApiError::ApiError(format!(
                "Failed to unsubscribe from events: {}",
                response.status()
            )))
        }
    }

    fn require_token(&self) -> Result<()> {
        if self.token.is_none() {
            return Err(KickApiError::ApiError(
                "OAuth token required for this endpoint".to_string(),
            ));
        }
        Ok(())
    }
}
