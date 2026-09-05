//! The typed price: what a `buyout` row says ([`Buyout`], C67) and what
//! it is about ([`PriceTarget`]). Built by the pricing slice's plan step 2
//! (`PRICING-SLICE.md`, 2026-09-05). Nothing here reads a store or a
//! daemon; this module is the value's shape, its parse, and its address.
//!
//! # Decisions as recorded
//!
//! **C67 — The v1 `buyout` and its target.** A typed `PriceTarget` — item
//! and character by id (C55), tab by `(realm, id)`, substash by `(realm,
//! parent, id)`, league-less (a merge keeps the row) — is the public API.
//! `type` is `exact` (`~price`), `negotiable` (`~b/o`), `no_price` or
//! `ignore`; the first two carry `amount` and a `currency` tag that must
//! resolve in the reference table (C68). `amount` is a decimal of at most
//! two fractional digits, or a lot ratio `wanted/lot` (T2) of two unreduced
//! positive integers; canonical text, structural equality; more digits are
//! refused. *Why:* the key mirrors the store's identity (C54, C58); the
//! game writes four places (T10), two ruled 2026-09-04; widening is
//! compatible, narrowing is not. *Details:* `price.rs`. *Pinned:* the
//! `c67_` tests.
//!
//! # As built
//!
//! **The target** is the row's `(scope, key)`, rendered and parsed by one
//! type so the realm-bearing keys were defined before the first
//! tab-scoped row landed (2b constraint (6)): `item/<id>`,
//! `character/<id>`, `tab/<realm>/<id>`, `substash/<realm>/<parent>/<id>`.
//! Ids are GGG's, opaque; a component is never empty and never holds a
//! `/`; the realm is [`Realm`]'s own word (`pc`, `xbox`, `sony`, `poe2`),
//! spelled out even for pc — the wire's pc-by-omission (C58) is a URL
//! rule, not a key rule. No league: a tab id is stable across a league
//! merge, and the row should be too.
//!
//! **The value** is stamped `version: 1` and is one of four shapes. In
//! JSON: `{"version":1,"type":"exact","amount":"12.5","currency":"chaos"}`
//! (or `negotiable`), `{"version":1,"type":"no_price"}`,
//! `{"version":1,"type":"ignore"}`. `amount` is text, never a JSON
//! number, so `2.50` and `2.5` cannot silently become one float: as text
//! the canonical spelling is the shortest — no leading zeros, no trailing
//! fraction zeros, no trailing dot — and a ratio is `wanted/lot` of two
//! positive integers kept exactly as written (`22/10` is not `11/5`, and
//! `3/1` is not `3`: the lot is a lot). [`Amount`] compares structurally
//! on those parts, so `"12.50"` typed by a human ([`Amount::from_str`])
//! equals `"12.5"`, while the JSON door holds a v1 value to its canonical
//! text through the store's exact round-trip (C66) — a non-canonical
//! spelling in a value is refused there naming the path, not rewritten.
//! A third fractional digit is refused; `0`, `0.00` and `0/5` are
//! refused (a price is positive); `10/0` is refused.
//!
//! `currency` is a **tag** of the shipped currency table
//! ([`crate::currency`]) — the immutable identity intent cites (C68),
//! never an alias or a display name. A word that resolves to a row by
//! alias or emit but is not that row's tag is refused naming the tag, so
//! the frontend can offer it; a word the table does not know is refused
//! naming the table version. A **retired** tag parses: a stored row may
//! cite it forever, and whether a *new* price may name one is the
//! writer's rule (the CLI's, at plan step 5), not the value's. `no_price`
//! and `ignore` carry no amount and no currency; a value that supplies
//! either is refused (the 0.18 userstore's `[ignore]` rows carried a
//! non-semantic 4321 `blessed`, which is exactly the shape this refuses).
//!
//! [`Buyout`] is an [`IntentValue`], so the store's write door runs the
//! version gate, this parse and the exact round-trip before the
//! compare-and-swap; nothing lands that this module would not read back.

use std::fmt;
use std::str::FromStr;

use acquisition_core::realm::Realm;
use acquisition_store::IntentValue;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::currency;

/// The row kind a price is stored under.
pub const BUYOUT_KIND: &str = "buyout";

/// The `buyout` value schema this build writes ([`Buyout`]).
pub const BUYOUT_VERSION: i64 = 1;

/// The scopes a price may sit on, as stored; the key grammar is
/// [`PriceTarget::address`].
pub const ITEM_SCOPE: &str = "item";
pub const CHARACTER_SCOPE: &str = "character";
pub const TAB_SCOPE: &str = "tab";
pub const SUBSTASH_SCOPE: &str = "substash";

/// What a price is about: the row's address, typed. Identity mirrors the
/// store's (C54, C55, C58): an item or character by its GGG id, a tab by
/// realm and id, a substash by realm, parent tab and its own id. No
/// league.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "scope", rename_all = "snake_case", deny_unknown_fields)]
pub enum PriceTarget {
    Item {
        id: String,
    },
    Character {
        id: String,
    },
    Tab {
        realm: Realm,
        id: String,
    },
    Substash {
        realm: Realm,
        parent: String,
        id: String,
    },
}

/// Why a `(scope, key)` is not a [`PriceTarget`], or a component cannot
/// be one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TargetError {
    /// Not one of the four price scopes.
    UnknownScope { scope: String },
    /// A key with the wrong number of `/`-separated parts for its scope.
    MalformedKey { scope: &'static str, key: String },
    /// A realm word [`Realm`] does not know.
    UnknownRealm { realm: String },
    /// An id, parent or realm component that is empty or holds a `/`.
    MalformedComponent { component: String },
}

impl fmt::Display for TargetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TargetError::UnknownScope { scope } => write!(
                f,
                "{scope:?} is not a price scope (item, character, tab, substash)"
            ),
            TargetError::MalformedKey { scope, key } => {
                write!(f, "{key:?} is not a {scope} key")
            }
            TargetError::UnknownRealm { realm } => write!(f, "{realm:?} is not a realm"),
            TargetError::MalformedComponent { component } => write!(
                f,
                "{component:?}: a target component is a non-empty id without `/`"
            ),
        }
    }
}

impl std::error::Error for TargetError {}

fn component(s: &str) -> Result<(), TargetError> {
    if s.is_empty() || s.contains('/') {
        return Err(TargetError::MalformedComponent {
            component: s.into(),
        });
    }
    Ok(())
}

fn realm(word: &str) -> Result<Realm, TargetError> {
    Realm::parse(word).ok_or_else(|| TargetError::UnknownRealm { realm: word.into() })
}

impl PriceTarget {
    /// The stored address: the scope and the key under it. Refuses a
    /// target whose components could not be read back (an empty id, a
    /// `/` inside one), so every address this renders parses again.
    pub fn address(&self) -> Result<(&'static str, String), TargetError> {
        Ok(match self {
            PriceTarget::Item { id } => {
                component(id)?;
                (ITEM_SCOPE, id.clone())
            }
            PriceTarget::Character { id } => {
                component(id)?;
                (CHARACTER_SCOPE, id.clone())
            }
            PriceTarget::Tab { realm, id } => {
                component(id)?;
                (TAB_SCOPE, format!("{}/{id}", realm.as_str()))
            }
            PriceTarget::Substash { realm, parent, id } => {
                component(parent)?;
                component(id)?;
                (SUBSTASH_SCOPE, format!("{}/{parent}/{id}", realm.as_str()))
            }
        })
    }

    /// The target a stored row's `(scope, key)` names — the inverse of
    /// [`PriceTarget::address`].
    pub fn from_address(scope: &str, key: &str) -> Result<PriceTarget, TargetError> {
        let parts: Vec<&str> = key.split('/').collect();
        let malformed = |scope: &'static str| TargetError::MalformedKey {
            scope,
            key: key.into(),
        };
        let target = match (scope, parts.as_slice()) {
            (ITEM_SCOPE, [id]) => PriceTarget::Item { id: (*id).into() },
            (ITEM_SCOPE, _) => return Err(malformed(ITEM_SCOPE)),
            (CHARACTER_SCOPE, [id]) => PriceTarget::Character { id: (*id).into() },
            (CHARACTER_SCOPE, _) => return Err(malformed(CHARACTER_SCOPE)),
            (TAB_SCOPE, [r, id]) => PriceTarget::Tab {
                realm: realm(r)?,
                id: (*id).into(),
            },
            (TAB_SCOPE, _) => return Err(malformed(TAB_SCOPE)),
            (SUBSTASH_SCOPE, [r, parent, id]) => PriceTarget::Substash {
                realm: realm(r)?,
                parent: (*parent).into(),
                id: (*id).into(),
            },
            (SUBSTASH_SCOPE, _) => return Err(malformed(SUBSTASH_SCOPE)),
            (other, _) => {
                return Err(TargetError::UnknownScope {
                    scope: other.into(),
                });
            }
        };
        // A split never yields a `/` inside a part; emptiness it can.
        target.address()?;
        Ok(target)
    }
}

impl fmt::Display for PriceTarget {
    /// `scope/key`, the address as one word.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.address() {
            Ok((scope, key)) => write!(f, "{scope}/{key}"),
            Err(e) => write!(f, "<invalid target: {e}>"),
        }
    }
}

/// How much: a decimal of at most two fractional digits, held as
/// hundredths, or a bulk ratio `wanted/lot` (T2) kept unreduced. Equality
/// is structural on those parts — `2.50` and `2.5` are one amount,
/// `22/10` and `11/5` are two.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Amount {
    /// A positive decimal, in hundredths: `12.5` is `1250`.
    Decimal { hundredths: u64 },
    /// `wanted` of the currency for a `lot` of the item; both positive.
    Ratio { wanted: u64, lot: u64 },
}

/// Why a text is not an [`Amount`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AmountError {
    pub text: String,
    pub why: &'static str,
}

impl fmt::Display for AmountError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "amount {:?}: {}", self.text, self.why)
    }
}

impl std::error::Error for AmountError {}

/// A positive integer with no leading zero, within `u64`.
fn positive_integer(s: &str) -> Result<u64, &'static str> {
    if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) || s.starts_with('0') {
        return Err("a positive integer has digits only and no leading zero");
    }
    s.parse().map_err(|_| "too large")
}

impl FromStr for Amount {
    type Err = AmountError;

    /// The human spelling: `12`, `12.5`, `12.50` (a trailing zero is
    /// tolerated and dropped), `22/10`. Refused: a third fractional digit
    /// (never rounded), a leading zero, a sign, whitespace, zero, a zero
    /// lot, and anything that is not digits, one `.` or one `/`.
    fn from_str(text: &str) -> Result<Amount, AmountError> {
        let err = |why: &'static str| AmountError {
            text: text.into(),
            why,
        };
        if let Some((wanted, lot)) = text.split_once('/') {
            let part = |s: &str| match positive_integer(s) {
                Ok(n) => Ok(n),
                Err("too large") => Err(err("too large")),
                Err(_) => Err(err("`wanted/lot` needs two positive integers")),
            };
            return Ok(Amount::Ratio {
                wanted: part(wanted)?,
                lot: part(lot)?,
            });
        }
        let (whole, fraction) = text.split_once('.').unwrap_or((text, ""));
        if text.contains('.') && fraction.is_empty() {
            return Err(err("a decimal does not end in `.`"));
        }
        if fraction.len() > 2 {
            return Err(err("at most two fractional digits (refused, not rounded)"));
        }
        if !fraction.bytes().all(|b| b.is_ascii_digit()) {
            return Err(err(
                "a decimal is digits, optionally `.` and one or two digits",
            ));
        }
        let units: u64 = if whole == "0" {
            0
        } else {
            match positive_integer(whole) {
                Ok(n) => n,
                Err("too large") => return Err(err("too large")),
                Err(_) => {
                    return Err(err(
                        "a decimal is digits, optionally `.` and one or two digits",
                    ));
                }
            }
        };
        let cents: u64 = match fraction.len() {
            0 => 0,
            1 => fraction.parse::<u64>().map_err(|_| err("not a decimal"))? * 10,
            _ => fraction.parse::<u64>().map_err(|_| err("not a decimal"))?,
        };
        let hundredths = units
            .checked_mul(100)
            .and_then(|h| h.checked_add(cents))
            .ok_or_else(|| err("too large"))?;
        if hundredths == 0 {
            return Err(err("a price is positive"));
        }
        Ok(Amount::Decimal { hundredths })
    }
}

impl fmt::Display for Amount {
    /// The canonical text: the shortest decimal, or `wanted/lot` verbatim.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Amount::Decimal { hundredths } => {
                let (units, cents) = (hundredths / 100, hundredths % 100);
                if cents == 0 {
                    write!(f, "{units}")
                } else if cents % 10 == 0 {
                    write!(f, "{units}.{}", cents / 10)
                } else {
                    write!(f, "{units}.{cents:02}")
                }
            }
            Amount::Ratio { wanted, lot } => write!(f, "{wanted}/{lot}"),
        }
    }
}

/// An amount in a currency: the tag intent cites (C68).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Price {
    pub amount: Amount,
    /// A tag of the shipped currency table; retired tags included.
    pub currency: String,
}

impl fmt::Display for Price {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.amount, self.currency)
    }
}

/// The `buyout` value, v1 (C67): four shapes, two with a price.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Buyout {
    /// `~price`: this amount.
    Exact(Price),
    /// `~b/o`: this amount, open to offers.
    Negotiable(Price),
    /// Listed with no price.
    NoPrice,
    /// A manual disposition: leave this out of the shop. Never denies an
    /// observed game price (C69).
    Ignore,
}

impl Buyout {
    /// The price, for the two shapes that carry one.
    pub fn price(&self) -> Option<&Price> {
        match self {
            Buyout::Exact(p) | Buyout::Negotiable(p) => Some(p),
            Buyout::NoPrice | Buyout::Ignore => None,
        }
    }

    /// The `type` word.
    pub fn kind(&self) -> &'static str {
        match self {
            Buyout::Exact(_) => "exact",
            Buyout::Negotiable(_) => "negotiable",
            Buyout::NoPrice => "no_price",
            Buyout::Ignore => "ignore",
        }
    }

    /// The value as it is written: the v1 wire shape, canonical text.
    pub fn to_value(&self) -> Value {
        // A wire struct of strings and options serializes without failing.
        serde_json::to_value(self).unwrap_or(Value::Null)
    }
}

impl fmt::Display for Buyout {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Buyout::Exact(p) => write!(f, "{p}"),
            Buyout::Negotiable(p) => write!(f, "{p} b/o"),
            Buyout::NoPrice => write!(f, "no price"),
            Buyout::Ignore => write!(f, "ignore"),
        }
    }
}

/// The v1 wire shape: every field explicit, unknown fields refused. The
/// combination rules (a price iff the type carries one) are checked in
/// [`Buyout::parse`], where the message can name what is wrong.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct BuyoutWireV1 {
    version: i64,
    #[serde(rename = "type")]
    kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    amount: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    currency: Option<String>,
}

impl Serialize for Buyout {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let price = self.price();
        BuyoutWireV1 {
            version: BUYOUT_VERSION,
            kind: self.kind().into(),
            amount: price.map(|p| p.amount.to_string()),
            currency: price.map(|p| p.currency.clone()),
        }
        .serialize(s)
    }
}

impl IntentValue for Buyout {
    const KIND: &'static str = BUYOUT_KIND;
    const VERSION: i64 = BUYOUT_VERSION;

    /// The strict parse: the v1 shape, the type word, the amount's
    /// grammar, and a currency that is a tag of the shipped table.
    fn parse(value: &Value) -> Result<Buyout, String> {
        let wire: BuyoutWireV1 =
            serde_json::from_value(value.clone()).map_err(|e| e.to_string())?;
        if wire.version != BUYOUT_VERSION {
            return Err(format!(
                "declares version {}, not this build's v{BUYOUT_VERSION}",
                wire.version
            ));
        }
        let carries_price = match wire.kind.as_str() {
            "exact" | "negotiable" => true,
            "no_price" | "ignore" => false,
            other => {
                return Err(format!(
                    "type {other:?} is not one of exact, negotiable, no_price, ignore"
                ));
            }
        };
        if !carries_price {
            if wire.amount.is_some() || wire.currency.is_some() {
                return Err(format!(
                    "type {:?} carries no amount and no currency",
                    wire.kind
                ));
            }
            return Ok(match wire.kind.as_str() {
                "no_price" => Buyout::NoPrice,
                _ => Buyout::Ignore,
            });
        }
        let amount = wire
            .amount
            .as_deref()
            .ok_or_else(|| format!("type {:?} needs an amount", wire.kind))?
            .parse::<Amount>()
            .map_err(|e| e.to_string())?;
        let word = wire
            .currency
            .as_deref()
            .ok_or_else(|| format!("type {:?} needs a currency", wire.kind))?;
        let table = currency::table().map_err(|e| e.to_string())?;
        let currency = match table.by_tag(word) {
            Some(row) => row.tag.clone(),
            None => match table.resolve(word) {
                Some(row) => {
                    return Err(format!(
                        "currency {word:?} is not a tag; it resolves to tag {:?} (currency table v{}) — intent cites the tag",
                        row.tag,
                        table.version()
                    ));
                }
                None => {
                    return Err(format!(
                        "currency {word:?} is not in currency table v{}",
                        table.version()
                    ));
                }
            },
        };
        let price = Price { amount, currency };
        Ok(match wire.kind.as_str() {
            "exact" => Buyout::Exact(price),
            _ => Buyout::Negotiable(price),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use acquisition_store::{AnnotationError, Annotations, Provenance, ValueError, check_value};
    use serde_json::json;

    fn exact(amount: &str, currency: &str) -> Buyout {
        Buyout::Exact(Price {
            amount: amount.parse().unwrap(),
            currency: currency.into(),
        })
    }

    /// C67 — the target's key carries the realm for tabs and substashes,
    /// and every address this renders parses back to the same target.
    #[test]
    fn c67_the_target_key_carries_the_realm_and_round_trips() {
        let targets = [
            (
                PriceTarget::Item {
                    id: "abc123".into(),
                },
                "item",
                "abc123",
            ),
            (
                PriceTarget::Character {
                    id: "deadbeef".into(),
                },
                "character",
                "deadbeef",
            ),
            (
                PriceTarget::Tab {
                    realm: Realm::Pc,
                    id: "t1".into(),
                },
                "tab",
                "pc/t1",
            ),
            (
                PriceTarget::Tab {
                    realm: Realm::Xbox,
                    id: "t1".into(),
                },
                "tab",
                "xbox/t1",
            ),
            (
                PriceTarget::Substash {
                    realm: Realm::Pc,
                    parent: "m1".into(),
                    id: "s9".into(),
                },
                "substash",
                "pc/m1/s9",
            ),
        ];
        for (target, scope, key) in targets {
            assert_eq!(
                target.address().unwrap(),
                (scope, key.to_string()),
                "{target}"
            );
            assert_eq!(PriceTarget::from_address(scope, key).unwrap(), target);
            assert_eq!(target.to_string(), format!("{scope}/{key}"));
        }
        // The same tab id under two realms is two rows (C58).
        let pc = PriceTarget::Tab {
            realm: Realm::Pc,
            id: "t1".into(),
        };
        let xbox = PriceTarget::Tab {
            realm: Realm::Xbox,
            id: "t1".into(),
        };
        assert_ne!(pc.address().unwrap(), xbox.address().unwrap());
        // pc is spelled out in the key: the wire's omission is not a key rule.
        assert!(pc.address().unwrap().1.starts_with("pc/"));
    }

    /// C67 — an address that could not be read back is refused at render,
    /// and a key that does not fit its scope is refused at parse.
    #[test]
    fn c67_a_target_that_cannot_round_trip_is_refused() {
        for bad in [
            PriceTarget::Item { id: "".into() },
            PriceTarget::Item { id: "a/b".into() },
            PriceTarget::Tab {
                realm: Realm::Pc,
                id: "".into(),
            },
            PriceTarget::Substash {
                realm: Realm::Pc,
                parent: "p/q".into(),
                id: "s".into(),
            },
        ] {
            assert!(
                matches!(bad.address(), Err(TargetError::MalformedComponent { .. })),
                "{bad:?}"
            );
        }
        assert_eq!(
            PriceTarget::from_address("tab", "t1"),
            Err(TargetError::MalformedKey {
                scope: "tab",
                key: "t1".into()
            })
        );
        assert_eq!(
            PriceTarget::from_address("tab", "ps5/t1"),
            Err(TargetError::UnknownRealm {
                realm: "ps5".into()
            })
        );
        assert!(matches!(
            PriceTarget::from_address("item", "a/b"),
            Err(TargetError::MalformedKey { .. })
        ));
        assert!(matches!(
            PriceTarget::from_address("substash", "pc//s"),
            Err(TargetError::MalformedComponent { .. })
        ));
        assert_eq!(
            PriceTarget::from_address("account", ""),
            Err(TargetError::UnknownScope {
                scope: "account".into()
            })
        );
    }

    /// C67 — the amount's grammar: two-place decimal or unreduced lot
    /// pair, canonical text, structural equality, more digits refused.
    #[test]
    fn c67_the_amount_is_a_two_place_decimal_or_an_unreduced_lot_pair() {
        let dec = |h: u64| Amount::Decimal { hundredths: h };
        let ratio = |w: u64, l: u64| Amount::Ratio { wanted: w, lot: l };
        for (text, amount, canonical) in [
            ("1", dec(100), "1"),
            ("150", dec(15000), "150"),
            ("2.5", dec(250), "2.5"),
            ("2.50", dec(250), "2.5"),
            ("2.05", dec(205), "2.05"),
            ("0.5", dec(50), "0.5"),
            ("0.01", dec(1), "0.01"),
            ("22/10", ratio(22, 10), "22/10"),
            ("55/600", ratio(55, 600), "55/600"),
            ("3/1", ratio(3, 1), "3/1"),
        ] {
            let parsed: Amount = text.parse().unwrap_or_else(|e| panic!("{e}"));
            assert_eq!(parsed, amount, "{text}");
            assert_eq!(parsed.to_string(), canonical, "{text}");
        }
        // Structural equality: a trailing zero is the same decimal; a
        // reduced ratio is a different ratio; a ratio over 1 is not a
        // decimal.
        assert_eq!("2.50".parse::<Amount>().unwrap(), "2.5".parse().unwrap());
        assert_ne!("22/10".parse::<Amount>().unwrap(), "11/5".parse().unwrap());
        assert_ne!("3/1".parse::<Amount>().unwrap(), "3".parse().unwrap());
        for (bad, why) in [
            ("2.505", "two fractional"),
            ("2.", "end in"),
            (".5", "digits"),
            ("02", "digits"),
            ("0", "positive"),
            ("0.00", "positive"),
            ("-1", "digits"),
            ("1 ", "digits"),
            ("1,5", "digits"),
            ("", "digits"),
            ("0/5", "positive integers"),
            ("5/0", "positive integers"),
            ("2.5/10", "positive integers"),
            ("1/2/3", "positive integers"),
            ("99999999999999999999", "too large"),
            ("1/99999999999999999999", "too large"),
        ] {
            let err = bad.parse::<Amount>().unwrap_err();
            assert!(err.why.contains(why), "{bad:?}: {err}");
        }
    }

    /// C67 — the four shapes, their JSON, and the exact round-trip
    /// through the store's generic: a v1 value re-serializes to what was
    /// read, so a non-canonical spelling is refused naming the path.
    #[test]
    fn c67_the_buyout_value_has_four_shapes_and_round_trips_exactly() {
        let cases = [
            (
                exact("12.5", "chaos"),
                json!({ "version": 1, "type": "exact", "amount": "12.5", "currency": "chaos" }),
            ),
            (
                Buyout::Negotiable(Price {
                    amount: "22/10".parse().unwrap(),
                    currency: "divine".into(),
                }),
                json!({ "version": 1, "type": "negotiable", "amount": "22/10", "currency": "divine" }),
            ),
            (Buyout::NoPrice, json!({ "version": 1, "type": "no_price" })),
            (Buyout::Ignore, json!({ "version": 1, "type": "ignore" })),
        ];
        for (buyout, wire) in cases {
            assert_eq!(buyout.to_value(), wire, "{buyout}");
            assert_eq!(check_value::<Buyout>(&wire).unwrap(), buyout);
        }
        // A trailing zero is the same amount in memory but not the
        // canonical text: refused at the door, naming the path, so the
        // stored text is always the one spelling.
        let err = check_value::<Buyout>(
            &json!({ "version": 1, "type": "exact", "amount": "12.50", "currency": "chaos" }),
        )
        .unwrap_err();
        assert_eq!(
            err,
            ValueError::NotCanonical {
                kind: "buyout",
                path: "/amount".into(),
                read: Some(json!("12.50")),
                canonical: Some(json!("12.5")),
            }
        );
        // A JSON number is not an amount: text only, so 2.50 cannot
        // become 2.5 by accident.
        let err = check_value::<Buyout>(
            &json!({ "version": 1, "type": "exact", "amount": 12.5, "currency": "chaos" }),
        )
        .unwrap_err();
        assert!(matches!(err, ValueError::Malformed { .. }), "{err}");
        // Display, for a human reading a row.
        assert_eq!(exact("12.5", "chaos").to_string(), "12.5 chaos");
        assert_eq!(
            Buyout::Negotiable(Price {
                amount: "3/1".parse().unwrap(),
                currency: "exalted".into()
            })
            .to_string(),
            "3/1 exalted b/o"
        );
    }

    /// C67 — what the value refuses: an unknown type, a price on a shape
    /// that carries none (the 0.18 `[ignore]` rows' 4321 blessed), a
    /// missing amount or currency, a third digit, an unknown field, a
    /// version that is not 1.
    #[test]
    fn c67_the_buyout_parse_is_strict() {
        let refused = |v: Value, why: &str| {
            let err = check_value::<Buyout>(&v).unwrap_err();
            assert!(err.to_string().contains(why), "{v}: {err}");
        };
        refused(
            json!({ "version": 1, "type": "b/o", "amount": "1", "currency": "chaos" }),
            "not one of",
        );
        refused(
            json!({ "version": 1, "type": "ignore", "amount": "4321", "currency": "blessed" }),
            "carries no amount",
        );
        refused(
            json!({ "version": 1, "type": "no_price", "currency": "chaos" }),
            "carries no amount",
        );
        refused(
            json!({ "version": 1, "type": "exact", "currency": "chaos" }),
            "needs an amount",
        );
        refused(
            json!({ "version": 1, "type": "exact", "amount": "1" }),
            "needs a currency",
        );
        refused(
            json!({ "version": 1, "type": "exact", "amount": "1.005", "currency": "chaos" }),
            "two fractional",
        );
        refused(
            json!({ "version": 1, "type": "exact", "amount": "1", "currency": "chaos", "note": "x" }),
            "note",
        );
        refused(json!({ "version": 0, "type": "ignore" }), "version 0");
        refused(json!({ "type": "ignore" }), "missing integer `version`");
        assert_eq!(
            check_value::<Buyout>(&json!({ "version": 2, "type": "ignore" })).unwrap_err(),
            ValueError::VersionUnsupported {
                kind: "buyout",
                found: 2,
                supported: 1
            }
        );
    }

    /// C67 — the currency is a tag of the shipped table (C68): an alias
    /// is refused naming the tag, an unknown word naming the version, and
    /// a retired tag still parses (a stored row may cite it forever).
    #[test]
    fn c67_the_currency_must_be_a_tag_of_the_reference_table() {
        let value =
            |word: &str| json!({ "version": 1, "type": "exact", "amount": "1", "currency": word });
        assert_eq!(
            check_value::<Buyout>(&value("chaos")).unwrap(),
            exact("1", "chaos")
        );
        // The legacy C++ tag is an alias of the game's word; the message
        // names the tag to cite instead.
        let err = check_value::<Buyout>(&value("exa"))
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("not a tag") && err.contains("\"exalted\""),
            "{err}"
        );
        let err = check_value::<Buyout>(&value("Chaos"))
            .unwrap_err()
            .to_string();
        assert!(err.contains("not in currency table v1"), "{err}");
        // Retired: parses; refusing a new one is the writer's rule.
        assert_eq!(
            check_value::<Buyout>(&value("chisel")).unwrap(),
            exact("1", "chisel")
        );
    }

    /// C66/C67 — through the store: a price lands typed, reads back typed
    /// and raw, and a `clear` then `set` on one target works through the
    /// tombstone with the revision carrying on (2b constraint (3), reduced).
    #[test]
    fn c67_clear_then_set_on_one_target_works_through_the_tombstone() {
        let mut a = Annotations::open_memory_for("u-1").unwrap();
        let via = Provenance::via("test");
        let target = PriceTarget::Tab {
            realm: Realm::Pc,
            id: "t1".into(),
        };
        let (scope, key) = target.address().unwrap();
        // set
        let row = a
            .put::<Buyout>(scope, &key, &exact("30", "chaos").to_value(), None, &via)
            .unwrap();
        assert_eq!(row.revision, 1);
        let (read, typed) = a.get_as::<Buyout>(scope, &key).unwrap().unwrap();
        assert_eq!((read.revision, typed), (1, exact("30", "chaos")));
        assert_eq!(
            PriceTarget::from_address(&read.scope, &read.key).unwrap(),
            target
        );
        // clear
        a.delete(scope, &key, BUYOUT_KIND, 1, &via).unwrap();
        assert!(a.get_as::<Buyout>(scope, &key).unwrap().is_none());
        // set again, as a caller who read "nothing there" would: a create.
        let row = a
            .put::<Buyout>(scope, &key, &Buyout::Ignore.to_value(), None, &via)
            .unwrap();
        assert_eq!(row.revision, 3, "the tombstone's revision carries on");
        let (_, typed) = a.get_as::<Buyout>(scope, &key).unwrap().unwrap();
        assert_eq!(typed, Buyout::Ignore);
        // A price that does not parse never lands, even over a tombstone.
        a.delete(scope, &key, BUYOUT_KIND, 3, &via).unwrap();
        let err = a
            .put::<Buyout>(
                scope,
                &key,
                &json!({ "version": 1, "type": "exact", "amount": "1", "currency": "exa" }),
                None,
                &via,
            )
            .unwrap_err();
        assert!(matches!(err, AnnotationError::Invalid(_)), "{err}");
        assert!(a.get(scope, &key, BUYOUT_KIND).unwrap().is_none());
        // One kind across every price scope: the list a pricing read wants.
        for t in [
            PriceTarget::Item { id: "i1".into() },
            PriceTarget::Character { id: "c1".into() },
            PriceTarget::Substash {
                realm: Realm::Pc,
                parent: "m1".into(),
                id: "s1".into(),
            },
        ] {
            let (scope, key) = t.address().unwrap();
            a.put::<Buyout>(scope, &key, &Buyout::NoPrice.to_value(), None, &via)
                .unwrap();
        }
        let rows = a.list(None, Some(BUYOUT_KIND)).unwrap();
        let targets: Vec<PriceTarget> = rows
            .iter()
            .map(|r| PriceTarget::from_address(&r.scope, &r.key).unwrap())
            .collect();
        assert_eq!(targets.len(), 3);
        assert!(
            targets
                .iter()
                .all(|t| !matches!(t, PriceTarget::Tab { .. }))
        );
    }
}
