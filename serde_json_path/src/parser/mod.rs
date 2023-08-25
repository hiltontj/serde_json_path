use std::fmt::Display;
use std::ops::Deref;

use nom::character::complete::char;
use nom::combinator::all_consuming;
use nom::error::{ContextError, ErrorKind, FromExternalError, ParseError};
use nom::Offset;
use nom::{branch::alt, combinator::map, multi::many0, sequence::preceded, IResult};
use serde_json_path_core::spec::query::{Query, QueryKind};
use serde_json_path_core::spec::segment::QuerySegment;

use self::segment::parse_segment;

pub(crate) mod primitive;
pub(crate) mod segment;
pub(crate) mod selector;
pub(crate) mod utils;

type PResult<'a, O> = IResult<&'a str, O, Error<&'a str>>;

#[derive(Debug, PartialEq)]
pub(crate) struct Error<I> {
    pub(crate) errors: Vec<ParserErrorInner<I>>,
}

impl<I> Error<I>
where
    I: Deref<Target = str>,
{
    pub(crate) fn calculate_position(&self, input: I) -> usize {
        self.errors
            .first()
            .map(|e| input.offset(&e.input))
            .unwrap_or(0)
    }
}

impl<I> std::fmt::Display for Error<I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(e) = self.errors.first() {
            if let Some(ctx) = e.context {
                write!(f, "in {ctx}, ")?;
            }
            write!(f, "{err}", err = e.kind)
        } else {
            write!(f, "empty error")
        }
    }
}

impl<I: std::fmt::Debug + std::fmt::Display> std::error::Error for Error<I> {}

#[derive(Debug, PartialEq)]
pub(crate) struct ParserErrorInner<I> {
    input: I,
    kind: ParserErrorKind,
    context: Option<&'static str>,
}

#[derive(Debug, PartialEq, thiserror::Error)]
pub(crate) enum ParserErrorKind {
    #[error("{0}")]
    Message(Box<str>),
    #[error("parser error")]
    Nom(ErrorKind),
}

impl<I> ParseError<I> for Error<I> {
    fn from_error_kind(input: I, kind: ErrorKind) -> Self {
        Self {
            errors: vec![ParserErrorInner {
                input,
                kind: ParserErrorKind::Nom(kind),
                context: None,
            }],
        }
    }

    fn append(input: I, kind: ErrorKind, mut other: Self) -> Self {
        other.errors.push(ParserErrorInner {
            input,
            kind: ParserErrorKind::Nom(kind),
            context: None,
        });
        other
    }
}

impl<I> ContextError<I> for Error<I> {
    fn add_context(_input: I, ctx: &'static str, mut other: Self) -> Self {
        if let Some(e) = other.errors.first_mut() {
            e.context = Some(ctx);
        };
        other
    }
}

impl<I, E> FromExternalError<I, E> for Error<I>
where
    E: std::error::Error + Display,
{
    fn from_external_error(input: I, _kind: ErrorKind, e: E) -> Self {
        Self {
            errors: vec![ParserErrorInner {
                input,
                kind: ParserErrorKind::Message(e.to_string().into()),
                context: None,
            }],
        }
    }
}

pub(crate) trait FromInternalError<I, E> {
    fn from_internal_error(input: I, e: E) -> Self;
}

impl<I, E> FromInternalError<I, E> for Error<I>
where
    E: std::error::Error + Display,
{
    fn from_internal_error(input: I, e: E) -> Self {
        Self {
            errors: vec![ParserErrorInner {
                input,
                kind: ParserErrorKind::Message(e.to_string().into()),
                context: None,
            }],
        }
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
fn parse_query(input: &str) -> PResult<Query> {
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
