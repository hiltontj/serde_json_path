//! Types representing segments in JSONPath
use serde_json::Value;

use crate::{node::LocatedNode, path::NormalizedPath};

use super::{query::Queryable, selector::Selector};

/// A segment of a JSONPath query
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct QuerySegment {
    /// The kind of segment
    pub kind: QuerySegmentKind,
    /// The segment
    pub segment: Segment,
}

impl QuerySegment {
    /// Is this a normal child segment
    pub fn is_child(&self) -> bool {
        matches!(self.kind, QuerySegmentKind::Child)
    }

    /// Is this a recursive descent child
    pub fn is_descendent(&self) -> bool {
        !self.is_child()
    }
}

impl std::fmt::Display for QuerySegment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if matches!(self.kind, QuerySegmentKind::Descendant) {
            write!(f, "..")?;
        }
        write!(f, "{segment}", segment = self.segment)
    }
}

/// The kind of query segment
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum QuerySegmentKind {
    /// A normal child
    ///
    /// Addresses the direct descented of the preceding segment
    Child,
    /// A descendant child
    ///
    /// Addresses all descendant children of the preceding segment, recursively
    Descendant,
}

impl Queryable for QuerySegment {
    #[cfg_attr(feature = "trace", tracing::instrument(name = "Query Path Segment", level = "trace", parent = None, ret))]
    fn query<'b>(&self, current: &'b Value, root: &'b Value) -> Vec<&'b Value> {
        let mut query = self.segment.query(current, root);
        if matches!(self.kind, QuerySegmentKind::Descendant) {
            query.append(&mut descend(self, current, root));
        }
        query
    }

    fn query_located<'b>(
        &self,
        current: &'b Value,
        root: &'b Value,
        parent: NormalizedPath<'b>,
    ) -> Vec<LocatedNode<'b>> {
        if matches!(self.kind, QuerySegmentKind::Descendant) {
            let mut result = self.segment.query_located(current, root, parent.clone());
            result.append(&mut descend_paths(self, current, root, parent));
            result
        } else {
            self.segment.query_located(current, root, parent)
        }
    }
}

#[cfg_attr(feature = "trace", tracing::instrument(name = "Descend", level = "trace", parent = None, ret))]
fn descend<'b>(segment: &QuerySegment, current: &'b Value, root: &'b Value) -> Vec<&'b Value> {
    let mut query = Vec::new();
    if let Some(list) = current.as_array() {
        for v in list {
            query.append(&mut segment.query(v, root));
        }
    } else if let Some(obj) = current.as_object() {
        for (_, v) in obj {
            query.append(&mut segment.query(v, root));
        }
    }
    query
}

fn descend_paths<'b>(
    segment: &QuerySegment,
    current: &'b Value,
    root: &'b Value,
    parent: NormalizedPath<'b>,
) -> Vec<LocatedNode<'b>> {
    let mut result = Vec::new();
    if let Some(list) = current.as_array() {
        for (i, v) in list.iter().enumerate() {
            result.append(&mut segment.query_located(v, root, parent.clone_and_push(i)));
        }
    } else if let Some(obj) = current.as_object() {
        for (k, v) in obj {
            result.append(&mut segment.query_located(v, root, parent.clone_and_push(k)));
        }
    }
    result
}

/// Represents the different forms of JSONPath segment
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Segment {
    /// Long hand segments contain multiple selectors inside square brackets
    LongHand(Vec<Selector>),
    /// Dot-name selectors are a short form for representing keys in an object
    DotName(String),
    /// The wildcard shorthand `.*`
    Wildcard,
}

impl Segment {
    /// Does this segment extract a singular node
    pub fn is_singular(&self) -> bool {
        match self {
            Segment::LongHand(selectors) => {
                if selectors.len() > 1 {
                    return false;
                }
                if let Some(s) = selectors.first() {
                    s.is_singular()
                } else {
                    // if the selector list is empty, this shouldn't be a valid
                    // JSONPath, but at least, it would be selecting nothing, and
                    // that could be considered singular, i.e., None.
                    true
                }
            }
            Segment::DotName(_) => true,
            Segment::Wildcard => false,
        }
    }

    /// Optionally produce self as a slice of selectors, from a long hand segment
    pub fn as_long_hand(&self) -> Option<&[Selector]> {
        match self {
            Segment::LongHand(v) => Some(v.as_slice()),
            _ => None,
        }
    }

    /// Optionally produce self as a single name segment
    pub fn as_dot_name(&self) -> Option<&str> {
        match self {
            Segment::DotName(s) => Some(s.as_str()),
            _ => None,
        }
    }
}

impl std::fmt::Display for Segment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Segment::LongHand(selectors) => {
                write!(f, "[")?;
                for (i, s) in selectors.iter().enumerate() {
                    write!(
                        f,
                        "{s}{comma}",
                        comma = if i == selectors.len() - 1 { "" } else { "," }
                    )?;
                }
                write!(f, "]")?;
            }
            Segment::DotName(name) => write!(f, ".{name}")?,
            Segment::Wildcard => write!(f, ".*")?,
        }
        Ok(())
    }
}

impl Queryable for Segment {
    #[cfg_attr(feature = "trace", tracing::instrument(name = "Query Segment", level = "trace", parent = None, ret))]
    fn query<'b>(&self, current: &'b Value, root: &'b Value) -> Vec<&'b Value> {
        let mut query = Vec::new();
        match self {
            Segment::LongHand(selectors) => {
                for selector in selectors {
                    query.append(&mut selector.query(current, root));
                }
            }
            Segment::DotName(key) => {
                if let Some(obj) = current.as_object() {
                    if let Some(v) = obj.get(key) {
                        query.push(v);
                    }
                }
            }
            Segment::Wildcard => {
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
        }
        query
    }

    fn query_located<'b>(
        &self,
        current: &'b Value,
        root: &'b Value,
        mut parent: NormalizedPath<'b>,
    ) -> Vec<LocatedNode<'b>> {
        let mut result = vec![];
        match self {
            Segment::LongHand(selectors) => {
                for s in selectors {
                    result.append(&mut s.query_located(current, root, parent.clone()));
                }
            }
            Segment::DotName(name) => {
                if let Some((k, v)) = current.as_object().and_then(|o| o.get_key_value(name)) {
                    parent.push(k);
                    result.push(LocatedNode {
                        loc: parent,
                        node: v,
                    });
                }
            }
            Segment::Wildcard => {
                if let Some(list) = current.as_array() {
                    for (i, v) in list.iter().enumerate() {
                        result.push(LocatedNode {
                            loc: parent.clone_and_push(i),
                            node: v,
                        });
                    }
                } else if let Some(obj) = current.as_object() {
                    for (k, v) in obj {
                        result.push(LocatedNode {
                            loc: parent.clone_and_push(k),
                            node: v,
                        });
                    }
                }
            }
        }
        result
    }
}
