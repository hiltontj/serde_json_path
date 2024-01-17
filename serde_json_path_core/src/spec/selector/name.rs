//! Name selector for selecting object keys in JSONPath
use serde_json::Value;

use crate::spec::query::{QueryResult, Queryable, TraversedPath};

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
    fn query<'b>(
        &self,
        current: &'b Value,
        _root: &'b Value,
        traversed_path: TraversedPath,
    ) -> QueryResult<'b> {
        let values = if let Some(obj) = current.as_object() {
            obj.get(&self.0).into_iter().collect()
        } else {
            vec![]
        };

        values
            .into_iter()
            .map(|v| {
                (
                    [traversed_path.as_slice(), &[self.0.to_string()]].concat(),
                    v,
                )
            })
            .collect()
    }
}

impl From<&str> for Name {
    fn from(s: &str) -> Self {
        Self(s.to_owned())
    }
}
