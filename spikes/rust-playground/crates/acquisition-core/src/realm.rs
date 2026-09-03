//! Realm: the coordinate above league (CONTEXT.md, decided 2026-09-02).
//!
//! PoE2 shares league names with PoE1 (`Standard` exists in both), so a
//! league alone does not locate anything. On the wire the realm is a path
//! segment that precedes the league or name (`/stash/xbox/Standard`,
//! `/character/poe2/Name`) and **pc is expressed by omission** — `pc` is
//! not a legal segment value — so every pc URL is byte-identical to what
//! was sent before realms existed.
//!
//! Which realms each endpoint family accepts is declared here, once, and
//! read by the daemon (URL rendering and admission), the mock (path
//! classification), and the planner (a policy naming tabs under `poe2` is
//! a parse error). Documented 2026-09-02: characters take `xbox|sony|poe2`;
//! stashes and account leagues take `xbox|sony` and are titled "PoE1
//! only". When GGG extends PoE2 to the other families, the change is one
//! arm in [`Family::accepts`] plus a first-contact sample under
//! `LIVE-TESTING.md`'s standing rule — an unobserved URL shape is never
//! sent.
//!
//! # Decisions as recorded
//!
//! The rulings are `CONTEXT.md`'s registry (`C<n>`); what follows is each
//! entry's full text as recorded there, moved here on 2026-09-02 because
//! the mechanism it describes is this module's. The registry is current;
//! this is the mechanism as decided, kept beside the code that implements it.
//!
//! ## C58 — Realm is a coordinate above league, everywhere
//!
//! **Realm is a coordinate above league, everywhere.** PoE2 1.0 ships
//!   December 2026 (announced late August); league names collide across
//!   realms (Standard exists in both games). Policy becomes
//!   `realms.<R>.leagues.<L>.{tabs, characters, max_age_seconds}`
//!   (the realm nesting is **policy v2**, built; `characters` beside `tabs`
//!   makes **v3** — a new sibling field is a shape change, so a v2 reader
//!   says "newer version", never "malformed"; each older version upgrades
//!   on parse, v1 as realm pc); the
//!   plan envelope carries realm beside league; the snapshot is taken per
//!   `(realm, league)`; `tabs` and `characters` facts carry a realm column
//!   (existing rows pc); realm is an explicit param on all four data kinds
//!   (`stashes`, `stash`, `characters`, `character`), defaulted to pc only
//!   at the decode boundary (`Endpoint::from_job`) so persisted pre-realm
//!   jobs still decode. On the wire realm is a path segment and **pc is
//!   expressed by omission — `pc` is not a legal segment value**
//!   (documented), so pc URLs stay byte-identical to every live send so
//!   far: the realm step costs no live spend, and the mock rehearsal
//!   journal proves the URLs did not move.
//!
//! ## C59 — Which realms each route accepts is declared in one place
//!
//! **Which realms each route accepts is declared in one place, never one
//!   shared list** (the `declare_route_knowledge` mold). Documented today:
//!   `/character` and `/character/{name}` take `xbox|sony|poe2`;
//!   `/stash…` and `/account/leagues` take `xbox|sony` and are titled
//!   "PoE1 only". A policy naming `tabs` under `poe2` is a structured parse
//!   error; no code path renders a stash URL with `poe2`. GGG is expected
//!   to extend PoE2 to the other endpoints (almost certainly the same
//!   segment): when it ships, the change is one row in that table plus a
//!   first-contact sample under the standing rule. Not pre-enabled — an
//!   unobserved URL shape is never sent.
//!
//! ## C58 — The realm step as built (2026-09-02)
//!
//! Properties fixed inside the ruling while building, each pinned by a
//! test (findings that bought them: `REFRESH-SLICE.md`):
//!
//! - **The realm table is one type in core** (`realm.rs`: `Realm`,
//!   `Family::accepts`), read by the daemon (rendering + admission), the
//!   mock (path classification), and the planner (policy parse) — the
//!   `declare_route_knowledge` mold, linkable by all three because the
//!   planner already depends on core. The store stays string-typed: it
//!   records the request's realm and never validates it.
//! - **A non-pc realm suffixes the limiter's route label**
//!   (`character-list/poe2`, `stash-list/xbox`) as well as the URL, so each
//!   realm's URL shape gets its own free HEAD before its first counted
//!   send; whether it shares the pc policy is learned from headers (N6
//!   already shares state by name). One extra HEAD per realm per lifetime.
//! - **Admission refuses a realm a family does not take** (`admit_realm`
//!   at submit and per apply tuple, before a job id exists) — the daemon
//!   side of "no code path renders a stash URL with poe2"; the planner
//!   side is the policy parse.
//! - **Items carry realm beside league** (stamped from the seam); the
//!   ad-hoc `refresh` kind forwards realm to its children; `leagues`
//!   stays realm-less (no consumer, not in the plan).
//! - **Mock consoles are truthful, not colliding**: an xbox/sony stash
//!   listing is empty and a tab fetch 404s (tab and item ids are
//!   GGG-unique, so pc's are never reused under another realm); the
//!   character list is per realm, with one poe2 character carrying a
//!   `skills` array (confirmed live 2026-09-02, N42–N44; the mock body
//!   now also carries an item-granted skill, the id-less shape N44
//!   records).
//! - Policy v2 parses v1 in place; the stored value stays what was
//!   written (`policy show` shows what the human typed).

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Realm {
    Pc,
    Xbox,
    Sony,
    Poe2,
}

impl Realm {
    pub const ALL: [Realm; 4] = [Realm::Pc, Realm::Xbox, Realm::Sony, Realm::Poe2];

    /// The realm a job or a policy means when it names none.
    pub const DEFAULT: Realm = Realm::Pc;

    pub fn parse(s: &str) -> Option<Realm> {
        Realm::ALL.into_iter().find(|r| r.as_str() == s)
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Realm::Pc => "pc",
            Realm::Xbox => "xbox",
            Realm::Sony => "sony",
            Realm::Poe2 => "poe2",
        }
    }

    /// What goes on the wire: `None` for pc (omitted), the segment
    /// otherwise.
    pub fn segment(self) -> Option<&'static str> {
        match self {
            Realm::Pc => None,
            other => Some(other.as_str()),
        }
    }

    /// The realm's contribution to a path or a route label, ready to
    /// splice: `""` for pc, `"/poe2"` otherwise. Used for URLs
    /// (`/character{infix}/{name}`) and for the limiter's route labels
    /// (`character-list{infix}`), so a realm's URL shape gets its own
    /// probe before its first counted send — whether it shares the pc
    /// policy is learned from headers, never assumed.
    pub fn infix(self) -> &'static str {
        match self {
            Realm::Pc => "",
            Realm::Xbox => "/xbox",
            Realm::Sony => "/sony",
            Realm::Poe2 => "/poe2",
        }
    }

    /// The `realm` entry of a job's params. Absent means pc — the decode
    /// default that keeps pre-realm persisted jobs valid; a value that is
    /// not a realm is a structured refusal, never a guess.
    pub fn from_params(params: &Value) -> Result<Realm, String> {
        match params.get("realm") {
            None | Some(Value::Null) => Ok(Realm::DEFAULT),
            Some(Value::String(s)) => Realm::parse(s).ok_or_else(|| {
                format!(
                    "unknown realm {s:?} (one of {})",
                    Realm::ALL.map(Realm::as_str).join(", ")
                )
            }),
            Some(other) => Err(format!("realm must be a string, not {other}")),
        }
    }
}

impl std::fmt::Display for Realm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// An endpoint family with its own documented realm acceptance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Family {
    /// `/character[/realm]` and `/character[/realm]/{name}`.
    Characters,
    /// `/stash[/realm]/{league}[/{id}[/{sub}]]` — PoE1 only.
    Stashes,
    /// `/account/leagues[/realm]` — PoE1 only.
    Leagues,
}

impl Family {
    /// Whether the family's endpoints take this realm, as documented. The
    /// one place that knowledge lives.
    pub fn accepts(self, realm: Realm) -> bool {
        match (self, realm) {
            (_, Realm::Pc | Realm::Xbox | Realm::Sony) => true,
            (Family::Characters, Realm::Poe2) => true,
            (Family::Stashes | Family::Leagues, Realm::Poe2) => false,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Family::Characters => "characters",
            Family::Stashes => "stashes",
            Family::Leagues => "leagues",
        }
    }

    /// The family's realm from a job's params, refused when the params
    /// name one the family does not take — so no code path can render a
    /// stash URL under `poe2`.
    pub fn realm_of(self, params: &Value) -> Result<Realm, String> {
        let realm = Realm::from_params(params)?;
        if !self.accepts(realm) {
            return Err(format!(
                "the {} endpoints do not take realm {realm} (documented: {})",
                self.name(),
                Realm::ALL
                    .into_iter()
                    .filter(|r| self.accepts(*r))
                    .map(Realm::as_str)
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        Ok(realm)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn pc_is_omitted_on_the_wire_and_every_other_realm_is_a_segment() {
        assert_eq!(Realm::Pc.segment(), None);
        assert_eq!(Realm::Pc.infix(), "");
        for realm in [Realm::Xbox, Realm::Sony, Realm::Poe2] {
            assert_eq!(realm.segment(), Some(realm.as_str()));
            assert_eq!(realm.infix(), format!("/{realm}"));
        }
        // `pc` is not a legal segment value, but it is the name of the
        // realm: parse accepts it and a param may say it explicitly.
        assert_eq!(Realm::parse("pc"), Some(Realm::Pc));
        assert_eq!(Realm::from_params(&json!({ "realm": "pc" })), Ok(Realm::Pc));
        assert_eq!(Realm::from_params(&json!({})), Ok(Realm::Pc));
        assert_eq!(Realm::from_params(&json!({ "realm": null })), Ok(Realm::Pc));
    }

    #[test]
    fn an_unknown_realm_is_refused_not_defaulted() {
        assert!(
            Realm::from_params(&json!({ "realm": "ps5" }))
                .unwrap_err()
                .contains("unknown realm")
        );
        assert!(
            Realm::from_params(&json!({ "realm": 2 }))
                .unwrap_err()
                .contains("must be a string")
        );
        assert_eq!(Realm::parse("PC"), None);
    }

    #[test]
    fn acceptance_is_per_family_as_documented() {
        for realm in [Realm::Pc, Realm::Xbox, Realm::Sony] {
            assert!(Family::Characters.accepts(realm));
            assert!(Family::Stashes.accepts(realm));
            assert!(Family::Leagues.accepts(realm));
        }
        assert!(Family::Characters.accepts(Realm::Poe2));
        assert!(!Family::Stashes.accepts(Realm::Poe2));
        assert!(!Family::Leagues.accepts(Realm::Poe2));
        let refused = Family::Stashes
            .realm_of(&json!({ "realm": "poe2" }))
            .unwrap_err();
        assert!(
            refused.contains("stashes endpoints do not take realm poe2"),
            "{refused}"
        );
        assert!(refused.contains("pc, xbox, sony"), "{refused}");
        assert_eq!(
            Family::Characters.realm_of(&json!({ "realm": "poe2" })),
            Ok(Realm::Poe2)
        );
    }

    #[test]
    fn serializes_as_its_lowercase_name() {
        assert_eq!(serde_json::to_value(Realm::Poe2).unwrap(), json!("poe2"));
        assert_eq!(
            serde_json::from_value::<Realm>(json!("xbox")).unwrap(),
            Realm::Xbox
        );
        assert!(serde_json::from_value::<Realm>(json!("PC")).is_err());
    }
}
