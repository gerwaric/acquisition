//! Temporary outside-review probes; no production changes.
mod common;
use common::*;
use serde_json::{Value, json};

#[test]
#[ignore = "outside-review reproduction: intentionally fails against 8506cc4b"]
fn captured_readings_through_answer() {
    let captures: Value = serde_json::from_str(include_str!(
        "../../../search/pseudo-stats/data/captures.json"
    ))
    .unwrap();
    let names = [
        ("pseudo_count_resistances", "count_res"),
        ("pseudo_count_elemental_resistances", "count_ele_res"),
        (
            "pseudo_total_all_elemental_resistances",
            "total_all_ele_res",
        ),
        ("pseudo_total_all_attributes", "total_all_attributes"),
    ];
    let mut s = store();
    list_tabs(&mut s, "pc", "Standard", json!([tab("t", "Capture")]), 10);
    let mut bodies = Vec::new();
    let mut expected = Vec::new();
    for key in [
        "R1", "g001", "g002", "g003", "g004", "h001", "h002", "h039", "h040",
    ] {
        for (i, capture) in captures["searches"][key]["fetched"]
            .as_array()
            .unwrap()
            .iter()
            .enumerate()
        {
            let id = format!("{key}_{i}");
            let mut more = json!({});
            for (array, lines) in capture["lines"].as_object().unwrap() {
                if array != "pseudoMods" {
                    more[array] = lines.clone();
                }
            }
            bodies.push(item(
                &id,
                capture["name"].as_str().unwrap_or(""),
                capture["baseType"].as_str().unwrap_or("Iron Ring"),
                capture["rarity"].as_str().unwrap_or("Rare"),
                more,
            ));
            for (site, name) in names {
                let hash = format!("stat.pseudo.{site}");
                if let Some(line) = capture["lines"]["pseudoMods"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .find(|line| line["hash"] == hash)
                {
                    let text = line["description"].as_str().unwrap();
                    let number = text
                        .split_whitespace()
                        .next()
                        .unwrap()
                        .trim_end_matches('%')
                        .parse::<f64>()
                        .unwrap();
                    expected.push((id.clone(), name, Some(number)));
                } else if (key == "h002" && name == "total_all_attributes")
                    || (key == "h040" && name == "count_ele_res")
                {
                    expected.push((id.clone(), name, None));
                }
            }
        }
    }
    fetch_tab(&mut s, "pc", "Standard", "t", "Capture", bodies, 20);
    let corpus = load(&s, Some("pc"));
    let mut misses = Vec::new();
    for (id, name, n) in &expected {
        let q = match n {
            Some(n) => format!("id:{id} pseudo.{name}={n}"),
            None => format!("id:{id} -has:pseudo.{name}"),
        };
        let a = as_json(&ask(&corpus, &q).unwrap());
        if a["total"]["matched"] != 1 {
            let absent = as_json(&ask(&corpus, &format!("id:{id} -has:pseudo.{name}")).unwrap());
            misses.push(format!(
                "{q}: {} matches; id:{id} -has:pseudo.{name}: {} matches",
                a["total"]["matched"], absent["total"]["matched"]
            ));
        }
    }
    println!(
        "{} captured readings (including absence); {} mismatches",
        expected.len(),
        misses.len()
    );
    for miss in &misses {
        println!("{miss}");
    }
    assert!(misses.is_empty());
}

#[test]
fn incomplete_count_is_carried_by_the_vocabulary() {
    let mut s = store();
    list_tabs(&mut s, "pc", "Standard", json!([tab("t", "Probe")]), 10);
    fetch_tab(
        &mut s,
        "pc",
        "Standard",
        "t",
        "Probe",
        vec![item(
            "long",
            "Long",
            "Iron Ring",
            "Rare",
            json!({"explicitMods": ["+20% to Fire Resistance", "+20% to Cold Resistance", "+10000000000% to Chaos Resistance"]}),
        )],
        20,
    );
    let corpus = load(&s, Some("pc"));
    assert_eq!(
        as_json(&ask(&corpus, "undecided(pseudo.count_res)").unwrap())["total"]["matched"],
        1
    );
    let request = serde_json::from_value(
        json!({"query": {"text": ""}, "view": {"counts": {"keys": ["line:count_res"]}}}),
    )
    .unwrap();
    let a = as_json(&acquisition_search::answer(&corpus, &request).unwrap());
    let bucket = a["view"]["counts"]["tables"][0]["buckets"]
        .as_array()
        .unwrap()
        .iter()
        .find(|b| b["value"] == "pseudo.count_res")
        .unwrap();
    println!(
        "incomplete pseudo.count_res vocabulary count: {}; route: {}",
        bucket["count"], bucket["term"]
    );
    assert_eq!(bucket["count"], 1);
}
