//! Verification of the clause registry, per the accepted
//! `clause-registry-design.md` §4.
//!
//! Split into focused tests so each mutation-check failure names its
//! broken rule directly. The six documented mutations map: remove a
//! citation → `coverage_and_citation_counts_are_consistent` (or the
//! open-set test, if the coverage state was edited too); nonexistent
//! fn / misspelled file → `every_cited_test_fn_exists_in_source`;
//! citation on an Excluded clause →
//! `coverage_and_citation_counts_are_consistent`; OPEN_UNTESTED edit
//! without the entry → `open_untested_matches_the_computed_set`;
//! duplicate ID → `ids_are_unique_and_owners_are_known`.

use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::path::Path;

use rate_limit_core::obligations::{CLAUSES, Coverage, OPEN_UNTESTED};

/// The literal an open `Untested` clause's note must carry
/// (design §D4 rule 1).
const OPEN_DISPOSITION: &str = "open — flagged for Tom";
/// The prefix a decided (accepted-not-fixed) disposition must carry,
/// followed by its register citation.
const ACCEPTED_DISPOSITION: &str = "accepted —";

/// The full owner vocabulary: M1–M13, C1–C5, X1–X2, G1–G6, B1–B14,
/// U1–U5, O1–O8. Generated, not listed, so a typo cannot silently
/// shrink it. (U grew to 5 at the 2026-08-13 ballot pass: the U5
/// headroom-instrumentation exclusion.)
fn known_owners() -> BTreeSet<String> {
    let series: [(&str, u32); 7] = [
        ("M", 13),
        ("C", 5),
        ("X", 2),
        ("G", 6),
        ("B", 14),
        ("U", 5),
        ("O", 8),
    ];
    let mut owners: BTreeSet<String> = series
        .iter()
        .flat_map(|(prefix, count)| (1..=*count).map(move |n| format!("{prefix}{n}")))
        .collect();
    // SHELL: the external-review shell-obligation owner, a deliberate
    // one-off series adopted by Tom 2026-08-13 (design §7 item 2).
    owners.insert("SHELL".to_string());
    owners
}

#[test]
fn ids_are_unique_and_owners_are_known() {
    let owners = known_owners();
    let mut seen = BTreeSet::new();
    for clause in CLAUSES {
        assert!(seen.insert(clause.id), "duplicate clause id: {}", clause.id);
        assert!(
            owners.contains(clause.owner),
            "clause {} has unknown owner {:?}",
            clause.id,
            clause.owner
        );
    }
    assert!(!CLAUSES.is_empty(), "the registry must not be empty");
}

#[test]
fn every_owner_series_member_appears() {
    let used: BTreeSet<&str> = CLAUSES.iter().map(|clause| clause.owner).collect();
    for owner in known_owners() {
        assert!(
            used.contains(owner.as_str()),
            "owner {owner} has no registry entry — a series row was dropped"
        );
    }
}

#[test]
fn coverage_and_citation_counts_are_consistent() {
    for clause in CLAUSES {
        assert!(
            !clause.text.is_empty(),
            "clause {} has empty text",
            clause.id
        );
        match clause.coverage {
            Coverage::Full | Coverage::Partial => {
                assert!(
                    !clause.citations.is_empty(),
                    "clause {} claims {:?} coverage with zero citations",
                    clause.id,
                    clause.coverage
                );
            }
            Coverage::Untested | Coverage::Excluded => {
                assert!(
                    clause.citations.is_empty(),
                    "clause {} is {:?} but cites tests — either the coverage \
                     state is stale or an exclusion is drifting",
                    clause.id,
                    clause.coverage
                );
            }
        }
        if matches!(clause.coverage, Coverage::Partial) {
            assert!(
                !clause.note.is_empty(),
                "Partial clause {} must state its missing delta in the note",
                clause.id
            );
        }
        if matches!(clause.coverage, Coverage::Excluded) {
            assert!(
                clause.owner.starts_with('U') || clause.owner.starts_with('O'),
                "clause {} is Excluded but owned by {} — only U-/O-series \
                 rows are exclusions",
                clause.id,
                clause.owner
            );
        }
    }
}

#[test]
fn every_cited_test_fn_exists_in_source() {
    // Cited files are read from the crate root and searched by
    // content for `fn <name>(` — this finds `#[cfg(test)]` modules
    // inside `src/` (which a `tests/` grep misses — the round-four
    // trap) and proptest-macro properties (whose `fn` is indented
    // inside the macro, which a `^fn ` scan misses).
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut sources: HashMap<&str, String> = HashMap::new();
    for clause in CLAUSES {
        for citation in clause.citations {
            let source = sources.entry(citation.file).or_insert_with(|| {
                fs::read_to_string(root.join(citation.file)).unwrap_or_else(|error| {
                    panic!(
                        "clause {} cites unreadable file {}: {error}",
                        clause.id, citation.file
                    )
                })
            });
            let needle = format!("fn {}(", citation.test_fn);
            assert!(
                source.contains(&needle),
                "clause {} cites {} :: {} but no such fn exists",
                clause.id,
                citation.file,
                citation.test_fn
            );
            assert!(
                !citation.must_assert.is_empty(),
                "clause {} citation {} has an empty must_assert",
                clause.id,
                citation.test_fn
            );
        }
    }
}

#[test]
fn untested_notes_carry_a_disposition() {
    for clause in CLAUSES {
        if matches!(clause.coverage, Coverage::Untested) {
            assert!(
                clause.note.contains(OPEN_DISPOSITION)
                    || clause.note.contains(ACCEPTED_DISPOSITION),
                "Untested clause {} names no disposition — silent untested \
                 state is unrepresentable (add {OPEN_DISPOSITION:?} or an \
                 {ACCEPTED_DISPOSITION:?} register citation)",
                clause.id
            );
        }
    }
}

#[test]
fn open_untested_matches_the_computed_set() {
    // The swept_phases pattern: changing a coverage state must touch
    // both the entry and this list, so every transition is a
    // deliberate two-line diff.
    let mut computed: Vec<&str> = CLAUSES
        .iter()
        .filter(|clause| {
            matches!(clause.coverage, Coverage::Untested) && clause.note.contains(OPEN_DISPOSITION)
        })
        .map(|clause| clause.id)
        .collect();
    computed.sort_unstable();

    let mut declared: Vec<&str> = OPEN_UNTESTED.to_vec();
    let before = declared.len();
    declared.sort_unstable();
    declared.dedup();
    assert_eq!(
        before,
        declared.len(),
        "OPEN_UNTESTED contains duplicate ids"
    );

    assert_eq!(
        declared, computed,
        "OPEN_UNTESTED and the registry's open Untested clauses disagree \
         (left: declared list, right: computed from CLAUSES)"
    );
}
