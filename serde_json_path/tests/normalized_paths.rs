//! Tests that normalized paths are valid JSONPath queries, see RFC 9535, section 2.7

use serde_json::{json, Map, Value};
use serde_json_path::JsonPath;

/// Locate the sole node of `{name: "sentinel"}` and re-query the document with its location
fn round_trip(name: &str) {
    let sentinel = json!("sentinel");
    let mut map = Map::new();
    map.insert(name.to_owned(), sentinel.clone());
    let document = Value::Object(map);

    let query = JsonPath::parse("$.*").expect("valid JSON Path string");
    let located = query.query_located(&document);
    let path = located
        .iter()
        .next()
        .expect("one node")
        .location()
        .to_string();

    let reparsed = JsonPath::parse(&path)
        .unwrap_or_else(|e| panic!("name {name:?} gives path {path:?}, which does not parse: {e}"));
    assert_eq!(
        reparsed.query(&document).all(),
        vec![&sentinel],
        "name {name:?} gives path {path:?}, which does not select the node it names",
    );
}

#[test]
fn round_trip_control_characters() {
    for c in '\u{0000}'..='\u{001F}' {
        round_trip(&c.to_string());
        round_trip(&format!("a{c}b"));
    }
}

#[test]
fn round_trip_quotes_and_backslashes() {
    for name in [
        "'",
        "''",
        "O'Reilly",
        r#"\"#,
        r#"\\"#,
        r#"a\b"#,
        r#"\'"#,
        r#"\u0041"#,
        r#"a'b\c"#,
    ] {
        round_trip(name);
    }
}

/// The example that motivated #113, which that PR did not reach
#[test]
fn apostrophe_in_name_is_escaped() {
    let document = json!({"O'Reilly": true});
    let query = JsonPath::parse("$.*").expect("valid JSON Path string");
    let located = query.query_located(&document);
    let path = located.iter().next().expect("one node").location();
    assert_eq!(path.to_string(), r#"$['O\'Reilly']"#);
    assert_eq!(
        serde_json::to_value(path).unwrap(),
        json!(r#"$['O\'Reilly']"#)
    );
}
