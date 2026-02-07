use serde::{Deserialize, Serialize};

/// An active event subscription
#[derive(Debug, Clone, Deserialize)]
pub struct EventSubscription {
    /// Unique subscription identifier
    pub id: String,

    /// The app that created this subscription
    pub app_id: String,

    /// The broadcaster this subscription is for
    pub broadcaster_user_id: u64,

    /// Event type name (e.g., "chat.message.created")
    pub event: String,

    /// Event version
    pub version: u32,

    /// Delivery method (e.g., "webhook")
    pub method: String,

    /// When the subscription was created (ISO 8601)
    pub created_at: String,

    /// When the subscription was last updated (ISO 8601)
    pub updated_at: String,
}

/// A single event to subscribe to
#[derive(Debug, Clone, Serialize)]
pub struct SubscribeEvent {
    /// Event type name (e.g., "chat.message.created")
    pub name: String,

    /// Event version
    pub version: u32,
}

/// Request body for creating event subscriptions
///
/// # Example
/// ```
/// use kick_api::{SubscribeRequest, SubscribeEvent};
///
/// let request = SubscribeRequest {
///     broadcaster_user_id: Some(12345),
///     method: "webhook".to_string(),
///     events: vec![
///         SubscribeEvent { name: "chat.message.created".to_string(), version: 1 },
///         SubscribeEvent { name: "channel.followed".to_string(), version: 1 },
///     ],
/// };
/// ```
#[derive(Debug, Clone, Serialize)]
pub struct SubscribeRequest {
    /// The broadcaster to subscribe to events for
    #[serde(skip_serializing_if = "Option::is_none")]
    pub broadcaster_user_id: Option<u64>,

    /// Delivery method (e.g., "webhook")
    pub method: String,

    /// List of events to subscribe to
    pub events: Vec<SubscribeEvent>,
}

/// Result of a single event subscription attempt
#[derive(Debug, Clone, Deserialize)]
pub struct SubscribeResult {
    /// Event type name
    pub name: String,

    /// Event version
    pub version: u32,

    /// Subscription ID if successful
    pub subscription_id: Option<String>,

    /// Error message if subscription failed
    pub error: Option<String>,
}
