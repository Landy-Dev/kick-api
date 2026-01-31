mod error;
mod client;

pub use error::{KickApiError, Result};
pub use client::KickApiClient;