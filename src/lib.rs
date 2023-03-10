//! This crate allows you to use JSONPath queries to extract nodelists from a [`serde_json::Value`].
//!
//! The crate intends to adhere to the [IETF JSONPath specification][jp_spec]. Check out the
//! specification to read more about JSONPath, and to find many examples of its usage.
//!
//! Please note that the specification has not yet been published as an RFC; therefore, this crate
//! may evolve as JSONPath becomes standardized. See [Unimplemented Features](#unimplemented-features)
//! for more details on which parts of the specification are not implemented by this crate.
//!
//! # Features
//!
//! This crate provides two key abstractions:
//!
//! * The [`JsonPath`] struct, which represents a parsed JSONPath query.
//! * The [`NodeList`] struct, which represents the result of a JSONPath query performed on a
//!   [`serde_json::Value`].
//!
//! In addition, the [`JsonPathExt`] trait is provided, which extends the [`serde_json::Value`]
//! type with the [`json_path`][JsonPathExt::json_path] method for performing JSONPath queries.
//!
//! # Usage
//!
//! ## Query for single nodes
//!
//! For queries that are expected to return a single node, use the [`one`][NodeList::one] method:
//!
//! ```rust
//! use serde_json::json;
//! use serde_json_path::JsonPath;
//!
//! # fn main() -> Result<(), serde_json_path::Error> {
//! let value = json!({ "foo": { "bar": ["baz", 42] } });
//! let path = JsonPath::parse("$.foo.bar[0]")?;
//! let node = path.query(&value).at_most_one().unwrap();
//! assert_eq!(node, "baz");
//! # Ok(())
//! # }
//! ```
//! In this regard, the only additional functionality that JSONPath provides over JSON Pointer,
//! and thereby the [`serde_json::Value::pointer`] method, is that you can use reverse array indices:
//!
//! ```rust
//! # use serde_json::json;
//! # use serde_json_path::JsonPath;
//! # fn main() -> Result<(), serde_json_path::Error> {
//! let value = json!([1, 2, 3, 4, 5]);
//! let path = JsonPath::parse("$[-1]")?;
//! let node = path.query(&value).at_most_one().unwrap();
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
//! # use serde_json_path::JsonPath;
//! # fn main() -> Result<(), serde_json_path::Error> {
//! let value = json!({ "foo": { "bar": ["baz", "bop"] } });
//! let path = JsonPath::parse("$.foo.bar[*]")?;
//! let nodes = path.query(&value).all();
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
//! # use serde_json_path::JsonPath;
//! # fn main() -> Result<(), serde_json_path::Error> {
//! let value = json!({ "foo": ["bar", "baz", "bop"] });
//! let path = JsonPath::parse("$['foo'][1:]")?;
//! let nodes = path.query(&value).all();
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
//! # use serde_json_path::JsonPath;
//! # fn main() -> Result<(), serde_json_path::Error> {
//! let value = json!({ "foo": [1, 2, 3, 4, 5] });
//! let path = JsonPath::parse("$.foo[?@ > 2 && @ < 5]")?;
//! let nodes = path.query(&value).all();
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
//! # use serde_json_path::JsonPath;
//! # fn main() -> Result<(), serde_json_path::Error> {
//! let value = json!([
//!     { "title": "Great Expectations", "price": 10 },
//!     { "title": "Tale of Two Cities", "price": 8 },
//!     { "title": "David Copperfield", "price": 17 }
//! ]);
//! let path = JsonPath::parse("$[?@.price > $[0].price].title")?;
//! let nodes = path.query(&value).all();
//! assert_eq!(nodes, vec!["David Copperfield"]);
//! # Ok(())
//! # }
//! ```
//! #### Recursive descent (`..`)
//!
//! ```rust
//! # use serde_json::json;
//! # use serde_json_path::JsonPath;
//! # fn main() -> Result<(), serde_json_path::Error> {
//! let value = json!({
//!     "foo": {
//!         "bar": {
//!             "baz": 1
//!         },
//!         "baz": 2
//!     }
//! });
//! let path = JsonPath::parse("$.foo..baz")?;
//! let nodes = path.query(&value).all();
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
//! # use serde_json_path::JsonPath;
//! # fn main() -> Result<(), serde_json_path::Error> {
//! let value = json!({
//!     "foo": [
//!         { "bar": 1 },
//!         { "bar": 2 },
//!         { "bar": 3 }
//!     ]
//! });
//! let path = JsonPath::parse("$.foo.*.bar")?;
//! let nodes = path.query(&value).all();
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
use std::{ops::Deref, slice::Iter, str::FromStr};

use nom::error::{convert_error, VerboseError};
use parser::{parse_path_main, Query};
use serde::{de::Visitor, Deserialize, Serialize};
use serde_json::Value;

use crate::parser::QueryValue;

mod parser;

/// A parsed JSON Path query string
///
/// This type represents a valid, parsed JSON Path query string. Please refer to the
/// [IETF JSONPath specification][jp_spec] for the details on what constitutes a valid JSON Path
/// query.
///
/// # Usage
///
/// A `JsonPath` can be parsed directly from an `&str` using the [`parse`][JsonPath::parse] method:
/// ```rust
/// # use serde_json_path::JsonPath;
/// # fn main() {
/// let path = JsonPath::parse("$.foo.*").expect("valid JSON Path");
/// # }
/// ```
/// It can then be used to query [`serde_json::Value`]'s with the [`query`][JsonPath::query] method:
/// ```rust
/// # use serde_json::json;
/// # use serde_json_path::JsonPath;
/// # fn main() {
/// # let path = JsonPath::parse("$.foo.*").expect("valid JSON Path");
/// let value = json!({"foo": [1, 2, 3, 4]});
/// let nodes = path.query(&value);
/// assert_eq!(nodes.all(), vec![1, 2, 3, 4]);
/// # }
/// ```
///
/// [jp_spec]: https://www.ietf.org/archive/id/draft-ietf-jsonpath-base-10.html
#[derive(Debug)]
pub struct JsonPath(Query);

impl JsonPath {
    /// Create a [`JsonPath`] by parsing a valid JSON Path query string
    ///
    /// # Example
    /// ```rust
    /// # use serde_json_path::JsonPath;
    /// # fn main() {
    /// let path = JsonPath::parse("$.foo[1:10:2].baz").expect("valid JSON Path");
    /// # }
    /// ```
    pub fn parse(path_str: &str) -> Result<Self, Error> {
        let (_, path) = parse_path_main(path_str).map_err(|err| match err {
            nom::Err::Error(e) | nom::Err::Failure(e) => (path_str, e),
            nom::Err::Incomplete(_) => unreachable!("we do not use streaming parsers"),
        })?;
        Ok(Self(path))
    }

    /// Query a [`serde_json::Value`] using this [`JsonPath`]
    ///
    /// # Example
    /// ```rust
    /// # use serde_json::json;
    /// # use serde_json_path::JsonPath;
    /// # fn main() -> Result<(), serde_json_path::Error> {
    /// let path = JsonPath::parse("$.foo[::2]")?;
    /// let value = json!({"foo": [1, 2, 3, 4]});
    /// let nodes = path.query(&value);
    /// assert_eq!(nodes.all(), vec![1, 3]);
    /// # Ok(())
    /// # }
    /// ```
    pub fn query<'b>(&self, value: &'b Value) -> NodeList<'b> {
        self.0.query_value(value, value).into()
    }
}

impl FromStr for JsonPath {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        JsonPath::parse(s)
    }
}

impl<'de> Deserialize<'de> for JsonPath {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct JsonPathVisitor;

        impl<'de> Visitor<'de> for JsonPathVisitor {
            type Value = JsonPath;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(formatter, "a string representing a JSON Path query")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                JsonPath::parse(v).map_err(serde::de::Error::custom)
            }
        }

        deserializer.deserialize_str(JsonPathVisitor)
    }
}

/// A list of nodes resulting from a JSONPath query
///
/// Each node within the list is a borrowed reference to the node in the original
/// [`serde_json::Value`] that was queried.
#[derive(Debug, Default, Eq, PartialEq, Serialize)]
pub struct NodeList<'a> {
    pub(crate) nodes: Vec<&'a Value>,
}

impl<'a> NodeList<'a> {
    /// Extract _at most_ one node from a [`NodeList`]
    ///
    /// This is intended for queries that are expected to optionally yield a single node.
    ///
    /// # Usage
    /// ```rust
    /// # use serde_json::json;
    /// # use serde_json_path::JsonPath;
    /// # fn main() -> Result<(), serde_json_path::Error> {
    /// let value = json!({"foo": ["bar", "baz"]});
    /// # {
    /// let path = JsonPath::parse("$.foo[0]")?;
    /// let node = path.query(&value).at_most_one();
    /// assert_eq!(node, Some(&json!("bar")));
    /// # }
    /// # {
    /// let path = JsonPath::parse("$.foo.*")?;
    /// let node = path.query(&value).at_most_one();
    /// assert!(node.is_none());
    /// # }
    /// # Ok(())
    /// # }
    /// ```
    pub fn at_most_one(self) -> Option<&'a Value> {
        if self.nodes.is_empty() || self.nodes.len() > 1 {
            None
        } else {
            self.nodes.get(0).copied()
        }
    }

    /// Extract _exactly_ one node from a [`NodeList`]
    ///
    /// This is intended for queries that are expected to yield exactly one node.
    ///
    /// # Usage
    /// ```rust
    /// # use serde_json::json;
    /// # use serde_json_path::JsonPath;
    /// # use serde_json_path::ExactlyOneError;
    /// # fn main() -> Result<(), serde_json_path::Error> {
    /// let value = json!({"foo": ["bar", "baz"]});
    /// # {
    /// let path = JsonPath::parse("$.foo[0]")?;
    /// let node = path.query(&value).exactly_one().unwrap();
    /// assert_eq!(node, "bar");
    /// # }
    /// # {
    /// let path = JsonPath::parse("$.foo.*")?;
    /// let error = path.query(&value).exactly_one().unwrap_err();
    /// assert!(matches!(error, ExactlyOneError::MoreThanOne(2)));
    /// # }
    /// # Ok(())
    /// # }
    /// ```
    pub fn exactly_one(self) -> Result<&'a Value, ExactlyOneError> {
        if self.nodes.is_empty() {
            Err(ExactlyOneError::Empty)
        } else if self.nodes.len() > 1 {
            Err(ExactlyOneError::MoreThanOne(self.nodes.len()))
        } else {
            Ok(self.nodes.get(0).unwrap())
        }
    }

    /// Extract all nodes yielded by the query.
    ///
    /// This is intended for queries that are expected to yield zero or more nodes.
    ///
    /// # Usage
    /// ```rust
    /// # use serde_json::json;
    /// # use serde_json_path::JsonPath;
    /// # fn main() -> Result<(), serde_json_path::Error> {
    /// let value = json!({"foo": ["bar", "baz"]});
    /// let path = JsonPath::parse("$.foo.*")?;
    /// let nodes = path.query(&value).all();
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

    /// Extract _at most_ one node from a [`NodeList`]
    ///
    /// This is intended for queries that are expected to optionally yield a single node.
    ///
    /// # Usage
    /// ```rust
    /// # use serde_json::json;
    /// # use serde_json_path::JsonPath;
    /// # fn main() -> Result<(), serde_json_path::Error> {
    /// let value = json!({"foo": ["bar", "baz"]});
    /// # {
    /// let path = JsonPath::parse("$.foo[0]")?;
    /// let node = path.query(&value).one();
    /// assert_eq!(node, Some(&json!("bar")));
    /// # }
    /// # {
    /// let path = JsonPath::parse("$.foo.*")?;
    /// let node = path.query(&value).one();
    /// assert!(node.is_none());
    /// # }
    /// # Ok(())
    /// # }
    /// ```
    #[deprecated(since = "0.5.1", note = "please use `at_most_one` instead")]
    pub fn one(self) -> Option<&'a Value> {
        if self.nodes.is_empty() || self.nodes.len() > 1 {
            None
        } else {
            self.nodes.get(0).copied()
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ExactlyOneError {
    #[error("nodelist expected to contain one entry, but is empty")]
    Empty,
    #[error("nodelist expected to contain one entry, but instead contains {0} entries")]
    MoreThanOne(usize),
}

impl<'a> From<Vec<&'a Value>> for NodeList<'a> {
    fn from(nodes: Vec<&'a Value>) -> Self {
        Self { nodes }
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
/// use serde_json_path::{JsonPath, JsonPathExt};
///
/// # fn main() -> Result<(), serde_json_path::Error> {
/// let value = json!({"foo": ["bar", "baz"]});
/// let path = JsonPath::parse("$.foo[*]")?;
/// let nodes = path.query(&value).all();
/// assert_eq!(nodes, vec!["bar", "baz"]);
/// # Ok(())
/// # }
/// ```
pub trait JsonPathExt {
    /// Query a [`serde_json::Value`] with a JSONPath query string
    fn json_path(&self, path: &JsonPath) -> NodeList;
}

impl JsonPathExt for Value {
    fn json_path(&self, path: &JsonPath) -> NodeList {
        path.query(self)
    }
}
