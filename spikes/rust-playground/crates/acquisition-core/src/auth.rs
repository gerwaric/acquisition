//! OAuth client mechanics: PKCE, token exchange/refresh, keyring storage.
//!
//! The flow is authorization code + PKCE with a loopback redirect, refresh
//! rotation, and OS keyring storage. Which provider it points at — the
//! localhost mock or real GGG — is entirely the `Provider` passed in.

use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use rand::RngCore as _;
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::provider::{Provider, SCOPES};
use crate::ratelimit::ChokePoint;

pub fn random_token(prefix: &str) -> String {
    let mut bytes = [0u8; 24];
    rand::rng().fill_bytes(&mut bytes);
    format!("{prefix}-{}", URL_SAFE_NO_PAD.encode(bytes))
}

/// S256 code-challenge transform.
pub fn s256(verifier: &str) -> String {
    URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()))
}

/// Returns (verifier, challenge).
pub fn pkce_pair() -> (String, String) {
    let mut bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut bytes);
    let verifier = URL_SAFE_NO_PAD.encode(bytes);
    let challenge = s256(&verifier);
    (verifier, challenge)
}

#[derive(Debug, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: u64,
    pub username: String,
}

pub async fn exchange_code(
    choke: &ChokePoint,
    provider: &Provider,
    code: &str,
    verifier: &str,
    redirect_uri: &str,
) -> Result<TokenResponse, String> {
    // GGG's public-client docs include `scope` in the exchange; no secret is
    // ever sent (and must not be — an empty client_secret is a server error).
    let scope = SCOPES.join(" ");
    token_request(
        choke,
        provider,
        &[
            ("grant_type", "authorization_code"),
            ("client_id", provider.client_id),
            ("code", code),
            ("code_verifier", verifier),
            ("redirect_uri", redirect_uri),
            ("scope", &scope),
        ],
    )
    .await
}

pub async fn refresh(
    choke: &ChokePoint,
    provider: &Provider,
    refresh_token: &str,
) -> Result<TokenResponse, String> {
    token_request(
        choke,
        provider,
        &[
            ("grant_type", "refresh_token"),
            ("client_id", provider.client_id),
            ("refresh_token", refresh_token),
        ],
    )
    .await
}

async fn token_request(
    choke: &ChokePoint,
    provider: &Provider,
    params: &[(&str, &str)],
) -> Result<TokenResponse, String> {
    let response = choke
        .post_form("oauth-token", &provider.token_url, params)
        .await
        .map_err(|error| error.to_string())?;
    let status = response.status;
    if !status.is_success() {
        let rate = response.rate;
        let body = response
            .body
            .unwrap_or_else(|error| format!("<body read transport failure: {error}>"));
        let body: String = body.chars().take(300).collect();
        return Err(format!(
            "token endpoint returned {status} (rate headers {rate}): {body}"
        ));
    }
    let body = response
        .body
        .expect("clean 2xx responses have a completed body");
    serde_json::from_str::<TokenResponse>(&body).map_err(|e| e.to_string())
}

// ---- keyring ------------------------------------------------------------
//
// Refresh tokens live in the OS keyring (invariant 5) — one JSON secret so
// username survives daemon restarts too. `ACQ_NO_KEYRING=1` degrades to
// in-memory-only sessions (still never plaintext on disk). The service name
// comes from the provider, so mock and real sessions can never cross.

const KEYRING_USER: &str = "oauth";

// Ad-hoc code signatures change on rebuild; if macOS ever prompts on reads of
// items created by an older build, the fix is signing the binary consistently.
fn entry(service: &str) -> Result<keyring::Entry, String> {
    if std::env::var_os("ACQ_NO_KEYRING").is_some() {
        return Err("disabled by ACQ_NO_KEYRING".into());
    }
    keyring::Entry::new(service, KEYRING_USER).map_err(|e| e.to_string())
}

pub fn keyring_save(service: &str, refresh_token: &str, username: &str) -> Result<(), String> {
    let secret = serde_json::json!({
        "refresh_token": refresh_token,
        "username": username,
    });
    entry(service)?.set_password(&secret.to_string()).map_err(|e| e.to_string())
}

/// Ok(None) means the keyring works but holds no session.
pub fn keyring_load(service: &str) -> Result<Option<(String, String)>, String> {
    match entry(service)?.get_password() {
        Ok(secret) => {
            let v: serde_json::Value =
                serde_json::from_str(&secret).map_err(|e| format!("corrupt secret: {e}"))?;
            match (v["refresh_token"].as_str(), v["username"].as_str()) {
                (Some(rt), Some(user)) => Ok(Some((rt.to_string(), user.to_string()))),
                _ => Err("corrupt secret: missing fields".into()),
            }
        }
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

pub fn keyring_clear(service: &str) -> Result<(), String> {
    match entry(service)?.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ratelimit::EndpointState;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    #[tokio::test]
    async fn oauth_clean_200_waits_for_body_completion_before_recording_success() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut request = [0; 4096];
            let _ = stream.read(&mut request).await.unwrap();
            let response = concat!(
                "HTTP/1.1 200 OK\r\n",
                "X-Rate-Limit-Policy: token-request-limit\r\n",
                "X-Rate-Limit-Rules: Ip\r\n",
                "X-Rate-Limit-Ip: 60:30:30\r\n",
                "X-Rate-Limit-Ip-State: 1:30:0\r\n",
                "Content-Type: application/json\r\n",
                "Content-Length: 100\r\n",
                "Connection: close\r\n\r\n",
                "{}"
            );
            stream.write_all(response.as_bytes()).await.unwrap();
        });
        let choke = ChokePoint::new();
        let provider = Provider::mock(&base);

        let error = refresh(&choke, &provider, "rt-test").await.unwrap_err();
        assert!(!error.contains("expected value"), "{error}");
        assert_eq!(
            choke.endpoint_state("oauth-token"),
            EndpointState::Policy("token-request-limit".into())
        );
        let send = choke.recent_sends().pop().unwrap();
        assert!(!send.ok);
        assert!(send.outcome.contains("body transfer failure"));
        server.await.unwrap();
    }
}
