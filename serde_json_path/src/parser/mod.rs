use std::num::ParseIntError;
use std::string::FromUtf16Error;

use nom::character::complete::char;
use nom::combinator::all_consuming;
use nom::error::{ContextError, ErrorKind, FromExternalError, ParseError};
use nom::{branch::alt, combinator::map, multi::many0, sequence::preceded, IResult};
use serde_json_path_core::spec::functions::FunctionValidationError;
use serde_json_path_core::spec::query::{Query, QueryKind};
use serde_json_path_core::spec::segment::QuerySegment;
use serde_json_path_core::spec::selector::filter::NonSingularQueryError;

use self::segment::parse_segment;

pub(crate) mod primitive;
pub(crate) mod segment;
pub(crate) mod selector;

type PResult<'a, O> = IResult<&'a str, O, ParserError<&'a str>>;

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum ParserError<I> {
    #[error(transparent)]
    FnValidation(FunctionValidationError),
    #[error(transparent)]
    NonSingularQuery(NonSingularQueryError),
    #[error("{0}")]
    Mapped(String),
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

impl<I> FromExternalError<I, FunctionValidationError> for ParserError<I> {
    fn from_external_error(_input: I, _kind: ErrorKind, error: FunctionValidationError) -> Self {
        Self::FnValidation(error)
    }
}

impl<I> FromExternalError<I, NonSingularQueryError> for ParserError<I> {
    fn from_external_error(_input: I, _kind: ErrorKind, error: NonSingularQueryError) -> Self {
        Self::NonSingularQuery(error)
    }
}

impl<I> FromExternalError<I, ParseIntError> for ParserError<I> {
    fn from_external_error(_input: I, _kind: ErrorKind, e: ParseIntError) -> Self {
        Self::Mapped(e.to_string())
    }
}

impl<I> FromExternalError<I, serde_json::Error> for ParserError<I> {
    fn from_external_error(_input: I, _kind: ErrorKind, e: serde_json::Error) -> Self {
        Self::Mapped(e.to_string())
    }
}

impl<I> FromExternalError<I, FromUtf16Error> for ParserError<I> {
    fn from_external_error(_input: I, _kind: ErrorKind, e: FromUtf16Error) -> Self {
        Self::Mapped(e.to_string())
    }
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
fn parse_path_segments(input: &str) -> PResult<Vec<QuerySegment>> {
    many0(parse_segment)(input)
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
fn parse_root_query(input: &str) -> PResult<Query> {
    map(preceded(char('$'), parse_path_segments), |segments| Query {
        kind: QueryKind::Root,
        segments,
    })(input)
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
fn parse_current_query(input: &str) -> PResult<Query> {
    map(preceded(char('@'), parse_path_segments), |segments| Query {
        kind: QueryKind::Current,
        segments,
    })(input)
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
pub(self) fn parse_query(input: &str) -> PResult<Query> {
    alt((parse_root_query, parse_current_query))(input)
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
pub(crate) fn parse_query_main(input: &str) -> PResult<Query> {
    all_consuming(parse_root_query)(input)
}

#[cfg(test)]
mod tests {
    use serde_json_path_core::spec::{
        query::QueryKind,
        segment::Segment,
        selector::{name::Name, Selector},
    };

    use super::{parse_query, parse_query_main};

    #[test]
    fn root_path() {
        {
            let (_, p) = parse_query("$").unwrap();
            assert!(matches!(p.kind, QueryKind::Root));
        }
        {
            let (_, p) = parse_query("$.name").unwrap();
            assert_eq!(p.segments[0].segment.as_dot_name().unwrap(), "name");
        }
        {
            let (_, p) = parse_query("$.names['first_name']..*").unwrap();
            assert_eq!(p.segments[0].segment.as_dot_name().unwrap(), "names");
            let clh = p.segments[1].segment.as_long_hand().unwrap();
            assert!(matches!(&clh[0], Selector::Name(Name(s)) if s == "first_name"));
            assert!(matches!(p.segments[2].segment, Segment::Wildcard));
        }
    }

    #[test]
    fn current_path() {
        {
            let (_, p) = parse_query("@").unwrap();
            assert!(matches!(p.kind, QueryKind::Current));
        }
    }

    #[test]
    fn no_tail() {
        assert!(parse_query_main("$.a['b']tail").is_err());
    }
}
