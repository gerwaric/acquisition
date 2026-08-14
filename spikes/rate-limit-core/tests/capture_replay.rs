use std::collections::{BTreeMap, BTreeSet};

use rate_limit_core::mock::model::{
    CounterModel, PolicyDefinition, RuleDefinition, ServerTime, WindowDefinition,
};

const CANONICAL_CAPTURE: &str = include_str!("../fixtures/capture-20260814-wired.json");
const MAX_FIXTURE_BYTES: usize = 2 * 1024 * 1024;
const MAX_JSON_DEPTH: usize = 16;
const MAX_JSON_ITEMS: usize = 10_000;
const MAX_JSON_STRING_BYTES: usize = 4_096;
const PHASE_CYCLE_MS: u64 = 60_000;
const KNOWN_BUCKETS_MS: [u64; 2] = [5_000, 60_000];

#[derive(Debug, Clone)]
struct ReplayRecord {
    kind: String,
    endpoint: String,
    policy: String,
    sent_ms: Option<u64>,
    received_ms: u64,
    status: u64,
    limits: Vec<LimitTriplet>,
    states: Vec<StateTriplet>,
}

#[derive(Debug, Clone, Copy)]
struct LimitTriplet {
    max_hits: u32,
    period_seconds: u64,
    restriction_seconds: u64,
}

#[derive(Debug, Clone, Copy)]
struct StateTriplet {
    current_hits: u32,
    period_seconds: u64,
    restriction_active_seconds: u64,
}

#[derive(Debug, Clone)]
struct BootSeed {
    policy: String,
    definition: PolicyDefinition,
    residue: usize,
}

#[derive(Debug, Clone, Copy)]
struct SaturationDiagnostic {
    observed_components: usize,
    minimum_agreements: usize,
    maximum_agreements: usize,
    best_phase_ms: u64,
}

#[test]
#[ignore = "§7.4 finding (CR-R1-F1 as amended by SD-R5-F2): the canonical trace violates B3 at 1,052 phases in 20 bands, phi=7,454..25,944 — see VIOLATING_BANDS; adjudication required"]
fn section_7_4_canonical_capture_replay_is_compliant_for_every_server_phase() {
    let records = canonical_records();
    let seeds = boot_seeds(&records);
    let replies: Vec<&ReplayRecord> = records
        .iter()
        .filter(|record| record.kind == "reply")
        .collect();

    assert_eq!(seeds.len(), 4, "the capture has one boot HEAD per policy");
    assert_eq!(replies.len(), 383, "canonical counted replay cardinality");
    assert_eq!(
        seeds
            .iter()
            .map(|seed| seed.policy.as_str())
            .collect::<BTreeSet<_>>()
            .len(),
        seeds.len(),
        "boot residue must be seeded exactly once per policy"
    );
    assert!(
        seeds.iter().all(|seed| seed.residue == 0),
        "the canonical wired capture booted with zero residue; initialization still runs once"
    );

    let mut minimum_agreements = usize::MAX;
    let mut maximum_agreements = 0;
    let mut best_phase_ms = 0;
    let observed_saturation_components = replies
        .iter()
        .map(|record| saturation_components(record))
        .sum::<usize>();

    for phase_ms in 0..PHASE_CYCLE_MS {
        let mut model = CounterModel::new(phase_ms);
        for seed in &seeds {
            model
                .insert_policy(seed.definition.clone())
                .expect("boot HEAD policies are unique");
            model
                .preload(&seed.policy, ServerTime::from_millis(0), seed.residue)
                .expect("boot HEAD residue is within the bounded capture budget");
        }

        let mut saturation_agreements = 0;
        for (index, record) in replies.iter().enumerate() {
            let sent_ms = record.sent_ms.expect("reply records carry sent_ms");
            let at = ServerTime::from_millis(sent_ms);
            let layer1 = model.record_layer1_arrival(at);
            assert!(
                !layer1.tripped,
                "§7.4/B10 violation at phase {phase_ms}, reply {index}, t={sent_ms}ms"
            );

            let judgment = model
                .judge(
                    &record.policy,
                    at,
                    u64::try_from(index + 1).expect("fixture cardinality is bounded"),
                    true,
                )
                .expect("fixture policy was initialized by its boot HEAD");
            assert!(
                !judgment.organic_violation,
                "§7.4/B2 violation at phase {phase_ms}, reply {index}, policy {}, t={sent_ms}ms: {:?}",
                record.policy, judgment.windows
            );

            for ((window, state), limit) in judgment
                .windows
                .iter()
                .zip(&record.states)
                .zip(&record.limits)
            {
                if state.current_hits == limit.max_hits && window.current_hits == state.current_hits
                {
                    saturation_agreements += 1;
                }
            }
        }

        if saturation_agreements > maximum_agreements {
            maximum_agreements = saturation_agreements;
            best_phase_ms = phase_ms;
        }
        minimum_agreements = minimum_agreements.min(saturation_agreements);
    }

    let diagnostic = SaturationDiagnostic {
        observed_components: observed_saturation_components,
        minimum_agreements,
        maximum_agreements,
        best_phase_ms,
    };
    assert_eq!(
        diagnostic.observed_components, 43,
        "canonical capture must retain its recorded saturation points"
    );
    assert!(
        has_recorded_ceiling(&records, 15),
        "N25/N26 diagnostic must include a recorded 15/15 state"
    );
    assert!(
        has_recorded_ceiling(&records, 30),
        "N25/N26 diagnostic must include a recorded 30/30 state"
    );
    assert!(
        diagnostic.maximum_agreements <= diagnostic.observed_components,
        "diagnostic agreement cannot exceed observed saturation components"
    );
    eprintln!(
        "§7.4 saturation diagnostic: {}/{} best agreement at phi={}ms; minimum agreement across phases={}",
        diagnostic.maximum_agreements,
        diagnostic.observed_components,
        diagnostic.best_phase_ms,
        diagnostic.minimum_agreements
    );
}

/// One contiguous run of violating phases and the reply that initiates the
/// overflow at every phase inside it. All twenty bands share the rest of the
/// counterexample shape (`band_overflow`): `stash-request-limit`, sustained
/// window (index 1), 31/30 under the 60 s adversarial bucket — at counted
/// reply 110 the server itself recorded `6:300:0` where the model computes 31.
#[derive(Debug, Clone, Copy)]
struct ViolatingBand {
    start_phase_ms: u64,
    end_phase_ms: u64,
    first_reply_index: usize,
    sent_ms: u64,
}

/// The complete §7.4 violating set (CR-R1-F1 as amended by SD-R5-F2): 1,052
/// phases in 20 disjoint bands across two clusters (initiating replies
/// 110–119 at t≈727–730 s and 125–134 at t≈743–746 s). The originally
/// recorded run aborted at the first violating phase, so only band one was
/// known; `section_7_4_exhaustive_enumeration_matches_the_band_table` proves
/// no violating phase exists outside this table.
const VIOLATING_BANDS: [ViolatingBand; 20] = [
    band(7_454, 7_466, 110, 727_453),
    band(7_705, 7_717, 111, 727_704),
    band(7_955, 7_967, 112, 727_954),
    band(8_205, 8_217, 113, 728_204),
    band(8_454, 8_468, 114, 728_453),
    band(8_704, 8_719, 115, 728_703),
    band(8_955, 8_970, 116, 728_954),
    band(9_205, 9_221, 117, 729_204),
    band(9_455, 9_471, 118, 729_454),
    band(9_705, 9_721, 119, 729_704),
    band(23_602, 23_691, 125, 743_601),
    band(23_853, 23_941, 126, 743_852),
    band(24_103, 24_192, 127, 744_102),
    band(24_352, 24_441, 128, 744_351),
    band(24_603, 24_692, 129, 744_602),
    band(24_853, 24_942, 130, 744_852),
    band(25_103, 25_193, 131, 745_102),
    band(25_354, 25_444, 132, 745_353),
    band(25_604, 25_693, 133, 745_603),
    band(25_854, 25_944, 134, 745_853),
];

const VIOLATING_PHASE_TOTAL: u64 = 1_052;

const fn band(
    start_phase_ms: u64,
    end_phase_ms: u64,
    first_reply_index: usize,
    sent_ms: u64,
) -> ViolatingBand {
    ViolatingBand {
        start_phase_ms,
        end_phase_ms,
        first_reply_index,
        sent_ms,
    }
}

fn band_overflow(band: ViolatingBand) -> CounterOverflow {
    CounterOverflow {
        reply_index: band.first_reply_index,
        policy: "stash-request-limit".to_owned(),
        sent_ms: band.sent_ms,
        window_index: 1,
        current_hits: 31,
        max_hits: 30,
    }
}

/// Pins every band's edges: both edge phases produce the band's initiating
/// overflow, and the phases immediately outside replay cleanly. Interior
/// phases are covered by the exhaustive enumeration below. This test encodes
/// the full SD-R5-F2 band table; the previously recorded "exactly
/// φ=7,454–7,466" claim fails it at band two's start.
#[test]
fn section_7_4_violating_band_edges_are_pinned_for_all_twenty_bands() {
    let records = canonical_records();
    let seeds = boot_seeds(&records);
    let replies: Vec<&ReplayRecord> = records
        .iter()
        .filter(|record| record.kind == "reply")
        .collect();

    let mut total = 0;
    let mut previous_end = None;
    for band in VIOLATING_BANDS {
        assert!(
            band.start_phase_ms <= band.end_phase_ms
                && previous_end.is_none_or(|end| end + 1 < band.start_phase_ms),
            "bands must be ascending and disjoint with clean gaps between them"
        );
        previous_end = Some(band.end_phase_ms);
        total += band.end_phase_ms - band.start_phase_ms + 1;

        for phase_ms in [band.start_phase_ms - 1, band.end_phase_ms + 1] {
            // Between-band neighbors double as the next band's outside edge;
            // re-checking them is cheaper than deduplication is legible.
            if VIOLATING_BANDS
                .iter()
                .any(|other| (other.start_phase_ms..=other.end_phase_ms).contains(&phase_ms))
            {
                continue;
            }
            assert!(
                first_overflow(phase_ms, &seeds, &replies).is_none(),
                "the phase immediately outside a band must replay cleanly: phi={phase_ms}"
            );
        }
        for phase_ms in [band.start_phase_ms, band.end_phase_ms] {
            assert_eq!(
                first_overflow(phase_ms, &seeds, &replies),
                Some(band_overflow(band)),
                "the band-edge counterexample changed at phi={phase_ms}"
            );
        }
    }
    assert_eq!(
        total, VIOLATING_PHASE_TOTAL,
        "the band table must account for every recorded violating phase"
    );
}

/// Exhaustive collect-instead-of-assert enumeration of all 60,000 phases,
/// asserting the violating set is exactly the band table — the run the
/// aborting gate could not perform (SD-R5-F2). It passes today; it exists so
/// any model or fixture change that moves the violating set fails loudly
/// against the recorded finding rather than shifting it silently.
#[test]
#[ignore = "exhaustive 60,000-phase enumeration (~7 s release, minutes in debug); ordinary CI is covered by the band-edge test"]
fn section_7_4_exhaustive_enumeration_matches_the_band_table() {
    let records = canonical_records();
    let seeds = boot_seeds(&records);
    let replies: Vec<&ReplayRecord> = records
        .iter()
        .filter(|record| record.kind == "reply")
        .collect();

    let mut violating_phases = 0_u64;
    for phase_ms in 0..PHASE_CYCLE_MS {
        let expected = VIOLATING_BANDS
            .iter()
            .find(|band| (band.start_phase_ms..=band.end_phase_ms).contains(&phase_ms))
            .map(|band| band_overflow(*band));
        violating_phases += u64::from(expected.is_some());
        assert_eq!(
            first_overflow(phase_ms, &seeds, &replies),
            expected,
            "the §7.4 violating set changed at phi={phase_ms}"
        );
    }
    assert_eq!(violating_phases, VIOLATING_PHASE_TOTAL);
}

#[test]
fn b12_canonical_sent_to_received_median_is_81_ms() {
    let records = canonical_records();
    let mut latencies: Vec<u64> = records
        .iter()
        .filter(|record| record.kind == "reply")
        .map(|record| {
            record
                .received_ms
                .checked_sub(record.sent_ms.expect("reply records carry sent_ms"))
                .expect("received_ms must not precede sent_ms")
        })
        .collect();
    latencies.sort_unstable();

    assert_eq!(latencies.len(), 383, "canonical latency sample count");
    assert_eq!(latencies[latencies.len() / 2], 81);
}

#[test]
fn section_7_4_saturation_diagnostic_has_a_43_of_43_witness_phase() {
    let records = canonical_records();
    let seeds = boot_seeds(&records);
    let replies: Vec<&ReplayRecord> = records
        .iter()
        .filter(|record| record.kind == "reply")
        .collect();
    let mut model = CounterModel::new(0);
    for seed in &seeds {
        model.insert_policy(seed.definition.clone()).unwrap();
        model
            .preload(&seed.policy, ServerTime::from_millis(0), seed.residue)
            .unwrap();
    }

    let mut observed_components = 0;
    let mut agreements = 0;
    for (index, record) in replies.iter().enumerate() {
        let at = ServerTime::from_millis(record.sent_ms.unwrap());
        assert!(!model.record_layer1_arrival(at).tripped);
        let judgment = model
            .judge(&record.policy, at, index as u64 + 1, true)
            .unwrap();
        assert!(!judgment.organic_violation);
        for ((window, state), limit) in judgment
            .windows
            .iter()
            .zip(&record.states)
            .zip(&record.limits)
        {
            if state.current_hits == limit.max_hits {
                observed_components += 1;
                agreements += usize::from(window.current_hits == state.current_hits);
            }
        }
    }

    assert_eq!(observed_components, 43);
    assert_eq!(agreements, observed_components);
}

fn canonical_records() -> Vec<ReplayRecord> {
    assert!(
        CANONICAL_CAPTURE.len() <= MAX_FIXTURE_BYTES,
        "fixture exceeds the test-local parse bound"
    );
    let root = JsonParser::new(CANONICAL_CAPTURE)
        .parse()
        .expect("canonical fixture is bounded valid JSON");
    let root = root.object("root");
    let provenance = root
        .get("provenance")
        .expect("fixture carries provenance")
        .object("provenance");
    assert_eq!(provenance.string("capture_date"), "2026-08-14");
    assert_eq!(provenance.number("capture_schema_version"), 1);
    assert_eq!(provenance.number("sanitizer_version"), 2);

    let raw_records = root
        .get("records")
        .expect("fixture carries records")
        .array("records");
    assert_eq!(raw_records.len(), 387, "canonical fixture cardinality");

    let mut records = Vec::with_capacity(raw_records.len());
    for (index, raw_record) in raw_records.iter().enumerate() {
        let record = raw_record.object("record");
        let kind = record.string("kind").to_owned();
        assert!(kind == "head" || kind == "reply", "record {index} kind");
        let headers = record
            .get("headers")
            .expect("record carries headers")
            .object("headers");
        let policy = headers.string("x-rate-limit-policy").to_owned();
        if let Some(top_level_policy) = record.optional_string("policy") {
            assert_eq!(top_level_policy, policy, "record {index} policy identity");
        }
        let rules = headers.string("x-rate-limit-rules");
        assert_eq!(rules, "Account", "canonical OAuth rule shape");
        let rule_key = format!("x-rate-limit-{}", rules.to_ascii_lowercase());
        let state_key = format!("{rule_key}-state");
        let limits = parse_limit_triplets(headers.string(&rule_key));
        let states = parse_state_triplets(headers.string(&state_key));
        assert_eq!(limits.len(), states.len(), "record {index} window arity");
        assert_eq!(limits.len(), 2, "record {index} RulePair shape");
        for (limit, state) in limits.iter().zip(&states) {
            assert_eq!(
                limit.period_seconds, state.period_seconds,
                "record {index} state/limit period"
            );
        }

        records.push(ReplayRecord {
            kind,
            endpoint: record.string("endpoint").to_owned(),
            policy,
            sent_ms: record.optional_number("sent_ms").map(nonnegative_millis),
            received_ms: nonnegative_millis(record.number("received_ms")),
            status: u64::try_from(record.number("status")).expect("status is nonnegative"),
            limits,
            states,
        });
    }

    let heads = records.iter().filter(|record| record.kind == "head");
    assert_eq!(heads.clone().count(), 4);
    assert!(heads.clone().all(|record| record.sent_ms.is_none()));
    assert!(heads.clone().all(|record| record.status == 204));
    assert_eq!(
        heads
            .map(|record| record.endpoint.as_str())
            .collect::<BTreeSet<_>>()
            .len(),
        4,
        "one initialization HEAD per touched endpoint"
    );
    let replies: Vec<_> = records
        .iter()
        .filter(|record| record.kind == "reply")
        .collect();
    assert!(replies.iter().all(|record| record.status == 200));
    assert!(
        replies
            .windows(2)
            .all(|pair| pair[0].sent_ms <= pair[1].sent_ms)
    );
    records
}

fn boot_seeds(records: &[ReplayRecord]) -> Vec<BootSeed> {
    records
        .iter()
        .filter(|record| record.kind == "head")
        .map(|record| {
            let windows = record
                .limits
                .iter()
                .zip(KNOWN_BUCKETS_MS)
                .map(|(limit, bucket_ms)| {
                    assert_eq!(
                        PHASE_CYCLE_MS % bucket_ms,
                        0,
                        "the exhaustive scalar phase cycle must cover this bucket"
                    );
                    WindowDefinition::new(
                        limit.max_hits,
                        limit.period_seconds * 1_000,
                        limit.restriction_seconds * 1_000,
                        bucket_ms,
                    )
                    .expect("capture limits fit the mock's structural bounds")
                })
                .collect();
            let definition = PolicyDefinition::new(
                record.policy.clone(),
                vec![
                    RuleDefinition::new("Account", windows)
                        .expect("capture rule fits the mock's structural bounds"),
                ],
            )
            .expect("capture policy fits the mock's structural bounds");
            assert!(
                record
                    .states
                    .iter()
                    .all(|state| state.restriction_active_seconds == 0),
                "canonical boot HEAD has no active restriction"
            );
            BootSeed {
                policy: record.policy.clone(),
                definition,
                residue: record
                    .states
                    .iter()
                    .map(|state| state.current_hits as usize)
                    .max()
                    .expect("RulePair state is nonempty"),
            }
        })
        .collect()
}

fn saturation_components(record: &ReplayRecord) -> usize {
    record
        .states
        .iter()
        .zip(&record.limits)
        .filter(|(state, limit)| state.current_hits == limit.max_hits)
        .count()
}

fn has_recorded_ceiling(records: &[ReplayRecord], expected: u32) -> bool {
    records.iter().any(|record| {
        record
            .states
            .iter()
            .zip(&record.limits)
            .any(|(state, limit)| state.current_hits == expected && limit.max_hits == expected)
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CounterOverflow {
    reply_index: usize,
    policy: String,
    sent_ms: u64,
    window_index: usize,
    current_hits: u32,
    max_hits: u32,
}

/// Replays the trace at one phase and returns the *initiating* overflow, if
/// any. The replay stops at the first overflow deliberately: once that
/// request produces the modeled 429, replaying the fixture's later
/// observed-200 traffic would only measure the expected restriction cascade,
/// not the initiating mismatch.
fn first_overflow(
    phase_ms: u64,
    seeds: &[BootSeed],
    replies: &[&ReplayRecord],
) -> Option<CounterOverflow> {
    let mut model = CounterModel::new(phase_ms);
    for seed in seeds {
        model.insert_policy(seed.definition.clone()).unwrap();
        model
            .preload(&seed.policy, ServerTime::from_millis(0), seed.residue)
            .unwrap();
    }

    for (reply_index, record) in replies.iter().enumerate() {
        let sent_ms = record.sent_ms.unwrap();
        let at = ServerTime::from_millis(sent_ms);
        assert!(!model.record_layer1_arrival(at).tripped);
        let judgment = model
            .judge(&record.policy, at, reply_index as u64 + 1, true)
            .unwrap();
        let overflow = judgment
            .windows
            .iter()
            .zip(&record.limits)
            .enumerate()
            .find(|(_, (window, limit))| window.current_hits > limit.max_hits)
            .map(|(window_index, (window, limit))| CounterOverflow {
                reply_index,
                policy: record.policy.clone(),
                sent_ms,
                window_index,
                current_hits: window.current_hits,
                max_hits: limit.max_hits,
            });
        assert_eq!(
            judgment.organic_violation,
            overflow.is_some(),
            "no pre-existing restriction may precede the initiating overflow at phi={phase_ms}, reply={reply_index}"
        );
        if overflow.is_some() {
            return overflow;
        }
    }
    None
}

fn parse_limit_triplets(raw: &str) -> Vec<LimitTriplet> {
    raw.split(',')
        .map(|triplet| {
            let fields = parse_triplet(triplet);
            LimitTriplet {
                max_hits: u32::try_from(fields[0]).expect("max_hits fits u32"),
                period_seconds: fields[1],
                restriction_seconds: fields[2],
            }
        })
        .collect()
}

fn parse_state_triplets(raw: &str) -> Vec<StateTriplet> {
    raw.split(',')
        .map(|triplet| {
            let fields = parse_triplet(triplet);
            StateTriplet {
                current_hits: u32::try_from(fields[0]).expect("current_hits fits u32"),
                period_seconds: fields[1],
                restriction_active_seconds: fields[2],
            }
        })
        .collect()
}

fn parse_triplet(raw: &str) -> [u64; 3] {
    let fields: Vec<u64> = raw
        .split(':')
        .map(|field| {
            field
                .parse()
                .expect("capture triplets contain decimal u64 fields")
        })
        .collect();
    fields
        .try_into()
        .expect("capture limit/state values are triplets")
}

fn nonnegative_millis(value: i64) -> u64 {
    u64::try_from(value).expect("relative client timing must be nonnegative")
}

#[derive(Debug)]
enum Json {
    Object(BTreeMap<String, Json>),
    Array(Vec<Json>),
    String(String),
    Number(i64),
    Bool,
    Null,
}

impl Json {
    fn object(&self, label: &str) -> &BTreeMap<String, Json> {
        match self {
            Self::Object(value) => value,
            _ => panic!("{label} must be a JSON object"),
        }
    }

    fn array(&self, label: &str) -> &[Json] {
        match self {
            Self::Array(value) => value,
            _ => panic!("{label} must be a JSON array"),
        }
    }
}

trait JsonObjectExt {
    fn string(&self, key: &str) -> &str;
    fn optional_string(&self, key: &str) -> Option<&str>;
    fn number(&self, key: &str) -> i64;
    fn optional_number(&self, key: &str) -> Option<i64>;
}

impl JsonObjectExt for BTreeMap<String, Json> {
    fn string(&self, key: &str) -> &str {
        self.optional_string(key)
            .unwrap_or_else(|| panic!("{key} must be a JSON string"))
    }

    fn optional_string(&self, key: &str) -> Option<&str> {
        match self.get(key) {
            Some(Json::String(value)) => Some(value),
            None => None,
            _ => panic!("{key} must be a JSON string when present"),
        }
    }

    fn number(&self, key: &str) -> i64 {
        self.optional_number(key)
            .unwrap_or_else(|| panic!("{key} must be a JSON integer"))
    }

    fn optional_number(&self, key: &str) -> Option<i64> {
        match self.get(key) {
            Some(Json::Number(value)) => Some(*value),
            None => None,
            _ => panic!("{key} must be a JSON integer when present"),
        }
    }
}

struct JsonParser<'a> {
    input: &'a [u8],
    position: usize,
    items: usize,
}

impl<'a> JsonParser<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            input: input.as_bytes(),
            position: 0,
            items: 0,
        }
    }

    fn parse(mut self) -> Result<Json, String> {
        let value = self.value(0)?;
        self.whitespace();
        if self.position != self.input.len() {
            return Err(format!("trailing JSON at byte {}", self.position));
        }
        Ok(value)
    }

    fn value(&mut self, depth: usize) -> Result<Json, String> {
        if depth > MAX_JSON_DEPTH {
            return Err("JSON nesting exceeds test-local bound".to_owned());
        }
        self.bump_item()?;
        self.whitespace();
        match self.peek() {
            Some(b'{') => self.object(depth + 1),
            Some(b'[') => self.array(depth + 1),
            Some(b'"') => self.string().map(Json::String),
            Some(b'-' | b'0'..=b'9') => self.number().map(Json::Number),
            Some(b't') => self.literal(b"true", Json::Bool),
            Some(b'f') => self.literal(b"false", Json::Bool),
            Some(b'n') => self.literal(b"null", Json::Null),
            _ => Err(format!("unexpected JSON token at byte {}", self.position)),
        }
    }

    fn object(&mut self, depth: usize) -> Result<Json, String> {
        self.expect(b'{')?;
        let mut object = BTreeMap::new();
        self.whitespace();
        if self.consume(b'}') {
            return Ok(Json::Object(object));
        }
        loop {
            self.whitespace();
            let key = self.string()?;
            self.whitespace();
            self.expect(b':')?;
            let value = self.value(depth)?;
            if object.insert(key, value).is_some() {
                return Err("duplicate JSON object key".to_owned());
            }
            self.whitespace();
            if self.consume(b'}') {
                break;
            }
            self.expect(b',')?;
        }
        Ok(Json::Object(object))
    }

    fn array(&mut self, depth: usize) -> Result<Json, String> {
        self.expect(b'[')?;
        let mut array = Vec::new();
        self.whitespace();
        if self.consume(b']') {
            return Ok(Json::Array(array));
        }
        loop {
            if array.len() == MAX_JSON_ITEMS {
                return Err("JSON array exceeds test-local item bound".to_owned());
            }
            array.push(self.value(depth)?);
            self.whitespace();
            if self.consume(b']') {
                break;
            }
            self.expect(b',')?;
        }
        Ok(Json::Array(array))
    }

    fn string(&mut self) -> Result<String, String> {
        self.expect(b'"')?;
        let mut value = String::new();
        loop {
            let byte = self
                .next()
                .ok_or_else(|| "unterminated JSON string".to_owned())?;
            match byte {
                b'"' => break,
                b'\\' => {
                    let escaped = self
                        .next()
                        .ok_or_else(|| "unterminated JSON escape".to_owned())?;
                    let character = match escaped {
                        b'"' => '"',
                        b'\\' => '\\',
                        b'/' => '/',
                        b'b' => '\u{0008}',
                        b'f' => '\u{000c}',
                        b'n' => '\n',
                        b'r' => '\r',
                        b't' => '\t',
                        b'u' => {
                            let mut codepoint = 0_u32;
                            for _ in 0..4 {
                                codepoint = codepoint * 16
                                    + self
                                        .next()
                                        .and_then(|digit| (digit as char).to_digit(16))
                                        .ok_or_else(|| "invalid JSON unicode escape".to_owned())?;
                            }
                            char::from_u32(codepoint)
                                .ok_or_else(|| "invalid JSON unicode scalar".to_owned())?
                        }
                        _ => return Err("invalid JSON escape".to_owned()),
                    };
                    value.push(character);
                }
                0x00..=0x1f => return Err("control byte in JSON string".to_owned()),
                ascii if ascii.is_ascii() => value.push(char::from(ascii)),
                _ => return Err("fixture JSON strings must be ASCII".to_owned()),
            }
            if value.len() > MAX_JSON_STRING_BYTES {
                return Err("JSON string exceeds test-local byte bound".to_owned());
            }
        }
        Ok(value)
    }

    fn number(&mut self) -> Result<i64, String> {
        let start = self.position;
        self.consume(b'-');
        let digits = self.position;
        while matches!(self.peek(), Some(b'0'..=b'9')) {
            self.position += 1;
        }
        if self.position == digits {
            return Err("JSON integer has no digits".to_owned());
        }
        if matches!(self.peek(), Some(b'.' | b'e' | b'E')) {
            return Err("fixture JSON numbers must be integers".to_owned());
        }
        std::str::from_utf8(&self.input[start..self.position])
            .map_err(|_| "JSON integer is not UTF-8".to_owned())?
            .parse()
            .map_err(|_| "JSON integer exceeds i64".to_owned())
    }

    fn literal(&mut self, literal: &[u8], value: Json) -> Result<Json, String> {
        if self.input.get(self.position..self.position + literal.len()) == Some(literal) {
            self.position += literal.len();
            Ok(value)
        } else {
            Err(format!("invalid JSON literal at byte {}", self.position))
        }
    }

    fn bump_item(&mut self) -> Result<(), String> {
        if self.items == MAX_JSON_ITEMS {
            return Err("JSON value count exceeds test-local bound".to_owned());
        }
        self.items += 1;
        Ok(())
    }

    fn whitespace(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\n' | b'\r' | b'\t')) {
            self.position += 1;
        }
    }

    fn expect(&mut self, expected: u8) -> Result<(), String> {
        if self.consume(expected) {
            Ok(())
        } else {
            Err(format!(
                "expected {:?} at byte {}",
                char::from(expected),
                self.position
            ))
        }
    }

    fn consume(&mut self, expected: u8) -> bool {
        if self.peek() == Some(expected) {
            self.position += 1;
            true
        } else {
            false
        }
    }

    fn peek(&self) -> Option<u8> {
        self.input.get(self.position).copied()
    }

    fn next(&mut self) -> Option<u8> {
        let byte = self.peek()?;
        self.position += 1;
        Some(byte)
    }
}
