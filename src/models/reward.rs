use serde::{Deserialize, Serialize};

/// Channel reward structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelReward {
    /// Unique identifier (ULID)
    pub id: String,

    /// Reward title (max 50 characters)
    pub title: String,

    /// Reward description (max 200 characters)
    pub description: String,

    /// Cost in channel points (minimum 1)
    pub cost: u32,

    /// Whether the reward is enabled
    #[serde(default = "default_true")]
    pub is_enabled: bool,

    /// Whether redemptions are paused
    #[serde(default)]
    pub is_paused: bool,

    /// Whether user input is required when redeeming
    #[serde(default)]
    pub is_user_input_required: bool,

    /// Whether redemptions skip the request queue (auto-accept)
    #[serde(default)]
    pub should_redemptions_skip_request_queue: bool,

    /// Background color (hex color code)
    #[serde(default = "default_color")]
    pub background_color: String,
}

/// Request body for creating a new reward
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRewardRequest {
    pub title: String,
    pub cost: u32,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_enabled: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_paused: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_user_input_required: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub should_redemptions_skip_request_queue: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub background_color: Option<String>,
}

/// Request body for updating a reward
#[derive(Debug, Clone, Serialize, Default)]
pub struct UpdateRewardRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_enabled: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_paused: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_user_input_required: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub should_redemptions_skip_request_queue: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub background_color: Option<String>,
}

/// Channel reward redemption
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelRewardRedemption {
    /// Unique identifier (ULID)
    pub id: String,

    /// When the reward was redeemed (ISO 8601)
    pub redeemed_at: String,

    /// User who redeemed the reward
    pub redeemer: RedemptionUser,

    /// Redemption status
    pub status: RedemptionStatus,

    /// User-provided input (if required)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_input: Option<String>,
}

/// User information in a redemption
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedemptionUser {
    pub user_id: u64,
}

/// Redemption status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RedemptionStatus {
    Pending,
    Accepted,
    Rejected,
}

/// Failed redemption (when batch operations fail)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailedRedemption {
    /// Redemption ID that failed
    pub id: String,

    /// Reason for failure
    pub reason: FailureReason,
}

/// Reasons why a redemption operation failed
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FailureReason {
    Unknown,
    NotPending,
    NotFound,
    NotOwned,
}

/// Request body for accepting/rejecting redemptions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManageRedemptionsRequest {
    /// Redemption IDs (1-25 ULIDs)
    pub ids: Vec<String>,
}

/// Response when accepting/rejecting redemptions
#[derive(Debug, Clone, Deserialize)]
pub struct ManageRedemptionsResponse {
    /// Successfully processed redemptions
    pub data: Vec<ChannelRewardRedemption>,

    /// Failed redemptions with reasons
    #[serde(default)]
    pub failed: Vec<FailedRedemption>,
}

// Helper functions for serde defaults
fn default_true() -> bool {
    true
}

fn default_color() -> String {
    "#00e701".to_string()
}
