//! Reference data's CLI surface: `acq reference currency` enumerates the
//! currency table the binary ships (C68: reference data is enumerable
//! through every surface and cited by version wherever used) and resolves
//! one word. Reads nothing but the binary — no store, no daemon, no
//! network — so it answers in every mode, logged in or not.
//!
//! Under C53: the default text is the table (a reference lookup is the
//! enumeration; the list *is* the answer, so the ten-or-fewer grouping
//! rule does not apply), one row per line with the words a parser
//! accepts and a retired mark; `--expand` adds each row's evidence and
//! the table's sources; `--json` is the table itself, whole, evidence
//! included. A word that does not resolve is a failure that names the
//! word and the version and says what to type next.

use acquisition_plan::currency::{Currency, CurrencyTable, table};
use anyhow::{Context, Result, bail};
use serde_json::json;

/// `acq reference currency [WORD] [--expand]`.
pub fn currency(word: Option<&str>, expand: bool, json: bool) -> Result<()> {
    let t = table().context("the currency table this build ships is not usable")?;
    match word {
        Some(word) => resolve(t, word, json),
        None => enumerate(t, expand, json),
    }
}

fn enumerate(t: &CurrencyTable, expand: bool, json: bool) -> Result<()> {
    if json {
        println!("{}", serde_json::to_string_pretty(t)?);
        return Ok(());
    }
    let active = t.active().count();
    let retired = t.rows().len() - active;
    let aliased = t.rows().iter().filter(|r| !r.aliases.is_empty()).count();
    println!(
        "currency table v{} ({}): {active} currencies the game writes, {retired} retired, {aliased} with a legacy alias",
        t.version(),
        t.status()
    );
    for row in t.rows() {
        print_row(row, expand);
    }
    if expand {
        println!("sources:");
        for s in t.sources() {
            println!("  {s}");
        }
    }
    println!("a price cites the word at left; `acq reference currency <word>` resolves one");
    Ok(())
}

fn resolve(t: &CurrencyTable, word: &str, json: bool) -> Result<()> {
    let Some(row) = t.resolve(word) else {
        bail!(
            "{word:?} is not a currency word in currency table v{} (words are exact and case-sensitive; `acq reference currency` lists them)",
            t.version()
        );
    };
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "word": word,
                "version": t.version(),
                "currency": row,
            }))?
        );
        return Ok(());
    }
    let via = if word == row.tag {
        "the tag".to_string()
    } else if word == row.emit {
        "the game's word".to_string()
    } else {
        format!("a legacy alias of {}", row.tag)
    };
    println!("{word} is {via}: currency table v{}", t.version());
    print_row(row, true);
    Ok(())
}

fn print_row(row: &Currency, expand: bool) {
    let mut notes = Vec::new();
    if row.emit != row.tag {
        notes.push(format!("game writes {}", row.emit));
    }
    if !row.aliases.is_empty() {
        notes.push(format!("also {}", row.aliases.join(", ")));
    }
    if let Some(when) = &row.retired {
        notes.push(format!("retired {when}"));
    }
    let line = format!("  {:<22} {:<28} {}", row.tag, row.display, notes.join("; "));
    println!("{}", line.trim_end());
    if expand {
        for e in &row.evidence {
            println!("{:>26} {e}", "");
        }
    }
}
