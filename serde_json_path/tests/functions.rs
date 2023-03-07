use once_cell::sync::Lazy;
use serde_json::json;
use serde_json_path::{Evaluator, FuncType, Function, JsonPathExt};

static FIRST: Evaluator = Lazy::new(|| {
    Box::new(|v| {
        if let Some(FuncType::Nodelist(ref nl)) = v.first() {
            FuncType::Node(nl.first().copied())
        } else {
            FuncType::Nothing
        }
    })
});

inventory::submit! {
    Function::new(
        "first",
        &FIRST,
    )
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
    let q = value
        .json_path("$[?first(@.books.*.author) == 'Alexandre Dumas']")
        .unwrap();
    assert_eq!(
        "The Rise and Fall of the Third Reich",
        q.one()
            .unwrap()
            .pointer("/books/1/title")
            .unwrap()
            .as_str()
            .unwrap(),
    );
}
