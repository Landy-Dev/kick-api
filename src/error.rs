use thiserror::Error;

#[derive(Error, Debug)]
pub enum KickApiError {
    #[error("HTTP request failed: {0}")]
    HttpRequestError(#[from] reqwest::Error),

    #[error("JSON serialization/deserialization error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("API returned an error: {0}")]
    ApiError(String),

    #[error("Unexpected error: {0}")]
    UnexpectedError(String),

    #[error("WebSocket error: {0}")]
    WebSocketError(#[from] tokio_tungstenite::tungstenite::Error),
}

pub type Result<T> = std::result::Result<T, KickApiError>;