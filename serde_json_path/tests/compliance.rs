use std::fs;

use serde::Deserialize;
use serde_json::Value;
use serde_json_path::JsonPath;
#[cfg(feature = "trace")]
use test_log::test;

#[derive(Deserialize)]
struct TestSuite {
    tests: Vec<TestCase>,
}

#[derive(Deserialize)]
struct TestCase {
    name: String,
    selector: String,
    #[serde(default)]
    document: Value,
    #[serde(flatten)]
    result: TestResult,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum TestResult {
    Deterministic { result: Vec<Value> },
    NonDeterministic { results: Vec<Vec<Value>> },
    InvalidSelector { invalid_selector: bool },
}

impl TestResult {
    fn verify(&self, name: &str, actual: Vec<&Value>) {
        match self {
            TestResult::Deterministic { result } => assert_eq!(
                result.iter().collect::<Vec<&Value>>(),
                actual,
                "{name}: incorrect result, expected {result:?}, got {actual:?}"
            ),
            TestResult::NonDeterministic { results } => {
                assert!(results
                    .iter()
                    .any(|r| r.iter().collect::<Vec<&Value>>().eq(&actual)))
            }
            TestResult::InvalidSelector { .. } => unreachable!(),
        }
    }

    fn is_invalid_selector(&self) -> bool {
        matches!(self, Self::InvalidSelector { invalid_selector } if *invalid_selector)
    }
}

#[test]
fn compliance_test_suite() {
    let cts_json_str = fs::read_to_string("../jsonpath-compliance-test-suite/cts.json")
        .expect("read cts.json file");

    let test_cases: TestSuite =
        serde_json::from_str(cts_json_str.as_str()).expect("parse cts_json_str");

    for (
        i,
        TestCase {
            name,
            selector,
            document,
            result,
        },
    ) in test_cases.tests.iter().enumerate()
    {
        println!("Test ({i}): {name}");
        let path = JsonPath::parse(selector);
        if result.is_invalid_selector() {
            assert!(
                path.is_err(),
                "{name}: parsing {selector:?} should have failed",
            );
        } else {
            let path = path.expect("valid JSON Path string");
            {
                // Query using JsonPath::query
                let actual = path.query(document).all();
                result.verify(name, actual);
            }
            {
                // Query using JsonPath::query_located
                let q = path.query_located(document);
                let actual = q.nodes().collect::<Vec<&Value>>();
                result.verify(name, actual);
            }
        }
    }
}

const TEST_CASE_N: usize = 10;

#[test]
// #[ignore = "this is only for testing individual CTS test cases as needed"]
fn compliance_single() {
    let cts_json_str = fs::read_to_string("../jsonpath-compliance-test-suite/cts.json")
        .expect("read cts.json file");

    let test_cases: TestSuite =
        serde_json::from_str(cts_json_str.as_str()).expect("parse cts_json_str");

    let TestCase {
        name,
        selector,
        document,
        result,
    } = &test_cases.tests[TEST_CASE_N];
    println!("Test Case: {name}");
    let path = JsonPath::parse(selector);
    if result.is_invalid_selector() {
        println!("...this test should fail");
        assert!(
            path.is_err(),
            "{name}: parsing {selector:?} should have failed",
        );
    } else {
        let path = path.expect("valid JSON Path string");
        {
            // Query using JsonPath::query
            let actual = path.query(document).all();
            result.verify(name, actual);
        }
        {
            // Query using JsonPath::query_located
            let q = path.query_located(document);
            let actual = q.nodes().collect::<Vec<&Value>>();
            result.verify(name, actual);
        }
    }
}
