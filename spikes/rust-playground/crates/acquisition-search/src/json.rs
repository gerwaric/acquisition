//! The tree's JSON form (C104: the tree is accepted and always returned).
//!
//! The shapes are the reference's worked example: `{"all": […]}`,
//! `{"field", "op", "value"}`, `{"exists": "lines", "where": …}` with
//! `{"attr", "op", "value"}` inside, `{"value": {"pseudo": …}, "op",
//! "number"}`. Reading is strict — an unknown key or a missing one is an
//! error that names its path, and a tree that is read re-serializes to
//! exactly what was read (the refresh slice's lesson on envelopes) — and
//! what is read is checked by [`crate::tree::check`] before it is
//! returned.

use serde_json::{Map, Value as Json, json};

use crate::error::{ErrorKind, LanguageError};
use crate::tree::{self, Collection, Member, Node, Number, Op, Probe, Value, ValueRef};

pub fn to_json(node: &Node) -> Json {
    match node {
        Node::All(children) => json!({ "all": children.iter().map(to_json).collect::<Vec<_>>() }),
        Node::Any(children) => json!({ "any": children.iter().map(to_json).collect::<Vec<_>>() }),
        Node::Not(inner) => json!({ "not": to_json(inner) }),
        Node::Holds { of, min, max } => {
            let mut map = Map::new();
            map.insert("holds".into(), of.iter().map(to_json).collect());
            if let Some(min) = min {
                map.insert("min".into(), json!(min));
            }
            if let Some(max) = max {
                map.insert("max".into(), json!(max));
            }
            Json::Object(map)
        }
        Node::Undecided(Probe::Thing(value)) => {
            json!({ "undecided": { "thing": value_ref_to_json(value) } })
        }
        Node::Undecided(Probe::Term(term)) => json!({ "undecided": { "term": to_json(term) } }),
        Node::Const(b) => json!({ "const": b }),
        Node::Test { field, op, value } => {
            json!({ "field": field, "op": op.as_str(), "value": value_to_json(value) })
        }
        Node::Has(name) => json!({ "has": name }),
        Node::Is(name) => json!({ "is": name }),
        Node::Members { of, where_ } => {
            json!({ "exists": of.json(), "where": member_to_json(where_) })
        }
        Node::Compare { value, op, rhs } => {
            json!({ "value": value_ref_to_json(value), "op": op.as_str(), "number": value_to_json(rhs) })
        }
    }
}

fn member_to_json(member: &Member) -> Json {
    match member {
        Member::All(children) => {
            json!({ "all": children.iter().map(member_to_json).collect::<Vec<_>>() })
        }
        Member::Any(children) => {
            json!({ "any": children.iter().map(member_to_json).collect::<Vec<_>>() })
        }
        Member::Not(inner) => json!({ "not": member_to_json(inner) }),
        Member::Const(b) => json!({ "const": b }),
        Member::Is(name) => json!({ "is": name }),
        Member::Test { attr, op, value } => {
            json!({ "attr": attr, "op": op.as_str(), "value": value_to_json(value) })
        }
    }
}

fn number_to_json(n: Number) -> Json {
    match n {
        Number::Int(i) => json!(i),
        Number::Float(f) => json!(f),
    }
}

fn value_to_json(value: &Value) -> Json {
    match value {
        Value::Text(text) => json!(text),
        Value::Number(n) => number_to_json(*n),
        Value::Range { from, to } => {
            let mut map = Map::new();
            if let Some(from) = from {
                map.insert("from".into(), number_to_json(*from));
            }
            if let Some(to) = to {
                map.insert("to".into(), number_to_json(*to));
            }
            Json::Object(map)
        }
    }
}

pub fn value_ref_to_json(value: &ValueRef) -> Json {
    match value {
        ValueRef::Field(name) => json!({ "field": name }),
        ValueRef::Pseudo { name, slot: None } => json!({ "pseudo": name }),
        ValueRef::Pseudo {
            name,
            slot: Some(slot),
        } => json!({ "pseudo": name, "slot": slot }),
        ValueRef::Sum { lines, slot } => {
            json!({ "sum": { "lines": member_to_json(lines), "slot": slot } })
        }
        ValueRef::Projection { lines, slot } => {
            json!({ "lines": member_to_json(lines), "slot": slot })
        }
    }
}

/// Read a tree, strictly, and check it.
pub fn from_json(json: &Json) -> Result<Node, LanguageError> {
    let node = node_from(json, "$")?;
    tree::check(&node)?;
    Ok(node)
}

/// Read a value, strictly, and check it.
pub fn value_ref_from_json(json: &Json) -> Result<ValueRef, LanguageError> {
    let value = value_ref_from(json, "$")?;
    tree::check(&Node::Undecided(Probe::Thing(value.clone())))?;
    Ok(value)
}

fn bad(path: &str, message: impl std::fmt::Display) -> LanguageError {
    LanguageError::new(ErrorKind::Tree, format!("{path}: {message}"))
}

/// The object at `path`, holding exactly `required` and at most `optional`.
fn object<'j>(
    json: &'j Json,
    path: &str,
    required: &[&str],
    optional: &[&str],
) -> Result<&'j Map<String, Json>, LanguageError> {
    let map = json
        .as_object()
        .ok_or_else(|| bad(path, "expected an object"))?;
    for key in required {
        if !map.contains_key(*key) {
            return Err(bad(path, format!("`{key}` is missing")));
        }
    }
    match map
        .keys()
        .find(|k| !required.contains(&k.as_str()) && !optional.contains(&k.as_str()))
    {
        Some(extra) => Err(bad(
            path,
            format!(
                "`{extra}` does not belong beside `{}`",
                required.join("`, `")
            ),
        )),
        None => Ok(map),
    }
}

fn string<'j>(json: &'j Json, path: &str) -> Result<&'j str, LanguageError> {
    json.as_str().ok_or_else(|| bad(path, "expected a string"))
}

fn list<T>(
    json: &Json,
    path: &str,
    each: fn(&Json, &str) -> Result<T, LanguageError>,
) -> Result<Vec<T>, LanguageError> {
    json.as_array()
        .ok_or_else(|| bad(path, "expected a list"))?
        .iter()
        .enumerate()
        .map(|(i, item)| each(item, &format!("{path}[{i}]")))
        .collect()
}

fn op_from(json: &Json, path: &str) -> Result<Op, LanguageError> {
    Op::parse(string(json, path)?).ok_or_else(|| bad(path, "expected one of : = ~ > >= < <="))
}

fn number_from(json: &Json, path: &str) -> Result<Number, LanguageError> {
    match json {
        Json::Number(n) => match n.as_i64() {
            Some(i) => Ok(Number::Int(i)),
            None => n
                .as_f64()
                .map(Number::from_f64)
                .ok_or_else(|| bad(path, "expected a number")),
        },
        _ => Err(bad(path, "expected a number")),
    }
}

fn value_from(json: &Json, path: &str) -> Result<Value, LanguageError> {
    match json {
        Json::String(text) => Ok(Value::Text(text.clone())),
        Json::Number(_) => Ok(Value::Number(number_from(json, path)?)),
        Json::Object(_) => {
            let map = object(json, path, &[], &["from", "to"])?;
            let side = |key: &str| {
                map.get(key)
                    .map(|n| number_from(n, &format!("{path}.{key}")))
                    .transpose()
            };
            Ok(Value::Range {
                from: side("from")?,
                to: side("to")?,
            })
        }
        _ => Err(bad(
            path,
            "expected a string, a number or a range {from, to}",
        )),
    }
}

fn bound_from(
    map: &Map<String, Json>,
    key: &str,
    path: &str,
) -> Result<Option<u32>, LanguageError> {
    map.get(key)
        .map(|n| {
            n.as_u64()
                .and_then(|n| u32::try_from(n).ok())
                .ok_or_else(|| {
                    bad(
                        &format!("{path}.{key}"),
                        "expected a whole number, zero or more",
                    )
                })
        })
        .transpose()
}

fn node_from(json: &Json, path: &str) -> Result<Node, LanguageError> {
    let map = json
        .as_object()
        .ok_or_else(|| bad(path, "expected an object"))?;
    let at = |key: &str| format!("{path}.{key}");
    if let Some(children) = map.get("all") {
        object(json, path, &["all"], &[])?;
        return Ok(Node::All(list(children, &at("all"), node_from)?));
    }
    if let Some(children) = map.get("any") {
        object(json, path, &["any"], &[])?;
        return Ok(Node::Any(list(children, &at("any"), node_from)?));
    }
    if let Some(inner) = map.get("not") {
        object(json, path, &["not"], &[])?;
        return Ok(Node::Not(Box::new(node_from(inner, &at("not"))?)));
    }
    if let Some(of) = map.get("holds") {
        object(json, path, &["holds"], &["min", "max"])?;
        return Ok(Node::Holds {
            of: list(of, &at("holds"), node_from)?,
            min: bound_from(map, "min", path)?,
            max: bound_from(map, "max", path)?,
        });
    }
    if let Some(probe) = map.get("undecided") {
        object(json, path, &["undecided"], &[])?;
        let inner = probe
            .as_object()
            .ok_or_else(|| bad(&at("undecided"), "expected {thing} or {term}"))?;
        return match (inner.get("thing"), inner.get("term"), inner.len()) {
            (Some(thing), None, 1) => Ok(Node::Undecided(Probe::Thing(value_ref_from(
                thing,
                &at("undecided.thing"),
            )?))),
            (None, Some(term), 1) => Ok(Node::Undecided(Probe::Term(Box::new(node_from(
                term,
                &at("undecided.term"),
            )?)))),
            _ => Err(bad(
                &at("undecided"),
                "expected {thing} or {term}, one of them",
            )),
        };
    }
    if let Some(b) = map.get("const") {
        object(json, path, &["const"], &[])?;
        return b
            .as_bool()
            .map(Node::Const)
            .ok_or_else(|| bad(&at("const"), "expected true or false"));
    }
    if let Some(name) = map.get("has") {
        object(json, path, &["has"], &[])?;
        return Ok(Node::Has(string(name, &at("has"))?.to_string()));
    }
    if let Some(name) = map.get("is") {
        object(json, path, &["is"], &[])?;
        return Ok(Node::Is(string(name, &at("is"))?.to_string()));
    }
    if let Some(of) = map.get("exists") {
        let map = object(json, path, &["exists", "where"], &[])?;
        let of = match string(of, &at("exists"))? {
            "lines" => Collection::Lines,
            "links" => Collection::Links,
            _ => return Err(bad(&at("exists"), "expected \"lines\" or \"links\"")),
        };
        return Ok(Node::Members {
            of,
            where_: Box::new(member_from(&map["where"], &at("where"))?),
        });
    }
    if map.contains_key("field") {
        let map = object(json, path, &["field", "op", "value"], &[])?;
        return Ok(Node::Test {
            field: string(&map["field"], &at("field"))?.to_string(),
            op: op_from(&map["op"], &at("op"))?,
            value: value_from(&map["value"], &at("value"))?,
        });
    }
    if map.contains_key("value") {
        let map = object(json, path, &["value", "op", "number"], &[])?;
        return Ok(Node::Compare {
            value: value_ref_from(&map["value"], &at("value"))?,
            op: op_from(&map["op"], &at("op"))?,
            rhs: value_from(&map["number"], &at("number"))?,
        });
    }
    Err(bad(
        path,
        "not a node: expected one of all, any, not, holds, undecided, const, has, is, exists, field, value",
    ))
}

fn member_from(json: &Json, path: &str) -> Result<Member, LanguageError> {
    let map = json
        .as_object()
        .ok_or_else(|| bad(path, "expected an object"))?;
    let at = |key: &str| format!("{path}.{key}");
    if let Some(children) = map.get("all") {
        object(json, path, &["all"], &[])?;
        return Ok(Member::All(list(children, &at("all"), member_from)?));
    }
    if let Some(children) = map.get("any") {
        object(json, path, &["any"], &[])?;
        return Ok(Member::Any(list(children, &at("any"), member_from)?));
    }
    if let Some(inner) = map.get("not") {
        object(json, path, &["not"], &[])?;
        return Ok(Member::Not(Box::new(member_from(inner, &at("not"))?)));
    }
    if let Some(b) = map.get("const") {
        object(json, path, &["const"], &[])?;
        return b
            .as_bool()
            .map(Member::Const)
            .ok_or_else(|| bad(&at("const"), "expected true or false"));
    }
    if let Some(name) = map.get("is") {
        object(json, path, &["is"], &[])?;
        return Ok(Member::Is(string(name, &at("is"))?.to_string()));
    }
    if map.contains_key("attr") {
        let map = object(json, path, &["attr", "op", "value"], &[])?;
        return Ok(Member::Test {
            attr: string(&map["attr"], &at("attr"))?.to_string(),
            op: op_from(&map["op"], &at("op"))?,
            value: value_from(&map["value"], &at("value"))?,
        });
    }
    Err(bad(
        path,
        "not a member's condition: expected one of all, any, not, const, is, attr",
    ))
}

fn value_ref_from(json: &Json, path: &str) -> Result<ValueRef, LanguageError> {
    let map = json
        .as_object()
        .ok_or_else(|| bad(path, "expected an object"))?;
    let at = |key: &str| format!("{path}.{key}");
    if let Some(name) = map.get("field") {
        object(json, path, &["field"], &[])?;
        return Ok(ValueRef::Field(string(name, &at("field"))?.to_string()));
    }
    if let Some(name) = map.get("pseudo") {
        object(json, path, &["pseudo"], &["slot"])?;
        return Ok(ValueRef::Pseudo {
            name: string(name, &at("pseudo"))?.to_string(),
            slot: map
                .get("slot")
                .map(|s| string(s, &at("slot")).map(str::to_string))
                .transpose()?,
        });
    }
    if let Some(sum) = map.get("sum") {
        object(json, path, &["sum"], &[])?;
        let inner = object(sum, &at("sum"), &["lines", "slot"], &[])?;
        return Ok(ValueRef::Sum {
            lines: Box::new(member_from(&inner["lines"], &at("sum.lines"))?),
            slot: string(&inner["slot"], &at("sum.slot"))?.to_string(),
        });
    }
    if map.contains_key("lines") {
        let map = object(json, path, &["lines", "slot"], &[])?;
        return Ok(ValueRef::Projection {
            lines: Box::new(member_from(&map["lines"], &at("lines"))?),
            slot: string(&map["slot"], &at("slot"))?.to_string(),
        });
    }
    Err(bad(
        path,
        "not a value: expected one of field, pseudo, sum, lines",
    ))
}

impl serde::Serialize for Node {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        to_json(self).serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for Node {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Node, D::Error> {
        let json = Json::deserialize(deserializer)?;
        from_json(&json).map_err(serde::de::Error::custom)
    }
}
