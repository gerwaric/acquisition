# The daemon split — open record

The daemon split makes the daemon its own artifact (`acqd`) and draws
the frontend boundary as crates and a wire: `acquisition-protocol`,
`acquisition-client`, `acquisition-daemon`, the world in the store, two
identities in the handshake. Opened 2026-09-10 at `10f9bab5`, in the
mold of `REFRESH-SLICE.md` and `PRICING-SLICE.md`; while the slice is
open this is where its findings and observations land, and at closure
it is cut to the permanent short record.

The design is `brainstorming-notes/18-the-daemon-split.md`, ruled by the
owner on 2026-09-09 (its §10, verbatim): §2 the boundary, §3 the ruled
lines (C1 amended, C82–C85 new, C31 amended), §4 the commit sequence, §5
the parking lot. This file never restates it. The rulings become real
in the registry as each commit lands (§10's landing map): C85 with
commit 0, C1 and C82 with the `acqd`/client split, C84 with the
identity, C83 and C31 with the world. Until then the packet is the
contract the code is held to (P1).

## Step ledger

One row per commit of §4; a row is filled when the commit lands.

| Step | Commits | What landed |
| --- | --- | --- |
| −1 restore green | `93ed626c` | the pricing property's inverse grammar fixed; the regression seed committed; gate green |
| 0 wire audit and pin | — | stable `hello`/`daemon_stop` plane; frame bound with `bad_request`; C85's semantics (`Subscription`, `resync_required { missed }`, subscribe-then-snapshot); closed `ErrorKind`; fixtures; contract tests; C85 in the registry |
| 1 protocol crate | — | `acquisition-protocol` extracted; the build script moved with today's input set |
| 2 consumers | — | plan, cli, mcp and the tests on the protocol crate; docs-check edges |
| 3 `acqd` and client | — | `acquisition-client`; core → `acquisition-daemon`; the `acqd` binary; `daemon run` gone; the sibling locator; the gate builds before it tests; C1 and C82 in the registry |
| 4 identity | — | shared-contract revision; artifact identity and hash; `hello`, `DaemonId`, `acq version`, journal header, `provenance.json` with both hashes; the artifact-mismatch test; C84 in the registry; standing-rule prose presented |
| 5 world | — | `world.rs`; rails state into the world, diagnostics bounded; the world lock and the real-mode lock; `hello` carries the world; C83 and the C31 amendment in the registry |
| 6 rendezvous | — | the socket derived into the runtime directory; `ACQ_SOCKET` removed, `tools/acq-as.sh` retired; legacy detection; the migration test |
| 7 live | — | both drivers in mock; the tracer under the rails; ledger row; `provenance.json` with `acqd`'s hash |

## Findings

One row per review round; the finding, then the property or test that
holds it now. The packet's own review rounds are its §8 and §9.

| Round | Commit | Findings | Held by |
| --- | --- | --- | --- |

## Observations still open

Agent observations that became neither a ruling nor a finding; each is
data for the commit that touches it.

- The rails state today sits beside the socket in the per-user temp
  directory, which macOS clears at reboot: a tripped tripwire does not
  survive a restart (packet §1, verified). Closed by step 5.
- C85's *Pinned* paths name the client crate, which exists from step 3;
  at step 0 the contract tests and fixtures live under
  `acquisition-core/tests/` and the registry entry names them there.
  Step 3 moves them and edits the pointer.
- The `ErrorKind` vocabulary is the step-0 audit's to determine under
  the owner's five properties (packet §10, ruling 6); a materially new
  semantic distinction comes back to the owner before landing.
- Lag is stageable in-process: the daemon's broadcast channels have
  capacity 256.
