//! `redact-pricing` — the writer of the real-scale pricing fixture (plan
//! step 7, item 2, `PRICING-SLICE.md`): one (realm, league)'s pricing
//! snapshot from the owner's own store, every name and id hashed, every
//! note, tab name, position and timestamp kept, written to stdout as the
//! `PricingSnapshot` JSON the suite reads back. The owner runs it, reads
//! the summary on stderr and the output, and commits the file beside the
//! price-notes corpus (`reference/`); the suite then runs the listing
//! state and the render on it. Nothing here is shipped.
//!
//! ```sh
//! cargo run -p acquisition-plan --example redact-pricing -- --league Standard \
//!     > crates/acquisition-plan/reference/pricing-snapshot-$(date +%F).json
//! ```
//!
//! Flags: `--league <name>` (required); `--realm <pc|xbox|sony|poe2>`
//! (pc); `--provider <ggg|mock>` (ggg); `--account <selector>` (the sole
//! account, else exact username, name without discriminator, or uuid —
//! `acq accounts` lists them); `--dir <provider store dir>` (the
//! platform's, `acq daemon status` prints it).
//!
//! **What it reads.** The store as every `acq` read opens it
//! (`Store::open`: WAL, the busy timeout — a daemon may be running) and
//! the account's annotations file bound to its uuid; then one
//! `pricing_snapshot` in one transaction, the same read `acq price` and
//! `acq shop render` make. No fact and no row is written; no daemon is
//! contacted; nothing touches the network.
//!
//! **What it hashes** — with a secret drawn for this run and kept nowhere,
//! so the output is consistent with itself and with nothing else: every
//! id (item, tab, substash, character, the account uuid; a tab's
//! `parent`, an item's `location_id` and `socketed_in`, the ids in a
//! buyout row's key) to a hex string of the original's length, so the
//! shapes the readers see are the owner's; every name (an item's, a
//! character's — the `name` and `id` inside its verbatim listing entry
//! and fetched envelope too — the account's, a row's `actor`) to
//! `name-<12 hex>`. **What it keeps:**
//! tab names (the prices live there), notes verbatim, positions,
//! containers, stack sizes, `inventoryId`, type lines (game vocabulary,
//! not the owner's), tab metadata (`public`, colour, the map name of a
//! substash), timestamps and response ids, leagues and realms, every
//! buyout value.

use std::io::{Read, Write};
use std::path::PathBuf;

use acquisition_plan::price::PriceTarget;
use acquisition_protocol::realm::Realm;
use acquisition_store::{
    AnnotationRow, Annotations, CharacterSnapshot, Index, ItemSnapshot, PricingSnapshot, Store,
    TabSnapshot, account_path, store_dir,
};
use anyhow::{Context, Result, anyhow, bail};
use serde_json::Value;
use sha2::{Digest, Sha256};

const USAGE: &str = "usage: cargo run -p acquisition-plan --example redact-pricing -- \
--league <name> [--realm pc] [--provider ggg] [--account <selector>] [--dir <provider store dir>] > <file>.json";

/// The per-run secret and the two hashes over it.
struct Redactor {
    key: [u8; 32],
}

impl Redactor {
    /// A secret from the OS, never written anywhere.
    fn new() -> Result<Redactor> {
        let mut key = [0u8; 32];
        std::fs::File::open("/dev/urandom")
            .and_then(|mut f| f.read_exact(&mut key))
            .context("reading 32 bytes of /dev/urandom for the run's secret")?;
        Ok(Redactor { key })
    }

    fn digest(&self, domain: &str, text: &str) -> String {
        let mut h = Sha256::new();
        h.update(self.key);
        h.update(domain.as_bytes());
        h.update([0]);
        h.update(text.as_bytes());
        h.finalize().iter().map(|b| format!("{b:02x}")).collect()
    }

    /// An id keeps its length (a 10-character tab id stays 10, a
    /// 64-character item id stays 64) so the shapes survive; the hex
    /// digest is truncated to it, never shorter than 8.
    fn id(&self, id: &str) -> String {
        let n = id.chars().count().clamp(8, 64);
        self.digest("id", id)[..n].to_string()
    }

    /// A name becomes `name-<12 hex>`; an empty name stays empty (an
    /// unnamed item has none to hide).
    fn name(&self, name: &str) -> String {
        if name.is_empty() {
            return String::new();
        }
        format!("name-{}", &self.digest("name", name)[..12])
    }

    /// Hash every `name` and every `id` string in a verbatim value (a
    /// listing entry, a fetched envelope — both carry the character's own
    /// id and name); everything else stays.
    fn scrub(&self, v: &mut Value) {
        match v {
            Value::Object(map) => {
                for (k, v) in map.iter_mut() {
                    match v {
                        Value::String(s) if k == "name" => *s = self.name(s),
                        Value::String(s) if k == "id" => *s = self.id(s),
                        other => self.scrub(other),
                    }
                }
            }
            Value::Array(items) => items.iter_mut().for_each(|v| self.scrub(v)),
            _ => {}
        }
    }

    fn tab(&self, t: TabSnapshot) -> TabSnapshot {
        TabSnapshot {
            id: self.id(&t.id),
            parent: t.parent.as_deref().map(|p| self.id(p)),
            ..t
        }
    }

    fn character(&self, c: CharacterSnapshot) -> CharacterSnapshot {
        let mut listed = c.listed;
        let mut fetched = c.fetched;
        self.scrub(&mut listed);
        self.scrub(&mut fetched);
        CharacterSnapshot {
            id: self.id(&c.id),
            name: self.name(&c.name),
            listed,
            fetched,
            ..c
        }
    }

    fn item(&self, i: ItemSnapshot) -> ItemSnapshot {
        ItemSnapshot {
            id: self.id(&i.id),
            location_id: self.id(&i.location_id),
            socketed_in: i.socketed_in.as_deref().map(|s| self.id(s)),
            name: self.name(&i.name),
            ..i
        }
    }

    /// A buyout row's key through the target grammar, so the ids inside
    /// it hash the way the facts' do and the row still names its item.
    fn buyout(&self, r: AnnotationRow) -> Result<AnnotationRow> {
        let target = PriceTarget::from_address(&r.scope, &r.key)
            .with_context(|| format!("buyout row {}/{} is not a price target", r.scope, r.key))?;
        let target = match target {
            PriceTarget::Item { id } => PriceTarget::Item { id: self.id(&id) },
            PriceTarget::Character { id } => PriceTarget::Character { id: self.id(&id) },
            PriceTarget::Tab { realm, id } => PriceTarget::Tab {
                realm,
                id: self.id(&id),
            },
            PriceTarget::Substash { realm, parent, id } => PriceTarget::Substash {
                realm,
                parent: self.id(&parent),
                id: self.id(&id),
            },
        };
        let (scope, key) = target.address()?;
        Ok(AnnotationRow {
            scope: scope.to_string(),
            key,
            actor: r.actor.as_deref().map(|a| self.name(a)),
            ..r
        })
    }

    fn snapshot(&self, s: PricingSnapshot) -> Result<PricingSnapshot> {
        Ok(PricingSnapshot {
            account_uuid: self.id(&s.account_uuid),
            account_name: s.account_name.as_deref().map(|n| self.name(n)),
            tabs: s.tabs.into_iter().map(|t| self.tab(t)).collect(),
            characters: s
                .characters
                .into_iter()
                .map(|c| self.character(c))
                .collect(),
            items: s.items.into_iter().map(|i| self.item(i)).collect(),
            buyouts: s
                .buyouts
                .into_iter()
                .map(|r| self.buyout(r))
                .collect::<Result<_>>()?,
            ..s
        })
    }
}

/// The summary the owner reads before committing: what the file holds,
/// by shape and count, never an id.
fn summary(s: &PricingSnapshot) -> String {
    let folders = s.tabs.iter().filter(|t| t.r#type == "Folder").count();
    let substashes = s
        .tabs
        .iter()
        .filter(|t| {
            t.parent
                .as_deref()
                .is_some_and(|p| s.tabs.iter().any(|q| q.id == p && q.r#type != "Folder"))
        })
        .count();
    let priced_names = s.tabs.iter().filter(|t| t.name.starts_with('~')).count();
    let noted = s.items.iter().filter(|i| i.note.is_some()).count();
    let tilde = s
        .items
        .iter()
        .filter(|i| i.note.as_deref().is_some_and(|n| n.starts_with('~')))
        .count();
    let mut by_scope = std::collections::BTreeMap::new();
    for r in &s.buyouts {
        *by_scope.entry(r.scope.as_str()).or_insert(0usize) += 1;
    }
    format!(
        "{}{}: {} tabs ({folders} folders, {substashes} substashes, {priced_names} named with a price), \
         {} characters, {} items ({noted} with a note, {tilde} of them price notes), {} buyout rows{}\n\
         names and ids hashed under this run's secret; review the file before committing it",
        if s.realm == "pc" {
            String::new()
        } else {
            format!("{}/", s.realm)
        },
        s.league,
        s.tabs.len(),
        s.characters.len(),
        s.items.len(),
        s.buyouts.len(),
        if by_scope.is_empty() {
            String::new()
        } else {
            format!(
                " ({})",
                by_scope
                    .iter()
                    .map(|(k, n)| format!("{n} {k}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        },
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
    let redacted = Redactor::new()?.snapshot(snapshot)?;

    let mut out = std::io::stdout().lock();
    serde_json::to_writer(&mut out, &redacted)?;
    out.write_all(b"\n")?;
    out.flush()?;
    eprintln!("{}", summary(&redacted));
    Ok(())
}
