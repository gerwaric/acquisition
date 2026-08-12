# Tokio actor slice hand-off

Status: **slice closed 2026-08-12** — Tom's re-review closed with no findings,
after the re-review's own findings were fixed in `02b60f47`. This slice
consumes the reviewed core and mock handoffs; it does not claim final M-series
verdicts. §3's "not yet covered" list is the live one: read it before starting
the next slice.

## 1. Silences taken

| Silence | Conservative reading | Next-call consequence |
|---|---|---|
| The core hand-off carries an open timeout obligation but names no duration. | `TRANSPORT_TIMEOUT = 30 s`, below the smallest padded N23 window horizon (10 s period + 60 s configured bucket); timeout is `on_unknown_outcome`, never rollback. | The next reservation remains pessimistically counted until window passage; the endpoint mapping stays established, so no extra HEAD is sent. |
| The actor queue and number of dynamic endpoint records lack a bound. | Cap pending requests at 10,000 and endpoint records at D5's five labels. | Overflow gets a typed error before storage growth; the next valid submission can still enter when queue space exists. |
| C4 says the 4xx tripwire has the fuse's “same shape” but gives no independent thresholds. | Use C3's reviewed burst/sustained thresholds (11/1 s and 500/60 s) for the 4xx counter as the least inventive literal reading. | Every 4xx is still counted; crossing either threshold halts, errors queued callers, and stops new dispatches. |
| The core’s `Blocked` outcome has no timer, and a constructed zero-hit policy can be permanently blocked without a live request to resolve it. | Error one queued caller rather than sleeping or inventing a retry time; live confirmation blocks wait for their in-flight outcome. | A malformed constructed policy cannot wedge the actor; a normal confirmation block is reconsidered after its response/timeout changes core state. |
| The docs set no actor ingress limit for a response. | `Transport::send` returns a private-field `WireResponse`; its 16 KiB / 32-header / 256-name-byte / 1,024-value-byte constructor is the sole actor ingress. Raw `Response<Vec<u8>>` cannot enter the actor. **Accepted X2 limitation:** this spike cannot force a future HTTP parser not to allocate upstream; that implementation must cap while collecting bytes. | An over-bound response is a transport error before it enters the actor; ordinary work resolves as unknown and the next probe enters D4 cooldown. |
| The docs do not say whether a renamed policy name can already name a distinct established policy. | Refuse the colliding ordinary observation without changing either policy route; never merge two histories by guesswork. | The caller and queued requests for the original policy receive scoped setup failure; the unrelated policy remains schedulable. |

The timeout, C4 threshold, actor queue-cap, and wire-ingress-bound items are doc findings to
carry into `result-draft.md`; D4 itself supplies the 60 s cooldown.

## 2. Seam map and invariant walk

- `PolicyEngine` and all reservation tokens are owned only by `Actor`; spawned
  transport tasks return wire outcomes and cannot schedule or consume tokens.
- Endpoint state is actor-owned: `Unknown → Probing → Established`, with a D4
  cooldown on failed probes. The explicit deque, not the mpsc mailbox, owns
  request order.
- Mock-only B13 correlation injection is opt-in through a request extension;
  the actor issues distinct IDs for HEAD and GET hand-offs.
- The scheduling turn drains already-received commands before making a permit
  decision. A later unknown endpoint is therefore a writer even if an earlier
  established GET is at the deque front; the first pending writer runs as soon
  as current readers drain.
- M5 keeps a stable internal policy anchor behind the server-visible route.
  Remapping replaces only the route and rule judgments, so every in-flight
  token still removes or reconciles its exact history entry. The actor applies
  `Notification::Remapped` to every endpoint sharing that established policy.
- Every ordinary refusal resolves its `RefusalTarget::Policy` from that anchor's
  current route, not the anchor spelling. `fail_policy` therefore drains the
  queued callers of a remapped endpoint on malformed/retry/escalation paths.

1. **No permanent wedge:** queued drops are pruned; a reservation that does
   not reach a transport task rolls back;
   wire timeout resolves as unknown; a no-timer `Blocked` state cannot retain a
   queue entry without a live request capable of changing it.
2. **One send, one entry:** only `start_ordinary` receives a reservation; it
   either stores it with exactly one transport task or rolls it back before
   hand-off. Every completion consumes it through one core entry point.
3. **Pessimism direction:** post-dispatch errors and timeouts use
   `on_unknown_outcome`; cancellation only drops the caller, never the
   dispatched work.
4. **Single scheduling authority:** the actor is the only caller of
   `try_reserve`; spacing, D5 permits, and core `NotBefore` only decide when
   this same loop asks it again.
5. **Entry-point invariant:** ordinary completions call `on_response`; probes
   call `on_probe_response`; unexpected cross-lane dispositions halt rather
   than becoming a send promise.
6. **Truthful notifications:** queue/permit/endpoint/halt mutations publish a
   watch snapshot; core `StateChanged` is interpreted as an additional publish
   cue, never fabricated into a core transition.

## 3. Coverage confession

Covered: paused-time boot HEAD followed by spaced GET, unique B13 IDs,
queued and dispatched cancellation (the latter still lands at the mock),
degraded-HEAD D4 failure, Cloudflare-shaped terminal classification/watch
publication, the fuse's exact burst/sustained boundaries and floor-compliant
property, all C4 thresholds/edges, X1 burst/sustained actor-boundary fault
injection, M5 remap including post-remap malformed queue drain, M6 same-policy
limit shrink with held history, M8 retry and escalation, and M13
forced-delay writer preference, HEAD exclusivity, FIFO, and in-flight cap
behavior, plus all pre-existing core/mock suites.
Header-count/name/value and body n/n+1 boundaries are unit-pinned at the
transport constructor before clone/scan.

Not yet covered: full M1–M13 scenario-driver runs; caller *drop* while
dispatched (explicit cancellation is covered); and scenario driver/judge
integration. The C3 floor property asserts on every
generated trace and cannot pass vacuously: each generated dispatch is checked
against independent timestamp-window arithmetic before it checks the counter
under test.

## 4. Judgment calls

- A bounded mpsc channel is ingress back-pressure only; the actor's deque is
  separately capped and remains the inspectable/reorderable queue.
- Correlation identity is opt-in request metadata rather than a production
  header: conformance uses `with_correlation_header`, while ordinary transport
  traffic is unchanged.
- A join-task panic halts and resolves all active tokens as unknown rather than
  guessing which bytes reached the wire. This is throughput-pessimistic but
  preserves schedulability and the reservation invariant.
