//! Name selector for selecting object keys in JSONPath
use serde_json::Value;

use crate::{node::LocatedNode, path::NormalizedPath, spec::query::Queryable};

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
        f.write_str("'")?;
        for ch in self.0.chars() {
            match ch {
                '\'' => f.write_str("\\'")?,
                '\\' => f.write_str("\\\\")?,
                '\x08' => f.write_str("\\b")?,
                '\x0c' => f.write_str("\\f")?,
                '\n' => f.write_str("\\n")?,
                '\r' => f.write_str("\\r")?,
                '\t' => f.write_str("\\t")?,
                c if (c as u32) < 0x20 => write!(f, "\\u{:04x}", c as u32)?,
                c => write!(f, "{c}")?,
            }
        }
        f.write_str("'")
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

    fn query_located<'b>(
        &self,
        current: &'b Value,
        _root: &'b Value,
        mut parent: NormalizedPath<'b>,
    ) -> Vec<LocatedNode<'b>> {
        if let Some((name, node)) = current.as_object().and_then(|o| o.get_key_value(&self.0)) {
            parent.push(name);
            vec![LocatedNode { loc: parent, node }]
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
