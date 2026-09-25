//! C93's own definition, as a test: what an answer calls decided on an
//! item with something unread is the same under every completion of it.
//! Step 4b's completion property (the build plan), through the crate's
//! boundary alone: a request in, an answer out, as JSON.
//!
//! An item is generated with holes — a flag that is no yes or no, an array
//! that is no array, an element that is no line or no socket, a number or a
//! name of the wrong type, a socket's colour or group — and stored beside
//! completions of itself: what could be read kept, each unread flag a yes
//! and a no, each unread array none and lines of the templates the
//! generated queries name, each unread number several, each unread socket
//! several colours and groups.
//! The evaluator over a fully readable body is the oracle; this asks only
//! that the stored item's answer never contradicts it.
//!
//! **What binds** — one counterexample is a defect:
//! - a completion leaves nothing open: no term, no root, no sort scalar;
//! - a term matched on the stored item is matched on every completion, and
//!   one that lacked is lacked on every one: an absence is claimed only
//!   when everything that could hold the thing was readable (C93);
//! - a term that failed is false on every completion. On a line's group
//!   the false may be split the other way — the stored item *failed* where
//!   a completion *lacked* — since *failed* on a group says the truth is
//!   established and the absence is not: an occurrence the selector may or
//!   may not pick is no known absence (`eval.rs`). Counted, never refused;
//! - the root's yes or no is every completion's;
//! - a sort scalar that is a value is every completion's value, and `no
//!   satisfying occurrence` is every completion's;
//! - an item counted as reaching a bound only together does so under
//!   every completion.
//!
//! **What is measured, never required**: how often an answer called
//! undecided is the same under every completion tried. A sampler cannot
//! show that no completion differs, and a rule may be careful on purpose.
//! `cargo test -p acquisition-search --test generated_completion -- --ignored --nocapture`
//! prints the tally the build plan's record explains.
//!
//! No query here holds `undecided( … )`: it says something of the evidence
//! and not of the item, so no completion is owed the same answer.

mod common;

use std::collections::{BTreeMap, HashMap};

use acquisition_search::Corpus;
use common::generated::*;
use proptest::prelude::*;
use serde_json::Value;

const STORED: &str = "i0";
/// How many of each kind the measurement prints.
const EXAMPLES: usize = 8;

#[derive(Debug, Default)]
struct Tally {
    cases: usize,
    erred: usize,
    /// A group that failed as stored and lacked under a completion.
    failed_then_lacked: usize,
    open: BTreeMap<&'static str, Open>,
    decided: BTreeMap<&'static str, usize>,
}

/// What was undecided as stored, by what the completions tried made of it:
/// they differed, or every one said the same — and then which, since an
/// undecided that every completion makes true is where a missed witness
/// would show.
#[derive(Debug, Default)]
struct Open {
    differed: usize,
    all: BTreeMap<String, (usize, Vec<String>)>,
}

impl Tally {
    fn open(&mut self, what: &'static str, each: &[String], example: impl FnOnce() -> String) {
        let entry = self.open.entry(what).or_default();
        if each.windows(2).any(|w| w[0] != w[1]) {
            entry.differed += 1;
            return;
        }
        let said = if what == "a sort scalar" {
            "one value".to_string()
        } else {
            each.first().cloned().unwrap_or_default()
        };
        let (count, examples) = entry.all.entry(said).or_default();
        *count += 1;
        if examples.len() < EXAMPLES {
            examples.push(example());
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Outcome {
    Matched,
    Failed,
    Lacked,
    Undecided,
}

impl Outcome {
    fn truth(self) -> Option<bool> {
        match self {
            Outcome::Matched => Some(true),
            Outcome::Failed | Outcome::Lacked => Some(false),
            Outcome::Undecided => None,
        }
    }
}

fn said_of(truth: Option<bool>) -> String {
    match truth {
        Some(true) => "yes",
        Some(false) => "no",
        None => "undecided",
    }
    .to_string()
}

fn check(
    corpus: &Corpus,
    scope: &Ids,
    q: &Q,
    sorts: &[Sort],
    stored: &Value,
    tally: &mut Tally,
) -> Result<(), String> {
    tally.cases += 1;
    let text = q_text(q, Spelling::Authored);
    let said = |what: &str| format!("`{text}` over {stored}: {what}");
    let completions: Vec<&String> = scope.iter().filter(|id| *id != STORED).collect();
    let mut cache = HashMap::new();

    match run(corpus, &request(&text, None, false, scope.len())) {
        // an authoring error says nothing of any item
        Err(_) => tally.erred += 1,
        Ok(answer) => {
            let kinds = terms(q);
            let listed = answer["terms"].as_array().ok_or("no terms block")?;
            if listed.len() != kinds.len() {
                return Err(said(
                    "the generator's terms and the answer's differ in number",
                ));
            }
            for (term, (_, kind)) in listed.iter().zip(&kinds) {
                let mut of: HashMap<String, Outcome> = HashMap::new();
                for (name, outcome) in [
                    ("matched", Outcome::Matched),
                    ("failed", Outcome::Failed),
                    ("lacked", Outcome::Lacked),
                    ("undecided", Outcome::Undecided),
                ] {
                    for id in members(corpus, &term[name], scope, &mut cache)? {
                        if of.insert(id, outcome).is_some() {
                            return Err(said("a term's routes overlap"));
                        }
                    }
                }
                let asked = &term["term"];
                let outcome = |id: &str| {
                    of.get(id)
                        .copied()
                        .ok_or_else(|| said(&format!("{asked} has no outcome on {id}")))
                };
                let each: Vec<Outcome> = completions
                    .iter()
                    .map(|id| outcome(id))
                    .collect::<Result<_, _>>()?;
                if each.contains(&Outcome::Undecided) {
                    return Err(said(&format!(
                        "{asked} is undecided on an item with nothing unread"
                    )));
                }
                let was = outcome(STORED)?;
                match was {
                    Outcome::Undecided => {
                        let each: Vec<String> = each.iter().map(|o| said_of(o.truth())).collect();
                        let what = match kind {
                            TermKind::Group => "a line's group",
                            TermKind::Sum => "a sum's comparison",
                            _ => "a field, a flag or a phrase",
                        };
                        tally.open(what, &each, || format!("{asked} over {stored}"));
                    }
                    Outcome::Failed if *kind == TermKind::Group => {
                        if each.iter().any(|o| o.truth() != Some(false)) {
                            return Err(said(&format!(
                                "{asked} failed as stored and matched under a completion"
                            )));
                        }
                        if each.contains(&Outcome::Lacked) {
                            tally.failed_then_lacked += 1;
                        }
                        *tally.decided.entry("a term").or_default() += 1;
                    }
                    decided => {
                        if let Some(other) = each.iter().find(|o| **o != decided) {
                            return Err(said(&format!(
                                "{asked} was {decided:?} as stored and {other:?} under a completion"
                            )));
                        }
                        *tally.decided.entry("a term").or_default() += 1;
                    }
                }
                if let Some(together) = term.get("together") {
                    let ids = members(corpus, together, scope, &mut cache)?;
                    if ids.contains(STORED) && completions.iter().any(|id| !ids.contains(*id)) {
                        return Err(said(&format!(
                            "{asked} reaches its bound only together as stored, and not under a completion"
                        )));
                    }
                }
            }

            // the root
            let matched: Ids = answer["rows"]
                .as_array()
                .ok_or("no rows")?
                .iter()
                .filter_map(|row| row["id"].as_str().map(str::to_string))
                .collect();
            let undecided = members(corpus, &answer["total"]["undecided"], scope, &mut cache)?;
            let root = |id: &str| match (matched.contains(id), undecided.contains(id)) {
                (true, _) => Some(true),
                (_, true) => None,
                _ => Some(false),
            };
            let each: Vec<Option<bool>> = completions.iter().map(|id| root(id)).collect();
            if each.contains(&None) {
                return Err(said("the root is undecided on an item with nothing unread"));
            }
            match root(STORED) {
                None => {
                    let each: Vec<String> = each.iter().map(|t| said_of(*t)).collect();
                    tally.open("the root", &each, || format!("`{text}` over {stored}"));
                }
                decided => {
                    if each.iter().any(|truth| *truth != decided) {
                        return Err(said(&format!(
                            "the root was {decided:?} as stored and another under a completion"
                        )));
                    }
                    *tally.decided.entry("the root").or_default() += 1;
                }
            }
        }
    }

    // the sort scalar, asked of every item: the empty query matches all
    for sort in sorts {
        let Some(value) = sort_text(sort, Spelling::Authored) else {
            continue;
        };
        let Ok(answer) = run(corpus, &request("", Some(&value), false, scope.len())) else {
            continue;
        };
        let scalars: HashMap<&str, &Value> = answer["rows"]
            .as_array()
            .ok_or("no rows")?
            .iter()
            .filter_map(|row| Some((row["id"].as_str()?, &row["sort"])))
            .collect();
        let scalar = |id: &str| {
            scalars
                .get(id)
                .copied()
                .ok_or_else(|| format!("`--sort {value}` over {stored}: no row for {id}"))
        };
        let each: Vec<&Value> = completions
            .iter()
            .map(|id| scalar(id))
            .collect::<Result<_, _>>()?;
        if each.iter().any(|s| s["status"] == "incomplete") {
            return Err(format!(
                "`--sort {value}` over {stored}: incomplete on an item with nothing unread"
            ));
        }
        let was = scalar(STORED)?;
        if was["status"] == "incomplete" {
            let each: Vec<String> = each.iter().map(|s| s.to_string()).collect();
            tally.open("a sort scalar", &each, || {
                format!("`--sort {value}` over {stored}")
            });
        } else {
            if let Some(other) = each.iter().find(|s| **s != was) {
                return Err(format!(
                    "`--sort {value}` over {stored}: {was} as stored and {other} under a completion"
                ));
            }
            *tally.decided.entry("a sort scalar").or_default() += 1;
        }
    }
    Ok(())
}

type Case = (
    Q,
    (G, String),
    (G, String),
    Body,
    Vec<(Body, u64)>,
    Vec<LineM>,
);

fn case() -> impl Strategy<Value = Case> {
    (
        query(false),
        (group(), proptest::sample::select(vec!["arg1", "avg"])),
        (group(), proptest::sample::select(vec!["arg1", "avg"])),
        holed(),
        proptest::collection::vec((body(false), any::<u64>()), 6),
        proptest::collection::vec(line(false), 5),
    )
        .prop_map(|(q, (p, ps), (s, ss), stored, fills, singles)| {
            (
                q,
                (p, ps.to_string()),
                (s, ss.to_string()),
                stored,
                fills,
                singles,
            )
        })
}

/// The completions tried of one stored body: nothing where something was
/// unread, every flag a no and then a yes; six as the dice fill them; one
/// line alone, five times; and every line at once, flagged no and yes. The
/// numbers, the names and the item's own flags go round a fixed list.
fn completions(stored: &Body, fills: &[(Body, u64)], singles: &[LineM]) -> Vec<Body> {
    let mut out = vec![
        stored.completed(&Body::blank(Tri::No), Coins::No),
        stored.completed(&Body::blank(Tri::Yes), Coins::Yes),
    ];
    for (n, (fill, coins)) in fills.iter().enumerate() {
        let round = Body::nth(n, Vec::new());
        let fill = Body {
            ilvl: round.ilvl,
            name: round.name,
            corrupted: round.corrupted,
            shaper: round.shaper,
            hunter: round.hunter,
            ..fill.clone()
        };
        out.push(stored.completed(&fill, Coins::Tossed(*coins)));
    }
    for (n, single) in singles.iter().enumerate() {
        let fill = Body::nth(n + 1, vec![single.clone()]);
        out.push(stored.completed(&fill, Coins::Tossed(n as u64)));
    }
    out.push(stored.completed(&Body::nth(6, Body::every_line(Tri::No)), Coins::No));
    out.push(stored.completed(&Body::nth(7, Body::every_line(Tri::Yes)), Coins::Yes));
    out
}

fn checked(case: &Case, tally: &mut Tally) -> Result<(), String> {
    let (q, projection, sum, stored, fills, singles) = case;
    let mut bodies = vec![stored.clone()];
    bodies.extend(completions(stored, fills, singles));
    if let Some(left) = bodies[1..].iter().find(|b| b.has_holes()) {
        return Err(format!("a completion with a hole left in it: {left:?}"));
    }
    let (corpus, scope) = fixture(bodies.iter().map(Body::json).collect());
    let sorts = [
        Sort::Proj(projection.0.clone(), projection.1.clone()),
        Sort::Sum(sum.0.clone(), sum.1.clone()),
        Sort::Field("ilvl"),
        Sort::Field("pseudo.total_res"),
        Sort::Field("pseudo.dps"),
        Sort::Field("pseudo.pdps"),
        Sort::Field("links"),
        Sort::Field("sockets.red"),
    ];
    check(&corpus, &scope, q, &sorts, &stored.json(), tally)
}

proptest! {
    #![proptest_config(ProptestConfig { cases: cases(256), failure_persistence: None, ..ProptestConfig::default() })]
    #[test]
    fn c93_what_is_decided_on_the_stored_item_is_decided_so_under_every_completion(case in case()) {
        checked(&case, &mut Tally::default()).map_err(TestCaseError::fail)?;
    }
}

/// The measured half: run by hand, never in the gate.
#[test]
#[ignore = "a measurement, printed: run by hand with --ignored --nocapture"]
fn how_often_an_undecided_answer_is_the_same_under_every_completion_tried() {
    use proptest::strategy::ValueTree;
    let mut runner = proptest::test_runner::TestRunner::deterministic();
    let mut tally = Tally::default();
    for _ in 0..cases(2000) {
        let case = case().new_tree(&mut runner).unwrap().current();
        checked(&case, &mut tally).unwrap();
    }
    println!(
        "{} cases, {} of them authoring errors; decided and held: {:?}",
        tally.cases, tally.erred, tally.decided
    );
    println!(
        "a group failed as stored, lacked under a completion: {}",
        tally.failed_then_lacked
    );
    for (what, open) in &tally.open {
        println!(
            "undecided — {what}: {} differed under the completions tried",
            open.differed
        );
        for (said, (count, examples)) in &open.all {
            println!("  {count} did not: every completion said {said}");
            for example in examples {
                println!("      {example}");
            }
        }
    }
}
