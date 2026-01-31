mod error;
mod client;
mod models;

pub use error::{KickApiError, Result};
pub use client::KickApiClient;
pub use models::*;