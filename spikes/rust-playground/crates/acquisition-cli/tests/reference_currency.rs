//! Process-level pin of `acq reference currency` (C68: reference data is
//! enumerable through every surface and cited by version wherever used;
//! CLI-GUIDE §8: `--json` is the contract, a failure is `{"error": …}` on
//! stdout with exit 1). No store, no daemon: the socket points into an
//! empty temp dir and `ACQ_NO_SPAWN=1` makes any contact an error, so
//! the command must answer from the binary alone.

use std::path::Path;
use std::process::{Command, Output};

use serde_json::Value;

fn acq(base: &Path, args: &[&str]) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_acq"));
    cmd.args(args)
        .env("ACQ_STORE_DIR", base)
        .env("ACQ_SOCKET", base.join("no.sock"))
        .env("ACQ_NO_KEYRING", "1")
        .env("ACQ_NO_SPAWN", "1");
    for var in ["ACQ_GGG", "ACQ_ACCOUNT", "ACQ_JOURNAL"] {
        cmd.env_remove(var);
    }
    cmd.output().expect("spawning acq")
}

fn sole_json(out: &Output) -> Value {
    let stdout = String::from_utf8(out.stdout.clone()).expect("stdout is UTF-8");
    serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("stdout is not exactly one JSON document ({e}):\n{stdout}"))
}

#[test]
fn c68_the_table_is_enumerable_by_version_with_no_store_and_no_daemon() {
    let tmp = tempdir();
    let out = acq(&tmp, &["--json", "reference", "currency"]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v = sole_json(&out);
    assert_eq!(v["version"], 1);
    let rows = v["currency"].as_array().unwrap();
    assert_eq!(rows.len(), 42);
    assert!(
        rows.iter()
            .all(|r| !r["evidence"].as_array().unwrap().is_empty())
    );
    assert_eq!(
        rows.iter().filter(|r| r.get("retired").is_some()).count(),
        3
    );

    let text = acq(&tmp, &["reference", "currency"]);
    assert!(text.status.success());
    let stdout = String::from_utf8(text.stdout).unwrap();
    assert!(stdout.starts_with("currency table v1 ("), "{stdout}");
    assert!(
        stdout.contains("39 currencies the game writes, 3 retired"),
        "{stdout}"
    );
    assert!(
        stdout
            .lines()
            .any(|l| l.contains("exalted") && l.contains("also exa"))
    );
    assert!(
        stdout
            .lines()
            .any(|l| l.contains("chisel") && l.contains("retired 2026-09-04"))
    );
    assert!(
        !stdout.contains("game:2026"),
        "evidence is the audit view: {stdout}"
    );

    let audit = acq(&tmp, &["reference", "currency", "--expand"]);
    let stdout = String::from_utf8(audit.stdout).unwrap();
    assert!(
        stdout.contains("game:2026-09-04 note `~price 999 chaos`"),
        "{stdout}"
    );
    assert!(stdout.contains("sources:"), "{stdout}");
}

#[test]
fn c68_one_word_resolves_exactly_or_fails_naming_the_version() {
    let tmp = tempdir();
    let out = acq(&tmp, &["--json", "reference", "currency", "exa"]);
    assert!(out.status.success());
    let v = sole_json(&out);
    assert_eq!(v["word"], "exa");
    assert_eq!(v["version"], 1);
    assert_eq!(v["currency"]["tag"], "exalted");
    assert_eq!(v["currency"]["emit"], "exalted");

    let text = acq(&tmp, &["reference", "currency", "exa"]);
    let stdout = String::from_utf8(text.stdout).unwrap();
    assert!(
        stdout.starts_with("exa is a legacy alias of exalted: currency table v1"),
        "{stdout}"
    );

    let miss = acq(&tmp, &["--json", "reference", "currency", "Chaos"]);
    assert_eq!(miss.status.code(), Some(1));
    let v = sole_json(&miss);
    let err = v["error"].as_str().unwrap();
    assert!(
        err.contains("\"Chaos\"") && err.contains("v1") && err.contains("acq reference currency"),
        "{err}"
    );

    let miss = acq(&tmp, &["reference", "currency", "Chaos"]);
    assert_eq!(miss.status.code(), Some(1));
    assert!(miss.stdout.is_empty());
    assert!(String::from_utf8_lossy(&miss.stderr).contains("not a currency word"));
}

fn tempdir() -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "acq-reference-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}
