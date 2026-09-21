//! The fifth outside audit (2026-09-21), of step 4b at `ef381909`: four
//! defects of the first surface that the generated properties had not
//! met, each reproduced here before it was fixed. C92 (a sum), C100 and
//! invariants 4 and 5 of the surface (a count has a route; every block is
//! bounded), C97 (`--describe` is the help).

mod common;

use acquisition_search::describe;
use common::generated::{fixture, request, run};
use serde_json::{Value, json};

fn lines(texts: &[&str]) -> Value {
    json!({ "explicitMods": texts })
}

/// A sum is its occurrences' sum, in whatever order the body lists them:
/// 0.1 + 0.2 + 0.3 is 0.6, never 0.6000000000000001, so `=0.6` matches
/// both items and both sort by one value. The mean of a ranged pair is a
/// sum too.
#[test]
fn c92_a_sum_of_decimals_is_exact_and_no_order_of_occurrences_changes_it() {
    let leech = |values: [&str; 3]| {
        let texts: Vec<String> = values
            .iter()
            .map(|v| format!("{v}% of Damage Leeched as Life"))
            .collect();
        json!({ "explicitMods": texts })
    };
    let (corpus, _) = fixture(vec![
        leech(["0.1", "0.2", "0.3"]),
        leech(["0.3", "0.2", "0.1"]),
        lines(&["Adds 0.1 to 0.2 Cold Damage"]),
    ]);
    let ids = |query: &str, sort: Option<&str>| -> Vec<(String, Value)> {
        let answer = run(&corpus, &request(query, sort, false, 10)).unwrap();
        let mut rows: Vec<(String, Value)> = answer["rows"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| {
                (
                    r["id"].as_str().unwrap().to_string(),
                    r["sort"]["value"].clone(),
                )
            })
            .collect();
        rows.sort_by(|a, b| a.0.cmp(&b.0));
        rows
    };
    let sum = "sum(\"#% of Damage Leeched as Life\")";
    let matched: Vec<String> = ids(&format!("{sum}=0.6"), None)
        .into_iter()
        .map(|r| r.0)
        .collect();
    assert_eq!(matched, ["i0", "i1"]);
    let sorted = ids("\"#% of Damage Leeched as Life\"", Some(sum));
    assert_eq!(sorted[0].1, json!(0.6));
    assert_eq!(sorted[1].1, json!(0.6));
    // reaching a bound only together is a sum as well
    let together = run(
        &corpus,
        &request(
            "line(\"#% of Damage Leeched as Life\" arg1>=0.6)",
            None,
            false,
            10,
        ),
    )
    .unwrap();
    assert_eq!(together["terms"][0]["together"]["count"], 2);
    // (0.1 + 0.2) / 2 is 0.15
    let avg: Vec<String> = ids("line(\"Adds # to # Cold Damage\" avg=0.15)", None)
        .into_iter()
        .map(|r| r.0)
        .collect();
    assert_eq!(avg, ["i2"]);
}

/// A suggestion's count is the count of the term it offers (invariant 4):
/// `=` is any-case, so two spellings of one line are one suggestion, and
/// asking it returns as many items as it said.
#[test]
fn c100_a_suggestion_counts_what_its_term_returns() {
    let (corpus, _) = fixture(vec![
        lines(&["Gain 5 Life per Enemy Killed"]),
        lines(&["Gain 7 Life per enemy killed"]),
        lines(&[
            "Gain 9 Life per Enemy Killed",
            "Gain 1 Life per enemy killed",
        ]),
    ]);
    let answer = run(
        &corpus,
        &request("line(\"Gain # Life per Enemy Kiledd\")", None, false, 10),
    )
    .unwrap();
    let suggestions = answer["zero"]["resolved_to_nothing"][0]["suggestions"]
        .as_array()
        .unwrap();
    assert_eq!(
        suggestions.len(),
        1,
        "one line, however GGG spelled it: {suggestions:?}"
    );
    for s in suggestions {
        let asked = run(
            &corpus,
            &request(s["term"].as_str().unwrap(), None, false, 10),
        )
        .unwrap();
        assert_eq!(asked["total"]["matched"], s["items"], "{s}");
    }
    assert_eq!(suggestions[0]["items"], 3);
    // a field's values too
    let (corpus, _) = fixture(vec![
        json!({ "name": "Doom Knot" }),
        json!({ "name": "DOOM KNOT" }),
    ]);
    let answer = run(&corpus, &request("name=\"Doom Knott\"", None, false, 10)).unwrap();
    let suggestions = answer["zero"]["resolved_to_nothing"][0]["suggestions"]
        .as_array()
        .unwrap();
    assert_eq!(suggestions.len(), 1, "{suggestions:?}");
    assert_eq!(suggestions[0]["items"], 2);
}

/// A selector that resolved to nothing is said to, whether or not it has
/// words a suggestion could be scored against: a pattern has none, and is
/// listed with no suggestion.
#[test]
fn c100_a_pattern_that_resolved_to_nothing_is_in_the_zero_block() {
    let (corpus, _) = fixture(vec![
        lines(&["+5 to maximum Life"]),
        json!({ "name": "Doom Knot" }),
    ]);
    for (query, of) in [
        ("line(template:nothing)", "template"),
        ("line(template~\"nothing\")", "template"),
        ("sum(line(template~\"nothing\").arg1)>0", "template"),
        ("name~\"nothing\"", "name"),
        ("name:nothing", "name"),
        // found by the cross-check the audit asked for, on its first day
        ("rarity:ma", "rarity"),
    ] {
        let answer = run(&corpus, &request(query, None, false, 10)).unwrap();
        let nothing = answer["zero"]["resolved_to_nothing"].as_array().unwrap();
        assert_eq!(nothing.len(), 1, "`{query}`: {nothing:?}");
        assert_eq!(nothing[0]["of"], of, "`{query}`");
    }
    // a pattern that found something is not
    let answer = run(
        &corpus,
        &request("line(template~\"life\" arg1>=90)", None, false, 10),
    )
    .unwrap();
    assert_eq!(answer["zero"]["resolved_to_nothing"], json!([]));
}

/// What a row shows of one term is bounded, and says what it left out
/// (invariant 5); `acq show <id>` is the route to the whole.
#[test]
fn c100_a_rows_evidence_is_bounded_and_says_what_it_left_out() {
    let forty: Vec<String> = (1..=40).map(|n| format!("+{n} to maximum Life")).collect();
    let (corpus, _) = fixture(vec![json!({ "explicitMods": forty })]);
    for (query, touched) in [
        ("line(template:life)", 40),
        ("sum(\"# to maximum Life\")>0", 41),
        ("text:life", 40),
    ] {
        let answer = run(&corpus, &request(query, None, false, 10)).unwrap();
        let matched = &answer["rows"][0]["matched"][0];
        let shows = matched["shows"].as_array().unwrap().len();
        assert!(shows <= 7, "`{query}` shows {shows}");
        assert_eq!(
            shows as u64 + matched["left_out"].as_u64().unwrap(),
            touched,
            "`{query}`: {matched}"
        );
    }
    // nothing left out is not said
    let (corpus, _) = fixture(vec![lines(&["+5 to maximum Life"])]);
    let answer = run(&corpus, &request("line(template:life)", None, false, 10)).unwrap();
    assert!(answer["rows"][0]["matched"][0].get("left_out").is_none());
}

/// `--describe` is the help C97 names: it says how terms compose and what
/// has a value, and a word it prints can be asked for alone.
#[test]
fn c97_describe_says_how_terms_compose_and_knows_the_words_it_prints() {
    let whole = serde_json::to_value(describe(&[]).unwrap()).unwrap();
    let names = |block: &str| -> Vec<String> {
        whole[block]
            .as_array()
            .unwrap_or_else(|| panic!("no `{block}` block"))
            .iter()
            .map(|n| n["name"].as_str().unwrap().to_string())
            .collect()
    };
    let composition = names("composition").join(" ");
    for word in ["and", "or", "not", "holds", "undecided", "true()"] {
        assert!(
            composition.contains(word),
            "composition lacks `{word}`: {composition}"
        );
    }
    let values = names("values").join(" ");
    for word in ["sum", "line(P).<slot>"] {
        assert!(values.contains(word), "values lacks `{word}`: {values}");
    }
    // every example the description prints is a query this build binds
    for block in ["fields", "line", "composition", "values"] {
        for entry in whole[block].as_array().unwrap() {
            for example in entry["examples"].as_array().into_iter().flatten() {
                acquisition_search::parse_query(example.as_str().unwrap())
                    .unwrap_or_else(|e| panic!("{example}: {e}"));
            }
        }
    }
    for (word, block) in [
        ("arg1", "slots"),
        ("arg7", "slots"),
        ("avg", "slots"),
        ("=", "operators"),
        (">=", "operators"),
        ("holds", "composition"),
        ("sum", "values"),
    ] {
        let one = serde_json::to_value(describe(&[word.to_string()]).unwrap())
            .unwrap_or_else(|_| panic!("`{word}`"));
        assert_eq!(one[block].as_array().unwrap().len(), 1, "`{word}`: {one}");
    }
}

/// The audit's follow-up, 1: a number is read once, where the body is
/// read, and one the search does not read — more digits than any game
/// displays — is an unread slot of its line (C93), never a sum that is
/// exact in one order of occurrences and rounded in another. So both
/// orders answer alike, and say why.
#[test]
fn c93_a_number_beyond_reach_is_unread_where_it_is_read_and_no_order_matters() {
    let huge = "10000000000000000000000000000";
    let life = |n: &str| format!("{n} to maximum Life");
    let (corpus, _) = fixture(vec![
        json!({ "explicitMods": [life(&format!("+{huge}")), life(&format!("+{huge}")),
            life(&format!("-{huge}")), life(&format!("-{huge}")), life("+0.0000000001")] }),
        json!({ "explicitMods": [life(&format!("+{huge}")), life(&format!("-{huge}")),
            life(&format!("+{huge}")), life(&format!("-{huge}")), life("+0.0000000001")] }),
        lines(&[
            "+1234567890.1234 to maximum Life",
            "-1234567890.1234 to maximum Life",
            "+0.0001 to maximum Life",
        ]),
    ]);
    let sum = "sum(\"# to maximum Life\")";
    let answer = run(&corpus, &request(&format!("{sum}>0"), Some(sum), false, 10)).unwrap();
    assert_eq!(
        answer["terms"][0]["undecided"]["count"], 2,
        "{}",
        answer["terms"][0]
    );
    // the largest number read whole: ten digits and four decimals, exactly
    assert_eq!(answer["rows"].as_array().unwrap().len(), 1);
    assert_eq!(answer["rows"][0]["id"], "i2");
    assert_eq!(answer["rows"][0]["sort"]["value"], json!(0.0001));
    let why = &answer["total"]["undecided_items"][0]["why"][0];
    assert_eq!(why["unread"], "the numbers of explicit lines");
    assert!(why["problem"].as_str().unwrap().contains("digits"), "{why}");
    // its text is a witness all the same
    let phrase = run(&corpus, &request("\"maximum Life\"", None, false, 10)).unwrap();
    assert_eq!(phrase["total"]["matched"], 3);
}

/// The follow-up, 2: what is shown of why an item is undecided is bounded
/// by the item's unread parts, in the item's order — never by the order
/// the terms were written in, which a rewrite that changes no meaning may
/// change (invariant 7).
#[test]
fn c100_the_reasons_shown_are_the_items_first_six_however_the_terms_are_ordered() {
    let flags = [
        "corrupted",
        "identified",
        "split",
        "duplicated",
        "replica",
        "fractured",
        "mutated",
    ];
    let mut body = json!({});
    for flag in flags {
        body[flag] = json!("unread");
    }
    let (corpus, _) = fixture(vec![body]);
    let shown = |order: &[&str]| -> (Vec<String>, Value, Vec<String>, Value) {
        let terms: Vec<String> = order.iter().map(|f| format!("is:{f}")).collect();
        let probe = run(
            &corpus,
            &request(&format!("undecided({})", terms.join(" ")), None, false, 10),
        )
        .unwrap();
        let matched = &probe["rows"][0]["matched"][0];
        let mut unread: Vec<String> = matched["shows"]
            .as_array()
            .unwrap()
            .iter()
            .map(|e| e["undecided"]["unread"].as_str().unwrap().to_string())
            .collect();
        unread.sort();
        let root = run(&corpus, &request(&terms.join(" "), None, false, 10)).unwrap();
        let item = &root["total"]["undecided_items"][0];
        let mut why: Vec<String> = item["why"]
            .as_array()
            .unwrap()
            .iter()
            .map(|w| w["unread"].as_str().unwrap().to_string())
            .collect();
        why.sort();
        (
            unread,
            matched["left_out"].clone(),
            why,
            item["why_left_out"].clone(),
        )
    };
    let forward = shown(&flags);
    let mut reversed = flags;
    reversed.reverse();
    assert_eq!(forward, shown(&reversed));
    assert_eq!(forward.0.len(), 6);
    assert_eq!(forward.1, json!(1));
    assert_eq!(forward.2.len(), 6);
    assert_eq!(forward.3, json!(1));
}

/// The audit's third review, 1: a number the search does not read is an
/// unread *slot* — not an absent one, and not an unread array. What needs
/// the number is undecided, through a not as well; what needs the line's
/// text, template, source or sibling numbers keeps its answer (C93: only
/// the terms that needed it).
#[test]
fn c93_an_unread_number_is_unknown_to_what_asks_it_and_to_nothing_else() {
    let (corpus, _) = fixture(vec![
        lines(&["1.12345 to Spirit"]),
        lines(&["Adds 1 to 10000000000 Cold Damage"]),
    ]);
    // (matched, undecided at the root) of a query asked of one item
    let of = |id: &str, query: &str| -> (u64, u64) {
        let a = run(
            &corpus,
            &request(&format!("id:{id} ({query})"), None, false, 10),
        )
        .unwrap();
        (
            a["total"]["matched"].as_u64().unwrap(),
            a["total"]["undecided"]["count"].as_u64().unwrap(),
        )
    };
    const MATCHED: (u64, u64) = (1, 0);
    const UNDECIDED: (u64, u64) = (0, 1);
    const NO: (u64, u64) = (0, 0);
    for (id, query, want) in [
        ("i0", "line(\"# to Spirit\" arg1>=0)", UNDECIDED),
        ("i0", "line(\"# to Spirit\" -arg1>=0)", UNDECIDED),
        ("i0", "-line(\"# to Spirit\" arg1>=0)", UNDECIDED),
        ("i0", "sum(\"# to Spirit\")>=0", UNDECIDED),
        ("i0", "undecided(line(\"# to Spirit\").arg1)", MATCHED),
        // the line is there, and no other is
        ("i0", "\"# to Spirit\"", MATCHED),
        ("i0", "line(template:spirit source=explicit)", MATCHED),
        ("i0", "-line(template:life)", MATCHED),
        ("i0", "line(template:life arg1>=0)", NO),
        // a sibling number that was read is a witness, and a wrong one is a no
        ("i1", "line(\"Adds # to # Cold Damage\" arg1=1)", MATCHED),
        ("i1", "line(\"Adds # to # Cold Damage\" low=2)", NO),
        ("i1", "line(\"Adds # to # Cold Damage\" high>=5)", UNDECIDED),
        ("i1", "line(\"Adds # to # Cold Damage\" avg>=5)", UNDECIDED),
        ("i1", "line(\"Adds # to # Cold Damage\" low=2 high>=5)", NO),
    ] {
        assert_eq!(of(id, query), want, "{id}: `{query}`");
    }
    // and the item says which numbers, of which lines
    let a = run(
        &corpus,
        &request("line(\"# to Spirit\" arg1>=0)", None, false, 10),
    )
    .unwrap();
    let why = &a["total"]["undecided_items"][0]["why"][0];
    assert_eq!(why["unread"], "the numbers of explicit lines", "{why}");
}

/// The third review, 2: a sum past what a float holds exactly is still the
/// decimal sum, read once: nineteen of the largest numbers the search
/// reads equal the bound typed for them.
#[test]
fn c92_a_sum_at_the_edge_of_what_is_read_equals_its_typed_bound() {
    let nineteen: Vec<String> = (0..19)
        .map(|_| "9999999999.9997 to Spirit".to_string())
        .collect();
    let (corpus, _) = fixture(vec![json!({ "explicitMods": nineteen })]);
    let sum = "sum(\"# to Spirit\")";
    let a = run(
        &corpus,
        &request(&format!("{sum}=189999999999.9943"), Some(sum), false, 10),
    )
    .unwrap();
    assert_eq!(a["total"]["matched"], 1);
    assert_eq!(a["rows"][0]["sort"]["value"], json!(189999999999.9943));
}

/// Found by the completion property the day its generators learned to
/// write an unread number, and nothing to do with one: an item "reaches a
/// bound only together" when its occurrences' sum does — and a sum of no
/// occurrence is no such thing, though 0 is at least any bound of zero or
/// less. The count said 1 and its route, which asks for a selected line,
/// returned none (invariant 4).
#[test]
fn c92_nothing_reaches_a_bound_together_without_an_occurrence_that_counts() {
    let (corpus, _) = fixture(vec![
        json!({ "explicitMods": [{ "description": "Cannot be Frozen", "flags": "unread" }] }),
        lines(&["-3 to maximum Life", "-4 to maximum Life"]),
    ]);
    // the selector is open on the first item's one line, which names no
    // number: the group failed there, and nothing of it sums
    let a = run(
        &corpus,
        &request(
            "line(template~\"Frozen|Life\" is:crafted arg1>=-5)",
            None,
            false,
            10,
        ),
    )
    .unwrap();
    assert_eq!(a["terms"][0]["failed"]["count"], 1, "{}", a["terms"][0]);
    assert_eq!(a["terms"][0]["together"]["count"], 0);
    // two that do sum: -3 and -4 are each under -2, and no sum of them is over
    let a = run(
        &corpus,
        &request("line(template:life arg1>=-2)", None, false, 10),
    )
    .unwrap();
    assert_eq!(a["terms"][0]["together"]["count"], 0);
    let a = run(
        &corpus,
        &request("line(template:life arg1<=-5)", None, false, 10),
    )
    .unwrap();
    assert!(
        a["terms"][0].get("together").is_none(),
        "an upper bound: not applicable"
    );
}
