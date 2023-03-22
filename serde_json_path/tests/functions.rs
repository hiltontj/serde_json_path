use serde_json::{json, Value};
use serde_json_path::JsonPath;
use serde_json_path_core::spec::functions::{NodesType, ValueType};
#[cfg(feature = "trace")]
use test_log::test;

#[test]
fn test_length() {
    let value = json!([
        "a short string",
        "a slightly longer string",
        "a string that is even longer!",
    ]);
    let query = JsonPath::parse("$[?length(@) > 14]").unwrap();
    let nodes = query.query(&value);
    println!("{nodes:#?}");
    assert_eq!(nodes.len(), 2);
}

#[test]
fn test_length_invalid_args() {
    let error = JsonPath::parse("$[? length(@.foo.*)]").unwrap_err();
    println!("{error:#?}");
}

#[test]
fn test_count() {
    let value = json!([
        {"foo": [1]},
        {"foo": [1, 2]},
    ]);
    let path = JsonPath::parse("$[?count(@.foo.*) > 1]").unwrap();
    let q = path.query(&value);
    assert_eq!(1, q.len());
}

fn simpsons_characters() -> Value {
    json!([
        { "name": "Homer Simpson" },
        { "name": "Marge Simpson" },
        { "name": "Bart Simpson" },
        { "name": "Lisa Simpson" },
        { "name": "Maggie Simpson" },
        { "name": "Ned Flanders" },
        { "name": "Rod Flanders" },
        { "name": "Todd Flanders" },
    ])
}

#[test]
fn test_match() {
    let value = simpsons_characters();
    let path = JsonPath::parse("$[? match(@.name, 'M[A-Za-z ]*Simpson')].name").unwrap();
    let nodes = path.query(&value);
    println!("{nodes:#?}");
    assert_eq!(2, nodes.len());
    assert_eq!("Marge Simpson", nodes.first().unwrap().as_str().unwrap(),);
}

#[test]
fn test_search() {
    let value = simpsons_characters();
    let path = JsonPath::parse("$[? search(@.name, 'Flanders')]").unwrap();
    let nodes = path.query(&value);
    println!("{nodes:#?}");
    assert_eq!(3, nodes.len());
}

#[serde_json_path::function]
fn first(nodes: NodesType) -> ValueType {
    match nodes.into_inner().first() {
        Some(v) => ValueType::ValueRef(v),
        None => ValueType::Nothing,
    }
}

#[test]
fn custom_first_function() {
    let value = json!([
        {
            "books": [
                {
                    "author": "Alexandre Dumas",
                    "title": "The Three Musketeers"
                },
                {
                    "author": "William Schirer",
                    "title": "The Rise and Fall of the Third Reich"
                }
            ]
        },
        {
            "books": [
                {
                    "author": "Charles Dickens",
                    "title": "Great Expectations"
                },
                {
                    "author": "Fyodor Dostoevsky",
                    "title": "The Brothers Karamazov"
                }
            ]
        }
    ]);
    let path = JsonPath::parse("$[?first(@.books.*.author) == 'Alexandre Dumas']").unwrap();
    let node = path.query(&value).exactly_one().unwrap();
    println!("{node:#?}");
    assert_eq!(
        "The Rise and Fall of the Third Reich",
        node.pointer("/books/1/title").unwrap().as_str().unwrap(),
    );
}
