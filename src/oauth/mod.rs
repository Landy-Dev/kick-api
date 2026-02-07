use oauth2::{
    AuthUrl, ClientId, ClientSecret, CsrfToken,
    PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, Scope, TokenUrl,
    basic::BasicClient,
};
use serde::Deserialize;
use std::env;

/// OAuth token response from Kick
///
/// Returned by `exchange_code()` and `refresh_token()`.
#[derive(Debug, Clone, Deserialize)]
pub struct OAuthTokenResponse {
    /// The access token for API requests
    pub access_token: String,

    /// The refresh token (use with `refresh_token()` to get a new access token)
    pub refresh_token: Option<String>,

    /// Token lifetime in seconds
    pub expires_in: u64,

    /// Space-separated list of granted scopes
    pub scope: String,

    /// Token type (typically "Bearer")
    pub token_type: String,
}

/// Holds OAuth credentials and client for Kick.com
pub struct KickOAuth {
    client: BasicClient,
}

impl KickOAuth {
    /// Creates a new OAuth client by loading credentials from environment variables
    ///
    /// Required env vars:
    /// - KICK_CLIENT_ID
    /// - KICK_CLIENT_SECRET
    /// - KICK_REDIRECT_URI
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        // Load environment variables
        let client_id = env::var("KICK_CLIENT_ID")?;
        let client_secret = env::var("KICK_CLIENT_SECRET")?;
        let redirect_uri = env::var("KICK_REDIRECT_URI")?;

        // Verify they're not empty
        if client_id.is_empty() || client_secret.is_empty() || redirect_uri.is_empty() {
            return Err("One or more OAuth credentials are empty!".into());
        }

        // Kick's OAuth endpoints
        let auth_url = AuthUrl::new("https://id.kick.com/oauth/authorize".to_string())?;
        let token_url = TokenUrl::new("https://id.kick.com/oauth/token".to_string())?;

        // Build the OAuth2 client (oauth2 4.4 API)
        let client = BasicClient::new(
            ClientId::new(client_id),
            Some(ClientSecret::new(client_secret)),
            auth_url,
            Some(token_url),
        )
        .set_redirect_uri(RedirectUrl::new(redirect_uri)?);

        Ok(Self { client })
    }

    /// Generates the authorization URL that users should visit
    ///
    /// Pass the scopes you need (must match what you configured in your Kick app)
    ///
    /// Returns (auth_url, csrf_token, pkce_verifier)
    /// - auth_url: The URL to send the user to
    /// - csrf_token: Save this! You'll verify it matches when they return
    /// - pkce_verifier: REQUIRED! Pass this to exchange_code() later
    pub fn get_authorization_url(&self, scopes: Vec<&str>) -> (String, CsrfToken, PkceCodeVerifier) {
        // Generate PKCE challenge (required by Kick)
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        let mut auth_request = self.client
            .authorize_url(CsrfToken::new_random)
            .set_pkce_challenge(pkce_challenge);

        // Add each scope
        for scope in scopes {
            auth_request = auth_request.add_scope(Scope::new(scope.to_string()));
        }

        let (auth_url, csrf_token) = auth_request.url();

        (auth_url.to_string(), csrf_token, pkce_verifier)
    }

    /// Exchanges the authorization code for an access token
    ///
    /// After the user authorizes, Kick redirects to your callback with a `code` parameter.
    /// Pass that code AND the pkce_verifier from get_authorization_url() to this function.
    ///
    /// Returns an `OAuthTokenResponse` with access_token, refresh_token, expires_in, etc.
    pub async fn exchange_code(
        &self,
        code: String,
        pkce_verifier: PkceCodeVerifier,
    ) -> Result<OAuthTokenResponse, Box<dyn std::error::Error>> {
        let client_id = env::var("KICK_CLIENT_ID")?;
        let client_secret = env::var("KICK_CLIENT_SECRET")?;
        let redirect_uri = env::var("KICK_REDIRECT_URI")?;

        let http_client = reqwest::Client::new();
        let response = http_client
            .post("https://id.kick.com/oauth/token")
            .form(&[
                ("grant_type", "authorization_code"),
                ("code", &code),
                ("client_id", &client_id),
                ("client_secret", &client_secret),
                ("redirect_uri", &redirect_uri),
                ("code_verifier", pkce_verifier.secret()),
            ])
            .send()
            .await?;

        let status = response.status();
        let body = response.text().await?;

        if status.is_success() {
            let token_response: OAuthTokenResponse = serde_json::from_str(&body)?;
            Ok(token_response)
        } else {
            Err(format!("Token exchange failed: {}", body).into())
        }
    }

    /// Refresh an access token using a refresh token
    ///
    /// When your access token expires, use the refresh token from the original
    /// `exchange_code()` response to get a new one.
    ///
    /// # Parameters
    /// - `refresh_token`: The refresh token from a previous token response
    pub async fn refresh_token(
        &self,
        refresh_token: &str,
    ) -> Result<OAuthTokenResponse, Box<dyn std::error::Error>> {
        let client_id = env::var("KICK_CLIENT_ID")?;
        let client_secret = env::var("KICK_CLIENT_SECRET")?;

        let http_client = reqwest::Client::new();
        let response = http_client
            .post("https://id.kick.com/oauth/token")
            .form(&[
                ("grant_type", "refresh_token"),
                ("refresh_token", refresh_token),
                ("client_id", &client_id),
                ("client_secret", &client_secret),
            ])
            .send()
            .await?;

        let status = response.status();
        let body = response.text().await?;

        if status.is_success() {
            let token_response: OAuthTokenResponse = serde_json::from_str(&body)?;
            Ok(token_response)
        } else {
            Err(format!("Token refresh failed: {}", body).into())
        }
    }

    /// Revoke an access or refresh token
    ///
    /// Invalidates the given token so it can no longer be used.
    ///
    /// # Parameters
    /// - `token`: The access token or refresh token to revoke
    pub async fn revoke_token(
        &self,
        token: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let client_id = env::var("KICK_CLIENT_ID")?;
        let client_secret = env::var("KICK_CLIENT_SECRET")?;

        let http_client = reqwest::Client::new();
        let response = http_client
            .post("https://id.kick.com/oauth/revoke")
            .form(&[
                ("token", token),
                ("client_id", &client_id),
                ("client_secret", &client_secret),
            ])
            .send()
            .await?;

        let status = response.status();
        if status.is_success() {
            Ok(())
        } else {
            let body = response.text().await?;
            Err(format!("Token revocation failed: {}", body).into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oauth_from_env() {
        // This will fail if env vars aren't set - that's expected
        // To test: set env vars first, then run `cargo test`
        dotenvy::dotenv().ok();

        match KickOAuth::from_env() {
            Ok(oauth) => {
                let scopes = vec!["user:read", "channel:read"];
                let (url, _csrf, _verifier) = oauth.get_authorization_url(scopes);
                println!("Auth URL: {}", url);
                assert!(url.contains("kick.com"));
                assert!(url.contains("code_challenge")); // Verify PKCE is included
            }
            Err(e) => {
                println!("Expected failure (env vars not set): {}", e);
            }
        }
    }
}
