//! Types representing queries in JSONPath
use serde_json::Value;

use super::segment::QuerySegment;

mod sealed {
    use crate::spec::{
        segment::{QuerySegment, Segment},
        selector::{
            filter::{Filter, SingularQuery},
            index::Index,
            name::Name,
            slice::Slice,
            Selector,
        },
    };

    use super::Query;

    pub trait Sealed {}
    impl Sealed for Query {}
    impl Sealed for QuerySegment {}
    impl Sealed for Segment {}
    impl Sealed for Slice {}
    impl Sealed for Name {}
    impl Sealed for Selector {}
    impl Sealed for Index {}
    impl Sealed for Filter {}
    impl Sealed for SingularQuery {}
}

/// A type that is query-able
pub trait Queryable: sealed::Sealed {
    /// Query `self` using a current node, and the root node
    fn query<'b>(&self, current: &'b Value, root: &'b Value) -> Vec<&'b Value>;
}

/// Represents a JSONPath expression
#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct Query {
    /// The kind of query, root (`$`), or current (`@`)
    pub kind: QueryKind,
    /// The segments constituting the query
    pub segments: Vec<QuerySegment>,
}

impl Query {
    pub(crate) fn is_singular(&self) -> bool {
        for s in &self.segments {
            if s.is_descendent() {
                return false;
            }
            if !s.segment.is_singular() {
                return false;
            }
        }
        true
    }
}

impl std::fmt::Display for Query {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            QueryKind::Root => write!(f, "$")?,
            QueryKind::Current => write!(f, "@")?,
        }
        for s in &self.segments {
            write!(f, "{s}")?;
        }
        Ok(())
    }
}

/// The kind of query
#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub enum QueryKind {
    /// A query against the root of a JSON object, i.e., with `$`
    #[default]
    Root,
    /// A query against the current node within a JSON object, i.e., with `@`
    Current,
}

impl Queryable for Query {
    #[cfg_attr(feature = "trace", tracing::instrument(name = "Main Query", level = "trace", parent = None, ret))]
    fn query<'b>(&self, current: &'b Value, root: &'b Value) -> Vec<&'b Value> {
        let mut query = match self.kind {
            QueryKind::Root => vec![root],
            QueryKind::Current => vec![current],
        };
        for segment in &self.segments {
            let mut new_query = Vec::new();
            for q in &query {
                new_query.append(&mut segment.query(q, root));
            }
            query = new_query;
        }
        query
    }
}
