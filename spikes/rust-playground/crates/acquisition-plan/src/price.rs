//! The typed price: what a `buyout` row says ([`Buyout`], C67) and what
//! it is about ([`PriceTarget`]). Built by the pricing slice's plan step 2
//! (`PRICING-SLICE.md`, 2026-09-05). Nothing here reads facts or a
//! daemon; this module is the value's shape, its parse, its address, and
//! — since plan step 5 (2026-09-06) — the one write path every frontend's
//! `set` and `clear` use ([`set_buyout`], [`clear_buyout`]), through the
//! intent file's own API and nothing else.
//!
//! # Decisions as recorded
//!
//! **C67 — The v1 `buyout` and its target.** A typed `PriceTarget` — item
//! and character by id (C55), tab by `(realm, id)`, substash by `(realm,
//! parent, id)`, league-less (a merge keeps the row) — is the public API.
//! `type` is `exact` (`~price`), `negotiable` (`~b/o`), `no_price` or
//! `skip` (`~skip`); the first two carry `amount` and a `currency` tag that must
//! resolve in the reference table (C68). `amount` is a decimal of at most
//! four fractional digits (T10), or a lot ratio `wanted/lot` (T2) of two
//! unreduced positive integers; canonical text, structural equality; more
//! digits are refused. *Why:* the key mirrors the store's identity (C54,
//! C58); the game writes four places (T10), two ruled 2026-09-04; widening
//! is compatible, narrowing is not. *Details:* `price.rs`. *Pinned:* the
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
//! `{"version":1,"type":"skip"}`. `amount` is text, never a JSON
//! number, so `2.50` and `2.5` cannot silently become one float: as text
//! the canonical spelling is the shortest — no leading zeros, no trailing
//! fraction zeros, no trailing dot — and a ratio is `wanted/lot` of two
//! positive integers kept exactly as written (`22/10` is not `11/5`, and
//! `3/1` is not `3`: the lot is a lot). [`Amount`] compares structurally
//! on those parts, so `"12.50"` typed by a human ([`Amount::from_str`])
//! equals `"12.5"`, while the JSON door holds a v1 value to its canonical
//! text through the store's exact round-trip (C66) — a non-canonical
//! spelling in a value is refused there naming the path, not rewritten.
//! A decimal has at most **four** fractional digits — what the game
//! itself writes (T10: `999.1234` in the fixture) — held as
//! ten-thousandths, so the same type carries an observed note's amount
//! at plan step 3 and a manual price, and the listing state (step 4,
//! `listing.rs`) compares them structurally; a fifth digit is refused. `0`, `0.00` and `0/5` are
//! refused (a price is positive); `10/0` is refused. (v1 was ruled at
//! two places on 2026-09-04 and widened to four on 2026-09-06 after an
//! outside review; widening is compatible, every stored two-place value
//! still parses.)
//!
//! `currency` is a **tag** of the shipped currency table
//! ([`crate::currency`]) — the immutable identity intent cites (C68),
//! never an alias or a display name. A word that resolves to a row by
//! alias or emit but is not that row's tag is refused naming the tag, so
//! the frontend can offer it; a word the table does not know is refused
//! naming the table version. A **retired** tag parses: a stored row may
//! cite it forever, and whether a *new* price may name one is the
//! writer's rule ([`Buyout::check_write`], run at the store's door on
//! every write and never on a read), not the parse's. `no_price`
//! and `skip` carry no amount and no currency; a value that supplies
//! either is refused (the 0.18 userstore's `[ignore]` rows carried a
//! non-semantic 4321 `blessed`, which is exactly the shape this refuses).
//!
//! [`Buyout`] is an [`IntentValue`], so the store's write door runs the
//! version gate, this parse and the exact round-trip before the
//! compare-and-swap; nothing lands that this module would not read back.
//!
//! **The write** ([`set_buyout`], [`clear_buyout`]) is a single-row
//! compare-and-swap through that door, built once for every frontend the
//! way `put_sync_policy` is: the caller passes the revision it reviewed
//! (or `None` to create), the frontend decides whether "whatever is
//! stored" is an acceptable default, and what comes back is a
//! [`PriceWrite`] — the row written and the row it replaced, raw with
//! provenance — which is C78's clause until receipts exist: a hand can
//! undo what the command printed. The one writer's rule C67 leaves
//! open, that a *new* price never names a retired tag, is the value's
//! [`Buyout::check_write`], which the store's door runs on every write
//! and never on a read (2026-09-07, after an outside review found the
//! rule bypassable through `Annotations::put`): a stored row citing one
//! still parses, and no frontend can write past it.

use std::fmt;
use std::str::FromStr;

use acquisition_protocol::realm::Realm;
use acquisition_store::{
    AnnotationError, AnnotationRow, Annotations, FactAddress, IntentValue, Provenance,
};
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

/// The store's address for the same identity (C67 mirrors C54 and C58),
/// so a frontend can ask the facts whether they hold the target.
impl<'a> From<&'a PriceTarget> for FactAddress<'a> {
    fn from(target: &'a PriceTarget) -> Self {
        match target {
            PriceTarget::Item { id } => FactAddress::Item { id },
            PriceTarget::Character { id } => FactAddress::Character { id },
            PriceTarget::Tab { realm, id } => FactAddress::Tab {
                realm: realm.as_str(),
                id,
            },
            PriceTarget::Substash { realm, parent, id } => FactAddress::Substash {
                realm: realm.as_str(),
                parent,
                id,
            },
        }
    }
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

impl FromStr for PriceTarget {
    type Err = TargetError;

    /// The address as one word, `scope/key` — what [`PriceTarget`]'s
    /// `Display` writes: `item/<id>`, `character/<id>`, `tab/<realm>/<id>`,
    /// `substash/<realm>/<parent>/<id>`.
    fn from_str(word: &str) -> Result<PriceTarget, TargetError> {
        match word.split_once('/') {
            Some((scope, key)) => PriceTarget::from_address(scope, key),
            None => Err(TargetError::UnknownScope { scope: word.into() }),
        }
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

/// How much: a decimal of at most four fractional digits, held as
/// ten-thousandths, or a bulk ratio `wanted/lot` (T2) kept unreduced.
/// Equality is structural on those parts — `2.50` and `2.5` are one
/// amount, `22/10` and `11/5` are two.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Amount {
    /// A positive decimal, in ten-thousandths: `12.5` is `125000`.
    Decimal { ten_thousandths: u64 },
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
    /// tolerated and dropped), `999.1234`, `22/10`. Refused: a fifth
    /// fractional digit (never rounded), a leading zero, a sign,
    /// whitespace, zero, a zero lot, and anything that is not digits, one
    /// `.` or one `/`.
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
        if fraction.len() > 4 {
            return Err(err("at most four fractional digits (refused, not rounded)"));
        }
        if !fraction.bytes().all(|b| b.is_ascii_digit()) {
            return Err(err(
                "a decimal is digits, optionally `.` and one to four digits",
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
                        "a decimal is digits, optionally `.` and one to four digits",
                    ));
                }
            }
        };
        // The fraction, right-padded to four places: `5` → 5000, `1234` → 1234.
        let fraction_part: u64 = if fraction.is_empty() {
            0
        } else {
            let digits = fraction.parse::<u64>().map_err(|_| err("not a decimal"))?;
            digits * 10u64.pow(4 - fraction.len() as u32)
        };
        let ten_thousandths = units
            .checked_mul(10_000)
            .and_then(|t| t.checked_add(fraction_part))
            .ok_or_else(|| err("too large"))?;
        if ten_thousandths == 0 {
            return Err(err("a price is positive"));
        }
        Ok(Amount::Decimal { ten_thousandths })
    }
}

impl fmt::Display for Amount {
    /// The canonical text: the shortest decimal, or `wanted/lot` verbatim.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Amount::Decimal { ten_thousandths } => {
                let (units, mut frac) = (ten_thousandths / 10_000, ten_thousandths % 10_000);
                if frac == 0 {
                    return write!(f, "{units}");
                }
                let mut places = 4;
                while frac % 10 == 0 {
                    frac /= 10;
                    places -= 1;
                }
                write!(f, "{units}.{frac:0places$}")
            }
            Amount::Ratio { wanted, lot } => write!(f, "{wanted}/{lot}"),
        }
    }
}

impl Serialize for Amount {
    /// The canonical text — never a JSON number (the module doc's reason).
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for Amount {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Amount, D::Error> {
        let text = String::deserialize(d)?;
        text.parse().map_err(serde::de::Error::custom)
    }
}

/// An amount in a currency: the tag intent cites (C68).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
    /// `~skip`'s word: leave this out of the shop — the forum analogue of
    /// the game's "Do not index". Renamed from `ignore` on 2026-09-06
    /// (C67), before any row existed.
    Skip,
}

impl Buyout {
    /// The price, for the two shapes that carry one.
    pub fn price(&self) -> Option<&Price> {
        match self {
            Buyout::Exact(p) | Buyout::Negotiable(p) => Some(p),
            Buyout::NoPrice | Buyout::Skip => None,
        }
    }

    /// The `type` word.
    pub fn kind(&self) -> &'static str {
        match self {
            Buyout::Exact(_) => "exact",
            Buyout::Negotiable(_) => "negotiable",
            Buyout::NoPrice => "no_price",
            Buyout::Skip => "skip",
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
            Buyout::Skip => write!(f, "skip"),
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

impl<'de> Deserialize<'de> for Buyout {
    /// The same strict parse as the write door, so a report or a plan
    /// that carries a value re-reads it exactly.
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Buyout, D::Error> {
        let value = Value::deserialize(d)?;
        Buyout::parse(&value).map_err(serde::de::Error::custom)
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
            "no_price" | "skip" => false,
            other => {
                return Err(format!(
                    "type {other:?} is not one of exact, negotiable, no_price, skip"
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
                _ => Buyout::Skip,
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

    /// The writer's rule (C67): a new price never names a retired tag,
    /// though a stored row may cite one forever. Run by the store's door
    /// on every write, through every frontend, and never on a read.
    fn check_write(&self) -> Result<(), String> {
        let Some(price) = self.price() else {
            return Ok(());
        };
        let table = currency::table().map_err(|e| e.to_string())?;
        match table
            .by_tag(&price.currency)
            .and_then(|row| row.retired.as_deref())
        {
            Some(retired) => Err(format!(
                "currency {:?} is retired ({retired}): a new price cannot name it, though a stored row may",
                price.currency
            )),
            None => Ok(()),
        }
    }
}

/// The outcome of one write, until receipts exist (C78): the row as
/// written — `None` after a clear — and the row it replaced — `None` on a
/// create — so what a command printed is enough to undo it by hand. Both
/// are the store's rows, raw, with their provenance; on success `prior`
/// is exactly the row the write replaced: a live row that moved between
/// the read and the write conflicts (C35), and a create replaces nothing.
/// `prior.value` may be one this build cannot read (a newer build's,
/// written past by an explicit revision): a frontend reads it through
/// [`Buyout::parse`], shows the JSON when that fails, and does not
/// promise to put it back.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PriceWrite {
    pub target: PriceTarget,
    pub written: Option<AnnotationRow>,
    pub prior: Option<AnnotationRow>,
}

/// Why a price write did not land. The value's own refusals — the strict
/// parse and the writer's rule ([`Buyout::check_write`], C67) — arrive
/// as [`AnnotationError::Invalid`] under `Store`.
#[derive(Debug)]
pub enum PriceWriteError {
    /// The target's components cannot be an address.
    Target(TargetError),
    /// The compare-and-swap, the busy timeout, the file: the store's own.
    Store(AnnotationError),
}

impl fmt::Display for PriceWriteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PriceWriteError::Target(e) => write!(f, "{e}"),
            PriceWriteError::Store(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for PriceWriteError {}

impl From<TargetError> for PriceWriteError {
    fn from(e: TargetError) -> PriceWriteError {
        PriceWriteError::Target(e)
    }
}

impl From<AnnotationError> for PriceWriteError {
    fn from(e: AnnotationError) -> PriceWriteError {
        PriceWriteError::Store(e)
    }
}

/// Write one price under compare-and-swap (C35) — the one write path
/// every frontend's `set` uses: the value goes through the store's typed
/// door as its canonical text (C66), where the writer's rule runs too
/// ([`Buyout::check_write`]: a new price never names a retired tag, C67
/// — the door's, so `Annotations::put` cannot be used to write past it),
/// and the row it replaced comes back (C78). No daemon, no quote, no job
/// (C64).
///
/// `expected_revision` is the store's compare-and-swap, verbatim:
/// `Some(r)` replaces exactly the revision the caller reviewed, and a row
/// that moved conflicts; `None` creates, refused if a live row exists — a
/// tombstone of any generation does not count, so a `clear` then `set`
/// works without the caller knowing it is there, and so a create cannot
/// tell which clear it follows. A frontend that wants "replace whatever
/// is stored" reads the current row itself and passes its revision here
/// — the blind form is a frontend policy, not this function's, the same
/// rule as [`crate::put_sync_policy`]; what the frontend owes that read
/// is to refuse a row it cannot parse (a newer build's), since a blind
/// replacement of one destroys intent this build cannot restore.
pub fn set_buyout(
    annotations: &mut Annotations,
    target: &PriceTarget,
    value: &Buyout,
    expected_revision: Option<i64>,
    provenance: &Provenance,
) -> Result<PriceWrite, PriceWriteError> {
    let (scope, key) = target.address()?;
    let prior = annotations.get(scope, &key, BUYOUT_KIND)?;
    let written = annotations.put::<Buyout>(
        scope,
        &key,
        &value.to_value(),
        expected_revision,
        provenance,
    )?;
    Ok(PriceWrite {
        target: target.clone(),
        written: Some(written),
        prior,
    })
}

/// Remove one price row under the same compare-and-swap, returning the
/// row it removed (C78). `expected_revision` is exact: what the caller
/// reviewed; the store tombstones the row, so the revision sequence
/// carries on and a stale writer still conflicts. Whether a target with
/// no row is a refusal is the frontend's call — the store reports it as a
/// conflict carrying nothing.
pub fn clear_buyout(
    annotations: &mut Annotations,
    target: &PriceTarget,
    expected_revision: i64,
    provenance: &Provenance,
) -> Result<PriceWrite, PriceWriteError> {
    let (scope, key) = target.address()?;
    let prior = annotations.get(scope, &key, BUYOUT_KIND)?;
    annotations.delete(scope, &key, BUYOUT_KIND, expected_revision, provenance)?;
    Ok(PriceWrite {
        target: target.clone(),
        written: None,
        prior,
    })
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

    /// A `buyout` as an older build stored it: the door's parse and the
    /// round-trip, no writer's rule — how a row citing a since-retired
    /// tag got there.
    #[derive(Serialize)]
    #[serde(transparent)]
    struct StoredBuyout(Value);

    impl IntentValue for StoredBuyout {
        const KIND: &'static str = BUYOUT_KIND;
        const VERSION: i64 = BUYOUT_VERSION;
        fn parse(value: &Value) -> Result<Self, String> {
            Buyout::parse(value)?;
            Ok(StoredBuyout(value.clone()))
        }
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

    /// C67 — the amount's grammar: a decimal of at most four places (what
    /// the game writes, T10) or an unreduced lot pair, canonical text,
    /// structural equality, more digits refused.
    #[test]
    fn c67_the_amount_is_a_four_place_decimal_or_an_unreduced_lot_pair() {
        let dec = |t: u64| Amount::Decimal { ten_thousandths: t };
        let ratio = |w: u64, l: u64| Amount::Ratio { wanted: w, lot: l };
        for (text, amount, canonical) in [
            ("1", dec(10_000), "1"),
            ("150", dec(1_500_000), "150"),
            ("12345", dec(123_450_000), "12345"),
            ("2.5", dec(25_000), "2.5"),
            ("2.50", dec(25_000), "2.5"),
            ("2.05", dec(20_500), "2.05"),
            ("0.5", dec(5_000), "0.5"),
            ("0.01", dec(100), "0.01"),
            ("0.0001", dec(1), "0.0001"),
            // The fixture's own decimals (price-notes-2026-09-04.txt).
            ("999.1", dec(9_991_000), "999.1"),
            ("999.12", dec(9_991_200), "999.12"),
            ("999.123", dec(9_991_230), "999.123"),
            ("999.1234", dec(9_991_234), "999.1234"),
            ("999.1230", dec(9_991_230), "999.123"),
            ("22/10", ratio(22, 10), "22/10"),
            ("55/600", ratio(55, 600), "55/600"),
            ("3/1", ratio(3, 1), "3/1"),
        ] {
            let parsed: Amount = text.parse().unwrap_or_else(|e| panic!("{e}"));
            assert_eq!(parsed, amount, "{}", text);
            assert_eq!(parsed.to_string(), canonical, "{}", text);
        }
        // Structural equality: a trailing zero is the same decimal; a
        // reduced ratio is a different ratio; a ratio over 1 is not a
        // decimal.
        assert_eq!("2.50".parse::<Amount>().unwrap(), "2.5".parse().unwrap());
        assert_ne!("22/10".parse::<Amount>().unwrap(), "11/5".parse().unwrap());
        assert_ne!("3/1".parse::<Amount>().unwrap(), "3".parse().unwrap());
        for (bad, why) in [
            ("2.12345", "four fractional"),
            ("999.00001", "four fractional"),
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
            (Buyout::Skip, json!({ "version": 1, "type": "skip" })),
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
            json!({ "version": 1, "type": "skip", "amount": "4321", "currency": "blessed" }),
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
            json!({ "version": 1, "type": "exact", "amount": "1.00005", "currency": "chaos" }),
            "four fractional",
        );
        refused(
            json!({ "version": 1, "type": "exact", "amount": "1", "currency": "chaos", "note": "x" }),
            "note",
        );
        refused(json!({ "version": 0, "type": "skip" }), "version 0");
        refused(json!({ "type": "skip" }), "missing integer `version`");
        assert_eq!(
            check_value::<Buyout>(&json!({ "version": 2, "type": "skip" })).unwrap_err(),
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
            .put::<Buyout>(scope, &key, &Buyout::Skip.to_value(), None, &via)
            .unwrap();
        assert_eq!(row.revision, 3, "the tombstone's revision carries on");
        let (_, typed) = a.get_as::<Buyout>(scope, &key).unwrap().unwrap();
        assert_eq!(typed, Buyout::Skip);
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

    /// C78 — until receipts exist, a single write returns the row's prior
    /// value, so it can be undone by hand: a create's prior is nothing, a
    /// replacement's is the row replaced, a clear's is the row cleared,
    /// and setting the prior back restores the value with the revision
    /// sequence carrying on (C35).
    #[test]
    fn c78_a_write_returns_the_prior_value_so_a_hand_can_undo_it() {
        let mut a = Annotations::open_memory_for("u-1").unwrap();
        let via = Provenance::via("test");
        let target = PriceTarget::Item { id: "i1".into() };
        let first = set_buyout(&mut a, &target, &exact("30", "chaos"), None, &via).unwrap();
        assert_eq!(first.target, target);
        assert_eq!(first.prior, None);
        let w1 = first.written.unwrap();
        assert_eq!((w1.revision, w1.written_via.as_str()), (1, "test"));
        assert_eq!(Buyout::parse(&w1.value).unwrap(), exact("30", "chaos"));
        let second = set_buyout(&mut a, &target, &Buyout::Skip, Some(1), &via).unwrap();
        assert_eq!(second.prior.as_ref(), Some(&w1));
        assert_eq!(second.written.as_ref().map(|r| r.revision), Some(2));
        let cleared = clear_buyout(&mut a, &target, 2, &via).unwrap();
        assert_eq!(cleared.written, None);
        assert_eq!(cleared.prior, second.written);
        assert!(a.get(ITEM_SCOPE, "i1", BUYOUT_KIND).unwrap().is_none());
        // Undo by hand: the prior value, set again, as a caller who read
        // "nothing there" would — a create over the tombstone.
        let undo = Buyout::parse(&cleared.prior.unwrap().value).unwrap();
        let restored = set_buyout(&mut a, &target, &undo, None, &via).unwrap();
        assert_eq!(restored.prior, None);
        let (row, typed) = a.get_as::<Buyout>(ITEM_SCOPE, "i1").unwrap().unwrap();
        assert_eq!((row.revision, typed), (4, Buyout::Skip));
    }

    /// C67 — whether a new price may name a retired tag is the writer's
    /// rule, not the value's: `set_buyout` refuses one naming when it was
    /// retired and lands nothing; a stored row citing it still reads, and
    /// is replaced like any other, its value returned as the prior.
    #[test]
    fn c67_a_new_price_never_names_a_retired_tag_though_a_stored_row_may() {
        let mut a = Annotations::open_memory_for("u-1").unwrap();
        let via = Provenance::via("test");
        let target = PriceTarget::Tab {
            realm: Realm::Pc,
            id: "t1".into(),
        };
        let refused = |err: PriceWriteError| match err {
            PriceWriteError::Store(AnnotationError::Invalid(ValueError::RefusedForWrite {
                kind,
                detail,
            })) => {
                assert_eq!(kind, BUYOUT_KIND);
                assert!(
                    detail.contains("\"chisel\" is retired (2026-09-04"),
                    "{detail}"
                );
            }
            other => panic!("expected RefusedForWrite, got {other}"),
        };
        refused(set_buyout(&mut a, &target, &exact("1", "chisel"), None, &via).unwrap_err());
        assert!(a.get(TAB_SCOPE, "pc/t1", BUYOUT_KIND).unwrap().is_none());
        // The rule is the door's: the store's own put refuses it too.
        let err = a
            .put::<Buyout>(
                TAB_SCOPE,
                "pc/t1",
                &exact("1", "chisel").to_value(),
                None,
                &via,
            )
            .unwrap_err();
        refused(PriceWriteError::Store(err));
        // A row written before the currency was retired: the value reads.
        a.put::<StoredBuyout>(
            TAB_SCOPE,
            "pc/t1",
            &exact("1", "chisel").to_value(),
            None,
            &via,
        )
        .unwrap();
        let (_, typed) = a.get_as::<Buyout>(TAB_SCOPE, "pc/t1").unwrap().unwrap();
        assert_eq!(typed, exact("1", "chisel"));
        let w = set_buyout(&mut a, &target, &exact("2", "chaos"), Some(1), &via).unwrap();
        assert_eq!(
            Buyout::parse(&w.prior.unwrap().value).unwrap(),
            exact("1", "chisel")
        );
    }

    /// C35 — the write is a compare-and-swap at this door too: a stale
    /// revision conflicts carrying the current row and changes nothing;
    /// so does a create over a row, and a clear at the wrong revision.
    #[test]
    fn c35_a_stale_revision_conflicts_and_changes_nothing() {
        let mut a = Annotations::open_memory_for("u-1").unwrap();
        let via = Provenance::via("test");
        let target = PriceTarget::Character { id: "c1".into() };
        let first = set_buyout(&mut a, &target, &exact("30", "chaos"), None, &via)
            .unwrap()
            .written
            .unwrap();
        for stale in [Some(5), None] {
            let err = set_buyout(&mut a, &target, &Buyout::Skip, stale, &via).unwrap_err();
            match err {
                PriceWriteError::Store(AnnotationError::Conflict { current: Some(row) }) => {
                    assert_eq!(*row, first)
                }
                other => panic!("expected a Conflict carrying the row, got {other}"),
            }
        }
        let err = clear_buyout(&mut a, &target, 7, &via).unwrap_err();
        assert!(
            matches!(
                err,
                PriceWriteError::Store(AnnotationError::Conflict { .. })
            ),
            "{err}"
        );
        let (row, typed) = a.get_as::<Buyout>(CHARACTER_SCOPE, "c1").unwrap().unwrap();
        assert_eq!((row.revision, typed), (1, exact("30", "chaos")));
        clear_buyout(&mut a, &target, 1, &via).unwrap();
        assert!(a.get(CHARACTER_SCOPE, "c1", BUYOUT_KIND).unwrap().is_none());
    }

    /// The plan step 7 property tests (`PRICING-SLICE.md`): the amount's
    /// grammar over every spelling, not the hand-picked ones above.
    mod properties {
        use super::*;
        use proptest::prelude::*;

        /// Any amount the type can hold: every positive ten-thousandth,
        /// every pair of positive integers.
        fn any_amount() -> impl Strategy<Value = Amount> {
            prop_oneof![
                (1..=u64::MAX).prop_map(|ten_thousandths| Amount::Decimal { ten_thousandths }),
                (1..=u64::MAX, 1..=u64::MAX)
                    .prop_map(|(wanted, lot)| Amount::Ratio { wanted, lot }),
            ]
        }

        proptest! {
            /// C67 — any amount's canonical text parses back to the same
            /// amount, is a fixed point of the parse (the shortest
            /// spelling: no trailing zero, no trailing `.`), and is the
            /// JSON, both ways.
            #[test]
            fn c67_any_amount_round_trips_through_its_canonical_text(a in any_amount()) {
                let text = a.to_string();
                prop_assert_eq!(text.parse::<Amount>(), Ok(a));
                prop_assert_eq!(text.parse::<Amount>().unwrap().to_string(), text.clone());
                if let Amount::Decimal { .. } = a {
                    prop_assert!(!text.ends_with('.'), "{}", text);
                    prop_assert!(!(text.contains('.') && text.ends_with('0')), "{}", text);
                }
                prop_assert_eq!(serde_json::to_value(a).unwrap(), Value::String(text.clone()));
                prop_assert_eq!(serde_json::from_value::<Amount>(json!(text)).unwrap(), a);
            }

            /// C67, T10 — any spelling of the decimal grammar (a whole
            /// part, up to four fractional digits) parses to the amount
            /// it names, zero refused; its canonical text is no longer
            /// and names the same amount.
            #[test]
            fn c67_any_spelling_of_the_decimal_grammar_parses_to_the_amount_it_names(
                whole in prop_oneof![Just("0".to_string()), "[1-9][0-9]{0,14}"],
                fraction in prop::option::of("[0-9]{1,4}"),
            ) {
                let text = match &fraction {
                    Some(f) => format!("{whole}.{f}"),
                    None => whole.clone(),
                };
                let units: u64 = whole.parse().unwrap();
                let frac: u64 = fraction.as_deref().map_or(0, |f| {
                    f.parse::<u64>().unwrap() * 10u64.pow(4 - f.len() as u32)
                });
                let want = units * 10_000 + frac;
                let got = text.parse::<Amount>();
                if want == 0 {
                    prop_assert!(got.is_err(), "{} parsed", text);
                } else {
                    prop_assert_eq!(got.clone(), Ok(Amount::Decimal { ten_thousandths: want }));
                    let canonical = got.unwrap().to_string();
                    prop_assert!(canonical.len() <= text.len(), "{} from {}", canonical, text);
                    prop_assert_eq!(canonical.parse::<Amount>(), text.parse::<Amount>());
                }
            }

            /// C67, T2 — any lot pair parses and is kept unreduced: its
            /// canonical text is the pair verbatim.
            #[test]
            fn c67_any_lot_pair_parses_and_is_kept_unreduced(
                wanted in "[1-9][0-9]{0,18}",
                lot in "[1-9][0-9]{0,18}",
            ) {
                let text = format!("{wanted}/{lot}");
                let a = text.parse::<Amount>().unwrap();
                prop_assert_eq!(a, Amount::Ratio { wanted: wanted.parse().unwrap(), lot: lot.parse().unwrap() });
                prop_assert_eq!(a.to_string(), text);
            }

            /// C47, C67 — any text parses or is refused naming the text and
            /// a reason, never panics; what parses is digits, `.` and `/`
            /// only and round-trips.
            #[test]
            fn c47_any_text_parses_or_is_refused_naming_it(text in "\\PC{0,12}") {
                match text.parse::<Amount>() {
                    Ok(a) => {
                        prop_assert!(text.bytes().all(|b| b.is_ascii_digit() || b == b'.' || b == b'/'), "{:?}", text);
                        prop_assert_eq!(a.to_string().parse::<Amount>(), Ok(a));
                    }
                    Err(e) => {
                        prop_assert_eq!(&e.text, &text);
                        prop_assert!(!e.why.is_empty());
                    }
                }
            }
        }
    }
}
