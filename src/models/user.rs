use serde::{Deserialize, Serialize};

/// User information
///
/// Returned when fetching user data via the `/users` endpoint
///
/// # Example Response
/// ```json
/// {
///   "user_id": 123456,
///   "name": "username",
///   "email": "user@example.com",
///   "profile_picture": "https://..."
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// Unique user identifier
    pub user_id: u64,

    /// Username
    pub name: String,

    /// Email address (only visible to the authenticated user)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,

    /// Profile picture URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_picture: Option<String>,
}

/// Token introspection response
///
/// Used to validate OAuth tokens (implements RFC 7662)
///
/// # Example Response (Active Token)
/// ```json
/// {
///   "active": true,
///   "client_id": "01XXXXX",
///   "token_type": "Bearer",
///   "scope": "user:read channel:read",
///   "exp": 1234567890
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenIntrospection {
    /// Whether the token is currently active and valid
    pub active: bool,

    /// Client ID that issued the token (only if active=true)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,

    /// Token type (e.g., "Bearer") (only if active=true)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_type: Option<String>,

    /// Space-separated list of scopes (only if active=true)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,

    /// Expiration timestamp (Unix epoch) (only if active=true)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exp: Option<i64>,
}

impl TokenIntrospection {
    /// Check if the token is active
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Get the scopes as a Vec<String>
    pub fn scopes(&self) -> Vec<String> {
        self.scope
            .as_ref()
            .map(|s| s.split_whitespace().map(String::from).collect())
            .unwrap_or_default()
    }

    /// Check if the token has a specific scope
    pub fn has_scope(&self, scope: &str) -> bool {
        self.scopes().iter().any(|s| s == scope)
    }

    /// Check if the token is expired
    pub fn is_expired(&self) -> bool {
        if let Some(exp) = self.exp {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64;
            now >= exp
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_scopes() {
        let token = TokenIntrospection {
            active: true,
            client_id: Some("test".to_string()),
            token_type: Some("Bearer".to_string()),
            scope: Some("user:read channel:read".to_string()),
            exp: Some(9999999999),
        };

        assert_eq!(token.scopes(), vec!["user:read", "channel:read"]);
        assert!(token.has_scope("user:read"));
        assert!(token.has_scope("channel:read"));
        assert!(!token.has_scope("chat:write"));
    }

    #[test]
    fn test_token_expiry() {
        let expired = TokenIntrospection {
            active: true,
            client_id: Some("test".to_string()),
            token_type: Some("Bearer".to_string()),
            scope: Some("user:read".to_string()),
            exp: Some(0), // Expired in 1970!
        };

        assert!(expired.is_expired());

        let valid = TokenIntrospection {
            active: true,
            client_id: Some("test".to_string()),
            token_type: Some("Bearer".to_string()),
            scope: Some("user:read".to_string()),
            exp: Some(9999999999), // Far future
        };

        assert!(!valid.is_expired());
    }
}
