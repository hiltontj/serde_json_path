//! This crate allows you to use JSONPath queries to extract nodelists from a [`serde_json::Value`].
//!
//! The crate intends to adhere to the [IETF JSONPath specification][jp_spec]. Check out the
//! specification to read more about JSONPath, and to find many examples of its usage.
//!
//! Please note that the specification has not yet been published as an RFC; therefore, this crate
//! may evolve as JSONPath becomes standardized. See [Unimplemented Features](#unimplemented-features)
//! for more details on which parts of the specification are not implemented by this crate.
//!
//! This crate provides two key abstractions:
//!
//! * The [`NodeList`] struct, which represents the result of a JSONPath query performed on a
//!   [`serde_json::Value`].
//! * The [`JsonPathExt`] trait, which extends the [`serde_json::Value`] type with the
//!   [`json_path`][JsonPathExt::json_path] method for performing JSONPath queries.
//!
//! # Usage
//!
//! ## Query for single nodes
//!
//! For queries that are expected to return a single node, use the [`one`][NodeList::one] method:
//!
//! ```rust
//! use serde_json::json;
//! use serde_json_path::JsonPathExt;
//!
//! # fn main() -> Result<(), serde_json_path::Error> {
//! let value = json!({ "foo": { "bar": ["baz", 42] } });
//! let node = value.json_path("$.foo.bar[0]")?.one().unwrap();
//! assert_eq!(node, "baz");
//! # Ok(())
//! # }
//! ```
//! In this regard, the only additional functionality that JSONPath provides over JSON Pointer,
//! and thereby the [`serde_json::Value::pointer`] method, is that you can use reverse array indices:
//!
//! ```rust
//! # use serde_json::json;
//! # use serde_json_path::JsonPathExt;
//! # fn main() -> Result<(), serde_json_path::Error> {
//! let value = json!([1, 2, 3, 4, 5]);
//! let node = value.json_path("$[-1]")?.one().unwrap();
//! assert_eq!(node, 5);
//! # Ok(())
//! # }
//! ```
//!
//! ## Query for multiple nodes
//!
//! For queries that you expect to return zero or many nodes, use the [`all`][NodeList::all]
//! method. There are several selectors in JSONPath that allow you to do this.
//!
//! #### Wildcards (`*`)
//!
//! Wildcards select everything under a current node. They work on both arrays, by selecting all
//! array elements, and on objects, by selecting all object key values:
//!
//! ```rust
//! # use serde_json::json;
//! # use serde_json_path::JsonPathExt;
//! # fn main() -> Result<(), serde_json_path::Error> {
//! let value = json!({ "foo": { "bar": ["baz", "bop"] } });
//! let nodes = value.json_path("$.foo.bar[*]")?.all();
//! assert_eq!(nodes, vec!["baz", "bop"]);
//! # Ok(())
//! # }
//! ```
//!
//! #### Slice selectors (`start:end:step`)
//!
//! Extract slices from JSON arrays using optional `start`, `end`, and `step` values. Reverse
//! indices can be used for `start` and `end`, and a negative `step` can be used to traverse
//! the array in reverse order.
//!
//! ```rust
//! # use serde_json::json;
//! # use serde_json_path::JsonPathExt;
//! # fn main() -> Result<(), serde_json_path::Error> {
//! let value = json!({ "foo": ["bar", "baz", "bop"] });
//! let nodes = value.json_path("$['foo'][1:]")?.all();
//! assert_eq!(nodes, vec!["baz", "bop"]);
//! # Ok(())
//! # }
//! ```
//!
//! #### Filter expressions (`?`)
//!
//! Filter selectors allow you to perform comparisons and check for existence of nodes. You can
//! combine these checks using the boolean `&&` and `||` operators and group using parentheses.
//! The current node (`@`) operator allows you to apply the filtering logic based on the current
//! node being filtered:
//!
//! ```rust
//! # use serde_json::json;
//! # use serde_json_path::JsonPathExt;
//! # fn main() -> Result<(), serde_json_path::Error> {
//! let value = json!({ "foo": [1, 2, 3, 4, 5] });
//! let nodes = value.json_path("$.foo[?@ > 2 && @ < 5]")?.all();
//! assert_eq!(nodes, vec![3, 4]);
//! # Ok(())
//! # }
//! ```
//!
//! You can form relative paths on the current node, as well as absolute paths on the root (`$`)
//! node when writing filters:
//!
//! ```rust
//! # use serde_json::json;
//! # use serde_json_path::JsonPathExt;
//! # fn main() -> Result<(), serde_json_path::Error> {
//! let value = json!([
//!     { "title": "Great Expectations", "price": 10 },
//!     { "title": "Tale of Two Cities", "price": 8 },
//!     { "title": "David Copperfield", "price": 17 }
//! ]);
//! let nodes = value.json_path("$[?@.price > $[0].price].title")?.all();
//! assert_eq!(nodes, vec!["David Copperfield"]);
//! # Ok(())
//! # }
//! ```
//! #### Recursive descent (`..`)
//!
//! ```rust
//! # use serde_json::json;
//! # use serde_json_path::JsonPathExt;
//! # fn main() -> Result<(), serde_json_path::Error> {
//! let value = json!({
//!     "foo": {
//!         "bar": {
//!             "baz": 1
//!         },
//!         "baz": 2
//!     }
//! });
//! let nodes = value.json_path("$.foo..baz")?.all();
//! assert_eq!(nodes, vec![2, 1]);
//! # Ok(())
//! # }
//! ```
//!
//! As seen there, one of the useful features of JSONPath is its ability to extract deeply nested values.
//! Here is another example:
//!
//! ```rust
//! # use serde_json::json;
//! # use serde_json_path::JsonPathExt;
//! # fn main() -> Result<(), serde_json_path::Error> {
//! let value = json!({
//!     "foo": [
//!         { "bar": 1 },
//!         { "bar": 2 },
//!         { "bar": 3 }
//!     ]
//! });
//! let nodes = value.json_path("$.foo.*.bar")?.all();
//! assert_eq!(nodes, vec![1, 2, 3]);
//! # Ok(())
//! # }
//! ```
//!
//! You can combine the above selectors to form powerful and useful queries with JSONPath.
//!
//! See the [integration tests][tests] in the repository for more examples based on those found in
//! the JSONPath specification.
//!
//! # Unimplemented Features
//!
//! Parts of the JSONPath specification that are not implemented by this crate are listed here.
//!
//! * [Function Extensions][func_ext]: this is a planned feature for the crate, but has not yet
//!   been implemented.
//! * [Normalized Paths][norm_path]: this is not a planned feature for the crate.
//!
//! [tests]: https://github.com/hiltontj/serde_json_path/blob/main/tests/main.rs
//! [jp_spec]: https://www.ietf.org/archive/id/draft-ietf-jsonpath-base-10.html
//! [func_ext]: https://www.ietf.org/archive/id/draft-ietf-jsonpath-base-10.html#name-function-extensions-2
//! [norm_path]: https://www.ietf.org/archive/id/draft-ietf-jsonpath-base-10.html#name-normalized-paths
use std::{ops::Deref, slice::Iter};

use nom::error::{convert_error, VerboseError};
use parser::parse_path_main;
use serde::Serialize;
use serde_json::Value;

use crate::parser::Queryable;

mod parser;

pub use parser::selector::function::{Evaluator, FuncType, Function};

/// A list of nodes resulting from a JSONPath query
///
/// Each node within the list is a borrowed reference to the node in the original
/// [`serde_json::Value`] that was queried.
#[derive(Debug, Default, Eq, PartialEq, Serialize)]
pub struct NodeList<'a> {
    pub(crate) nodes: Vec<&'a Value>,
}

impl<'a> NodeList<'a> {
    /// Extract exactly one node from a [`NodeList`]
    ///
    /// This is intended for queries that are expected to yield a single node.
    ///
    /// # Usage
    /// ```rust
    /// # use serde_json::json;
    /// # use serde_json_path::JsonPathExt;
    /// # fn main() -> Result<(), serde_json_path::Error> {
    /// let value = json!({"foo": ["bar", "baz"]});
    /// let query = value.json_path("$.foo[0]")?;
    /// assert_eq!(query.one(), Some(&json!("bar")));
    /// # Ok(())
    /// # }
    /// ```
    pub fn one(self) -> Option<&'a Value> {
        if self.nodes.is_empty() || self.nodes.len() > 1 {
            None
        } else {
            self.nodes.get(0).copied()
        }
    }

    /// Extract all nodes yielded by the query.
    ///
    /// This is intended for queries that are expected to yield zero or more nodes.
    ///
    /// # Usage
    /// ```rust
    /// # use serde_json::json;
    /// # use serde_json_path::JsonPathExt;
    /// # fn main() -> Result<(), serde_json_path::Error> {
    /// let value = json!({"foo": ["bar", "baz"]});
    /// let nodes = value.json_path("$.foo.*")?.all();
    /// assert_eq!(nodes, vec!["bar", "baz"]);
    /// # Ok(())
    /// # }
    /// ```
    pub fn all(self) -> Vec<&'a Value> {
        self.nodes
    }

    /// Get the length of a [`NodeList`]
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Check if a [NodeList] is empty
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Get an iterator over a [`NodeList`]
    ///
    /// Note that [`NodeList`] also implements [`IntoIterator`].
    pub fn iter(&self) -> Iter<'_, &Value> {
        self.nodes.iter()
    }
}

impl<'a> IntoIterator for NodeList<'a> {
    type Item = &'a Value;

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.nodes.into_iter()
    }
}

/// A JSONPath error
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

/// Extension trait that allows for JSONPath queries directly on [`serde_json::Value`]
///
/// ## Usage
/// ```rust
/// use serde_json::json;
/// use serde_json_path::JsonPathExt;
///
/// # fn main() -> Result<(), serde_json_path::Error> {
/// let value = json!({"foo": ["bar", "baz"]});
/// let nodes = value.json_path("$.foo[*]")?.all();
/// assert_eq!(nodes, vec!["bar", "baz"]);
/// # Ok(())
/// # }
/// ```
pub trait JsonPathExt {
    /// Query a [`serde_json::Value`] with a JSONPath query string
    fn json_path(&self, path_str: &str) -> Result<NodeList, Error>;
}

impl JsonPathExt for Value {
    fn json_path(&self, path_str: &str) -> Result<NodeList, Error> {
        let (_, path) = parse_path_main(path_str).map_err(|err| match err {
            nom::Err::Error(e) | nom::Err::Failure(e) => (path_str, e),
            nom::Err::Incomplete(_) => unreachable!("we do not use streaming parsers"),
        })?;
        let nodes = path.query(self, self);
        Ok(NodeList { nodes })
    }
}
