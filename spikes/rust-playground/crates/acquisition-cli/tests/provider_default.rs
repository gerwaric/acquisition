//! C88 through the binaries: the real provider is the default and the
//! mock is opted into; the retired knob and an unknown provider word are
//! refused at start by `acq` and by `acqd`, naming the remedy; and a
//! real-mode daemon is started by a human — a use verb in real mode
//! without a terminal on stderr (every process here has its streams
//! piped, as an agent's shell, cron or ssh without a tty would) reports
//! absence and the remedy instead of spawning, while the mock spawns
//! freely from the same place. Nothing reaches GGG: the real-mode
//! commands here never get a daemon, by the rule under test, and the
//! world's root is checked to prove no spawn was attempted.

use std::process::Command;

mod harness;

use harness::{acq, acqd, command, isolate, scratch, sole_json, text};

#[test]
fn c88_the_real_provider_is_the_default_and_the_retired_knob_is_refused() {
    let scratch = scratch("c88-knob");
    let base = &scratch.0;

    // Unset: ggg. The harness sets the mock; unset it to see the default.
    let out = command(base, &["version", "--json"])
        .env_remove("ACQ_PROVIDER")
        .output()
        .unwrap();
    assert!(out.status.success(), "{}", text(&out));
    assert_eq!(sole_json(&out)["provider"], "ggg");
    // Opted in: mock.
    let out = acq(base, &["version", "--json"]);
    assert_eq!(sole_json(&out)["provider"], "mock");

    // The retired knob, whatever its value, and an unknown word: refused
    // before any verb runs, in prose and as the total JSON error (C11).
    for (env, said) in [
        (("ACQ_GGG", "1"), "ACQ_GGG is no longer read"),
        (("ACQ_GGG", "0"), "ACQ_GGG is no longer read"),
        (("ACQ_PROVIDER", "Mock"), "names no provider"),
    ] {
        let out = command(base, &["version"])
            .env(env.0, env.1)
            .output()
            .unwrap();
        assert!(!out.status.success(), "{}", text(&out));
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(
            stderr.contains(said) && stderr.contains("C88"),
            "{env:?}: {stderr}"
        );
        let out = command(base, &["version", "--json"])
            .env(env.0, env.1)
            .output()
            .unwrap();
        assert!(!out.status.success(), "{}", text(&out));
        let error = sole_json(&out)["error"].as_str().unwrap().to_string();
        assert!(error.contains(said), "{env:?}: {error}");
    }

    // The daemon refuses the same way, on its own stderr, before it makes
    // a world: no root, no log.
    let mut daemon = Command::new(acqd());
    isolate(&mut daemon, base);
    let out = daemon.env("ACQ_GGG", "1").output().unwrap();
    assert_eq!(out.status.code(), Some(2), "{}", text(&out));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.starts_with("acqd: ACQ_GGG is no longer read"),
        "{stderr}"
    );
    assert!(
        !base.join("store").exists(),
        "the refused daemon made a world"
    );
}

#[test]
fn c88_a_real_mode_daemon_is_started_from_a_terminal_and_the_mock_from_anywhere() {
    let scratch = scratch("c88-tty");
    let base = &scratch.0;

    // Real mode, no terminal on stderr (piped here): the use door observes
    // absence and names the remedy; nothing is spawned — the world's root
    // is never created (a spawn creates it first, C83).
    let out = command(base, &["profile"])
        .env("ACQ_PROVIDER", "ggg")
        .output()
        .unwrap();
    assert!(!out.status.success(), "{}", text(&out));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("started from a terminal")
            && stderr.contains("C88")
            && stderr.contains("ACQ_PROVIDER=mock"),
        "{stderr}"
    );
    assert!(
        !base.join("store").exists(),
        "a real-mode use verb without a terminal created the world's root — a spawn"
    );
    let out = command(base, &["profile", "--json"])
        .env("ACQ_PROVIDER", "ggg")
        .output()
        .unwrap();
    assert!(!out.status.success(), "{}", text(&out));
    assert!(
        sole_json(&out)["error"]
            .as_str()
            .unwrap()
            .contains("started from a terminal"),
        "{}",
        text(&out)
    );

    // The mock, from the same non-terminal: spawned freely.
    let out = acq(base, &["submit", "sleep", "--detach"]);
    assert!(out.status.success(), "{}", text(&out));
    let status = sole_json(&acq(base, &["daemon", "status", "--json"]));
    assert_eq!(status["running"], true, "{status}");
    assert_eq!(status["provider"], "mock", "{status}");
    let out = acq(base, &["daemon", "stop", "--json"]);
    assert!(out.status.success(), "{}", text(&out));
}
