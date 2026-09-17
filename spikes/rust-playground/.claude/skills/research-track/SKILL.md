---
name: research-track
description: Run one research track of a slice's working directory (search/ is the first) as a subagent under a committed brief, then review and commit it with its story. Use for any track whose work is a read or a measurement with a fixed input; never for a seat, which a session sits in.
---

# Research track

A track is a directory with one README (`search/README.md`, rules in
force). A read or a measurement with a fixed input runs as an Opus
subagent under a brief; the reviewer — the session — checks the result
and commits it with the story. Ran five times on 2026-09-13/14 and
repeated two traps, so it is a skill (P6).

## Before the run

1. Write `<track>/BRIEF.md`: the rules that bind the runner (reading
   order; work only inside the track; never the index, another track,
   `CONTEXT.md`, `decisions/`, `SURFACES.md`; never commit, push,
   fetch, or spawn), the inputs by path, the outputs by file and
   script, the findings expected, an acceptance sentence, and the
   report asked for (at most 300 words), which names what the runner
   left out and the one cut it would most want reversed — the reviewer
   restores from that line, never from the transcript (the digest run,
   2026-09-16: every restore came from it). The README already holds
   the question; the brief holds only the procedure.
2. Commit the brief before the run, so any session — or Codex — can
   run it.
3. Launch one general-purpose subagent per track with a prompt that
   names the brief's path and repeats the prohibitions; tracks in
   parallel only when their directories are disjoint.

## After the run

4. Read the track's README and its `data/` heads yourself; check the
   acceptance sentence by hand where it names a command (the seat
   projection's fractured-mod query was run at the commit); every
   claim points at a file. A finding you disagree with is a Review row,
   not a silent edit.
5. `git rm <track>/BRIEF.md`; set the index row's status and byte
   count (the runner cannot — it is forbidden the index, and every
   report says so); commit with the story, citing the brief's commit.

## Traps

- **`git add search` while another subagent is still writing sweeps
  its half-written files into your commit** (2026-09-13, re-cut with a
  soft reset). Add the track's directory and the index, never the
  parent.
- The runner leaves the index row stale by design; forgetting it is the
  reviewer's error, not the runner's.
- A brief that lists an input the owner has not supplied (a worked
  example, a sibling clone) yields open questions, not a stalled run:
  say in the brief that it may be absent.
- `timeout` is not on macOS; a stdio smoke test is driven from Python.
