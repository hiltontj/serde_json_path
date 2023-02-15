use std::{ops::Deref, slice::Iter};

use nom::error::{convert_error, VerboseError};
use parser::parse_path;
use serde::Serialize;
use serde_json::Value;

use crate::parser::QueryValue;

mod parser;

#[derive(Debug, Default, Eq, PartialEq, Serialize)]
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

    pub fn iter(&self) -> Iter<'_, &Value> {
        self.nodes.iter()
    }
}

impl<'a> IntoIterator for Query<'a> {
    type Item = &'a Value;

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.nodes.into_iter()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid JSONPath string:\n{message}")]
    InvalidJsonPathString { message: String },
}

impl<T> From<(T, VerboseError<T>)> for Error
where
    T: Deref<Target = str>,
{
    fn from((i, e): (T, VerboseError<T>)) -> Self {
        Self::InvalidJsonPathString {
            message: convert_error(i, e),
        }
    }
}

pub trait JsonPathExt {
    fn json_path(&self, path_str: &str) -> Result<Query, Error>;
}

impl JsonPathExt for Value {
    fn json_path(&self, path_str: &str) -> Result<Query, Error> {
        let (_, path) = parse_path(path_str).map_err(|err| match err {
            nom::Err::Error(e) | nom::Err::Failure(e) => (path_str, e),
            nom::Err::Incomplete(_) => unreachable!("we do not use streaming parsers"),
        })?;
        let nodes = path.query_value(self, self);
        Ok(Query { nodes })
    }
}
