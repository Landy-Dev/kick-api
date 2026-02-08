use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::error::{KickApiError, Result};
use crate::models::live_chat::{LiveChatMessage, PusherEvent, PusherMessage};

const PUSHER_URL: &str = "wss://ws-us2.pusher.com/app/32cbd69e4b950bf97679?protocol=7&client=js&version=8.4.0&flash=false";

type WsStream = tokio_tungstenite::WebSocketStream<
    tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
>;

/// Client for receiving live chat messages over Kick's Pusher WebSocket.
///
/// This connects to the public Pusher channel for a chatroom and yields
/// chat messages in real time. No authentication is required.
///
/// The chatroom ID can be found by visiting
/// `https://kick.com/api/v2/channels/{slug}` in a browser and searching
/// for `"chatroom":{"id":`.
///
/// # Example
/// ```no_run
/// use kick_api::LiveChatClient;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let mut chat = LiveChatClient::connect(27670567).await?;
/// while let Some(msg) = chat.next_message().await? {
///     println!("{}: {}", msg.sender.username, msg.content);
/// }
/// # Ok(())
/// # }
/// ```
pub struct LiveChatClient {
    ws: WsStream,
}

impl LiveChatClient {
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
