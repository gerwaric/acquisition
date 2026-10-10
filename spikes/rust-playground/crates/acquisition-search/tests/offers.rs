//! What a refusal offers (`search/LEDGER.md`, H5; the build plan, step
//! 11b): a way on that is a command at a terminal is a request too,
//! beside the reading, for a caller that sends requests and types
//! nothing — and each one runs.

mod common;

use acquisition_search::answer::{Asked, Request};
use acquisition_search::{Corpus, SearchError};
use common::*;
use serde_json::{Value, json};

/// Every request a refusal offers, each read as `--request` reads one.
fn offered(e: &Value) -> Vec<Request> {
    e["offers"]
        .as_array()
        .unwrap_or_else(|| panic!("no offers: {e}"))
        .iter()
        .map(|offer| Asked::read(offer).unwrap().request)
        .collect()
}

/// A continuation refused across a changed basis offers the request from
/// its start: the command the reading is, and a request that answers.
#[test]
fn a_changed_basis_offers_the_request_from_its_start() {
    let mut s = worked();
    let corpus = load(&s, Some("pc"));
    let first = as_json(
        &Asked::read(&json!({ "query": { "text": "" }, "view": { "rows": { "limit": 3 } } }))
            .unwrap()
            .answer(&corpus)
            .unwrap(),
    );
    let next = Asked::read(&first["view"]["rows"]["next"]).unwrap();
    fetch_tab(&mut s, "pc", "Standard", "t3", "Never", vec![], 40);
    let later = load(&s, Some("pc"));
    let e = next.answer(&later).unwrap_err().to_json();
    assert_eq!(e["kind"], "basis_changed", "{e}");
    let offers = offered(&e);
    assert_eq!(offers.len(), 1, "{e}");
    // one maker: the reading is the offered request's own command
    assert_eq!(e["readings"], json!([offers[0].command()]));
    assert_eq!(
        e["offers"][0]["request"],
        json!({
            "scope": { "account": "A", "realm": "pc", "membership": "live" },
            "query": { "text": "" },
            "view": { "rows": { "limit": 3 } },
        })
    );
    let again = as_json(&acquisition_search::answer(&later, &offers[0]).unwrap());
    assert_eq!(again["rows"], first["rows"]);
}

/// `show` of a tab's id offers the search that lists what is in it.
#[test]
fn an_id_that_is_a_tab_offers_the_search_of_it() {
    let s = worked();
    let e = show(&s, "t1", false).unwrap_err().to_json();
    assert_eq!(e["kind"], "not_an_item", "{e}");
    let offers = offered(&e);
    assert_eq!(e["readings"], json!([offers[0].command()]));
    assert_eq!(offers[0].query.text.as_deref(), Some("id:t1"));
    let corpus = offers[0].corpus(&s, &intent(&s)).unwrap();
    let listed = as_json(&acquisition_search::answer(&corpus, &offers[0]).unwrap());
    assert!(listed["total"]["matched"].as_u64().unwrap() > 0, "{listed}");
    assert!(
        listed["rows"]
            .as_array()
            .unwrap()
            .iter()
            .all(|row| row["place"]["id"] == "t1"),
        "{listed}"
    );
}

/// Over a store of several realms a request naming none is refused, and
/// offered again once for each realm held: the flag a terminal adds, and
/// the request itself with that realm in its scope.
#[test]
fn a_realm_needed_offers_the_request_in_each_realm() {
    let mut s = worked();
    list_tabs(&mut s, "xbox", "Standard", json!([tab("x1", "Box")]), 40);
    fetch_tab(
        &mut s,
        "xbox",
        "Standard",
        "x1",
        "Box",
        vec![ring("rx", "Rare", json!({}))],
        41,
    );
    let request: Request = serde_json::from_value(json!({
        "query": { "text": "rarity=rare" },
        "view": { "counts": { "keys": ["tab"] } },
    }))
    .unwrap();
    let e = request.corpus(&s, &intent(&s)).err().unwrap().to_json();
    assert_eq!(e["kind"], "realm_needed", "{e}");
    assert_eq!(e["readings"], json!(["--realm pc", "--realm xbox"]));
    let offers = offered(&e);
    assert_eq!(offers.len(), 2, "{e}");
    for (offer, realm) in offers.iter().zip(["pc", "xbox"]) {
        // the request sent, in that realm, and named for the account it
        // was asked of as every request an answer holds is: with a second
        // account known, one that names none is refused
        let mut sent = serde_json::to_value(&request).unwrap();
        sent["scope"]["realm"] = json!(realm);
        sent["scope"]["account"] = json!("A");
        assert_eq!(serde_json::to_value(offer).unwrap(), sent);
        let corpus = offer.corpus(&s, &intent(&s)).unwrap();
        let a = as_json(&acquisition_search::answer(&corpus, offer).unwrap());
        assert_eq!(a["scope"]["realm"], realm);
    }
    // an account the request named is kept as it was written
    let mut named = request.clone();
    named.scope.account = Some("u-1".to_string());
    let e = named.corpus(&s, &intent(&s)).err().unwrap().to_json();
    assert_eq!(e["offers"][0]["request"]["scope"]["account"], "u-1");
    // the store's own read, asked with no request, has none to offer
    let bare: SearchError = Corpus::load(&s, &intent(&s), None).err().unwrap();
    assert!(bare.to_json().get("offers").is_none());
}

/// A refusal with no way on that is a request offers none: the key is
/// absent, never an empty list.
#[test]
fn a_refusal_with_no_request_to_offer_has_no_offers() {
    let s = worked();
    let e = show(&s, "nope", false).unwrap_err().to_json();
    assert_eq!(e["kind"], "item_not_found");
    assert!(e.get("offers").is_none(), "{e}");
    let corpus = load(&s, Some("pc"));
    let e = ask(&corpus, "ring").unwrap_err().to_json();
    assert_eq!(e["kind"], "bare_word");
    assert!(e.get("offers").is_none(), "{e}");
}
