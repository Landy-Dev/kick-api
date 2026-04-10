use serde::Deserialize;

/// Paginated response from the followed channels endpoint.
///
/// **⚠️ Unofficial API** — This uses Kick's internal v2 API, not the public
/// API. It may break without notice.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FollowedChannelsResponse {
    /// Cursor for fetching the next page. `None` when there are no more results.
    #[serde(default)]
    pub next_cursor: Option<u64>,

    /// The list of followed channels.
    #[serde(default)]
    pub channels: Vec<FollowedChannel>,
}

/// A followed channel from Kick's unofficial v2 API.
///
/// Returned inside [`FollowedChannelsResponse`] by
/// [`fetch_followed_channels`](crate::fetch_followed_channels).
///
/// **⚠️ Unofficial API** — This uses Kick's internal v2 API, not the public
/// API. It may break without notice.
#[derive(Debug, Clone, Deserialize)]
pub struct FollowedChannel {
    /// Whether the channel is currently live
    #[serde(default)]
    pub is_live: bool,

    /// Profile picture URL
    #[serde(default)]
    pub profile_picture: Option<String>,

    /// Channel URL slug (lowercase)
    #[serde(default)]
    pub channel_slug: Option<String>,

    /// Current viewer count (0 if offline)
    #[serde(default)]
    pub viewer_count: u64,

    /// Category name (e.g. "Just Chatting", "IRL"). Empty string if offline.
    #[serde(default)]
    pub category_name: Option<String>,

    /// Display username
    #[serde(default)]
    pub user_username: Option<String>,

    /// Current stream title (`None` if offline)
    #[serde(default)]
    pub session_title: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_followed_channels_response() {
        let json = r##"{
            "nextCursor": 5,
            "channels": [
                {
                    "is_live": true,
                    "profile_picture": "https://files.kick.com/images/user/57253/profile_image/thumb.webp",
                    "channel_slug": "knut",
                    "viewer_count": 151,
                    "category_name": "IRL",
                    "user_username": "Knut",
                    "session_title": "NPC Show + Iron World Fit Week expo"
                },
                {
                    "is_live": false,
                    "profile_picture": "https://files.kick.com/images/user/73899717/profile_image/thumb.webp",
                    "channel_slug": "anxstasia",
                    "viewer_count": 0,
                    "category_name": "",
                    "user_username": "anxstasia",
                    "session_title": null
                }
            ]
        }"##;

        let resp: FollowedChannelsResponse = serde_json::from_str(json).unwrap();

        assert_eq!(resp.next_cursor, Some(5));
        assert_eq!(resp.channels.len(), 2);

        // Live channel
        let ch = &resp.channels[0];
        assert!(ch.is_live);
        assert_eq!(ch.channel_slug, Some("knut".into()));
        assert_eq!(ch.viewer_count, 151);
        assert_eq!(ch.category_name, Some("IRL".into()));
        assert_eq!(ch.user_username, Some("Knut".into()));
        assert!(ch.session_title.is_some());
        assert!(ch.profile_picture.is_some());

        // Offline channel
        let ch2 = &resp.channels[1];
        assert!(!ch2.is_live);
        assert_eq!(ch2.channel_slug, Some("anxstasia".into()));
        assert_eq!(ch2.viewer_count, 0);
        assert!(ch2.session_title.is_none());
    }

    #[test]
    fn test_deserialize_empty_followed_response() {
        let json = r##"{"nextCursor": null, "channels": []}"##;
        let resp: FollowedChannelsResponse = serde_json::from_str(json).unwrap();
        assert!(resp.next_cursor.is_none());
        assert!(resp.channels.is_empty());
    }

    #[test]
    fn test_deserialize_minimal_followed_channel() {
        let json = r##"{"channels": [{"is_live": false}]}"##;
        let resp: FollowedChannelsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.channels.len(), 1);
        assert!(!resp.channels[0].is_live);
        assert!(resp.channels[0].channel_slug.is_none());
        assert_eq!(resp.channels[0].viewer_count, 0);
    }
}
