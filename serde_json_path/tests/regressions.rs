//! Regression tests for reported issues

use serde_json::json;
use serde_json_path::JsonPath;
#[cfg(feature = "trace")]
use test_log::test;

// This test is meant for issue #49, which can be found here:
// https://github.com/hiltontj/serde_json_path/issues/49
#[test]
fn issue_49() {
    let value = json!({"a": 1, "b": 2});
    let path = JsonPath::parse("$[?(@.a == 2)]").expect("parses JSONPath");
    assert!(path.query(&value).is_empty());
}

// This test is meant for issue #60, which can be found here:
// https://github.com/hiltontj/serde_json_path/issues/60
#[test]
fn issue_60() {
    let value = json!([{"foo": "bar"}, {"foo": "biz"}]);
    let path = JsonPath::parse("$[? match(@.foo, '|')]").expect("parses JSONPath");
    assert!(path.query(&value).is_empty());
}

#[test]
fn name_selector_display_escapes_special_characters() {
    let value = json!({
        "a'b": 1,
        "a\\b": 2,
        "a\nb": 3,
        "a\rb": 4,
        "a\tb": 5,
        "a\u{0008}b": 6,
        "a\u{000c}b": 7,
        "a\u{0000}b": 8,
        "a\"b": 9
    });
    let cases = [
        ("$[\"a'b\"]", "$['a\\'b']", 1),
        ("$['a\\\\b']", "$['a\\\\b']", 2),
        ("$['a\\nb']", "$['a\\nb']", 3),
        ("$['a\\rb']", "$['a\\rb']", 4),
        ("$['a\\tb']", "$['a\\tb']", 5),
        ("$['a\\bb']", "$['a\\bb']", 6),
        ("$['a\\fb']", "$['a\\fb']", 7),
        ("$['a\\u0000b']", "$['a\\u0000b']", 8),
        ("$['a\"b']", "$['a\"b']", 9),
    ];

    for (input, expected_display, expected_val) in cases {
        let parsed = JsonPath::parse(input).expect("parses input JSONPath");
        let rendered = parsed.to_string();
        assert_eq!(rendered, expected_display);
        let reparsed = JsonPath::parse(&rendered).expect("reparses displayed JSONPath");
        assert_eq!(parsed.query(&value).all(), vec![&json!(expected_val)]);
        assert_eq!(reparsed.query(&value).all(), vec![&json!(expected_val)]);
    }
}
