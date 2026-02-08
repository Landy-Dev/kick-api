use crate::api::{ChannelsApi, ChatApi, EventsApi, ModerationApi, RewardsApi, UsersApi};

const KICK_BASE_URL: &str = "https://api.kick.com/public/v1";

/// Main Kick API client
///
/// # Example
/// ```no_run
/// use kick_api::KickApiClient;
///
/// // Without authentication (limited endpoints)
/// let client = KickApiClient::new();
///
/// // With OAuth token
/// let client = KickApiClient::with_token("your_token_here".to_string());
///
/// // Use the API modules
/// let channel = client.channels().get("xqc").await?;
/// let rewards = client.rewards().get_all().await?;
/// ```
#[derive(Debug, Clone)]
pub struct KickApiClient {
    base_url: String,
    client: reqwest::Client,
    oauth_token: Option<String>,
}

impl KickApiClient {
    /// Create a new client without authentication (for public endpoints only)
    pub fn new() -> Self {
        KickApiClient {
            base_url: KICK_BASE_URL.to_string(),
            client: reqwest::Client::new(),
            oauth_token: None,
        }
    }

    /// Create a client with OAuth authentication
    ///
    /// # Parameters
    /// - `token`: Your OAuth access token from the authorization flow
    pub fn with_token(token: String) -> Self {
        KickApiClient {
            base_url: KICK_BASE_URL.to_string(),
            client: reqwest::Client::new(),
            oauth_token: Some(token),
        }
    }

    /// Access the Channels API
    ///
    /// # Example
    /// ```no_run
    /// let channel = client.channels().get("xqc").await?;
    /// let my_channels = client.channels().get_mine().await?;
    /// ```
    pub fn channels(&self) -> ChannelsApi<'_> {
        ChannelsApi::new(&self.client, &self.oauth_token, &self.base_url)
    }

    /// Access the Rewards API
    ///
    /// # Example
    /// ```no_run
    /// let rewards = client.rewards().get_all().await?;
    /// let reward = client.rewards().create(request).await?;
    /// ```
    pub fn rewards(&self) -> RewardsApi<'_> {
        RewardsApi::new(&self.client, &self.oauth_token, &self.base_url)
    }

    /// Access the Users API
    ///
    /// # Example
    /// ```no_run
    /// let me = client.users().get_me().await?;
    /// let users = client.users().get(vec![123, 456]).await?;
    /// let token_info = client.users().introspect_token().await?;
    /// ```
    pub fn users(&self) -> UsersApi<'_> {
        UsersApi::new(&self.client, &self.oauth_token, &self.base_url)
    }

    /// Access the Chat API
    ///
    /// # Example
    /// ```no_run
    /// let response = client.chat().send_message(request).await?;
    /// client.chat().delete_message("msg_id").await?;
    /// ```
    pub fn chat(&self) -> ChatApi<'_> {
        ChatApi::new(&self.client, &self.oauth_token, &self.base_url)
    }

    /// Access the Moderation API
    ///
    /// # Example
    /// ```no_run
    /// client.moderation().ban(ban_request).await?;
    /// client.moderation().unban(unban_request).await?;
    /// ```
    pub fn moderation(&self) -> ModerationApi<'_> {
        ModerationApi::new(&self.client, &self.oauth_token, &self.base_url)
    }

    /// Access the Events/Webhooks API
    ///
    /// # Example
    /// ```no_run
    /// let subs = client.events().list(None).await?;
    /// let results = client.events().subscribe(request).await?;
    /// client.events().unsubscribe(vec!["id".to_string()]).await?;
    /// ```
    pub fn events(&self) -> EventsApi<'_> {
        EventsApi::new(&self.client, &self.oauth_token, &self.base_url)
    }
}

impl Default for KickApiClient {
    fn default() -> Self {
        Self::new()
    }
}
