use nom::character::complete::char;
use nom::combinator::all_consuming;
use nom::error::VerboseError;
use nom::{branch::alt, combinator::map, multi::many0, sequence::preceded, IResult};
use serde_json::Value;

use self::segment::{parse_segment, PathSegment};

pub mod primitive;
pub mod segment;
pub mod selector;

type PResult<'a, O> = IResult<&'a str, O, VerboseError<&'a str>>;

pub trait QueryValue {
    fn query_value<'b>(&self, current: &'b Value, root: &'b Value) -> Vec<&'b Value>;
}

/// Represents a JSONPath expression
#[derive(Debug, PartialEq, Clone, Default)]
pub struct Query {
    kind: PathKind,
    pub segments: Vec<PathSegment>,
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

impl QueryValue for Query {
    fn query_value<'b>(&self, current: &'b Value, root: &'b Value) -> Vec<&'b Value> {
        let mut query = match self.kind {
            PathKind::Root => vec![root],
            PathKind::Current => vec![current],
        };
        for segment in &self.segments {
            let mut new_query = Vec::new();
            for q in &query {
                new_query.append(&mut segment.query_value(q, root));
            }
            query = new_query;
        }
        query
    }
}

fn parse_path_segments(input: &str) -> PResult<Vec<PathSegment>> {
    many0(parse_segment)(input)
}

fn parse_root_path(input: &str) -> PResult<Query> {
    map(preceded(char('$'), parse_path_segments), |segments| Query {
        kind: PathKind::Root,
        segments,
    })(input)
}

fn parse_current_path(input: &str) -> PResult<Query> {
    map(preceded(char('@'), parse_path_segments), |segments| Query {
        kind: PathKind::Current,
        segments,
    })(input)
}

pub(self) fn parse_path(input: &str) -> PResult<Query> {
    alt((parse_root_path, parse_current_path))(input)
}

pub fn parse_path_main(input: &str) -> PResult<Query> {
    all_consuming(parse_root_path)(input)
}

#[cfg(test)]
mod tests {
    use crate::parser::{
        segment::Segment,
        selector::{Name, Selector},
        PathKind,
    };

    use super::{parse_path, parse_path_main};

    #[test]
    fn root_path() {
        {
            let (_, p) = parse_path("$").unwrap();
            assert!(matches!(p.kind, PathKind::Root));
        }
        {
            let (_, p) = parse_path("$.name").unwrap();
            assert_eq!(p.segments[0].segment.as_dot_name().unwrap(), "name");
        }
        {
            let (_, p) = parse_path("$.names['first_name']..*").unwrap();
            assert_eq!(p.segments[0].segment.as_dot_name().unwrap(), "names");
            let clh = p.segments[1].segment.as_long_hand().unwrap();
            assert!(matches!(&clh[0], Selector::Name(Name(s)) if s == "first_name"));
            assert!(matches!(p.segments[2].segment, Segment::Wildcard));
        }
    }

    #[test]
    fn current_path() {
        {
            let (_, p) = parse_path("@").unwrap();
            assert!(matches!(p.kind, PathKind::Current));
        }
    }

    #[test]
    fn no_tail() {
        assert!(parse_path_main("$.a['b']tail").is_err());
    }
}
