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

/// A failed token request, keeping the HTTP status (when one landed) so
/// the daemon can tell a rejected grant from a network blip (L0 rail 2).
#[derive(Debug, Clone)]
pub struct TokenError {
    pub status: Option<u16>,
    pub message: String,
}

impl std::fmt::Display for TokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl TokenError {
    fn transport(message: String) -> TokenError {
        TokenError {
            status: None,
            message,
        }
    }

    /// A grant the provider rejected outright: 4xx other than 429. 5xx,
    /// 429, and transport failures are not (they may be transient).
    pub fn is_rejected_grant(&self) -> bool {
        self.status
            .is_some_and(|status| (400..500).contains(&status) && status != 429)
    }
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
) -> Result<TokenResponse, TokenError> {
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
) -> Result<TokenResponse, TokenError> {
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
) -> Result<TokenResponse, TokenError> {
    let response = choke
        .post_form("oauth-token", &provider.token_url, params, choke.now())
        .await
        .map_err(|error| TokenError::transport(error.to_string()))?;
    let status = response.status;
    if !status.is_success() {
        let rate = response.rate;
        let body = response
            .body
            .unwrap_or_else(|error| format!("<body read transport failure: {error}>"));
        let body: String = body.chars().take(300).collect();
        return Err(TokenError {
            status: Some(status.as_u16()),
            message: format!("token endpoint returned {status} (rate headers {rate}): {body}"),
        });
    }
    let body = response
        .body
        .expect("clean 2xx responses have a completed body");
    serde_json::from_str::<TokenResponse>(&body).map_err(|e| TokenError {
        status: Some(status.as_u16()),
        message: e.to_string(),
    })
}

// ---- keyring ------------------------------------------------------------
//
// Refresh tokens live in the OS keyring (invariant 5) — one JSON secret so
// username survives daemon restarts too. `ACQ_NO_KEYRING=1` degrades to
// in-memory-only sessions (still never plaintext on disk). The service name
// comes from the provider, so mock and real sessions can never cross.

// One keyring entry per account: the entry's user is the GGG username
// (`name#discriminator`), so two accounts on one provider never share a
// secret. The account index (`acquisition_store::Index`) is how a daemon
// knows which entries exist — the keyring cannot enumerate.

// Ad-hoc code signatures change on rebuild; if macOS ever prompts on reads of
// items created by an older build, the fix is signing the binary consistently.
fn entry(service: &str, username: &str) -> Result<keyring::Entry, String> {
    if std::env::var_os("ACQ_NO_KEYRING").is_some() {
        return Err("disabled by ACQ_NO_KEYRING".into());
    }
    keyring::Entry::new(service, username).map_err(|e| e.to_string())
}

pub fn keyring_save(service: &str, refresh_token: &str, username: &str) -> Result<(), String> {
    entry(service, username)?
        .set_password(refresh_token)
        .map_err(|e| e.to_string())
}

/// The stored refresh token for one account; Ok(None) means the keyring
/// works but holds no entry for it.
pub fn keyring_load(service: &str, username: &str) -> Result<Option<String>, String> {
    match entry(service, username)?.get_password() {
        Ok(secret) => Ok(Some(secret)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

pub fn keyring_clear(service: &str, username: &str) -> Result<(), String> {
    match entry(service, username)?.delete_credential() {
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
    use tokio::sync::{mpsc, oneshot};

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
        assert!(!error.message.contains("expected value"), "{error}");
        assert_eq!(
            choke.endpoint_state("oauth-token"),
            EndpointState::Policy("token-request-limit".into())
        );
        let send = choke.recent_sends().pop().unwrap();
        assert!(!send.ok);
        assert!(send.outcome.contains("body transfer failure"));
        server.await.unwrap();
    }

    #[tokio::test]
    async fn code_exchange_and_refresh_serialize_across_token_policy_discovery() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let (arrived_tx, mut arrived_rx) = mpsc::unbounded_channel();
        let (release_first, release_first_rx) = oneshot::channel();
        let (release_second, release_second_rx) = oneshot::channel();
        let server = tokio::spawn(async move {
            let releases = [release_first_rx, release_second_rx];
            let mut handlers = Vec::new();
            for (index, release) in releases.into_iter().enumerate() {
                let (mut stream, _) = listener.accept().await.unwrap();
                let request = crate::mockggg::read_request(&mut stream).await.unwrap();
                assert_eq!(request.method, "POST", "token traffic must not HEAD-probe");
                arrived_tx.send(request.body).unwrap();
                handlers.push(tokio::spawn(async move {
                    let body = serde_json::json!({
                        "access_token": format!("at-{index}"),
                        "refresh_token": format!("rt-{index}"),
                        "expires_in": 3600,
                        "username": "test-user",
                    })
                    .to_string();
                    let headers = format!(
                        concat!(
                            "HTTP/1.1 200 OK\r\n",
                            "X-Rate-Limit-Policy: token-request-limit\r\n",
                            "X-Rate-Limit-Rules: Ip\r\n",
                            "X-Rate-Limit-Ip: 60:30:30\r\n",
                            "X-Rate-Limit-Ip-State: {}:30:0\r\n",
                            "Content-Type: application/json\r\n",
                            "Content-Length: {}\r\n",
                            "Connection: close\r\n\r\n"
                        ),
                        index + 1,
                        body.len(),
                    );
                    stream.write_all(headers.as_bytes()).await.unwrap();
                    release.await.unwrap();
                    stream.write_all(body.as_bytes()).await.unwrap();
                }));
            }
            for handler in handlers {
                handler.await.unwrap();
            }
        });

        let choke = std::sync::Arc::new(ChokePoint::new());
        let provider = std::sync::Arc::new(Provider::mock(&base));
        let exchange = {
            let choke = choke.clone();
            let provider = provider.clone();
            tokio::spawn(async move {
                exchange_code(&choke, &provider, "code", "verifier", "http://callback").await
            })
        };
        let first = arrived_rx.recv().await.unwrap();
        assert!(first.contains("grant_type=authorization_code"));
        tokio::time::timeout(std::time::Duration::from_secs(1), async {
            while choke.endpoint_state("oauth-token")
                != EndpointState::Policy("token-request-limit".into())
            {
                tokio::task::yield_now().await;
            }
        })
        .await
        .expect("the first response headers discover the token policy");

        let refresh = {
            let choke = choke.clone();
            let provider = provider.clone();
            tokio::spawn(async move { refresh(&choke, &provider, "rt-old").await })
        };
        assert!(
            tokio::time::timeout(std::time::Duration::from_millis(50), arrived_rx.recv())
                .await
                .is_err(),
            "token grants overlapped before policy discovery completed"
        );

        release_first.send(()).unwrap();
        exchange.await.unwrap().unwrap();
        let second = tokio::time::timeout(std::time::Duration::from_secs(1), arrived_rx.recv())
            .await
            .expect("refresh proceeds under the learned token policy")
            .unwrap();
        assert!(second.contains("grant_type=refresh_token"));
        release_second.send(()).unwrap();
        refresh.await.unwrap().unwrap();

        assert_eq!(
            choke.endpoint_state("oauth-token"),
            EndpointState::Policy("token-request-limit".into())
        );
        server.await.unwrap();
    }
}
