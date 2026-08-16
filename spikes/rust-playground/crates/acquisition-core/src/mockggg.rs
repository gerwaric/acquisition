//! A tiny fake OAuth authorization server standing in for GGG's.
//!
//! Hosted by the daemon on 127.0.0.1 so the whole login flow — browser page,
//! authorize redirect, PKCE-checked token exchange, refresh rotation — runs
//! for real without a single packet leaving the machine. The HTTP handling is
//! deliberately minimal; this is scaffolding, not a web server.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

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

/// Start the provider on an ephemeral port; returns its base URL.
pub async fn start() -> Result<String> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let base = format!("http://127.0.0.1:{}", listener.local_addr()?.port());
    let codes: Arc<Mutex<HashMap<String, PendingCode>>> = Arc::default();
    tokio::spawn(async move {
        loop {
            let Ok((stream, _)) = listener.accept().await else { break };
            tokio::spawn(handle(stream, codes.clone()));
        }
    });
    Ok(base)
}

async fn handle(mut stream: TcpStream, codes: Arc<Mutex<HashMap<String, PendingCode>>>) {
    let Some(req) = read_request(&mut stream).await else { return };
    match (req.method.as_str(), req.path.as_str()) {
        ("GET", "/authorize") => {
            let page = format!(
                "<h1>Fake GGG OAuth</h1>\
                 <p>This is the playground's mock provider — no real accounts here.</p>\
                 <p>Pretend you just logged in as <b>{USERNAME}</b>.</p>\
                 <p><a href=\"/approve?{}\">Authorize {}</a></p>",
                req.raw_query,
                auth::CLIENT_ID,
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
    let content_length = lines
        .filter_map(|l| l.split_once(':'))
        .find(|(k, _)| k.eq_ignore_ascii_case("content-length"))
        .and_then(|(_, v)| v.trim().parse::<usize>().ok())
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
    Some(HttpRequest { method, path, raw_query, query, body: String::from_utf8_lossy(&body).into_owned() })
}

pub async fn respond(stream: &mut TcpStream, status: &str, content_type: &str, body: &str) {
    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    let _ = stream.write_all(response.as_bytes()).await;
}

pub fn urlencode(s: &str) -> String {
    url::form_urlencoded::byte_serialize(s.as_bytes()).collect()
}
