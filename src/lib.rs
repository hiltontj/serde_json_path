use parser::parse_path;
use serde_json::Value;

use crate::parser::QueryValue;

mod parser;

#[derive(Debug, Default, Eq, PartialEq)]
pub struct Query<'a> {
    pub(crate) nodes: Vec<&'a Value>,
}

impl<'a> Query<'a> {
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
