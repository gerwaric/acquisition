//! `listing-report` — our side of the site-as-the-oracle comparison
//! (plan step 7, item 5, `PRICING-SLICE.md`; `brainstorming-notes/15`):
//! one (realm, league)'s listing state from the owner's own store,
//! written to stdout as the whole `ListingReport` JSON — the report the
//! CLI resolves and then filters into its `status`, `list` and `show`
//! views (`--json` prints the view, never the whole) — **unredacted**:
//! the ids are the join key against the trade site's rows, and the names
//! read beside them. The owner redirects it
//! to a local file under `runs/` (gitignored); `tools/site-join.py` reads
//! it against the table `tools/site-listings.py` made. Nothing here is
//! shipped and nothing here is committed.
//!
//! ```sh
//! cargo run -p acquisition-plan --example listing-report -- --league Standard \
//!     > runs/site/listing-report-$(date +%F).json
//! ```
//!
//! Flags are `redact-pricing`'s: `--league <name>` (required); `--realm
//! <pc|xbox|sony|poe2>` (pc); `--provider <ggg|mock>` (ggg); `--account
//! <selector>` (the sole account, else exact username, name without
//! discriminator, or uuid — `acq accounts` lists them); `--dir <provider
//! store dir>` (the platform's, `acq daemon status` prints it).
//!
//! **What it reads.** The store as every `acq` read opens it
//! (`Store::open`: WAL, the busy timeout — a daemon may be running) and
//! the account's annotations file bound to its uuid; then one
//! `pricing_snapshot` in one transaction, the same read `acq price` and
//! `acq shop render` make, resolved by the one pure function
//! (`listing::resolve`, C69). No fact and no row is written; no daemon is
//! contacted; nothing touches the network; `ACQ_GGG` is never read.

use std::io::Write;
use std::path::PathBuf;

use acquisition_core::realm::Realm;
use acquisition_plan::listing::{ListingReport, resolve};
use acquisition_store::{Annotations, Index, Store, account_path, store_dir};
use anyhow::{Context, Result, anyhow, bail};

const USAGE: &str = "usage: cargo run -p acquisition-plan --example listing-report -- \
--league <name> [--realm pc] [--provider ggg] [--account <selector>] [--dir <provider store dir>] > <file>.json";

/// The summary on stderr: the report's own counts, so the owner sees
/// what the file holds before the join reads it.
fn summary(r: &ListingReport) -> String {
    let c = &r.counts;
    let by = |m: &std::collections::BTreeMap<String, usize>| {
        m.iter()
            .map(|(k, n)| format!("{n} {k}"))
            .collect::<Vec<_>>()
            .join(", ")
    };
    format!(
        "{}{}: {} items in {} containers; game statements {}; effective {}; \
         {} priced tabs ({} public); {} residue; rows {} applied, {} unmatched, {} unreadable\n\
         unredacted: keep the file under runs/, never commit it",
        if r.header.realm == Realm::Pc {
            String::new()
        } else {
            format!("{}/", r.header.realm.as_str())
        },
        r.header.league,
        c.items,
        c.containers,
        by(&c.by_game_statement),
        by(&c.by_effective),
        c.priced_tabs,
        c.priced_tabs_public,
        c.residue,
        r.rows.applied,
        r.rows.unmatched.len(),
        r.rows.unreadable.len(),
    )
}

fn main() -> Result<()> {
    let mut provider = "ggg".to_string();
    let mut dir: Option<PathBuf> = None;
    let mut account: Option<String> = None;
    let mut realm = "pc".to_string();
    let mut league: Option<String> = None;
    let mut args = std::env::args().skip(1);
    while let Some(flag) = args.next() {
        let mut value = |name: &str| {
            args.next()
                .ok_or_else(|| anyhow!("{name} takes a value\n{USAGE}"))
        };
        match flag.as_str() {
            "--provider" => provider = value("--provider")?,
            "--dir" => dir = Some(PathBuf::from(value("--dir")?)),
            "--account" => account = Some(value("--account")?),
            "--realm" => realm = value("--realm")?,
            "--league" => league = Some(value("--league")?),
            "-h" | "--help" => {
                println!("{USAGE}");
                return Ok(());
            }
            other => bail!("unknown argument {other:?}\n{USAGE}"),
        }
    }
    let Some(league) = league else {
        bail!("--league is required\n{USAGE}");
    };
    let realm = Realm::parse(&realm)
        .ok_or_else(|| anyhow!("--realm {realm:?} is not a realm (pc, xbox, sony, poe2)"))?;
    let dir = dir.unwrap_or_else(|| store_dir(&provider));
    let index = Index::load(&dir)?;
    let entry = index.resolve(account.as_deref())?.clone();
    let Some(uuid) = entry.uuid.as_deref() else {
        bail!(
            "account {} has no recorded uuid; log in once with `acq auth`",
            entry.username
        );
    };
    let store = Store::open(&account_path(&dir, &entry.username))?;
    let annotations = Annotations::open_for(&dir, uuid)?;
    let snapshot = store.pricing_snapshot(realm.as_str(), &league, &annotations)?;
    let report = resolve(&snapshot).context("resolving the listing state")?;

    let mut out = std::io::stdout().lock();
    serde_json::to_writer(&mut out, &report)?;
    out.write_all(b"\n")?;
    out.flush()?;
    eprintln!("{}", summary(&report));
    Ok(())
}
