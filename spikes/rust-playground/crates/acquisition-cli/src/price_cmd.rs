//! The listing state's CLI surface: `acq price status | show | list` read
//! the shared store and the intent file (no daemon, no network), resolve
//! the state through `acquisition-plan` ([`resolve`], C69/C70/C80), and
//! render it under C53 — the default text is the decision view, `--expand`
//! the audit view, `--json` the [`ListingReport`] itself, of which every
//! line here is a function. Nothing here writes: `set` and `clear` are
//! the pricing slice's step 5 (`PRICING-SLICE.md`).
//!
//! Under C53: `status` opens with the one line that answers "what is
//! listed" (items, priced in game, by hand, agree, conflict, unlisted),
//! says which nothing when there is nothing, reports the rows that
//! change the next decision (unreadable, naming nothing here), and ends
//! with the next action; `--expand` adds the game side's coverage, the
//! row accounting, the bases and the versions. `list` groups items by
//! container in listing order, lists ten or fewer per group and counts
//! more, and leaves `none` out unless asked; `--expand` lists every item
//! with its texts. `show` is one target: both sides with their causes
//! and revisions, the raw note beside the parse and the tab name beside
//! its reading (the slice's done criterion), and for a container its
//! items summarized. Escapes are not used; ages are text, epochs JSON.

use std::str::FromStr;

use acquisition_core::realm::Realm;
use acquisition_plan::game_side::{GamePrice, Source};
use acquisition_plan::listing::{Listing, ListingReport, Relation, resolve};
use acquisition_plan::price::PriceTarget;
use acquisition_store::{Store, account_path};
use anyhow::{Context, Result, bail};
use serde_json::json;

use crate::plan_cmd::open_intent;
use crate::store_cmd::{ago, clip, realm_prefix};

/// Groups of this many items or fewer are listed one per line; larger
/// groups are counted (C53; `--expand` lists every one).
const LIST_UP_TO: usize = 10;

/// The state of one (realm, league), from the selected account's facts
/// and intent.
fn report(realm: Realm, league: &str) -> Result<ListingReport> {
    let (dir, entry, annotations) = open_intent()?;
    let store = Store::open(&account_path(&dir, &entry.username))?;
    let snapshot = store.pricing_snapshot(realm.as_str(), league, &annotations)?;
    resolve(&snapshot).context("resolving the listing state")
}

/// `acq price status`.
pub fn status(realm: Realm, league: &str, expand: bool, json: bool) -> Result<()> {
    let r = report(realm, league)?;
    if json {
        println!("{}", serde_json::to_string_pretty(&r.summary())?);
        return Ok(());
    }
    print!("{}", render_status(&r, acquisition_store::now(), expand));
    Ok(())
}

/// `acq price show <target>`.
pub fn show(
    target: &str,
    realm: Option<Realm>,
    league: &str,
    expand: bool,
    json: bool,
) -> Result<()> {
    let target = PriceTarget::from_str(target).map_err(|e| {
        anyhow::anyhow!(
            "{e}; a target is one word: item/<id>, character/<id>, tab/<realm>/<id> or substash/<realm>/<parent>/<id>"
        )
    })?;
    let realm = match (&target, realm) {
        (
            PriceTarget::Tab { realm: r, .. } | PriceTarget::Substash { realm: r, .. },
            Some(given),
        ) if *r != given => {
            bail!("{target} is a {r} address; --realm {given} disagrees")
        }
        (PriceTarget::Tab { realm: r, .. } | PriceTarget::Substash { realm: r, .. }, _) => *r,
        (_, given) => given.unwrap_or(Realm::DEFAULT),
    };
    let r = report(realm, league)?;
    let Some(l) = r.find(&target) else {
        bail!(
            "{target} is not in the facts for {}{league} ({} items, {} tabs and characters on record); \
             `acq items search` finds an item's id, `acq tabs` a tab's",
            realm_prefix(realm),
            r.counts.items,
            r.counts.containers
        );
    };
    if json {
        let items: Vec<&Listing> = r.items_in(&target).collect();
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "schema": r.schema,
                "realm": r.realm,
                "league": r.league,
                "taken_at": r.taken_at,
                "note_parser_version": r.note_parser_version,
                "currency_table_version": r.currency_table_version,
                "listing": l,
                "items": items,
            }))?
        );
        return Ok(());
    }
    print!("{}", render_show(&r, l, acquisition_store::now(), expand));
    Ok(())
}

/// `acq price list`.
pub fn list(
    realm: Realm,
    league: &str,
    relation: Option<&str>,
    location: Option<&str>,
    expand: bool,
    json: bool,
) -> Result<()> {
    let relation = match relation {
        None => None,
        Some(word) => Some(Relation::parse(word).ok_or_else(|| {
            anyhow::anyhow!(
                "{word:?} is not a relation (one of {})",
                Relation::ALL.map(Relation::as_str).join(", ")
            )
        })?),
    };
    let location = match location {
        None => None,
        Some(word) => {
            let t = PriceTarget::from_str(word)
                .map_err(|e| anyhow::anyhow!("{e}; --in takes a container address"))?;
            if matches!(t, PriceTarget::Item { .. }) {
                bail!("--in takes a container (tab, substash or character), not an item");
            }
            Some(t)
        }
    };
    let r = report(realm, league)?;
    let items: Vec<&Listing> = r
        .listings
        .iter()
        .filter(|l| l.subject.is_item())
        .filter(|l| match relation {
            Some(rel) => l.relation == rel,
            None => l.relation != Relation::None,
        })
        .filter(|l| {
            location
                .as_ref()
                .is_none_or(|loc| l.subject.location.as_ref() == Some(loc))
        })
        .collect();
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "schema": r.schema,
                "realm": r.realm,
                "league": r.league,
                "taken_at": r.taken_at,
                "note_parser_version": r.note_parser_version,
                "currency_table_version": r.currency_table_version,
                "relation": relation.map(Relation::as_str),
                "in": location,
                "items": items,
            }))?
        );
        return Ok(());
    }
    print!(
        "{}",
        render_list(&r, &items, relation, location.as_ref(), expand)
    );
    Ok(())
}

fn plural(n: usize, one: &str, many: &str) -> String {
    format!("{n} {}", if n == 1 { one } else { many })
}

fn count_of(r: &ListingReport, relation: Relation) -> usize {
    r.counts
        .by_relation
        .get(relation.as_str())
        .copied()
        .unwrap_or(0)
}

fn reading_count(r: &ListingReport, kind: &str) -> usize {
    r.counts.by_game_reading.get(kind).copied().unwrap_or(0)
}

/// The by-relation breakdown of a set of items, in a fixed order, zeros
/// left out.
fn breakdown(items: &[&Listing]) -> String {
    let parts: Vec<String> = Relation::ALL
        .into_iter()
        .filter_map(|rel| {
            let n = items.iter().filter(|l| l.relation == rel).count();
            (n > 0).then(|| format!("{n} {}", relation_word(rel)))
        })
        .collect();
    parts.join(", ")
}

fn relation_word(rel: Relation) -> &'static str {
    match rel {
        Relation::None => "unlisted",
        Relation::ManualOnly => "by hand only",
        Relation::GameOnly => "in game only",
        Relation::Agree => "agree",
        Relation::Conflict => "conflict",
    }
}

fn render_status(r: &ListingReport, now: i64, expand: bool) -> String {
    let mut out = String::new();
    let place = format!("{}{}", realm_prefix(r.realm), r.league);
    let c = &r.counts;
    if c.items == 0 {
        if r.stash_listing.is_none() && r.character_listing.is_none() {
            out.push_str(&format!(
                "nothing on record for {place}: no stash listing, no character listing\n"
            ));
        } else {
            out.push_str(&format!(
                "no items on record for {place}: {} listed, none fetched with items\n",
                plural(c.containers, "tab or character", "tabs and characters")
            ));
        }
        out.push_str("next: `acq refresh --plan` shows what a refresh would fetch\n");
        return out;
    }
    let game_priced = reading_count(r, "exact") + reading_count(r, "negotiable");
    let by_hand = count_of(r, Relation::ManualOnly)
        + count_of(r, Relation::Agree)
        + count_of(r, Relation::Conflict);
    out.push_str(&format!(
        "{} in {place}: {game_priced} priced in game, {by_hand} by hand; {} agree, {} conflict, {} unlisted\n",
        plural(c.items, "item", "items"),
        count_of(r, Relation::Agree),
        count_of(r, Relation::Conflict),
        count_of(r, Relation::None),
    ));
    let skipped = reading_count(r, "skip");
    let invalid = reading_count(r, "invalid");
    if skipped + invalid > 0 {
        out.push_str(&format!(
            "in game also: {skipped} skipped (~skip), {invalid} invalid notes (a broken note is not replaced by its tab's price)\n"
        ));
    }
    let rows = &r.rows;
    if !rows.unmatched.is_empty() || !rows.unreadable.is_empty() || rows.other_realm > 0 {
        out.push_str(&format!(
            "rows: {} name nothing in these facts, {} unreadable, {} for other realms\n",
            rows.unmatched.len(),
            rows.unreadable.len(),
            rows.other_realm
        ));
    }
    if expand {
        out.push_str(&format!(
            "game side: {} priced tab names ({} public); {} priced items in public tabs, {} not public; note parser v{}, currency table v{}\n",
            c.priced_tabs,
            c.priced_tabs_public,
            c.game_priced_public,
            c.game_priced_not_public,
            r.note_parser_version,
            r.currency_table_version
        ));
        out.push_str(&format!(
            "by hand: {} of {} rows apply here; {} items inherit a container's row\n",
            rows.applied, rows.total, c.inherited
        ));
        let basis = |b: Option<acquisition_store::ListingBasis>| match b {
            Some(b) => format!(
                "response {} {}",
                b.response_id,
                ago(now, Some(b.fetched_at))
            ),
            None => "none".into(),
        };
        out.push_str(&format!(
            "basis: stash listing {}, character listing {}; snapshot {}\n",
            basis(r.stash_listing),
            basis(r.character_listing),
            ago(now, Some(r.taken_at))
        ));
        if !rows.unmatched.is_empty() {
            out.push_str("rows naming nothing in these facts:\n");
            for t in &rows.unmatched {
                out.push_str(&format!("  {t}\n"));
            }
        }
        if !rows.unreadable.is_empty() {
            out.push_str("rows that cannot be read:\n");
            for p in &rows.unreadable {
                out.push_str(&format!(
                    "  {}/{} revision {}: {}\n",
                    p.scope, p.key, p.revision, p.why
                ));
            }
        }
    }
    let next = if count_of(r, Relation::Conflict) > 0 {
        "`acq price list --relation conflict` names each conflict"
    } else if !rows.unreadable.is_empty() {
        "`acq price status --expand` lists the rows that cannot be read"
    } else {
        "`acq price list` groups the listed items by tab; `acq price show <target>` is one listing"
    };
    out.push_str(&format!("next: {next}\n"));
    out
}

fn manual_cell(l: &Listing) -> String {
    match &l.manual {
        Some(m) => m.value.to_string(),
        None if l.manual_problem.is_some() => "unreadable".into(),
        None => "-".into(),
    }
}

fn game_cell(l: &Listing) -> String {
    match &l.game.reading {
        GamePrice::None => "-".into(),
        GamePrice::Invalid { .. } => "invalid".into(),
        other => other.to_string(),
    }
}

fn item_line(l: &Listing) -> String {
    let s = &l.subject;
    let mut label = s.label();
    if !s.name.is_empty() && !s.type_line.is_empty() && s.name != s.type_line {
        label = format!("{} {}", s.name, s.type_line);
    }
    if let Some(n) = s.stack_size {
        label = format!("{label} x{n}");
    }
    let PriceTarget::Item { id } = &s.target else {
        return String::new();
    };
    format!(
        "  {:<13} {:<18} {:<18} {:<40} {}\n",
        l.relation.as_str(),
        clip(&manual_cell(l), 18),
        clip(&game_cell(l), 18),
        clip(&label, 40),
        id
    )
}

/// The audit lines under an item: each text verbatim beside its reading.
fn item_texts(l: &Listing) -> String {
    let mut out = String::new();
    let g = &l.game;
    if let Some(n) = &g.note {
        out.push_str(&format!("      note {:?} reads {}\n", n.text, n.reading));
    }
    if let Some(t) = &g.tab_name {
        out.push_str(&format!(
            "      tab {} {:?} reads {}, {}{}\n",
            g.tab.as_deref().unwrap_or("?"),
            t.text,
            t.reading,
            public_word(g.public),
            g.substash_name
                .as_deref()
                .map(|n| format!("; substash {n:?}"))
                .unwrap_or_default()
        ));
    }
    if let Some(m) = &l.manual {
        out.push_str(&format!(
            "      row {} revision {}{}\n",
            m.from,
            m.revision,
            if m.inherited { " (inherited)" } else { "" }
        ));
    }
    if let Some(p) = &l.manual_problem {
        out.push_str(&format!("      by hand: {p}\n"));
    }
    out
}

fn public_word(public: Option<bool>) -> &'static str {
    match public {
        Some(true) => "public",
        Some(false) => "not public",
        None => "no stash to publish",
    }
}

/// A container's name with what its tab name says and whether it is
/// public — the context a group's items share, said once (C53).
fn container_label(r: &ListingReport, c: &Listing) -> String {
    let mut name = if c.subject.name.is_empty() {
        "(unnamed)".to_string()
    } else {
        c.subject.name.clone()
    };
    if c.subject.tab_type.is_none() {
        return name;
    }
    // A substash under its parent: the parent's name is the one read.
    if let PriceTarget::Substash { realm, parent, .. } = &c.subject.target {
        let parent = PriceTarget::Tab {
            realm: *realm,
            id: parent.clone(),
        };
        let parent_name = r
            .find(&parent)
            .map(|p| p.subject.label())
            .unwrap_or_else(|| "(parent not on record)".into());
        name = format!("{parent_name} / {name}");
    }
    let name = clip(&name, 60);
    let mut notes = Vec::new();
    if c.game.reading != GamePrice::None {
        notes.push(game_cell(c));
    }
    if let Some(public) = c.game.public {
        notes.push(public_word(Some(public)).to_string());
    }
    if notes.is_empty() {
        name
    } else {
        format!("{name} ({})", notes.join(", "))
    }
}

fn render_list(
    r: &ListingReport,
    items: &[&Listing],
    relation: Option<Relation>,
    location: Option<&PriceTarget>,
    expand: bool,
) -> String {
    let mut out = String::new();
    let place = format!("{}{}", realm_prefix(r.realm), r.league);
    let scope = match (relation, location) {
        (Some(rel), Some(loc)) => format!(" with relation {rel} in {loc}"),
        (Some(rel), None) => format!(" with relation {rel}"),
        (None, Some(loc)) => format!(" in {loc}"),
        (None, None) => String::new(),
    };
    if items.is_empty() {
        let nothing = if r.counts.items == 0 {
            "no items on record".to_string()
        } else if relation.is_none() {
            format!(
                "{} items on record, none listed by hand or in game",
                r.counts.items
            )
        } else {
            format!("{} items on record, none match", r.counts.items)
        };
        out.push_str(&format!("{nothing} for {place}{scope}\n"));
        out.push_str("next: `acq price status` says what is on record\n");
        return out;
    }
    // Groups by container, in the containers' order (tabs in listing
    // order, then characters); a location not on record comes last.
    let mut groups: Vec<(&PriceTarget, Vec<&Listing>)> = r
        .listings
        .iter()
        .filter(|c| !c.subject.is_item())
        .map(|c| (&c.subject.target, Vec::new()))
        .collect();
    for l in items {
        let Some(loc) = &l.subject.location else {
            continue;
        };
        match groups.iter_mut().find(|(t, _)| *t == loc) {
            Some((_, v)) => v.push(l),
            None => groups.push((loc, vec![l])),
        }
    }
    groups.retain(|(_, v)| !v.is_empty());
    out.push_str(&format!(
        "{} listed in {place}{scope}: {}; in {}\n",
        plural(items.len(), "item", "items"),
        breakdown(items),
        plural(groups.len(), "container", "containers")
    ));
    for (loc, group) in &groups {
        let label = r
            .find(loc)
            .map(|c| container_label(r, c))
            .unwrap_or_else(|| "(not on record)".into());
        out.push_str(&format!(
            "{label}  {}: {}  {loc}\n",
            plural(group.len(), "item", "items"),
            breakdown(group)
        ));
        if group.len() <= LIST_UP_TO || expand {
            for l in group {
                out.push_str(&item_line(l));
                if expand {
                    out.push_str(&item_texts(l));
                }
            }
        }
    }
    if !expand && groups.iter().any(|(_, g)| g.len() > LIST_UP_TO) {
        out.push_str("next: `--expand` lists every item; `acq price show item/<id>` is one\n");
    } else {
        out.push_str("next: `acq price show item/<id>` is one listing with its texts\n");
    }
    out
}

fn render_show(r: &ListingReport, l: &Listing, now: i64, expand: bool) -> String {
    let mut out = String::new();
    let s = &l.subject;
    let mut head = format!("{}: {}", s.target, s.label());
    if !s.name.is_empty() && !s.type_line.is_empty() && s.name != s.type_line {
        head = format!("{}: {} {}", s.target, s.name, s.type_line);
    }
    if let Some(n) = s.stack_size {
        head.push_str(&format!(" x{n}"));
    }
    if let Some(t) = &s.tab_type {
        head.push_str(&format!(" ({t})"));
    }
    if let Some(loc) = &s.location {
        let name = r.find(loc).map(|c| c.subject.label()).unwrap_or_default();
        head.push_str(&format!("  in {name:?} {loc}"));
        if let Some(c) = &s.container
            && c != "items"
        {
            head.push_str(&format!(" ({c})"));
        }
        if let Some(p) = &s.socketed_in {
            head.push_str(&format!(", socketed in {p}"));
        }
    }
    out.push_str(&head);
    out.push('\n');
    out.push_str(&format!("relation {}: {}\n", l.relation, l.why));

    match (&l.manual, &l.manual_problem) {
        (Some(m), _) => {
            let from = if m.inherited {
                format!("inherited from {}", m.from)
            } else {
                "own row".into()
            };
            let mut line = format!(
                "by hand: {} ({from}, revision {}, set {}",
                m.value,
                m.revision,
                ago(now, Some(m.updated_at))
            );
            if expand {
                line.push_str(&format!(" via {}", m.written_via));
                if let Some(a) = &m.actor {
                    line.push_str(&format!(" as {a}"));
                }
            }
            line.push_str(")\n");
            out.push_str(&line);
        }
        (None, Some(p)) => out.push_str(&format!("by hand: unreadable — {p}\n")),
        (None, None) => out.push_str("by hand: nothing (no row on it or above it)\n"),
    }

    let g = &l.game;
    let from = match g.source {
        Some(Source::Note) => " from the note",
        Some(Source::TabName) => " from the tab name",
        None => "",
    };
    out.push_str(&format!("in game: {}{from}\n", g.reading));
    if let Some(n) = &g.note {
        out.push_str(&format!("  note {:?} reads {}\n", n.text, n.reading));
    }
    match &g.tab_name {
        Some(t) => out.push_str(&format!(
            "  tab {} {:?} reads {}, {}\n",
            g.tab.as_deref().unwrap_or("?"),
            t.text,
            t.reading,
            public_word(g.public)
        )),
        None => {
            let why = if s.tab_type.as_deref() == Some("Folder") {
                "a folder's name is never a price (C80)"
            } else if matches!(
                s.target,
                PriceTarget::Character { .. } | PriceTarget::Item { .. }
            ) && g.substash_name.is_none()
                && s.location
                    .as_ref()
                    .is_none_or(|t| matches!(t, PriceTarget::Character { .. }))
            {
                "no stash to publish"
            } else {
                "the parent tab is not on record"
            };
            out.push_str(&format!("  no tab name read: {why}\n"));
        }
    }
    if let Some(n) = &g.substash_name {
        out.push_str(&format!(
            "  substash {n:?} (its parent's name and public are read, C80)\n"
        ));
    }

    if expand {
        let seen = match (l.basis.response, l.basis.at) {
            (Some(id), at) => format!("response {id} {}", ago(now, at)),
            (None, Some(at)) => ago(now, Some(at)),
            (None, None) => "never".into(),
        };
        out.push_str(&format!(
            "basis: {seen}; note parser v{}, currency table v{}; snapshot {}\n",
            r.note_parser_version,
            r.currency_table_version,
            ago(now, Some(r.taken_at))
        ));
    }

    if !s.is_item() {
        let items: Vec<&Listing> = r.items_in(&s.target).collect();
        if items.is_empty() {
            out.push_str("items: none on record here\n");
        } else {
            out.push_str(&format!(
                "items: {} — {}\n",
                plural(items.len(), "item", "items"),
                breakdown(&items)
            ));
            if items.len() <= LIST_UP_TO || expand {
                for l in &items {
                    out.push_str(&item_line(l));
                    if expand {
                        out.push_str(&item_texts(l));
                    }
                }
            } else {
                out.push_str(&format!(
                    "next: `acq price list --in {}` lists them; `--expand` here lists every one\n",
                    s.target
                ));
                return out;
            }
        }
    }
    let next = match &s.location {
        Some(loc) => format!("`acq price list --in {loc}` is its neighbours"),
        None => "`acq price show item/<id>` is one item".into(),
    };
    out.push_str(&format!("next: {next}\n"));
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use acquisition_store::{
        AnnotationRow, CharacterSnapshot, ItemSnapshot, ListingBasis, PricingSnapshot, TabSnapshot,
    };
    use serde_json::{Value, json};

    fn tab(id: &str, parent: Option<&str>, name: &str, r#type: &str, public: bool) -> TabSnapshot {
        TabSnapshot {
            id: id.into(),
            parent: parent.map(str::to_string),
            name: name.into(),
            r#type: r#type.into(),
            idx: None,
            listed_at: Some(1_000),
            listed_response: Some(2),
            fetched_at: Some(1_100),
            metadata: if public {
                json!({ "public": true })
            } else {
                Value::Null
            },
            item_count: 0,
        }
    }

    fn item(id: &str, kind: &str, location: &str, note: Option<&str>) -> ItemSnapshot {
        ItemSnapshot {
            id: id.into(),
            location_kind: kind.into(),
            location_id: location.into(),
            container: Some("items".into()),
            socketed_in: None,
            name: String::new(),
            type_line: "Chaos Orb".into(),
            stack_size: Some(10),
            x: Some(0),
            y: Some(0),
            note: note.map(str::to_string),
            seen_response: Some(7),
            last_seen: 1_200,
        }
    }

    fn row(scope: &str, key: &str, value: Value, revision: i64) -> AnnotationRow {
        AnnotationRow {
            scope: scope.into(),
            key: key.into(),
            kind: "buyout".into(),
            value,
            revision,
            created_at: 900,
            updated_at: 950,
            written_via: "cli".into(),
            actor: Some("tom".into()),
        }
    }

    /// A priced public tab with twelve items (one noted, one ignored by
    /// hand), a map tab with a substash holding one item, a character
    /// wearing a noted item, and one row naming nothing here.
    fn snapshot() -> PricingSnapshot {
        let mut items = vec![item("i-worn", "character", "ch1", Some("~b/o 2 divine"))];
        for i in 0..12 {
            items.push(item(&format!("i-{i:02}"), "stash", "c1", None));
        }
        items[2].note = Some("~price 5 chaos".into()); // i-01, the row's item
        items.push(item("i-sub", "stash", "s1", None));
        PricingSnapshot {
            account_uuid: "u-1".into(),
            account_name: Some("tom".into()),
            realm: "pc".into(),
            league: "Standard".into(),
            taken_at: 2_000,
            stash_listing: Some(ListingBasis {
                response_id: 2,
                fetched_at: 1_000,
            }),
            tabs: vec![
                tab("c1", None, "~price 3 chaos (A)", "PremiumStash", true),
                tab(
                    "m1",
                    None,
                    "~price 1 divine (Remove-only)",
                    "MapStash",
                    false,
                ),
                tab("s1", Some("m1"), "1 (Remove-only)", "MapStash", false),
            ],
            character_listing: Some(ListingBasis {
                response_id: 3,
                fetched_at: 1_001,
            }),
            characters: vec![CharacterSnapshot {
                id: "ch1".into(),
                name: "Exile".into(),
                league: Some("Standard".into()),
                listed_at: Some(1_001),
                listed_response: Some(3),
                fetched_at: Some(1_101),
                listed: Value::Null,
                fetched: Value::Null,
            }],
            items,
            buyouts: vec![
                row("item", "i-01", json!({ "version": 1, "type": "ignore" }), 4),
                row(
                    "item",
                    "i-gone",
                    json!({ "version": 1, "type": "ignore" }),
                    5,
                ),
            ],
        }
    }

    fn report() -> ListingReport {
        resolve(&snapshot()).unwrap()
    }

    /// C53 — `status` opens with the one answer, names the rows that
    /// change the next decision, ends with the next action; `--expand`
    /// adds the coverage, the accounting and the bases; an empty league
    /// says which nothing.
    #[test]
    fn c53_price_status_opens_with_one_line_and_ends_with_the_next_action() {
        let r = report();
        let text = render_status(&r, 8_000, false);
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(
            lines[0],
            "14 items in Standard: 14 priced in game, 1 by hand; 0 agree, 1 conflict, 0 unlisted"
        );
        assert_eq!(
            lines[1],
            "rows: 1 name nothing in these facts, 0 unreadable, 0 for other realms"
        );
        assert_eq!(
            lines[2],
            "next: `acq price list --relation conflict` names each conflict"
        );
        assert_eq!(lines.len(), 3, "{text}");
        assert!(!text.contains("parser"), "{text}");

        let text = render_status(&r, 8_000, true);
        assert!(
            text.contains("game side: 2 priced tab names (1 public); 12 priced items in public tabs, 2 not public; note parser v1, currency table v1"),
            "{text}"
        );
        assert!(
            text.contains("by hand: 1 of 2 rows apply here; 0 items inherit"),
            "{text}"
        );
        assert!(text.contains("basis: stash listing response 2 1h ago, character listing response 3 1h ago; snapshot 1h ago"), "{text}");
        assert!(
            text.contains("rows naming nothing in these facts:\n  item/i-gone\n"),
            "{text}"
        );

        let mut empty = snapshot();
        empty.items.clear();
        let text = render_status(&resolve(&empty).unwrap(), 8_000, false);
        assert!(text.starts_with("no items on record for Standard: 4 tabs and characters listed, none fetched with items\n"), "{text}");
        assert!(
            text.ends_with("next: `acq refresh --plan` shows what a refresh would fetch\n"),
            "{text}"
        );
        empty.stash_listing = None;
        empty.character_listing = None;
        let text = render_status(&resolve(&empty).unwrap(), 8_000, false);
        assert!(
            text.starts_with(
                "nothing on record for Standard: no stash listing, no character listing\n"
            ),
            "{text}"
        );
    }

    /// C53 — `list` groups by container with the shared context said
    /// once, lists ten or fewer and counts more, `--expand` lists every
    /// item with its texts, and a filter that matches nothing says so.
    #[test]
    fn c53_price_list_groups_by_container_and_lists_ten_or_fewer() {
        let r = report();
        let items: Vec<&Listing> = r
            .listings
            .iter()
            .filter(|l| l.subject.is_item() && l.relation != Relation::None)
            .collect();
        let text = render_list(&r, &items, None, None, false);
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(
            lines[0],
            "14 items listed in Standard: 12 in game only, 1 conflict, 1 unlisted; in 3 containers"
                .replace(
                    "12 in game only, 1 conflict, 1 unlisted",
                    "13 in game only, 1 conflict"
                ),
            "{text}"
        );
        assert_eq!(
            lines[1],
            "~price 3 chaos (A) (3 chaos, public)  12 items: 11 in game only, 1 conflict  tab/pc/c1"
        );
        // Twelve is more than ten: counted, not listed.
        assert_eq!(
            lines[2],
            "~price 1 divine (Remove-only) / 1 (Remove-only) (1 divine, not public)  1 item: 1 in game only  substash/pc/m1/s1"
        );
        assert!(
            lines[3]
                .starts_with("  game_only     -                  1 divine           Chaos Orb x10"),
            "{}",
            lines[3]
        );
        assert!(lines[3].ends_with("i-sub"), "{}", lines[3]);
        assert!(
            lines[4].starts_with("Exile  1 item: 1 in game only  character/ch1"),
            "{}",
            lines[4]
        );
        assert!(lines[5].contains("2 divine b/o"), "{}", lines[5]);
        assert_eq!(
            lines[6],
            "next: `--expand` lists every item; `acq price show item/<id>` is one"
        );
        assert_eq!(lines.len(), 7, "{text}");

        let text = render_list(&r, &items, None, None, true);
        assert!(
            text.contains("  conflict      ignore             5 chaos            Chaos Orb x10"),
            "{text}"
        );
        assert!(
            text.contains("      note \"~price 5 chaos\" reads 5 chaos\n"),
            "{text}"
        );
        assert!(
            text.contains("      tab c1 \"~price 3 chaos (A)\" reads 3 chaos, public\n"),
            "{text}"
        );
        assert!(text.contains("      row item/i-01 revision 4\n"), "{text}");
        assert_eq!(text.matches("      tab c1 ").count(), 12, "{text}");

        let none: Vec<&Listing> = Vec::new();
        let text = render_list(&r, &none, Some(Relation::Agree), None, false);
        assert_eq!(
            text,
            "14 items on record, none match for Standard with relation agree\nnext: `acq price status` says what is on record\n"
        );
    }

    /// The done criterion: `show` puts the raw note beside its parse and
    /// the tab name beside its reading, names the row and its revision,
    /// and summarizes a container's items under it.
    #[test]
    fn the_raw_note_sits_beside_the_parse_in_show() {
        let r = report();
        let l = r.find(&PriceTarget::Item { id: "i-01".into() }).unwrap();
        let text = render_show(&r, l, 8_000, false);
        assert_eq!(
            text,
            "item/i-01: Chaos Orb x10  in \"~price 3 chaos (A)\" tab/pc/c1\n\
             relation conflict: by hand: ignore; in game: 5 chaos — ignore leaves it out of the shop and does not deny the in-game price (C69)\n\
             by hand: ignore (own row, revision 4, set 1h ago)\n\
             in game: 5 chaos from the note\n\
             \x20 note \"~price 5 chaos\" reads 5 chaos\n\
             \x20 tab c1 \"~price 3 chaos (A)\" reads 3 chaos, public\n\
             next: `acq price list --in tab/pc/c1` is its neighbours\n"
        );
        let text = render_show(&r, l, 8_000, true);
        assert!(text.contains("set 1h ago via cli as tom)"), "{text}");
        assert!(
            text.contains(
                "basis: response 7 1h ago; note parser v1, currency table v1; snapshot 1h ago\n"
            ),
            "{text}"
        );

        // A substash item: its own name beside the parent's reading.
        let l = r.find(&PriceTarget::Item { id: "i-sub".into() }).unwrap();
        let text = render_show(&r, l, 8_000, false);
        assert!(text.contains("in game: 1 divine from the tab name\n  tab m1 \"~price 1 divine (Remove-only)\" reads 1 divine, not public\n  substash \"1 (Remove-only)\" (its parent's name and public are read, C80)\n"), "{text}");
        assert!(
            text.contains("by hand: nothing (no row on it or above it)\n"),
            "{text}"
        );

        // A character item: no stash to publish.
        let l = r
            .find(&PriceTarget::Item {
                id: "i-worn".into(),
            })
            .unwrap();
        let text = render_show(&r, l, 8_000, false);
        assert!(
            text.contains("  no tab name read: no stash to publish\n"),
            "{text}"
        );

        // A container: the items under it, counted past ten.
        let l = r
            .find(&PriceTarget::Tab {
                realm: Realm::Pc,
                id: "c1".into(),
            })
            .unwrap();
        let text = render_show(&r, l, 8_000, false);
        assert!(text.starts_with("tab/pc/c1: ~price 3 chaos (A) (PremiumStash)\nrelation game_only: in game: 3 chaos; no row applies\n"), "{text}");
        assert!(
            text.contains("items: 12 items — 11 in game only, 1 conflict\n"),
            "{text}"
        );
        assert!(text.ends_with("next: `acq price list --in tab/pc/c1` lists them; `--expand` here lists every one\n"), "{text}");
        let text = render_show(&r, l, 8_000, true);
        assert_eq!(text.matches("\n  game_only ").count(), 11, "{text}");
    }
}
