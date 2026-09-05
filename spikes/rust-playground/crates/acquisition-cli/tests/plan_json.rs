//! Process-level pin of the `--json` stdout contract for the policy/plan
//! surface (C44 end to end: a moved policy revision refuses apply with
//! the remedy named; C38: a tampered envelope never reaches the daemon;
//! C41: plan and no-op apply run with no daemon at all — `ACQ_NO_SPAWN`
//! makes any contact an error): the CLI is itself an API (CONTEXT.md),
//! step 7's apply will consume `refresh --plan --json`, and only a spawned
//! binary can prove what actually lands on stdout — the in-process tests
//! cannot see it.
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
                realm: "pc".into(),
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
    // C65: the row says which channel wrote it; no actor was claimed.
    assert_eq!(row["written_via"], json!("cli"));
    assert!(row.get("actor").is_none(), "{row}");
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

    // ---- the apply gates that run before any daemon contact ----
    // (ACQ_NO_SPAWN=1 stands for the whole test, so any path that reached
    // the daemon would fail with its distinct "forbids starting" error —
    // the assertions below prove each gate fires first.)

    // The step-7 staleness ruling, end to end: the reviewed envelope cites
    // revision 1; a policy write moves intent to revision 2; applying the
    // stale plan is a structured refusal naming both revisions.
    let stale_plan = base.join("stale-plan.json");
    std::fs::write(&stale_plan, serde_json::to_vec(&envelope).unwrap()).unwrap();
    let out = acq(&base, &["policy", "set", policy, "--json"]);
    assert!(out.status.success(), "{out:?}");
    assert_eq!(sole_json(&out)["revision"], json!(2));
    let apply_arg = format!("--apply={}", stale_plan.display());
    let out = acq(&base, &["refresh", &apply_arg, "--json"]);
    assert_eq!(out.status.code(), Some(1), "{out:?}");
    let err = sole_json(&out);
    let msg = err["error"].as_str().unwrap();
    assert!(
        msg.contains("revision 1") && msg.contains("revision 2") && msg.contains("replan"),
        "{msg}"
    );

    // A plan at the current revision passes the gate — and then needs the
    // daemon, which this harness forbids: the refusal is the connect
    // policy's, proving apply spends only through a daemon.
    let out = acq(&base, &["refresh", "--apply", "--json"]);
    assert_eq!(out.status.code(), Some(1), "{out:?}");
    let msg = sole_json(&out);
    let msg = msg["error"].as_str().unwrap();
    assert!(msg.contains("ACQ_NO_SPAWN"), "{msg}");

    // A tampered envelope never reaches the daemon either: the planner's
    // validating parse refuses it whole (forged counts are exactly what
    // admission budgeting must not trust).
    let mut forged: Value = envelope.clone();
    forged["logical_requests"] = json!(0);
    let forged_path = base.join("forged-plan.json");
    std::fs::write(&forged_path, serde_json::to_vec(&forged).unwrap()).unwrap();
    let apply_arg = format!("--apply={}", forged_path.display());
    let out = acq(&base, &["refresh", &apply_arg, "--json"]);
    assert_eq!(out.status.code(), Some(1), "{out:?}");
    let err = sole_json(&out);
    assert!(
        err["error"].as_str().unwrap().contains("malformed"),
        "{err}"
    );

    // An empty plan applies as a no-op without any daemon at all: fetch
    // the one tab through the store crate (as the daemon would record it),
    // and the compiled plan has nothing to do.
    let mock = base.join("mock");
    let mut store = Store::open(&account_path(&mock, USER)).unwrap();
    store
        .record(
            &Endpoint::Stash {
                realm: "pc".into(),
                league: "Standard".into(),
                id: "t1".into(),
                sub: None,
            },
            &json!({ "league": "Standard", "id": "t1", "deep": false }),
            200,
            &json!({ "stash": { "id": "t1", "name": "One", "type": "PremiumStash", "items": [] } }),
            acquisition_store::now(),
        )
        .unwrap();
    drop(store);
    let out = acq(&base, &["refresh", "--apply", "--json"]);
    assert!(out.status.success(), "{out:?}");
    let done = sole_json(&out);
    assert_eq!(done["requests"], json!(0));
    assert!(
        done["note"].as_str().unwrap().contains("nothing to do"),
        "{done}"
    );

    let _ = std::fs::remove_dir_all(&base);
}
