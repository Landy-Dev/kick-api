use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::error::{KickApiError, Result};
use crate::models::ChannelInfo;
use crate::models::FollowedChannelsResponse;
use crate::models::live_chat::{LiveChatMessage, PusherEvent, PusherMessage};

const PUSHER_URL: &str = "wss://ws-us2.pusher.com/app/32cbd69e4b950bf97679?protocol=7&client=js&version=8.4.0&flash=false";

type WsStream = tokio_tungstenite::WebSocketStream<
    tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
>;

/// Client for receiving live chat messages over Kick's Pusher WebSocket.
///
/// **⚠️ Unofficial API** — This connects to Kick's internal Pusher WebSocket,
/// not the public API. It may change or break without notice.
///
/// Connects to the public Pusher channel for a chatroom and yields chat
/// messages in real time. **No authentication is required.**
///
/// # Connecting
///
/// There are two ways to connect:
///
/// - [`connect_by_username`](Self::connect_by_username) — pass a Kick username
///   and the chatroom ID is resolved automatically (requires `curl` on PATH).
/// - [`connect`](Self::connect) — pass a chatroom ID directly.
///
/// # Example
///
/// ```no_run
/// use kick_api::LiveChatClient;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let mut chat = LiveChatClient::connect_by_username("xqc").await?;
/// while let Some(msg) = chat.next_message().await? {
///     println!("{}: {}", msg.sender.username, msg.content);
/// }
/// # Ok(())
/// # }
/// ```
pub struct LiveChatClient {
    ws: WsStream,
}

impl std::fmt::Debug for LiveChatClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LiveChatClient").finish_non_exhaustive()
    }
}

impl LiveChatClient {
    /// Connect to a chatroom by the channel's username/slug.
    ///
    /// Looks up the chatroom ID via Kick's public API and connects to the
    /// WebSocket. No authentication is required.
    ///
    /// # Example
    /// ```no_run
    /// use kick_api::LiveChatClient;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut chat = LiveChatClient::connect_by_username("xqc").await?;
    /// while let Some(msg) = chat.next_message().await? {
    ///     println!("{}: {}", msg.sender.username, msg.content);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn connect_by_username(username: &str) -> Result<Self> {
        let info = fetch_channel_info_inner(username).await?;
        Self::connect(info.chatroom.id).await
    }

    /// Connect to a chatroom by its ID.
    ///
    /// Opens a WebSocket to Pusher and subscribes to the chatroom's public
    /// channel. No authentication is required.
    ///
    /// To find a channel's chatroom ID, visit
    /// `https://kick.com/api/v2/channels/{slug}` in a browser and look for
    /// `"chatroom":{"id":`.
    pub async fn connect(chatroom_id: u64) -> Result<Self> {
        let channel = format!("chatrooms.{chatroom_id}.v2");

        let (mut ws, _) = connect_async(PUSHER_URL)
            .await
            .map_err(KickApiError::WebSocketError)?;

        // Wait for pusher:connection_established
        wait_for_event(&mut ws, "pusher:connection_established").await?;

        // Subscribe to the chatroom channel
        let subscribe = serde_json::json!({
            "event": "pusher:subscribe",
            "data": {
                "auth": "",
                "channel": channel,
            }
        });
        ws.send(Message::Text(subscribe.to_string().into()))
            .await
            .map_err(KickApiError::WebSocketError)?;

        // Wait for subscription confirmation
        wait_for_event(&mut ws, "pusher_internal:subscription_succeeded").await?;

        Ok(Self { ws })
    }

    /// Receive the next raw Pusher event.
    ///
    /// Returns all events from the subscribed channel (chat messages, pins,
    /// subs, bans, etc.). Automatically handles Pusher-level pings and
    /// internal protocol events. Returns `None` if the connection is closed.
    pub async fn next_event(&mut self) -> Result<Option<PusherEvent>> {
        loop {
            let Some(frame) = self.ws.next().await else {
                return Ok(None);
            };

            let frame = frame.map_err(KickApiError::WebSocketError)?;

            let text = match frame {
                Message::Text(t) => t,
                Message::Close(_) => return Ok(None),
                Message::Ping(data) => {
                    self.ws
                        .send(Message::Pong(data))
                        .await
                        .map_err(KickApiError::WebSocketError)?;
                    continue;
                }
                _ => continue,
            };

            let pusher_msg: PusherMessage = match serde_json::from_str(&text) {
                Ok(m) => m,
                Err(_) => continue,
            };

            // Handle Pusher-level pings automatically
            if pusher_msg.event == "pusher:ping" {
                let pong = serde_json::json!({ "event": "pusher:pong", "data": {} });
                self.ws
                    .send(Message::Text(pong.to_string().into()))
                    .await
                    .map_err(KickApiError::WebSocketError)?;
                continue;
            }

            // Skip internal Pusher protocol events
            if pusher_msg.event.starts_with("pusher:")
                || pusher_msg.event.starts_with("pusher_internal:")
            {
                continue;
            }

            return Ok(Some(PusherEvent {
                event: pusher_msg.event,
                channel: pusher_msg.channel,
                data: pusher_msg.data,
            }));
        }
    }

    /// Receive the next chat message.
    ///
    /// Blocks until a chat message arrives. Automatically handles Pusher-level
    /// pings and skips non-chat events. Returns `None` if the connection is
    /// closed.
    pub async fn next_message(&mut self) -> Result<Option<LiveChatMessage>> {
        loop {
            let Some(event) = self.next_event().await? else {
                return Ok(None);
            };

            if event.event != "App\\Events\\ChatMessageEvent" {
                continue;
            }

            // Data is double-encoded: outer JSON has `data` as a string
            let msg: LiveChatMessage = match serde_json::from_str(&event.data) {
                Ok(m) => m,
                Err(_) => continue,
            };

            return Ok(Some(msg));
        }
    }

    /// Send a Pusher-level ping to keep the connection alive.
    pub async fn send_ping(&mut self) -> Result<()> {
        let ping = serde_json::json!({ "event": "pusher:ping", "data": {} });
        self.ws
            .send(Message::Text(ping.to_string().into()))
            .await
            .map_err(KickApiError::WebSocketError)?;
        Ok(())
    }

    /// Close the WebSocket connection.
    pub async fn close(&mut self) -> Result<()> {
        self.ws
            .close(None)
            .await
            .map_err(KickApiError::WebSocketError)?;
        Ok(())
    }
}

/// Fetch public channel information from Kick's v2 API.
///
/// **⚠️ Unofficial API** — This uses Kick's internal v2 API
/// (`/api/v2/channels/{slug}`), not the public API. It may change or break
/// without notice.
///
/// Returns chatroom settings, subscriber badges, user profile, and livestream
/// status for any channel. **No authentication required.**
///
/// This uses `curl` as a subprocess because Kick's Cloudflare protection blocks
/// HTTP libraries based on TLS fingerprinting. `curl` ships with Windows 10+,
/// macOS, and virtually all Linux distributions.
///
/// # Example
///
/// ```no_run
/// use kick_api::fetch_channel_info;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let info = fetch_channel_info("xqc").await?;
///
/// // Chatroom settings
/// println!("Chatroom ID: {}", info.chatroom.id);
/// println!("Slow mode: {}", info.chatroom.slow_mode);
/// println!("Followers only: {}", info.chatroom.followers_mode);
///
/// // Subscriber badges
/// for badge in &info.subscriber_badges {
///     println!("{}mo badge: {}", badge.months, badge.badge_image.src);
/// }
///
/// // Livestream status
/// if let Some(stream) = &info.livestream {
///     println!("{} is live with {} viewers", info.slug, stream.viewer_count);
/// }
/// # Ok(())
/// # }
/// ```
pub async fn fetch_channel_info(username: &str) -> Result<ChannelInfo> {
    fetch_channel_info_inner(username).await
}

/// Fetch the list of channels the authenticated user follows.
///
/// **⚠️ Unofficial API** — This uses Kick's internal v2 API
/// (`/api/v2/channels/followed`), not the public API. It may change or break
/// without notice.
///
/// Requires a valid session/bearer token (the same token used when logged in
/// to kick.com). This is **not** an OAuth App Access Token from the public
/// API — it is the session token from your browser cookies.
///
/// Uses `curl` as a subprocess to bypass Cloudflare TLS fingerprinting.
///
/// # Example
///
/// ```no_run
/// use kick_api::fetch_followed_channels;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let token = "your_session_token";
/// let resp = fetch_followed_channels(token).await?;
/// for ch in &resp.channels {
///     let status = if ch.is_live {
///         format!("LIVE ({} viewers)", ch.viewer_count)
///     } else {
///         "Offline".to_string()
///     };
///     println!("{}: {}",
///         ch.user_username.as_deref().unwrap_or("?"), status);
/// }
/// # Ok(())
/// # }
/// ```
pub async fn fetch_followed_channels(token: &str) -> Result<FollowedChannelsResponse> {
    let url = "https://kick.com/api/v2/channels/followed";
    let auth_header = format!("Bearer {}", token);

    let mut cmd = tokio::process::Command::new("curl");
    cmd.args([
        "-s",
        "-H", "Accept: application/json",
        "-H", "User-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/146.0.0.0 Safari/537.36",
        "-H", &format!("Authorization: {}", auth_header),
        url,
    ]);

    // Prevent a visible console window from flashing on Windows
    #[cfg(target_os = "windows")]
    {
        #[allow(unused_imports)]
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }

    let output = cmd
        .output()
        .await
        .map_err(|e| KickApiError::UnexpectedError(format!(
            "Failed to run curl (is it installed?): {}", e
        )))?;

    if !output.status.success() {
        return Err(KickApiError::ApiError(format!(
            "curl failed for followed channels: exit code {:?}",
            output.status.code()
        )));
    }

    let resp: FollowedChannelsResponse = serde_json::from_slice(&output.stdout)
        .map_err(|e| KickApiError::ApiError(format!(
            "Failed to parse followed channels response: {}", e
        )))?;

    Ok(resp)
}

async fn fetch_channel_info_inner(username: &str) -> Result<ChannelInfo> {
    let url = format!("https://kick.com/api/v2/channels/{}", username);

    let mut cmd = tokio::process::Command::new("curl");
    cmd.args(["-s", "-H", "Accept: application/json", "-H", "User-Agent: Chatterino7", &url]);

    // Prevent a visible console window from flashing on Windows
    #[cfg(target_os = "windows")]
    {
        #[allow(unused_imports)]
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }

    let output = cmd
        .output()
        .await
        .map_err(|e| KickApiError::UnexpectedError(format!(
            "Failed to run curl (is it installed?): {}", e
        )))?;

    if !output.status.success() {
        return Err(KickApiError::ApiError(format!(
            "curl failed for channel '{}': exit code {:?}",
            username,
            output.status.code()
        )));
    }

    let info: ChannelInfo = serde_json::from_slice(&output.stdout)
        .map_err(|e| KickApiError::ApiError(format!(
            "Failed to parse channel response for '{}': {}", username, e
        )))?;

    Ok(info)
}

/// Wait for a specific Pusher event on the WebSocket.
async fn wait_for_event(ws: &mut WsStream, event_name: &str) -> Result<()> {
    loop {
        let Some(frame) = ws.next().await else {
            return Err(KickApiError::UnexpectedError(format!(
                "Connection closed while waiting for '{event_name}'"
            )));
        };

        let frame = frame.map_err(KickApiError::WebSocketError)?;

        let text = match frame {
            Message::Text(t) => t,
            Message::Ping(data) => {
                ws.send(Message::Pong(data))
                    .await
                    .map_err(KickApiError::WebSocketError)?;
                continue;
            }
            _ => continue,
        };

        let msg: PusherMessage = match serde_json::from_str(&text) {
            Ok(m) => m,
            Err(_) => continue,
        };

        if msg.event == event_name {
            return Ok(());
        }
    }
}
