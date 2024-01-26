//! Types representing the different selectors in JSONPath
pub mod filter;
pub mod index;
pub mod name;
pub mod slice;

use serde_json::Value;

use self::{filter::Filter, index::Index, name::Name, slice::Slice};

use super::{
    path::{NormalizedPath, PathElement},
    query::Queryable,
};

/// A JSONPath selector
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Selector {
    /// Select an object key
    Name(Name),
    /// Select all nodes
    ///
    /// For an object, this produces a nodelist of all member values; for an array, this produces a
    /// nodelist of all array elements.
    Wildcard,
    /// Select an array element
    Index(Index),
    /// Select a slice from an array
    ArraySlice(Slice),
    /// Use a filter to select nodes
    Filter(Filter),
}

impl Selector {
    /// Will the selector select at most only a single node
    pub fn is_singular(&self) -> bool {
        matches!(self, Selector::Name(_) | Selector::Index(_))
    }
}

impl std::fmt::Display for Selector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Selector::Name(name) => write!(f, "{name}"),
            Selector::Wildcard => write!(f, "*"),
            Selector::Index(index) => write!(f, "{index}"),
            Selector::ArraySlice(slice) => write!(f, "{slice}"),
            Selector::Filter(filter) => write!(f, "?{filter}"),
        }
    }
}

impl Queryable for Selector {
    #[cfg_attr(feature = "trace", tracing::instrument(name = "Query Selector", level = "trace", parent = None, ret))]
    fn query<'b>(&self, current: &'b Value, root: &'b Value) -> Vec<&'b Value> {
        let mut query = Vec::new();
        match self {
            Selector::Name(name) => query.append(&mut name.query(current, root)),
            Selector::Wildcard => {
                if let Some(list) = current.as_array() {
                    for v in list {
                        query.push(v);
                    }
                } else if let Some(obj) = current.as_object() {
                    for (_, v) in obj {
                        query.push(v);
                    }
                }
            }
            Selector::Index(index) => query.append(&mut index.query(current, root)),
            Selector::ArraySlice(slice) => query.append(&mut slice.query(current, root)),
            Selector::Filter(filter) => query.append(&mut filter.query(current, root)),
        }
        query
    }

    fn query_paths<'b>(
        &self,
        current: &'b Value,
        root: &'b Value,
        parent: NormalizedPath<'b>,
    ) -> Vec<NormalizedPath<'b>> {
        match self {
            Selector::Name(name) => name.query_paths(current, root, parent),
            Selector::Wildcard => {
                if let Some(list) = current.as_array() {
                    list.iter()
                        .enumerate()
                        .map(|(i, _)| {
                            let mut new_path = parent.clone();
                            new_path.push(PathElement::from(i));
                            new_path
                        })
                        .collect()
                } else if let Some(obj) = current.as_object() {
                    obj.keys()
                        .map(|k| {
                            let mut new_path = parent.clone();
                            new_path.push(PathElement::from(k));
                            new_path
                        })
                        .collect()
                } else {
                    vec![]
                }
            }
            Selector::Index(index) => index.query_paths(current, root, parent),
            Selector::ArraySlice(slice) => slice.query_paths(current, root, parent),
            Selector::Filter(filter) => filter.query_paths(current, root, parent),
        }
    }
}
