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
