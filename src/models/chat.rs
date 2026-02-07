use serde::{Deserialize, Serialize};

/// Request body for sending a chat message
///
/// # Example
/// ```
/// use kick_api::SendMessageRequest;
///
/// let request = SendMessageRequest {
///     r#type: "user".to_string(),
///     content: "Hello chat!".to_string(),
///     broadcaster_user_id: Some(12345),
///     reply_to_message_id: None,
/// };
/// ```
#[derive(Debug, Clone, Serialize)]
pub struct SendMessageRequest {
    /// Message type (e.g., "user")
    pub r#type: String,

    /// Message content text
    pub content: String,

    /// The broadcaster's channel to send the message in
    #[serde(skip_serializing_if = "Option::is_none")]
    pub broadcaster_user_id: Option<u64>,

    /// ID of message to reply to
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_to_message_id: Option<String>,
}

/// Response from sending a chat message
#[derive(Debug, Clone, Deserialize)]
pub struct SendMessageResponse {
    /// Whether the message was successfully sent
    pub is_sent: bool,

    /// The ID of the sent message
    pub message_id: String,
}
