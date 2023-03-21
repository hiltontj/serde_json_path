use nom::character::complete::char;
use nom::combinator::all_consuming;
use nom::error::{ContextError, ErrorKind, FromExternalError, ParseError};
use nom::{branch::alt, combinator::map, multi::many0, sequence::preceded, IResult};
use serde_json_path_core::spec::functions::FunctionValidationError;
use serde_json_path_core::spec::query::{PathKind, Query};
use serde_json_path_core::spec::segment::PathSegment;

use self::segment::parse_segment;

pub mod primitive;
pub mod segment;
pub mod selector;

type PResult<'a, O> = IResult<&'a str, O, ParserError<&'a str>>;

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum ParserError<I> {
    #[error(transparent)]
    FnValidation(FunctionValidationError),
    #[error("nom error")]
    Nom(I, ErrorKind),
}

impl<I> ParseError<I> for ParserError<I> {
    fn from_error_kind(input: I, kind: ErrorKind) -> Self {
        Self::Nom(input, kind)
    }

    fn append(_: I, _: ErrorKind, other: Self) -> Self {
        other
    }
}

impl<I> ContextError<I> for ParserError<I> {
    fn add_context(_input: I, _ctx: &'static str, other: Self) -> Self {
        other
    }
}

impl<I, E> FromExternalError<I, E> for ParserError<I> {
    fn from_external_error(input: I, kind: ErrorKind, _: E) -> Self {
        Self::Nom(input, kind)
    }
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
fn parse_path_segments(input: &str) -> PResult<Vec<PathSegment>> {
    many0(parse_segment)(input)
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
fn parse_root_path(input: &str) -> PResult<Query> {
    map(preceded(char('$'), parse_path_segments), |segments| Query {
        kind: PathKind::Root,
        segments,
    })(input)
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
fn parse_current_path(input: &str) -> PResult<Query> {
    map(preceded(char('@'), parse_path_segments), |segments| Query {
        kind: PathKind::Current,
        segments,
    })(input)
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
pub(self) fn parse_path(input: &str) -> PResult<Query> {
    alt((parse_root_path, parse_current_path))(input)
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
pub fn parse_path_main(input: &str) -> PResult<Query> {
    all_consuming(parse_root_path)(input)
}

#[cfg(test)]
mod tests {
    use serde_json_path_core::spec::{
        query::PathKind,
        segment::Segment,
        selector::{name::Name, Selector},
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
