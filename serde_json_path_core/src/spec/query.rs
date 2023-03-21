use serde_json::Value;

use super::segment::PathSegment;

pub trait Queryable {
    fn query<'b>(&self, current: &'b Value, root: &'b Value) -> Vec<&'b Value>;
}

/// Represents a JSONPath expression
#[derive(Debug, PartialEq, Clone, Default)]
pub struct Query {
    pub kind: PathKind,
    pub segments: Vec<PathSegment>,
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
            PathKind::Root => write!(f, "$")?,
            PathKind::Current => write!(f, "@")?,
        }
        for s in &self.segments {
            write!(f, "{s}")?;
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub enum PathKind {
    #[default]
    Root,
    Current,
}

impl Queryable for Query {
    #[cfg_attr(feature = "trace", tracing::instrument(name = "Main Query", level = "trace", parent = None, ret))]
    fn query<'b>(&self, current: &'b Value, root: &'b Value) -> Vec<&'b Value> {
        let mut query = match self.kind {
            PathKind::Root => vec![root],
            PathKind::Current => vec![current],
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
