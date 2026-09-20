//! The deriver's census of a corpus, for the M2 differential
//! (`search/BUILD-PLAN.md`, "Measurements the build owes"): items in as
//! JSON lines on stdin — `{"facts": {…}, "body": "…"}`, the shape the
//! store's read hands over (C103) — and one JSON document out: a row per
//! (source, template) with its items, lines, flags, numbers and whether
//! the ranged rule takes it, the shapes properties render to, and what was
//! unread by part.
//!
//! Driven by `search/item-facts/scripts/m2-differential.py`, which sets it
//! against the Python census over the same input. Never in the gate: the
//! input is a store copy under a track's `raw/`.

use std::collections::{BTreeMap, BTreeSet};
use std::io::{self, BufRead, Write};
use std::time::Instant;

use acquisition_search::{Facts, Line, derive};
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
struct Row {
    facts: Facts,
    body: String,
}

#[derive(Default)]
struct Tally {
    items: u64,
    lines: u64,
    numbers: u64,
    flags: BTreeMap<String, u64>,
    example: String,
}

fn ranged(line: &Line) -> bool {
    line.slots().iter().any(|(word, _)| word == "low")
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut rows: BTreeMap<(String, String), Tally> = BTreeMap::new();
    let mut ranged_templates: BTreeSet<String> = BTreeSet::new();
    let mut unread: BTreeMap<(String, String), u64> = BTreeMap::new();
    // a property's shape: its array and its text with every digit a `#`
    let mut shapes: BTreeMap<(String, String), u64> = BTreeMap::new();
    let (mut items, mut properties, mut displayed) = (0u64, 0u64, 0u64);
    let mut deriving = std::time::Duration::ZERO;

    for line in io::stdin().lock().lines() {
        let row: Row = serde_json::from_str(&line?)?;
        let started = Instant::now();
        let item = derive(row.facts, &row.body);
        deriving += started.elapsed();

        items += 1;
        properties += item.properties.len() as u64;
        displayed += item.displayed().count() as u64;
        for p in &item.properties {
            let shape = p.text.replace(|c: char| c.is_ascii_digit(), "#");
            *shapes.entry((p.array.clone(), shape)).or_default() += 1;
        }
        let mut seen = BTreeSet::new();
        for line in &item.lines {
            let key = (line.source.clone(), line.template.clone());
            let tally = rows.entry(key.clone()).or_default();
            tally.lines += 1;
            tally.numbers += line.numbers.len() as u64;
            for flag in &line.flags {
                *tally.flags.entry(flag.clone()).or_default() += 1;
            }
            if tally.example.is_empty() {
                tally.example = line.text.clone();
            }
            if seen.insert(key) {
                tally.items += 1;
            }
            if ranged(line) {
                ranged_templates.insert(line.template.clone());
            }
        }
        // one count per item and kind of problem: the index is detail
        let kinds: BTreeSet<(String, String)> = item
            .unread
            .iter()
            .map(|u| {
                let part = serde_json::to_string(&u.part).unwrap_or_default();
                let problem = u.problem.replace(|c: char| c.is_ascii_digit(), "");
                (part, problem)
            })
            .collect();
        for kind in kinds {
            *unread.entry(kind).or_default() += 1;
        }
    }

    let rows: Vec<_> = rows
        .into_iter()
        .map(|((source, template), t)| {
            json!({
                "source": source,
                "ranged": ranged_templates.contains(&template),
                "template": template,
                "items": t.items,
                "lines": t.lines,
                "numbers": t.numbers,
                "flags": t.flags,
                "example": t.example,
            })
        })
        .collect();
    let unread: Vec<_> = unread
        .into_iter()
        .map(|((part, problem), items)| json!({"part": part, "problem": problem, "items": items}))
        .collect();
    let shapes: Vec<_> = shapes
        .into_iter()
        .map(|((array, shape), n)| json!({"array": array, "shape": shape, "properties": n}))
        .collect();
    let out = json!({
        "items": items,
        "properties": properties,
        "displayed": displayed,
        "derive_seconds": deriving.as_secs_f64(),
        "rows": rows,
        "property_shapes": shapes,
        "unread": unread,
    });
    let mut stdout = io::stdout().lock();
    serde_json::to_writer(&mut stdout, &out)?;
    stdout.write_all(b"\n")?;
    Ok(())
}
