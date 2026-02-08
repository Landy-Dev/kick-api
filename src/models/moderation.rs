use serde::{Deserialize, Serialize};

/// Request body for banning a user
///
/// If `duration` is provided, this is a timeout (temporary ban).
/// If `duration` is `None`, this is a permanent ban.
///
/// # Example
/// ```
/// use kick_api::BanRequest;
///
/// // Permanent ban
/// let ban = BanRequest {
///     broadcaster_user_id: 12345,
///     user_id: 67890,
///     reason: Some("Spamming".to_string()),
///     duration: None,
/// };
///
/// // 10-minute timeout
/// let timeout = BanRequest {
///     broadcaster_user_id: 12345,
///     user_id: 67890,
///     reason: Some("Cool off".to_string()),
///     duration: Some(600),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BanRequest {
    /// The broadcaster's channel where the ban applies
    pub broadcaster_user_id: u64,

    /// The user to ban
    pub user_id: u64,

    /// Reason for the ban
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,

    /// Duration in seconds (None = permanent ban)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<u32>,
}

/// Request body for unbanning a user
///
/// # Example
/// ```
/// use kick_api::UnbanRequest;
///
/// let unban = UnbanRequest {
///     broadcaster_user_id: 12345,
///     user_id: 67890,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnbanRequest {
    /// The broadcaster's channel where the unban applies
    pub broadcaster_user_id: u64,

    /// The user to unban
    pub user_id: u64,
}
