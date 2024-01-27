//! Name selector for selecting object keys in JSONPath
use serde_json::Value;

use crate::spec::{path::NormalizedPath, query::Queryable};

/// Select a single JSON object key
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Name(pub String);

impl Name {
    /// Get as a string slice
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "'{name}'", name = self.0)
    }
}

impl Queryable for Name {
    #[cfg_attr(feature = "trace", tracing::instrument(name = "Query Name", level = "trace", parent = None, ret))]
    fn query<'b>(&self, current: &'b Value, _root: &'b Value) -> Vec<&'b Value> {
        if let Some(obj) = current.as_object() {
            obj.get(&self.0).into_iter().collect()
        } else {
            vec![]
        }
    }

    fn query_paths<'b>(
        &self,
        current: &'b Value,
        _root: &'b Value,
        mut parent: NormalizedPath<'b>,
    ) -> Vec<(NormalizedPath<'b>, &'b Value)> {
        if let Some((s, v)) = current.as_object().and_then(|o| o.get_key_value(&self.0)) {
            parent.push(s);
            vec![(parent, v)]
        } else {
            vec![]
        }
    }
}

impl From<&str> for Name {
    fn from(s: &str) -> Self {
        Self(s.to_owned())
    }
}
