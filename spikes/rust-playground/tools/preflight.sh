#!/bin/bash
# preflight.sh — the live drivers' shared preflight (LIVE-TESTING.md,
# "Build before you run"; brainstorming-notes/16 §6.3). Sourced, not run:
#
#   . "$here/tools/preflight.sh"
#   preflight            # after MODE, RUN_DIR and SOCK are chosen, before
#                        # any acq binary runs
#
# Needs `here` (the workspace root), `ACQ` (the binary path), `ACQD` (the
# daemon beside it, which the driver starts and owns), `MODE`
# (live|mock), `RUN_DIR` (exists) and `SOCK`; leaves `head`, `tip`, `ver`
# set and `$RUN_DIR/provenance.json` written. Neither binary carries a
# commit (C84: the identity is the shared-contract revision both are
# compiled against and, for `acqd`, the hash of its file), so a stale
# binary is prevented rather than detected, and the pairing of what ran
# with what was checked out is recorded where the evidence is — both
# executables take part in a run (acq compiles and submits the plan,
# acqd sends), so provenance hashes both, separately named, and after the
# run `provenance_matches_journal` holds the journal's `daemon` to the
# acqd hash specifically (owner ruling 2026-09-09, packet §10 item 7).

preflight() {
    # 1. No binary this run starts may spawn or replace a daemon: the only
    #    daemons here are the ones the driver starts, with the rails it
    #    says, and with no daemon up nothing can appear (C10, C3).
    export ACQ_NO_SPAWN=1

    # 2. The ledger cites a tip; the tip must identify the rung that ran —
    #    the driver, the verifier, the control documents, the crates and
    #    the lock — not only the binary. Live refuses working-tree changes
    #    there; mock notes them and continues (a rehearsal of a change is
    #    the point).
    head=$(git -C "$here" rev-parse HEAD)
    tip=${head:0:12}
    dirty=$(git -C "$here" status --porcelain -- tools LIVE-TESTING.md RUN-LEDGER.md CONTEXT.md decisions crates Cargo.toml Cargo.lock)
    if [ -n "$dirty" ]; then
        echo "working tree differs from $tip in the rung's own files:" >&2
        echo "$dirty" >&2
        if [ "$MODE" = live ]; then
            echo "refusing: commit first — the ledger's tip must name what ran" >&2
            exit 2
        fi
        echo "(mock mode: continuing anyway)"
    fi

    # 3. Refuse any daemon on the socket, this revision's or another's
    #    (`daemon status` reports both without touching either), before
    #    the build: never rebuild under a live daemon.
    if [ -x "$ACQ" ]; then preflight_refuse_daemon "before the build"; fi

    # 4. Build, locked: the build must not rewrite the lock the check in
    #    step 2 just read. Cheap when fresh.
    (cd "$here" && cargo build --workspace --locked --quiet) || { echo "refusing: cargo build failed" >&2; exit 2; }
    ver=$("$ACQ" --version)
    [ -x "$ACQD" ] || { echo "refusing: no daemon at $ACQD beside $ACQ (C82) — cargo build --workspace" >&2; exit 2; }

    # 5. Again, with the binary that will run.
    preflight_refuse_daemon "after the build"

    # 6. What ran, beside what was checked out: the run record maps the
    #    journal's contract revision and daemon hash to HEAD. `acq version
    #    --json` names the sibling acqd it found (path, length, mtime);
    #    the hash of that file is taken here, and the journal header's
    #    `daemon` must equal it (checked after the run). Written before
    #    any wire phase; a bundle that checksums its evidence includes
    #    this file.
    "$ACQ" version --json | jq \
        --arg head "$head" \
        --arg tree "$([ -z "$dirty" ] && echo clean || echo dirty)" \
        --arg dirty "$dirty" \
        --arg mode "$MODE" \
        --arg exe "$ACQ" \
        --arg exe_sha256 "$(shasum -a 256 "$ACQ" | cut -d' ' -f1)" \
        --arg acqd "$ACQD" \
        --arg acqd_sha256 "$(shasum -a 256 "$ACQD" | cut -d' ' -f1)" \
        --arg rustc "$(rustc -Vv)" \
        --arg cargo "$(cargo -V)" \
        --arg at "$(date -u +%FT%TZ)" \
        '. + {head: $head, tree: $tree, dirty_files: ($dirty | split("\n") | map(select(. != ""))),
              mode: $mode, exe: $exe, exe_sha256: $exe_sha256, acqd: $acqd, acqd_sha256: $acqd_sha256,
              rustc: $rustc, cargo: $cargo, recorded_at: $at}' \
        >"$RUN_DIR/provenance.json"
}

# provenance_matches_journal <journal file>: every daemon lifetime in the
# journal — each `open` line — must carry the contract revision and the
# acqd hash provenance.json recorded before the run (C84). A lifetime that
# names another daemon is the rung-8 mistake made visible: some other
# acqd sent. Exits 2 on any mismatch, naming the line.
provenance_matches_journal() {
    local journal=$1 want_contract want_daemon bad
    want_contract=$(jq -r '.contract' "$RUN_DIR/provenance.json")
    want_daemon=$(jq -r '.acqd_sha256' "$RUN_DIR/provenance.json")
    bad=$(jq -r --arg c "$want_contract" --arg d "$want_daemon" \
        'select(.event == "open") | select(.contract != $c or .daemon != $d)
         | "pid \(.pid): contract \(.contract) daemon \(.daemon)"' "$journal")
    if [ -n "$bad" ]; then
        echo "*** the journal names a daemon other than the one provenance.json recorded" >&2
        echo "*** (contract $want_contract, acqd sha256 $want_daemon):" >&2
        echo "$bad" >&2
        exit 2
    fi
    echo "provenance: every lifetime in the journal is contract $want_contract, acqd sha256 ${want_daemon:0:12}… (provenance.json)"
}

preflight_refuse_daemon() { # <when>
    local pid
    pid=$("$ACQ" daemon status --json 2>/dev/null | jq -r '.pid // empty')
    if [ -n "$pid" ]; then
        echo "refusing: a daemon (pid $pid) is running on $SOCK $1 — acq daemon stop first" >&2
        exit 2
    fi
}
