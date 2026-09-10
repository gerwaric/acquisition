#!/bin/bash
# preflight.sh — the live drivers' shared preflight (LIVE-TESTING.md,
# "Build before you run"; brainstorming-notes/16 §6.3). Sourced, not run:
#
#   . "$here/tools/preflight.sh"
#   preflight            # after MODE, RUN_DIR and SOCK are chosen, before
#                        # any acq binary runs
#
# Needs `here` (the workspace root), `ACQ` (the binary path), `MODE`
# (live|mock), `RUN_DIR` (exists) and `SOCK`; leaves `head`, `tip`, `ver`
# set and `$RUN_DIR/provenance.json` written. The binary carries no
# commit (C10: its identity is the runtime revision of its sources), so a
# stale binary is prevented rather than detected, and the pairing of what
# ran with what was checked out is recorded where the evidence is.

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
    (cd "$here" && cargo build --locked --quiet) || { echo "refusing: cargo build failed" >&2; exit 2; }
    ver=$("$ACQ" --version)

    # 5. Again, with the binary that will run.
    preflight_refuse_daemon "after the build"

    # 6. What ran, beside what was checked out: the run record maps the
    #    journal's runtime revision to HEAD. Written before any wire phase;
    #    a bundle that checksums its evidence includes this file.
    "$ACQ" version --json | jq \
        --arg head "$head" \
        --arg tree "$([ -z "$dirty" ] && echo clean || echo dirty)" \
        --arg dirty "$dirty" \
        --arg mode "$MODE" \
        --arg exe "$ACQ" \
        --arg exe_sha256 "$(shasum -a 256 "$ACQ" | cut -d' ' -f1)" \
        --arg rustc "$(rustc -Vv)" \
        --arg cargo "$(cargo -V)" \
        --arg at "$(date -u +%FT%TZ)" \
        '. + {head: $head, tree: $tree, dirty_files: ($dirty | split("\n") | map(select(. != ""))),
              mode: $mode, exe: $exe, exe_sha256: $exe_sha256, rustc: $rustc, cargo: $cargo, recorded_at: $at}' \
        >"$RUN_DIR/provenance.json"
}

preflight_refuse_daemon() { # <when>
    local pid
    pid=$("$ACQ" daemon status --json 2>/dev/null | jq -r '.pid // empty')
    if [ -n "$pid" ]; then
        echo "refusing: a daemon (pid $pid) is running on $SOCK $1 — acq daemon stop first" >&2
        exit 2
    fi
}
