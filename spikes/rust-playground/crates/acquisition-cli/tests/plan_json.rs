//! Process-level pin of the `--json` stdout contract for the policy/plan
//! surface: the CLI is itself an API (CONTEXT.md), step 7's apply will
//! consume `refresh --plan --json`, and only a spawned binary can prove
//! what actually lands on stdout — the in-process tests cannot see it.
//!
//! Facts are seeded through the store crate directly (the same crate the
//! daemon writes through), so no daemon runs: the socket points into an
//! empty temp dir and `ACQ_NO_SPAWN=1` makes any spawn attempt an error
//! rather than a stray process.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use acquisition_plan::RefreshPlan;
use acquisition_store::{Endpoint, Index, Store, account_path};
use serde_json::{Value, json};

const USER: &str = "Alice#1234";
const UUID: &str = "u-e2e";

fn acq(base: &Path, args: &[&str]) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_acq"));
    cmd.args(args)
        .env("ACQ_STORE_DIR", base)
        .env("ACQ_SOCKET", base.join("no.sock"))
        .env("ACQ_NO_KEYRING", "1")
        .env("ACQ_NO_SPAWN", "1");
    // The parent shell may carry live-run or selection state; the contract
    // under test is the isolated mock one.
    for var in [
        "ACQ_GGG",
        "ACQ_ACCOUNT",
        "ACQ_TRIPWIRE",
        "ACQ_MAX_SENDS",
        "ACQ_JOURNAL",
        "ACQ_IDLE_SHUTDOWN",
    ] {
        cmd.env_remove(var);
    }
    cmd.output().expect("spawning acq")
}

/// stdout must be exactly one JSON document (plus whitespace) — that is
/// the whole point of `--json`: pipeable, no banners, no trailing chatter.
fn sole_json(out: &Output) -> Value {
    let stdout = String::from_utf8(out.stdout.clone()).expect("stdout is UTF-8");
    serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("stdout is not exactly one JSON document ({e}):\n{stdout}"))
}

fn seed_store(base: &Path) -> PathBuf {
    let mock = base.join("mock");
    std::fs::create_dir_all(&mock).unwrap();
    // Seed at the current clock: the plan below compiles at real time, and
    // a fresh listing keeps the expected action set to exactly one
    // never-fetched tab (an ancient listing would add a re-list).
    let at = acquisition_store::now();
    let mut index = Index::load(&mock).unwrap();
    index.record_login(USER, UUID, false, at).unwrap();
    let mut store = Store::open(&account_path(&mock, USER)).unwrap();
    store
        .record(
            &Endpoint::Profile,
            &json!({}),
            200,
            &json!({ "uuid": UUID, "name": USER }),
            at,
        )
        .unwrap();
    store
        .record(
            &Endpoint::Stashes {
                league: "Standard".into(),
            },
            &json!({ "league": "Standard" }),
            200,
            &json!({ "stashes": [
                { "id": "t1", "name": "One", "type": "PremiumStash", "index": 0 },
            ] }),
            at,
        )
        .unwrap();
    mock
}

#[test]
fn json_stdout_is_the_contract_surface() {
    let base = std::env::temp_dir().join(format!("acq-e2e-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    seed_store(&base);

    // No policy yet: `policy show --json` answers null, exit 0 — absent
    // intent is data, not an error.
    let out = acq(&base, &["policy", "show", "--json"]);
    assert!(out.status.success(), "{out:?}");
    assert_eq!(sole_json(&out), Value::Null);

    // `--json` is total: a refusal is {"error": …} on stdout with exit 1,
    // not prose on stderr.
    let out = acq(&base, &["refresh", "--plan", "--json"]);
    assert_eq!(out.status.code(), Some(1), "{out:?}");
    let err = sole_json(&out);
    assert!(
        err["error"].as_str().unwrap().contains("no sync policy"),
        "{err}"
    );

    // A typo'd policy is refused as {"error": …} naming the field, and
    // stores nothing (the show below still answers null).
    let out = acq(
        &base,
        &[
            "policy",
            "set",
            r#"{"version":1,"leagues":{"Standard":{"tabs":"all","max_age_secs":60}}}"#,
            "--json",
        ],
    );
    assert_eq!(out.status.code(), Some(1), "{out:?}");
    assert!(
        sole_json(&out)["error"]
            .as_str()
            .unwrap()
            .contains("max_age_secs")
    );
    let out = acq(&base, &["policy", "show", "--json"]);
    assert_eq!(sole_json(&out), Value::Null);

    // A valid write answers the stored row; show answers the same row.
    let policy = r#"{"version":1,"leagues":{"Standard":{"tabs":"all","max_age_seconds":3600}}}"#;
    let out = acq(&base, &["policy", "set", policy, "--json"]);
    assert!(out.status.success(), "{out:?}");
    let row = sole_json(&out);
    assert_eq!(row["revision"], json!(1));
    assert_eq!(row["value"], serde_json::from_str::<Value>(policy).unwrap());
    let out = acq(&base, &["policy", "show", "--json"]);
    assert_eq!(sole_json(&out), row);

    // A conflicting --if-revision is {"error": …} naming the current
    // revision, exit 1.
    let out = acq(
        &base,
        &["policy", "set", policy, "--if-revision", "7", "--json"],
    );
    assert_eq!(out.status.code(), Some(1), "{out:?}");
    assert!(
        sole_json(&out)["error"]
            .as_str()
            .unwrap()
            .contains("revision 1")
    );

    // The plan: stdout is exactly one JSON document that the planner's own
    // validating parse accepts — schema stamp, recomputed derived
    // quantities and all — so what a pipe receives is what apply can
    // trust. No daemon means no quote, and the reason goes to stderr,
    // never into the envelope.
    let out = acq(&base, &["refresh", "--plan", "--json"]);
    assert!(out.status.success(), "{out:?}");
    let envelope = sole_json(&out);
    let plan = RefreshPlan::from_value(&envelope)
        .unwrap_or_else(|e| panic!("stdout is not a valid plan envelope ({e}):\n{envelope:#}"));
    assert_eq!(plan.league, "Standard");
    assert_eq!(plan.account_uuid, UUID);
    assert_eq!(plan.account_name.as_deref(), Some(USER));
    assert_eq!(plan.provider, "mock");
    assert_eq!(plan.basis.policy_revision, 1);
    // The seeded facts: one listed, never-fetched tab (the listing itself
    // is fresh at plan time, so the fetch is the single action).
    assert_eq!(plan.logical_requests, 1, "{:?}", plan.actions);
    assert_eq!(plan.quote, None);
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(stderr.contains("no quote"), "{stderr}");

    let _ = std::fs::remove_dir_all(&base);
}
