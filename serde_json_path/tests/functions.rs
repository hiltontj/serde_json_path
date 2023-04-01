use std::cmp::Ordering;

use serde_json::{json, Value};
use serde_json_path::{JsonPath, JsonPathExt};
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

#[test]
fn test_count_singular_query() {
    let value = json!([
        {"foo": "bar"},
        {"foo": "baz"},
    ]);
    let path = JsonPath::parse("$[? count(@.foo) == 1]").unwrap();
    let q = path.query(&value);
    println!("{q:#?}");
    assert_eq!(q.len(), 2);
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

fn get_some_books() -> Value {
    json!([
        {
            "books": [
                {
                    "author": "Alexandre Dumas",
                    "title": "The Three Musketeers",
                    "price": 14.99
                },
                {
                    "author": "Leo Tolstoy",
                    "title": "War and Peace",
                    "price": 19.99
                }
            ]
        },
        {
            "books": [
                {
                    "author": "Charles Dickens",
                    "title": "Great Expectations",
                    "price": 13.99
                },
                {
                    "author": "Fyodor Dostoevsky",
                    "title": "The Brothers Karamazov",
                    "price": 12.99
                }
            ]
        }
    ])
}

#[serde_json_path::function]
fn first(nodes: NodesType) -> ValueType {
    match nodes.first() {
        Some(v) => ValueType::Node(v),
        None => ValueType::Nothing,
    }
}

#[serde_json_path::function]
fn sort<'a>(nodes: NodesType<'a>, on: ValueType<'a>) -> NodesType<'a> {
    if let Some(Ok(path)) = on.as_value().and_then(Value::as_str).map(JsonPath::parse) {
        let mut nl = nodes.all();
        nl.sort_by(|a, b| {
            if let (Some(a), Some(b)) = (
                a.json_path(&path).exactly_one().ok(),
                b.json_path(&path).exactly_one().ok(),
            ) {
                match (a, b) {
                    (Value::Bool(b1), Value::Bool(b2)) => b1.cmp(b2),
                    (Value::Number(n1), Value::Number(n2)) => {
                        if let (Some(f1), Some(f2)) = (n1.as_f64(), n2.as_f64()) {
                            f1.partial_cmp(&f2).unwrap_or(Ordering::Equal)
                        } else if let (Some(u1), Some(u2)) = (n1.as_u64(), n2.as_u64()) {
                            u1.cmp(&u2)
                        } else if let (Some(i1), Some(i2)) = (n1.as_i64(), n2.as_i64()) {
                            i1.cmp(&i2)
                        } else {
                            Ordering::Equal
                        }
                    }
                    (Value::String(s1), Value::String(s2)) => s1.cmp(s2),
                    _ => Ordering::Equal,
                }
            } else {
                Ordering::Equal
            }
        });
        nl.into()
    } else {
        nodes
    }
}

#[serde_json_path::function]
fn get<'a>(node: ValueType<'a>, path: ValueType<'a>) -> ValueType<'a> {
    if let Some(Ok(path)) = path.as_value().and_then(Value::as_str).map(JsonPath::parse) {
        match node {
            ValueType::Node(v) => v
                .json_path(&path)
                .exactly_one()
                .map(ValueType::Node)
                .unwrap_or_default(),
            // This is awkward, because we can't return a reference to a value owned by `node`
            // This means that `get` can only be used when dealing with nodes within the JSON object
            // being queried.
            _ => ValueType::Nothing,
        }
    } else {
        ValueType::Nothing
    }
}

#[test]
fn custom_first_function() {
    let value = get_some_books();
    let path = JsonPath::parse("$[? first(@.books.*.author) == 'Alexandre Dumas']").unwrap();
    let node = path.query(&value).exactly_one().unwrap();
    println!("{node:#?}");
    assert_eq!(
        "War and Peace",
        node.pointer("/books/1/title").unwrap().as_str().unwrap(),
    );
}

#[test]
fn function_as_argument() {
    let value = get_some_books();
    let path = JsonPath::parse("$[? length(first(@.books.*.title)) == 18 ]").unwrap();
    let node = path.query(&value).exactly_one().unwrap();
    println!("{node:#?}");
    assert_eq!(
        "The Brothers Karamazov",
        node.pointer("/books/1/title").unwrap().as_str().unwrap(),
    )
}

#[test]
fn combine_custom_functions() {
    let value = get_some_books();
    let path = JsonPath::parse(
        // `sort` the books in each node by price, take the `first` one, and check for specific
        // title by using `get`:
        "$[? get(first(sort(@.books.*, '$.price')), '$.title') == 'The Brothers Karamazov']",
    )
    .unwrap();
    let node = path.query(&value).exactly_one().unwrap();
    println!("{node:#?}");
    assert_eq!(
        "The Brothers Karamazov",
        node.pointer("/books/1/title").unwrap().as_str().unwrap(),
    );
}
