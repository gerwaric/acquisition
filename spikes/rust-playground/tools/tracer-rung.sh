#!/bin/bash
# Tracer rung (LIVE-TESTING.md, "Tracer rung — policy → plan → apply →
# replan"): the owner's first real use of the refresh slice, driven end to
# end with the rails on, verified from the journal, and with the friction
# notes collected as the run goes rather than recalled afterwards. The
# human parts stay human: run this from a terminal (the daemon needs your
# keychain, and the login needs your browser), and every wire phase waits
# for an explicit enter.
#
#   tools/tracer-rung.sh [--account SEL] [--league L] [--max-age S] \
#                        [--cycles K] <tab1,...,tabN | all>       # live
#   tools/tracer-rung.sh --mock [--cycles K] <tab1,...|all>        # rehearsal
#
# The selection becomes the sync policy (`acq policy set`): the named tab
# ids, or `all` for every tab the league lists. The planner matches ids
# exactly, and a substash's id is its own — so an id list never covers the
# substashes a map/unique fetch discovers (they show up in `acq tabs`,
# uncovered, and the loop closes after one working cycle); `all` covers
# them and runs the discovery cycle. Live, `all` on a 322-tab account is a
# 323-request plan with ~343 s limiter holds every 30 sends (rung 10's
# shape) and then every substash — the owner's call, not this script's.
#
# Shape of the run (one fresh daemon per wire phase, each under an EXACT
# send ceiling, stopped when its phase is over):
#   0  preflight (no wire): provenance, leftover env, no daemon, account
#   1  login, only if the account's index entry has no uuid (intent binds
#      to the uuid; a login predating uuid-at-login has none) — 2 sends
#   2  policy written, then `refresh --plan` with NO daemon: compiled
#      offline, the note on stderr says so
#   3+ per cycle: the offline envelope derives the ceiling (1 token POST +
#      one HEAD probe per route the plan touches + one GET per action);
#      a daemon comes up under it; `refresh --plan --json` again, now with
#      the daemon's quote (or its note says why not — the `/profile`
#      discriminator residual is collected here, not pre-fixed); that
#      envelope is checked to be the offline one plus the quote (actions
#      in order, basis, identity, revision), its actions are shown and
#      confirmed, and THAT FILE is applied as one `apply` parent with
#      `--max-requests` = its own count. The ceiling is exact, so the
#      daemon halts on the bound right after the last planned send — the
#      expected end of a cycle; a send the plan did not project would
#      consume the bound and show as a planned child refused. Repeat until
#      the plan is empty (the loop closed) or --cycles is hit. An empty
#      plan's `--apply` runs with no daemon at all: the no-op must contact
#      nothing, and the socket is checked dead after it.
#   4  the next read = facts: `acq tabs`, `acq store status/events`; a
#      selected tab missing from the store, or a read that fails, fails
#      the run — the readback is part of what the rung tests
#   5  evidence + verification: per lifetime, probe before first send per
#      route, no non-2xx; every probe's reported hits are at most this
#      run's own earlier sends on that route (so the run's FIRST probe on
#      a route must report 0 — the standing rule — and a later cycle's
#      probe may carry the earlier cycle's hits, which are ours); GET
#      count == the applied plan's logical count (zero 429 re-sends: the
#      estimate's minimum held); ledger row draft and the friction notes.
# Mock mode proves the script on the in-process provider with the same
# exact ceilings: isolated socket + store (fresh each run), a throwaway
# mock login in the mock's own keyring service (the session must survive
# the per-cycle daemon restarts; NOT ACQ_NO_KEYRING), logout at the end.
# One mock caveat: the mock provider dies with each daemon, so a later
# cycle's probe reads fresh counters there — a probe carrying the earlier
# cycle's hits is evidence only live.
# With a non-tty stdin (an agent session) the enter gates and note prompts
# are skipped; live mode refuses a non-tty.
set -euo pipefail

here=$(cd "$(dirname "$0")/.." && pwd)
ACQ="$here/target/debug/acq"

MODE=live
ACCOUNT=
LEAGUE=Standard
MAX_AGE=3600
CYCLES=4
while [ $# -gt 0 ]; do
    case "$1" in
    --mock) MODE=mock; shift ;;
    --account) ACCOUNT=${2:?--account needs a value}; shift 2 ;;
    --league) LEAGUE=${2:?--league needs a value}; shift 2 ;;
    --max-age) MAX_AGE=${2:?--max-age needs a value}; shift 2 ;;
    --cycles) CYCLES=${2:?--cycles needs a value}; shift 2 ;;
    *) break ;;
    esac
done
SELECTION=${1:?usage: tracer-rung.sh [--mock] [--account SEL] [--league L] [--max-age S] [--cycles K] <tab1,...|all>}
command -v jq >/dev/null || { echo "jq is required" >&2; exit 2; }
command -v python3 >/dev/null || { echo "python3 is required" >&2; exit 2; }
INTERACTIVE=0
if [ -t 0 ]; then INTERACTIVE=1; fi
if [ "$MODE" = live ] && [ "$INTERACTIVE" = 0 ]; then
    echo "refusing: stdin is not a terminal — the wire phases gate on enter." >&2
    echo "run this directly from a terminal, not via a captured or piped shell." >&2
    exit 2
fi

# ---- preflight (no wire) ---------------------------------------------------

# Isolation knobs left over from other work would silently redirect this
# run; the rails knobs are set per daemon below, so a leftover value there
# is dropped rather than refused.
for v in ACQ_SOCKET ACQ_STORE_DIR ACQ_NO_KEYRING ACQ_NO_SPAWN ACQ_JOURNAL; do
    if [ -n "${!v:-}" ]; then
        echo "refusing: $v is set in this shell (leftover from other work); unset it first" >&2
        exit 2
    fi
done
for v in ACQ_GGG ACQ_TRIPWIRE ACQ_MAX_SENDS ACQ_IDLE_SHUTDOWN; do
    if [ -n "${!v:-}" ]; then echo "note: $v was set in this shell; this script sets it per daemon"; fi
done
unset ACQ_GGG ACQ_TRIPWIRE ACQ_MAX_SENDS ACQ_IDLE_SHUTDOWN

tip=$(git -C "$here" rev-parse --short=12 HEAD)
ver=$("$ACQ" --version)
case "$ver" in
*"$tip"*-dirty* | *dirty*)
    echo "refusing: binary is dirty ($ver) — commit, cargo build, retry" >&2
    [ "$MODE" = mock ] && echo "(mock mode: continuing anyway)" || exit 2 ;;
*"$tip"*) ;;
*)
    echo "refusing: binary ($ver) does not match HEAD ($tip) — cargo build first" >&2
    exit 2 ;;
esac

RUN_DIR="$here/runs/$(date -u +%F)-tracer"
if [ "$MODE" = mock ]; then RUN_DIR="$RUN_DIR-mock"; fi
mkdir -p "$RUN_DIR"
FRICTION="$RUN_DIR/friction.md"
# A rehearsal starts from nothing every time: the mock store is this run's.
if [ "$MODE" = mock ]; then rm -rf "$RUN_DIR/store"; fi

if [ "$MODE" = live ]; then
    export ACQ_GGG=1
    T=${TMPDIR:-/tmp}; T=${T%/}
    SOCK="$T/acquisition-playground.sock"
    PROVIDER=ggg
else
    export ACQ_SOCKET=/tmp/acq-tracer.sock ACQ_STORE_DIR="$RUN_DIR/store"
    SOCK=$ACQ_SOCKET
    PROVIDER=mock
fi
# The only daemons in this run are the ones this script starts, with the
# rails it says; a client must never lazy-spawn (or replace) one. The
# offline claims lean on this too: with no daemon up, nothing can appear.
export ACQ_NO_SPAWN=1
JOURNAL="${SOCK%.sock}.$PROVIDER.sends.jsonl"
LOG="${SOCK%.sock}.log"

status_json() { "$ACQ" daemon status --json 2>/dev/null || echo '{}'; }
daemon_up() { [ -S "$SOCK" ] && [ "$(status_json | jq -r '.pid // empty')" != "" ]; }
journal_size() { if [ -f "$JOURNAL" ]; then wc -c <"$JOURNAL" | tr -d ' '; else echo 0; fi; }
# Sends journaled since a byte offset (event lines excluded).
sends_since() { tail -c +$(($1 + 1)) "$JOURNAL" 2>/dev/null | grep -c '"method"' || true; }

if daemon_up; then
    echo "refusing: a daemon is already running on $SOCK — acq daemon stop first" >&2
    exit 2
fi

COMPLETED=0
CLIENT_PID=
cleanup() {
    if [ -n "$CLIENT_PID" ]; then kill "$CLIENT_PID" 2>/dev/null || true; fi
    if [ "$COMPLETED" != 1 ]; then
        echo ""
        if [ -z "${PID:-}" ]; then
            echo "*** aborted before any daemon started; nothing was sent."
        elif daemon_up; then
            echo "*** aborted mid-run with a daemon still up (possibly halted). Jobs are on disk:"
            echo "*** acq daemon status / acq jobs; cancel what should not resume; acq daemon stop."
            echo "*** Partial evidence is in $RUN_DIR and $JOURNAL"
        else
            echo "*** aborted mid-run; no daemon is up. Partial evidence is in $RUN_DIR and $JOURNAL"
        fi
        if [ -s "$FRICTION" ]; then echo "*** friction notes so far: $FRICTION"; fi
    fi
    return 0
}
trap cleanup EXIT

spawn_daemon() { # <max_sends> <outfile>
    env ACQ_TRIPWIRE=1 ACQ_MAX_SENDS="$1" ACQ_IDLE_SHUTDOWN=600 \
        "$ACQ" daemon run >"$RUN_DIR/$2" 2>&1 &
    for _ in $(seq 1 100); do
        pid=$(status_json | jq -r '.pid // empty')
        [ -n "$pid" ] && { echo "$pid"; return 0; }
        sleep 0.1
    done
    echo "daemon did not come up; see $RUN_DIR/$2 and $LOG" >&2
    return 1
}

stop_daemon() {
    "$ACQ" daemon stop >/dev/null 2>&1 || true
    for _ in $(seq 1 100); do
        daemon_up || return 0
        sleep 0.1
    done
    echo "daemon on $SOCK did not stop" >&2
    return 1
}

# The rails state after a wire phase. The ceiling is exact, so the bound
# is REACHED at the last planned send (rails.rs trips on sends >= max) —
# that is the expected end of a phase, provided the journal shows exactly
# the ceiling's sends and nothing was refused. Any other halt ends the run:
# a ceiling halt with fewer sends than planned means an unprojected send
# consumed the bound; a tripwire trip is a real violation.
check_rails() { # <phase> <journal offset at phase start> <expected sends>
    local halted sent
    halted=$(status_json | jq -r '.rails.halted // empty')
    sent=$(sends_since "$2")
    if [ -z "$halted" ]; then
        if [ "$sent" != "$3" ]; then
            echo "*** $1: $sent send(s) journaled, $3 planned" >&2
            return 1
        fi
        return 0
    fi
    case "$halted" in
    *ceiling*)
        if [ "$sent" = "$3" ]; then
            echo "rails: bound reached exactly after the planned $3 send(s) ($halted) — nothing refused"
            return 0
        fi
        echo "*** the ceiling halted the daemon during $1 after $sent send(s), $3 planned:" >&2
        echo "*** a send the plan did not project consumed the bound; read $JOURNAL. acq jobs" >&2
        echo "*** shows what was refused." >&2
        return 1 ;;
    *)
        echo "*** TRIPWIRE TRIP during $1: $halted" >&2
        echo "*** stop. Read the journal; observe the 360 s post-violation rule." >&2
        return 1 ;;
    esac
}

confirm() {
    [ "$INTERACTIVE" = 1 ] || return 0
    echo ""
    echo ">>> $1"
    read -r -p ">>> enter to proceed (ctrl-c to abort) "
}

# Friction is data, collected at the moment it happens. Empty skips.
note() { # <phase>
    [ "$INTERACTIVE" = 1 ] || return 0
    local text
    echo ""
    read -r -p "friction note for \"$1\" (anything that felt wrong, slow, or surprising; enter to skip): " text
    if [ -n "$text" ]; then
        if [ ! -s "$FRICTION" ]; then
            printf '# Friction notes — tracer rung %s (%s)\n\n' "$(date -u +%F)" "$MODE" >"$FRICTION"
        fi
        printf -- '- **%s**: %s\n' "$1" "$text" >>"$FRICTION"
    fi
}

echo "tracer rung ($MODE) — binary $ver"
echo "socket $SOCK | journal $JOURNAL | evidence -> $RUN_DIR"
echo "league $LEAGUE | selection $SELECTION | max_age_seconds $MAX_AGE | up to $CYCLES cycle(s)"

# ---- phase 0: the account ---------------------------------------------------

# The selector resolves the way `acq` resolves it (index.rs
# `account_matches`): the username or the username without its
# #discriminator, both case-insensitive, or the exact uuid; exactly one
# hit. The exact username is what gets exported.
resolve_account() { # <selector>
    "$ACQ" accounts --json | jq -r --arg s "$1" '
        ($s | ascii_downcase) as $l
        | [.[] | select((.username | ascii_downcase) == $l
                        or (.username | ascii_downcase | split("#")[0]) == $l
                        or .uuid == $s)]
        | if length == 1 then .[0].username else "AMBIGUOUS_OR_NONE:\(length)" end'
}
account_uuid() { "$ACQ" accounts --json | jq -r --arg u "$ACQ_ACCOUNT" '.[] | select(.username == $u) | .uuid // empty'; }

NEED_LOGIN=0
if [ "$MODE" = live ]; then
    echo "accounts on this machine:"
    "$ACQ" accounts || true
    persisted=$("$ACQ" accounts --json 2>/dev/null |
        jq '[.[] | select(.persisted)] | length' || echo 0)
    if [ -n "$ACCOUNT" ]; then
        resolved=$(resolve_account "$ACCOUNT")
        case "$resolved" in
        AMBIGUOUS_OR_NONE:*)
            echo "refusing: --account $ACCOUNT matches ${resolved#*:} accounts (needs exactly one:" >&2
            echo "the exact username, the username without #discriminator, or the uuid)" >&2
            exit 2 ;;
        esac
        export ACQ_ACCOUNT="$resolved"
    elif [ "$persisted" -gt 1 ]; then
        echo "refusing: $persisted persisted accounts — every job and store read needs a selector." >&2
        echo "rerun with --account <selector>, the account whose tabs these are." >&2
        exit 2
    else
        ACQ_ACCOUNT=$("$ACQ" accounts --json | jq -r '[.[] | select(.persisted)][0].username // empty')
        export ACQ_ACCOUNT
    fi
    [ -n "${ACQ_ACCOUNT:-}" ] || { echo "no persisted account; log in first (ACQ_GGG=1 acq auth)" >&2; exit 2; }
    echo "acting as: $ACQ_ACCOUNT"
    uuid=$(account_uuid)
    if [ -z "$uuid" ]; then
        echo "the index has no uuid for $ACQ_ACCOUNT (a login predating uuid-at-login);"
        echo "intent binds to the uuid, so phase 1 logs in again as this account."
        NEED_LOGIN=1
    else
        echo "uuid on record ($uuid); no login needed — each daemon restores the persisted session."
    fi
else
    NEED_LOGIN=1
fi
OFFSET=$(journal_size)
LOGIN_LIFETIME=0

# ---- phase 1: login (only when intent cannot bind without it) ---------------

if [ "$NEED_LOGIN" = 1 ]; then
    if [ "$MODE" = live ]; then
        confirm "phase 1: fresh daemon (ceiling 2), browser login as $ACQ_ACCOUNT — a code-exchange POST and the login's own GET /profile, exactly"
    fi
    login_offset=$(journal_size)
    PID=$(spawn_daemon 2 daemon-login.out)
    echo "login daemon: pid $PID"
    if [ "$MODE" = live ]; then
        "$ACQ" auth 2>&1 | tee "$RUN_DIR/auth.out"
        logged=$(grep -o 'logged in as .*' "$RUN_DIR/auth.out" | sed 's/logged in as //' || true)
        if [ "$logged" != "$ACQ_ACCOUNT" ]; then
            echo "*** the login came back as '${logged:-nobody}', not $ACQ_ACCOUNT — stop; the policy would bind to the wrong identity" >&2
            exit 1
        fi
        uuid=$(account_uuid)
        [ -n "$uuid" ] || { echo "*** login succeeded but the index still has no uuid for $ACQ_ACCOUNT" >&2; exit 1; }
        echo "uuid recorded: $uuid"
    else
        # Scripted mock login (AGENTS.md): auth --no-browser, then approve.
        "$ACQ" auth --no-browser >"$RUN_DIR/auth.out" 2>&1 &
        AUTH_PID=$!
        URL=
        for _ in $(seq 1 50); do
            URL=$(grep -o 'http://[^ ]*authorize[^ ]*' "$RUN_DIR/auth.out" | head -1 || true)
            [ -n "$URL" ] && break
            sleep 0.1
        done
        [ -n "$URL" ] || { echo "no login URL in auth.out" >&2; exit 1; }
        APPROVE=${URL/\/authorize?/\/approve?}
        curl -sL "$APPROVE&user=tracer-rung" >/dev/null
        wait "$AUTH_PID"
    fi
    check_rails "login" "$login_offset" 2
    stop_daemon
    LOGIN_LIFETIME=1
    note "logging in"
fi

# ---- phase 2: intent, then the plan with no daemon --------------------------

if [ "$SELECTION" = all ]; then
    tabs_json='"all"'
else
    tabs_json=$(echo "$SELECTION" | tr ',' '\n' | grep . | jq -R . | jq -sc .)
fi
POLICY=$(jq -nc --arg l "$LEAGUE" --argjson t "$tabs_json" --argjson a "$MAX_AGE" \
    '{version: 1, leagues: {($l): {tabs: $t, max_age_seconds: $a}}}')
echo ""
echo "phase 2: writing the sync policy (no daemon, no wire):"
echo "  $POLICY"
"$ACQ" policy set "$POLICY" --json >"$RUN_DIR/policy.json"
REVISION=$(jq -r '.revision' "$RUN_DIR/policy.json")
echo "  revision $REVISION"
"$ACQ" policy show | tee "$RUN_DIR/policy-show.txt"

# The offline plan, human and JSON, with the socket dead: whatever the
# note says, it must be "no quote" — a plan compiled with no daemon.
offline_plan() { # <tag>
    daemon_up && { echo "a daemon is up before the offline plan — not offline" >&2; return 1; }
    "$ACQ" refresh --plan --league "$LEAGUE" >"$RUN_DIR/plan-$1-offline.txt" 2>&1 || {
        cat "$RUN_DIR/plan-$1-offline.txt" >&2; return 1; }
    "$ACQ" refresh --plan --league "$LEAGUE" --json \
        >"$RUN_DIR/plan-$1-offline.json" 2>"$RUN_DIR/plan-$1-offline.note" || {
        cat "$RUN_DIR/plan-$1-offline.note" >&2; return 1; }
    daemon_up && { echo "*** a daemon appeared during the offline plan — the quote path spawned one" >&2; return 1; }
    if ! grep -q "no quote" "$RUN_DIR/plan-$1-offline.note"; then
        echo "*** the offline plan's note is not a 'no quote' line:" >&2
        cat "$RUN_DIR/plan-$1-offline.note" >&2
        return 1
    fi
    return 0
}

# What two envelopes must agree on to be "the same plan": everything but
# the quote, the two timestamps a re-read stamps afresh, and the derived
# ages inside reasons (`age_seconds` advances with the clock between two
# compiles; the reason KIND stays and is compared).
plan_identity() {
    jq -S 'del(.quote, .generated_at, .basis.snapshot_taken_at)
           | walk(if type == "object" then del(.age_seconds) else . end)' "$1"
}
# The action list of an envelope, one line each — rendered from the file
# that will be applied, so what is confirmed is what goes out.
render_actions() {
    jq -r '.actions[] | "  \(.action)  \(.league)  \(.parent // "")\(if .parent then "/" else "" end)\(.id // "")  \(.name // "")  \(.reason.kind)"' "$1"
}

echo ""
echo "the plan, compiled offline (no daemon is running):"
offline_plan c1
cat "$RUN_DIR/plan-c1-offline.txt"
echo "(stderr) $(cat "$RUN_DIR/plan-c1-offline.note")"
note "writing the policy and reading the first plan"

# ---- phase 3: the cycles ----------------------------------------------------

# One row per cycle for the verifier: cycle, lifetime index in this run's
# journal, logical requests, probes expected, ceiling, quote outcome.
CYCLE_ROWS="$RUN_DIR/cycles.tsv"
: >"$CYCLE_ROWS"
CLOSED=0
for c in $(seq 1 "$CYCLES"); do
    tag="c$c"
    if [ "$c" -gt 1 ]; then
        echo ""
        echo "cycle $c: the plan, compiled offline (no daemon is running):"
        offline_plan "$tag"
        cat "$RUN_DIR/plan-$tag-offline.txt"
    fi
    plan="$RUN_DIR/plan-$tag-offline.json"
    logical=$(jq -r '.logical_requests' "$plan")
    lists=$(jq '[.actions[] | select(.action == "list_stashes")] | length' "$plan")
    fetches=$(jq '[.actions[] | select(.action == "fetch_tab")] | length' "$plan")
    subs=$(jq '[.actions[] | select(.action == "fetch_substash")] | length' "$plan")
    wire_min=$(jq -r '.wire_sends.min' "$plan")
    wire_max=$(jq -r '.wire_sends.max' "$plan")
    [ "$wire_min" = "$logical" ] || { echo "plan's wire minimum $wire_min != logical $logical" >&2; exit 1; }

    if [ "$logical" = 0 ]; then
        # The closed loop: an empty plan applies as a no-op that contacts
        # nothing. No daemon is up, and none may appear.
        echo ""
        echo "cycle $c: the plan is empty — applying it with no daemon (must contact nothing):"
        "$ACQ" refresh --apply --league "$LEAGUE" --json | tee "$RUN_DIR/apply-$tag-noop.json"
        [ "$(jq -r '.requests' "$RUN_DIR/apply-$tag-noop.json")" = 0 ] || { echo "no-op apply reported requests != 0" >&2; exit 1; }
        daemon_up && { echo "*** the no-op apply spawned a daemon" >&2; exit 1; }
        CLOSED=$c
        break
    fi

    probes=0
    [ "$lists" -gt 0 ] && probes=$((probes + 1))
    [ $((fetches + subs)) -gt 0 ] && probes=$((probes + 1))
    ceiling=$((1 + probes + logical))
    pace=""
    if [ "$MODE" = live ]; then
        stash_gets=$((fetches + subs))
        if [ "$stash_gets" -gt 30 ]; then
            pace=" — more than 30 stash GETs: a ~15 s hold after each 15 and a ~343 s hold after each 30 (rung 10's shape); that is the limiter working"
        elif [ "$stash_gets" -gt 15 ]; then
            pace=" — more than 15 stash GETs: expect one ~15 s limiter hold before the 16th (rung 7b); that is the limiter working"
        fi
    fi
    confirm "cycle $c: fresh daemon with ceiling $ceiling = 1 token POST + $probes probe HEAD(s) + $logical GET(s) ($lists listing, $fetches tab, $subs substash; plan says $wire_min..$wire_max wire sends)$pace"
    cycle_offset=$(journal_size)
    PID=$(spawn_daemon "$ceiling" "daemon-$tag.out")
    echo "daemon $tag: pid $PID (ceiling $ceiling)"

    echo ""
    echo "the same plan, now with the daemon up (quote attempt; nothing is sent):"
    "$ACQ" refresh --plan --league "$LEAGUE" 2>&1 | tee "$RUN_DIR/plan-$tag.txt"
    "$ACQ" refresh --plan --league "$LEAGUE" --json \
        >"$RUN_DIR/plan-$tag.json" 2>"$RUN_DIR/plan-$tag.note" || {
        cat "$RUN_DIR/plan-$tag.note" >&2; exit 1; }
    if [ "$(jq -r '.quote != null' "$RUN_DIR/plan-$tag.json")" = true ]; then
        quote="quoted"
    else
        quote="unquoted: $(tr -d '\n' <"$RUN_DIR/plan-$tag.note")"
    fi
    echo "quote outcome: $quote"
    # The envelope that will be applied must be the offline one (the
    # ceiling's source) plus the quote: same actions in order, basis,
    # identity, revision, counts. Facts moving between the two compiles
    # is a stop, not something to paper over.
    if [ "$(plan_identity "$plan")" != "$(plan_identity "$RUN_DIR/plan-$tag.json")" ]; then
        echo "*** the envelope compiled with the daemon up is not the offline envelope plus a quote:" >&2
        diff <(plan_identity "$plan") <(plan_identity "$RUN_DIR/plan-$tag.json") >&2 || true
        stop_daemon
        exit 1
    fi
    [ "$(sends_since "$cycle_offset")" = 0 ] || { echo "*** the quote sent something" >&2; exit 1; }
    echo ""
    echo "the envelope about to be applied ($RUN_DIR/plan-$tag.json), its $logical action(s) in order:"
    render_actions "$RUN_DIR/plan-$tag.json"
    confirm "apply exactly these $logical request(s) as one apply parent (--max-requests $logical); long waits are the limiter"
    "$ACQ" refresh --apply="$RUN_DIR/plan-$tag.json" --max-requests "$logical" --json \
        >"$RUN_DIR/apply-$tag.json" 2>"$RUN_DIR/apply-$tag.err" &
    CLIENT_PID=$!
    if wait "$CLIENT_PID"; then :; else
        CLIENT_PID=
        echo "*** apply exited non-zero:" >&2
        cat "$RUN_DIR/apply-$tag.json" "$RUN_DIR/apply-$tag.err" >&2
        check_rails "apply" "$cycle_offset" "$ceiling" || true
        exit 1
    fi
    CLIENT_PID=
    outcome=$(jq -r '.outcome' "$RUN_DIR/apply-$tag.json")
    requests=$(jq -r '.payload.requests // empty' "$RUN_DIR/apply-$tag.json")
    done_n=$(jq -r '.payload.children.done // 0' "$RUN_DIR/apply-$tag.json")
    failed_n=$(jq -r '.payload.children.failed // 0' "$RUN_DIR/apply-$tag.json")
    echo "apply: outcome $outcome, $requests request(s) admitted, children done $done_n / failed $failed_n"
    if [ "$outcome" != success ] || [ "$requests" != "$logical" ] || [ "$done_n" != "$logical" ]; then
        echo "*** the apply did not execute exactly the plan:" >&2
        cat "$RUN_DIR/apply-$tag.json" >&2
        "$ACQ" jobs >&2 || true
        check_rails "apply" "$cycle_offset" "$ceiling" || true
        exit 1
    fi
    check_rails "apply" "$cycle_offset" "$ceiling"
    "$ACQ" jobs --json >"$RUN_DIR/jobs-$tag.json"
    stop_daemon
    printf '%s\t%s\t%s\t%s\t%s\t%s\n' "$c" $((LOGIN_LIFETIME + c)) "$logical" "$probes" "$ceiling" "$quote" >>"$CYCLE_ROWS"
    note "cycle $c (plan → quote → apply)"
done
if [ "$CLOSED" = 0 ]; then
    echo ""
    echo "note: $CYCLES cycle(s) run and the plan is still not empty — the loop did not close within --cycles;"
    # shellcheck disable=SC2016  # the backticks are literal
    echo 'the next `acq refresh --plan` says what is left. Recorded as such, not as a failure.'
fi

# ---- phase 4: the next read = facts -------------------------------------------

echo ""
echo "phase 4: the facts, read back from the store (no daemon):"
"$ACQ" tabs --league "$LEAGUE" >"$RUN_DIR/tabs.txt"
"$ACQ" tabs --league "$LEAGUE" --json >"$RUN_DIR/tabs.json"
if [ "$SELECTION" = all ]; then
    head -40 "$RUN_DIR/tabs.txt"
else
    head -1 "$RUN_DIR/tabs.txt"
    unknown=$(jq -r '.unknown_tabs | join(" ")' "$RUN_DIR/plan-c1-offline.json")
    missing=0
    for id in $(echo "$SELECTION" | tr ',' ' '); do
        if grep -- "^$id " "$RUN_DIR/tabs.txt"; then continue; fi
        case " $unknown " in
        *" $id "*) echo "$id: not in the store — the first plan reported it as unknown (a typo, or a tab that is gone); never fetched" ;;
        *) echo "*** $id: selected, planned, but not in the store's tab list after the run" >&2; missing=1 ;;
        esac
    done
    tail -1 "$RUN_DIR/tabs.txt"
    [ "$missing" = 0 ] || exit 1
    # Discovery the id list cannot cover: substashes under a selected
    # parent, present in the store but outside the policy.
    uncovered=$(jq -r --argjson sel "$tabs_json" '[.[] | .parent as $p | .id as $i | select($p != null and ($sel | index($p)) != null and ($sel | index($i)) == null) | $i] | length' "$RUN_DIR/tabs.json")
    if [ "$uncovered" -gt 0 ]; then
        echo "observation: $uncovered substash(es) discovered under selected tabs are NOT covered by the id list —"
        echo "the planner matches ids exactly. Naming them (a policy revision) or 'all' would plan them next."
    fi
fi
"$ACQ" store status 2>&1 | tee "$RUN_DIR/store-status.txt"
"$ACQ" store events --hours 1 >"$RUN_DIR/store-events.txt"
echo "item events from this run: $(grep -c . "$RUN_DIR/store-events.txt" || true) line(s) in $RUN_DIR/store-events.txt"
"$ACQ" refresh --plan --league "$LEAGUE" >"$RUN_DIR/plan-final.txt" 2>&1
if [ "$CLOSED" != 0 ] && ! grep -q "nothing to do" "$RUN_DIR/plan-final.txt"; then
    echo "*** the loop was recorded as closed but the final plan is not empty:" >&2
    cat "$RUN_DIR/plan-final.txt" >&2
    exit 1
fi
note "reading the facts back"
if [ "$MODE" = mock ]; then
    PID=$(spawn_daemon 2 daemon-logout.out)
    "$ACQ" auth logout >/dev/null 2>&1 || true
    stop_daemon
fi
# The wire phases are over and no daemon is left; what remains is analysis,
# so the trap's "a daemon may still be up" warning no longer applies.
COMPLETED=1

# ---- phase 5: evidence and verification -----------------------------------------

cp "$JOURNAL" "$RUN_DIR/sends.jsonl"
cp "$LOG" "$RUN_DIR/daemon.log" 2>/dev/null || true

verify() { python3 - "$JOURNAL" "$OFFSET" "$CYCLE_ROWS" "$LOGIN_LIFETIME" "$CLOSED" "$MODE" <<'PY'
import json, sys

journal, offset, rows_path, login_lifetime, closed, mode = sys.argv[1:7]
login_lifetime, closed = int(login_lifetime), int(closed)

f = open(journal); f.seek(int(offset))
lifetimes, cur = [], None
for raw in f:
    if not raw.strip():
        continue
    l = json.loads(raw)
    if l.get("event") == "open":
        cur = {"pid": l["pid"], "build": l["build"], "clock": l["clock"], "sends": []}
        lifetimes.append(cur)
        continue
    if cur is None:
        cur = {"pid": l.get("pid"), "build": "?", "clock": "?", "sends": []}
        lifetimes.append(cur)
    cur["sends"].append(l)

cycles = []
for line in open(rows_path):
    if line.strip():
        c, lt, logical, probes, ceiling, quote = line.rstrip("\n").split("\t")
        cycles.append({"cycle": int(c), "lifetime": int(lt), "logical": int(logical),
                       "probes": int(probes), "ceiling": int(ceiling), "quote": quote})

# Routes taught by their first GET rather than a probe (declared route
# knowledge in daemon.rs), plus the token endpoint, which has no probe.
NO_PROBE = {"oauth-token", "profile", "league"}
fail, totals = [], []
expected = login_lifetime + len(cycles)
if mode == "mock" and len(lifetimes) == expected + 1 and not lifetimes[-1]["sends"]:
    # Mock mode ends with a throwaway daemon for the logout; it sends nothing.
    lifetimes.pop()
    print("(the mock-mode logout daemon, 0 sends, is left out of the count)")

from datetime import datetime

def when(s):
    return datetime.fromisoformat(s["ts"].replace("Z", "+00:00"))

def reported_rules(rate):
    """Every (hits, period_seconds) pair in every *-state header."""
    rules = []
    for k, v in (rate or {}).items():
        if k.endswith("-state"):
            for rule in str(v).split(","):
                parts = rule.split(":")
                if len(parts) >= 2 and parts[0].isdigit() and parts[1].isdigit():
                    rules.append((int(parts[0]), int(parts[1])))
    return rules

def bucket(period):
    """GGG's timing bucket past a window (N11/N12; rung 7b: +5 s on the
    10 s window, rung 10: +60 s on the 300 s window) — a send that old
    may still be counted, so it is still 'ours'."""
    return 60 if period >= 300 else 5

# This run's own counted GETs per exact route (account included), with
# their times, so a probe's reported hits per window can be bounded by
# what we ourselves sent inside that window (plus its bucket) — never by
# the run's cumulative total, which would let outside traffic hide
# behind aged-out sends of ours.
ours = {}
for i, lt in enumerate(lifetimes, 1):
    label = "login" if (login_lifetime and i == 1) else f"cycle {i - login_lifetime}"
    print(f"lifetime {i} ({label}): pid {lt['pid']}  build {lt['build']}  clock {lt['clock']}")
    counts, first = {}, {}
    for s in lt["sends"]:
        m, r, st = s["method"], s["route"], s.get("status")
        base = r.split("@", 1)[0]
        counts[m] = counts.get(m, 0) + 1
        first.setdefault(base, m)
        flag = ""
        if s.get("error") or st is None or st >= 400:
            flag = "  <-- NOT OK"
            fail.append(f"lifetime {i}: {m} {r} -> {st} error={s.get('error')}")
        if st == 429:
            fail.append(f"lifetime {i}: 429 on {r}")
        if m == "HEAD":
            t = when(s)
            rules = reported_rules(s.get("rate"))
            checks, over = [], False
            for hits, period in rules:
                mine = sum(1 for sent in ours.get(r, []) if (t - sent).total_seconds() <= period + bucket(period))
                checks.append(f"{hits} of ours {mine} in {period}s")
                if hits > mine:
                    over = True
            verdict = ""
            if over:
                verdict = f"  <-- reports more hits than this run sent inside the window ({'; '.join(checks)}): someone else is on this account"
                fail.append(f"lifetime {i}: probe on {r} reported {'; '.join(checks)}")
            elif rules and not any(h for h, _ in rules):
                verdict = "  (0 hits: nothing else on this account" + (
                    " — standing rule met)" if not ours.get(r) else "; this run's earlier sends have aged out)")
            elif rules:
                verdict = f"  (hits within this run's own sends in each window: {'; '.join(checks)} — expected)"
            print(f"  HEAD {r} -> {st}  rate {json.dumps(s.get('rate'))}{flag}{verdict}")
        else:
            print(f"  {m} {r} -> {st}  wait_ms {s.get('wait_ms')}{flag}")
            if m == "GET":
                ours.setdefault(r, []).append(when(s))
    for base, m in first.items():
        if base not in NO_PROBE and m != "HEAD":
            fail.append(f"lifetime {i}: first send on {base} was {m}, not the probe")
    t = f"{counts.get('POST',0)}/{counts.get('HEAD',0)}/{counts.get('GET',0)}"
    totals.append(f"{t} = {len(lt['sends'])}")
    print(f"  totals (POST/HEAD/GET): {totals[-1]}")

if len(lifetimes) != expected:
    fail.append(f"expected {expected} daemon lifetime(s) in this run's journal, saw {len(lifetimes)}")
if login_lifetime and lifetimes:
    login = lifetimes[0]["sends"]
    posts = sum(1 for s in login if s["method"] == "POST")
    gets = sum(1 for s in login if s["method"] == "GET" and s["route"].startswith("profile"))
    if posts != 1 or gets != 1 or len(login) != 2:
        fail.append(f"login lifetime: expected exactly one code-exchange POST and one GET /profile, saw {len(login)} sends")

print()
print("plan vs journal, per cycle (the wire estimate's minimum should hold exactly):")
for c in cycles:
    idx = c["lifetime"] - 1
    if idx >= len(lifetimes):
        fail.append(f"cycle {c['cycle']}: no journal lifetime for it")
        continue
    sends = lifetimes[idx]["sends"]
    gets = sum(1 for s in sends if s["method"] == "GET")
    heads = sum(1 for s in sends if s["method"] == "HEAD")
    posts = sum(1 for s in sends if s["method"] == "POST")
    ok = gets == c["logical"] and heads == c["probes"] and posts == 1 and len(sends) == c["ceiling"]
    print(f"  cycle {c['cycle']}: plan {c['logical']} request(s) + {c['probes']} probe(s) + 1 token POST"
          f" -> journal {posts} POST / {heads} HEAD / {gets} GET (ceiling {c['ceiling']}); {c['quote']}"
          + ("" if ok else "  <-- MISMATCH"))
    if not ok:
        fail.append(f"cycle {c['cycle']}: journal {posts}/{heads}/{gets} vs plan {c['logical']} + {c['probes']} probes + 1 POST")
if closed:
    print(f"  cycle {closed}: empty plan, no-op apply, no daemon, nothing journaled — the loop closed")
else:
    print("  the loop did not close within the cycle budget (recorded, not a failure)")

print()
if fail:
    print("CHECKS FAILED — a ledger row still gets written, saying what happened:")
    for x in fail:
        print(f"  - {x}")
    sys.exit(1)
quotes = ", ".join(f"c{c['cycle']} {c['quote'].split(':')[0]}" for c in cycles)
print("checks passed: every route probed before its first send in every lifetime,")
print("no probe reported hits beyond this run's own sends inside each window (the")
print("first probe on each route saw 0), no non-2xx, and each cycle's journal matches its plan")
print("exactly (no 429 re-sends: the estimate's minimum held). Quote outcomes: " + (quotes or "none") + ".")
print()
print("draft ledger row:")
lt = ", ".join(f"L{i+1} {t}" for i, t in enumerate(totals))
cyc = "; ".join(f"c{c['cycle']} {c['logical']} req" for c in cycles)
print(f"| <date> | tracer | <tip> | pass | {lt} | 0 | policy → plan → apply → replan"
      f" ({cyc}{'; closed' if closed else '; not closed'}); each cycle's sends == its plan + probes + POST;"
      f" quote: {quotes or 'n/a'}; friction notes in the rung section; runs/<date>-tracer/ |")
PY
}
if ! verify | tee "$RUN_DIR/summary.txt"; then
    echo ""
    echo "*** verification FAILED — see above; evidence in $RUN_DIR" >&2
    exit 1
fi

echo ""
echo "evidence in $RUN_DIR (journal, daemon logs, plans, apply results, store reads, summary)."
if [ -s "$FRICTION" ]; then
    echo ""
    echo "friction notes ($FRICTION):"
    cat "$FRICTION"
else
    echo "no friction notes were entered."
fi
if [ "$MODE" = live ]; then
    echo ""
    echo "next: paste the ledger row into LIVE-TESTING.md's run ledger and the friction"
    echo "notes into the rung section; anything learned about GGG goes to ground truth master-side."
fi
