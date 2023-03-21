use serde_json::json;
use serde_json_path::JsonPath;
use serde_json_path_core::spec::functions::{NodesType, ValueType};
#[cfg(feature = "trace")]
use test_log::test;

#[serde_json_path::function]
fn first(nodes: NodesType) -> ValueType {
    match nodes.into_inner().first() {
        Some(v) => ValueType::ValueRef(v),
        None => ValueType::Nothing,
    }
}

#[test]
fn first_function() {
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
