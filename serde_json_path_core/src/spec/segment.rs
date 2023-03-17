use serde_json::Value;

use super::{query::Queryable, selector::Selector};

#[derive(Debug, PartialEq, Clone)]
pub struct PathSegment {
    pub kind: PathSegmentKind,
    pub segment: Segment,
}

impl PathSegment {
    pub fn is_child(&self) -> bool {
        matches!(self.kind, PathSegmentKind::Child)
    }

    pub fn is_descendent(&self) -> bool {
        !self.is_child()
    }
}

impl std::fmt::Display for PathSegment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if matches!(self.kind, PathSegmentKind::Descendant) {
            write!(f, "..")?;
        }
        write!(f, "{segment}", segment = self.segment)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum PathSegmentKind {
    Child,
    Descendant,
}

impl Queryable for PathSegment {
    #[cfg_attr(feature = "trace", tracing::instrument(name = "Query Path Segment", level = "trace", parent = None, ret))]
    fn query<'b>(&self, current: &'b Value, root: &'b Value) -> Vec<&'b Value> {
        let mut query = self.segment.query(current, root);
        if matches!(self.kind, PathSegmentKind::Descendant) {
            query.append(&mut descend(self, current, root));
        }
        query
    }
}

fn descend<'b>(segment: &PathSegment, current: &'b Value, root: &'b Value) -> Vec<&'b Value> {
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

#[derive(Debug, PartialEq, Clone)]
pub enum Segment {
    LongHand(Vec<Selector>),
    DotName(String),
    Wildcard,
}

impl Segment {
    pub fn is_singular(&self) -> bool {
        match self {
            Segment::LongHand(selectors) => {
                if selectors.len() > 1 {
                    return false;
                }
                if let Some(s) = selectors.first() {
                    return s.is_singular();
                } else {
                    // if the selector list is empty, this shouldn't be a valid
                    // JSONPath, but at least, it would be selecting nothing, and
                    // that could be considered singular, i.e., None.
                    return true;
                }
            }
            Segment::DotName(_) => true,
            Segment::Wildcard => false,
        }
    }

    pub fn as_long_hand(&self) -> Option<&[Selector]> {
        match self {
            Segment::LongHand(v) => Some(v.as_slice()),
            _ => None,
        }
    }

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
}
