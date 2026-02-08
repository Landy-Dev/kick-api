mod channels;
mod chat;
mod events;
mod moderation;
mod rewards;
mod users;

pub use channels::ChannelsApi;
pub use chat::ChatApi;
pub use events::EventsApi;
pub use moderation::ModerationApi;
pub use rewards::RewardsApi;
pub use users::UsersApi;

pub(crate) fn require_token(token: &Option<String>) -> crate::error::Result<()> {
    if token.is_none() {
        return Err(crate::error::KickApiError::ApiError(
            "OAuth token required for this endpoint".to_string(),
        ));
    }
    Ok(())
}
