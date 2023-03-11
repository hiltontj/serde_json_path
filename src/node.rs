use std::slice::Iter;

use serde::Serialize;
use serde_json::Value;

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
    /// # use serde_json_path::AtMostOneError;
    /// # fn main() -> Result<(), serde_json_path::Error> {
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
        if self.nodes.is_empty() {
            Ok(None)
        } else if self.nodes.len() > 1 {
            Err(AtMostOneError(self.nodes.len()))
        } else {
            Ok(self.nodes.get(0).copied())
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
    pub fn exactly_one(&self) -> Result<&'a Value, ExactlyOneError> {
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

    /// Returns the first node in the [`NodeList`], or `None` if it is empty
    pub fn first(&self) -> Option<&'a Value> {
        self.nodes.first().copied()
    }

    /// Returns the last node in the [`NodeList`], or `None` if it is empty
    pub fn last(&self) -> Option<&'a Value> {
        self.nodes.last().copied()
    }

    /// Returns the node at the given index in the [`NodeList`], or `None` if the given index is
    /// out of bounds.
    pub fn get(&self, index: usize) -> Option<&'a Value> {
        self.nodes.get(index).copied()
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
    #[deprecated(
        since = "0.5.1",
        note = "it is recommended to use `at_most_one`, `exactly_one`, `first`, `last`, or `get` instead"
    )]
    pub fn one(self) -> Option<&'a Value> {
        if self.nodes.is_empty() || self.nodes.len() > 1 {
            None
        } else {
            self.nodes.get(0).copied()
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("nodelist expected to contain at most one entry, but instead contains {0} entries")]
pub struct AtMostOneError(pub usize);

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

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::JsonPath;

    #[test]
    fn api_tests() {
        let v = json!([1, 2, 3, 4, 5]);
        let q = JsonPath::parse("$.*").expect("valid query").query(&v);
        assert_eq!(q.first().unwrap(), 1);
        assert_eq!(q.last().unwrap(), 5);
        assert_eq!(q.get(1).unwrap(), 2);
    }
}
