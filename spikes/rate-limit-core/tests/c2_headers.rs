use std::time::Duration;

use http::{HeaderMap, HeaderValue};
use proptest::prelude::*;
use rate_limit_core::header::{PolicyParseError, parse_policy};

fn headers(limit: &str, state: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
        "x-rate-limit-policy",
        HeaderValue::from_static("stash-request-limit"),
    );
    headers.insert("x-rate-limit-rules", HeaderValue::from_static("Account"));
    headers.insert(
        "x-rate-limit-account",
        HeaderValue::from_str(limit).expect("test limit header is valid HTTP"),
    );
    headers.insert(
        "x-rate-limit-account-state",
        HeaderValue::from_str(state).expect("test state header is valid HTTP"),
    );
    headers
}

// C2: a two-triplet rule with increasing periods is a RulePair.
#[test]
fn parses_two_triplets_as_burst_and_sustained() {
    let headers = headers("15:10:60, 30:300:300", "1:10:0, 1:300:0");
    let snapshot = parse_policy(&headers).expect("the documented pair shape should parse");
    let pair = &snapshot.rules[0].pair;

    assert_eq!(pair.burst().period, Duration::from_secs(10));
    assert_eq!(pair.sustained().period, Duration::from_secs(300));
}

// C2: one- and three-triplet rules are out of the RulePair model.
#[test]
fn rejects_non_pair_shapes() {
    for (limit, expected_count) in [("15:10:60", 1), ("15:10:60, 30:300:300, 60:600:600", 3)] {
        let error = parse_policy(&headers(limit, "1:10:0, 1:300:0")).unwrap_err();
        assert!(matches!(
            error,
            PolicyParseError::UnexpectedPolicyShape {
                triplet_count,
                ..
            } if triplet_count == expected_count
        ));
    }
}

// C2: shape includes positional ordering, not just cardinality.
#[test]
fn rejects_non_increasing_periods() {
    for limit in ["15:300:60, 30:300:300", "15:300:60, 30:10:300"] {
        assert!(matches!(
            parse_policy(&headers(limit, "1:300:0, 1:300:0")),
            Err(PolicyParseError::NonIncreasingPeriods { .. })
        ));
    }
}

// C2 / N20: absence is an error variant, never an empty list — for every
// required header, not just the rules list.
#[test]
fn missing_headers_are_typed() {
    for missing in [
        "x-rate-limit-policy",
        "x-rate-limit-rules",
        "x-rate-limit-account",
        "x-rate-limit-account-state",
    ] {
        let mut headers = headers("15:10:60, 30:300:300", "1:10:0, 1:300:0");
        headers.remove(missing);

        assert!(
            matches!(
                parse_policy(&headers),
                Err(PolicyParseError::MissingHeader { name }) if name == missing
            ),
            "expected MissingHeader for {missing}"
        );
    }
}

// C2: a rule name that cannot form a header name — or an empty slot in the
// rules list — is its own typed variant.
#[test]
fn invalid_rule_names_are_typed() {
    for rules in ["Account, ", "bad name", ""] {
        let mut headers = headers("15:10:60, 30:300:300", "1:10:0, 1:300:0");
        headers.insert(
            "x-rate-limit-rules",
            HeaderValue::from_str(rules).expect("test rules value is valid HTTP"),
        );

        assert!(
            matches!(
                parse_policy(&headers),
                Err(PolicyParseError::InvalidRuleName { .. })
            ),
            "expected InvalidRuleName for {rules:?}"
        );
    }
}

// C2: a header present but not decodable as visible ASCII is typed absence
// of a *usable* value, distinct from a missing header.
#[test]
fn undecodable_header_value_is_typed() {
    let mut headers = headers("15:10:60, 30:300:300", "1:10:0, 1:300:0");
    headers.insert(
        "x-rate-limit-policy",
        HeaderValue::from_bytes(&[0x80]).expect("obs-text bytes are legal header values"),
    );

    assert!(matches!(
        parse_policy(&headers),
        Err(PolicyParseError::InvalidHeaderValue { name })
            if name == "x-rate-limit-policy"
    ));
}

// C2: limit and state headers must describe the same windows.
#[test]
fn state_periods_mismatch_is_typed() {
    assert!(matches!(
        parse_policy(&headers("15:10:60, 30:300:300", "1:20:0, 1:300:0")),
        Err(PolicyParseError::StatePeriodsMismatch { .. })
    ));
}

// O6 case/order domain, pinned as current behavior:
// - header-name lookup is case-insensitive (the http crate normalizes), so a
//   rule spelled ACCOUNT finds x-rate-limit-account, and the snapshot keeps
//   the wire spelling;
// - a duplicated rule name parses into two identical rules — harmless today
//   because reconciliation takes the max deficit, never the sum;
// - a duplicated header (appended, not replaced) resolves to its first value.
#[test]
fn header_case_and_duplicates_resolve_first_value_and_preserve_spelling() {
    let mut mixed_case = headers("15:10:60, 30:300:300", "1:10:0, 1:300:0");
    mixed_case.insert("x-rate-limit-rules", HeaderValue::from_static("ACCOUNT"));
    let snapshot = parse_policy(&mixed_case).expect("case-insensitive lookup succeeds");
    assert_eq!(snapshot.rules[0].name, "ACCOUNT");

    let mut duplicated_rule = headers("15:10:60, 30:300:300", "1:10:0, 1:300:0");
    duplicated_rule.insert(
        "x-rate-limit-rules",
        HeaderValue::from_static("Account, Account"),
    );
    let snapshot = parse_policy(&duplicated_rule).expect("duplicate rule names parse");
    assert_eq!(snapshot.rules.len(), 2);
    assert_eq!(snapshot.rules[0], snapshot.rules[1]);

    let mut duplicated_header = headers("15:10:60, 30:300:300", "1:10:0, 1:300:0");
    duplicated_header.append(
        "x-rate-limit-account",
        HeaderValue::from_static("99:5:60, 99:600:300"),
    );
    let snapshot = parse_policy(&duplicated_header).expect("first header value wins");
    assert_eq!(
        snapshot.rules[0].pair.burst().period,
        Duration::from_secs(10)
    );
}

// C2: malformed triplets fail without indexing or panicking. Numeric fields
// are strict ASCII digits — the `+`/whitespace leniency of str::parse is out.
#[test]
fn malformed_triplet_is_typed() {
    for limit in [
        "15:ten:60, 30:300:300",
        "+15:10:60, 30:300:300",
        "15 :10:60, 30:300:300",
        "15:10:60:70, 30:300:300",
    ] {
        assert!(
            matches!(
                parse_policy(&headers(limit, "1:10:0, 1:300:0")),
                Err(PolicyParseError::MalformedTriplet { .. })
            ),
            "expected MalformedTriplet for {limit:?}"
        );
    }
}

proptest! {
    // C2: arbitrary representable two-window policies round-trip into RulePair.
    #[test]
    fn valid_pairs_round_trip(
        burst_hits in any::<u32>(),
        sustained_hits in any::<u32>(),
        burst_period in 0_u32..u32::MAX,
        period_delta in 1_u32..=u32::MAX,
        burst_restriction in any::<u32>(),
        sustained_restriction in any::<u32>(),
    ) {
        let sustained_period = burst_period.saturating_add(period_delta);
        prop_assume!(sustained_period > burst_period);
        let limit = format!(
            "{burst_hits}:{burst_period}:{burst_restriction}, \
             {sustained_hits}:{sustained_period}:{sustained_restriction}"
        );
        let state = format!("0:{burst_period}:0, 0:{sustained_period}:0");

        let snapshot = parse_policy(&headers(&limit, &state)).unwrap();
        let pair = &snapshot.rules[0].pair;

        prop_assert_eq!(pair.burst().max_hits, burst_hits);
        prop_assert_eq!(pair.burst().period, Duration::from_secs(burst_period.into()));
        prop_assert_eq!(pair.sustained().max_hits, sustained_hits);
        prop_assert_eq!(
            pair.sustained().period,
            Duration::from_secs(sustained_period.into())
        );
    }

    // C2: arbitrary non-triplet text is rejected and never panics.
    #[test]
    fn malformed_text_never_parses(raw in "[A-Za-z]{0,32}") {
        let limit = format!("{raw}, 30:300:300");
        let is_malformed = matches!(
            parse_policy(&headers(&limit, "0:10:0, 0:300:0")),
            Err(PolicyParseError::MalformedTriplet { .. })
        );
        prop_assert!(is_malformed);
    }
}
