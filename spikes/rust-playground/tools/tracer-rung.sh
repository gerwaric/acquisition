#!/bin/bash
# Tracer rung (LIVE-TESTING.md, "Tracer rung — policy → plan → apply →
# replan"): the owner's first real use of the refresh slice, driven end to
# end with the rails on, verified from the journal, and with the friction
# notes collected as the run goes rather than recalled afterwards. The
# human parts stay human: run this from a terminal (the daemon needs your
# keychain, and the login needs your browser), and every wire phase waits
# for an explicit enter.
#
#   tools/tracer-rung.sh [--account SEL] [--realm R] [--league L] \
#                        [--max-age S] [--cycles K] [--characters all|id,...] \
#                        <tab1,...,tabN | all | none>                       # live
#   tools/tracer-rung.sh --mock [--cycles K] [--characters …] <tabs|all|none>  # rehearsal
#
# The selection becomes the sync policy (`acq policy set`, policy v3): the
# named tab ids, or `all` for every tab the league lists, or `none` for no
# tab coverage at all; `--characters` adds the character facet the same
# way (`all`, or GGG character ids — the full 64-hex, from `acq store
# characters`). A character-only run (`none` plus `--characters`) is how
# the PoE2 realm is driven: stashes are PoE1 only, so `--realm poe2` takes
# no tab selection. The character list is realm-wide and gets its own
# probe; each character fetch is one GET on `character-request-limit`
# (5 per 10 s, 30 per 300 s — the stash policy's shape).
# A policy id covers the tab
# and its children (CONTEXT.md, decided 2026-09-01): a map/unique tab's
# substashes are planned the cycle after the parent's first fetch lands
# their stubs (a plan never expands itself), a folder's children at once.
# So an id list that names a map tab runs the discovery cycle too, and the
# loop closes when every covered substash is fetched or skipped as an
# empty stub. Live, `all` on a 322-tab account is a 323-request plan with
# ~343 s limiter holds every 30 sends (rung 10's shape) and then every
# substash — the owner's call, not this script's; `all` defaults
# max_age_seconds to a day so the hour-long cycle does not make its own
# facts stale before the next plan (the driver refuses a window shorter
# than the cycle it projects).
#
# Shape of the run (one fresh daemon per wire phase, each under an EXACT
# send ceiling, stopped when its phase is over):
#   0  preflight (no wire): provenance (the binary's stamp is HEAD, and
#      live refuses working-tree changes to the rung's own files, so the
#      ledger's tip names what ran), leftover env, no daemon, account
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
#      expected end of a cycle, and it is checked as such: the daemon
#      must report the tripwire armed, the ceiling equal to the plan, that
#      many sends counted, and a ceiling halt in force, with the journal
#      agreeing. A send the plan did not project would consume the bound
#      and show as a planned child refused. Repeat until
#      the plan is empty (the loop closed) or --cycles is hit. An empty
#      plan's `--apply` runs with no daemon at all: the no-op must contact
#      nothing, and the socket is checked dead after it.
#   4  the next read = facts: `acq tabs`, `acq store status/events`; a
#      selected tab missing from the store, or a read that fails, fails
#      the run — the readback is part of what the rung tests
#   5  evidence + verification (tools/tracer-verify.py; its --self-test
#      pins the branches the mock cannot reach): per lifetime, probe
#      before first send per route, no non-2xx; every probe's reported
#      hits, per window, are at most this run's own GETs on that route
#      inside the window plus its timing bucket (so the run's FIRST probe
#      on a route must report 0 — the standing rule — and a later cycle's
#      probe may carry only the earlier cycle's hits, which are ours); GET
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
REALM=pc
LEAGUE=Standard
MAX_AGE=
CYCLES=4
CHARACTERS=
while [ $# -gt 0 ]; do
    case "$1" in
    --mock) MODE=mock; shift ;;
    --account) ACCOUNT=${2:?--account needs a value}; shift 2 ;;
    --realm) REALM=${2:?--realm needs a value}; shift 2 ;;
    --league) LEAGUE=${2:?--league needs a value}; shift 2 ;;
    --max-age) MAX_AGE=${2:?--max-age needs a value}; shift 2 ;;
    --cycles) CYCLES=${2:?--cycles needs a value}; shift 2 ;;
    --characters) CHARACTERS=${2:?--characters needs a value}; shift 2 ;;
    *) break ;;
    esac
done
SELECTION=${1:?usage: tracer-rung.sh [--mock] [--account SEL] [--realm R] [--league L] [--max-age S] [--cycles K] [--characters all|id,...] <tab1,...|all|none>}
if [ "$SELECTION" = none ] && [ -z "$CHARACTERS" ]; then
    echo "refusing: tabs 'none' with no --characters names no work (the policy would be refused)" >&2
    exit 2
fi
command -v jq >/dev/null || { echo "jq is required" >&2; exit 2; }
command -v python3 >/dev/null || { echo "python3 is required" >&2; exit 2; }
# The freshness window must outlive the cycle that fills it, or the next
# plan re-lists and re-fetches what the previous one just landed: an id
# list's cycle is seconds, `all` on a real account is an hour (rung 10),
# so `all` defaults to a day. Checked again per cycle against the plan.
if [ -z "$MAX_AGE" ]; then
    if [ "$SELECTION" = all ]; then MAX_AGE=86400; else MAX_AGE=3600; fi
fi
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

# The ledger cites a tip; the tip must identify the rung that ran — the
# driver, the verifier, the control documents, and the crates — not only
# the binary. Live refuses working-tree changes there; mock only notes.
dirty=$(git -C "$here" status --porcelain -- tools LIVE-TESTING.md CONTEXT.md crates Cargo.toml Cargo.lock)
if [ -n "$dirty" ]; then
    echo "working tree differs from $tip in the rung's own files:" >&2
    echo "$dirty" >&2
    if [ "$MODE" = live ]; then
        echo "refusing: commit (and cargo build) first — the ledger's tip must name what ran" >&2
        exit 2
    fi
    echo "(mock mode: continuing anyway)"
fi

RUN_START=$(date +%s)
RUN_DIR="$here/runs/$(date -u +%F)-tracer"
# rehearsals are reproducible and live apart from the evidence the ledger cites
if [ "$MODE" = mock ]; then RUN_DIR="$here/runs/mock/$(date -u +%F)-tracer"; fi
# One directory per attempt: a repeated run the same day gets a time
# suffix rather than overwriting or mixing with the earlier evidence.
if [ -d "$RUN_DIR" ] && [ -n "$(ls -A "$RUN_DIR")" ]; then
    RUN_DIR="$RUN_DIR-$(date -u +%H%M%S)"
fi
mkdir -p "$RUN_DIR"
FRICTION="$RUN_DIR/friction.md"

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

# The rails state after a wire phase, read from the daemon that ran it.
# The ceiling is exact, so the bound is REACHED at the last planned send
# (rails.rs trips on sends >= max): the expected end of a phase is a
# daemon that reports the tripwire armed, a ceiling equal to the plan,
# exactly that many sends counted, a ceiling halt in force, and the
# journal agreeing — every one of those, or the phase fails. A journal
# count that merely matches is not evidence the rail was there.
check_rails() { # <phase> <journal offset at phase start> <expected sends>
    local st armed max sends halted journaled
    st=$(status_json)
    armed=$(echo "$st" | jq -r '.rails.tripwire_enabled // false')
    max=$(echo "$st" | jq -r '.rails.max_sends // "none"')
    sends=$(echo "$st" | jq -r '.rails.sends // "unknown"')
    halted=$(echo "$st" | jq -r '.rails.halted // empty')
    journaled=$(sends_since "$2")
    local ok=1
    [ "$armed" = true ] || { echo "*** $1: the daemon reports the tripwire NOT armed" >&2; ok=0; }
    [ "$max" = "$3" ] || { echo "*** $1: the daemon's ceiling is $max, the plan's is $3" >&2; ok=0; }
    [ "$sends" = "$3" ] || { echo "*** $1: the daemon counted $sends send(s), $3 planned" >&2; ok=0; }
    [ "$journaled" = "$3" ] || { echo "*** $1: $journaled send(s) journaled, $3 planned" >&2; ok=0; }
    case "$halted" in
    "") echo "*** $1: no halt in force — the bound was not reached (or the rail was not read)" >&2; ok=0 ;;
    *ceiling*) ;;
    *)
        echo "*** TRIPWIRE TRIP during $1: $halted" >&2
        echo "*** stop. Read the journal; observe the 360 s post-violation rule." >&2
        return 1 ;;
    esac
    if [ "$ok" = 1 ]; then
        echo "rails: armed, ceiling $max, bound reached exactly after the planned $3 send(s) ($halted) — nothing refused"
        return 0
    fi
    echo "*** $1 did not end on the bound as planned; read $JOURNAL, and acq jobs for anything refused" >&2
    return 1
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
echo "realm $REALM | league $LEAGUE | selection $SELECTION | max_age_seconds $MAX_AGE | up to $CYCLES cycle(s)"

# ---- phase 0: the account ---------------------------------------------------

# The selector resolves the way `acq` resolves it (index.rs
# `account_matches`): the username or the username without its
# #discriminator, both case-insensitive (Unicode lowercasing, as Rust's
# `to_lowercase`), or the exact uuid; exactly one hit. The exact username
# is what gets exported.
resolve_account() { # <selector>
    "$ACQ" accounts --json | python3 -c '
import json, sys
sel = sys.argv[1]; low = sel.lower()
hits = [e["username"] for e in json.load(sys.stdin)
        if e["username"].lower() == low
        or e["username"].lower().split("#", 1)[0] == low
        or e.get("uuid") == sel]
print(hits[0] if len(hits) == 1 else f"AMBIGUOUS_OR_NONE:{len(hits)}")' "$1"
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
LOG_OFFSET=0
if [ -f "$LOG" ]; then LOG_OFFSET=$(wc -c <"$LOG" | tr -d ' '); fi
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

selection_json() { # <all|id,...> → "all" or a JSON id array
    if [ "$1" = all ]; then echo '"all"'; else echo "$1" | tr ',' '\n' | grep . | jq -R . | jq -sc .; fi
}
# Policy v3 (CONTEXT.md, 2026-09-02): leagues under realms, and per league
# the facets it covers — `tabs` and/or `characters`; an absent facet is no
# coverage of it (`none` leaves `tabs` out).
tabs_json='[]'
[ "$SELECTION" != none ] && tabs_json=$(selection_json "$SELECTION")
chars_json='[]'
[ -n "$CHARACTERS" ] && chars_json=$(selection_json "$CHARACTERS")
POLICY=$(jq -nc --arg r "$REALM" --arg l "$LEAGUE" --argjson t "$tabs_json" --argjson c "$chars_json" --argjson a "$MAX_AGE" \
    '{version: 3, realms: {($r): {leagues: {($l): ({max_age_seconds: $a}
        + (if $t != [] then {tabs: $t} else {} end)
        + (if $c != [] then {characters: $c} else {} end))}}}}')
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
    "$ACQ" refresh --plan --realm "$REALM" --league "$LEAGUE" >"$RUN_DIR/plan-$1-offline.txt" 2>&1 || {
        cat "$RUN_DIR/plan-$1-offline.txt" >&2; return 1; }
    "$ACQ" refresh --plan --realm "$REALM" --league "$LEAGUE" --json \
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
# The action block of an envelope — rendered from the file that will be
# applied, through the CLI's own renderer (`acq refresh --plan=FILE`: the
# grouped view, the same text a human reviewing the file would read), so
# what is confirmed is what goes out. The verdict line through the blank
# line before the quote.
render_actions() {
    "$ACQ" refresh --plan="$1" | awk '/^[0-9]+ requests?,/ { on = 1 } on && /^$/ { exit } on { print }'
}
# The quote block of a rendered plan (or the one-line reason there is none).
quote_block() {
    if grep -q '^quote (' "$1"; then sed -n '/^quote (/,/^next:/p' "$1" | grep -v '^next:'
    else grep -E '^(no quote|daemon quote)' "$1" || true; fi
}

echo ""
echo "the plan, compiled offline (no daemon is running; checked):"
offline_plan c1
cat "$RUN_DIR/plan-c1-offline.txt"
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
    clists=$(jq '[.actions[] | select(.action == "list_characters")] | length' "$plan")
    cfetches=$(jq '[.actions[] | select(.action == "fetch_character")] | length' "$plan")
    [ $((lists + fetches + subs + clists + cfetches)) = "$logical" ] || { echo "plan carries an action kind this driver does not count" >&2; exit 1; }
    wire_min=$(jq -r '.wire_sends.min' "$plan")
    wire_max=$(jq -r '.wire_sends.max' "$plan")
    [ "$wire_min" = "$logical" ] || { echo "plan's wire minimum $wire_min != logical $logical" >&2; exit 1; }

    if [ "$logical" = 0 ]; then
        # The closed loop: an empty plan applies as a no-op that contacts
        # nothing. No daemon is up, and none may appear.
        echo ""
        echo "cycle $c: the plan is empty — applying it with no daemon (must contact nothing):"
        "$ACQ" refresh --apply --realm "$REALM" --league "$LEAGUE" --json | tee "$RUN_DIR/apply-$tag-noop.json"
        [ "$(jq -r '.requests' "$RUN_DIR/apply-$tag-noop.json")" = 0 ] || { echo "no-op apply reported requests != 0" >&2; exit 1; }
        daemon_up && { echo "*** the no-op apply spawned a daemon" >&2; exit 1; }
        CLOSED=$c
        break
    fi

    # One free HEAD per route this lifetime: the stash list, the stash
    # fetch, the character list, the character fetch — each its own route
    # (and, off pc, its own realm-suffixed route).
    probes=0
    [ "$lists" -gt 0 ] && probes=$((probes + 1))
    [ $((fetches + subs)) -gt 0 ] && probes=$((probes + 1))
    [ "$clists" -gt 0 ] && probes=$((probes + 1))
    [ "$cfetches" -gt 0 ] && probes=$((probes + 1))
    ceiling=$((1 + probes + logical))
    pace=""
    if [ "$MODE" = live ]; then
        stash_gets=$((fetches + subs))
        # Duration of this cycle on the wire, over-estimated on purpose:
        # a ~343 s hold per 30 stash GETs and a ~15 s hold per 15 (rungs
        # 10 and 7b; 323 GETs took 61 min live) plus a minute of slack.
        # The character policy has the same windows (5:10, 30:300) and is
        # paced independently, so the two facets run side by side and the
        # cycle lasts as long as the longer one, not the sum.
        # The facts a cycle lands at its start are the oldest when the
        # NEXT plan compiles, so the window must outlive the cycle with
        # margin: refuse unless it is at least twice the estimate.
        gets=$stash_gets
        [ "$cfetches" -gt "$gets" ] && gets=$cfetches
        est=$(( (gets / 30) * 343 + (gets / 15) * 15 + 60 ))
        if [ $((est * 2)) -gt "$MAX_AGE" ]; then
            echo "*** cycle $c would take ~$est s on the wire but max_age_seconds is $MAX_AGE:" >&2
            echo "*** what it lands would be stale (or nearly) for the next plan, and the loop" >&2
            echo "*** could not close. Rerun with --max-age of at least $((est * 2)) (e.g. 86400)." >&2
            exit 2
        fi
        # The facts this plan leaves alone as fresh keep ageing while the
        # cycle runs: if the oldest of them plus the cycle's duration
        # exceeds the window, the NEXT plan refetches them — a refetch
        # cycle (seen 2026-09-02: 51 min old at start + a 13 min cycle
        # against 3600 s bought a 6-request cycle 2). Harmless, the loop
        # still closes, so this warns rather than refuses.
        oldest=$(jq -r --argjson now "$(date +%s)" '
            [ ((.skipped_tabs[], .skipped_characters[]) | select(.reason.kind == "fresh") | .reason.age_seconds),
              (if (.basis.stash_listing != null) and ([.actions[] | select(.action == "list_stashes")] | length) == 0
               then ($now - .basis.stash_listing.fetched_at) else empty end),
              (if (.basis.character_listing != null) and ([.actions[] | select(.action == "list_characters")] | length) == 0
               then ($now - .basis.character_listing.fetched_at) else empty end) ] | max // 0' "$plan")
        if [ $((oldest + est)) -gt "$MAX_AGE" ]; then
            echo "note: the oldest fact this plan keeps as fresh is ${oldest}s old and the cycle is ~${est}s;"
            echo "      together they pass the ${MAX_AGE}s window, so expect the next plan to refetch it (a refetch cycle, not a fault)."
        fi
        if [ "$gets" -gt 30 ]; then
            pace=" — more than 30 GETs on one policy: a ~15 s hold after each 15 and a ~343 s hold after each 30 (rung 10's shape), ~$est s in all; that is the limiter working"
        elif [ "$gets" -gt 15 ]; then
            pace=" — more than 15 GETs on one policy: expect one ~15 s limiter hold before the 16th (rung 7b); that is the limiter working"
        fi
    fi
    confirm "cycle $c: fresh daemon with ceiling $ceiling = 1 token POST + $probes probe HEAD(s) + $logical GET(s) ($lists stash listing, $fetches tab, $subs substash, $clists character listing, $cfetches character; plan says $wire_min..$wire_max wire sends)$pace"
    cycle_offset=$(journal_size)
    PID=$(spawn_daemon "$ceiling" "daemon-$tag.out")
    echo "daemon $tag: pid $PID (ceiling $ceiling)"

    echo ""
    echo "the same plan, now with the daemon up (quote attempt; nothing is sent) — its quote:"
    "$ACQ" refresh --plan --realm "$REALM" --league "$LEAGUE" >"$RUN_DIR/plan-$tag.txt" 2>&1
    quote_block "$RUN_DIR/plan-$tag.txt"
    "$ACQ" refresh --plan --realm "$REALM" --league "$LEAGUE" --json \
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
    echo "the envelope about to be applied ($RUN_DIR/plan-$tag.json), rendered from that file:"
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
if [ "$SELECTION" != none ]; then
"$ACQ" tabs --realm "$REALM" --league "$LEAGUE" >"$RUN_DIR/tabs.txt"
"$ACQ" tabs --realm "$REALM" --league "$LEAGUE" --json >"$RUN_DIR/tabs.json"
fi
if [ "$SELECTION" = none ]; then
    echo "(no tab coverage in this run)"
elif [ "$SELECTION" = all ]; then
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
    # Children of a selected parent are covered through it: every one on
    # record must have been fetched by the time the loop closed, except a
    # substash stub the final plan skips as empty (nothing to fetch).
    children=$(jq -c --argjson sel "$tabs_json" '[.[] | .parent as $p | select($p != null and ($sel | index($p)) != null)]' "$RUN_DIR/tabs.json")
    n_children=$(echo "$children" | jq 'length')
    n_unfetched=$(echo "$children" | jq '[.[] | select(.fetched_at == null)] | length')
    echo "children of selected tabs on record: $n_children (substashes and folder children), $n_unfetched never fetched"
    if [ "$CLOSED" != 0 ] && [ "$n_unfetched" -gt 0 ]; then
        "$ACQ" refresh --plan --realm "$REALM" --league "$LEAGUE" --json >"$RUN_DIR/plan-final.json" 2>/dev/null || true
        n_empty=$(jq '[.skipped_tabs[]? | select(.reason.kind == "empty_stub")] | length' "$RUN_DIR/plan-final.json" 2>/dev/null || echo 0)
        if [ "$n_unfetched" != "$n_empty" ]; then
            echo "*** $n_unfetched covered child(ren) never fetched but only $n_empty skipped as empty stubs — the loop closed with covered work undone" >&2
            exit 1
        fi
        echo "(all $n_unfetched are empty stubs the final plan skips — nothing to fetch)"
    fi
fi
if [ -n "$CHARACTERS" ]; then
    # The character facet's readback: every covered character on record in
    # this (realm, league) must have been fetched by the time the loop
    # closed, unless the final plan skips it for a reason that never
    # fetches (deleted, expired, no league). Coverage is exact by id.
    "$ACQ" store characters --realm "$REALM" >"$RUN_DIR/characters.txt"
    "$ACQ" store characters --realm "$REALM" --json >"$RUN_DIR/characters.json"
    head -20 "$RUN_DIR/characters.txt"
    if [ "$CHARACTERS" = all ]; then
        covered=$(jq -c --arg l "$LEAGUE" '[.[] | select(.league == $l)]' "$RUN_DIR/characters.json")
    else
        covered=$(jq -c --argjson sel "$chars_json" '[.[] | select(.id as $i | $sel | index($i) != null)]' "$RUN_DIR/characters.json")
        unknown=$(jq -r '.unknown_characters | join(" ")' "$RUN_DIR/plan-c1-offline.json")
        for id in $(echo "$CHARACTERS" | tr ',' ' '); do
            if [ "$(echo "$covered" | jq --arg i "$id" '[.[] | select(.id == $i)] | length')" = 1 ]; then continue; fi
            case " $unknown " in
            *" $id "*) echo "$id: not in the store — the first plan reported it as unknown (a typo, or a character that is gone); never fetched" ;;
            *) echo "*** $id: selected, planned, but not in the store's character list after the run" >&2; exit 1 ;;
            esac
        done
    fi
    n_covered=$(echo "$covered" | jq 'length')
    n_unfetched=$(echo "$covered" | jq '[.[] | select(.fetched_at == null)] | length')
    echo "covered characters on record: $n_covered, $n_unfetched never fetched"
    if [ "$CLOSED" != 0 ] && [ "$n_unfetched" -gt 0 ]; then
        "$ACQ" refresh --plan --realm "$REALM" --league "$LEAGUE" --json >"$RUN_DIR/plan-final.json" 2>/dev/null || true
        n_never=$(jq '[.skipped_characters[]? | select(.reason.kind == "deleted" or .reason.kind == "expired" or .reason.kind == "no_league")] | length' "$RUN_DIR/plan-final.json" 2>/dev/null || echo 0)
        if [ "$n_unfetched" != "$n_never" ]; then
            echo "*** $n_unfetched covered character(s) never fetched but only $n_never skipped as deleted/expired/no-league — the loop closed with covered work undone" >&2
            exit 1
        fi
        echo "(all $n_unfetched are skips the final plan never fetches — deleted, expired, or no league)"
    fi
fi
"$ACQ" store status 2>&1 | tee "$RUN_DIR/store-status.txt"
# Everything since the run started (plus an hour of margin), not a fixed
# hour: an `all` cycle alone is longer than that. The CLI's default
# limit (200, oldest first) would silently drop a big run's later
# events, so the limit is explicit and hitting it is a failure.
hours=$(python3 -c "import time; print((time.time() - $RUN_START) / 3600 + 1)")
EVENT_LIMIT=1000000
"$ACQ" store events --hours "$hours" --limit "$EVENT_LIMIT" >"$RUN_DIR/store-events.txt"
"$ACQ" store events --hours "$hours" --limit "$EVENT_LIMIT" --json >"$RUN_DIR/store-events.json"
events_n=$(jq 'length' "$RUN_DIR/store-events.json")
if [ "$events_n" -ge "$EVENT_LIMIT" ]; then
    echo "*** the item-event readback hit its limit ($EVENT_LIMIT): evidence would be truncated" >&2
    exit 1
fi
echo "item events since the run started: $events_n, in $RUN_DIR/store-events.txt"
"$ACQ" refresh --plan --realm "$REALM" --league "$LEAGUE" >"$RUN_DIR/plan-final.txt" 2>&1
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

# The evidence is this run's slice of the journal and of the daemon log
# (both files are cumulative on disk), plus the verifier as it was when
# the run was checked — copied into the bundle with its checksum — and
# the verification runs on the saved journal from byte 0 through that
# copy, so re-running verify.sh later reproduces the verdict exactly,
# wherever the bundle lives and whatever the working tree's verifier
# becomes.
tail -c +$((OFFSET + 1)) "$JOURNAL" >"$RUN_DIR/sends.jsonl"
if [ -f "$LOG" ]; then tail -c +$((LOG_OFFSET + 1)) "$LOG" >"$RUN_DIR/daemon.log"; fi
cp "$here/tools/tracer-verify.py" "$RUN_DIR/tracer-verify.py"
(cd "$RUN_DIR" && shasum -a 256 tracer-verify.py sends.jsonl cycles.tsv >checksums.sha256)
cat >"$RUN_DIR/verify.sh" <<EOS
#!/bin/sh
# Re-verify this bundle: $(basename "$RUN_DIR"), binary $ver, rung tip $tip.
# Uses the verifier copied into the bundle, never the working tree's.
cd "\$(dirname "\$0")" && shasum -a 256 -c checksums.sha256 >/dev/null &&
    python3 ./tracer-verify.py sends.jsonl 0 cycles.tsv $LOGIN_LIFETIME $CLOSED $MODE $REALM
EOS
chmod +x "$RUN_DIR/verify.sh"

# The full per-send form is the evidence (`summary.txt`, what verify.sh
# reproduces); the terminal gets the brief form — one line per lifetime
# with its totals, probe verdicts, and the limiter holds it saw — and the
# verdict is the full run's.
verify() { # [brief]
    python3 "$RUN_DIR/tracer-verify.py" "$RUN_DIR/sends.jsonl" 0 "$CYCLE_ROWS" "$LOGIN_LIFETIME" "$CLOSED" "$MODE" "$REALM" ${1:+brief}
}
if verify >"$RUN_DIR/summary.txt"; then
    verify brief
else
    cat "$RUN_DIR/summary.txt"
    echo ""
    echo "*** verification FAILED — see above; evidence in $RUN_DIR" >&2
    exit 1
fi
echo "(per-send detail in $RUN_DIR/summary.txt)"

echo ""
echo "evidence in $RUN_DIR (this run's journal and daemon-log slices, plans, apply results,"
echo "store reads, summary, the verifier copy and checksums; ./verify.sh re-runs the verification)."
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
