//! `acq search` and `acq show`: the clap adapter of `acquisition-search`
//! and its rendering. No daemon, no network: the store is read directly,
//! as `acq tabs` reads it. Rulings: `decisions/search.md`; the surface:
//! `search/DESIGN.md`, "The request at a terminal".
//!
//! # As built
//!
//! - **Text is a function of JSON (C53).** `--json` prints the crate's
//!   [`Answer`], [`Describe`] or [`Shown`] whole; the text below is
//!   rendered from that same value and from nothing else. Ages are text,
//!   epochs JSON.
//! - **A failure is structured on both paths (C11)**: `--json` prints
//!   `{"error", "kind", "readings"}` and exits 1; the text prints the
//!   message and each reading on a line of its own, ready to paste.
//! - **An answer never prints a command this build refuses** (the build
//!   plan, rule 5): a block the reference bounds with a route to the whole
//!   prints the count left out and the unbuilt construct's name. The flags
//!   of later steps exist, so that typing one is refused by that name
//!   (`acquisition_search::not_built`) and never as an unknown flag.
//! - **The query may come from a file or stdin** (`--query-file`, the
//!   plan's gap 2): a template with an apostrophe cannot be typed plainly
//!   inside a shell's single quotes. A route is printed shell-quoted.

use std::io::Read as _;

use acquisition_search::answer::{Answer, Count, Request, Rows, Scope, View};
use acquisition_search::corpus::Realm;
use acquisition_search::describe::Describe;
use acquisition_search::show::Shown;
use acquisition_search::{Corpus, SearchError, answer, describe, not_built, show};
use anyhow::{Context as _, Result};
use serde::Serialize;

use crate::store_cmd::{self, ago};

/// What `acq search` was typed with.
#[derive(clap::Args)]
pub struct SearchArgs {
    /// The query (`acq search --describe` prints the language). None:
    /// every item in scope. One that starts with `-`, the language's not,
    /// goes after `--`, as every route prints it: `acq search --realm pc
    /// -- '-is:corrupted'`.
    pub query: Option<String>,
    /// The realm searched: pc, xbox, sony, poe2, or all. Over a store
    /// holding one realm it may be omitted and the answer prints it; over
    /// several, a search names one (C96).
    #[arg(long)]
    pub realm: Option<String>,
    /// Read the query from a file, or from stdin with `-`: a query with an
    /// apostrophe needs no shell quoting this way.
    #[arg(long, value_name = "FILE|-", conflicts_with = "query")]
    pub query_file: Option<String>,
    /// Order the rows by a value: `ilvl`, `stack`, `'line("T").arg1'`,
    /// `'sum("T")'`. A line's scalar is its largest satisfying occurrence;
    /// an item with none sorts last either way (C92).
    #[arg(long, value_name = "VALUE")]
    pub sort: Option<String>,
    /// Largest first.
    #[arg(long, requires = "sort")]
    pub desc: bool,
    /// How many rows to return; the rest are counted.
    #[arg(long, default_value_t = acquisition_search::answer::DEFAULT_LIMIT)]
    pub limit: usize,
    /// Print every count's route — the command that returns exactly its
    /// members — instead of the first few.
    #[arg(long)]
    pub routes: bool,
    /// The language as this build knows it: fields, what a line has,
    /// operators, closed value sets, slots, what is not built and the
    /// limits stated; or only the entries named (`--describe league,line`).
    #[arg(long, value_name = "NAME,…", num_args = 0..=1, value_delimiter = ',')]
    pub describe: Option<Vec<String>>,
    /// Not built (step 5): counts by key, one table each.
    #[arg(long, value_name = "KEY,…")]
    pub count: Option<String>,
    /// Not built (step 5): one crossed table.
    #[arg(long, value_name = "KEY,KEY")]
    pub cross: Option<String>,
    /// Not built (step 5): one summed value beside a count.
    #[arg(long, value_name = "VALUE")]
    pub sum: Option<String>,
    /// Not built (step 10): the caller names a row's fields.
    #[arg(long, value_name = "NAME,…")]
    pub fields: Option<String>,
    /// Not built (step 10): continue an answer past its limit.
    #[arg(long, value_name = "TOKEN")]
    pub next: Option<String>,
    /// Not built (step 10): one node forced true and forced false.
    #[arg(long, value_name = "PATH")]
    pub explain: Option<String>,
    /// Not built (step 10): matches | corpus.
    #[arg(long, value_name = "WHICH")]
    pub context: Option<String>,
    /// Not built (step 10): `locations`, the full coverage list.
    #[arg(long, value_name = "WHAT")]
    pub view: Option<String>,
    /// Not built (step 10): print the request as JSON without running it.
    #[arg(long)]
    pub print_request: bool,
    /// Not built (step 10): run a request read from a file.
    #[arg(long, value_name = "FILE|-")]
    pub request: Option<String>,
    /// Not built (step 10): run another account's request deliberately.
    #[arg(long)]
    pub rebind: bool,
}

/// What `acq show` was typed with.
#[derive(clap::Args)]
pub struct ShowArgs {
    /// An item's id, whole, as an answer printed it.
    pub id: String,
    /// Also print the body as the store holds it, verbatim.
    #[arg(long)]
    pub body: bool,
    /// Not built (step 10): why the item does or does not match a query.
    #[arg(long, value_name = "QUERY")]
    pub against: Option<String>,
    /// Not built (step 10): the item as a named basis held it.
    #[arg(long, value_name = "BASIS")]
    pub basis: Option<String>,
}

/// Print a failure on the path the caller chose, and exit 1 through main.
fn fail(e: SearchError, json: bool) -> anyhow::Error {
    if json {
        println!("{}", e.to_json());
    } else {
        match &e {
            SearchError::Language(e) => {
                eprintln!("Error: {}", e.message);
                e.readings.iter().for_each(|r| eprintln!("    {r}"));
            }
            SearchError::Scope {
                message, offers, ..
            } => {
                eprintln!("Error: {message}");
                offers.iter().for_each(|o| eprintln!("    {o}"));
            }
            SearchError::Store(e) => eprintln!("Error: {e:#}"),
        }
    }
    crate::AlreadyReported.into()
}

fn emit<T: Serialize>(value: &T, json: bool, text: impl FnOnce(&T) -> String) -> Result<()> {
    if json {
        println!("{}", serde_json::to_string_pretty(value)?);
    } else {
        print!("{}", text(value));
    }
    Ok(())
}

pub fn search(args: SearchArgs, json: bool) -> Result<()> {
    let unbuilt = [
        ("--count", args.count.is_some()),
        ("--cross", args.cross.is_some()),
        ("--sum", args.sum.is_some()),
        ("--fields", args.fields.is_some()),
        ("--next", args.next.is_some()),
        ("--explain", args.explain.is_some()),
        ("--context", args.context.is_some()),
        ("--view locations", args.view.is_some()),
        ("--print-request", args.print_request),
        ("--request", args.request.is_some()),
        ("--rebind", args.rebind),
    ];
    if let Some((construct, _)) = unbuilt.iter().find(|(_, typed)| *typed) {
        return Err(fail(not_built(construct).into(), json));
    }
    if let Some(names) = &args.describe {
        let described = describe(names).map_err(|e| fail(e.into(), json))?;
        return emit(&described, json, describe_text);
    }
    let text = match &args.query_file {
        Some(path) if path == "-" => {
            let mut text = String::new();
            std::io::stdin().read_to_string(&mut text)?;
            text
        }
        Some(path) => std::fs::read_to_string(path)
            .with_context(|| format!("reading the query from {path}"))?,
        None => args.query.clone().unwrap_or_default(),
    };
    let realm = match &args.realm {
        Some(word) => Some(Realm::parse(word).map_err(|e| fail(e, json))?),
        None => None,
    };
    let request = Request {
        scope: Scope::default(),
        query: acquisition_search::answer::QueryInput {
            text: Some(text.trim().to_string()),
            tree: None,
        },
        view: View {
            rows: Rows {
                limit: Some(args.limit),
                sort: args.sort.clone(),
                desc: args.desc,
            },
        },
    };
    // an authoring error is said before the store is read
    request.bind().map_err(|e| fail(e.into(), json))?;
    let store = store_cmd::open()?;
    let corpus = Corpus::load(&store, realm.as_ref()).map_err(|e| fail(e, json))?;
    let answered = answer(&corpus, &request).map_err(|e| fail(e, json))?;
    emit(&answered, json, |a| answer_text(a, args.routes))
}

pub fn show_item(args: ShowArgs, json: bool) -> Result<()> {
    for (construct, typed) in [
        ("show --against", args.against.is_some()),
        ("show --basis", args.basis.is_some()),
    ] {
        if typed {
            return Err(fail(not_built(construct).into(), json));
        }
    }
    let store = store_cmd::open()?;
    let shown = show(&store, &args.id, args.body).map_err(|e| fail(e, json))?;
    emit(&shown, json, shown_text)
}

// ---- text, a function of the JSON ---------------------------------------------------------------

fn plural(n: usize, one: &str, many: &str) -> String {
    format!("{n} {}", if n == 1 { one } else { many })
}

fn number(value: &serde_json::Value) -> String {
    value.to_string()
}

/// A name and a type line as the game shows them together; a rendering,
/// never a field (the reference, *Item-level*).
fn title(name: Option<&str>, typeline: Option<&str>) -> String {
    match (name, typeline) {
        (Some(name), Some(typeline)) => format!("{name} {typeline}"),
        (name, typeline) => name.or(typeline).unwrap_or("(no name)").to_string(),
    }
}

fn place_text(place: &acquisition_search::corpus::Place) -> String {
    let mut out = place
        .league
        .clone()
        .unwrap_or_else(|| "no league".to_string());
    if let Some(parent) = &place.parent {
        out.push_str(&format!(
            " / {}",
            parent.name.as_deref().unwrap_or(&parent.id)
        ));
    }
    out.push_str(&format!(
        " / {}",
        place.name.as_deref().unwrap_or(&place.id)
    ));
    if place.kind == "character" {
        out.push_str(&format!(
            " ({})",
            place.container.as_deref().unwrap_or("character")
        ));
    }
    if let Some(parent) = &place.socketed_in {
        out.push_str(&format!(
            " · socketed in {}",
            parent.name.as_deref().unwrap_or(&parent.id)
        ));
    }
    out
}

fn answer_text(a: &Answer, all_routes: bool) -> String {
    let now = acquisition_store::now();
    let mut out = String::new();
    let mut line = |s: String| {
        out.push_str(s.trim_end());
        out.push('\n');
    };
    let query = if a.query.text.is_empty() {
        "(every item in scope)"
    } else {
        &a.query.text
    };
    line(format!("query   {query}"));

    let scope = &a.scope;
    let account = scope.account.name.as_deref().unwrap_or(&scope.account.uuid);
    let mut first = format!(
        "scope   account {account} · {} · live · {} · {} fetched",
        scope.realm.as_str(),
        plural(scope.coverage.items, "item", "items"),
        plural(scope.coverage.fetched, "location", "locations"),
    );
    if let (Some(oldest), Some(newest)) = (scope.coverage.oldest_fetch, scope.coverage.newest_fetch)
    {
        first.push_str(&format!(
            " (oldest {}, newest {})",
            ago(now, Some(oldest)),
            ago(now, Some(newest))
        ));
    }
    line(first);
    line(format!(
        "        {} never fetched · location list seen {} · the full list: --view locations, not built (step 10)",
        scope.coverage.never_fetched,
        ago(now, scope.coverage.list_seen),
    ));
    if scope.realm_not_held {
        line(format!(
            "        this store holds nothing under {}: it holds {}",
            scope.realm.as_str(),
            if scope.coverage.realms_held.is_empty() {
                "no realm".to_string()
            } else {
                scope.coverage.realms_held.join(", ")
            }
        ));
    }
    line(format!(
        "basis   snapshot {} · facts v{} · derivation {}",
        a.basis.snapshot.response, a.basis.snapshot.facts_version, a.basis.derivation
    ));

    if !a.terms.is_empty() {
        line(format!(
            "terms   each term evaluated independently over live {} items in all leagues",
            scope.realm.as_str()
        ));
        let width = a
            .terms
            .iter()
            .map(|t| t.term.chars().count())
            .max()
            .unwrap_or(0)
            .min(60);
        for term in &a.terms {
            let mut counts: Vec<String> = [
                (&term.matched, "matched"),
                (&term.failed, "failed"),
                (&term.lacked, "lacked"),
                (&term.undecided, "undecided"),
            ]
            .iter()
            .filter(|(count, word)| count.count > 0 || *word == "matched")
            .map(|(count, word)| format!("{} {word}", count.count))
            .collect();
            if let Some(together) = term.together.as_ref().filter(|t| t.count.count > 0) {
                counts.push(format!(
                    "{} {} {} only together",
                    together.count.count,
                    if together.count.count == 1 {
                        "reaches"
                    } else {
                        "reach"
                    },
                    number(&together.bound)
                ));
            }
            line(format!(
                "  {:<5}{:<width$}  {}",
                term.path,
                term.term,
                counts.join(" · ")
            ));
            if let Some(resolved) = &term.resolved {
                // the five most carried; the JSON lists ten
                const SHOWN: usize = 5;
                let mut values: Vec<String> = resolved
                    .values
                    .iter()
                    .take(SHOWN)
                    .map(|v| format!("{} ({})", v.value.replace('\n', "\\n"), v.items))
                    .collect();
                let more = resolved.more + resolved.values.len().saturating_sub(SHOWN);
                if more > 0 {
                    values.push(format!("{more} more: --count line, not built (step 5)"));
                }
                if !values.is_empty() {
                    line(format!("       → {}", values.join(" · ")));
                }
            }
        }
    }

    let mut total = format!("total   {}", plural(a.total.matched, "match", "matches"));
    if a.total.undecided.count > 0 {
        total.push_str(&format!(" · {} undecided", a.total.undecided.count));
    }
    line(total);

    for (i, row) in a.rows.iter().enumerate() {
        let mut head = title(row.name.as_deref(), row.typeline.as_deref());
        if let Some(rarity) = &row.rarity {
            head.push_str(&format!(" · {}", rarity.to_lowercase()));
        }
        head.push_str(&format!(" · {}", place_text(&row.place)));
        line(format!(
            "{}{head}",
            if i == 0 { "rows    " } else { "        " }
        ));
        let mut shows: Vec<String> = Vec::new();
        if let Some(sorted) = &row.sort {
            shows.push(match (&sorted.value, sorted.status) {
                (Some(value), None) => format!("sorts by {}", number(value)),
                (Some(value), Some(status)) => format!("sorts last: {status} at {}", number(value)),
                (None, status) => format!("sorts last: {}", status.unwrap_or("no value")),
            });
        }
        for touched in &row.matched {
            for evidence in &touched.shows {
                use acquisition_search::answer::Evidence;
                shows.push(match evidence {
                    Evidence::Line {
                        source,
                        flags,
                        text,
                    } => {
                        let kind: Vec<&str> = std::iter::once(source.as_str())
                            .chain(flags.iter().map(String::as_str))
                            .collect();
                        format!("{} ({})", text.replace('\n', " / "), kind.join(", "))
                    }
                    Evidence::Shown { part, text } => format!("{text} ({part})"),
                    Evidence::Value { name, value } => format!("{name} {}", number(value)),
                    Evidence::Undecided { path, term, reason } => format!(
                        "term {path} {term} is undecided: {} unread — {}; {}",
                        reason.unread, reason.problem, reason.hint
                    ),
                });
            }
        }
        shows.dedup();
        if !shows.is_empty() {
            line(format!("            {}", shows.join(" · ")));
        }
        line(format!("            id {}", row.id));
    }
    let rows = &a.view.rows;
    if rows.left_out > 0 {
        line(format!(
            "more    {} of {} shown: a larger --limit returns the rest (--next: not built, step 10)",
            rows.returned, a.total.matched
        ));
    }

    if let Some(zero) = &a.zero {
        for nothing in &zero.resolved_to_nothing {
            line(format!(
                "nothing in scope carries the {} of term {}: {} — {}",
                nothing.of, nothing.path, nothing.term, zero.said
            ));
            for s in &nothing.suggestions {
                line(format!(
                    "            {}   ({})",
                    s.term,
                    plural(s.items, "item", "items")
                ));
            }
        }
        if scope.coverage.never_fetched > 0 {
            line(format!(
                "            {} never fetched: nothing here says what they hold",
                plural(
                    scope.coverage.never_fetched,
                    "location was",
                    "locations were"
                )
            ));
        }
    }

    for (i, item) in a.total.undecided_items.iter().enumerate() {
        line(format!(
            "{}{} · id {}",
            if i == 0 { "undecided " } else { "          " },
            item.name.as_deref().unwrap_or("(no name)"),
            item.id
        ));
        for why in &item.why {
            line(format!(
                "            term {} {}: {} unread — {}; {}",
                why.path, why.term, why.reason.unread, why.reason.problem, why.reason.hint
            ));
        }
    }
    if a.total.undecided.count > a.total.undecided_items.len() {
        line(format!(
            "          {} more: the undecided route below lists them",
            a.total.undecided.count - a.total.undecided_items.len()
        ));
    }

    // routes: by default the ones a next step hangs on — what is undecided,
    // what holds only together, and, when nothing matched, what each term
    // matched on its own; `--routes` prints every count's
    let mut first: Vec<(String, &Count)> = Vec::new();
    let mut rest: Vec<(String, &Count)> = Vec::new();
    first.push(("undecided at the root".to_string(), &a.total.undecided));
    for term in &a.terms {
        first.push((format!("term {} undecided", term.path), &term.undecided));
        if let Some(together) = &term.together {
            first.push((format!("term {} only together", term.path), &together.count));
        }
    }
    for term in &a.terms {
        let matched = (format!("term {} matched", term.path), &term.matched);
        if a.total.matched == 0 {
            first.push(matched)
        } else {
            rest.push(matched)
        }
        rest.push((format!("term {} failed", term.path), &term.failed));
        rest.push((format!("term {} lacked", term.path), &term.lacked));
    }
    let routed = |routes: Vec<(String, &'_ Count)>| -> Vec<(String, usize, String)> {
        routes
            .into_iter()
            .filter_map(|(what, count)| {
                count
                    .route
                    .as_ref()
                    .map(|r| (what, count.count, r.command()))
            })
            .collect()
    };
    let (first, rest) = (routed(first), routed(rest));
    let hidden = if all_routes { 0 } else { rest.len() };
    let shown: Vec<&(String, usize, String)> = first
        .iter()
        .chain(rest.iter().filter(|_| all_routes))
        .collect();
    if !shown.is_empty() || hidden > 0 {
        line(if hidden > 0 {
            format!("routes  each returns exactly the members counted ({hidden} more: --routes)")
        } else {
            "routes  each returns exactly the members counted".to_string()
        });
        for (what, count, command) in shown {
            line(format!("  {what} ({count}), over the scope:"));
            line(format!("    {command}"));
        }
    }
    out
}

fn describe_text(d: &Describe) -> String {
    let mut out = String::new();
    let mut block = |head: &str, entries: &[acquisition_search::describe::Named]| {
        if entries.is_empty() {
            return;
        }
        out.push_str(&format!("{head}\n"));
        for e in entries {
            out.push_str(&format!("  {:<12}{:<11}{}\n", e.name, e.kind, e.what));
            if !e.values.is_empty() {
                out.push_str(&format!("  {:<23}one of: {}\n", "", e.values.join(", ")));
            }
            if !e.examples.is_empty() {
                out.push_str(&format!("  {:<23}e.g. {}\n", "", e.examples.join("   ")));
            }
        }
    };
    block("fields", &d.fields);
    block("inside line( … )", &d.line);
    block("operators", &d.operators);
    block("slots", &d.slots);
    block("computed values", &d.computed);
    if !d.not_built.is_empty() {
        out.push_str("not built, refused by name\n");
        for n in &d.not_built {
            out.push_str(&format!(
                "  {:<18}step {:<3}{}\n",
                n.construct, n.step, n.what
            ));
        }
    }
    if !d.limits.is_empty() {
        out.push_str("limits, as the search states them\n");
        for l in &d.limits {
            out.push_str(&format!("  {:<6}{}\n", l.id, l.said));
        }
    }
    out
}

fn shown_text(s: &Shown) -> String {
    let item = &s.item;
    let mut out = String::new();
    let mut head = title(item.name.as_deref(), item.typeline.as_deref());
    for (label, value) in [("", &item.rarity), ("frame ", &item.frame)] {
        if let Some(value) = value {
            head.push_str(&format!(" · {label}{}", value.to_lowercase()));
        }
    }
    out.push_str(&format!("{head}\n"));
    out.push_str(&format!("id      {}\n", item.facts.id));
    out.push_str(&format!(
        "place   {} · {}\n",
        s.place.realm,
        place_text(&s.place)
    ));
    if let Some(base) = &item.base {
        out.push_str(&format!("base    {base}\n"));
    }
    if let Some(stack) = item.stack {
        out.push_str(&format!("stack   {stack}\n"));
    }
    if let Some(note) = &item.note {
        out.push_str(&format!("note    {note}\n"));
    }
    if !item.flags.is_empty() {
        out.push_str(&format!("is      {}\n", item.flags.join(", ")));
    }
    let shown: Vec<&str> = item
        .properties
        .iter()
        .map(|p| p.text.as_str())
        .chain(item.item_level.as_deref())
        .chain(item.requires.as_deref())
        .collect();
    for (i, text) in shown.iter().enumerate() {
        out.push_str(&format!(
            "{}{}\n",
            if i == 0 { "shown   " } else { "        " },
            text.replace('\n', " / ")
        ));
    }
    for (i, l) in s.lines.iter().enumerate() {
        let kind: Vec<&str> = std::iter::once(l.line.source.as_str())
            .chain(l.line.flags.iter().map(String::as_str))
            .collect();
        out.push_str(&format!(
            "{}{:<22}{}\n",
            if i == 0 { "lines   " } else { "        " },
            kind.join(", "),
            l.line.text.replace('\n', " / ")
        ));
        let slots: Vec<String> = l
            .slots
            .iter()
            .map(|(word, n)| format!("{word} {}", number(n)))
            .collect();
        out.push_str(&format!(
            "        {:<22}\"{}\"{}\n",
            "",
            l.line.template.replace('\n', "\\n"),
            if slots.is_empty() {
                String::new()
            } else {
                format!("   {}", slots.join(" · "))
            }
        ));
        if let Some(limit) = l.limit {
            out.push_str(&format!("        {:<22}{limit}\n", ""));
        }
    }
    if item.unread.is_empty() {
        out.push_str("unread  nothing\n");
    }
    for (i, u) in item.unread.iter().enumerate() {
        out.push_str(&format!(
            "{}{}\n",
            if i == 0 { "unread  " } else { "        " },
            u.problem
        ));
    }
    out.push_str(&format!(
        "basis   snapshot {} · facts v{} · derivation {}\n",
        s.basis.snapshot.response, s.basis.snapshot.facts_version, s.basis.derivation
    ));
    if let Some(body) = &s.body {
        out.push_str(&format!("body    {body}\n"));
    }
    out
}
