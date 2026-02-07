# kick-api

Rust client for the [Kick.com API](https://kick.com).

Covers channels, users, chat, moderation, rewards, and event subscriptions. Handles OAuth authentication and automatic retry on rate limits (429).

## Installation

```toml
[dependencies]
kick-api = "0.1"
tokio = { version = "1", features = ["full"] }
```

## Usage

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
