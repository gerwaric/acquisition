//! M1 (`search/BUILD-PLAN.md`, "Measurements the build owes"): the search
//! read over a whole corpus — `Store::read_corpus`, every realm — timed,
//! with what it handed over counted. Peak memory is the caller's to take
//! (`/usr/bin/time -l`), so that it is the process's and not an estimate.
//!
//! ```sh
//! /usr/bin/time -l target/release/examples/read-corpus <copy.db> [stream|hold|parse]
//! ```
//!
//! `stream` (the default) drops each item once counted, `hold` keeps every
//! item until the read ends — what a consumer that holds the corpus as
//! text would pay — and `parse` parses each body as JSON and drops it.
//!
//! Never in the gate, and never the owner's store: the file is a sqlite
//! `.backup` copy under a track's `raw/`, and any other path is refused.
//! Opening it is `Store::open`, which may write the copy's WAL side files.

use std::path::Path;
use std::time::Instant;

use acquisition_store::{RealmScope, Store};
use serde_json::json;

fn main() -> anyhow::Result<()> {
    let mut args = std::env::args().skip(1);
    let Some(path) = args.next() else {
        anyhow::bail!("usage: read-corpus <copy.db under raw/> [stream|hold|parse]");
    };
    let mode = args.next().unwrap_or_else(|| "stream".into());
    if !matches!(mode.as_str(), "stream" | "hold" | "parse") {
        anyhow::bail!("unknown mode {mode}: stream, hold or parse");
    }
    let path = Path::new(&path);
    if !path.components().any(|c| c.as_os_str() == "raw") {
        anyhow::bail!(
            "{} is not under a raw/ directory: this measures a copy",
            path.display()
        );
    }
    let opened = Instant::now();
    let store = Store::open(path)?;
    let open_ms = opened.elapsed().as_secs_f64() * 1e3;
    let started = Instant::now();
    let report = store.read_corpus(RealmScope::All, |header, items| {
        let header_ms = started.elapsed().as_secs_f64() * 1e3;
        let (mut n, mut bytes, mut unparsed) = (0u64, 0u64, 0u64);
        let mut held = Vec::new();
        for item in items {
            let item = item?;
            n += 1;
            bytes += item.body.len() as u64;
            match mode.as_str() {
                "hold" => held.push(item),
                "parse" if serde_json::from_str::<serde_json::Value>(&item.body).is_err() => {
                    unparsed += 1
                }
                _ => {}
            }
        }
        Ok(json!({
            "mode": mode,
            "revision": header.revision,
            "realms": header.realms,
            "listings": header.listings.len(),
            "locations": header.locations.len(),
            "items": n,
            "body_bytes": bytes,
            "unparsed": unparsed,
            "open_ms": open_ms,
            "header_ms": header_ms,
            "read_ms": started.elapsed().as_secs_f64() * 1e3,
        }))
    })?;
    println!("{report}");
    Ok(())
}
