# agent-seat — brief for the run (2026-09-14)

**Runner: a fresh Fable session in Claude Code, started from a terminal
by the owner — not a subagent.** The seat is the point: the runner is
the agent, drives the surfaces as they are, and records where they fail
it. The kickoff prompt is one line: *"Run `search/agent-seat/BRIEF.md`."*
This brief is committed before the run and deleted in the track's
closing commit, cited by hash. The owner approved the use of a read-only
copy of his real store (2026-09-13).

## Rules that bind this run

- Read, in order: `search/README.md` (rules in force, traps), this
  track's `README.md`, `CLI-REFERENCE.md` and `MCP-REFERENCE.md` (the
  standing rule before running `acq` or driving `acq-mcp`), the
  mock-session skill (`.claude/skills/mock-session/SKILL.md`),
  `decisions/frontends.md` (C53: the decision view, the audit view, the
  JSON contract), and `search/owner-seat/data/questions.md`. Nothing
  else until the seat has been sat in — the point is the experience,
  not the research.
- Work only inside this track's directory, plus the closing edits every
  full session makes: the index row in `search/README.md`, the
  session-close skill, the commit with the story. Never touch another
  track, `CONTEXT.md`, `decisions/`, or `SURFACES.md`.
- **Reads only.** No `auth`, `refresh`, `submit`, `policy set`,
  `price set`, `store import`, `store rebuild`; no `apply_plan`,
  `set_sync_policy`, `submit_job`. `ACQ_NO_SPAWN=1` guarantees no daemon
  starts; a tool or verb that wants one refuses — record the refusal
  text as a finding and move on.
- **In phase one, never open the store file with `sqlite3` or a
  script** (C48: raw SQL is not a surface). When the surfaces cannot
  answer and the file could, write down that you were tempted, and what
  you would have typed. That temptation is the finding. Phase two lifts
  this for read-only connections only, after phase one is on disk.
- The copy is the owner's account data: it lives under `raw/`
  (gitignored), is described in `MANIFEST.md`, and is never committed.
  Absolute paths; never `cd` in a compound command.
- Write the README last, from `data/`, in the shape the rules give
  (status line, at most five headline bullets, findings as tables or
  short `**F<n> — …**` paragraphs, numbers in one table, open questions
  naming the experiment, Provenance, an empty Review table; about 12 KB).

## Set up (verified by hand 2026-09-14, no daemon)

```sh
export ACQ_PROVIDER=mock ACQ_NO_KEYRING=1 ACQ_NO_SPAWN=1
unset ACQ_GGG ACQ_TRIPWIRE ACQ_MAX_SENDS
export ACQ_STORE_DIR=<abs>/search/agent-seat/raw/world     # gitignored (search/*/raw/)
export ACQ_LOG_DIR=<abs>/search/agent-seat/raw/logs
mkdir -p "$ACQ_STORE_DIR/mock" "$ACQ_LOG_DIR"
R="$HOME/Library/Application Support/gerwaric.acquisition-playground/store/ggg"
sqlite3 "$R/GERWARIC_7694.db" ".backup '$ACQ_STORE_DIR/mock/GERWARIC_7694.db'"        # .backup, never cp: WAL
sqlite3 "$R/cac319d8-e65f-4afa-92be-1de85d620033.annotations.db" \
        ".backup '$ACQ_STORE_DIR/mock/cac319d8-e65f-4afa-92be-1de85d620033.annotations.db'"
python3 -c 'import json,os,sys; r=sys.argv[1]; a=json.load(open(r+"/accounts.json")); a["accounts"]=[x for x in a["accounts"] if x["username"].startswith("GERWARIC")]; json.dump(a,open(os.environ["ACQ_STORE_DIR"]+"/mock/accounts.json","w"),indent=2)' "$R"
cargo build --workspace && alias acq=<abs>/target/debug/acq   # C82
acq store status --json      # 22,721 items, 3,444 tabs, 65 characters, 22,751 events
acq daemon status            # "daemon is not running; no queue on disk" — and it stays so
```

Record each copy in `MANIFEST.md` (source path, date, `.backup`,
sha256). The MCP server: before starting the session, from the same
terminal, register it with the same environment —
`claude mcp add --scope local acq-seat -e ACQ_PROVIDER=mock -e ACQ_NO_KEYRING=1 -e ACQ_NO_SPAWN=1 -e ACQ_STORE_DIR=… -e ACQ_LOG_DIR=… -- <abs>/target/debug/acq-mcp`
— and remove it at the close. Its tools are `MCP-REFERENCE.md`; the
store tools answer with no daemon (checked over stdio 2026-09-14:
`initialize`, then `search_items` returned rows with whole bodies).

Three facts about this corpus, so the seat is not surprised: it is the
spike's store alone (22,721 items; the census's 36,139 included the C++
store), `items search ""` matches everything, and a search row carries
the whole GGG body — 50 rows of JSON are 149 KB, all rows are 63.5 MB.

## The protocol

1. **Orientation from zero.** Before any query, using only the MCP
   instructions and tool list and `acq --help`: write down what an agent
   can learn about this corpus and its vocabulary — which fields exist,
   which values, which mods — without pulling rows. If the answer is
   "nothing", that is F1.
2. **Twelve questions.** The owner's seven, verbatim from
   `owner-seat/data/questions.md`, and five of the agent's own: facets
   before rows (how many items per tab, league, rarity); refining the
   previous answer without restating it; explaining why an item matched
   or did not; finding one item again by id; and one whole-corpus mod
   query with a value (`+# to maximum Life` at 90 or more), which is
   where a query that pulls everything into context fails. For each,
   try to answer with the surfaces as they are — CLI and MCP both, since
   the two seats differ — and log every call: surface, verb or tool,
   arguments, rows, bytes returned, wall time, and whether it went into
   the session's context or to a file.
3. **Where it failed, and what you did instead.** The workaround is
   data: pulling 63 MB and filtering it outside the surfaces, or
   answering from memory, or giving up. Say which, and what a surface
   would have to offer for the question to cost one call.
4. **Write it down** (`data/`, then the README):
   - `data/questions.json` — the twelve: the question, the query as the
     agent *wanted* to write it (your own notation, not a model), the
     expected result shape, what happened, the calls it took, the
     bytes it cost.
   - `data/calls.csv` — the log from step 2, one row per call.
   - The requirements list — one line each with the failure it
     prevents — beside owner-seat's R-lines, numbered on from R9, and a
     note where the agent's need and the owner's are the same need.
   - Findings: schema discovery, facets and counts, result shape and
     the body, refinement, explain, stable ids, errors, what the MCP
     instructions and tool descriptions got right and wrong (C53).
5. **Phase two — read-only SQL, after phase one is written down.** Only
   once `data/questions.json` and `data/calls.csv` hold phase one — an
   agent that has seen a lines table stops noticing what the tools lack.
   Then re-answer the same twelve questions with SQL, on both seats where
   they differ, over two files, and log every call the same way (a
   `phase` column in `calls.csv`; a second block per question in
   `questions.json`):
   - the store copy itself, `$ACQ_STORE_DIR/mock/GERWARIC_7694.db` — the
     internal schema, the item as a JSON body;
   - engine-bench's seat projection at real scale,
     `<abs>/search/engine-bench/raw/seat-projection.db` (written by its
     crate's `--seat` mode from the same two stores; ruled facts only —
     read its `items` and `lines` DDL first, the comments are part of it).
   Read-only is enforced, never requested: `sqlite3 -readonly`, or a
   `file:…?mode=ro` URI plus `PRAGMA query_only=1` from a script; never
   the annotations file. Log the DDL read as a call with its bytes;
   count the wrong-column and wrong-path mistakes; note every question
   the JSON body made hard (a `json_extract` over an array of objects, a
   number inside display text) and every one the projection could not
   answer at all — that list is the projection's requirements, and it
   goes in the README beside the R-lines. Say what SQL cost the agent,
   not only what it enabled.
6. **Close** as a full session does: the session-close skill, the index
   row (`first pass complete — <date>`, the README's bytes), this brief
   deleted, the commit with the story, `claude mcp remove acq-seat`.

## Acceptance

A reader can say, for each of the twelve questions, what it cost an
agent today in calls and bytes, where the surfaces stopped, and what
one change would have made it a single call — with the same numbers
for the CLI seat and the MCP seat where they differ; and then, under
read-only SQL, which questions became one call over which file, at
what cost in schema reading and mistakes, and which the projection
could not answer.
