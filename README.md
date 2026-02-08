# kick-api

Rust client for the [Kick.com API](https://kick.com).

Covers channels, users, chat, moderation, rewards, event subscriptions, and **live chat over WebSocket**. Handles OAuth authentication and automatic retry on rate limits (429).

## Installation

```toml
[dependencies]
kick-api = "0.1"
tokio = { version = "1", features = ["full"] }
```

## Live Chat (WebSocket)

Read live chat messages from any channel in real time — no authentication required.

```rust
use kick_api::LiveChatClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut chat = LiveChatClient::connect(27670567).await?;

    while let Some(msg) = chat.next_message().await? {
        println!("{}: {}", msg.sender.username, msg.content);
    }

    Ok(())
}
```

**Finding a chatroom ID:** Visit `https://kick.com/api/v2/channels/{slug}` in your browser and search for `"chatroom":{"id":`. The Kick website API is behind Cloudflare, so this must be done from a browser.

Use `next_event()` instead of `next_message()` to receive all Pusher events (subscriptions, bans, polls, etc.).

## REST API

```rust
use kick_api::KickApiClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = KickApiClient::with_token("your_oauth_token".to_string());

    // Get a channel
    let channel = client.channels().get("xqc").await?;
    println!("{}", channel.slug);

    // Get the authenticated user
    let me = client.users().get_me().await?;
    println!("{}", me.name);

    // Send a chat message
    use kick_api::SendMessageRequest;
    let msg = SendMessageRequest {
        r#type: "user".to_string(),
        content: "Hello chat!".to_string(),
        broadcaster_user_id: Some(12345),
        reply_to_message_id: None,
    };
    client.chat().send_message(msg).await?;

    Ok(())
}
```

## API Coverage

| Module | Endpoints |
|--------|-----------|
| **Live Chat** | Real-time chat messages via Pusher WebSocket (no auth) |
| **Channels** | Get by slug, get own channels |
| **Users** | Get by ID, get authenticated user, token introspection |
| **Chat** | Send message, delete message |
| **Moderation** | Ban/timeout, unban |
| **Rewards** | CRUD for channel rewards, manage redemptions |
| **Events** | List/create/delete webhook subscriptions |

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT License](LICENSE-MIT) at your option.

## Disclaimer

Unofficial library, not affiliated with Kick.com.
