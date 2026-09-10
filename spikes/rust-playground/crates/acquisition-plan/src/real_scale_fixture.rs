//! The real-scale pricing fixture (`reference/pricing-snapshot-2026-09-07.json`;
//! plan step 7, item 2, `PRICING-SLICE.md`): the owner's pc/Standard
//! league as `redact-pricing` wrote it — 402 tabs, 41 characters, 1,977
//! items, 16 buyout rows — every name and id hashed, every note, tab
//! name, position, timestamp and buyout value kept. Read by the tests
//! that run the listing state (`listing.rs`) and the render (`shop.rs`)
//! at the scale of a real account. Test-only; the fixture is evidence,
//! never shipped data.

use acquisition_protocol::realm::Realm;
use acquisition_store::PricingSnapshot;

use crate::price::PriceTarget;

const PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/reference/pricing-snapshot-2026-09-07.json"
);

/// The tab the owner priced by hand for validation reading 2, under the
/// name they gave it (the writer keeps tab names): the one public tab
/// with a row on it, 80 items, 50 of them with a note.
pub(crate) const OWNERS_TEST_TAB: &str = "ACQUISITION-PRICE-TEST";

pub(crate) fn snapshot() -> PricingSnapshot {
    let text = std::fs::read_to_string(PATH).unwrap_or_else(|e| panic!("{PATH}: {e}"));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("{PATH}: {e}"))
}

/// The target of [`OWNERS_TEST_TAB`] in the snapshot.
pub(crate) fn owners_test_tab(s: &PricingSnapshot) -> PriceTarget {
    let tab = s
        .tabs
        .iter()
        .find(|t| t.name == OWNERS_TEST_TAB)
        .unwrap_or_else(|| panic!("{PATH}: no tab named {OWNERS_TEST_TAB}"));
    PriceTarget::Tab {
        realm: Realm::parse(&s.realm).unwrap(),
        id: tab.id.clone(),
    }
}
