use std::fs;

use serde::Deserialize;
use serde_json::Value;
use serde_json_path::JsonPathExt;

#[derive(Deserialize)]
struct TestSuite {
    tests: Vec<TestCase>,
}

#[derive(Deserialize)]
struct TestCase {
    name: String,
    selector: String,
    #[serde(default)]
    invalid_selector: bool,
    #[serde(default)]
    document: Value,
    #[serde(default)]
    result: Vec<Value>,
}

#[test]
fn compliace_test_suite() {
    let cts_json_str =
        fs::read_to_string("jsonpath-compliance-test-suite/cts.json").expect("read cts.json file");

    let test_cases: TestSuite =
        serde_json::from_str(cts_json_str.as_str()).expect("parse cts_json_str");

    for TestCase {
        name,
        selector,
        invalid_selector,
        document,
        result,
    } in test_cases.tests
    {
        let query = document.json_path(selector.as_str());
        if invalid_selector {
            assert!(
                query.is_err(),
                "{name}: parsing {selector:?} should have failed",
            );
        } else if let Ok(nodelist) = query {
            let actual = nodelist.all();
            let expected = result.iter().collect::<Vec<&Value>>();
            assert_eq!(
                actual, expected,
                "{name}: incorrect result, expected {expected:?}, got {actual:?}"
            );
        } else {
            panic!("{name}: invalid test case")
        }
    }
}
