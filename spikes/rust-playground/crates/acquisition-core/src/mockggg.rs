//! A tiny fake OAuth authorization server standing in for GGG's.
//!
//! Hosted by the daemon on 127.0.0.1 so the whole login flow — browser page,
//! authorize redirect, PKCE-checked token exchange, refresh rotation — runs
//! for real without a single packet leaving the machine. The HTTP handling is
//! deliberately minimal; this is scaffolding, not a web server.

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use anyhow::Result;
use serde_json::json;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use crate::auth;

const USERNAME: &str = "ExileTester";
const ACCESS_TOKEN_TTL_SECONDS: u64 = 60;

struct PendingCode {
    challenge: String,
    redirect_uri: String,
}

/// A server-side rate-limit policy, simulated truthfully enough to test the
/// limiter against: real sliding windows, real restrictions, real 429s with
/// `Retry-After`, state headers that are post-increment and 1:1 (N25), and
/// HEADs that report but don't count (N24). Not simulated: the server's
/// timing-bucket quantization (N11–N12) — the limiter pads for it anyway.
struct MockPolicy {
    name: &'static str,
    /// `(max_hits, period, restriction)` per window, initial first (N23).
    windows: &'static [(u32, u64, u64)],
    hits: VecDeque<Instant>,
    restricted_until: Option<Instant>,
}

impl MockPolicy {
    fn new(name: &'static str, windows: &'static [(u32, u64, u64)]) -> Self {
        MockPolicy { name, windows, hits: VecDeque::new(), restricted_until: None }
    }

    fn hits_within(&self, period: u64, now: Instant) -> u32 {
        self.hits.iter().filter(|&&h| now.duration_since(h) < Duration::from_secs(period)).count() as u32
    }

    /// Apply one request. Returns `(ok, extra response headers)`; when
    /// `!ok` the caller answers 429. `counts` is false for HEAD.
    fn request(&mut self, counts: bool, now: Instant) -> (bool, String) {
        let longest = self.windows.iter().map(|w| w.1).max().unwrap_or(0);
        while self.hits.front().is_some_and(|&h| now.duration_since(h) >= Duration::from_secs(longest)) {
            self.hits.pop_front();
        }
        if self.restricted_until.is_some_and(|t| t <= now) {
            self.restricted_until = None;
        }
        let mut ok = true;
        if self.restricted_until.is_some() {
            ok = false;
        } else if counts {
            // The request that would exceed a window is rejected and starts
            // that window's restriction.
            if let Some(w) = self.windows.iter().find(|w| self.hits_within(w.1, now) >= w.0) {
                self.restricted_until = Some(now + Duration::from_secs(w.2));
                ok = false;
            } else {
                self.hits.push_back(now);
            }
        }
        let limits: Vec<String> = self.windows.iter().map(|w| format!("{}:{}:{}", w.0, w.1, w.2)).collect();
        let restricted_for = self
            .restricted_until
            .map(|t| t.saturating_duration_since(now).as_secs_f64().ceil() as u64)
            .unwrap_or(0);
        let state: Vec<String> = self
            .windows
            .iter()
            .map(|w| format!("{}:{}:{}", self.hits_within(w.1, now), w.1, restricted_for))
            .collect();
        let mut headers = format!(
            "X-Rate-Limit-Policy: {}\r\nX-Rate-Limit-Rules: Account\r\nX-Rate-Limit-Account: {}\r\nX-Rate-Limit-Account-State: {}\r\n",
            self.name,
            limits.join(","),
            state.join(","),
        );
        if !ok {
            headers.push_str(&format!("Retry-After: {restricted_for}\r\n"));
        }
        (ok, headers)
    }
}

type Policies = Arc<Mutex<HashMap<&'static str, MockPolicy>>>;

/// Start the provider on an ephemeral port; returns its base URL.
pub async fn start() -> Result<String> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let base = format!("http://127.0.0.1:{}", listener.local_addr()?.port());
    let codes: Arc<Mutex<HashMap<String, PendingCode>>> = Arc::default();
    // Real policy shapes from the first capture (N23): /character is the
    // real character-list policy; /fetch borrows character-request-limit's
    // shape under a mock name so the limiter sees two independent policies.
    let policies: Policies = Arc::new(Mutex::new(HashMap::from([
        ("/character", MockPolicy::new("character-list-request-limit", &[(2, 10, 60), (5, 300, 300)])),
        ("/fetch", MockPolicy::new("mock-fetch-request-limit", &[(5, 10, 60), (30, 300, 300)])),
    ])));
    tokio::spawn(async move {
        loop {
            let Ok((stream, _)) = listener.accept().await else { break };
            tokio::spawn(handle(stream, codes.clone(), policies.clone()));
        }
    });
    Ok(base)
}

async fn handle(
    mut stream: TcpStream,
    codes: Arc<Mutex<HashMap<String, PendingCode>>>,
    policies: Policies,
) {
    let Some(req) = read_request(&mut stream).await else { return };
    match (req.method.as_str(), req.path.as_str()) {
        ("GET", "/authorize") => {
            let client_id = req.query.get("client_id").cloned().unwrap_or_default();
            let page = format!(
                "<h1>Fake GGG OAuth</h1>\
                 <p>This is the playground's mock provider — no real accounts here.</p>\
                 <p>Pretend you just logged in as <b>{USERNAME}</b>.</p>\
                 <p><a href=\"/approve?{}\">Authorize {client_id}</a></p>",
                req.raw_query,
            );
            respond(&mut stream, "200 OK", "text/html", &page).await;
        }
        ("GET", "/approve") => {
            let (Some(redirect_uri), Some(state), Some(challenge)) = (
                req.query.get("redirect_uri"),
                req.query.get("state"),
                req.query.get("code_challenge"),
            ) else {
                respond(&mut stream, "400 Bad Request", "text/plain", "missing params").await;
                return;
            };
            let code = auth::random_token("code");
            codes.lock().unwrap().insert(
                code.clone(),
                PendingCode {
                    challenge: challenge.clone(),
                    redirect_uri: redirect_uri.clone(),
                },
            );
            let location = format!(
                "{redirect_uri}?code={code}&state={}",
                urlencode(state)
            );
            let head = format!(
                "HTTP/1.1 302 Found\r\nLocation: {location}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
            );
            let _ = stream.write_all(head.as_bytes()).await;
        }
        // Fake data endpoints mirroring GGG's `GET /character` (same job code
        // path in mock and real mode) plus `/fetch`, each behind its own
        // truthfully simulated rate-limit policy.
        ("GET" | "HEAD", "/character" | "/fetch") => {
            let authed = req
                .headers
                .get("authorization")
                .is_some_and(|v| v.starts_with("Bearer "));
            if req.path == "/character" && !authed {
                respond(&mut stream, "401 Unauthorized", "application/json",
                        &json!({ "error": "no bearer token" }).to_string()).await;
                return;
            }
            let (ok, extra) = policies
                .lock()
                .unwrap()
                .get_mut(req.path.as_str())
                .expect("policy for path")
                .request(req.method == "GET", Instant::now());
            if req.method == "HEAD" {
                respond_with(&mut stream, "204 No Content", "application/json", &extra, "").await;
                return;
            }
            if !ok {
                respond_with(&mut stream, "429 Too Many Requests", "application/json", &extra,
                             &json!({ "error": "rate limited" }).to_string()).await;
                return;
            }
            let body = if req.path == "/character" {
                json!({
                    "characters": [
                        { "id": "fake0001", "name": "StashHoarder", "realm": "pc",
                          "class": "Scion", "league": "Standard", "level": 97 },
                        { "id": "fake0002", "name": "MuleQuadTab", "realm": "pc",
                          "class": "Witch", "league": "Standard", "level": 12 },
                    ],
                })
            } else {
                json!({
                    "items": [
                        { "name": "Headhunter", "type": "Leather Belt" },
                        { "name": "Tabula Rasa", "type": "Simple Robe" },
                    ],
                })
            };
            respond_with(&mut stream, "200 OK", "application/json", &extra, &body.to_string()).await;
        }
        ("POST", "/token") => {
            let form: HashMap<String, String> =
                url::form_urlencoded::parse(req.body.as_bytes()).into_owned().collect();
            let reply = token_reply(&form, &codes);
            match reply {
                Ok(body) => respond(&mut stream, "200 OK", "application/json", &body).await,
                Err(msg) => {
                    let body = json!({ "error": msg }).to_string();
                    respond(&mut stream, "400 Bad Request", "application/json", &body).await;
                }
            }
        }
        _ => respond(&mut stream, "404 Not Found", "text/plain", "not found").await,
    }
}

fn token_reply(
    form: &HashMap<String, String>,
    codes: &Mutex<HashMap<String, PendingCode>>,
) -> Result<String, String> {
    match form.get("grant_type").map(String::as_str) {
        Some("authorization_code") => {
            let (Some(code), Some(verifier), Some(redirect_uri)) = (
                form.get("code"),
                form.get("code_verifier"),
                form.get("redirect_uri"),
            ) else {
                return Err("invalid_request".into());
            };
            let Some(pending) = codes.lock().unwrap().remove(code) else {
                return Err("invalid_grant: unknown or reused code".into());
            };
            if auth::s256(verifier) != pending.challenge {
                return Err("invalid_grant: PKCE verification failed".into());
            }
            if *redirect_uri != pending.redirect_uri {
                return Err("invalid_grant: redirect_uri mismatch".into());
            }
            Ok(tokens())
        }
        // Stateless on purpose: any well-shaped refresh token survives a
        // provider restart, so keyring persistence works across daemon lives.
        Some("refresh_token") => match form.get("refresh_token") {
            Some(rt) if rt.starts_with("rt-") => Ok(tokens()),
            _ => Err("invalid_grant: bad refresh token".into()),
        },
        _ => Err("unsupported_grant_type".into()),
    }
}

fn tokens() -> String {
    json!({
        "access_token": auth::random_token("at"),
        "refresh_token": auth::random_token("rt"),
        "expires_in": ACCESS_TOKEN_TTL_SECONDS,
        "token_type": "Bearer",
        "username": USERNAME,
    })
    .to_string()
}

// ---- minimal HTTP plumbing (shared with the daemon's loopback listener) --

pub struct HttpRequest {
    pub method: String,
    pub path: String,
    pub raw_query: String,
    pub query: HashMap<String, String>,
    /// Lowercased header names.
    pub headers: HashMap<String, String>,
    pub body: String,
}

pub async fn read_request(stream: &mut TcpStream) -> Option<HttpRequest> {
    let mut buf = Vec::new();
    let mut tmp = [0u8; 4096];
    let header_end = loop {
        let n = stream.read(&mut tmp).await.ok()?;
        if n == 0 {
            return None;
        }
        buf.extend_from_slice(&tmp[..n]);
        if let Some(pos) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
            break pos;
        }
        if buf.len() > 65536 {
            return None;
        }
    };
    let head = std::str::from_utf8(&buf[..header_end]).ok()?.to_string();
    let mut lines = head.lines();
    let mut parts = lines.next()?.split_whitespace();
    let method = parts.next()?.to_string();
    let target = parts.next()?.to_string();
    let headers: HashMap<String, String> = lines
        .filter_map(|l| l.split_once(':'))
        .map(|(k, v)| (k.to_ascii_lowercase(), v.trim().to_string()))
        .collect();
    let content_length = headers
        .get("content-length")
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(0);
    let mut body = buf[header_end + 4..].to_vec();
    while body.len() < content_length {
        let n = stream.read(&mut tmp).await.ok()?;
        if n == 0 {
            break;
        }
        body.extend_from_slice(&tmp[..n]);
    }
    body.truncate(content_length);
    let (path, raw_query) = match target.split_once('?') {
        Some((p, q)) => (p.to_string(), q.to_string()),
        None => (target, String::new()),
    };
    let query = url::form_urlencoded::parse(raw_query.as_bytes()).into_owned().collect();
    Some(HttpRequest { method, path, raw_query, query, headers, body: String::from_utf8_lossy(&body).into_owned() })
}

pub async fn respond(stream: &mut TcpStream, status: &str, content_type: &str, body: &str) {
    respond_with(stream, status, content_type, "", body).await;
}

/// `extra_headers` must be zero or more full "Name: value\r\n" lines.
pub async fn respond_with(
    stream: &mut TcpStream,
    status: &str,
    content_type: &str,
    extra_headers: &str,
    body: &str,
) {
    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}; charset=utf-8\r\n{extra_headers}Content-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    let _ = stream.write_all(response.as_bytes()).await;
}

pub fn urlencode(s: &str) -> String {
    url::form_urlencoded::byte_serialize(s.as_bytes()).collect()
}
