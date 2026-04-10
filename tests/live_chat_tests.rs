use kick_api::{LiveChatClient, fetch_channel_info, fetch_followed_channels};
use std::time::Duration;

/// Integration test: connect by username and read events.
///
/// Run with:
///   cargo test --test live_chat_tests -- --ignored test_connect_by_username
#[tokio::test]
#[ignore]
async fn test_connect_by_username() {
    let username = std::env::var("KICK_TEST_USERNAME")
        .unwrap_or_else(|_| "hello_kiko".to_string());

    println!("Connecting to {}'s chat...", username);
    let mut chat = LiveChatClient::connect_by_username(&username)
        .await
        .expect("Should connect by username");

    // Try to read one event with a timeout
    let result = tokio::time::timeout(
        Duration::from_secs(10),
        chat.next_event(),
    )
    .await;

    match result {
        Ok(Ok(Some(event))) => {
            println!("Received event: {}", event.event);
            assert!(!event.event.is_empty());
        }
        Ok(Ok(None)) => println!("Connection closed"),
        Ok(Err(e)) => panic!("Error: {}", e),
        Err(_) => println!("No events in 10s — channel is quiet, but connection works"),
    }

    chat.close().await.expect("Should close cleanly");
}

/// Integration test: fetch full channel info including chatroom settings and badges.
///
/// Run with:
///   cargo test --test live_chat_tests -- --ignored test_fetch_channel_info
#[tokio::test]
#[ignore]
async fn test_fetch_channel_info() {
    let username = std::env::var("KICK_TEST_USERNAME")
        .unwrap_or_else(|_| "hello_kiko".to_string());

    let info = fetch_channel_info(&username)
        .await
        .expect("Should fetch channel info");

    // Basic channel info
    println!("Channel: {} (ID: {})", info.slug, info.id);
    assert_eq!(info.slug, username);
    assert!(info.id > 0);
    assert!(info.user_id > 0);

    // Chatroom info
    println!("Chatroom ID: {}", info.chatroom.id);
    println!("Chat mode: {:?}", info.chatroom.chat_mode);
    println!("Slow mode: {} ({}s interval)", info.chatroom.slow_mode, info.chatroom.message_interval);
    println!("Followers only: {}", info.chatroom.followers_mode);
    println!("Subscribers only: {}", info.chatroom.subscribers_mode);
    assert!(info.chatroom.id > 0);

    // Subscriber badges
    println!("Subscriber badge tiers: {}", info.subscriber_badges.len());
    for badge in &info.subscriber_badges {
        println!("  {}mo: {}", badge.months, badge.badge_image.src);
        assert!(badge.months > 0);
        assert!(!badge.badge_image.src.is_empty());
    }

    // User profile
    if let Some(user) = &info.user {
        println!("User: {} (bio: {:?})", user.username, user.bio);
        assert!(!user.username.is_empty());
    }

    // Livestream (may be None if offline)
    match &info.livestream {
        Some(stream) => {
            println!("LIVE: {} — {} viewers", stream.session_title.as_deref().unwrap_or("(no title)"), stream.viewer_count);
            assert!(stream.is_live);
        }
        None => println!("Channel is offline"),
    }

    println!("Followers: {}, Verified: {}", info.followers_count, info.verified);
}

/// Integration test: fetch channel info and use chatroom ID to connect.
///
/// Run with:
///   cargo test --test live_chat_tests -- --ignored test_fetch_then_connect
#[tokio::test]
#[ignore]
async fn test_fetch_then_connect() {
    let username = std::env::var("KICK_TEST_USERNAME")
        .unwrap_or_else(|_| "hello_kiko".to_string());

    // Step 1: fetch channel info
    let info = fetch_channel_info(&username)
        .await
        .expect("Should fetch channel info");

    let chatroom_id = info.chatroom.id;
    println!("Resolved {} -> chatroom {}", username, chatroom_id);

    // Step 2: connect using the chatroom ID
    let mut chat = LiveChatClient::connect(chatroom_id)
        .await
        .expect("Should connect with resolved chatroom ID");

    // Step 3: try to read an event
    let result = tokio::time::timeout(Duration::from_secs(10), chat.next_event()).await;

    match result {
        Ok(Ok(Some(event))) => println!("Got event: {}", event.event),
        Ok(Ok(None)) => println!("Connection closed"),
        Ok(Err(e)) => panic!("Error: {}", e),
        Err(_) => println!("No events in 10s — connection works"),
    }

    chat.close().await.expect("Should close cleanly");
}

/// Integration test: connect to a real Kick chatroom and read events.
///
/// This test connects to a chatroom, waits briefly for any event, then
/// disconnects. It verifies that the WebSocket handshake and Pusher
/// subscription work correctly.
///
/// Note: This test hits a real WebSocket server and may be slow or flaky
/// depending on network conditions. It is marked #[ignore] so it only
/// runs when explicitly requested.
///
/// Run with:
///   cargo test --test live_chat_tests -- --ignored
#[tokio::test]
#[ignore]
async fn test_connect_to_chatroom() {
    // Use a known active chatroom. You can replace this with hello_kiko's
    // chatroom ID once you look it up.
    // Visit https://kick.com/api/v2/channels/hello_kiko to find the ID.
    let chatroom_id: u64 = std::env::var("KICK_TEST_CHATROOM_ID")
        .unwrap_or_else(|_| "27670567".to_string())
        .parse()
        .expect("KICK_TEST_CHATROOM_ID must be a number");

    let mut chat = LiveChatClient::connect(chatroom_id)
        .await
        .expect("Should connect to chatroom WebSocket");

    // Try to read one event with a timeout — the chatroom may be quiet
    let result = tokio::time::timeout(Duration::from_secs(10), chat.next_event()).await;

    match result {
        Ok(Ok(Some(event))) => {
            println!("Received event: {}", event.event);
            assert!(!event.event.is_empty());
            assert!(!event.data.is_empty());
        }
        Ok(Ok(None)) => {
            println!("Connection closed (chatroom may be inactive)");
        }
        Ok(Err(e)) => {
            panic!("WebSocket error: {}", e);
        }
        Err(_) => {
            // Timeout is fine — means we connected but no messages arrived
            println!("No events in 10s (chatroom is quiet) — connection works");
        }
    }

    chat.close().await.expect("Should close cleanly");
}

/// Integration test: connect and try to read a chat message.
///
/// Run with:
///   cargo test --test live_chat_tests -- --ignored
#[tokio::test]
#[ignore]
async fn test_read_chat_message() {
    let chatroom_id: u64 = std::env::var("KICK_TEST_CHATROOM_ID")
        .unwrap_or_else(|_| "27670567".to_string())
        .parse()
        .expect("KICK_TEST_CHATROOM_ID must be a number");

    let mut chat = LiveChatClient::connect(chatroom_id)
        .await
        .expect("Should connect to chatroom WebSocket");

    // Wait up to 30 seconds for a chat message
    let result = tokio::time::timeout(Duration::from_secs(30), chat.next_message()).await;

    match result {
        Ok(Ok(Some(msg))) => {
            println!("Got message from {}: {}", msg.sender.username, msg.content);
            assert!(!msg.id.is_empty(), "Message ID should not be empty");
            assert!(!msg.content.is_empty(), "Message content should not be empty");
            assert!(!msg.sender.username.is_empty(), "Sender username should not be empty");
            assert!(msg.sender.id > 0, "Sender ID should be positive");
        }
        Ok(Ok(None)) => {
            println!("Connection closed before a message arrived");
        }
        Ok(Err(e)) => {
            panic!("Error reading message: {}", e);
        }
        Err(_) => {
            println!("No chat messages in 30s — channel is quiet, but connection works");
        }
    }

    chat.close().await.expect("Should close cleanly");
}

/// Integration test: fetch followed channels (unofficial API).
///
/// Requires a valid session token in KICK_SESSION_TOKEN env var.
///
/// Run with:
///   KICK_SESSION_TOKEN=your_token cargo test --test live_chat_tests -- --ignored test_fetch_followed_channels
#[tokio::test]
#[ignore]
async fn test_fetch_followed_channels() {
    let token = std::env::var("KICK_SESSION_TOKEN")
        .expect("KICK_SESSION_TOKEN env var required for this test");

    let resp = fetch_followed_channels(&token)
        .await
        .expect("Should fetch followed channels");

    println!("Following {} channels (nextCursor: {:?}):", resp.channels.len(), resp.next_cursor);
    for ch in &resp.channels {
        let status = if ch.is_live {
            format!(
                "LIVE — {} ({} viewers)",
                ch.session_title.as_deref().unwrap_or("(no title)"),
                ch.viewer_count,
            )
        } else {
            "Offline".to_string()
        };
        println!(
            "  {} [{}] — {}",
            ch.user_username.as_deref().unwrap_or("?"),
            ch.channel_slug.as_deref().unwrap_or("?"),
            status,
        );
    }

    // Basic assertions — if we got here the API call and deserialization worked
    // The user should be following at least one channel for a meaningful test
    assert!(
        !resp.channels.is_empty(),
        "Expected at least one followed channel (is the session token valid?)"
    );

    let first = &resp.channels[0];
    assert!(first.channel_slug.is_some());
}

/// Integration test: verify ping keeps the connection alive.
///
/// Run with:
///   cargo test --test live_chat_tests -- --ignored
#[tokio::test]
#[ignore]
async fn test_send_ping() {
    let chatroom_id: u64 = std::env::var("KICK_TEST_CHATROOM_ID")
        .unwrap_or_else(|_| "27670567".to_string())
        .parse()
        .expect("KICK_TEST_CHATROOM_ID must be a number");

    let mut chat = LiveChatClient::connect(chatroom_id)
        .await
        .expect("Should connect to chatroom WebSocket");

    // Sending a ping should not error
    chat.send_ping().await.expect("Ping should succeed");

    chat.close().await.expect("Should close cleanly");
}
