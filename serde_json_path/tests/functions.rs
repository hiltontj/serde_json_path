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
                    "title": "The Three Musketeers"
                },
                {
                    "author": "Leo Tolstoy",
                    "title": "War and Peace"
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
    ])
}

#[serde_json_path::function]
fn first(nodes: NodesType) -> ValueType {
    match nodes.into_inner().first() {
        Some(v) => ValueType::Node(v),
        None => ValueType::Nothing,
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
