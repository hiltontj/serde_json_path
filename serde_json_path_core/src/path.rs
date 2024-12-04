//! Types for representing [Normalized Paths][norm-paths] from the JSONPath specification
//!
//! [norm-paths]: https://www.rfc-editor.org/rfc/rfc9535.html#name-normalized-paths
use std::{
    cmp::Ordering,
    fmt::Display,
    slice::{Iter, SliceIndex},
};

use serde::Serialize;

// Documented in the serde_json_path crate, for linking purposes
#[allow(missing_docs)]
#[derive(Debug, Default, Eq, PartialEq, Clone, PartialOrd)]
pub struct NormalizedPath<'a>(Vec<PathElement<'a>>);

impl<'a> NormalizedPath<'a> {
    pub(crate) fn push<T: Into<PathElement<'a>>>(&mut self, elem: T) {
        self.0.push(elem.into())
    }

    pub(crate) fn clone_and_push<T: Into<PathElement<'a>>>(&self, elem: T) -> Self {
        let mut new_path = self.clone();
        new_path.push(elem.into());
        new_path
    }

    /// Get the [`NormalizedPath`] as a [JSON Pointer][json-pointer] string
    ///
    /// This can be used with the [`serde_json::Value::pointer`] or
    /// [`serde_json::Value::pointer_mut`] methods.
    ///
    /// # Example
    /// ```rust
    /// # use serde_json::json;
    /// # use serde_json_path::JsonPath;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut value = json!({"foo": ["bar", "baz"]});
    /// let path = JsonPath::parse("$.foo[? @ == 'bar']")?;
    /// let pointer= path
    ///     .query_located(&value)
    ///     .exactly_one()?
    ///     .location()
    ///     .to_json_pointer();
    /// *value.pointer_mut(&pointer).unwrap() = "bop".into();
    /// assert_eq!(value, json!({"foo": ["bop", "baz"]}));
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [json-pointer]: https://datatracker.ietf.org/doc/html/rfc6901
    pub fn to_json_pointer(&self) -> String {
        self.0
            .iter()
            .map(PathElement::to_json_pointer)
            .fold(String::from(""), |mut acc, s| {
                acc.push('/');
                acc.push_str(&s);
                acc
            })
    }

    /// Check if the [`NormalizedPath`] is empty
    ///
    /// An empty normalized path represents the location of the root node of the JSON object,
    /// i.e., `$`.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Get the length of the [`NormalizedPath`]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Get an iterator over the [`PathElement`]s of the [`NormalizedPath`]
    ///
    /// Note that [`NormalizedPath`] also implements [`IntoIterator`]
    ///
    /// # Example
    /// ```rust
    /// # use serde_json::json;
    /// # use serde_json_path::JsonPath;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut value = json!({"foo": {"bar": 1, "baz": 2, "bop": 3}});
    /// let path = JsonPath::parse("$.foo[? @ == 2]")?;
    /// let location = path.query_located(&value).exactly_one()?.to_location();
    /// let elements: Vec<String> = location
    ///     .iter()
    ///     .map(|ele| ele.to_string())
    ///     .collect();
    /// assert_eq!(elements, ["foo", "baz"]);
    /// # Ok(())
    /// # }
    /// ```
    pub fn iter(&self) -> Iter<'_, PathElement<'a>> {
        self.0.iter()
    }

    /// Get the [`PathElement`] at `index`, or `None` if the index is out of bounds
    ///
    /// # Example
    /// ```rust
    /// # use serde_json::json;
    /// # use serde_json_path::JsonPath;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let value = json!({"foo": {"bar": {"baz": "bop"}}});
    /// let path = JsonPath::parse("$..baz")?;
    /// let location = path.query_located(&value).exactly_one()?.to_location();
    /// assert_eq!(location.to_string(), "$['foo']['bar']['baz']");
    /// assert!(location.get(0).is_some_and(|p| p == "foo"));
    /// assert!(location.get(1..).is_some_and(|p| p == ["bar", "baz"]));
    /// assert!(location.get(3).is_none());
    /// # Ok(())
    /// # }
    /// ```
    pub fn get<I>(&self, index: I) -> Option<&I::Output>
    where
        I: SliceIndex<[PathElement<'a>]>,
    {
        self.0.get(index)
    }

    /// Get the first [`PathElement`], or `None` if the path is empty
    ///
    /// # Example
    /// ```rust
    /// # use serde_json::json;
    /// # use serde_json_path::JsonPath;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let value = json!(["foo", true, {"bar": false}, {"bar": true}]);
    /// let path = JsonPath::parse("$..[? @ == false]")?;
    /// let location = path.query_located(&value).exactly_one()?.to_location();
    /// assert_eq!(location.to_string(), "$[2]['bar']");
    /// assert!(location.first().is_some_and(|p| *p == 2));
    /// # Ok(())
    /// # }
    /// ```
    pub fn first(&self) -> Option<&PathElement<'a>> {
        self.0.first()
    }

    /// Get the last [`PathElement`], or `None` if the path is empty
    ///
    /// # Example
    /// ```rust
    /// # use serde_json::json;
    /// # use serde_json_path::JsonPath;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let value = json!({"foo": {"bar": [1, 2, 3]}});
    /// let path = JsonPath::parse("$..[? @ == 2]")?;
    /// let location = path.query_located(&value).exactly_one()?.to_location();
    /// assert_eq!(location.to_string(), "$['foo']['bar'][1]");
    /// assert!(location.last().is_some_and(|p| *p == 1));
    /// # Ok(())
    /// # }
    /// ```
    pub fn last(&self) -> Option<&PathElement<'a>> {
        self.0.last()
    }
}

impl<'a> IntoIterator for NormalizedPath<'a> {
    type Item = PathElement<'a>;

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl Display for NormalizedPath<'_> {
    /// Format the [`NormalizedPath`] as a JSONPath string using the canonical bracket notation
    /// as per the [JSONPath Specification][norm-paths]
    ///
    /// # Example
    /// ```rust
    /// # use serde_json::json;
    /// # use serde_json_path::JsonPath;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let value = json!({"foo": ["bar", "baz"]});
    /// let path = JsonPath::parse("$.foo[0]")?;
    /// let location = path.query_located(&value).exactly_one()?.to_location();
    /// assert_eq!(location.to_string(), "$['foo'][0]");
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [norm-paths]: https://www.rfc-editor.org/rfc/rfc9535.html#name-normalized-paths
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "$")?;
        for elem in &self.0 {
            match elem {
                PathElement::Name(name) => write!(f, "['{name}']")?,
                PathElement::Index(index) => write!(f, "[{index}]")?,
            }
        }
        Ok(())
    }
}

impl Serialize for NormalizedPath<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_string().as_str())
    }
}

/// An element within a [`NormalizedPath`]
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum PathElement<'a> {
    /// A key within a JSON object
    Name(&'a str),
    /// An index of a JSON Array
    Index(usize),
}

impl PathElement<'_> {
    fn to_json_pointer(&self) -> String {
        match self {
            PathElement::Name(s) => s.replace('~', "~0").replace('/', "~1"),
            PathElement::Index(i) => i.to_string(),
        }
    }

    /// Get the underlying name if the [`PathElement`] is `Name`, or `None` otherwise
    pub fn as_name(&self) -> Option<&str> {
        match self {
            PathElement::Name(n) => Some(n),
            PathElement::Index(_) => None,
        }
    }

    /// Get the underlying index if the [`PathElement`] is `Index`, or `None` otherwise
    pub fn as_index(&self) -> Option<usize> {
        match self {
            PathElement::Name(_) => None,
            PathElement::Index(i) => Some(*i),
        }
    }

    /// Test if the [`PathElement`] is `Name`
    pub fn is_name(&self) -> bool {
        self.as_name().is_some()
    }

    /// Test if the [`PathElement`] is `Index`
    pub fn is_index(&self) -> bool {
        self.as_index().is_some()
    }
}

impl PartialOrd for PathElement<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (PathElement::Name(a), PathElement::Name(b)) => a.partial_cmp(b),
            (PathElement::Index(a), PathElement::Index(b)) => a.partial_cmp(b),
            _ => None,
        }
    }
}

impl PartialEq<str> for PathElement<'_> {
    fn eq(&self, other: &str) -> bool {
        match self {
            PathElement::Name(s) => s.eq(&other),
            PathElement::Index(_) => false,
        }
    }
}

impl PartialEq<&str> for PathElement<'_> {
    fn eq(&self, other: &&str) -> bool {
        match self {
            PathElement::Name(s) => s.eq(other),
            PathElement::Index(_) => false,
        }
    }
}

impl PartialEq<usize> for PathElement<'_> {
    fn eq(&self, other: &usize) -> bool {
        match self {
            PathElement::Name(_) => false,
            PathElement::Index(i) => i.eq(other),
        }
    }
}

impl Display for PathElement<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PathElement::Name(n) => write!(f, "{n}"),
            PathElement::Index(i) => write!(f, "{i}"),
        }
    }
}

impl<'a> From<&'a String> for PathElement<'a> {
    fn from(s: &'a String) -> Self {
        Self::Name(s.as_str())
    }
}

impl From<usize> for PathElement<'_> {
    fn from(index: usize) -> Self {
        Self::Index(index)
    }
}

impl Serialize for PathElement<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            PathElement::Name(s) => serializer.serialize_str(s),
            PathElement::Index(i) => serializer.serialize_u64(*i as u64),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{NormalizedPath, PathElement};

    #[test]
    fn normalized_path_to_json_pointer() {
        let np = NormalizedPath(vec![
            PathElement::Name("foo"),
            PathElement::Index(42),
            PathElement::Name("bar"),
        ]);
        assert_eq!(np.to_json_pointer(), "/foo/42/bar");
    }

    #[test]
    fn normalized_path_to_json_pointer_with_escapes() {
        let np = NormalizedPath(vec![
            PathElement::Name("foo~bar"),
            PathElement::Index(42),
            PathElement::Name("baz/bop"),
        ]);
        assert_eq!(np.to_json_pointer(), "/foo~0bar/42/baz~1bop");
    }
}
