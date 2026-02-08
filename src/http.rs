use std::time::Duration;

use crate::error::Result;

const MAX_RETRIES: u32 = 3;

pub(crate) async fn send_with_retry(
    client: &reqwest::Client,
    request: reqwest::RequestBuilder,
) -> Result<reqwest::Response> {
    let mut current = request.build()?;

    for attempt in 0..=MAX_RETRIES {
        // Clone before executing so we have a copy for the next retry
        let next = if attempt < MAX_RETRIES {
            current.try_clone()
        } else {
            None
        };

        let response = client.execute(current).await?;

        if response.status() == 429 && attempt < MAX_RETRIES {
            let retry_after = response
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or(1);

            tokio::time::sleep(Duration::from_secs(retry_after)).await;

            // Use the cloned request for the next attempt
            current = next.ok_or_else(|| {
                crate::error::KickApiError::UnexpectedError(
                    "request could not be cloned for retry".to_string(),
                )
            })?;
        } else {
            return Ok(response);
        }
    }

    unreachable!()
}
