#!/bin/bash
# preflight.sh — the live drivers' shared preflight (LIVE-TESTING.md,
# "Build before you run"; brainstorming-notes/16 §6.3). Sourced, not run:
#
#   . "$here/tools/preflight.sh"
#   preflight            # after MODE and RUN_DIR are chosen, before any
#                        # acq binary runs
#
# Needs `here` (the workspace root), `ACQ` (the binary path), `ACQD` (the
# daemon beside it, which the driver starts and owns), `MODE`
# (live|mock) and `RUN_DIR` (exists); leaves `head`, `tip`, `ver` set
# and `$RUN_DIR/provenance.json` written. The socket is derived from the
# world (C83) and never named here. Neither binary carries a
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

    # 3. Refuse any daemon on this world's socket, this revision's or
    #    another's (`daemon status` reports both without touching either,
    #    and a daemon from before the rendezvous on the fixed legacy
    #    socket too), before the build: never rebuild under a live
    #    daemon. The endpoints the drivers and the rung-11 helper once
    #    chose by hand under the retired `ACQ_SOCKET` are known here and
    #    nowhere else, so they are probed here (C83, the split's step 6;
    #    parked for removal with the legacy detection, decisions/daemon.md).
    preflight_refuse_legacy_endpoints
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

# provenance_matches_journal <journal file>: which daemon sent, held to
# provenance.json (C84). A lifetime is an `open` header and the sends
# after it up to the next header — the journal readers' model — so the
# journal must hold at least one header, every send must carry the pid
# of the latest header before it (a send under no header, or under
# another daemon's header, is a daemon that never said who it was —
# review 2026-09-10: a check that read the headers alone passed a journal
# with none), and every header must carry the contract revision and the
# acqd hash recorded before the run (a header naming another daemon is
# the rung-8 mistake made visible). A pid the OS reuses for a successor
# is a second lifetime, not a fault (review 2026-09-11). Exits 2 on any
# problem, naming each; `tools/preflight-breakers.sh` stages every
# refusal.
provenance_matches_journal() {
    local journal=$1 want_contract want_daemon bad
    want_contract=$(jq -r '.contract' "$RUN_DIR/provenance.json")
    want_daemon=$(jq -r '.acqd_sha256' "$RUN_DIR/provenance.json")
    [ -n "$want_contract" ] && [ "$want_contract" != null ] && [ -n "$want_daemon" ] && [ "$want_daemon" != null ] \
        || { echo "*** provenance.json names no contract or acqd hash" >&2; exit 2; }
    bad=$(jq -rn --arg c "$want_contract" --arg d "$want_daemon" '
        reduce inputs as $l ({ headers: 0, current: null, problems: [] };
          if $l.event == "open" then
              .headers += 1 | .current = $l.pid
              | if $l.contract != $c or $l.daemon != $d
                then .problems += ["pid \($l.pid): header names contract \($l.contract) daemon \($l.daemon)"] else . end
            elif $l.method != null then
              if .current == null
              then .problems += ["pid \($l.pid): \($l.method) \($l.route) with no open header before it"]
              elif .current != $l.pid
              then .problems += ["pid \($l.pid): \($l.method) \($l.route) under pid \(.current)\u0027s header, not its own"]
              else . end
            else . end)
        | if .headers == 0 then .problems += ["no open header in the journal at all"] else . end
        | .problems[]' "$journal") || { echo "*** could not read $journal" >&2; exit 2; }
    if [ -n "$bad" ]; then
        echo "*** the journal does not name the daemon provenance.json recorded" >&2
        echo "*** (contract $want_contract, acqd sha256 $want_daemon):" >&2
        echo "$bad" >&2
        exit 2
    fi
    echo "provenance: every lifetime in the journal is contract $want_contract, acqd sha256 ${want_daemon:0:12}…, and every send has its header (provenance.json)"
}

preflight_refuse_daemon() { # <when>
    local pid socket
    pid=$("$ACQ" daemon status --json 2>/dev/null | jq -r '.pid // empty')
    if [ -n "$pid" ]; then
        socket=$("$ACQ" daemon status --json 2>/dev/null | jq -r '.socket // "its socket"')
        echo "refusing: a daemon (pid $pid) is running on $socket $1 — acq daemon stop first" >&2
        exit 2
    fi
}

# The sockets a daemon of this playground listened on before the
# rendezvous derived from the world: the fixed default, and the values
# `ACQ_SOCKET` was set to by the drivers and the rung-11 helper. A daemon
# answering on any of them predates the split's step 6 and would be
# invisible to this run's binaries while it sent; refuse until it is
# stopped by hand. A stale socket file nothing answers on is ignored. The
# probe itself failing — no python3, an import error, a timeout, any
# error but "nothing there" — refuses too: a check that cannot run must
# not pass (review 2026-09-11; `tools/preflight-breakers.sh` stages each).
preflight_refuse_legacy_endpoints() {
    local t=${TMPDIR:-/tmp}; t=${t%/}
    local sock rc
    for sock in "$t/acquisition-playground.sock" /tmp/acquisition-playground.sock \
        /tmp/acq-tracer.sock /tmp/acq-persist.sock /tmp/acq-r11-A.sock /tmp/acq-r11-B.sock; do
        [ -S "$sock" ] || continue
        rc=0; preflight_probe_socket "$sock" || rc=$?
        case $rc in
        10)
            echo "refusing: a daemon from before the rendezvous is listening on $sock (an ACQ_SOCKET endpoint of history; the knob is gone)" >&2
            echo "  stop it first: \`acq daemon stop\` reaches the fixed default socket; for the others, \`lsof -U | grep $(basename "$sock")\` names the pid to kill" >&2
            exit 2
            ;;
        11) ;;
        *)
            echo "refusing: could not probe $sock (the probe exited $rc) — a socket file stands there and this check cannot tell whether a daemon answers; fix the probe (python3) or remove the file by hand" >&2
            exit 2
            ;;
        esac
    done
}

# Exit 10: something accepts on the Unix socket at $1. Exit 11: nothing
# does (the file is gone, or nothing listens — ECONNREFUSED). Anything
# else — 0 and 1 included, which a python3 that did not run this script
# would exit with, and 127 for none at all — is the probe failing to
# tell (an import failure, a timeout, another error), which the caller
# must refuse on. The two answers are codes no generic failure produces.
preflight_probe_socket() { # <socket path>
    python3 - "$1" <<'PROBE'
import socket
import sys

s = socket.socket(socket.AF_UNIX)
s.settimeout(2)
try:
    s.connect(sys.argv[1])
except (FileNotFoundError, ConnectionRefusedError):
    sys.exit(11)
except OSError as e:
    print(f"probe: {e}", file=sys.stderr)
    sys.exit(3)
sys.exit(10)
PROBE
}
