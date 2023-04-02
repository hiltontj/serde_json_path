use serde_json::Value;

use crate::{JsonPath, NodeList};

/// Extension trait that allows for JSONPath queries directly on [`serde_json::Value`]
///
/// ## Usage
/// ```rust
/// use serde_json::json;
/// use serde_json_path::{JsonPath, JsonPathExt};
///
/// # fn main() -> Result<(), serde_json_path::ParseError> {
/// let value = json!({"foo": ["bar", "baz"]});
/// let query = JsonPath::parse("$.foo[*]")?;
/// let nodes = value.json_path(&query).all();
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
