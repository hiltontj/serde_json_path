//! Types for representing [Normalized Paths][norm-paths] from the JSONPath specification
//!
//! [norm-paths]: https://www.ietf.org/archive/id/draft-ietf-jsonpath-base-21.html#name-normalized-paths
use std::{cmp::Ordering, fmt::Display, slice::Iter};

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
    /// # {
    /// let path = JsonPath::parse("$.foo[? @ == 'bar']")?;
    /// let pointer= path
    ///     .query_located(&value)
    ///     .exactly_one()?
    ///     .location()
    ///     .as_json_pointer();
    /// *value.pointer_mut(&pointer).unwrap() = "bop".into();
    /// assert_eq!(value, json!({"foo": ["bop", "baz"]}));
    /// # }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [json-pointer]: https://datatracker.ietf.org/doc/html/rfc6901
    pub fn as_json_pointer(&self) -> String {
        self.0
            .iter()
            .map(PathElement::as_json_pointer)
            .fold(String::from(""), |mut acc, s| {
                acc.push('/');
                acc.push_str(&s);
                acc
            })
    }

    /// Check if the [`NormalizedPath`] is empty
    ///
    /// This would also represent the normalized path of the root node in a JSON object, i.e.,
    /// `$`.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Get the length of the [`NormalizedPath`]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Get an iterator over the [`PathElement`]s of the [`NormalizedPath`]
    pub fn iter(&self) -> Iter<'_, PathElement<'a>> {
        self.0.iter()
    }
}

impl<'a> Display for NormalizedPath<'a> {
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

impl<'a> Serialize for NormalizedPath<'a> {
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

impl<'a> PathElement<'a> {
    fn as_json_pointer(&self) -> String {
        match self {
            PathElement::Name(s) => s.replace('~', "~0").replace('/', "~1"),
            PathElement::Index(i) => i.to_string(),
        }
    }
}

impl<'a> PartialOrd for PathElement<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (PathElement::Name(a), PathElement::Name(b)) => a.partial_cmp(b),
            (PathElement::Index(a), PathElement::Index(b)) => a.partial_cmp(b),
            _ => None,
        }
    }
}

impl<'a> Display for PathElement<'a> {
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

impl<'a> From<usize> for PathElement<'a> {
    fn from(index: usize) -> Self {
        Self::Index(index)
    }
}

impl<'a> Serialize for PathElement<'a> {
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
        assert_eq!(np.as_json_pointer(), "/foo/42/bar",);
    }

    #[test]
    fn normalized_path_to_json_pointer_with_escapes() {
        let np = NormalizedPath(vec![
            PathElement::Name("foo~bar"),
            PathElement::Index(42),
            PathElement::Name("baz/bop"),
        ]);
        assert_eq!(np.as_json_pointer(), "/foo~0bar/42/baz~1bop",);
    }
}
