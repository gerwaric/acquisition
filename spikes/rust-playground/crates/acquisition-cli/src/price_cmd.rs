//! The listing state's CLI surface: `acq price status | show | list` read
//! the shared store and the intent file (no daemon, no network), resolve
//! the state through `acquisition-plan` ([`resolve`], C69/C70/C80), and
//! render it under C53 — the default text is the decision view, `--expand`
//! the audit view, `--json` the [`ListingReport`] itself, of which every
//! line here is a function. `set` and `clear` (plan step 5) are the two
//! writes: one row each through the plan crate's shared write ([`set_buyout`],
//! [`clear_buyout`]) under compare-and-swap, printing the receipt —
//! what the target is now, what it was (C78), and the command that puts
//! the prior back.
//!
//! Under C53: `status` opens with the one line that answers "what is
//! listed" (items, priced in game, by hand, agree, conflict, unlisted),
//! says which nothing when there is nothing — and still reports the
//! intent rows, since orphaned intent stays visible (C35) — reports the
//! rows that change the next decision (unreadable, naming nothing here),
//! and ends with the next action; `--expand` adds the game side's
//! coverage, the row accounting, the bases and the versions. `list`
//! groups items by container in listing order, lists ten or fewer per
//! group and counts more, and leaves `none` out unless asked; `--in` is
//! the items physically in a container, `--covered-by` the items a row
//! on a target would cover (C70), and a container not on record is a
//! refusal, never an empty selection; `--expand` lists every item with
//! its texts. `show` is one target: both sides with their causes and
//! revisions, the raw note beside the parse and the tab name beside its
//! reading (the slice's done criterion), and for a container both item
//! sets. The text renderers read [`ListView`] and [`ShowView`] — the
//! `--json` documents — and nothing else, so text is a function of JSON
//! by construction; `tests/price_json.rs` pins the documents from a
//! spawned binary. Escapes are not used; ages are text, epochs JSON.

use std::str::FromStr;

use acquisition_plan::game_side::{GamePrice, Source};
use acquisition_plan::listing::{
    ListFilter, ListView, Listing, ListingReport, Relation, ReportHeader, ShowView, Side, resolve,
};
use acquisition_plan::price::{
    BUYOUT_KIND, BUYOUT_VERSION, Buyout, PriceTarget, PriceWrite, clear_buyout, set_buyout,
};
use acquisition_protocol::realm::Realm;
use acquisition_store::{
    AnnotationRow, Annotations, FactAddress, IntentValue, Provenance, Store, account_path,
    check_value,
};
use anyhow::{Context, Result, bail};
use serde::Serialize;
use serde_json::json;

use crate::plan_cmd::{WRITTEN_VIA, open_intent};
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

/// A target address as one word, or the grammar to type.
fn parse_target(word: &str) -> Result<PriceTarget> {
    PriceTarget::from_str(word).map_err(|e| {
        anyhow::anyhow!(
            "{e}; a target is one word: item/<id>, character/<id>, tab/<realm>/<id> or substash/<realm>/<parent>/<id>"
        )
    })
}

/// The realm a command reads: a tab or substash address carries its own
/// and an explicit `--realm` must agree; an item or character address
/// takes `--realm`, default pc.
fn realm_of(targets: &[&PriceTarget], given: Option<Realm>) -> Result<Realm> {
    let mut realm = given;
    for t in targets {
        if let PriceTarget::Tab { realm: r, .. } | PriceTarget::Substash { realm: r, .. } = t {
            match realm {
                Some(have) if have != *r => {
                    bail!("{t} is a {r} address; {have} was given (--realm or another address)")
                }
                _ => realm = Some(*r),
            }
        }
    }
    Ok(realm.unwrap_or(Realm::DEFAULT))
}

/// The refusal for a target these facts do not hold.
fn not_on_record(r: &ListingReport, target: &PriceTarget) -> anyhow::Error {
    anyhow::anyhow!(
        "{target} is not in the facts for {}{} ({} items, {} tabs and characters on record); \
         `acq items search` finds an item's id, `acq tabs` a tab's",
        realm_prefix(r.header.realm),
        r.header.league,
        r.counts.items,
        r.counts.containers
    )
}

/// `acq price show <target>`.
pub fn show(
    target: &str,
    realm: Option<Realm>,
    league: &str,
    expand: bool,
    json: bool,
) -> Result<()> {
    let target = parse_target(target)?;
    let realm = realm_of(&[&target], realm)?;
    let r = report(realm, league)?;
    let view = r
        .show_view(&target)
        .ok_or_else(|| not_on_record(&r, &target))?;
    if json {
        println!("{}", serde_json::to_string_pretty(&view)?);
        return Ok(());
    }
    print!("{}", render_show(&view, acquisition_store::now(), expand));
    Ok(())
}

/// `list`'s selection flags as typed: relation and effective-side words,
/// container addresses — parsed here into a [`ListFilter`].
#[derive(Debug, Default)]
pub struct ListArgs {
    pub relation: Option<String>,
    pub effective: Option<String>,
    pub location: Option<String>,
    pub covered_by: Option<String>,
}

/// `acq price list`.
pub fn list(
    realm: Option<Realm>,
    league: &str,
    args: &ListArgs,
    expand: bool,
    json: bool,
) -> Result<()> {
    let (relation, effective, location, covered_by) = (
        args.relation.as_deref(),
        args.effective.as_deref(),
        args.location.as_deref(),
        args.covered_by.as_deref(),
    );
    const SIDES: [&str; 4] = ["game", "manual", "none", "unresolved"];
    let effective = match effective {
        None => None,
        Some(word) if SIDES.contains(&word) => Some(word.to_string()),
        Some(word) => bail!(
            "{word:?} is not an effective side (one of {})",
            SIDES.join(", ")
        ),
    };
    let relation = match relation {
        None => None,
        Some(word) => Some(Relation::parse(word).ok_or_else(|| {
            anyhow::anyhow!(
                "{word:?} is not a relation (one of {})",
                Relation::ALL.map(Relation::as_str).join(", ")
            )
        })?),
    };
    let container = |flag: &str, word: Option<&str>| -> Result<Option<PriceTarget>> {
        let Some(word) = word else {
            return Ok(None);
        };
        let t = parse_target(word)?;
        if matches!(t, PriceTarget::Item { .. }) {
            bail!("{flag} takes a container (tab, substash or character), not an item");
        }
        Ok(Some(t))
    };
    let location = container("--in", location)?;
    let covered_by = container("--covered-by", covered_by)?;
    let named: Vec<&PriceTarget> = [&location, &covered_by].into_iter().flatten().collect();
    let realm = realm_of(&named, realm)?;
    let r = report(realm, league)?;
    let filter = ListFilter {
        relation,
        effective,
        r#in: location.clone(),
        covered_by: covered_by.clone(),
    };
    let Some(view) = r.list_view(filter) else {
        let missing = [&location, &covered_by]
            .into_iter()
            .flatten()
            .find(|t| r.find(t).is_none())
            .cloned()
            .unwrap_or(PriceTarget::Item { id: "?".into() });
        return Err(not_on_record(&r, &missing));
    };
    if json {
        println!("{}", serde_json::to_string_pretty(&view)?);
        return Ok(());
    }
    print!("{}", render_list(&view, expand));
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
    r.counts.by_game_statement.get(kind).copied().unwrap_or(0)
}

/// The by-relation breakdown of a set of items, in a fixed order, zeros
/// left out; an unresolved item (a row that cannot be read could
/// decide, C81) is counted as such, never as unlisted.
fn breakdown(items: &[&Listing]) -> String {
    let unresolved = |l: &Listing| l.effective.kind == "unresolved";
    let mut parts: Vec<String> = Relation::ALL
        .into_iter()
        .filter_map(|rel| {
            let n = items
                .iter()
                .filter(|l| l.relation == rel && !(rel == Relation::None && unresolved(l)))
                .count();
            (n > 0).then(|| format!("{n} {}", relation_word(rel)))
        })
        .collect();
    let n = items.iter().filter(|l| unresolved(l)).count();
    if n > 0 {
        parts.push(format!("{n} unresolved"));
    }
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

/// The rows that change the next decision, one line, only when there
/// are any; the same line whether or not there are items (C35: orphaned
/// intent stays visible).
fn rows_line(r: &ListingReport) -> Option<String> {
    let rows = &r.rows;
    (!rows.unmatched.is_empty() || !rows.unreadable.is_empty() || rows.other_realm > 0).then(
        || {
            format!(
                "rows: {} of {} apply here; {} name nothing in these facts, {} unreadable, {} for other realms\n",
                rows.applied,
                rows.total,
                rows.unmatched.len(),
                rows.unreadable.len(),
                rows.other_realm
            )
        },
    )
}

fn rows_detail(r: &ListingReport) -> String {
    let mut out = String::new();
    let rows = &r.rows;
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
    out
}

fn render_status(r: &ListingReport, now: i64, expand: bool) -> String {
    let mut out = String::new();
    let h = &r.header;
    let place = format!("{}{}", realm_prefix(h.realm), h.league);
    let c = &r.counts;
    if c.items == 0 {
        if h.stash_listing.is_none() && h.character_listing.is_none() {
            out.push_str(&format!(
                "nothing on record for {place}: no stash listing, no character listing\n"
            ));
        } else {
            out.push_str(&format!(
                "no items on record for {place}: {} listed, none fetched with items\n",
                plural(c.containers, "tab or character", "tabs and characters")
            ));
        }
        if let Some(line) = rows_line(r) {
            out.push_str(&line);
        } else if r.rows.total > 0 {
            out.push_str(&format!(
                "rows: {} on record, none naming these facts yet\n",
                r.rows.total
            ));
        }
        if expand {
            out.push_str(&rows_detail(r));
        }
        let next = if !r.rows.unreadable.is_empty() {
            "`acq price status --expand` lists the rows that cannot be read"
        } else {
            "`acq refresh --plan` shows what a refresh would fetch"
        };
        out.push_str(&format!("next: {next}\n"));
        return out;
    }
    let game_priced = reading_count(r, "exact") + reading_count(r, "negotiable");
    let by_hand = count_of(r, Relation::ManualOnly)
        + count_of(r, Relation::Agree)
        + count_of(r, Relation::Conflict);
    let effective = |side: &str| c.by_effective.get(side).copied().unwrap_or(0);
    let unresolved = effective("unresolved");
    out.push_str(&format!(
        "{} in {place}: {} the game decides, {} decided by hand, {} nothing applies{}\n",
        plural(c.items, "item", "items"),
        effective("game"),
        effective("manual"),
        effective("none"),
        if unresolved > 0 {
            format!(", {unresolved} unresolved (a row that cannot be read could decide)")
        } else {
            String::new()
        }
    ));
    out.push_str(&format!(
        "sides: {game_priced} priced in game, {by_hand} by hand; {} agree, {} conflict\n",
        count_of(r, Relation::Agree),
        count_of(r, Relation::Conflict),
    ));
    if c.residue > 0 {
        out.push_str(&format!(
            "{} items carry price text the index cannot see (a note on a character, a name on a non-public tab): shown, not a side (C81)\n",
            c.residue
        ));
    }
    let skipped = reading_count(r, "skip");
    let invalid = c.invalid_notes;
    if skipped + invalid > 0 {
        out.push_str(&format!(
            "in game also: {skipped} skipped (~skip), {invalid} invalid notes (no effect; the tab applies, T18)\n"
        ));
    }
    if c.league_unknown > 0 {
        out.push_str(&format!(
            "{} items belong to a character the listing gave no league: they appear in every league's report and are not evidence for this one\n",
            c.league_unknown
        ));
    }
    let rows = &r.rows;
    if let Some(line) = rows_line(r) {
        out.push_str(&line);
    }
    if expand {
        out.push_str(&format!(
            "game side: {} priced tab names ({} public); {} priced items in public tabs; note parser v{}, currency table v{}\n",
            c.priced_tabs,
            c.priced_tabs_public,
            c.game_priced,
            h.note_parser_version,
            h.currency_table_version
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
            basis(h.stash_listing),
            basis(h.character_listing),
            ago(now, Some(h.taken_at))
        ));
        out.push_str(&rows_detail(r));
    }
    let next = if unresolved > 0 {
        "`acq price list --effective unresolved` names each item a row that cannot be read could decide"
    } else if count_of(r, Relation::Conflict) > 0 {
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

/// Who decides (C81): `game`, `hand`, `?` (a row that cannot be read
/// could), or `-`.
fn wins_cell(l: &Listing) -> &'static str {
    match (l.effective.side, l.effective.kind.as_str()) {
        (Some(Side::Game), _) => "game",
        (Some(Side::Manual), _) => "hand",
        (None, "unresolved") => "?",
        (None, _) => "-",
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
        "  {:<12} {:<5} {:<18} {:<18} {:<40} {}\n",
        l.relation.as_str(),
        wins_cell(l),
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
    out.push_str(&format!("      effective: {}\n", l.effective.why));
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
fn container_label(containers: &[Listing], c: &Listing) -> String {
    let mut name = if c.subject.name.is_empty() {
        "(unnamed)".to_string()
    } else {
        c.subject.name.clone()
    };
    if c.subject.tab_type.is_none() && !c.subject.league_unknown {
        return name;
    }
    // A substash under its parent: the parent's name is the one read.
    if let PriceTarget::Substash { realm, parent, .. } = &c.subject.target {
        let parent = PriceTarget::Tab {
            realm: *realm,
            id: parent.clone(),
        };
        let parent_name = containers
            .iter()
            .find(|p| p.subject.target == parent)
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
    if c.subject.league_unknown {
        notes.push("league unknown".into());
    }
    if notes.is_empty() {
        name
    } else {
        format!("{name} ({})", notes.join(", "))
    }
}

fn render_list(view: &ListView, expand: bool) -> String {
    let mut out = String::new();
    let h = &view.header;
    let items: Vec<&Listing> = view.items.iter().collect();
    let place = format!("{}{}", realm_prefix(h.realm), h.league);
    let mut scope = String::new();
    if let Some(rel) = view.filter.relation {
        scope.push_str(&format!(" with relation {rel}"));
    }
    if let Some(side) = &view.filter.effective {
        scope.push_str(&format!(" decided by {side}"));
    }
    if let Some(loc) = &view.filter.r#in {
        scope.push_str(&format!(" in {loc}"));
    }
    if let Some(t) = &view.filter.covered_by {
        scope.push_str(&format!(" covered by {t}"));
    }
    if items.is_empty() {
        let nothing = if view.items_on_record == 0 {
            "no items on record".to_string()
        } else if view.filter.relation.is_none() {
            format!(
                "{} items on record, none listed by hand or in game",
                view.items_on_record
            )
        } else {
            format!("{} items on record, none match", view.items_on_record)
        };
        out.push_str(&format!("{nothing} for {place}{scope}\n"));
        out.push_str("next: `acq price status` says what is on record\n");
        return out;
    }
    // Groups by container, in the containers' order (tabs in listing
    // order, then characters); a location not on record comes last.
    let mut groups: Vec<(&PriceTarget, Vec<&Listing>)> = view
        .containers
        .iter()
        .map(|c| (&c.subject.target, Vec::new()))
        .collect();
    for l in &items {
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
        breakdown(&items),
        plural(groups.len(), "container", "containers")
    ));
    for (loc, group) in &groups {
        let label = view
            .containers
            .iter()
            .find(|c| &c.subject.target == *loc)
            .map(|c| container_label(&view.containers, c))
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

fn render_show(view: &ShowView, now: i64, expand: bool) -> String {
    let mut out = String::new();
    let h: &ReportHeader = &view.header;
    let l = &view.listing;
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
        let name = view
            .container
            .as_ref()
            .map(|c| c.subject.label())
            .unwrap_or_default();
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
    if s.league_unknown {
        out.push_str(
            "league unknown: the listing gave this character no league, so it appears in every league's report and is not evidence for this one\n",
        );
    }
    out.push_str(&format!("relation {}: {}\n", l.relation, l.why));
    out.push_str(&format!(
        "effective: {} — {}\n",
        l.effective, l.effective.why
    ));

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
    let residue = if g.residue {
        " (price text the index cannot see: shown, not a side, C81)"
    } else {
        ""
    };
    out.push_str(&format!("in game: {}{from}{residue}\n", g.reading));
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
            h.note_parser_version,
            h.currency_table_version,
            ago(now, Some(h.taken_at))
        ));
        if s.is_item() {
            out.push_str(&format!(
                "chain: {}\n",
                l.chain
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(" > ")
            ));
        }
    }

    if !s.is_item() {
        let here: Vec<&Listing> = view.items_here.iter().collect();
        let below: Vec<&Listing> = view.items_covered_below.iter().collect();
        // Two sets (C70): physically here, and covered through children.
        if here.is_empty() {
            out.push_str("items here: none on record\n");
        } else {
            out.push_str(&format!(
                "items here: {} — {}\n",
                plural(here.len(), "item", "items"),
                breakdown(&here)
            ));
        }
        if !below.is_empty() {
            out.push_str(&format!(
                "items covered through children (C70): {} — {}\n",
                plural(below.len(), "item", "items"),
                breakdown(&below)
            ));
        }
        // Each set listed under its own line (ten or fewer, or --expand);
        // a set past the threshold defers to `list`.
        let mut deferred = false;
        for (set, is_below) in [(&here, false), (&below, true)] {
            if set.is_empty() {
                continue;
            }
            if set.len() <= LIST_UP_TO || expand {
                if is_below && !here.is_empty() {
                    out.push_str("covered through children:\n");
                }
                for l in set {
                    out.push_str(&item_line(l));
                    if expand {
                        out.push_str(&item_texts(l));
                    }
                }
            } else {
                deferred = true;
            }
        }
        if deferred {
            out.push_str(&format!(
                "next: `acq price list --in {}` lists them, `--covered-by` the covered set; `--expand` here lists every one\n",
                s.target
            ));
            return out;
        }
    }
    let next = match &s.location {
        Some(loc) => format!("`acq price list --in {loc}` is its neighbours"),
        None => "`acq price show item/<id>` is one item".into(),
    };
    out.push_str(&format!("next: {next}\n"));
    out
}

/// The value words `set` takes: the C67 type word — or the game's own
/// (`price`, `b/o`, `~price`, `~b/o`, `~skip`) — then, for a priced
/// type, the amount and the currency tag. Built into the v1 wire shape
/// and read through the value's own strict parse, so the CLI has no
/// second grammar: a fifth digit, an alias that is not a tag, an amount
/// on `skip`, each refuses in the value's words.
fn parse_buyout(kind: &str, amount: Option<&str>, currency: Option<&str>) -> Result<Buyout> {
    let kind = match kind {
        "exact" | "price" | "~price" => "exact",
        "negotiable" | "b/o" | "~b/o" => "negotiable",
        "no_price" | "no-price" => "no_price",
        "skip" | "~skip" => "skip",
        other => bail!(
            "{other:?} is not a price type: exact (the game's ~price), negotiable (~b/o), no_price or skip"
        ),
    };
    if matches!(kind, "exact" | "negotiable") && (amount.is_none() || currency.is_none()) {
        bail!(
            "{kind} takes an amount and a currency tag: `{kind} 12.5 chaos`, `{kind} 1/5 divine`; \
             `acq reference currency` lists the tags"
        );
    }
    let mut value = json!({ "version": BUYOUT_VERSION, "type": kind });
    if let Some(amount) = amount {
        value["amount"] = json!(amount);
    }
    if let Some(currency) = currency {
        value["currency"] = json!(currency);
    }
    Buyout::parse(&value).map_err(|e| anyhow::anyhow!("{e}"))
}

/// The blind default's read: the row's current revision, or none. A row
/// this build cannot read (a newer build's value, which the listing state
/// reports as unreadable) is refused here rather than replaced — a blind
/// write over it would destroy intent this build cannot restore, and the
/// receipt's undo would be a value it cannot write. An explicit
/// `--if-revision` is the reviewed path past it.
fn current_revision(annotations: &Annotations, target: &PriceTarget) -> Result<Option<i64>> {
    let (scope, key) = target.address()?;
    let Some(row) = annotations.get(scope, &key, BUYOUT_KIND)? else {
        return Ok(None);
    };
    if let Err(e) = check_value::<Buyout>(&row.value) {
        bail!(
            "{target} holds a value this build cannot read (revision {}: {e}); \
             not replaced blind — `--if-revision {}` replaces it deliberately, \
             `acq price show {target}` names it",
            row.revision,
            row.revision
        );
    }
    Ok(Some(row.revision))
}

/// `acq price set <target> <type> [<amount> <currency>]`: one row through
/// the shared write ([`set_buyout`]). `--if-revision` is the CAS at the
/// human boundary, the same rule as `acq policy set`: given, the write
/// lands only over exactly that revision; omitted, it replaces whatever
/// is stored — read just before the put, so a write racing in between is
/// still a structured conflict, never a clobber.
pub fn set(
    target: &str,
    kind: &str,
    amount: Option<&str>,
    currency: Option<&str>,
    if_revision: Option<i64>,
    json: bool,
) -> Result<()> {
    let target = parse_target(target)?;
    let value = parse_buyout(kind, amount, currency)?;
    let (dir, entry, mut annotations) = open_intent()?;
    let expected = match if_revision {
        Some(revision) => Some(revision),
        None => current_revision(&annotations, &target)?,
    };
    let write = set_buyout(
        &mut annotations,
        &target,
        &value,
        expected,
        &Provenance::via(WRITTEN_VIA),
    )?;
    // Read after the write, so the note never gates it (C64): whether the
    // facts hold the target at all, in any league.
    let in_facts = Store::open(&account_path(&dir, &entry.username))?
        .holds(&FactAddress::from(&target))
        .context("reading the facts for the target")?;
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&SetReceipt {
                write: &write,
                in_facts
            })?
        );
        return Ok(());
    }
    print!("{}", render_write(&write, Some(in_facts)));
    Ok(())
}

/// `set --json`: the [`PriceWrite`] plus whether the facts hold the
/// target — the text's note, as JSON (C53). Absent from `clear`, which
/// only ever removes a row that exists.
#[derive(Serialize)]
struct SetReceipt<'a> {
    #[serde(flatten)]
    write: &'a PriceWrite,
    in_facts: bool,
}

/// `acq price clear <target>`: remove the row, under the same CAS. A
/// target with no row is a refusal that says so — every nothing says
/// which nothing (C53).
pub fn clear(target: &str, if_revision: Option<i64>, json: bool) -> Result<()> {
    let target = parse_target(target)?;
    let (_, _, mut annotations) = open_intent()?;
    let expected = match if_revision {
        Some(revision) => revision,
        None => match current_revision(&annotations, &target)? {
            Some(revision) => revision,
            None => bail!(
                "nothing to clear: {target} has no price row of its own \
                 (`acq price show {target}` reads what covers it)"
            ),
        },
    };
    let write = clear_buyout(
        &mut annotations,
        &target,
        expected,
        &Provenance::via(WRITTEN_VIA),
    )?;
    report_write(&write, json)
}

fn report_write(write: &PriceWrite, json: bool) -> Result<()> {
    if json {
        println!("{}", serde_json::to_string_pretty(write)?);
        return Ok(());
    }
    print!("{}", render_write(write, None));
    Ok(())
}

/// A stored value as text, or its JSON when this build cannot read it.
fn value_text(row: &AnnotationRow) -> String {
    match check_value::<Buyout>(&row.value) {
        Ok(value) => value.to_string(),
        Err(_) => format!("an unreadable value {}", row.value),
    }
}

/// The `set` words that write this value again — the undo, typed.
fn set_words(value: &Buyout) -> String {
    match value.price() {
        Some(price) => format!("{} {price}", value.kind()),
        None => value.kind().into(),
    }
}

/// The write receipt as text (C53): what the target is now and what it
/// was (C78), then the next action — reading it beside the game side,
/// and the command that puts the prior back. After a replacement that
/// command carries `--if-revision` naming the revision just written, so
/// it undoes this write and not a later one; after a clear it is a
/// create, which the store lands over the tombstone whatever came
/// between (the create cannot say which clear it follows). A prior this
/// build cannot read is shown as its JSON and promised no undo.
fn render_write(write: &PriceWrite, in_facts: Option<bool>) -> String {
    let target = &write.target;
    let was = match &write.prior {
        None => "was unset".to_string(),
        Some(prior) => format!("was {} (revision {})", value_text(prior), prior.revision),
    };
    let prior_words = write
        .prior
        .as_ref()
        .and_then(|prior| check_value::<Buyout>(&prior.value).ok())
        .map(|value| set_words(&value));
    let mut out = String::new();
    match &write.written {
        Some(row) => {
            out.push_str(&format!(
                "{target}: {} (revision {}), {was}\n",
                value_text(row),
                row.revision
            ));
            if in_facts == Some(false) {
                out.push_str(&format!(
                    "note: the facts hold no {target} (not fetched yet, or a typo); the row stands, \
                     and `acq price status` counts it under \"name nothing in these facts\"\n"
                ));
            }
            let mut next = format!("`acq price show {target}` reads it beside the game side");
            match (&write.prior, prior_words) {
                (Some(_), Some(words)) => next.push_str(&format!(
                    "; `acq price set {target} {words} --if-revision {}` puts the prior back",
                    row.revision
                )),
                (Some(_), None) => next.push_str(
                    "; the prior cannot be put back by this build (its JSON is in --json)",
                ),
                (None, _) => {}
            }
            out.push_str(&format!("next: {next}\n"));
        }
        None => {
            out.push_str(&format!("{target}: cleared, {was}\n"));
            let next = match prior_words {
                Some(words) => format!("`acq price set {target} {words}` puts it back"),
                None => {
                    "the cleared value cannot be put back by this build (its JSON is in --json)"
                        .to_string()
                }
            };
            out.push_str(&format!("next: {next}\n"));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::real_scale_fixture;
    use acquisition_store::{
        AnnotationRow, CharacterSnapshot, ItemSnapshot, ListingBasis, PricingSnapshot, TabSnapshot,
    };
    use serde_json::{Value, json};
    use std::time::{Duration, Instant};

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
            inventory_id: Some(
                if kind == "stash" {
                    "Stash1"
                } else {
                    "BodyArmour"
                }
                .into(),
            ),
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

    /// A priced public tab with twelve items (one noted, one skipped by
    /// hand), a non-public priced map tab holding one item directly and
    /// one in its substash, a character wearing a noted item, and one row
    /// naming nothing here.
    fn snapshot() -> PricingSnapshot {
        let mut items = vec![item("i-worn", "character", "ch1", Some("~b/o 2 divine"))];
        for i in 0..12 {
            items.push(item(&format!("i-{i:02}"), "stash", "c1", None));
        }
        items[2].note = Some("~price 5 chaos".into()); // i-01, the row's item
        items.push(item("i-mapdirect", "stash", "m1", None));
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
                row("item", "i-01", json!({ "version": 1, "type": "skip" }), 4),
                row("item", "i-gone", json!({ "version": 1, "type": "skip" }), 5),
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
            "15 items in Standard: 12 the game decides, 0 decided by hand, 3 nothing applies"
        );
        assert_eq!(
            lines[1],
            "sides: 12 priced in game, 1 by hand; 0 agree, 1 conflict"
        );
        assert_eq!(
            lines[2],
            "3 items carry price text the index cannot see (a note on a character, a name on a non-public tab): shown, not a side (C81)"
        );
        assert_eq!(
            lines[3],
            "rows: 1 of 2 apply here; 1 name nothing in these facts, 0 unreadable, 0 for other realms"
        );
        assert_eq!(
            lines[4],
            "next: `acq price list --relation conflict` names each conflict"
        );
        assert_eq!(lines.len(), 5, "{text}");
        assert!(!text.contains("parser"), "{text}");

        let text = render_status(&r, 8_000, true);
        assert!(
            text.contains("game side: 2 priced tab names (1 public); 12 priced items in public tabs; note parser v2, currency table v1"),
            "{text}"
        );
        assert!(
            text.contains("by hand: 1 of 2 rows apply here; 0 items inherit"),
            "{text}"
        );
        assert!(
            text.contains("basis: stash listing response 2 1h ago, character listing response 3 1h ago; snapshot 1h ago"),
            "{text}"
        );
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
        let view = r.list_view(ListFilter::default()).unwrap();
        let text = render_list(&view, false);
        let lines: Vec<&str> = text.lines().collect();
        // The map-tab item and the character's are relation `none` now:
        // their price text is residue (C81), so the default leaves them out.
        assert_eq!(
            lines[0],
            "12 items listed in Standard: 11 in game only, 1 conflict; in 1 container"
        );
        assert_eq!(
            lines[1],
            "~price 3 chaos (A) (3 chaos, public)  12 items: 11 in game only, 1 conflict  tab/pc/c1"
        );
        // Twelve is more than ten: counted, not listed.
        assert_eq!(
            lines[2],
            "next: `--expand` lists every item; `acq price show item/<id>` is one"
        );
        assert_eq!(lines.len(), 3, "{text}");

        let text = render_list(&view, true);
        assert!(
            text.contains(
                "  conflict     game  skip               5 chaos            Chaos Orb x10"
            ),
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
        assert!(
            text.contains("      effective: 5 chaos in game: the game wins a tie with the row on the item (C81)\n"),
            "{text}"
        );
        assert_eq!(text.matches("      tab c1 ").count(), 12, "{text}");

        // The residue items, under `--relation none`: a substash labelled
        // under its parent's name, the character last.
        let view = r
            .list_view(ListFilter {
                relation: Some(Relation::None),
                ..ListFilter::default()
            })
            .unwrap();
        let text = render_list(&view, false);
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(
            lines[0],
            "3 items listed in Standard with relation none: 3 unlisted; in 3 containers"
        );
        assert_eq!(
            lines[1],
            "~price 1 divine (Remove-only) (not public)  1 item: 1 unlisted  tab/pc/m1"
        );
        assert!(lines[2].ends_with("i-mapdirect"), "{}", lines[2]);
        assert_eq!(
            lines[3],
            "~price 1 divine (Remove-only) / 1 (Remove-only) (not public)  1 item: 1 unlisted  substash/pc/m1/s1"
        );
        assert!(
            lines[4].starts_with(
                "  none         -     -                  -                  Chaos Orb x10"
            ),
            "{}",
            lines[4]
        );
        assert!(lines[4].ends_with("i-sub"), "{}", lines[4]);
        assert_eq!(lines[5], "Exile  1 item: 1 unlisted  character/ch1");
        assert!(lines[6].ends_with("i-worn"), "{}", lines[6]);

        let view = r
            .list_view(ListFilter {
                relation: Some(Relation::Agree),
                ..ListFilter::default()
            })
            .unwrap();
        let text = render_list(&view, false);
        assert_eq!(
            text,
            "15 items on record, none match for Standard with relation agree\nnext: `acq price status` says what is on record\n"
        );
    }

    /// The done criterion: `show` puts the raw note beside its parse and
    /// the tab name beside its reading, names the row and its revision,
    /// and summarizes a container's items under it.
    #[test]
    fn the_raw_note_sits_beside_the_parse_in_show() {
        let r = report();
        let show = |target: PriceTarget| r.show_view(&target).unwrap();
        let l = show(PriceTarget::Item { id: "i-01".into() });
        let text = render_show(&l, 8_000, false);
        assert_eq!(
            text,
            "item/i-01: Chaos Orb x10  in \"~price 3 chaos (A)\" tab/pc/c1\n\
             relation conflict: by hand: skip; in game: 5 chaos\n\
             effective: 5 chaos — 5 chaos in game: the game wins a tie with the row on the item (C81)\n\
             by hand: skip (own row, revision 4, set 1h ago)\n\
             in game: 5 chaos from the note\n\
             \x20 note \"~price 5 chaos\" reads 5 chaos\n\
             \x20 tab c1 \"~price 3 chaos (A)\" reads 3 chaos, public\n\
             next: `acq price list --in tab/pc/c1` is its neighbours\n"
        );
        let text = render_show(&l, 8_000, true);
        assert!(text.contains("set 1h ago via cli as tom)"), "{text}");
        assert!(text.contains("chain: item/i-01 > tab/pc/c1\n"), "{text}");
        assert!(
            text.contains(
                "basis: response 7 1h ago; note parser v2, currency table v1; snapshot 1h ago\n"
            ),
            "{text}"
        );

        // A substash item: its own name beside the parent's reading.
        let l = show(PriceTarget::Item { id: "i-sub".into() });
        let text = render_show(&l, 8_000, false);
        assert!(
            text.contains("in game: none (price text the index cannot see: shown, not a side, C81)\n  tab m1 \"~price 1 divine (Remove-only)\" reads 1 divine, not public\n  substash \"1 (Remove-only)\" (its parent's name and public are read, C80)\n"),
            "{text}"
        );
        assert!(
            text.contains("effective: none — nothing applies: the price text in game is not where the index can see it (C81)\n"),
            "{text}"
        );
        assert!(
            text.contains("by hand: nothing (no row on it or above it)\n"),
            "{text}"
        );

        // A character item: no stash to publish.
        let l = show(PriceTarget::Item {
            id: "i-worn".into(),
        });
        let text = render_show(&l, 8_000, false);
        assert!(
            text.contains("  no tab name read: no stash to publish\n"),
            "{text}"
        );

        // A container: the items under it, counted past ten.
        let l = show(PriceTarget::Tab {
            realm: Realm::Pc,
            id: "c1".into(),
        });
        let text = render_show(&l, 8_000, false);
        assert!(
            text.starts_with("tab/pc/c1: ~price 3 chaos (A) (PremiumStash)\nrelation game_only: in game: 3 chaos; no row applies\neffective: 3 chaos — 3 chaos in game; no row applies\n"),
            "{text}"
        );
        assert!(
            text.contains("items here: 12 items — 11 in game only, 1 conflict\n"),
            "{text}"
        );
        assert!(!text.contains("covered through children"), "{text}");
        assert!(
            text.ends_with("next: `acq price list --in tab/pc/c1` lists them, `--covered-by` the covered set; `--expand` here lists every one\n"),
            "{text}"
        );
        let text = render_show(&l, 8_000, true);
        assert_eq!(text.matches("\n  game_only ").count(), 11, "{text}");
        // A mixed parent tab: its own item here and its substash's covered
        // (C70), each set listed under its own line.
        let l = show(PriceTarget::Tab {
            realm: Realm::Pc,
            id: "m1".into(),
        });
        let text = render_show(&l, 8_000, false);
        assert!(
            text.contains("items here: 1 item — 1 unlisted\nitems covered through children (C70): 1 item — 1 unlisted\n"),
            "{text}"
        );
        let direct = text.find("i-mapdirect\n").unwrap();
        let covered_head = text.find("covered through children:\n").unwrap();
        let sub = text.find("i-sub\n").unwrap();
        assert!(direct < covered_head && covered_head < sub, "{text}");
    }

    /// A tab or substash address carries its realm; `--realm` must agree;
    /// an item or character takes `--realm`, default pc.
    #[test]
    fn the_realm_comes_from_the_address_and_a_disagreement_refuses() {
        let xbox = PriceTarget::Tab {
            realm: Realm::Xbox,
            id: "t".into(),
        };
        let item = PriceTarget::Item { id: "i".into() };
        assert_eq!(realm_of(&[&xbox], None).unwrap(), Realm::Xbox);
        assert_eq!(realm_of(&[&xbox], Some(Realm::Xbox)).unwrap(), Realm::Xbox);
        assert_eq!(realm_of(&[&item], None).unwrap(), Realm::Pc);
        assert_eq!(realm_of(&[&item], Some(Realm::Sony)).unwrap(), Realm::Sony);
        assert_eq!(realm_of(&[&item, &xbox], None).unwrap(), Realm::Xbox);
        let err = realm_of(&[&xbox], Some(Realm::Pc)).unwrap_err().to_string();
        assert!(
            err.contains("tab/xbox/t is a xbox address; pc was given"),
            "{err}"
        );
        let pc = PriceTarget::Tab {
            realm: Realm::Pc,
            id: "u".into(),
        };
        assert!(realm_of(&[&xbox, &pc], None).is_err());
    }

    /// C35 — intent stays visible with no facts: the empty-league status
    /// still reports the rows and picks its next action from them.
    #[test]
    fn c35_an_empty_league_still_reports_its_rows() {
        let mut empty = snapshot();
        empty.items.clear();
        empty.buyouts.push(AnnotationRow {
            scope: "item".into(),
            key: "i-new".into(),
            kind: "buyout".into(),
            value: json!({ "version": 9, "type": "exact" }),
            revision: 7,
            created_at: 900,
            updated_at: 950,
            written_via: "cli".into(),
            actor: None,
        });
        let r = resolve(&empty).unwrap();
        let text = render_status(&r, 8_000, false);
        assert_eq!(
            text,
            "no items on record for Standard: 4 tabs and characters listed, none fetched with items\n\
             rows: 0 of 3 apply here; 2 name nothing in these facts, 1 unreadable, 0 for other realms\n\
             next: `acq price status --expand` lists the rows that cannot be read\n"
        );
        let text = render_status(&r, 8_000, true);
        assert!(
            text.contains("rows that cannot be read:\n  item/i-new revision 7:"),
            "{text}"
        );
        assert!(
            text.contains("rows naming nothing in these facts:\n  item/i-01\n  item/i-gone\n"),
            "{text}"
        );
    }

    /// `set`'s value words are one grammar with the value's own: the
    /// C67 type words and the game's, an amount tolerated as a human
    /// types it and stored canonical, and every refusal in the value's
    /// words — a priced type without its two words, an amount on `skip`,
    /// an alias that is not a tag.
    #[test]
    fn set_reads_the_value_words_through_the_value_s_own_parse() {
        let exact = |amount: &str, currency: &str| {
            Buyout::Exact(acquisition_plan::price::Price {
                amount: amount.parse().unwrap(),
                currency: currency.into(),
            })
        };
        assert_eq!(
            parse_buyout("exact", Some("12.50"), Some("chaos")).unwrap(),
            exact("12.5", "chaos")
        );
        assert_eq!(
            parse_buyout("~price", Some("1"), Some("divine")).unwrap(),
            exact("1", "divine")
        );
        assert_eq!(
            parse_buyout("b/o", Some("1/5"), Some("divine"))
                .unwrap()
                .to_string(),
            "1/5 divine b/o"
        );
        assert_eq!(parse_buyout("skip", None, None).unwrap(), Buyout::Skip);
        assert_eq!(parse_buyout("~skip", None, None).unwrap(), Buyout::Skip);
        assert_eq!(
            parse_buyout("no-price", None, None).unwrap(),
            Buyout::NoPrice
        );
        for (words, said) in [
            (("bogus", None, None), "is not a price type"),
            (
                ("exact", Some("1"), None),
                "takes an amount and a currency tag",
            ),
            (
                ("negotiable", None, None),
                "takes an amount and a currency tag",
            ),
            (("skip", Some("1"), Some("chaos")), "carries no amount"),
            (
                ("exact", Some("1"), Some("exa")),
                "resolves to tag \"exalted\"",
            ),
            (
                ("exact", Some("1"), Some("c")),
                "is not in currency table v1",
            ),
            (
                ("exact", Some("1.23456"), Some("chaos")),
                "four fractional digits",
            ),
            (("exact", Some("0"), Some("chaos")), "positive"),
        ] {
            let err = parse_buyout(words.0, words.1, words.2).unwrap_err();
            assert!(err.to_string().contains(said), "{words:?}: {err}");
        }
    }

    /// C53, C78 — the receipt's text says what the target is now and
    /// what it was, and ends with the action that undoes it: the prior
    /// value as `set` words, with `--if-revision` naming the revision
    /// just written after a replacement (so it undoes this write and not
    /// a later one), a create after a clear; a prior this build cannot
    /// read is shown as its JSON and promised no undo.
    #[test]
    fn the_write_receipt_says_what_was_and_how_to_put_it_back() {
        let target = PriceTarget::Item { id: "i1".into() };
        let row = |value: Value, revision: i64| AnnotationRow {
            scope: "item".into(),
            key: "i1".into(),
            kind: "buyout".into(),
            value,
            revision,
            created_at: 0,
            updated_at: 0,
            written_via: "cli".into(),
            actor: None,
        };
        let exact = json!({ "version": 1, "type": "exact", "amount": "12.5", "currency": "chaos" });
        let bo =
            json!({ "version": 1, "type": "negotiable", "amount": "1/5", "currency": "divine" });
        let created = PriceWrite {
            target: target.clone(),
            written: Some(row(exact.clone(), 1)),
            prior: None,
        };
        assert_eq!(
            render_write(&created, None),
            "item/i1: 12.5 chaos (revision 1), was unset\n\
             next: `acq price show item/i1` reads it beside the game side\n"
        );
        // A target the facts do not hold is noted between the row and the
        // next step, never refused (C64); one they do says nothing.
        assert_eq!(
            render_write(&created, Some(true)),
            render_write(&created, None)
        );
        assert_eq!(
            render_write(&created, Some(false)),
            "item/i1: 12.5 chaos (revision 1), was unset\n\
             note: the facts hold no item/i1 (not fetched yet, or a typo); the row stands, \
             and `acq price status` counts it under \"name nothing in these facts\"\n\
             next: `acq price show item/i1` reads it beside the game side\n"
        );
        let replaced = PriceWrite {
            target: target.clone(),
            written: Some(row(bo.clone(), 2)),
            prior: Some(row(exact.clone(), 1)),
        };
        assert_eq!(
            render_write(&replaced, None),
            "item/i1: 1/5 divine b/o (revision 2), was 12.5 chaos (revision 1)\n\
             next: `acq price show item/i1` reads it beside the game side; \
             `acq price set item/i1 exact 12.5 chaos --if-revision 2` puts the prior back\n"
        );
        let cleared = PriceWrite {
            target: target.clone(),
            written: None,
            prior: Some(row(bo, 2)),
        };
        assert_eq!(
            render_write(&cleared, None),
            "item/i1: cleared, was 1/5 divine b/o (revision 2)\n\
             next: `acq price set item/i1 negotiable 1/5 divine` puts it back\n"
        );
        let newer = json!({ "version": 9, "type": "auction" });
        let unreadable = PriceWrite {
            target,
            written: None,
            prior: Some(row(newer, 3)),
        };
        let text = render_write(&unreadable, None);
        assert!(
            text.starts_with(
                "item/i1: cleared, was an unreadable value {\"type\":\"auction\",\"version\":9} (revision 3)\n"
            ),
            "{text}"
        );
        assert!(
            text.ends_with(
                "next: the cleared value cannot be put back by this build (its JSON is in --json)\n"
            ),
            "{text}"
        );
        let overwritten = PriceWrite {
            target: PriceTarget::Item { id: "i1".into() },
            written: Some(row(exact, 4)),
            prior: Some(row(json!({ "version": 9, "type": "auction" }), 3)),
        };
        let text = render_write(&overwritten, None);
        assert!(
            text.ends_with(
                "next: `acq price show item/i1` reads it beside the game side; \
                 the prior cannot be put back by this build (its JSON is in --json)\n"
            ),
            "{text}"
        );
    }

    /// The listing state's audit views over the real-scale fixture tiled
    /// to 32k items (plan step 7, item 2, `PRICING-SLICE.md`): `list
    /// --effective none --expand` — every item nothing applies to, each
    /// with its texts — `show --expand` on the owner's test tab, and
    /// `status --expand`. Measured 2026-09-07 in a debug build: 300 ms,
    /// 7 ms and under a millisecond. The bound is a cliff, not a budget —
    /// and each view still says everything it has.
    #[test]
    fn c53_the_expanded_list_show_and_status_are_linear_over_the_owners_league_tiled() {
        let s = real_scale_fixture::tiled(16);
        let r = resolve(&s).unwrap();
        let tab = real_scale_fixture::owners_test_tab(&s);
        let now = s.taken_at;

        let t = Instant::now();
        let view = r
            .list_view(ListFilter {
                effective: Some("none".into()),
                ..Default::default()
            })
            .unwrap();
        let list = render_list(&view, true);
        let show = r.show_view(&tab).unwrap();
        let shown = render_show(&show, now, true);
        let status = render_status(&r, now, true);
        let took = t.elapsed();
        assert!(
            took < Duration::from_secs(5),
            "the expanded views took {took:?} over {} items",
            r.counts.items
        );

        assert_eq!(r.counts.items, 1977 * 16);
        assert_eq!(view.items.len(), 1889 * 16);
        assert!(
            list.starts_with(&format!(
                "{} items listed in Standard decided by none: ",
                view.items.len()
            )),
            "{}",
            list.lines().next().unwrap()
        );
        assert_eq!(
            list.lines().filter(|l| l.starts_with("  none ")).count(),
            view.items.len()
        );
        assert_eq!(show.items_here.len(), 80 * 16);
        // A line per item and its texts under it: the 50 notes' lines
        // per copy, and the effective line under every item.
        assert_eq!(
            shown
                .lines()
                .filter(|l| l.starts_with("      effective: "))
                .count(),
            show.items_here.len()
        );
        assert_eq!(
            shown
                .lines()
                .filter(|l| l.starts_with("      note "))
                .count(),
            50 * 16
        );
        assert!(
            status.starts_with(&format!("{} items in Standard: ", r.counts.items)),
            "{}",
            status.lines().next().unwrap()
        );
    }
}
