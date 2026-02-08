use serde::Deserialize;

/// Pusher wire-format message (outer envelope)
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct PusherMessage {
    pub event: String,
    pub data: String,
    #[serde(default)]
    pub channel: Option<String>,
}

/// A raw Pusher event received from the WebSocket.
///
/// Useful for debugging or handling event types beyond chat messages.
#[derive(Debug, Clone)]
pub struct PusherEvent {
    /// The Pusher event name (e.g. `App\Events\ChatMessageEvent`)
    pub event: String,
    /// The channel this event was received on, if any
    pub channel: Option<String>,
    /// The raw JSON data payload (may need a second parse — Pusher double-encodes)
    pub data: String,
}

/// A live chat message received over the Pusher WebSocket
#[derive(Debug, Clone, Deserialize)]
pub struct LiveChatMessage {
    /// Unique message identifier
    pub id: String,

    /// The chatroom this message was sent in (may not be present in all payloads)
    #[serde(default)]
    pub chatroom_id: Option<u64>,

    /// Message text content
    pub content: String,

    /// Message type (e.g. "message" or "reply")
    #[serde(rename = "type")]
    pub r#type: String,

    /// ISO 8601 timestamp of when the message was created
    #[serde(default)]
    pub created_at: Option<String>,

    /// The user who sent this message
    pub sender: ChatSender,

    /// Reply metadata, present when this message is a reply
    #[serde(default)]
    pub metadata: Option<ChatMessageMetadata>,
}

/// Metadata attached to a reply message
#[derive(Debug, Clone, Deserialize)]
pub struct ChatMessageMetadata {
    /// The original message being replied to
    #[serde(default)]
    pub original_sender: Option<OriginalSender>,

    /// The original message content
    #[serde(default)]
    pub original_message: Option<OriginalMessage>,
}

/// The sender of the message being replied to
#[derive(Debug, Clone, Deserialize)]
pub struct OriginalSender {
    pub username: String,
}

/// The content of the message being replied to
#[derive(Debug, Clone, Deserialize)]
pub struct OriginalMessage {
    pub content: String,
}

/// Sender information for a live chat message
#[derive(Debug, Clone, Deserialize)]
pub struct ChatSender {
    /// Unique user identifier
    pub id: u64,

    /// Display username
    pub username: String,

    /// URL-friendly username slug
    #[serde(default)]
    pub slug: Option<String>,

    /// Visual identity (color, badges)
    pub identity: ChatIdentity,
}

/// Visual identity information for a chat sender
#[derive(Debug, Clone, Deserialize)]
pub struct ChatIdentity {
    /// Username color hex code
    pub color: String,

    /// List of badges the user has
    pub badges: Vec<ChatBadge>,
}

/// A badge displayed next to a user's name in chat
#[derive(Debug, Clone, Deserialize)]
pub struct ChatBadge {
    /// Badge type identifier
    #[serde(rename = "type")]
    pub r#type: String,

    /// Badge display text
    pub text: String,

    /// Optional count (e.g. subscription months)
    #[serde(default)]
    pub count: Option<u32>,
}
