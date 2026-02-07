use serde::{Deserialize, Serialize};

/// Channel information
///
/// Returned when fetching channel data via the `/channels` endpoint
///
/// # Example Response
/// ```json
/// {
///   "broadcaster_user_id": 123456,
///   "slug": "xqc",
///   "stream_title": "LIVE NOW",
///   "channel_description": "Welcome to my channel!"
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    /// Number of active subscribers
    pub active_subscribers_count: u32,

    /// Banner picture URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub banner_picture: Option<String>,

    /// Unique broadcaster user identifier
    pub broadcaster_user_id: u32,

    /// Number of canceled subscribers
    pub canceled_subscribers_count: u32,

    /// Current stream category
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<Category>,

    /// Channel description text
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_description: Option<String>,

    /// Channel URL slug (unique username)
    pub slug: String,

    /// Current stream information (if live)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<Stream>,

    /// Current stream title
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_title: Option<String>,
}

/// Stream category information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    /// Unique category identifier
    pub id: u32,

    /// Category name (e.g., "Just Chatting", "Fortnite")
    pub name: String,

    /// Category thumbnail URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<String>,
}

/// Live stream information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stream {
    /// Custom tags set by the streamer
    #[serde(default)]
    pub custom_tags: Vec<String>,

    /// Whether the stream is currently live
    pub is_live: bool,

    /// Whether the stream is marked as mature content
    pub is_mature: bool,

    /// Stream key identifier
    pub key: String,

    /// Stream language code (e.g., "en")
    pub language: String,

    /// When the stream started (ISO 8601)
    pub start_time: String,

    /// Stream thumbnail URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<String>,

    /// Stream URL
    pub url: String,

    /// Current viewer count
    pub viewer_count: u32,
}
