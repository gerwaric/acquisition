//! OAuth client mechanics: PKCE, token exchange/refresh, keyring storage.
//!
//! The flow is real (authorization code + PKCE, loopback redirect, refresh
//! rotation, OS keyring) but always points at the localhost mock provider —
//! never GGG. Swapping in real endpoints later is a config change here, not a
//! redesign.

use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use rand::RngCore as _;
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::ratelimit::{ChokePoint, Endpoint};

pub const CLIENT_ID: &str = "acquisition-playground";

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
    provider_base: &str,
    code: &str,
    verifier: &str,
    redirect_uri: &str,
) -> Result<TokenResponse, String> {
    token_request(
        choke,
        provider_base,
        &[
            ("grant_type", "authorization_code"),
            ("client_id", CLIENT_ID),
            ("code", code),
            ("code_verifier", verifier),
            ("redirect_uri", redirect_uri),
        ],
    )
    .await
}

pub async fn refresh(
    choke: &ChokePoint,
    provider_base: &str,
    refresh_token: &str,
) -> Result<TokenResponse, String> {
    token_request(
        choke,
        provider_base,
        &[
            ("grant_type", "refresh_token"),
            ("client_id", CLIENT_ID),
            ("refresh_token", refresh_token),
        ],
    )
    .await
}

async fn token_request(
    choke: &ChokePoint,
    provider_base: &str,
    params: &[(&str, &str)],
) -> Result<TokenResponse, String> {
    let response = choke
        .post_form(Endpoint::OauthToken, &format!("{provider_base}/token"), params)
        .await?;
    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(format!("provider returned {status}: {body}"));
    }
    response.json::<TokenResponse>().await.map_err(|e| e.to_string())
}

// ---- keyring ------------------------------------------------------------
//
// Refresh tokens live in the OS keyring (invariant 5) — one JSON secret so
// username survives daemon restarts too. `ACQ_NO_KEYRING=1` degrades to
// in-memory-only sessions (still never plaintext on disk).

const KEYRING_SERVICE: &str = "acquisition-playground";
const KEYRING_USER: &str = "oauth";

// Ad-hoc code signatures change on rebuild; if macOS ever prompts on reads of
// items created by an older build, the fix is signing the binary consistently.
fn entry() -> Result<keyring::Entry, String> {
    if std::env::var_os("ACQ_NO_KEYRING").is_some() {
        return Err("disabled by ACQ_NO_KEYRING".into());
    }
    keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER).map_err(|e| e.to_string())
}

pub fn keyring_save(refresh_token: &str, username: &str) -> Result<(), String> {
    let secret = serde_json::json!({
        "refresh_token": refresh_token,
        "username": username,
    });
    entry()?.set_password(&secret.to_string()).map_err(|e| e.to_string())
}

/// Ok(None) means the keyring works but holds no session.
pub fn keyring_load() -> Result<Option<(String, String)>, String> {
    match entry()?.get_password() {
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

pub fn keyring_clear() -> Result<(), String> {
    match entry()?.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}
