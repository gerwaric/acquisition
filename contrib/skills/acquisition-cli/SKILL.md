---
name: acquisition-cli
description: Inspect the inventory published by a running Acquisition application and manage application-owned Path of Exile refreshes through acquisitionctl. Use when an agent needs Acquisition status, tabs, characters, normalized items, effective prices, or refresh progress/results; when asked to refresh Acquisition without canceling work on agent disconnect; or when diagnosing acquisitionctl JSON and exit statuses.
---

# Acquisition CLI

Use `acquisitionctl`; do not read Acquisition databases, settings, OAuth tokens,
or POESESSID values directly. The open GUI owns network access, rate limiting,
persistence, and refresh jobs.

Before issuing commands, resolve the executable once and substitute its path for
`acquisitionctl` below:

1. Use an explicit path supplied by the user or the shell's `ACQUISITIONCTL`
   variable.
2. If the GUI is running, prefer its release-matched CLI: on macOS and Windows,
   derive the sibling `acquisitionctl` or `acquisitionctl.exe` from the running
   GUI executable; on Linux, look beside the running GUI AppImage for the
   separately downloaded `acquisitionctl-*.AppImage`.
3. Next try `acquisitionctl` on `PATH`, then `build/acquisitionctl` for a local
   build.
4. For an inactive macOS installation, try
   `/Applications/acquisition.app/Contents/MacOS/acquisitionctl` and the same
   path under `~/Applications`.
5. For an inactive Windows installation, resolve the target of its
   `acquisition` Start Menu shortcut and use that directory.
6. On Linux, also check `~/Applications` for the CLI AppImage and ensure it is
   executable.

If none exists, tell the user which CLI artifact is missing rather than reading
private application files as a fallback.

Pass `--data-dir <path>` when Acquisition uses a non-default data directory.
Every successful command except `--help` and `--version` writes one JSON
envelope to stdout. Those two informational options print plain text. Treat
stderr as a human diagnostic only.

## Inspect state

Start with:

```sh
acquisitionctl status --json
```

Check `result.service_state` before inventory commands:

- `needs_login`: ask the user to finish login in the GUI.
- `loading_cache`: wait and query status again.
- `ready`: inventory commands are available.

`refresh_state` describes the worker; `active_refresh_id` identifies a refresh
started through the CLI. Inventory responses pair `instance_id` with
`inventory_revision`; compare both, never revision alone.

## View inventory

List bounded location pages:

```sh
acquisitionctl tabs --limit 100 --json
acquisitionctl tabs --cursor '<next_cursor>' --json
```

Follow `next_cursor` until null. As with item pages, discard a partial tab
traversal and restart if the instance or revision changes.

Fetch bounded item pages:

```sh
acquisitionctl items --limit 50 --json
acquisitionctl items --cursor '<next_cursor>' --json
```

For a location filter, always provide both parts of the typed identity:

```sh
acquisitionctl items --tab '<id>' --kind stash --limit 50 --json
acquisitionctl items --tab '<id>' --kind character --limit 50 --json
```

Fetch one stable item id:

```sh
acquisitionctl item '<item-id>' --json
```

Use `effective_price`; do not infer price by parsing `note`. Item locations
include display `id` and `fetch_source_id` because special-tab children refresh
independently while displaying under a parent.

To consume every page:

1. Save the first response's `instance_id` and `inventory_revision`. Cursors
   are application-authenticated and must never be decoded or modified.
2. Process `result.items`; a sparse filter may produce an empty page with a
   non-null cursor because each request has a source-scan bound.
3. Repeat with only `--cursor` while `next_cursor` is non-null.
4. On `revision_changed`, discard the partial traversal and restart at page one.

A refresh updates published state incrementally. If the task requires a stable
post-refresh view, wait for its terminal operation before starting pagination.

## Refresh safely

A refresh performs network work and follows the GUI's existing behavior,
including already-enabled automatic shop updates. Start one only when the user
has requested or clearly authorized a refresh.

Start and retain the operation id:

```sh
acquisitionctl refresh start --json
```

The application owns accepted work. Closing the terminal, ending the agent
session, or losing the client connection does not cancel it. The CLI retries an
ambiguous start once with the same idempotency key. If it returns
`start_unconfirmed`, retain `error.operation_id` and query that id before ever
issuing another start.

Prefer recoverable polling for long refreshes:

```sh
acquisitionctl refresh status '<operation-id>' --json
```

Use `refresh wait` only when blocking is appropriate:

```sh
acquisitionctl refresh wait '<operation-id>' --timeout 300 --json
```

A wait timeout stops observation only; the refresh continues. Query the same id
later. Never repeatedly call `refresh start` to poll.

Interpret terminal outcomes:

- `state=completed`, `outcome.clean=true`: clean completion.
- `state=completed`, `outcome.clean=false`: completion with structured skipped
  sources; do not describe it as fully synchronized.
- `state=failed`: terminal failure with a structured error.

Refresh completion covers inventory refresh, not completion of asynchronous
forum posting.

## Exit statuses

- `0`: command succeeded; `refresh wait` completed cleanly.
- `1`: the Qt command-line parser rejected an unknown or malformed option.
- `2`: invalid command combination or structured service error.
- `3`: Acquisition is not running for that data directory.
- `4`: transport or protocol failure; `start_unconfirmed` includes the
  operation id that must be queried before another start.
- `5`: refresh busy/not accepted, or `refresh wait` observed failure.
- `6`: `refresh wait` completed with skipped sources.
- `7`: observation timeout; the refresh continues.

For service and client responses, inspect the JSON body; exit status is only a
control-flow aid. Parser failures (exit `1`) and locally rejected command usage
(exit `2`) may have empty stdout, so read their stderr diagnostic instead.
