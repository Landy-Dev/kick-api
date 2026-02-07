use crate::error::{KickApiError, Result};
use crate::models::{
    ChannelReward, ChannelRewardRedemption, CreateRewardRequest, ManageRedemptionsRequest,
    ManageRedemptionsResponse, RedemptionStatus, UpdateRewardRequest,
};
use reqwest;

/// Rewards API - handles all channel reward endpoints
pub struct RewardsApi<'a> {
    client: &'a reqwest::Client,
    token: &'a Option<String>,
    base_url: &'a str,
}

impl<'a> RewardsApi<'a> {
    /// Create a new RewardsApi instance
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

    /// Get all channel rewards
    ///
    /// Requires OAuth token with `channel:rewards:read` scope
    ///
    /// # Example
    /// ```no_run
    /// let rewards = client.rewards().get_all().await?;
    /// for reward in rewards {
    ///     println!("Reward: {} - {} points", reward.title, reward.cost);
    /// }
    /// ```
    pub async fn get_all(&self) -> Result<Vec<ChannelReward>> {
        self.require_token()?;

        let url = format!("{}/channels/rewards", self.base_url);
        let response = self
            .client
            .get(&url)
            .header("Accept", "*/*")
            .bearer_auth(self.token.as_ref().unwrap())
            .send()
            .await?;

        self.parse_response(response).await
    }

    /// Create a new channel reward
    ///
    /// Requires OAuth token with `channel:rewards:write` scope
    ///
    /// # Example
    /// ```no_run
    /// use kick_api::CreateRewardRequest;
    ///
    /// let request = CreateRewardRequest {
    ///     title: "Song Request".to_string(),
    ///     cost: 500,
    ///     description: Some("Request a song!".to_string()),
    ///     is_user_input_required: Some(true),
    ///     ..Default::default()
    /// };
    ///
    /// let reward = client.rewards().create(request).await?;
    /// ```
    pub async fn create(&self, request: CreateRewardRequest) -> Result<ChannelReward> {
        self.require_token()?;

        let url = format!("{}/channels/rewards", self.base_url);
        let response = self
            .client
            .post(&url)
            .header("Accept", "*/*")
            .bearer_auth(self.token.as_ref().unwrap())
            .json(&request)
            .send()
            .await?;

        self.parse_single_response(response).await
    }

    /// Update an existing reward
    ///
    /// Requires OAuth token with `channel:rewards:write` scope
    ///
    /// # Example
    /// ```no_run
    /// use kick_api::UpdateRewardRequest;
    ///
    /// let update = UpdateRewardRequest {
    ///     cost: Some(1000),
    ///     is_paused: Some(true),
    ///     ..Default::default()
    /// };
    ///
    /// let reward = client.rewards().update("reward_id", update).await?;
    /// ```
    pub async fn update(
        &self,
        reward_id: &str,
        request: UpdateRewardRequest,
    ) -> Result<ChannelReward> {
        self.require_token()?;

        let url = format!("{}/channels/rewards/{}", self.base_url, reward_id);
        let response = self
            .client
            .patch(&url)
            .header("Accept", "*/*")
            .bearer_auth(self.token.as_ref().unwrap())
            .json(&request)
            .send()
            .await?;

        self.parse_single_response(response).await
    }

    /// Delete a reward
    ///
    /// Requires OAuth token with `channel:rewards:write` scope
    pub async fn delete(&self, reward_id: &str) -> Result<()> {
        self.require_token()?;

        let url = format!("{}/channels/rewards/{}", self.base_url, reward_id);
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
                "Failed to delete reward: {}",
                response.status()
            )))
        }
    }

    /// Get reward redemptions
    ///
    /// Requires OAuth token with `channel:rewards:read` scope
    ///
    /// # Parameters
    /// - `reward_id`: Optional - filter by specific reward
    /// - `status`: Optional - filter by status (defaults to pending)
    pub async fn get_redemptions(
        &self,
        reward_id: Option<&str>,
        status: Option<RedemptionStatus>,
    ) -> Result<Vec<ChannelRewardRedemption>> {
        self.require_token()?;

        let url = format!("{}/channels/rewards/redemptions", self.base_url);
        let mut request = self
            .client
            .get(&url)
            .header("Accept", "*/*")
            .bearer_auth(self.token.as_ref().unwrap());

        if let Some(id) = reward_id {
            request = request.query(&[("reward_id", id)]);
        }

        if let Some(s) = status {
            let status_str = match s {
                RedemptionStatus::Pending => "pending",
                RedemptionStatus::Accepted => "accepted",
                RedemptionStatus::Rejected => "rejected",
            };
            request = request.query(&[("status", status_str)]);
        }

        let response = request.send().await?;
        self.parse_response(response).await
    }

    /// Accept pending redemptions
    ///
    /// Requires OAuth token with `channel:rewards:write` scope
    ///
    /// # Parameters
    /// - `redemption_ids`: List of redemption IDs to accept (1-25)
    pub async fn accept_redemptions(
        &self,
        redemption_ids: Vec<String>,
    ) -> Result<ManageRedemptionsResponse> {
        self.manage_redemptions("accept", redemption_ids).await
    }

    /// Reject pending redemptions
    ///
    /// Requires OAuth token with `channel:rewards:write` scope
    ///
    /// # Parameters
    /// - `redemption_ids`: List of redemption IDs to reject (1-25)
    pub async fn reject_redemptions(
        &self,
        redemption_ids: Vec<String>,
    ) -> Result<ManageRedemptionsResponse> {
        self.manage_redemptions("reject", redemption_ids).await
    }

    // Helper methods

    fn require_token(&self) -> Result<()> {
        if self.token.is_none() {
            return Err(KickApiError::ApiError(
                "OAuth token required for this endpoint".to_string(),
            ));
        }
        Ok(())
    }

    async fn parse_response<T: serde::de::DeserializeOwned>(
        &self,
        response: reqwest::Response,
    ) -> Result<Vec<T>> {
        if response.status().is_success() {
            let body = response.text().await?;

            #[derive(serde::Deserialize)]
            struct DataResponse<T> {
                data: Vec<T>,
            }

            let resp: DataResponse<T> = serde_json::from_str(&body)
                .map_err(|e| KickApiError::ApiError(format!("JSON parse error: {}", e)))?;

            Ok(resp.data)
        } else {
            Err(KickApiError::ApiError(format!(
                "Request failed: {}",
                response.status()
            )))
        }
    }

    async fn parse_single_response<T: serde::de::DeserializeOwned>(
        &self,
        response: reqwest::Response,
    ) -> Result<T> {
        if response.status().is_success() {
            let body = response.text().await?;

            #[derive(serde::Deserialize)]
            struct DataResponse<T> {
                data: T,
            }

            let resp: DataResponse<T> = serde_json::from_str(&body)
                .map_err(|e| KickApiError::ApiError(format!("JSON parse error: {}", e)))?;

            Ok(resp.data)
        } else {
            Err(KickApiError::ApiError(format!(
                "Request failed: {}",
                response.status()
            )))
        }
    }

    async fn manage_redemptions(
        &self,
        action: &str,
        redemption_ids: Vec<String>,
    ) -> Result<ManageRedemptionsResponse> {
        self.require_token()?;

        let url = format!("{}/channels/rewards/redemptions/{}", self.base_url, action);
        let request_body = ManageRedemptionsRequest { ids: redemption_ids };

        let response = self
            .client
            .post(&url)
            .header("Accept", "*/*")
            .bearer_auth(self.token.as_ref().unwrap())
            .json(&request_body)
            .send()
            .await?;

        if response.status().is_success() {
            let body = response.text().await?;
            let resp: ManageRedemptionsResponse = serde_json::from_str(&body)
                .map_err(|e| KickApiError::ApiError(format!("JSON parse error: {}", e)))?;
            Ok(resp)
        } else {
            Err(KickApiError::ApiError(format!(
                "Failed to {} redemptions: {}",
                action,
                response.status()
            )))
        }
    }
}
