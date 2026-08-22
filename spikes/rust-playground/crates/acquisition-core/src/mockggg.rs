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
        MockPolicy {
            name,
            windows,
            hits: VecDeque::new(),
            restricted_until: None,
        }
    }

    fn hits_within(&self, period: u64, now: Instant) -> u32 {
        self.hits
            .iter()
            .filter(|&&h| now.duration_since(h) < Duration::from_secs(period))
            .count() as u32
    }

    /// Apply one request. Returns `(ok, extra response headers)`; when
    /// `!ok` the caller answers 429. `counts` is false for HEAD.
    fn request(&mut self, counts: bool, now: Instant) -> (bool, String) {
        let longest = self.windows.iter().map(|w| w.1).max().unwrap_or(0);
        while self
            .hits
            .front()
            .is_some_and(|&h| now.duration_since(h) >= Duration::from_secs(longest))
        {
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
            if let Some(w) = self
                .windows
                .iter()
                .find(|w| self.hits_within(w.1, now) >= w.0)
            {
                self.restricted_until = Some(now + Duration::from_secs(w.2));
                ok = false;
            } else {
                self.hits.push_back(now);
            }
        }
        let limits: Vec<String> = self
            .windows
            .iter()
            .map(|w| format!("{}:{}:{}", w.0, w.1, w.2))
            .collect();
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

/// The mock account's stash tree, shaped like the real API answered on
/// 2026-08-20: folders carry their children in the *list*; map/unique tabs
/// reveal substash stubs (with `metadata.items` counts and `parent`) only
/// when fetched; substashes carry the items. `(id, name, type, items,
/// parent)`; folders and map/unique parents have no items of their own.
const MOCK_TABS: &[(&str, &str, &str, usize, Option<&str>)] = &[
    ("cur1", "Currency", "CurrencyStash", 3, None),
    ("dump", "Dump", "PremiumStash", 5, None),
    ("fold", "Old leagues", "Folder", 0, None),
    ("old1", "Old 1", "PremiumStash", 2, Some("fold")),
    ("old2", "Old 2", "QuadStash", 2, Some("fold")),
    ("uniq", "Uniques", "UniqueStash", 0, None),
    ("u001", "", "UniqueStash", 2, Some("uniq")),
    ("u002", "", "UniqueStash", 3, Some("uniq")),
    ("u003", "", "UniqueStash", 4, Some("uniq")),
    ("maps", "Maps", "MapStash", 0, None),
    ("m001", "", "MapStash", 1, Some("maps")),
    ("m002", "", "MapStash", 2, Some("maps")),
    ("m003", "", "MapStash", 3, Some("maps")),
    ("m004", "", "MapStash", 4, Some("maps")),
    ("mape", "Maps (empty)", "MapStash", 0, None),
];

fn mock_tab(
    id: &str,
) -> Option<&'static (
    &'static str,
    &'static str,
    &'static str,
    usize,
    Option<&'static str>,
)> {
    MOCK_TABS.iter().find(|t| t.0 == id)
}

fn mock_items(tab: &str, n: usize) -> Vec<serde_json::Value> {
    (0..n)
        .map(|i| {
            json!({
                "id": format!("{tab}-item{i}"), "name": "", "typeLine": format!("Fake Item {i}"),
                "baseType": format!("Fake Item {i}"), "w": 1, "h": 1, "x": i, "y": 0,
                "inventoryId": "Stash1", "league": "Standard", "frameType": 0, "identified": true,
            })
        })
        .collect()
}

/// Substash stubs as the real API lists them on a fetched map/unique tab.
fn mock_children_stubs(parent: &str) -> Vec<serde_json::Value> {
    MOCK_TABS
        .iter()
        .filter(|t| t.4 == Some(parent) && t.3 > 0)
        .map(|t| {
            let mut md = json!({ "items": t.3 });
            if t.2 == "MapStash" {
                md["map"] = json!({ "index": 0, "name": format!("Map (Tier {})", t.3), "section": format!("tier{}", t.3) });
            }
            json!({ "id": t.0, "name": t.1, "type": t.2, "parent": parent, "metadata": md })
        })
        .collect()
}

/// The stash list: top-level tabs, folders with nested children, no items.
fn mock_stash_list(league: &str) -> serde_json::Value {
    let top: Vec<serde_json::Value> = MOCK_TABS
        .iter()
        .enumerate()
        .filter(|(_, t)| t.4.is_none())
        .map(|(i, t)| {
            let mut v = json!({ "id": t.0, "name": t.1, "type": t.2, "index": i, "metadata": { "colour": "7c5436" }, "league": league });
            if t.2 == "Folder" {
                v["children"] = json!(MOCK_TABS
                    .iter()
                    .enumerate()
                    .filter(|(_, c)| c.4 == Some(t.0))
                    .map(|(j, c)| json!({ "id": c.0, "name": c.1, "type": c.2, "index": j, "folder": t.0, "metadata": { "colour": "7c5436", "folder": true } }))
                    .collect::<Vec<_>>());
            }
            v
        })
        .collect();
    json!({ "stashes": top })
}

/// One fetched tab or substash.
fn mock_stash(id: &str, sub: Option<&str>) -> Option<serde_json::Value> {
    let t = mock_tab(sub.unwrap_or(id))?;
    if sub.is_some() && t.4 != Some(id) {
        return None; // a substash is only reachable under its own parent
    }
    let mut v = json!({ "id": t.0, "name": t.1, "type": t.2, "index": 0, "metadata": { "colour": "7c5436" }, "items": mock_items(t.0, t.3) });
    if let Some(p) = t.4 {
        v["parent"] = json!(p);
        v["metadata"]["items"] = json!(t.3);
    }
    if matches!(t.2, "MapStash" | "UniqueStash") && t.4.is_none() {
        let stubs = mock_children_stubs(t.0);
        if !stubs.is_empty() {
            v["children"] = json!(stubs);
        }
    }
    Some(json!({ "stash": v }))
}

/// Start the provider on an ephemeral port; returns its base URL.
pub async fn start() -> Result<String> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let base = format!("http://127.0.0.1:{}", listener.local_addr()?.port());
    let codes: Arc<Mutex<HashMap<String, PendingCode>>> = Arc::default();
    // Real policy shapes from the first capture (N23): /character is the
    // real character-list policy; /fetch borrows character-request-limit's
    // shape under a mock name so the limiter sees two independent policies.
    let policies: Policies = Arc::new(Mutex::new(HashMap::from([
        (
            "/character",
            MockPolicy::new(
                "character-list-request-limit",
                &[(2, 10, 60), (5, 300, 300)],
            ),
        ),
        (
            "/stash",
            MockPolicy::new("stash-list-request-limit", &[(10, 15, 60), (30, 60, 300)]),
        ),
        (
            "/stash/tab",
            MockPolicy::new("stash-request-limit", &[(15, 10, 60), (30, 300, 300)]),
        ),
        (
            "/fetch",
            MockPolicy::new("mock-fetch-request-limit", &[(5, 10, 60), (30, 300, 300)]),
        ),
        (
            "/token",
            MockPolicy::new("token-request-limit", &[(60, 30, 30)]),
        ),
    ])));
    tokio::spawn(async move {
        loop {
            let Ok((stream, _)) = listener.accept().await else {
                break;
            };
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
    let Some(req) = read_request(&mut stream).await else {
        return;
    };
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
                respond(
                    &mut stream,
                    "400 Bad Request",
                    "text/plain",
                    "missing params",
                )
                .await;
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
            let location = format!("{redirect_uri}?code={code}&state={}", urlencode(state));
            let head = format!(
                "HTTP/1.1 302 Found\r\nLocation: {location}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
            );
            let _ = stream.write_all(head.as_bytes()).await;
        }
        // Fake data endpoints mirroring GGG's `GET /character` (same job code
        // path in mock and real mode) plus `/fetch`, each behind its own
        // truthfully simulated rate-limit policy.
        ("GET" | "HEAD", path)
            if path == "/character" || path == "/fetch" || path.starts_with("/stash/") =>
        {
            // `/stash/{league}` is the list; `/stash/{league}/{id}[/{sub}]` is
            // one tab, under its own policy (N7).
            let stash_parts: Vec<&str> = path.trim_start_matches("/stash/").split('/').collect();
            let policy_key = if path.starts_with("/stash/") {
                if stash_parts.len() >= 2 {
                    "/stash/tab"
                } else {
                    "/stash"
                }
            } else {
                path
            };
            let authed = req
                .headers
                .get("authorization")
                .is_some_and(|v| v.starts_with("Bearer "));
            if policy_key != "/fetch" && !authed {
                respond(
                    &mut stream,
                    "401 Unauthorized",
                    "application/json",
                    &json!({ "error": "no bearer token" }).to_string(),
                )
                .await;
                return;
            }
            let (ok, extra) = policies
                .lock()
                .unwrap()
                .get_mut(policy_key)
                .expect("policy for path")
                .request(req.method == "GET", Instant::now());
            if req.method == "HEAD" {
                // ACQ_MOCK_DEGRADED_HEAD=1 reproduces the Dec-2023 regression
                // (N20): policy name present, every other header missing.
                let extra = if std::env::var_os("ACQ_MOCK_DEGRADED_HEAD").is_some() {
                    extra
                        .lines()
                        .filter(|l| l.starts_with("X-Rate-Limit-Policy"))
                        .map(|l| format!("{l}\r\n"))
                        .collect()
                } else {
                    extra
                };
                respond_with(
                    &mut stream,
                    "204 No Content",
                    "application/json",
                    &extra,
                    "",
                )
                .await;
                return;
            }
            if !ok {
                respond_with(
                    &mut stream,
                    "429 Too Many Requests",
                    "application/json",
                    &extra,
                    &json!({ "error": "rate limited" }).to_string(),
                )
                .await;
                return;
            }
            let body = if policy_key == "/stash/tab" {
                match mock_stash(stash_parts[1], stash_parts.get(2).copied()) {
                    Some(v) => v,
                    None => {
                        respond(
                            &mut stream,
                            "404 Not Found",
                            "application/json",
                            &json!({ "error": "no such stash" }).to_string(),
                        )
                        .await;
                        return;
                    }
                }
            } else if policy_key == "/stash" {
                mock_stash_list(stash_parts[0])
            } else if req.path == "/character" {
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
            respond_with(
                &mut stream,
                "200 OK",
                "application/json",
                &extra,
                &body.to_string(),
            )
            .await;
        }
        ("POST", "/token") => {
            let (ok, extra) = policies
                .lock()
                .unwrap()
                .get_mut("/token")
                .expect("token policy")
                .request(true, Instant::now());
            if !ok {
                respond_with(
                    &mut stream,
                    "429 Too Many Requests",
                    "application/json",
                    &extra,
                    &json!({ "error": "rate limited" }).to_string(),
                )
                .await;
                return;
            }
            let form: HashMap<String, String> = url::form_urlencoded::parse(req.body.as_bytes())
                .into_owned()
                .collect();
            let reply = token_reply(&form, &codes);
            match reply {
                Ok(body) => {
                    respond_with(&mut stream, "200 OK", "application/json", &extra, &body).await;
                }
                Err(msg) => {
                    let body = json!({ "error": msg }).to_string();
                    respond_with(
                        &mut stream,
                        "400 Bad Request",
                        "application/json",
                        &extra,
                        &body,
                    )
                    .await;
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
    let query = url::form_urlencoded::parse(raw_query.as_bytes())
        .into_owned()
        .collect();
    Some(HttpRequest {
        method,
        path,
        raw_query,
        query,
        headers,
        body: String::from_utf8_lossy(&body).into_owned(),
    })
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
