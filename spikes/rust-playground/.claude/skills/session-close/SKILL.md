---
name: session-close
description: End a working session on the Rust playground — route what was learned to its one home, keep the always-loaded documents at budget, commit with the story in the message. Use before the last commit of any session, and whenever a slice closes.
---

# Session close

The always-loaded documents accrete when a session ends by writing its
story into `CONTEXT.md`. This procedure gives the story a home instead.
The rule is one authoritative home per fact (`AGENTS.md`, "Routing");
the ladder behind it is `brainstorming-notes/09-settling-the-record.md`.

## 1. Gate

```sh
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
git diff --check
tools/docs-check.sh
```

Run each command bare and read its exit status. A check piped through
`grep` or `tail` reports the filter's exit, not the check's: on
2026-09-03 an over-length registry entry was committed that way.

## 2. Route each thing learned

For every item the session produced, exactly one of:

| It is | It goes to |
| --- | --- |
| a ruling by the owner, or a boundary property | the registry, in `decisions/<area>.md` (or `CONTEXT.md` only if every area must know it): the next `C<n>` (never reuse an id), the ruling verbatim, *Why:* in a sentence, *Details:*/*Pinned:*/*Evidence:* pointers — one bullet under the length limit; the mechanism it implies goes to the code's doc comment under its id |
| a property now pinned by a test | the test's name carries the id (`c44_stale_revision_refused`), or a comment cites it; the entry's *Pinned:* names the file |
| a review finding | a row in the slice's closed record (`REFRESH-SLICE.md` is the shape), with the fix commit |
| the story of what was built and why | the commit message |
| a live run | one run-ledger row in `LIVE-TESTING.md`; evidence in `runs/` |
| a fact about GGG | a numbered ground-truth claim, authored master-side and cherry-picked here |
| how a mechanism works | a doc comment on the code |
| an observation with no ruling yet | the slice record's "Observations still open" while the slice is open; otherwise the area's `decisions/<area>.md`, "Parked", **with a trigger** (`CONTEXT.md` only if it crosses every area) |
| a procedure run twice that repeated a trap | a skill file under `.claude/skills/`, referenced by path from `AGENTS.md` |
| the owner's verdict on a reading | recorded verbatim from the conversation, marked as such |

Do not write "built on <date>", "step N done", or a list of what a test
covers into `CONTEXT.md`: git holds the first two, the test the third.
`tools/docs-check.sh` reports decisions nothing cites; when you touch the
code behind one, name it in a test or a doc comment so the report shrinks.

## 3. When a slice closes

Cut its `CONTEXT.md` section to rulings, properties and pointers; cite
the last full-text commit in the section; give the slice a closed
record in the mold of `NETWORK-CLEANUP.md` (step ledger, findings table,
what the runs taught, observations still open). Strike-through items in
a "Parked" list are deleted, not kept; a park whose trigger fired goes with them.

## 4. Commit

One commit per concern. The message carries the narrative — what
changed, what the review found, what the run showed — because that is
where the next session will read it (`git log`, the slice's range).
