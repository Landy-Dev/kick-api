use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    pub active_subscribers_count: u32,
    pub banner_picture: Option<String>,
    pub broadcaster_user_id: u32,
    pub canceled_subscribers_count: u32,
    pub category: Option<Category>,
    pub channel_description: Option<String>,
    pub slug: String,
    pub stream: Option<Stream>,
    pub stream_title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub id: u32,
    pub name: String,
    pub thumbnail: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stream {
    pub custom_tags: Vec<String>,
    pub is_live: bool,
    pub is_mature: bool,
    pub key: String,
    pub language: String,
    pub start_time: String,
    pub thumbnail: Option<String>,
    pub url: String,
    pub viewer_count: u32,
}