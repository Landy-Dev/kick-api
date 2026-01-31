 use kick_api::KickApiClient;

  #[tokio::main]
  async fn main() -> Result<(), Box<dyn std::error::Error>> {
      let client = KickApiClient::new();

      println!("Fetching channel info for 'xqc'...");

      match client.get_channel("xqc").await {
          Ok(body) => {
              println!("Success! Response:\n{}", body);
          }
          Err(e) => {
              eprintln!("Error: {}", e);
          }
      }

      Ok(())
  }