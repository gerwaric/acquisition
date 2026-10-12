# Brief: C++ → Rust contamination reading (layer 2)

## Question

The owner intends to license the Rust workspace at
`spikes/rust-playground` under MIT OR Apache-2.0. The C++ app at the
repository root is GPL-3.0 and 64 of its files carry a copyright header
naming someone other than the owner (mostly Ilya Zhuravlev; list in
`scope.txt` beside this brief). Copyright protects *expression*, not
facts or ideas. For your assigned pairs, decide whether any Rust code is
derived from the C++ *expression*, as opposed to sharing GGG facts or
reimplementing the same idea.

## What is already known (do not redo)

`overlap-report.txt` beside this brief is a lexical scan over all 64
files against the whole Rust tree: 0 shared six-word comment shingles;
the only shared string literals are game vocabulary (mod text, currency
names, stash kinds, forum markup); shared identifiers are English words
and game terms. So there is no verbatim copying to find. Your job is the
structural question the scan cannot answer: the same non-obvious
procedure reproduced step for step, the same arbitrary constants,
the same table rows in the same selection and order, the same
decomposition where a fresh author would likely have chosen another.

`cpp-commits.txt` lists the 58 playground commits whose messages
mention C++ or porting. Read the ones that touch your pairs (`git show
<hash> --stat` and the message; never check anything out).

## Rules that bind you

- Read-only. Never edit, create or delete any file under the repository.
  Never commit, push, fetch, stash, checkout or spawn agents.
- Write only inside your own output directory named in your prompt.
- Read `docs/design/network-ground-truth.md` headings first (`grep -n
  '^#'`): anything recorded there as a GGG fact is FACT by definition.
- Do not read `CONTEXT.md`, `decisions/`, or other readers' directories.
- Large files are read by section (`grep -n` then `sed -n` by range),
  never whole.

## Procedure per pair

1. Read the C++ side in full (they are small).
2. Find the Rust counterpart by concept: grep for the game terms, tags,
   formats and field names the C++ handles, then read those regions.
3. For every resemblance you find, write one row in `findings.tsv`:
   `class  confidence  cpp_path:line  rust_path:line  what_is_shared  why_this_class`
   where class is FACT (GGG/game/forum data or API shape), IDEA (same
   algorithm, design or decomposition, independently expressed),
   EXPRESSION (copied or closely paraphrased code, comments, strings,
   arbitrary constants, or a non-obvious structure reproduced step for
   step), and confidence is high/medium/low. A pair with no resemblance
   at all gets one row with class NONE.
4. Any data table in Rust whose rows correspond to a C++ table gets a
   row in `tables.tsv`: `cpp_table  rust_table  cpp_rows  rust_rows  shared_rows  order_preserved  notes`.
   Count by reading, never estimate.
5. Write `README.md` last, from the two TSVs: a headline block of at
   most six lines stating the verdict per pair, then per-pair detail.
   Guide: under 12 KB. Every claim points at a file:line.

## Acceptance

Every pair in your prompt has at least one row in `findings.tsv`, and
every EXPRESSION or IDEA row with high confidence quotes or paraphrases
both sides in `README.md` so a reviewer can judge without reopening the
files.

## Report (your final message, at most 300 words)

The verdict per pair, what you left out, and the one cut you would most
want reversed. The reviewer restores from that line, never from your
transcript.
