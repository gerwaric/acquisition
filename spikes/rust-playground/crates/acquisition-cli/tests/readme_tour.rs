//! The README's tour is a hand-written copy of the CLI's shape, and a
//! copy is honest only while something checks it (P5: structure over
//! discipline). Every `acq …` line in the README's fenced block must
//! resolve under clap: the verb path, and every `--flag` the line names
//! on that subcommand (or a global). A verb's semantics is its `--help`;
//! the tour shows the shape, and this test keeps the shape true — a
//! renamed verb or a dropped flag fails here, which the identifier scan
//! in `tools/docs-check.sh` cannot see.
//!
//! Driven through the binary (`acq <path> --help` must exit 0 and its
//! text must name each flag) so the crate needs no library target.
//!
//! The tour's grammar, as this test reads it: a line starting `acq` is a
//! verb line, `#` opens a comment, ` | ` separates alternatives — an
//! alternative that starts with a flag adds flags to the same path, one
//! that starts with a word replaces the previous path's last verb — and
//! a token starting with `<`, `[`, `'`, `@` or `-` ends the verb path.

use std::process::Command;

fn help(path: &[&str]) -> Result<String, String> {
    let out = Command::new(env!("CARGO_BIN_EXE_acq"))
        .args(path)
        .arg("--help")
        .env("ACQ_NO_SPAWN", "1")
        .output()
        .expect("running acq --help");
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).into_owned())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).into_owned())
    }
}

fn tour_lines() -> Vec<String> {
    let readme = concat!(env!("CARGO_MANIFEST_DIR"), "/../../README.md");
    let text = std::fs::read_to_string(readme).unwrap_or_else(|e| panic!("{readme}: {e}"));
    let mut inside = false;
    let mut lines = Vec::new();
    for line in text.lines() {
        if line.starts_with("```sh") {
            inside = true;
            continue;
        }
        if inside && line.starts_with("```") {
            break;
        }
        if inside {
            lines.push(line.to_string());
        }
    }
    assert!(!lines.is_empty(), "no ```sh block in {readme}");
    lines
}

fn is_verb(tok: &str) -> bool {
    !tok.is_empty()
        && tok
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        && !tok.starts_with('-')
}

fn flags_in(text: &str) -> Vec<String> {
    let mut flags = Vec::new();
    for tok in text.split_whitespace() {
        let tok = tok.trim_start_matches('[');
        if let Some(rest) = tok.strip_prefix("--") {
            let name: String = rest
                .chars()
                .take_while(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == '-')
                .collect();
            if !name.is_empty() {
                flags.push(name);
            }
        }
    }
    flags
}

/// `(verb path, flags)` for each alternative of one tour line.
fn parse(line: &str) -> Vec<(Vec<String>, Vec<String>)> {
    let code = line.split(" #").next().unwrap_or("").trim();
    let mut out = Vec::new();
    let mut path: Vec<String> = Vec::new();
    for (i, alt) in code.split(" | ").enumerate() {
        let mut toks = alt.split_whitespace().peekable();
        if i == 0 {
            assert_eq!(toks.next(), Some("acq"), "not a verb line: {line}");
            path.clear();
        } else if toks.peek().is_some_and(|t| is_verb(t)) {
            path.pop();
        }
        while let Some(t) = toks.peek() {
            if is_verb(t) {
                path.push(t.to_string());
                toks.next();
            } else {
                break;
            }
        }
        out.push((path.clone(), flags_in(alt)));
    }
    out
}

#[test]
fn every_tour_line_resolves_under_clap() {
    let mut checked = 0;
    let mut failures = Vec::new();
    for line in tour_lines() {
        if !line.starts_with("acq ") {
            continue;
        }
        for (path, flags) in parse(&line) {
            let path_ref: Vec<&str> = path.iter().map(String::as_str).collect();
            let text = match help(&path_ref) {
                Ok(t) => t,
                Err(e) => {
                    failures.push(format!("`acq {}`: no such verb\n{e}", path.join(" ")));
                    continue;
                }
            };
            for flag in flags {
                let needle = format!("--{flag}");
                let named = text.match_indices(&needle).any(|(i, _)| {
                    text[i + needle.len()..]
                        .chars()
                        .next()
                        .is_none_or(|c| !(c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'))
                });
                if !named {
                    failures.push(format!(
                        "`acq {}`: `{needle}` is not in its --help\n    {line}",
                        path.join(" ")
                    ));
                }
            }
            checked += 1;
        }
    }
    assert!(
        checked >= 20,
        "the tour has only {checked} verb lines — was the block renamed?"
    );
    assert!(
        failures.is_empty(),
        "the README's tour no longer matches the CLI (a verb's semantics is its --help; the tour shows the shape):\n{}",
        failures.join("\n")
    );
}

#[test]
fn the_grammar_reads_alternatives() {
    let parsed = parse("acq jobs [--watch] | status <id> | result <id>   # x");
    assert_eq!(parsed[0], (vec!["jobs".into()], vec!["watch".into()]));
    assert_eq!(parsed[1], (vec!["status".into()], vec![]));
    assert_eq!(parsed[2], (vec!["result".into()], vec![]));
    let parsed = parse("acq auth status | check | logout");
    assert_eq!(parsed[2].0, vec!["auth".to_string(), "logout".to_string()]);
    let parsed = parse("acq refresh --tabs a,b,c | --all [--deep]");
    assert_eq!(
        parsed[1],
        (vec!["refresh".into()], vec!["all".into(), "deep".into()])
    );
    let parsed = parse("acq policy set '<json>'|-|@FILE [--if-revision N]");
    assert_eq!(
        parsed[0],
        (
            vec!["policy".into(), "set".into()],
            vec!["if-revision".into()]
        )
    );
    let parsed = parse("acq refresh --apply[=plan.json] [--max-requests N]");
    assert_eq!(
        parsed[0].1,
        vec!["apply".to_string(), "max-requests".to_string()]
    );
    let parsed = parse("acq <verb> --help");
    assert_eq!(parsed[0], (vec![], vec!["help".into()]));
}
