-- The shared store. One file per provider; opened by the daemon (writer,
-- via `record`) and by every frontend (readers). Raw responses are kept
-- verbatim except at the item seams: every item array is lifted out into
-- `items`, one row per GGG item id, socketed gems included.
--
-- `realm` is the coordinate above league (CONTEXT.md, 2026-09-02): PoE2
-- shares league names with PoE1. It is the *request's* realm, stamped from
-- the job params (pc when omitted), never read from a body.

CREATE TABLE IF NOT EXISTS responses (
    id          INTEGER PRIMARY KEY,
    endpoint    TEXT    NOT NULL,   -- leagues|profile|characters|character|stashes|stash
    params      TEXT    NOT NULL,   -- the job params, verbatim JSON
    fetched_at  INTEGER NOT NULL,   -- unix seconds
    status      INTEGER NOT NULL,
    envelope    TEXT    NOT NULL,   -- the body with item arrays removed; `_split` lists their counts
    item_count  INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS leagues (
    id       TEXT PRIMARY KEY,
    json     TEXT    NOT NULL,
    seen_at  INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS account (
    uuid     TEXT PRIMARY KEY,
    name     TEXT,
    json     TEXT    NOT NULL,
    seen_at  INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS characters (
    name        TEXT PRIMARY KEY,
    realm       TEXT    NOT NULL DEFAULT 'pc',
    league      TEXT,
    class       TEXT,
    level       INTEGER,
    json        TEXT    NOT NULL,   -- list entry, or the fetched character minus its item arrays
    listed_at   INTEGER,
    fetched_at  INTEGER,
    removed_at  INTEGER
);

CREATE TABLE IF NOT EXISTS tabs (
    realm       TEXT NOT NULL DEFAULT 'pc',
    league      TEXT NOT NULL,
    id          TEXT NOT NULL,
    parent      TEXT,               -- folder id (from the list) or parent tab id (substash)
    name        TEXT,
    type        TEXT,
    idx         INTEGER,
    json        TEXT NOT NULL,      -- the fetched tab minus items and children; the first list entry until a fetch lands
    listed_json TEXT,               -- the latest list entry / substash stub, verbatim (a fetch never touches it)
    listed_at   INTEGER,
    listed_response INTEGER,        -- responses.id of the listing (or parent fetch) that last listed this tab
    fetched_at  INTEGER,
    removed_at  INTEGER,
    PRIMARY KEY (realm, league, id)
);

CREATE TABLE IF NOT EXISTS items (
    id             TEXT PRIMARY KEY,   -- GGG item id, stable across moves
    realm          TEXT,
    league         TEXT,
    location_kind  TEXT NOT NULL,      -- stash | character
    location_id    TEXT NOT NULL,      -- tab/substash id, or character name
    socketed_in    TEXT,               -- parent item id for socketed gems
    name           TEXT,
    type_line      TEXT,
    base_type      TEXT,
    rarity         TEXT,
    stack_size     INTEGER,
    x              INTEGER,
    y              INTEGER,
    w              INTEGER,
    h              INTEGER,
    json           TEXT    NOT NULL,   -- verbatim, minus socketedItems
    first_seen     INTEGER NOT NULL,
    last_seen      INTEGER NOT NULL,
    removed_at     INTEGER             -- set when a fetch of its location no longer had it
);
CREATE INDEX IF NOT EXISTS items_location ON items (location_kind, location_id);
CREATE INDEX IF NOT EXISTS items_socketed_in ON items (socketed_in);
CREATE INDEX IF NOT EXISTS items_names ON items (name, type_line, base_type);

-- What each ingest concluded, so "what changed since last time" is a query.
CREATE TABLE IF NOT EXISTS item_events (
    id             INTEGER PRIMARY KEY,
    response_id    INTEGER NOT NULL,
    at             INTEGER NOT NULL,
    item_id        TEXT    NOT NULL,
    kind           TEXT    NOT NULL,   -- added | moved | changed | removed
    from_location  TEXT,
    to_location    TEXT
);
CREATE INDEX IF NOT EXISTS item_events_at ON item_events (at);
