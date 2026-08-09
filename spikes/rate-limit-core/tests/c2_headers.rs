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
    let pair = &snapshot.rules()[0].pair;

    assert_eq!(pair.burst().period(), Duration::from_secs(10));
    assert_eq!(pair.sustained().period(), Duration::from_secs(300));
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

// C2 / N20: absence is an error variant, never an empty list.
#[test]
fn missing_rules_header_is_typed() {
    let mut headers = headers("15:10:60, 30:300:300", "1:10:0, 1:300:0");
    headers.remove("x-rate-limit-rules");

    assert!(matches!(
        parse_policy(&headers),
        Err(PolicyParseError::MissingHeader { name })
            if name == "x-rate-limit-rules"
    ));
}

// C2: malformed triplets fail without indexing or panicking.
#[test]
fn malformed_triplet_is_typed() {
    assert!(matches!(
        parse_policy(&headers("15:ten:60, 30:300:300", "1:10:0, 1:300:0")),
        Err(PolicyParseError::MalformedTriplet { .. })
    ));
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
        let pair = &snapshot.rules()[0].pair;

        prop_assert_eq!(pair.burst().max_hits(), burst_hits);
        prop_assert_eq!(pair.burst().period(), Duration::from_secs(burst_period.into()));
        prop_assert_eq!(pair.sustained().max_hits(), sustained_hits);
        prop_assert_eq!(
            pair.sustained().period(),
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
