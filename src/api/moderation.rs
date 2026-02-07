use crate::error::{KickApiError, Result};
use crate::models::{BanRequest, UnbanRequest};
use reqwest;

/// Moderation API - handles ban/unban endpoints
///
/// Scopes required: `moderation:ban`
pub struct ModerationApi<'a> {
    client: &'a reqwest::Client,
    token: &'a Option<String>,
    base_url: &'a str,
}

impl<'a> ModerationApi<'a> {
    /// Create a new ModerationApi instance
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

    /// Ban or timeout a user in a channel
    ///
    /// If `duration` is provided in the request, this is a timeout (temporary ban).
    /// If `duration` is `None`, this is a permanent ban.
    ///
    /// Requires OAuth token with `moderation:ban` scope
    ///
    /// # Example
    /// ```no_run
    /// use kick_api::BanRequest;
    ///
    /// // Permanent ban
    /// let request = BanRequest {
    ///     broadcaster_user_id: 12345,
    ///     user_id: 67890,
    ///     reason: Some("Breaking rules".to_string()),
    ///     duration: None,
    /// };
    /// client.moderation().ban(request).await?;
    /// ```
    pub async fn ban(&self, request: BanRequest) -> Result<()> {
        self.require_token()?;

        let url = format!("{}/moderation/bans", self.base_url);
        let response = self
            .client
            .post(&url)
            .header("Accept", "*/*")
            .bearer_auth(self.token.as_ref().unwrap())
            .json(&request)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(KickApiError::ApiError(format!(
                "Failed to ban user: {}",
                response.status()
            )))
        }
    }

    /// Unban a user in a channel
    ///
    /// Requires OAuth token with `moderation:ban` scope
    ///
    /// # Example
    /// ```no_run
    /// use kick_api::UnbanRequest;
    ///
    /// let request = UnbanRequest {
    ///     broadcaster_user_id: 12345,
    ///     user_id: 67890,
    /// };
    /// client.moderation().unban(request).await?;
    /// ```
    pub async fn unban(&self, request: UnbanRequest) -> Result<()> {
        self.require_token()?;

        let url = format!("{}/moderation/bans", self.base_url);
        let response = self
            .client
            .delete(&url)
            .header("Accept", "*/*")
            .bearer_auth(self.token.as_ref().unwrap())
            .json(&request)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(KickApiError::ApiError(format!(
                "Failed to unban user: {}",
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
