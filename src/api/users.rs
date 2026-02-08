use crate::error::{KickApiError, Result};
use crate::models::{TokenIntrospection, User};
use reqwest;

/// Users API - handles all user-related endpoints
pub struct UsersApi<'a> {
    client: &'a reqwest::Client,
    token: &'a Option<String>,
    base_url: &'a str,
}

impl<'a> UsersApi<'a> {
    /// Create a new UsersApi instance
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

    /// Get users by their IDs
    ///
    /// If no IDs are provided, returns the authenticated user's information.
    ///
    /// Requires OAuth token with `user:read` scope
    ///
    /// # Example
    /// ```no_run
    /// // Get specific users
    /// let users = client.users().get(vec![123, 456]).await?;
    ///
    /// // Get current authenticated user
    /// let me = client.users().get_me().await?;
    /// ```
    pub async fn get(&self, user_ids: Vec<u64>) -> Result<Vec<User>> {
        super::require_token(self.token)?;

        let url = format!("{}/users", self.base_url);
        let mut request = self
            .client
            .get(&url)
            .header("Accept", "*/*")
            .bearer_auth(self.token.as_ref().unwrap());

        // If IDs provided, add them as separate query params
        // Format: ?id=123&id=456 (not comma-separated)
        if !user_ids.is_empty() {
            for id in user_ids {
                request = request.query(&[("id", id)]);
            }
        }

        let response = crate::http::send_with_retry(self.client, request).await?;
        self.parse_response(response).await
    }

    /// Get the currently authenticated user's information
    ///
    /// This is a convenience method that calls `get()` with no IDs.
    ///
    /// Requires OAuth token with `user:read` scope
    ///
    /// # Example
    /// ```no_run
    /// let me = client.users().get_me().await?;
    /// println!("Logged in as: {}", me.name);
    /// ```
    pub async fn get_me(&self) -> Result<User> {
        let users = self.get(vec![]).await?;
        users
            .into_iter()
            .next()
            .ok_or_else(|| KickApiError::ApiError("No user data returned".to_string()))
    }

    /// Introspect an OAuth token (validate it)
    ///
    /// This validates the token passed in the Authorization header.
    /// Implements RFC 7662 OAuth 2.0 Token Introspection.
    ///
    /// **Note:** This endpoint is deprecated but still functional.
    ///
    /// Requires OAuth token (no specific scope needed)
    ///
    /// # Example
    /// ```no_run
    /// let introspection = client.users().introspect_token().await?;
    ///
    /// if introspection.is_active() {
    ///     println!("Token is valid!");
    ///     println!("Scopes: {:?}", introspection.scopes());
    ///
    ///     if introspection.has_scope("user:read") {
    ///         println!("Has user:read permission");
    ///     }
    ///
    ///     if introspection.is_expired() {
    ///         println!("Token is expired!");
    ///     }
    /// } else {
    ///     println!("Token is invalid");
    /// }
    /// ```
    pub async fn introspect_token(&self) -> Result<TokenIntrospection> {
        super::require_token(self.token)?;

        let url = format!("{}/token/introspect", self.base_url);
        let request = self
            .client
            .post(&url)
            .header("Accept", "*/*")
            .bearer_auth(self.token.as_ref().unwrap());
        let response = crate::http::send_with_retry(self.client, request).await?;

        if response.status().is_success() {
            let body = response.text().await?;

            #[derive(serde::Deserialize)]
            struct IntrospectResponse {
                data: TokenIntrospection,
            }

            let resp: IntrospectResponse = serde_json::from_str(&body)
                .map_err(|e| KickApiError::ApiError(format!("JSON parse error: {}", e)))?;

            Ok(resp.data)
        } else {
            Err(KickApiError::ApiError(format!(
                "Token introspection failed: {}",
                response.status()
            )))
        }
    }

    // Helper methods

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
}
