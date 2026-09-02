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
    envelope    TEXT    NOT NULL,   -- the body with item arrays removed; `_split` lists their counts —
                                    -- or the whole body verbatim when the fetch was withheld
    item_count  INTEGER NOT NULL,
    withheld    INTEGER NOT NULL DEFAULT 0  -- items this fetch carried for a location a listing had retired:
                                            -- kept here, landed nowhere (membership is the listing's)
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

-- Characters are keyed by the GGG character `id` (identity; stable across
-- renames); `name` is the address the fetch endpoint takes and can move.
-- `league` is listing-owned — the coverage coordinate as the basis listing
-- said it — and a fetch never overwrites it (CONTEXT.md, 2026-09-02).
CREATE TABLE IF NOT EXISTS characters (
    id          TEXT PRIMARY KEY,
    realm       TEXT    NOT NULL DEFAULT 'pc',
    name        TEXT    NOT NULL,
    league      TEXT,
    class       TEXT,
    level       INTEGER,
    json        TEXT    NOT NULL,   -- the fetched character minus its item arrays; the list entry until a fetch lands
    listed_json TEXT,               -- the latest list entry, verbatim (a fetch never touches it)
    listed_at   INTEGER,
    listed_response INTEGER,        -- responses.id of the listing that last listed this character
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
    realm          TEXT    NOT NULL DEFAULT 'pc',
    league         TEXT,
    location_kind  TEXT NOT NULL,      -- stash | character
    location_id    TEXT NOT NULL,      -- tab/substash id, or character id
    container      TEXT,               -- the array the item came from: `items` (stash), or a character's
                                       -- inventory|equipment|jewels|rucksack|guardian|skills; an ingest fact,
                                       -- not in the item's json (NULL: recorded before facts v4)
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
    seen_response  INTEGER,            -- responses.id of the fetch that last saw the item at its location:
                                       -- membership is per response, never per clock second
    removed_at     INTEGER             -- set when a fetch of its location no longer had it, or the location was retired
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
