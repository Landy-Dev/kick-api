use crate::error::{KickApiError, Result};
use crate::models::{SendMessageRequest, SendMessageResponse};
use reqwest;

/// Chat API - handles chat message endpoints
///
/// Scopes required: `chat:write`, `moderation:chat_message:manage`
pub struct ChatApi<'a> {
    client: &'a reqwest::Client,
    token: &'a Option<String>,
    base_url: &'a str,
}

impl<'a> ChatApi<'a> {
    /// Create a new ChatApi instance
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

    /// Send a chat message
    ///
    /// Requires OAuth token with `chat:write` scope
    ///
    /// # Example
    /// ```no_run
    /// use kick_api::SendMessageRequest;
    ///
    /// let request = SendMessageRequest {
    ///     r#type: "user".to_string(),
    ///     content: "Hello chat!".to_string(),
    ///     broadcaster_user_id: Some(12345),
    ///     reply_to_message_id: None,
    /// };
    /// let response = client.chat().send_message(request).await?;
    /// println!("Message sent: {}", response.message_id);
    /// ```
    pub async fn send_message(&self, request: SendMessageRequest) -> Result<SendMessageResponse> {
        self.require_token()?;

        let url = format!("{}/chat", self.base_url);
        let response = self
            .client
            .post(&url)
            .header("Accept", "*/*")
            .bearer_auth(self.token.as_ref().unwrap())
            .json(&request)
            .send()
            .await?;

        if response.status().is_success() {
            let body = response.text().await?;

            #[derive(serde::Deserialize)]
            struct DataResponse {
                data: SendMessageResponse,
            }

            let resp: DataResponse = serde_json::from_str(&body)
                .map_err(|e| KickApiError::ApiError(format!("JSON parse error: {}", e)))?;

            Ok(resp.data)
        } else {
            Err(KickApiError::ApiError(format!(
                "Failed to send message: {}",
                response.status()
            )))
        }
    }

    /// Delete a chat message
    ///
    /// Requires OAuth token with `moderation:chat_message:manage` scope
    ///
    /// # Example
    /// ```no_run
    /// client.chat().delete_message("message_id_here").await?;
    /// ```
    pub async fn delete_message(&self, message_id: &str) -> Result<()> {
        self.require_token()?;

        let url = format!("{}/chat/{}", self.base_url, message_id);
        let response = self
            .client
            .delete(&url)
            .header("Accept", "*/*")
            .bearer_auth(self.token.as_ref().unwrap())
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(KickApiError::ApiError(format!(
                "Failed to delete message: {}",
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
