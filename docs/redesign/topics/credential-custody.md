# Credential custody (OAuth + POESESSID)

Status: investigation result, unreviewed
Provenance: researched 2026-08-08 (Claude, Tom's session). Code claims
are traced against the working tree on the `redesign` branch; all
`file:line` cites are to that tree. External claims were researched
the same day by web-research subagents and carry the **external** lane
with URL and retrieval date (2026-08-08 throughout). This continues
`topics/shop-write-path.md` §6, which settled the POESESSID side; this
doc settles the OAuth side and the target security model. Scope guards
were honored: no live authentication attempts, no pathofexile.com
requests beyond public developer-docs pages, and no stored credential
values were read or quoted.

## The answer in one paragraph

Credential custody confines cleanly to a future core. Every credential
consumer is core-side under the A0 split: both POESESSID consumers
(shop-write-path §6) and the OAuth bearer, which attaches at a single
chokepoint (`networkmanager.cpp:115-123`) with credential-free request
builders everywhere else. The current OAuth flow is already the
RFC 8252 native-app pattern — system browser plus loopback listener —
which runs entirely in a Tauri host process with the webview
uninvolved, and GGG's documented public-client constraints (PKCE S256,
local redirect URI, 7-day refresh tokens) are all satisfiable from
Rust with the `oauth2` crate. The current guarantees are enumerable
(§2): strong in flight, weak at rest (plaintext in two files plus
accumulating version backups), and leaky in logs (three leak-on-error
or trace-level paths, candidate findings below). A Tauri-style model
meets the bar and exceeds it cheaply: OS keychain via `keyring`
(~10-line API, actively maintained), host-scoped auth pinned in one
HTTP client instead of inherited as convention, and a command surface
where no secret ever crosses IPC outbound. One empirical question
survives research and becomes the spike (§7): whether GGG's server
accepts the same flow from a non-Qt implementation under the existing
`acquisition` registration.

## 1. OAuth custody today (traced)

### At rest

- The full token — access token, refresh token, username, sub, scope,
  expirations — is serialized to JSON and stored plaintext in the
  **global** SQLite datastore: `data` table, key `oauth_token`
  (`oauthmanager.cpp:146-155`, `sqlitedatastore.cpp:40`,
  `sqlitedatastore.cpp:121-131`). The file is
  `<data-dir>/data/<md5("|")>` — the account-independent datastore
  (`application.cpp:202-204`, `sqlitedatastore.cpp:168-190` with empty
  username/league). One row, last-authenticated-user-wins.
- POESESSID is plaintext in `settings.ini` under `session_id`
  (`application.cpp:355`), as shop-write-path §6 recorded.
- **Backups multiply the copies.** On every version change,
  `SaveDataOnNewVersion` copies the whole `data/` directory — token
  file included — into `data-backup-<version>[-n]`
  (`application.cpp:441-508`). Backups are never pruned, so plaintext
  refresh tokens accumulate on disk (bounded in usefulness by the
  7-day refresh lifetime, but access tokens in a fresh backup are
  live for up to 10 hours).
- **Clearing.** The only path that clears the stored token is the
  "Remember me" checkbox being unchecked when the login dialog is
  destroyed: settings are cleared (dropping `session_id`) and the
  `oauth_token` row is overwritten with an empty string
  (`logindialog.cpp:141-157`). There is no logout action anywhere
  else; POESESSID additionally clears on rejection via
  `Application::ClearSessionId` (`application.cpp:343-347`).

### In memory

Plain heap strings, no zeroization or memory locking: the token lives
in `OAuthManager::m_token`, inside Qt's
`QOAuth2AuthorizationCodeFlow`, as the pre-formatted `Bearer …` byte
array in `NetworkManager::m_bearerToken` (`networkmanager.cpp:99-103`),
and as a copy in `LoginDialog::m_current_token`
(`logindialog.cpp:350`). POESESSID sits in the default in-memory
cookie jar. A same-user process can read all of this with OS debugging
APIs — true of any native app, and (per §4) unchanged by Tauri.

### In flight

- **Authorization** uses the system browser:
  `QAbstractOAuth::authorizeWithBrowser` is connected to
  `QDesktopServices::openUrl` (`oauthmanager.cpp:95`); no login page is
  ever embedded. The redirect lands on a loopback listener:
  `QOAuthHttpServerReplyHandler` on host `127.0.0.1`, path
  `/auth/path-of-exile` (`oauthmanager.cpp:30-31`,
  `oauthmanager.cpp:58-61`). No port is specified, and the Qt
  constructor "Calls listen() with port 0 and address LocalHost" —
  i.e. an **OS-assigned random port per run** (external:
  https://doc.qt.io/qt-6/qoauthhttpserverreplyhandler.html, retrieved
  2026-08-08). The listener is opened in `initLogin`
  (`oauthmanager.cpp:185-192`) and closed on grant
  (`oauthmanager.cpp:161`), so it runs only while a login is in
  progress.
- **Port-flexibility evidence (inferred).** Because the port is random
  per run, the redirect URI GGG receives differs on every login, and
  logins have worked in production for years. GGG's server therefore
  evidently implements RFC 8252 §7.3-style port-agnostic loopback
  matching for this client — a fact GGG's docs are silent on (§5) and
  which the exact registered redirect URI (known only to Tom's
  registration records) could confirm.
- **Token exchange and refresh** POST to
  `https://www.pathofexile.com/oauth/token` over TLS through the
  shared `NetworkManager` (`oauthmanager.cpp:24`,
  `oauthmanager.cpp:74-76`). The `setModifyParametersFunction` hack
  removes an *empty* `client_secret` parameter during refresh because
  GGG's server rejects it ("as of 3.26", `oauthmanager.cpp:84-92`) —
  useful evidence that GGG expects the parameter absent, not empty,
  for public clients.
- **The bearer attaches at one chokepoint**: `createRequest` adds
  `Authorization` only when the host is exactly
  `api.pathofexile.com` (`networkmanager.cpp:115-123`). Request
  builders are credential-free (`poe_utils.cpp:22-29`; the rate
  limiter deliberately knows nothing about OAuth,
  `ratelimiter.h:177-180`), so there is no second path a token can
  take onto the wire.
- **The POESESSID cookie is NOT host-scoped.** It is installed
  domain-wide on `.pathofexile.com` (`networkmanager.cpp:93-96`), and
  standard cookie domain-matching sends it to every
  `*.pathofexile.com` host — the intended consumers on `www.` (legacy
  index, forum), but also `api.pathofexile.com` and the OAuth token
  endpoint (inferred: from the cookie domain plus
  `QNetworkCookieJar`'s suffix matching; not verified with a packet
  capture). The brief's expected guarantee "cookie never sent to
  `api.`" **does not hold today** — the API evidently tolerates the
  cookie. Candidate finding 4.

### Refresh lifecycle and failure

- Auto-refresh is on with a 300 s lead (`oauthmanager.cpp:81-82`);
  each refresh re-persists the token and swaps the bearer
  (`oauthmanager.cpp:141-157`). Refresh-token lifetime is hard-coded
  at 7 days for a public client (`oauthtoken.cpp:12-14`), matching
  GGG's docs (§5).
- At startup an existing row triggers `refreshTokens()` immediately
  (`oauthmanager.cpp:103-110`) — the persisted **access token is never
  replayed from disk**; only the refresh token is used.
- **Failure is logged and nothing else.** `requestFailed` and
  `serverReportedErrorOccurred` land in slots that only write log
  lines (`oauthmanager.cpp:113-139`). No de-arming, no credential
  clearing, no UI event: mid-session, an expired or revoked bearer
  surfaces as per-request 401 `FetchError`s (no OAuth-side 401
  classification exists anywhere — grep-verified; the only 401/403
  handling is the shop's POESESSID path). This is **asymmetric with
  the POESESSID guard**, which de-arms automation, clears the
  credential, and notifies (shop-write-path §2). `grantAccess` is
  consumed only by `LoginDialog` (`logindialog.cpp:126-129`); after
  login nothing listens to OAuth state at all.
- Dead state: `m_authenticated` is initialized false and never
  written; `isAuthenticatedChanged` is emitted but connected nowhere
  (`oauthmanager.h:38`, `oauthmanager.h:50`). Candidate finding 6.

### Logging

- POESESSID is masked in its own log line
  (`networkmanager.cpp:87-91`). The network capture file is
  credential-clean by construction: reply-side allowlist of
  `x-rate-limit*`/`retry-after`/`date` headers only, no request
  headers, no bodies (`networkcapture.cpp:84-92`).
- **The raw bearer token is written to the log at trace level**
  (`networkmanager.cpp:121`), and the log level is user-selectable in
  the login dialog, trace included (`logindialog.cpp:468-473`).
  Candidate finding 1.
- **The Authorization mask in `logHeaders` can never fire**:
  `networkmanager.cpp:172` compares `name` — the "request"/"reply"
  label — against "Authorization" instead of comparing `header`.
  Latent today: the call sites (`ratelimiter.cpp:431-435`,
  `ratelimitmanager.cpp:515-516` and siblings) log pre-send requests —
  the bearer is added to a *copy* inside `createRequest` — and reply
  headers, so no Authorization header currently flows through. Any
  future post-send request log would leak unmasked. Candidate
  finding 2.
- **Token bytes can reach the error log via `glz::format_error`**,
  which embeds source-buffer context: on token serialization failure
  (`oauthmanager.cpp:148-151`) and on token parse failure
  (`json_readers.cpp:44-48` via `readOAuthToken`). Inferred from
  glaze's context-printing behavior; not reproduced. Candidate
  finding 3.
- The username is logged at info level (`oauthmanager.cpp:107`,
  `oauthmanager.cpp:144`) — identity, not secret.

## 2. Current guarantees — the bar a future core must meet

Everything here is traced-in-code unless marked. **Held today:**

- **CG1.** The OAuth bearer is sent only to `api.pathofexile.com`,
  attached at a single chokepoint; request builders and the rate-limit
  machinery never see it.
- **CG2.** The bearer never reaches `www.pathofexile.com` — required,
  since it actively breaks the legacy and forum regimes (N21, N22).
- **CG3.** Authorization uses the system browser; no login UI is
  embedded; the loopback listener binds 127.0.0.1 only and only while
  a login is in progress; PKCE S256; no client secret exists in the
  binary (public client).
- **CG4.** POESESSID rejection is a de-arming terminal event: cookie
  and setting cleared, automation disabled, user notified — classified
  from 200-status HTML as well as 401/403 (shop-write-path §2,
  commit 8b761e2c).
- **CG5.** POESESSID is masked in log output; the network capture
  file contains no credentials by allowlist.
- **CG6.** What survives restart: the token JSON (global datastore)
  and `session_id` (settings.ini). The access token is never replayed
  from disk — startup always refreshes. "Remember me" off clears both
  on exit.
- **CG7.** Token exchange and refresh go over TLS to GGG's documented
  endpoints only.

**Not held — the honest weaknesses (where "exceed" is cheap):**

- **CW1.** Plaintext at rest in two files, with version backups
  multiplying copies indefinitely (§1).
- **CW2.** The cookie is not host-scoped: it rides to `api.` and the
  OAuth token endpoint (inferred, §1). The symmetric guarantee to CG1
  simply does not exist.
- **CW3.** Three log-leak paths: trace-level raw bearer, the dead
  Authorization mask, and `format_error` token context (§1).
- **CW4.** No OAuth de-arming: refresh failure and bearer rejection
  are log-only, asymmetric with CG4.
- **CW5.** CG1/CG2 are unpinned convention — N21 already notes
  nothing enforces the host scoping; no test covers it.
- **CW6.** The POESESSID dialog reads the stored secret back into the
  UI as its pre-filled text (`mainwindow.cpp:1643`) — a secret
  readback a target model should not reproduce (§3).

## 3. The A0 credential surface

Commands (complete list; nothing else fell out of §1):

- `set_poesessid(secret)` — known from shop-write-path §5. This is
  the **one place a secret legitimately crosses IPC, inbound only**:
  the user pastes the cookie in the UI. Unavoidable while manual
  paste is the mechanism; it never needs to flow back out.
- `clear_poesessid()` — today reachable as rejection fallout and by
  submitting an empty string (`application.cpp:349-357`).
- `oauth_login()` — fires the browser flow; no arguments, no return
  value beyond eventual status events (today:
  `LoginDialog::OnAuthenticateButtonClicked` →
  `OAuthManager::initLogin`).
- `oauth_logout()` / forget-credentials — today only the
  remember-me-off clear (`logindialog.cpp:141-157`); a real command
  is a small improvement, not new machinery.

Read side — **status only, never a secret**:

- `auth_status` — authenticated flag, username, access and refresh
  expirations (exactly what `LoginWithOAuth` consumes today,
  `logindialog.cpp:320-339`).
- Scope mismatch — requested-but-not-granted scopes, which today is
  only a log line (`oauthmanager.cpp:162-168`) but is UI-worthy.
- `oauth_login_failed(reason)` — surfacing what today dies in the log
  (CW4).
- `session_rejected` — the existing de-arming event.
- An `oauth_rejected` de-arming event symmetric with CG4 — new in the
  target design, closing CW4.

**Does the webview ever legitimately need a secret? No — with one
deliberate UX change.** The expected answer holds for OAuth
unconditionally: today's UI consumes only username and expirations.
The exception is CW6: the current POESESSID dialog pre-fills the
stored value. Under A0 the field should be **write-only** (paste-only,
blank on open); the alternative — a `get_poesessid` command — was
rejected because it would put a secret on the IPC boundary solely for
display convenience.

## 4. Tauri's security model as it bears on custody

All external lane; official Tauri v2 docs, retrieved 2026-08-08.

- **Trust boundary.** Tauri separates a Core (Rust) process with
  "complete operating system access" from WebView processes; "Any
  code executed in the WebView has only access to exposed system
  resources via the well-defined IPC layer"
  (https://v2.tauri.app/security/,
  https://v2.tauri.app/concept/process-model/). IPC is serialized
  message passing, not shared memory
  (https://v2.tauri.app/concept/inter-process-communication/), so
  webview JS has no path to Rust-held token state beyond what
  commands and events expose (inferred from the process model; the
  docs never state it in memory-read terms).
- **Commands are deny-by-default in effect**: a command is invokable
  only if a permission allowing it is referenced by a capability
  attached to that webview's label
  (https://v2.tauri.app/security/permissions/,
  https://v2.tauri.app/security/capabilities/). Remote content gets
  IPC access only via an explicit `remote` capability field. Caveats
  the docs state plainly: capability files in `capabilities/` are
  auto-enabled; on Linux/Android Tauri "is unable to distinguish
  between requests from an embedded iframe and the window itself";
  and the boundary has had a real bypass (CVE-2024-35222, iframe IPC
  access, fixed;
  https://github.com/tauri-apps/tauri/security/advisories/GHSA-57fm-592m-34r7).
- **Events are NOT capability-gated** — "no support of the
  capabilities system to fine grain control event data", and global
  `emit()` reaches every listener
  (https://v2.tauri.app/develop/calling-frontend/). Design rule for
  the A0 surface: **no secrets in event payloads, ever** — which §3
  already satisfies.
- **CSP is off unless configured**
  (https://v2.tauri.app/security/csp/); the **isolation pattern**
  (still in v2) interposes trusted JS that can veto each IPC call,
  aimed exactly at compromised frontend bundles
  (https://v2.tauri.app/concept/inter-process-communication/isolation/).
  DevTools are debug-build-only unless opted in
  (https://v2.tauri.app/develop/debug/).
- **Threat-model deltas vs. the current Qt app:**
  - *Same-machine, same-user attacker*: **unchanged**. Tauri claims no
    protection; same-user processes can read either process's memory.
    OS keychains don't change this either (§6) — they protect disk,
    backups, and other users, not same-user code.
  - *XSS / compromised frontend bundle*: a **new surface Qt doesn't
    have**, but bounded: with §3's surface, a hostile frontend can
    invoke allowed commands (worst case: spam `post_shop`,
    `set_poesessid` garbage) but can never read a credential. And
    because the serialized rate-limit gate stays core-side, a
    compromised webview **cannot drive the client past GGG's limits**
    — directly relevant to the blacklist history. CSP + isolation
    pattern shrink it further.
  - *Malicious npm dependency*: new vs. Qt (which has no npm
    surface); runtime-constrained by the same capability/CSP/isolation
    machinery. A malicious Rust crate is game over in both worlds.
- **Net for custody**: keep every secret in the Rust host; shape
  commands as "do X" (post shop, refresh) rather than "give me the
  credential"; use `emit_to`, never global emit, for auth events.
  §3's surface already has this shape.

## 5. The OAuth flow under Tauri

External lane throughout; retrieved 2026-08-08.

- **RFC 8252 mandates the current architecture**: "native apps MUST
  use an external user-agent to perform OAuth authorization
  requests" and MUST NOT use embedded user-agents; PKCE is mandatory
  for public native clients; loopback IP literals are preferred over
  `localhost` (https://datatracker.ietf.org/doc/html/rfc8252). The
  Qt app already conforms (§1) — under Tauri the flow **moves
  language, not shape**, and the webview is never navigated to the
  authorize URL.
- **GGG's public-client rules**
  (https://www.pathofexile.com/developer/docs/authorization):
  authorization-code + PKCE only, "must use a local redirect URI
  (ie. http://127.0.0.1:8080/callback)"; `code_challenge_method`
  "must be S256"; access tokens 10 hours, refresh tokens 7 days
  (matching the hard-coded `oauthtoken.cpp:14`); refresh rotation
  inherits the original expiry, so >7 days idle means interactive
  re-auth; no `service:*` scopes; shared rate limits across all
  public clients. Redirect URIs "must match your client's registered
  URI" — but the docs are **silent on loopback port flexibility**;
  the production evidence in §1 (random port per run, logins work)
  says GGG's server allows it, consistent with the OAuth 2.1 posture
  the index page claims.
- **User-Agent**: the index page requires the prefix format
  `User-Agent: OAuth {clientId}/{version} (contact: {contact})`
  (https://www.pathofexile.com/developer/docs/index). The current app
  sends `acquisition/<version> (contact: gerwaric@gmail.com)` —
  **missing the `OAuth ` prefix** (`networkmanager.cpp:15-16`,
  `CMakeLists.txt:11-29`). Tolerated in practice; candidate finding 5.
- **Rust building blocks**: the `oauth2` crate v5 (maintained,
  2025-01 release) supports public clients — `ClientId` alone,
  `set_client_secret()` optional — with PKCE S256 and refresh
  (https://docs.rs/oauth2/latest/oauth2/). That "secret optional"
  design matches the empty-`client_secret` rejection evidence in §1.
  The loopback listener is a plain `TcpListener` or the community
  `tauri-plugin-oauth` (FabianLars;
  https://github.com/FabianLars/tauri-plugin-oauth) — a convenience,
  not a requirement. Nothing requires the webview to see the code,
  verifier, or tokens at any step.
- **Registration constraints — the real risk is any change**:
  - The docs currently state "We are currently unable to process new
    applications" and document **no channel for modifying** an
    existing client (no email, no procedure;
    https://www.pathofexile.com/developer/docs/index). Whether
    `/my-account/applications` allows self-service redirect-URI edits
    is undocumented. GGG historically registered by email and rejects
    LLM-generated requests (repo lane: this investigation's brief).
  - Custom-scheme redirects (tauri-plugin-deep-link) were therefore
    **rejected**: GGG's rule names only local redirect URIs, and a
    scheme change would require exactly the registration change that
    has no documented channel.
  - "One product per registered application" (index page) argues for
    sharing the single `acquisition` client id across the Qt→Rust
    transition rather than registering a second client — fine for a
    public client, which has no secret to share.
  - **Net: reuse the existing registration byte-for-byte** (same
    client id, loopback redirect, same callback path); a design that
    changes nothing GGG-visible has no registration dependency at all.

## 6. Storage at rest: the OS keychain option

External lane throughout; retrieved 2026-08-08.

- **`keyring` (keyring-rs) is the candidate that fits.** v4.1.6
  released 2026-08-01 (one week before this research), ~2.9M
  downloads/month, 1,144 dependent crates
  (https://lib.rs/crates/keyring). Backends: Windows Credential
  Manager, macOS Keychain, Linux Secret Service or kernel keyutils
  (https://docs.rs/keyring/latest/keyring/). The whole needed API is
  `Entry::new` / `set_password` / `get_password` /
  `delete_credential` — the canonical example is 9 lines. CI never
  touches real keychains (`keyring-core` ships a mock store). Called
  directly from the Rust core, **no Tauri plugin needed** — the
  community keyring plugins only matter if frontend JS touched
  secrets, which §3 rules out.
- **Platform caveats worth designing for**: Windows generic
  credentials cap at 2560 bytes
  (https://learn.microsoft.com/en-us/windows/win32/api/wincred/ns-wincred-credentialw)
  — store fields as separate entries, or keep one random wrapping key
  in the keychain and an encrypted token file on disk. Headless
  Linux may lack a Secret Service provider; the documented fallback
  is keyutils (in-memory, non-persistent) or app-level degradation —
  and since re-running the OAuth flow is cheap, "fall back to
  re-auth" is a legitimate floor for the token (POESESSID would fall
  back to re-paste).
- **What the keychain actually buys — honest statement**: encryption
  at rest keyed to the user's login; protection against disk theft,
  backups, and other OS users. It does **not** protect against code
  running as the same user (Windows: any same-session process can
  read generic credentials; Linux Secret Service: "This
  specification does not mandate any form of access control",
  http://specifications.freedesktop.org/secret-service/latest-single/;
  macOS is the partial exception — non-creator apps trigger a
  consent prompt). This exceeds CW1 meaningfully and is what
  mainstream desktop apps ship; nothing local exceeds it without
  hardware attestation.
- **Alternatives evaluated and rejected**: IOTA Stronghold /
  tauri-plugin-stronghold — an app-managed encrypted snapshot, not
  the OS keychain; the app must supply the password (bootstrapping
  problem: storing that password recreates plaintext-at-rest one
  level up), and the core crate is dormant (last release 2024-05,
  last dev commit 2023; https://lib.rs/crates/iota_stronghold).
  tauri-plugin-store — plain JSON persistence, no security property
  claimed (https://v2.tauri.app/plugin/store/); equivalent to today's
  plaintext.
- **Verdict input for §7**: exceeding plaintext-at-rest is cheap
  enough to be the default in a Rust core. In the current C++ app the
  equivalent (QtKeychain, a third-party dependency) is real work for
  a codebase under a rewrite decision — noted, not recommended now.

## 7. Verdict

**Research settles the custody-model question; one empirical question
remains and becomes the spike.**

Settled (this doc): custody confines entirely to the core with no
webview secret exposure (§3); the Tauri trust boundary supports —
and its docs recommend — exactly that shape (§4); the OAuth flow
runs whole in the Rust host, per the same RFC the current app
already follows (§5); exceeding the at-rest posture is cheap by
default via `keyring` (§6). The A0 addendum is §3's command/event
list; the standing rules are: **no secret in any event payload, no
outbound secret in any command result, host-scoped auth pinned in
the core's HTTP client** (bearer → `api.` only; cookie → `www.`
only — deliberately *stricter* than today, closing CW2).

**The spike reduces to one falsifiable prototype:**

> Does the existing `acquisition` public-client registration accept
> the full authorization-code + PKCE flow from a non-Qt
> implementation — Rust `oauth2` crate, system browser, loopback
> listener on an arbitrary OS-assigned port, `client_secret`
> omitted, User-Agent per GGG's documented format?

Pass: one manual login by Tom yields access + refresh tokens; one
GET against `api.pathofexile.com` succeeds with the bearer; one
refresh grant succeeds. Fail: any server-side rejection traceable to
redirect URI/port matching, `client_secret` handling, or User-Agent.
Roughly a hundred lines and a single deliberate live login —
deliberately outside this investigation's scope guards, in scope for
a spike branch. It doubles as the registration-risk check: passing
means the Qt→Rust move needs **no GGG-side change at all**, which
matters while the registration-change channel is frozen/undocumented
(§5). What would invalidate a pass later: GGG tightening redirect
matching or registration policy, a Tauri IPC advisory, or `keyring`
platform regressions.

**Adopt in the current C++ app regardless of the rewrite decision**
(F-numbers to be assigned on master, per the F68–F70 precedent):

1. Pin host-scoped auth with tests — bearer never to `www.`, and
   decide/enforce cookie scoping (closes CW5, answers N21's "pinned
   by nothing").
2. Scope the POESESSID cookie to `www.pathofexile.com` (closes CW2;
   both real consumers are on `www.`, so nothing should break —
   verify the legacy index still authenticates).
3. Fix the three log-leak paths (CW3): drop or mask the trace-level
   bearer log, fix the `logHeaders` comparison, and stop passing
   token-bearing buffers to `format_error` output.
4. Consider OAuth de-arm symmetry (CW4): surface refresh failure to
   the UI instead of only the log.

## Candidate findings (for `docs/cleanup/findings.md`, on master)

Found while tracing; described, not fixed, per the register's rules.

1. **Raw bearer token logged at trace level.**
   `networkmanager.cpp:121` writes the full `Bearer …` value via
   `spdlog::trace`; the log level is user-selectable in the login
   dialog, trace included (`logindialog.cpp:468-473`), so a user
   asked to "turn on trace logging and send the log" ships their
   token.
2. **The Authorization log mask can never fire.**
   `networkmanager.cpp:172` compares `name` — the "request"/"reply"
   label — against "Authorization" instead of comparing `header`.
   Latent today (call sites log pre-send requests and reply headers,
   neither carrying the bearer), but any future post-send request
   log would leak unmasked.
3. **Token bytes can reach the error log via `glz::format_error`.**
   On token serialization failure (`oauthmanager.cpp:148-151`) and
   token parse failure (`json_readers.cpp:44-48` via
   `readOAuthToken`), the formatted error embeds context from the
   token-bearing buffer.
4. **The POESESSID cookie is not host-scoped.** Installed domain-wide
   on `.pathofexile.com` (`networkmanager.cpp:93-96`), it is sent to
   `api.pathofexile.com` and the OAuth token endpoint, not just its
   `www.` consumers. The secret reaches hosts that never need it.
5. **User-Agent does not follow GGG's documented format.** The docs
   require the prefix `OAuth {clientId}/{version}`; the app sends
   `acquisition/<version> (contact: …)` without the `OAuth ` prefix
   (`networkmanager.cpp:15-16`). Tolerated in practice, but it is
   the one documented API-citizenship rule the client visibly
   breaks — worth weighing given the project's history.
6. **Dead OAuth/session code.** `OAuthManager::m_authenticated` is
   never written and `isAuthenticatedChanged` is connected nowhere
   (`oauthmanager.h:38`, `oauthmanager.h:50`);
   `LoginDialog::OnSessionIDChanged` is declared and defined but
   never connected (`logindialog.h:42`). F69 precedent.

## Dead ends and rejected interpretations

- **Rejected: a `get_poesessid` command** so the webview can pre-fill
  the settings dialog as today's UI does. Write-only field instead —
  putting a secret on the IPC boundary for display convenience is
  exactly the anti-pattern §4 warns about.
- **Rejected: custom-scheme (deep-link) redirect under Tauri.**
  GGG's public-client rule names local redirect URIs only, and the
  change would require a registration modification with no
  documented channel (§5).
- **Rejected: a second client id for the Rust app.** The "one
  product per registered application" rule and the registration
  freeze both cut toward sharing `acquisition` (§5).
- **Not re-opened: gating OAuth traffic through the hub.** The D5
  scope rationale (login and refresh are a handful of requests,
  three orders of magnitude below the layer-1 trigger) transfers to
  a rewrite unchanged; refresh stays outside the gate.
- **Dead end: an official Tauri "never send secrets to the
  frontend" sentence.** It does not exist in the v2 docs; the
  position is inferred from the trust-boundary model and the
  "webview is insecure" assumption, and one explicit warning exists
  only in an unofficial docs mirror. Recorded so a future reader
  does not go looking for a stronger citation than exists.
- **Expected but absent: GGG guidance on desktop token storage and
  session cookies.** The developer docs say "You must not share
  access tokens with anyone but their owner" and address token
  storage only in web-app terms; POESESSID appears nowhere in them
  (retrieved 2026-08-08). Silence, not permission — noted.

## Verification notes

- **Traced in code** (working tree, `redesign`, 2026-08-08):
  `oauthmanager.h/.cpp`, `oauthtoken.h/.cpp`,
  `networkmanager.h/.cpp`, `logindialog.cpp`, `application.cpp`
  (wiring, session-id handling, version backup),
  `sqlitedatastore.cpp`, `networkcapture.cpp`, the
  `logRequest`/`logReply` call sites in `ratelimiter.cpp` and
  `ratelimitmanager.cpp`, `poe_utils.cpp` builders, `json_readers.cpp`,
  `CMakeLists.txt` name/email values.
- **Grep-verified absences**: no consumer of the `oauth_token` row
  besides `OAuthManager` and the `LoginDialog` clear; no
  `setBearerToken` caller besides `receiveToken`; no 401/403
  handling outside the shop; no connect of `isAuthenticatedChanged`
  or `OnSessionIDChanged`.
- **Taken from repo docs, not re-verified**: N21/N22 regime claims,
  D5 scope rationale, shop-write-path §2/§5/§6 conclusions,
  F68–F70 precedent.
- **External** (all retrieved 2026-08-08, gathered by three
  web-research subagents; URLs inline in §4–§6 plus the Qt handler
  doc in §1): Tauri v2 official security/IPC/capability/CSP/event
  docs and advisory GHSA-57fm-592m-34r7; GGG developer docs
  (authorization, index); RFC 8252; docs.rs/crates.io/lib.rs pages
  for `oauth2`, `keyring`, `iota_stronghold`, Tauri plugins;
  Microsoft wincred docs; Apple keychain guides; the freedesktop
  Secret Service spec; doc.qt.io for `QOAuthHttpServerReplyHandler`.
- **Inferred** (stated as such where used): cookie domain-matching
  delivery to `api.` (no packet capture); GGG port-agnostic loopback
  matching (random port per run + production logins succeed);
  `format_error` embedding token bytes (from glaze's documented
  context behavior; not reproduced).
- No live network verification; no credential values read or quoted.

## Open questions

- What exact redirect URI (host/port/path) is registered for the
  `acquisition` client? Only Tom's registration records can say; it
  would turn the port-flexibility inference into a measured claim.
- Does GGG's "unable to process new applications" freeze also cover
  modifications to existing clients, and does
  `/my-account/applications` allow self-service edits? Undocumented;
  answerable by Tom looking at the account page (no API traffic
  involved).
- Are GGG access tokens small enough for a single Windows credential
  entry (2560-byte cap), or does the keychain design need the
  wrapping-key pattern from §6? Answerable by inspecting a token's
  length (not its value) in a live session.
- Should the C++ app's cookie scoping fix (adopt-now item 2) land
  before or with the host-scoping pin (item 1)? Same test surface;
  probably one change.
