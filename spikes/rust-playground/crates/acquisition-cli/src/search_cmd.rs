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

use acquisition_search::answer::{Answer, Count, Request, Rows, Scope, View, ViewOut};
use acquisition_search::corpus::Realm;
use acquisition_search::counts::{Bucket, CrossOut, Label, SumOf, Summed, Table};
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
    /// several it is required, one of them or `all` (C96).
    #[arg(long)]
    pub realm: Option<String>,
    /// Read the query from a file, or from stdin with `-`: a query with an
    /// apostrophe needs no shell quoting this way.
    #[arg(long, value_name = "FILE|-", conflicts_with = "query")]
    pub query_file: Option<String>,
    /// Order the rows by a value: `ilvl`, `stack`, `'line("T").arg1'`,
    /// `'sum("T")'`, `pseudo.total_res`. A line's scalar is its largest
    /// satisfying occurrence; an item with none sorts last either way
    /// (C92).
    #[arg(long, value_name = "VALUE")]
    pub sort: Option<String>,
    /// Largest first.
    #[arg(long, requires = "sort")]
    pub desc: bool,
    /// How many rows to return — or, of a count, how many values a table
    /// lists; the rest are counted.
    #[arg(long, default_value_t = acquisition_search::answer::DEFAULT_LIMIT)]
    pub limit: usize,
    /// Print every count's route — the command that returns exactly its
    /// members — instead of the first few.
    #[arg(long)]
    pub routes: bool,
    /// The language as this build knows it: fields, what a line has,
    /// operators, closed value sets, slots, the computed values with
    /// their definitions, what is not built and the limits stated; or
    /// only the entries named (`--describe league,line,total_res`).
    #[arg(long, value_name = "NAME,…", num_args = 0..=1, value_delimiter = ',')]
    pub describe: Option<Vec<String>>,
    /// Count the matches by a key, one table for each key named, and show
    /// no rows: a field (`tab`, `league`, `rarity`, `base`, …; `--describe
    /// counts` lists them), or `line`, the vocabulary — the templates the
    /// matching items carry, ranked, each with the term that selects it and
    /// the range of its numbers. `line:` takes the rest of the list as
    /// texts to narrow by, a table each, and `~` before one makes it a
    /// pattern: `--count tab,line:resist,life`. An item with no value is
    /// counted under `none`, one whose value could not be read under
    /// `undecided` (C105); every count has its route (`--routes`).
    #[arg(long, value_name = "KEY,…")]
    pub count: Option<String>,
    /// Count the matches by two fields at once, one table of the cells
    /// that hold an item: `--cross league,tab`.
    #[arg(long, value_name = "KEY,KEY")]
    pub cross: Option<String>,
    /// Beside each count, the sum of one number over its items: `stack`,
    /// `ilvl`, `'sum("T")'`, `pseudo.total_res`. An item lacking the thing
    /// adds nothing and is counted as lacking; one unread leaves a
    /// subtotal marked incomplete (C95).
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
    let keys = |raw: Option<&str>| -> Result<Option<Vec<String>>> {
        raw.map(|raw| {
            keys(raw).map_err(|message| {
                fail(
                    SearchError::Scope {
                        kind: "view",
                        message,
                        offers: Vec::new(),
                    },
                    json,
                )
            })
        })
        .transpose()
    };
    let view = View::of(
        keys(args.count.as_deref())?,
        keys(args.cross.as_deref())?,
        args.sum.clone(),
        Rows {
            limit: Some(args.limit),
            sort: args.sort.clone(),
            desc: args.desc,
        },
    )
    .map_err(|e| fail(e.into(), json))?;
    let request = Request {
        scope: Scope::default(),
        query: acquisition_search::answer::QueryInput {
            text: Some(text.trim().to_string()),
            tree: None,
        },
        view,
    };
    // an authoring error is said before the store is read
    request.check().map_err(|e| fail(e.into(), json))?;
    let store = store_cmd::open()?;
    let corpus = Corpus::load(&store, realm.as_ref()).map_err(|e| fail(e, json))?;
    let answered = answer(&corpus, &request).map_err(|e| fail(e, json))?;
    emit(&answered, json, |a| answer_text(a, args.routes))
}

/// The keys of `--count`, as a terminal spells them: a comma list, where
/// `line:` takes the rest of the list as its texts — `line:resist,life` is
/// the vocabulary narrowed twice, a table each (the reference's synopsis,
/// `--count line[:text,…]`) — and a text is a pattern after `~`. A text
/// with a comma, or a `~` of its own, is quoted, `"…"`, with the
/// language's three escapes (`\"`, `\\`, `\n`) and no other; what is
/// quoted is the text as written, never syntax, and a quote that does not
/// close is an error, as it is in a query (outside audit, 2026-09-22).
fn keys(raw: &str) -> Result<Vec<String>, String> {
    // each part as characters, each marked quoted or not: what was quoted
    // is text as written, and only an unquoted character is syntax
    let mut parts: Vec<Vec<(char, bool)>> = vec![Vec::new()];
    let (mut quoted, mut escaped) = (false, false);
    for c in raw.chars() {
        let part = parts
            .last_mut()
            .unwrap_or_else(|| unreachable!("one part at least"));
        match c {
            _ if escaped => {
                part.push((
                    match c {
                        '"' | '\\' => c,
                        'n' => '\n',
                        other => {
                            return Err(format!(
                                "`\\{other}` is no escape a quoted text takes: \\\" \\\\ and \\n are the only ones"
                            ));
                        }
                    },
                    true,
                ));
                escaped = false;
            }
            '\\' if quoted => escaped = true,
            '"' => quoted = !quoted,
            ',' if !quoted => parts.push(Vec::new()),
            _ => part.push((c, quoted)),
        }
    }
    if quoted || escaped {
        return Err(format!(
            "a quoted text in `{raw}` never closes: a text with a comma or a ~ of its own is written \"…\", the language's escapes inside"
        ));
    }
    let mut narrowing = false;
    Ok(parts
        .into_iter()
        .map(|mut part| {
            // unquoted whitespace at the ends is none of the text
            while part.first().is_some_and(|(c, q)| !q && c.is_whitespace()) {
                part.remove(0);
            }
            while part.last().is_some_and(|(c, q)| !q && c.is_whitespace()) {
                part.pop();
            }
            let text = |part: &[(char, bool)]| part.iter().map(|(c, _)| *c).collect::<String>();
            let unquoted_prefix = |part: &[(char, bool)], word: &str| {
                part.len() >= word.len()
                    && part[..word.len()]
                        .iter()
                        .zip(word.chars())
                        .all(|((c, q), w)| !q && c.eq_ignore_ascii_case(&w))
            };
            let pattern_marked = |part: &[(char, bool)]| part.first() == Some(&('~', false));
            if unquoted_prefix(&part, "line:") || unquoted_prefix(&part, "line~") {
                narrowing = true;
                let pattern = part[4].0 == '~';
                let rest = &part[5..];
                // `line:~pattern` is `line~pattern`
                return if pattern {
                    format!("line~{}", text(rest))
                } else if pattern_marked(rest) {
                    format!("line~{}", text(&rest[1..]))
                } else {
                    format!("line:{}", text(rest))
                };
            }
            if !narrowing {
                text(&part)
            } else if pattern_marked(&part) {
                format!("line~{}", text(&part[1..]))
            } else {
                format!("line:{}", text(&part))
            }
        })
        .collect())
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
        "        {} location{} never fetched · location list seen {} · the full list: --view locations, not built (step 10)",
        scope.coverage.never_fetched,
        if scope.coverage.never_fetched == 1 {
            ""
        } else {
            "s"
        },
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
        "basis   store {} · snapshot {} · facts v{} · derivation {} · classes v{} · totals v{}",
        a.basis.store,
        a.basis.snapshot.response,
        a.basis.snapshot.facts_version,
        a.basis.derivation,
        a.basis.classes,
        a.basis.totals
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
                    // the count that lists every value, where one does
                    values.push(match &resolved.rest {
                        Some(rest) => format!("{more} more: {}", rest.command()),
                        None => format!("{more} more"),
                    });
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
        // a scope that spans realms says which one each item is in, as
        // `show` does; under one realm the scope line has said it
        if a.scope.realm == Realm::All {
            head.push_str(&format!(" · {}", row.place.realm));
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
                    Evidence::Value { name, value } => format!("{name} {}", json_text(value)),
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
        for touched in row.matched.iter().filter(|t| t.left_out > 0) {
            line(format!(
                "            {} more of term {}: {}",
                touched.left_out,
                touched.path,
                a.show_command(&row.id)
            ));
        }
        line(format!("            id {}", row.id));
    }
    match &a.view {
        ViewOut::Rows(rows) if rows.left_out > 0 => line(format!(
            "more    {} of {} shown: a larger --limit returns the rest (--next: not built, step 10)",
            rows.returned, a.total.matched
        )),
        ViewOut::Rows(_) => {}
        ViewOut::Counts(counts) => {
            if let Some(sum) = &counts.sum {
                line(format!("sum     {}", sum_of_text(sum)));
            }
            for table in &counts.tables {
                table_text(table, a.scope.realm == Realm::All)
                    .into_iter()
                    .for_each(&mut line);
            }
        }
        ViewOut::Cross(cross) => {
            if let Some(sum) = &cross.sum {
                line(format!("sum     {}", sum_of_text(sum)));
            }
            cross_text(cross, a.scope.realm == Realm::All)
                .into_iter()
                .for_each(&mut line);
        }
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
        if item.why_left_out > 0 {
            line(format!(
                "            {} more unread: {}",
                item.why_left_out,
                a.show_command(&item.id)
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
    let mut buckets: Vec<(String, &Count)> = Vec::new();
    match &a.view {
        ViewOut::Rows(_) => {}
        ViewOut::Counts(counts) => {
            for table in &counts.tables {
                for bucket in &table.buckets {
                    let what = format!(
                        "{} {}",
                        table.key,
                        label_text(&bucket.label, a.scope.realm == Realm::All)
                    );
                    for kind in bucket.sources.iter().chain(&bucket.flags) {
                        buckets.push((format!("{what}, {}", kind.kind), &kind.count));
                    }
                    buckets.push((what, &bucket.count));
                }
            }
        }
        ViewOut::Cross(cross) => {
            for cell in &cross.cells {
                let of: Vec<String> = cell
                    .of
                    .iter()
                    .map(|l| label_text(l, a.scope.realm == Realm::All))
                    .collect();
                buckets.push((of.join(" × "), &cell.count));
            }
            for margin in &cross.margins {
                first.push((format!("{} (none)", margin.key), &margin.none));
                first.push((format!("{} (undecided)", margin.key), &margin.undecided));
            }
        }
    }
    // what a count could not place is what a next step hangs on
    for (what, count) in buckets {
        if what.ends_with("(none)") || what.ends_with("(undecided)") {
            first.push((what, count));
        } else {
            rest.push((what, count));
        }
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

// ---- a count's text ------------------------------------------------------------------------------------

fn json_text(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(text) => text.replace('\n', "\\n"),
        other => other.to_string(),
    }
}

/// A bucket's name: its value, or `(none)`, `(undecided)` — in brackets,
/// since a tab may be named `none`; a tab's league and id beside its
/// name, and its realm where the scope spans realms.
fn label_text(label: &Label, all_realms: bool) -> String {
    let mut out = match (label.bucket, &label.value) {
        ("value", Some(value)) => json_text(value),
        ("value", None) => "(no name)".to_string(),
        (bucket, _) => format!("({bucket})"),
    };
    if let Some(realm) = label.realm.as_ref().filter(|_| all_realms) {
        out.push_str(&format!(" · {realm}"));
    }
    if let Some(id) = &label.id {
        out.push_str(&format!(
            " · {} · {id}",
            label.league.as_deref().unwrap_or("no league")
        ));
    }
    out
}

fn summed_text(sum: &Summed) -> String {
    let mut out = number(&sum.value);
    if sum.lacking > 0 {
        out.push_str(&format!(" · {} lacking", sum.lacking));
    }
    if sum.incomplete {
        out.push_str(&format!(" · incomplete: {} unread", sum.unread));
    }
    out
}

fn sum_of_text(sum: &SumOf) -> String {
    format!("{} over every match: {}", sum.name, summed_text(&sum.total))
}

fn bucket_text(bucket: &Bucket, all_realms: bool) -> Vec<String> {
    let mut first = format!(
        "  {:>7}  {}",
        bucket.count.count,
        label_text(&bucket.label, all_realms)
    );
    if let Some(sum) = &bucket.sum {
        first.push_str(&format!("   sum {}", summed_text(sum)));
    }
    let slots: Vec<String> = bucket
        .slots
        .iter()
        .map(|s| {
            let end = |n: &Option<serde_json::Value>| n.as_ref().map_or("?".to_string(), number);
            format!(
                "{} {}..{}{}",
                s.slot,
                end(&s.min),
                end(&s.max),
                if s.incomplete { " (incomplete)" } else { "" }
            )
        })
        .collect();
    if !slots.is_empty() {
        first.push_str(&format!("   {}", slots.join(" · ")));
    }
    if !bucket.tally.is_empty() {
        let tally: Vec<String> = bucket
            .tally
            .iter()
            .map(|t| format!("{} unread {}", t.unread, t.items))
            .collect();
        first.push_str(&format!("   {}", tally.join(" · ")));
    }
    let mut out = vec![first];
    // a vocabulary row carries the term that selects it, ready to paste
    if !bucket.sources.is_empty() {
        if let Some(term) = &bucket.label.term {
            out.push(format!("           {term}"));
        }
        let kinds: Vec<String> = bucket
            .sources
            .iter()
            .chain(&bucket.flags)
            .map(|k| format!("{} {}", k.kind, k.count.count))
            .collect();
        out.push(format!("           {}", kinds.join(" · ")));
        // a kind with no route says why
        for k in bucket.sources.iter().chain(&bucket.flags) {
            if let Some(needs) = &k.needs {
                out.push(format!("           {}: {needs}", k.kind));
            }
        }
    }
    if let Some(needs) = &bucket.label.needs {
        out.push(format!("           {needs}"));
    }
    out
}

fn table_text(table: &Table, all_realms: bool) -> Vec<String> {
    let (one, many) = if table.key.starts_with("line") {
        ("template", "templates")
    } else {
        ("value", "values")
    };
    let mut head = format!(
        "count   {} · {}",
        table.key,
        plural(table.values, one, many)
    );
    if table.left_out > 0 {
        head.push_str(&format!(
            " · {} shown: a larger --limit returns the rest",
            table.values - table.left_out
        ));
    }
    std::iter::once(head)
        .chain(
            table
                .buckets
                .iter()
                .flat_map(|b| bucket_text(b, all_realms)),
        )
        .collect()
}

fn cross_text(cross: &CrossOut, all_realms: bool) -> Vec<String> {
    let mut head = format!(
        "cross   {} · {}",
        cross.keys.join(" × "),
        plural(cross.cells_in_all, "cell", "cells")
    );
    if cross.left_out > 0 {
        head.push_str(&format!(
            " · {} shown: a larger --limit returns the rest",
            cross.cells_in_all - cross.left_out
        ));
    }
    let mut out = vec![head];
    for cell in &cross.cells {
        let of: Vec<String> = cell.of.iter().map(|l| label_text(l, all_realms)).collect();
        let mut row = format!("  {:>7}  {}", cell.count.count, of.join(" × "));
        if let Some(sum) = &cell.sum {
            row.push_str(&format!("   sum {}", summed_text(sum)));
        }
        out.push(row);
        // a cell with no route says why, once per value that has none
        for label in &cell.of {
            if let Some(needs) = &label.needs {
                out.push(format!(
                    "           {}: {needs}",
                    label_text(label, all_realms)
                ));
            }
        }
    }
    for margin in &cross.margins {
        if margin.none.count + margin.undecided.count > 0 {
            out.push(format!(
                "           {}: {} none · {} undecided",
                margin.key, margin.none.count, margin.undecided.count
            ));
        }
    }
    if !cross.tally.is_empty() {
        let tally: Vec<String> = cross
            .tally
            .iter()
            .map(|t| format!("{} unread {}", t.unread, t.items))
            .collect();
        out.push(format!("           undecided: {}", tally.join(" · ")));
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
            out.push_str(&format!("  {:<16}{:<12}{}\n", e.name, e.kind, e.what));
            if !e.values.is_empty() {
                out.push_str(&format!("  {:<28}one of: {}\n", "", e.values.join(", ")));
            }
            if !e.examples.is_empty() {
                out.push_str(&format!("  {:<28}e.g. {}\n", "", e.examples.join("   ")));
            }
        }
    };
    block("fields", &d.fields);
    block("inside line( … )", &d.line);
    block("values", &d.values);
    block("composition", &d.composition);
    block("operators", &d.operators);
    block("slots", &d.slots);
    // the totals' table is named once, over the block, with its version
    // and source (C106's clause (c))
    block(&format!("computed values — {}", d.totals), &d.computed);
    block("counts", &d.counts);
    if !d.not_built.is_empty() {
        out.push_str("not built, refused by name\n");
        for n in &d.not_built {
            out.push_str(&format!(
                "  {:<22}{:<20}{}\n",
                n.construct,
                n.step_text(),
                n.what
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
    // what the class table says (C106): the class, or why there is none
    use acquisition_search::Classed;
    match &s.class {
        Classed::Is(class) => out.push_str(&format!("class   {class}\n")),
        Classed::Open(why) => out.push_str(&format!("class   undecided: {}\n", why.problem)),
        Classed::BaseUnread => out.push_str("class   undecided: the base was not read\n"),
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
        "basis   store {} · snapshot {} · facts v{} · derivation {} · classes v{} · totals v{}\n",
        s.basis.store,
        s.basis.snapshot.response,
        s.basis.snapshot.facts_version,
        s.basis.derivation,
        s.basis.classes,
        s.basis.totals
    ));
    if let Some(body) = &s.body {
        out.push_str(&format!("body    {body}\n"));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::keys;

    /// The count list's grammar, both ways: every spelling a continuation
    /// prints (`answer::listed_text`) reads back as the key it encodes,
    /// and what the reference's synopsis shows reads as it says.
    #[test]
    fn a_count_list_reads_back_what_a_continuation_prints() {
        for (typed, read) in [
            ("tab,league,rarity", vec!["tab", "league", "rarity"]),
            ("line:resist,life", vec!["line:resist", "line:life"]),
            (
                "tab,line:resist,~^adds",
                vec!["tab", "line:resist", "line~^adds"],
            ),
            ("line:~Life", vec!["line~Life"]),
            // unquoted, the comma is the list's: the review's case
            ("line~[a-z]{1,2}", vec!["line~[a-z]{1", "line:2}"]),
            (r#"line~"[a-z]{1,2}""#, vec!["line~[a-z]{1,2}"]),
            (r#"line:", half""#, vec!["line:, half"]),
            (r#"line:"a\"b""#, vec![r#"line:a"b"#]),
            (r#"line:"a\\b""#, vec![r"line:a\b"]),
            (r#"line:"~x""#, vec!["line:~x"]),
            (r#"line:"Life ""#, vec!["line:Life "]),
            (
                r#"line:life , "two words" "#,
                vec!["line:life", "line:two words"],
            ),
            (r#"line:"a\nb""#, vec!["line:a\nb"]),
        ] {
            assert_eq!(keys(typed).unwrap(), read, "{typed}");
        }
        for bad in [r#"line:"Life"#, r#"line:"Life\q""#, r#"line:"Life\"#] {
            assert!(keys(bad).is_err(), "{bad}");
        }
    }
}
