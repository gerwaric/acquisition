# Relicensing audit, 2026-10-11

Question: before declaring the Rust workspace MIT OR Apache-2.0, does any
of it derive from the *expression* in a C++ file whose copyright header
names someone other than the owner? Facts and ideas are not copyright;
code, comments, strings and table rows are.

Audited: commit 6ce6500f, branch spikes/rust-playground, Rust files under
`spikes/rust-playground/` — every `file:line` in the tables carries that
prefix; after the root swap the prefix is stale and nothing else is.
Scope: the 64 C++ files in `scope.txt` (SPDX header not "Tom Holz";
`git blame` confirms no Tom-headed file carries others' lines in bulk).

Files
- `BRIEF.md` — the readers' brief: rules, procedure, classes, acceptance.
- `overlap.py`, `overlap-report.txt` — lexical scan and its output as run.
- `scope.txt` — the 64 files. `cpp-commits.txt` — the 58 spike commits
  whose message mentions C++ or porting.
- `findings-merged.tsv` (54 rows: class, confidence, cpp:line, rust:line,
  what, why), `tables-merged.tsv` (15 rows: table-to-table counts).
- `reader-pricing/`, `reader-store/`, `reader-search/` — each reader's
  README and its two TSVs.

Command as run, from the repository root at 6ce6500f:
    python3 -I audits/relicense-2026-10/overlap.py "$PWD"
The script reads `<root>/src` (C++) and `<root>/spikes/rust-playground`
(Rust). After the split the C++ side is `acquisition-qt` at 6ce6500f and
the Rust side is this repository; a rerun needs the two-path form of the
script, written when first needed.

Result: clean at HEAD. 0 shared six-word comment shingles (22,631 C++ /
189,181 Rust); 20 shared literals ≥8 chars after the fact filter, all game
vocabulary; 54 reader findings — 14 NONE, 18 FACT, 19 IDEA, 3 EXPRESSION
(two low: spoiler-title leading space, `[items]` token; one medium, in
history). The medium one: 51a33751 (2026-09-03) drafted
`reference/currency-v1.toml` from `src/currency.cpp` — 51 of 52 alias
strings (`comm -12`), C++ enum order, display names, source cited in the
draft's header lines 11–33; replaced by 10ef2374 (2026-09-04). Owner's
ruling 2026-10-11: "accepted" — history left as committed, under GPL.
Verified by hand in the audit session: `git show 51a33751:spikes/rust-
playground/crates/acquisition-plan/reference/currency-v1.toml`.

Readers were three general-purpose Opus subagents, read-only, one per
area; the reviewing session restored each one's "cut to reverse"
(eval.rs/group.rs vs Search::Matches: 0 shared vocabulary; totals-v6.toml:
33 mod strings trace via pseudomods.cpp to Ilya's 2014 list, itself
poe.trade's — FACT).
