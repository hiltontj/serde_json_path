//! Tests for the JSONPath Compliance Test Suite

use std::fs;

use serde::Deserialize;
use serde_json::Value;
use serde_json_path::{JsonPath, LocatedNodeList};
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
    Deterministic {
        result: Vec<Value>,
        result_paths: Vec<String>,
    },
    NonDeterministic {
        results: Vec<Vec<Value>>,
        results_paths: Vec<Vec<String>>,
    },
    InvalidSelector {
        invalid_selector: bool,
    },
}

impl TestResult {
    fn verify(&self, name: &str, actual: Vec<&Value>) {
        match self {
            TestResult::Deterministic { result, .. } => assert_eq!(
                result.iter().collect::<Vec<&Value>>(),
                actual,
                "{name}: incorrect result, expected {result:?}, got {actual:?}"
            ),
            TestResult::NonDeterministic { results, .. } => {
                assert!(results
                    .iter()
                    .any(|r| r.iter().collect::<Vec<&Value>>().eq(&actual)))
            }
            TestResult::InvalidSelector { .. } => unreachable!(),
        }
    }

    /// Verify the normalized paths of the located nodes, see RFC 9535, section 2.7
    fn verify_paths(&self, name: &str, actual: Vec<String>) {
        match self {
            TestResult::Deterministic { result_paths, .. } => assert_eq!(
                result_paths, &actual,
                "{name}: incorrect result paths, expected {result_paths:?}, got {actual:?}"
            ),
            TestResult::NonDeterministic { results_paths, .. } => assert!(
                results_paths.contains(&actual),
                "{name}: incorrect result paths, expected one of {results_paths:?}, got {actual:?}"
            ),
            TestResult::InvalidSelector { .. } => unreachable!(),
        }
    }

    fn is_invalid_selector(&self) -> bool {
        matches!(self, Self::InvalidSelector { invalid_selector } if *invalid_selector)
    }
}

/// A normalized path is itself a valid JSONPath query that selects the node it names
fn verify_paths_round_trip(name: &str, document: &Value, q: &LocatedNodeList<'_>) {
    for node in q.iter() {
        let path = node.location().to_string();
        let reparsed = JsonPath::parse(&path)
            .unwrap_or_else(|e| panic!("{name}: normalized path {path:?} does not parse: {e}"));
        let selected = reparsed.query(document).all();
        assert!(
            selected.len() == 1 && std::ptr::eq(selected[0], node.node()),
            "{name}: normalized path {path:?} selected {} nodes instead of the node it names",
            selected.len(),
        );
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
                let paths = q.locations().map(|l| l.to_string()).collect();
                result.verify_paths(name, paths);
                verify_paths_round_trip(name, document, &q);
            }
        }
    }
}

const TEST_CASE_N: usize = 10;

#[test]
#[ignore = "this is only for testing individual CTS test cases as needed"]
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
            let paths = q.locations().map(|l| l.to_string()).collect();
            result.verify_paths(name, paths);
            verify_paths_round_trip(name, document, &q);
        }
    }
}
