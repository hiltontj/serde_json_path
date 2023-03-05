use serde_json::{json, Value};
use serde_json_path::JsonPathExt;
use test_log::test;

fn spec_example_json() -> Value {
    json!({
        "store": {
            "book": [
                {
                    "category": "reference",
                    "author": "Nigel Rees",
                    "title": "Sayings of the Century",
                    "price": 8.95
                },
                {
                    "category": "fiction",
                    "author": "Evelyn Waugh",
                    "title": "Sword of Honour",
                    "price": 12.99
                },
                {
                    "category": "fiction",
                    "author": "Herman Melville",
                    "title": "Moby Dick",
                    "isbn": "0-553-21311-3",
                    "price": 8.99
                },
                {
                    "category": "fiction",
                    "author": "J. R. R. Tolkien",
                    "title": "The Lord of the Rings",
                    "isbn": "0-395-19395-8",
                    "price": 22.99
                }
            ],
            "bicycle": {
                "color": "red",
                "price": 399
            }
        }
    })
}

#[test]
fn spec_example_1() {
    let value = spec_example_json();
    let nodes = value.json_path("$.store.book[*].author").unwrap().all();
    assert_eq!(
        nodes,
        vec![
            "Nigel Rees",
            "Evelyn Waugh",
            "Herman Melville",
            "J. R. R. Tolkien"
        ]
    );
}

#[test]
fn spec_example_2() {
    let value = spec_example_json();
    let nodes = value.json_path("$..author").unwrap().all();
    assert_eq!(
        nodes,
        vec![
            "Nigel Rees",
            "Evelyn Waugh",
            "Herman Melville",
            "J. R. R. Tolkien"
        ]
    );
}

#[test]
fn spec_example_3() {
    let value = spec_example_json();
    let nodes = value.json_path("$.store.*").unwrap().all();
    assert_eq!(nodes.len(), 2);
    assert!(nodes
        .iter()
        .any(|&node| node == value.pointer("/store/book").unwrap()));
}

#[test]
fn spec_example_4() {
    let value = spec_example_json();
    let nodes = value.json_path("$.store..price").unwrap().all();
    assert_eq!(nodes, vec![399., 8.95, 12.99, 8.99, 22.99]);
}

#[test]
fn spec_example_5() {
    let value = spec_example_json();
    let q = value.json_path("$..book[2]").unwrap();
    let node = q.one().unwrap();
    assert_eq!(node, value.pointer("/store/book/2").unwrap());
}

#[test]
fn spec_example_6() {
    let value = spec_example_json();
    let q = value.json_path("$..book[-1]").unwrap();
    let node = q.one().unwrap();
    assert_eq!(node, value.pointer("/store/book/3").unwrap());
}

#[test]
fn spec_example_7() {
    let value = spec_example_json();
    {
        let q = value.json_path("$..book[0,1]").unwrap();
        assert_eq!(q.len(), 2);
    }
    {
        let q = value.json_path("$..book[:2]").unwrap();
        assert_eq!(q.len(), 2);
    }
}

#[test]
fn spec_example_8() {
    let value = spec_example_json();
    let q = value.json_path("$..book[?(@.isbn)]").unwrap();
    assert_eq!(q.len(), 2);
}

#[test]
fn spec_example_9() {
    let value = spec_example_json();
    let q = value.json_path("$..book[?(@.price<10)]").unwrap();
    assert_eq!(q.len(), 2);
}

#[test]
fn spec_example_10() {
    let value = spec_example_json();
    let q = value.json_path("$..*").unwrap();
    assert_eq!(q.len(), 27);
}

#[test]
fn test_length() {
    let value = spec_example_json();
    let q = value
        .json_path("$.store.book[?length(@.title) > 10]")
        .unwrap();
    assert_eq!(3, q.len());
}

#[test]
fn test_count() {
    tracing::info!("counting!");
    let value = json!([
        {"foo": [1]},
        {"foo": [1, 2]},
    ]);
    let q = value.json_path("$[?count(@.foo.*) > 1]").unwrap();
    assert_eq!(1, q.len());
}
