use serde_json::{json, Value};
use serde_json_path::JsonPath;
#[cfg(feature = "trace")]
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
    let path = JsonPath::parse("$.store.book[*].author").unwrap();
    let nodes = path.query(&value).all();
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
    let path = JsonPath::parse("$..author").unwrap();
    let nodes = path.query(&value).all();
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
    let path = JsonPath::parse("$.store.*").unwrap();
    let nodes = path.query(&value).all();
    assert_eq!(nodes.len(), 2);
    assert!(nodes
        .iter()
        .any(|&node| node == value.pointer("/store/book").unwrap()));
}

#[test]
fn spec_example_4() {
    let value = spec_example_json();
    let path = JsonPath::parse("$.store..price").unwrap();
    let nodes = path.query(&value).all();
    assert_eq!(nodes, vec![399., 8.95, 12.99, 8.99, 22.99]);
}

#[test]
fn spec_example_5() {
    let value = spec_example_json();
    let path = JsonPath::parse("$..book[2]").unwrap();
    let node = path.query(&value).at_most_one().unwrap();
    assert!(node.is_some());
    assert_eq!(node, value.pointer("/store/book/2"));
}

#[test]
fn spec_example_6() {
    let value = spec_example_json();
    let path = JsonPath::parse("$..book[-1]").unwrap();
    let node = path.query(&value).at_most_one().unwrap();
    assert!(node.is_some());
    assert_eq!(node, value.pointer("/store/book/3"));
}

#[test]
fn spec_example_7() {
    let value = spec_example_json();
    {
        let path = JsonPath::parse("$..book[0,1]").unwrap();
        let nodes = path.query(&value).all();
        assert_eq!(nodes.len(), 2);
    }
    {
        let path = JsonPath::parse("$..book[:2]").unwrap();
        let nodes = path.query(&value).all();
        assert_eq!(nodes.len(), 2);
    }
}

#[test]
fn spec_example_8() {
    let value = spec_example_json();
    let path = JsonPath::parse("$..book[?(@.isbn)]").unwrap();
    let nodes = path.query(&value);
    assert_eq!(nodes.len(), 2);
}

#[test]
fn spec_example_9() {
    let value = spec_example_json();
    let path = JsonPath::parse("$..book[?(@.price<10)]").unwrap();
    let nodes = path.query(&value);
    assert_eq!(nodes.len(), 2);
}

#[test]
fn spec_example_10() {
    let value = spec_example_json();
    let path = JsonPath::parse("$..*").unwrap();
    let nodes = path.query(&value);
    assert_eq!(nodes.len(), 27);
}
