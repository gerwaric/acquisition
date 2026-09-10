//! The real-scale pricing fixture (plan step 7, item 2, `PRICING-SLICE.md`)
//! for the text views' time guards: `acquisition-plan`'s
//! `reference/pricing-snapshot-2026-09-07.json` — the owner's pc/Standard
//! league, names and ids hashed — tiled to the scale at which a
//! quadratic view is unmistakable. Test-only.

use acquisition_plan::price::PriceTarget;
use acquisition_protocol::realm::Realm;
use acquisition_store::{ItemSnapshot, PricingSnapshot};

const PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../acquisition-plan/reference/pricing-snapshot-2026-09-07.json"
);

/// The tab the owner priced by hand for validation reading 2, under the
/// name they gave it: the one public tab with a row on it, 80 items.
const OWNERS_TEST_TAB: &str = "ACQUISITION-PRICE-TEST";

fn snapshot() -> PricingSnapshot {
    let text = std::fs::read_to_string(PATH).unwrap_or_else(|e| panic!("{PATH}: {e}"));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("{PATH}: {e}"))
}

/// The fixture with its items repeated `times` over — each copy a
/// distinct item at the same location and position — so the views run
/// over tens of thousands of items, the scale the step-6 review's
/// quadratic `--expand` was found at (26k). Only the original items are
/// named by a row; the copies inherit what their containers say.
pub(crate) fn tiled(times: usize) -> PricingSnapshot {
    let mut s = snapshot();
    let originals = s.items.clone();
    for k in 1..times {
        s.items.extend(originals.iter().map(|i| ItemSnapshot {
            id: format!("{}-{k}", i.id),
            socketed_in: i.socketed_in.as_ref().map(|p| format!("{p}-{k}")),
            ..i.clone()
        }));
    }
    s
}

/// The target of the owner's test tab in the snapshot.
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
