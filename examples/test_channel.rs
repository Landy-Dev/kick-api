 use kick_api::KickApiClient;

  #[tokio::main]
  async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = KickApiClient::new();

    println!("Fetching channel info for 'xqc'...");

    match client.get_channel("xqc").await {
        Ok(channel) => {
            println!("Channel: {}", channel.slug);
            println!("Stream title: {:?}", channel.stream_title);
            if let Some(stream) = &channel.stream {
                println!("Live: {}", stream.is_live);
                println!("Viewers: {}", stream.viewer_count);
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }

      Ok(())
  }