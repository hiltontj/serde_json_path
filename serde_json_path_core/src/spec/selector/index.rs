//! Index selectors in JSONPath
use serde_json::Value;

use crate::spec::{path::NormalizedPath, query::Queryable};

/// For selecting array elements by their index
///
/// Can use negative indices to index from the end of an array
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Index(pub isize);

impl std::fmt::Display for Index {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{index}", index = self.0)
    }
}

impl Queryable for Index {
    #[cfg_attr(feature = "trace", tracing::instrument(name = "Query Index", level = "trace", parent = None, ret))]
    fn query<'b>(&self, current: &'b Value, _root: &'b Value) -> Vec<&'b Value> {
        if let Some(list) = current.as_array() {
            if self.0 < 0 {
                self.0
                    .checked_abs()
                    .and_then(|i| usize::try_from(i).ok())
                    .and_then(|i| list.len().checked_sub(i))
                    .and_then(|i| list.get(i))
                    .into_iter()
                    .collect()
            } else {
                usize::try_from(self.0)
                    .ok()
                    .and_then(|i| list.get(i))
                    .into_iter()
                    .collect()
            }
        } else {
            vec![]
        }
    }

    fn query_located<'b>(
        &self,
        current: &'b Value,
        _root: &'b Value,
        mut parent: NormalizedPath<'b>,
    ) -> Vec<(NormalizedPath<'b>, &'b Value)> {
        if let Some((index, value)) = current.as_array().and_then(|list| {
            if self.0 < 0 {
                self.0
                    .checked_abs()
                    .and_then(|i| usize::try_from(i).ok())
                    .and_then(|i| list.len().checked_sub(i))
                    .and_then(|i| list.get(i).map(|v| (i, v)))
            } else {
                usize::try_from(self.0)
                    .ok()
                    .and_then(|i| list.get(i).map(|v| (i, v)))
            }
        }) {
            parent.push(index);
            vec![(parent, value)]
        } else {
            vec![]
        }
    }
}

impl From<isize> for Index {
    fn from(i: isize) -> Self {
        Self(i)
    }
}
