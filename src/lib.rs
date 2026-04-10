//! # kick-api
//!
//! Rust client for the [Kick.com](https://kick.com) API.
//!
//! Covers channels, users, chat, moderation, rewards, event subscriptions, and
//! live chat over WebSocket. Handles OAuth 2.1 (PKCE) authentication and
//! automatic retry on rate limits (429).
//!
//! ## Live Chat (no auth required)
//!
//! ```no_run
//! use kick_api::LiveChatClient;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Connect by username
//! let mut chat = LiveChatClient::connect_by_username("xqc").await?;
//!
//! // Or connect by chatroom ID directly
//! // let mut chat = LiveChatClient::connect(668).await?;
//!
//! while let Some(msg) = chat.next_message().await? {
//!     println!("{}: {}", msg.sender.username, msg.content);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## REST API (requires OAuth token)
//!
//! ```no_run
//! use kick_api::{KickApiClient, SendMessageRequest};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = KickApiClient::with_token("your_oauth_token".to_string());
//!
//! // Channels
//! let channel = client.channels().get("xqc").await?;
//!
//! // Users
//! let me = client.users().get_me().await?;
//!
//! // Chat
//! let msg = SendMessageRequest {
//!     r#type: "user".to_string(),
//!     content: "Hello chat!".to_string(),
//!     broadcaster_user_id: Some(12345),
//!     reply_to_message_id: None,
//! };
//! client.chat().send_message(msg).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Unofficial API
//!
//! Some features use Kick's **internal v2 API** (`kick.com/api/v2/...`) rather
//! than the public API. These are reverse-engineered and **may break without
//! notice**. They use `curl` as a subprocess to bypass Cloudflare TLS
//! fingerprinting.
//!
//! | Function | Auth | Description |
//! |----------|------|-------------|
//! | [`LiveChatClient`] | None | Real-time chat via Pusher WebSocket |
//! | [`fetch_channel_info`] | None | Chatroom settings, badges, livestream status |
//! | [`fetch_followed_channels`] | Session token | Paginated list of followed channels ([`FollowedChannelsResponse`]) |
//!
//! ```no_run
//! use kick_api::{fetch_channel_info, fetch_followed_channels};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Public — no auth
//! let info = fetch_channel_info("xqc").await?;
//! println!("Chatroom ID: {}", info.chatroom.id);
//!
//! // Requires session token (from browser cookies, not an OAuth app token)
//! let resp = fetch_followed_channels("your_session_token").await?;
//! for ch in &resp.channels {
//!     println!("{}: {} viewers",
//!         ch.user_username.as_deref().unwrap_or("?"), ch.viewer_count);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Authentication
//!
//! Kick uses OAuth 2.1 with PKCE. Use [`KickOAuth`] to handle the flow:
//!
//! ```no_run
//! use kick_api::KickOAuth;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Load from KICK_CLIENT_ID, KICK_CLIENT_SECRET, KICK_REDIRECT_URI
//! let oauth = KickOAuth::from_env()?;
//! let scopes = vec!["chat:write", "user:read", "channel:read"];
//! let (auth_url, csrf_token, pkce_verifier) = oauth.get_authorization_url(scopes);
//! // Send the user to auth_url, then exchange the code:
//! // let token = oauth.exchange_code(code, pkce_verifier).await?;
//! # Ok(())
//! # }
//! ```
//!
//! For server-to-server access (no user interaction), use an App Access Token:
//!
//! ```no_run
//! use kick_api::KickOAuth;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Only needs KICK_CLIENT_ID and KICK_CLIENT_SECRET
//! let oauth = KickOAuth::from_env_server()?;
//! let token = oauth.get_app_access_token().await?;
//! let client = kick_api::KickApiClient::with_token(token.access_token);
//! # Ok(())
//! # }
//! ```

mod error;
mod client;
mod http;
mod live_chat;
mod models;
mod oauth;
mod api;

pub use error::{KickApiError, Result};
pub use client::KickApiClient;
pub use live_chat::{LiveChatClient, fetch_channel_info, fetch_followed_channels};
pub use models::*;
pub use oauth::{KickOAuth, OAuthTokenResponse};
pub use api::{ChannelsApi, ChatApi, EventsApi, LivestreamsApi, ModerationApi, RewardsApi, UsersApi};