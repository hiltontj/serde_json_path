#![allow(dead_code)]
use parser::parse_path;
use serde_json::Value;

use crate::parser::QueryValue;

mod parser;

#[derive(Debug, Default)]
pub struct Query<'a> {
    pub(crate) nodes: Vec<&'a Value>,
}

impl<'a> Query<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn as_node(&self) -> Option<&Value> {
        if self.nodes.is_empty() || self.nodes.len() > 1 {
            None
        } else {
            self.nodes.get(0).copied()
        }
    }

    pub fn as_node_list(&self) -> Option<&[&Value]> {
        if self.nodes.is_empty() {
            None
        } else {
            Some(&self.nodes)
        }
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error<T> {
    #[error("invalid JSON Path string")]
    InvalidJsonPathString(#[from] nom::Err<nom::error::Error<T>>),
}

pub trait JsonPathExt {
    fn json_path<'a>(&self, path_str: &'a str) -> Result<Query, Error<&'a str>>;
}

impl JsonPathExt for Value {
    fn json_path<'a>(&self, path_str: &'a str) -> Result<Query, Error<&'a str>> {
        let (_, path) = parse_path(path_str)?;
        let nodes = path.query_value(self, self);
        Ok(Query { nodes })
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{json, Value};

    use crate::JsonPathExt;

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
        let q = value.json_path("$.store.book[*].author").unwrap();
        let nodes = q.as_node_list().unwrap();
        assert_eq!(nodes.len(), 4);
        assert_eq!(nodes[2], "Herman Melville");
    }

    #[test]
    fn spec_example_2() {
        let value = spec_example_json();
        let q = value.json_path("$..author").unwrap();
        let nodes = q.as_node_list().unwrap();
        assert_eq!(nodes.len(), 4);
        assert_eq!(nodes[2], "Herman Melville");
    }

    #[test]
    fn spec_example_3() {
        let value = spec_example_json();
        let q = value.json_path("$.store.*").unwrap();
        let nodes = q.as_node_list().unwrap();
        assert_eq!(nodes.len(), 2);
        assert!(nodes
            .iter()
            .any(|&node| node == value.pointer("/store/book").unwrap()));
    }

    #[test]
    fn spec_example_4() {
        let value = spec_example_json();
        let q = value.json_path("$.store..price").unwrap();
        assert_eq!(q.len(), 5);
    }

    #[test]
    fn spec_example_5() {
        let value = spec_example_json();
        let q = value.json_path("$..book[2]").unwrap();
        let node = q.as_node().unwrap();
        assert_eq!(node, value.pointer("/store/book/2").unwrap());
    }

    #[test]
    fn spec_example_6() {
        let value = spec_example_json();
        let q = value.json_path("$..book[-1]").unwrap();
        let node = q.as_node().unwrap();
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
}
