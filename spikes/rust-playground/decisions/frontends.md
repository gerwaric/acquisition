# Decisions — Frontends and output

Part of the decision registry (`CONTEXT.md`, "Decisions — the registry"): the rulings that bind the CLI, the MCP server, `dash`, rendering, `--json`, agent-boundary defaults (`acquisition-cli`, `acquisition-mcp`). Read this before touching that code; the cross-cutting decisions and the invariants are in `CONTEXT.md` and are always loaded. Same rules as there: one entry per decision, stable `C<n>` ids never reused, the ruling verbatim, *Why:*, and pointers; the mechanism lives in the code's doc comments under the id.

## Frontends and output

- **C11 — CLI emits structured output** (`--json` on every command, the error path included: failures are `{"error": …}` on stdout with exit 1, and a failed job exits 1 in both modes). *Why:* the CLI is itself an API; makes MCP and agent use nearly free.
- **C13 — The MCP server is a fourth thin client (`acquisition-mcp`, binary `acq-mcp`, official `rmcp` SDK over stdio), never in-process with the daemon.** It never kills or replaces a daemon and never spawns one in real mode (login is human, via the CLI); it lazy-spawns only in mock mode. *Why:* daemon-hosted queries make the daemon an application server. *Details:* `acquisition-mcp/src/main.rs` doc, C13. *Pinned:* `tests/plan_loop.rs`, `tests/ggg_refusal.rs`. Decided 2026-08-30.
- **C16 — Tauri for GUI** (webview frontend). *Why:* item search/grid/filter UIs are a strength of web tech; egui considered and passed on for data-heavy views.
- **C52 — An agent never clobbers intent it has not read.** `set_sync_policy` has no blind-replace form: replacing an existing policy must name the revision it replaces. Owner-revisable default, 2026-09-01. *Pinned:* `acquisition-mcp/tests/plan_loop.rs` (create-only CAS).
- **C53 — Legible output has three levels: default text is the decision view, `--expand` is the audit view, and JSON is the contract.** Text remains a function of JSON; grouping is presentation, never authorization. Default text states repeated context once, omits prose and provenance that do not change the next decision, opens with one answer before detail, and ends with the next action. Ten or fewer entities are listed, more counted; failures name target, cause and evidence; every nothing says which nothing; ages are text, epochs JSON; escapes only on a tty; JSON changes are additive. Ruled 2026-09-02; amended 2026-09-03 after the live density reading (tab ids at right, pinned). *Details:* `plan_cmd.rs` doc, C53. *Pinned:* the `c53_` tests. *Evidence:* ledger row 2026-09-03.

## Parked (do not build yet; each with its trigger)

Scope this area has deferred, with the trigger that reopens it, so deferral never needs re-arguing. A fired trigger deletes its entry in the build's commit; what crosses every area is parked in `CONTEXT.md` instead. An entry is the item, where it lands, and the trigger that reopens it, with at most one clause of why; a workaround is a README known gap, history is git.

- Queue-management UI (drag-to-reorder, per-job progress bars) → a rendering problem by construction. Trigger: the GUI.
- The read economy as a ruling (summary by default, filters, bounded detail) → surface design under C53 until then. Trigger: the first MCP pricing consumer's re-read record (the pricing slice's MCP step).
- Results over a subtree (today `list`, then `result` per child: hundreds for a real map tab) → results on the event channel as jobs finish, or a `results` verb. Trigger: a second consumer (GUI or MCP) showing which. Frontend boundary finding from `acq pull`, 2026-08-24.
