use std::str::FromStr;

use serde::{de::Visitor, Deserialize, Serialize};
use serde_json::Value;
use serde_json_path_core::{
    node::NodeList,
    spec::query::{Query, Queryable},
};

use crate::{parser::parse_query_main, ParseError};

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
#[derive(Debug, PartialEq, Eq, Clone, Default)]
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
    pub fn parse(path_str: &str) -> Result<Self, ParseError> {
        let (_, path) = parse_query_main(path_str).map_err(|err| match err {
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
    /// # fn main() -> Result<(), serde_json_path::ParseError> {
    /// let path = JsonPath::parse("$.foo[::2]")?;
    /// let value = json!({"foo": [1, 2, 3, 4]});
    /// let nodes = path.query(&value);
    /// assert_eq!(nodes.all(), vec![1, 3]);
    /// # Ok(())
    /// # }
    /// ```
    pub fn query<'b>(&self, value: &'b Value) -> NodeList<'b> {
        self.0.query(value, value).into()
    }
}

impl FromStr for JsonPath {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        JsonPath::parse(s)
    }
}

impl std::fmt::Display for JsonPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{path}", path = self.0)
    }
}

impl Serialize for JsonPath {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(self)
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

#[cfg(test)]
mod tests {
    use serde_json::{from_value, json, to_value};

    use crate::JsonPath;

    #[test]
    fn test_send() {
        fn assert_send<T: Send>() {}
        assert_send::<JsonPath>();
    }

    #[test]
    fn test_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<JsonPath>();
    }

    #[test]
    fn serde_round_trip() {
        let j1 = json!("$.foo['bar'][1:10][?@.baz > 10 && @.foo.bar < 20]");
        let p1 = from_value::<JsonPath>(j1).expect("deserializes");
        let p2 = to_value(&p1)
            .and_then(from_value::<JsonPath>)
            .expect("round trip");
        assert_eq!(p1, p2);
    }
}
