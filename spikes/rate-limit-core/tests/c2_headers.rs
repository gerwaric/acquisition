use std::time::Duration;

use http::{HeaderMap, HeaderValue};
use proptest::prelude::*;
use rate_limit_core::header::{
    MAX_DIAGNOSTIC_BYTES, MAX_HEADER_VALUE_BYTES, MAX_POLICY_NAME_BYTES, MAX_RULE_NAME_BYTES,
    PolicyParseError, parse_policy,
};

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

// D8 Full grammar (external review finding 3): an empty or whitespace policy
// name, a zero-hit limit, or a zero period is not a Full policy — post-
// seeding these would be wire inputs that permanently block a policy.
#[test]
fn d8_non_full_policies_are_rejected() {
    for name in ["", "   "] {
        let mut headers = headers("15:10:60, 30:300:300", "1:10:0, 1:300:0");
        headers.insert(
            "x-rate-limit-policy",
            HeaderValue::from_str(name).expect("test name is valid HTTP"),
        );
        assert!(
            matches!(
                parse_policy(&headers),
                Err(PolicyParseError::EmptyPolicyName)
            ),
            "expected EmptyPolicyName for {name:?}"
        );
    }

    // Zero limit hits, zero limit period, zero state period.
    for (limit, state) in [
        ("0:10:60, 30:300:300", "1:10:0, 1:300:0"),
        ("15:0:60, 30:300:300", "1:0:0, 1:300:0"),
        ("15:10:60, 30:300:300", "1:0:0, 1:300:0"),
    ] {
        assert!(
            matches!(
                parse_policy(&headers(limit, state)),
                Err(PolicyParseError::OutOfRangeTriplet { .. })
            ),
            "expected OutOfRangeTriplet for limit={limit:?} state={state:?}"
        );
    }
}

// Absolute wire ceilings (external review finding 4): once seeding makes
// configuration wire-derived, these parse bounds are the only non-circular
// limit on downstream allocation. Boundaries pinned at n and n+1.
#[test]
fn wire_ceilings_are_enforced_at_the_boundary() {
    // max_hits: 10_000 parses, 10_001 refuses.
    assert!(parse_policy(&headers("10000:10:60, 10000:300:300", "1:10:0, 1:300:0")).is_ok());
    assert!(matches!(
        parse_policy(&headers("10001:10:60, 10000:300:300", "1:10:0, 1:300:0")),
        Err(PolicyParseError::OutOfRangeTriplet { .. })
    ));

    // period: 21600 parses (as sustained), 21601 refuses.
    assert!(parse_policy(&headers("15:10:60, 30:21600:300", "1:10:0, 1:21600:0")).is_ok());
    assert!(matches!(
        parse_policy(&headers("15:10:60, 30:21601:300", "1:10:0, 1:21601:0")),
        Err(PolicyParseError::OutOfRangeTriplet { .. })
    ));

    // Rule count: 8 named rules stop at the missing-header error (the names
    // resolve no headers), the 9th refuses before any lookup.
    let mut eight = headers("15:10:60, 30:300:300", "1:10:0, 1:300:0");
    eight.insert(
        "x-rate-limit-rules",
        HeaderValue::from_static("r1,r2,r3,r4,r5,r6,r7,r8"),
    );
    assert!(matches!(
        parse_policy(&eight),
        Err(PolicyParseError::MissingHeader { .. })
    ));
    let mut nine = headers("15:10:60, 30:300:300", "1:10:0, 1:300:0");
    nine.insert(
        "x-rate-limit-rules",
        HeaderValue::from_static("Account,r2,r3,r4,r5,r6,r7,r8,r9"),
    );
    assert!(matches!(
        parse_policy(&nine),
        Err(PolicyParseError::TooManyRules { limit: 8 })
    ));

    // Triplet count: nine triplets in one header refuse without parsing
    // past the bound.
    let many = "1:1:0, 2:2:0, 3:3:0, 4:4:0, 5:5:0, 6:6:0, 7:7:0, 8:8:0, 9:9:0";
    assert!(matches!(
        parse_policy(&headers(many, "1:10:0, 1:300:0")),
        Err(PolicyParseError::TooManyTriplets { .. })
    ));
}

// Byte ceilings (follow-up review 2026-08-10): wire-sized names and
// diagnostics were the remaining unbounded copies after the count/numeric
// ceilings. Boundaries pinned at n and n+1.
#[test]
fn wire_byte_ceilings_are_enforced_at_the_boundary() {
    // Policy name: at the ceiling parses, one byte over refuses.
    for (len, expect_ok) in [
        (MAX_POLICY_NAME_BYTES, true),
        (MAX_POLICY_NAME_BYTES + 1, false),
    ] {
        let mut headers = headers("15:10:60, 30:300:300", "1:10:0, 1:300:0");
        headers.insert(
            "x-rate-limit-policy",
            HeaderValue::from_str(&"p".repeat(len)).expect("test name is valid HTTP"),
        );
        let result = parse_policy(&headers);
        if expect_ok {
            assert!(result.is_ok(), "a {len}-byte policy name should parse");
        } else {
            assert!(matches!(
                result,
                Err(PolicyParseError::PolicyNameTooLong { limit }) if limit == MAX_POLICY_NAME_BYTES
            ));
        }
    }

    // Rule name: at the ceiling proceeds to header lookup (MissingHeader —
    // the name resolves no header), one byte over refuses before any lookup.
    let mut at_limit = headers("15:10:60, 30:300:300", "1:10:0, 1:300:0");
    at_limit.insert(
        "x-rate-limit-rules",
        HeaderValue::from_str(&"r".repeat(MAX_RULE_NAME_BYTES)).expect("valid HTTP"),
    );
    assert!(matches!(
        parse_policy(&at_limit),
        Err(PolicyParseError::MissingHeader { .. })
    ));
    let mut over_limit = headers("15:10:60, 30:300:300", "1:10:0, 1:300:0");
    over_limit.insert(
        "x-rate-limit-rules",
        HeaderValue::from_str(&"r".repeat(MAX_RULE_NAME_BYTES + 1)).expect("valid HTTP"),
    );
    assert!(matches!(
        parse_policy(&over_limit),
        Err(PolicyParseError::RuleNameTooLong { limit }) if limit == MAX_RULE_NAME_BYTES
    ));
}

// Whole header values are byte-gated before any conversion, trim, split,
// or digit scan (follow-up review P2) — wire length must not size parsing
// work either. Boundary pinned at n and n+1 through a value that parses
// fully at the ceiling.
#[test]
fn header_value_byte_gate_is_enforced_before_parsing() {
    let at_gate = format!("Account,{}", " ".repeat(MAX_HEADER_VALUE_BYTES - 8));
    let over_gate = format!("Account,{}", " ".repeat(MAX_HEADER_VALUE_BYTES - 7));

    // At the gate the value is scanned and fails on its own merits (the
    // all-whitespace second slot); one byte over refuses before any scan.
    let mut at_limit = headers("15:10:60, 30:300:300", "1:10:0, 1:300:0");
    at_limit.insert(
        "x-rate-limit-rules",
        HeaderValue::from_str(&at_gate).expect("valid HTTP"),
    );
    assert!(matches!(
        parse_policy(&at_limit),
        Err(PolicyParseError::InvalidRuleName { .. })
    ));

    let mut over_limit = headers("15:10:60, 30:300:300", "1:10:0, 1:300:0");
    over_limit.insert(
        "x-rate-limit-rules",
        HeaderValue::from_str(&over_gate).expect("valid HTTP"),
    );
    assert!(matches!(
        parse_policy(&over_limit),
        Err(PolicyParseError::HeaderValueTooLong { name, limit })
            if name == "x-rate-limit-rules" && limit == MAX_HEADER_VALUE_BYTES
    ));
}

// Diagnostics quote at most MAX_DIAGNOSTIC_BYTES of raw wire text — a
// malformed field must not size the error allocation. Fields stay under
// the whole-value byte gate so the truncation layer itself is exercised.
#[test]
fn diagnostic_raw_fields_are_truncated() {
    let huge = "x".repeat(900);

    let limit = format!("{huge}, 30:300:300");
    match parse_policy(&headers(&limit, "1:10:0, 1:300:0")) {
        Err(PolicyParseError::MalformedTriplet { raw, .. }) => {
            assert!(raw.len() <= MAX_DIAGNOSTIC_BYTES);
        }
        other => panic!("expected MalformedTriplet, got {other:?}"),
    }

    // An all-whitespace rule slot is empty after trim, so it reaches
    // InvalidRuleName (not the length ceiling) with an unbounded raw.
    let mut rules = headers("15:10:60, 30:300:300", "1:10:0, 1:300:0");
    rules.insert(
        "x-rate-limit-rules",
        HeaderValue::from_str(&format!("Account,{}", " ".repeat(900))).expect("valid HTTP"),
    );
    match parse_policy(&rules) {
        Err(PolicyParseError::InvalidRuleName { raw }) => {
            assert!(raw.len() <= MAX_DIAGNOSTIC_BYTES);
        }
        other => panic!("expected InvalidRuleName, got {other:?}"),
    }
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
    // C2: arbitrary representable two-window policies round-trip into
    // RulePair. Domain matches the D8 grammar and wire ceilings — the
    // rejected complements have their own pinned tests above.
    #[test]
    fn valid_pairs_round_trip(
        burst_hits in 1_u32..=10_000,
        sustained_hits in 1_u32..=10_000,
        burst_period in 1_u32..1_800,
        period_delta in 1_u32..1_800,
        burst_restriction in any::<u32>(),
        sustained_restriction in any::<u32>(),
    ) {
        let sustained_period = burst_period + period_delta;
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
