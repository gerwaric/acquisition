//! The build a basis names (C98; B1, `search/LEDGER.md`) as the build
//! script makes it, asked of trees of this test's own: which files the
//! name is a digest of, and which it never is. The routine is the one
//! the contract revision is made by (C84), so what holds here holds of
//! both.

#[allow(dead_code)]
#[path = "../build.rs"]
mod build;

use std::path::{Path, PathBuf};

fn tree(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("acq-build-name-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("src/deep")).unwrap();
    std::fs::write(root.join("Cargo.lock"), "lock").unwrap();
    std::fs::write(root.join("src/a.rs"), "fn a() {}").unwrap();
    std::fs::write(root.join("src/deep/t.toml"), "x = 1").unwrap();
    root
}

fn name(root: &Path) -> String {
    build::digest_under(root, b"test/1", &["Cargo.lock", "src"])
}

/// The name moves on any edit to a file of a kind a build reads, and on
/// one added; a file an operating system or an editor writes unasked —
/// a dotfile, at any depth — is no part of it.
#[test]
fn the_name_moves_on_a_source_and_never_on_a_dotfile() {
    let root = tree("dotfile");
    let first = name(&root);
    assert_eq!(first.len(), 12);
    assert_eq!(name(&root), first);
    for stray in [
        "src/.DS_Store",
        "src/deep/.DS_Store",
        "src/.a.rs.swp",
        "src/.#a.rs",
    ] {
        std::fs::write(root.join(stray), "anything").unwrap();
        assert_eq!(name(&root), first, "{stray}");
    }
    std::fs::write(root.join("src/a.rs"), "fn a() { }").unwrap();
    let edited = name(&root);
    assert_ne!(edited, first);
    std::fs::write(root.join("src/deep/b.json"), "{}").unwrap();
    let added = name(&root);
    assert_ne!(added, edited);
    std::fs::write(root.join("Cargo.lock"), "lock 2").unwrap();
    assert_ne!(name(&root), added);
    let _ = std::fs::remove_dir_all(&root);
}

/// A file of a kind the digest does not know stops the build and is
/// named: a leftover is never hashed into the name in silence, and a new
/// kind of source is never left out of it in silence.
#[test]
fn a_file_of_a_kind_the_digest_does_not_know_stops_the_build() {
    for stray in [
        "src/a.rs.orig",
        "src/deep/table.csv",
        "src/README",
        "src/a.rs~",
    ] {
        let root = tree("unknown");
        std::fs::write(root.join(stray), "anything").unwrap();
        let stopped = std::panic::catch_unwind(|| name(&root)).unwrap_err();
        let said = stopped
            .downcast_ref::<String>()
            .cloned()
            .unwrap_or_default();
        assert!(said.contains(stray), "{stray}: {said}");
        assert!(said.contains("add the kind or remove the file"), "{said}");
        let _ = std::fs::remove_dir_all(&root);
    }
}

/// What the name covers: this crate, the planner, the store, and the
/// protocol crate the planner resolves a price through; the root
/// manifest and the lock. Every input is there, and the name the build
/// carries is this routine's over them.
#[test]
fn the_name_covers_every_crate_an_answer_is_computed_by() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for input in [
        "Cargo.toml",
        "Cargo.lock",
        "crates/acquisition-search/Cargo.toml",
        "crates/acquisition-search/src",
        "crates/acquisition-search/reference",
        "crates/acquisition-plan/Cargo.toml",
        "crates/acquisition-plan/src",
        "crates/acquisition-plan/reference",
        "crates/acquisition-store/Cargo.toml",
        "crates/acquisition-store/src",
        "crates/acquisition-protocol/Cargo.toml",
        "crates/acquisition-protocol/src",
    ] {
        assert!(build::INPUTS.contains(&input), "{input} is no input");
        assert!(root.join(input).exists(), "{input}");
    }
    assert_eq!(build::INPUTS.len(), 12);
    assert_eq!(
        build::digest_under(&root.canonicalize().unwrap(), build::DOMAIN, build::INPUTS),
        env!("ACQ_SEARCH_BUILD")
    );
}
