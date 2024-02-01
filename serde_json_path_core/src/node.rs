//! Types representing nodes within a JSON object
use std::{iter::FusedIterator, slice::Iter};

use serde::Serialize;
use serde_json::Value;

use crate::path::NormalizedPath;

/// A list of nodes resulting from a JSONPath query
///
/// Each node within the list is a borrowed reference to the node in the original
/// [`serde_json::Value`] that was queried.
#[derive(Debug, Default, Eq, PartialEq, Serialize, Clone)]
pub struct NodeList<'a>(pub(crate) Vec<&'a Value>);

impl<'a> NodeList<'a> {
    /// Extract _at most_ one node from a [`NodeList`]
    ///
    /// This is intended for queries that are expected to optionally yield a single node.
    ///
    /// # Usage
    /// ```rust
    /// # use serde_json::json;
    /// # use serde_json_path::JsonPath;
    /// # use serde_json_path::AtMostOneError;
    /// # fn main() -> Result<(), serde_json_path::ParseError> {
    /// let value = json!({"foo": ["bar", "baz"]});
    /// # {
    /// let path = JsonPath::parse("$.foo[0]")?;
    /// let node = path.query(&value).at_most_one().unwrap();
    /// assert_eq!(node, Some(&json!("bar")));
    /// # }
    /// # {
    /// let path = JsonPath::parse("$.foo.*")?;
    /// let error = path.query(&value).at_most_one().unwrap_err();
    /// assert!(matches!(error, AtMostOneError(2)));
    /// # }
    /// # Ok(())
    /// # }
    /// ```
    pub fn at_most_one(&self) -> Result<Option<&'a Value>, AtMostOneError> {
        if self.0.is_empty() {
            Ok(None)
        } else if self.0.len() > 1 {
            Err(AtMostOneError(self.0.len()))
        } else {
            Ok(self.0.first().copied())
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
    /// # fn main() -> Result<(), serde_json_path::ParseError> {
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
    pub fn exactly_one(&self) -> Result<&'a Value, ExactlyOneError> {
        if self.0.is_empty() {
            Err(ExactlyOneError::Empty)
        } else if self.0.len() > 1 {
            Err(ExactlyOneError::MoreThanOne(self.0.len()))
        } else {
            Ok(self.0.first().unwrap())
        }
    }

    /// Extract all nodes yielded by the query
    ///
    /// This is intended for queries that are expected to yield zero or more nodes.
    ///
    /// # Usage
    /// ```rust
    /// # use serde_json::json;
    /// # use serde_json_path::JsonPath;
    /// # fn main() -> Result<(), serde_json_path::ParseError> {
    /// let value = json!({"foo": ["bar", "baz"]});
    /// let path = JsonPath::parse("$.foo.*")?;
    /// let nodes = path.query(&value).all();
    /// assert_eq!(nodes, vec!["bar", "baz"]);
    /// # Ok(())
    /// # }
    /// ```
    pub fn all(self) -> Vec<&'a Value> {
        self.0
    }

    /// Get the length of a [`NodeList`]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Check if a [`NodeList`] is empty
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Get an iterator over a [`NodeList`]
    ///
    /// Note that [`NodeList`] also implements [`IntoIterator`].
    pub fn iter(&self) -> Iter<'_, &Value> {
        self.0.iter()
    }

    /// Returns the first node in the [`NodeList`], or `None` if it is empty
    ///
    /// # Usage
    /// ```rust
    /// # use serde_json::json;
    /// # use serde_json_path::JsonPath;
    /// # fn main() -> Result<(), serde_json_path::ParseError> {
    /// let value = json!({"foo": ["bar", "baz"]});
    /// let path = JsonPath::parse("$.foo.*")?;
    /// let node = path.query(&value).first();
    /// assert_eq!(node, Some(&json!("bar")));
    /// # Ok(())
    /// # }
    /// ```
    pub fn first(&self) -> Option<&'a Value> {
        self.0.first().copied()
    }

    /// Returns the last node in the [`NodeList`], or `None` if it is empty
    ///
    /// # Usage
    /// ```rust
    /// # use serde_json::json;
    /// # use serde_json_path::JsonPath;
    /// # fn main() -> Result<(), serde_json_path::ParseError> {
    /// let value = json!({"foo": ["bar", "baz"]});
    /// let path = JsonPath::parse("$.foo.*")?;
    /// let node = path.query(&value).last();
    /// assert_eq!(node, Some(&json!("baz")));
    /// # Ok(())
    /// # }
    /// ```
    pub fn last(&self) -> Option<&'a Value> {
        self.0.last().copied()
    }

    /// Returns the node at the given index in the [`NodeList`], or `None` if the given index is
    /// out of bounds.
    ///
    /// # Usage
    /// ```rust
    /// # use serde_json::json;
    /// # use serde_json_path::JsonPath;
    /// # fn main() -> Result<(), serde_json_path::ParseError> {
    /// let value = json!({"foo": ["bar", "biz", "bop"]});
    /// let path = JsonPath::parse("$.foo.*")?;
    /// let nodes = path.query(&value);
    /// assert_eq!(nodes.get(1), Some(&json!("biz")));
    /// assert!(nodes.get(4).is_none());
    /// # Ok(())
    /// # }
    /// ```
    pub fn get(&self, index: usize) -> Option<&'a Value> {
        self.0.get(index).copied()
    }

    /// Extract _at most_ one node from a [`NodeList`]
    ///
    /// This is intended for queries that are expected to optionally yield a single node.
    ///
    /// # Usage
    /// ```rust
    /// # use serde_json::json;
    /// # use serde_json_path::JsonPath;
    /// # fn main() -> Result<(), serde_json_path::ParseError> {
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
    #[deprecated(
        since = "0.5.1",
        note = "it is recommended to use `at_most_one`, `exactly_one`, `first`, `last`, or `get` instead"
    )]
    pub fn one(self) -> Option<&'a Value> {
        if self.0.is_empty() || self.0.len() > 1 {
            None
        } else {
            self.0.first().copied()
        }
    }
}

impl<'a> From<Vec<&'a Value>> for NodeList<'a> {
    fn from(nodes: Vec<&'a Value>) -> Self {
        Self(nodes)
    }
}

impl<'a> IntoIterator for NodeList<'a> {
    type Item = &'a Value;

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

/// A node within a JSON value, along with its location
#[derive(Debug, Eq, PartialEq, Serialize, Clone)]
pub struct LocatedNode<'a> {
    pub(crate) loc: NormalizedPath<'a>,
    pub(crate) node: &'a Value,
}

impl<'a> LocatedNode<'a> {
    /// Get the location of the node as a [`NormalizedPath`]
    pub fn location(&self) -> &NormalizedPath<'a> {
        &self.loc
    }

    /// Take the location of the node as a [`NormalizedPath`]
    pub fn to_location(self) -> NormalizedPath<'a> {
        self.loc
    }

    /// Get the node itself
    pub fn node(&self) -> &'a Value {
        self.node
    }
}

impl<'a> From<LocatedNode<'a>> for NormalizedPath<'a> {
    fn from(node: LocatedNode<'a>) -> Self {
        node.to_location()
    }
}

#[allow(missing_docs)]
#[derive(Debug, Default, Eq, PartialEq, Serialize, Clone)]
pub struct LocatedNodeList<'a>(Vec<LocatedNode<'a>>);

impl<'a> LocatedNodeList<'a> {
    /// Extract _at most_ one entry from a [`LocatedNodeList`]
    ///
    /// This is intended for queries that are expected to optionally yield a single node.
    ///
    /// # Usage
    /// ```rust
    /// # use serde_json::json;
    /// # use serde_json_path::JsonPath;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let value = json!({"foo": ["bar", "baz"]});
    /// # {
    /// let path = JsonPath::parse("$.foo[0]")?;
    /// let Some(node) = path.query_located(&value).at_most_one()? else {
    ///     /* ... */
    /// #    unreachable!("query should not be empty");
    /// };
    /// assert_eq!("$['foo'][0]", node.location().to_string());
    /// # }
    /// # Ok(())
    /// # }
    /// ```
    pub fn at_most_one(mut self) -> Result<Option<LocatedNode<'a>>, AtMostOneError> {
        if self.0.is_empty() {
            Ok(None)
        } else if self.0.len() > 1 {
            Err(AtMostOneError(self.0.len()))
        } else {
            Ok(Some(self.0.pop().unwrap()))
        }
    }

    /// Extract _exactly_ one entry from a [`LocatedNodeList`]
    ///
    /// This is intended for queries that are expected to yield a single node.
    ///
    /// # Usage
    /// ```rust
    /// # use serde_json::json;
    /// # use serde_json_path::JsonPath;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let value = json!({"foo": ["bar", "baz"]});
    /// # {
    /// let path = JsonPath::parse("$.foo[? @ == 'bar']")?;
    /// let node = path.query_located(&value).exactly_one()?;
    /// assert_eq!("$['foo'][0]", node.location().to_string());
    /// # }
    /// # Ok(())
    /// # }
    /// ```
    pub fn exactly_one(mut self) -> Result<LocatedNode<'a>, ExactlyOneError> {
        if self.0.is_empty() {
            Err(ExactlyOneError::Empty)
        } else if self.0.len() > 1 {
            Err(ExactlyOneError::MoreThanOne(self.0.len()))
        } else {
            Ok(self.0.pop().unwrap())
        }
    }

    /// Extract all located nodes yielded by the query
    ///
    /// This is intended for queries that are expected to yield zero or more nodes.
    ///
    /// # Usage
    /// ```rust
    /// # use serde_json::json;
    /// # use serde_json_path::JsonPath;
    /// # fn main() -> Result<(), serde_json_path::ParseError> {
    /// let value = json!({"foo": ["bar", "baz"]});
    /// let path = JsonPath::parse("$.foo.*")?;
    /// let nodes = path.query_located(&value).all();
    /// assert_eq!(nodes[0].location().to_string(), "$['foo'][0]");
    /// assert_eq!(nodes[0].node(), "bar");
    /// assert_eq!(nodes[1].location().to_string(), "$['foo'][1]");
    /// assert_eq!(nodes[1].node(), "baz");
    /// # Ok(())
    /// # }
    /// ```
    pub fn all(self) -> Vec<LocatedNode<'a>> {
        self.0
    }

    /// Get the length of a [`LocatedNodeList`]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Check if a [`LocatedNodeList`] is empty
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Get an iterator over a [`LocatedNodeList`]
    ///
    /// Note that [`LocatedNodeList`] also implements [`IntoIterator`].
    ///
    /// To iterate over just locations, see [`locations`][LocatedNodeList::locations]. To iterate
    /// over just nodes, see [`nodes`][LocatedNodeList::nodes].
    ///
    /// # Example
    /// ```rust
    /// # use serde_json::{json, Value};
    /// # use serde_json_path::JsonPath;
    /// # fn main() -> Result<(), serde_json_path::ParseError> {
    /// let value = json!({"foo": ["bar", "baz"]});
    /// let path = JsonPath::parse("$.foo.*")?;
    /// let pairs: Vec<(String, &Value)> = path
    ///     .query_located(&value)
    ///     .iter()
    ///     .map(|q| (q.location().to_string(), q.node()))
    ///     .collect();
    /// assert_eq!(pairs[0], ("$['foo'][0]".to_owned(), &json!("bar")));
    /// assert_eq!(pairs[1], ("$['foo'][1]".to_owned(), &json!("baz")));
    /// # Ok(())
    /// # }
    /// ```
    pub fn iter(&self) -> Iter<'_, LocatedNode<'a>> {
        self.0.iter()
    }

    /// Get an iterator over the locations of nodes within a [`LocatedNodeList`]
    ///
    /// # Usage
    /// ```rust
    /// # use serde_json::json;
    /// # use serde_json_path::JsonPath;
    /// # fn main() -> Result<(), serde_json_path::ParseError> {
    /// let value = json!({"foo": ["bar", "baz"]});
    /// let path = JsonPath::parse("$.foo.*")?;
    /// let locations: Vec<String> = path
    ///     .query_located(&value)
    ///     .locations()
    ///     .map(|loc| loc.to_string())
    ///     .collect();
    /// assert_eq!(locations, ["$['foo'][0]", "$['foo'][1]"]);
    /// # Ok(())
    /// # }
    /// ```
    pub fn locations(&self) -> Locations<'_> {
        Locations { inner: self.iter() }
    }

    /// Get an iterator over the nodes within a [`LocatedNodeList`]
    pub fn nodes(&self) -> Nodes<'_> {
        Nodes { inner: self.iter() }
    }

    /// Deduplicate a [`LocatedNodeList`] and return the result
    ///
    /// See also, [`dedup_in_place`][LocatedNodeList::dedup_in_place].
    ///
    /// # Usage
    /// ```rust
    /// # use serde_json::json;
    /// # use serde_json_path::JsonPath;
    /// # fn main() -> Result<(), serde_json_path::ParseError> {
    /// let value = json!({"foo": ["bar", "baz"]});
    /// let path = JsonPath::parse("$.foo[0, 0, 1, 1]")?;
    /// let nodes = path.query_located(&value);
    /// assert_eq!(4, nodes.len());
    /// let nodes = path.query_located(&value).dedup();
    /// assert_eq!(2, nodes.len());
    /// # Ok(())
    /// # }
    /// ```
    pub fn dedup(mut self) -> Self {
        self.dedup_in_place();
        self
    }

    /// Deduplicate a [`LocatedNodeList`] _in-place_
    ///
    /// See also, [`dedup`][LocatedNodeList::dedup].
    ///
    /// # Usage
    /// ```rust
    /// # use serde_json::json;
    /// # use serde_json_path::JsonPath;
    /// # fn main() -> Result<(), serde_json_path::ParseError> {
    /// let value = json!({"foo": ["bar", "baz"]});
    /// let path = JsonPath::parse("$.foo[0, 0, 1, 1]")?;
    /// let mut nodes = path.query_located(&value);
    /// assert_eq!(4, nodes.len());
    /// nodes.dedup_in_place();
    /// assert_eq!(2, nodes.len());
    /// # Ok(())
    /// # }
    /// ```
    pub fn dedup_in_place(&mut self) {
        // This unwrap should be safe, since the paths corresponding to
        // a query against a Value will always be ordered.
        self.0
            .sort_unstable_by(|a, b| a.loc.partial_cmp(&b.loc).unwrap());
        self.0.dedup();
    }

    /// Return the first entry in the [`LocatedNodeList`], or `None` if it is empty
    ///
    /// # Usage
    /// ```rust
    /// # use serde_json::json;
    /// # use serde_json_path::JsonPath;
    /// # use serde_json_path::LocatedNode;
    /// # fn main() -> Result<(), serde_json_path::ParseError> {
    /// let value = json!({"foo": ["bar", "baz"]});
    /// let path = JsonPath::parse("$.foo.*")?;
    /// let nodes = path.query_located(&value);
    /// let first = nodes.first().map(LocatedNode::node);
    /// assert_eq!(first, Some(&json!("bar")));
    /// # Ok(())
    /// # }
    /// ```
    pub fn first(&self) -> Option<&LocatedNode<'a>> {
        self.0.first()
    }

    /// Return the last entry in the [`LocatedNodeList`], or `None` if it is empty
    ///
    /// # Usage
    /// ```rust
    /// # use serde_json::json;
    /// # use serde_json_path::JsonPath;
    /// # use serde_json_path::LocatedNode;
    /// # fn main() -> Result<(), serde_json_path::ParseError> {
    /// let value = json!({"foo": ["bar", "baz"]});
    /// let path = JsonPath::parse("$.foo.*")?;
    /// let nodes = path.query_located(&value);
    /// let last = nodes.last().map(LocatedNode::node);
    /// assert_eq!(last, Some(&json!("baz")));
    /// # Ok(())
    /// # }
    /// ```
    pub fn last(&self) -> Option<&LocatedNode<'a>> {
        self.0.last()
    }

    /// Returns the node at the given index in the [`LocatedNodeList`], or `None` if the
    /// given index is out of bounds.
    ///
    /// # Usage
    /// ```rust
    /// # use serde_json::json;
    /// # use serde_json_path::JsonPath;
    /// # use serde_json_path::LocatedNode;
    /// # fn main() -> Result<(), serde_json_path::ParseError> {
    /// let value = json!({"foo": ["bar", "biz", "bop"]});
    /// let path = JsonPath::parse("$.foo.*")?;
    /// let nodes = path.query_located(&value);
    /// assert_eq!(nodes.get(1).map(LocatedNode::node), Some(&json!("biz")));
    /// assert!(nodes.get(4).is_none());
    /// # Ok(())
    /// # }
    /// ```
    pub fn get(&self, index: usize) -> Option<&LocatedNode<'a>> {
        self.0.get(index)
    }
}

impl<'a> From<Vec<LocatedNode<'a>>> for LocatedNodeList<'a> {
    fn from(v: Vec<LocatedNode<'a>>) -> Self {
        Self(v)
    }
}

impl<'a> IntoIterator for LocatedNodeList<'a> {
    type Item = LocatedNode<'a>;

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

/// An iterator over the locations in a [`LocatedNodeList`]
///
/// Produced by the [`LocatedNodeList::locations`] method.
#[derive(Debug)]
pub struct Locations<'a> {
    inner: Iter<'a, LocatedNode<'a>>,
}

impl<'a> Iterator for Locations<'a> {
    type Item = &'a NormalizedPath<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|l| l.location())
    }
}

impl<'a> DoubleEndedIterator for Locations<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back().map(|l| l.location())
    }
}

impl<'a> ExactSizeIterator for Locations<'a> {
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<'a> FusedIterator for Locations<'a> {}

/// An iterator over the nodes in a [`LocatedNodeList`]
///
/// Produced by the [`LocatedNodeList::nodes`] method.
#[derive(Debug)]
pub struct Nodes<'a> {
    inner: Iter<'a, LocatedNode<'a>>,
}

impl<'a> Iterator for Nodes<'a> {
    type Item = &'a Value;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|l| l.node())
    }
}

impl<'a> DoubleEndedIterator for Nodes<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back().map(|l| l.node())
    }
}

impl<'a> ExactSizeIterator for Nodes<'a> {
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<'a> FusedIterator for Nodes<'a> {}

/// Error produced when expecting no more than one node from a query
#[derive(Debug, thiserror::Error)]
#[error("nodelist expected to contain at most one entry, but instead contains {0} entries")]
pub struct AtMostOneError(pub usize);

/// Error produced when expecting exactly one node from a query
#[derive(Debug, thiserror::Error)]
pub enum ExactlyOneError {
    /// The query resulted in an empty [`NodeList`]
    #[error("nodelist expected to contain one entry, but is empty")]
    Empty,
    /// The query resulted in a [`NodeList`] containing more than one node
    #[error("nodelist expected to contain one entry, but instead contains {0} entries")]
    MoreThanOne(usize),
}

impl ExactlyOneError {
    /// Check that it is the `Empty` variant
    pub fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }

    /// Check that it is the `MoreThanOne` variant
    pub fn is_more_than_one(&self) -> bool {
        self.as_more_than_one().is_some()
    }

    /// Extract the number of nodes, if it was more than one, or `None` otherwise
    pub fn as_more_than_one(&self) -> Option<usize> {
        match self {
            ExactlyOneError::Empty => None,
            ExactlyOneError::MoreThanOne(u) => Some(*u),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::node::NodeList;
    use serde_json::{json, to_value};
    use serde_json_path::JsonPath;

    #[test]
    fn test_send() {
        fn assert_send<T: Send>() {}
        assert_send::<NodeList>();
    }

    #[test]
    fn test_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<NodeList>();
    }

    #[test]
    fn test_serialize() {
        let v = json!([1, 2, 3, 4]);
        let q = JsonPath::parse("$.*").expect("valid query").query(&v);
        assert_eq!(to_value(q).expect("serialize"), v);
    }
}
