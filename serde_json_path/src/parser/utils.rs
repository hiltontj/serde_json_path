use nom::{error::ParseError, IResult, Parser};

use super::FromInternalError;

/// Prevent upstream poisoning by [`nom::combinator::cut`]
///
/// It is common when using [`nom::branch::alt`] to terminate a branch, and the entire `alt` tree,
/// using the `cut` combinator. This has the unfortunate effect of "poisoning" upstream parsers by
/// propagating a nom `Failure` up the parser chain. Any upstream usage of, e.g., `alt`, would then
/// also terminate, and not attempt other branches.
///
/// Using `uncut`, you can wrap `alt` to prevent this poisoning.
///
/// # Example
///
/// ```ignore
/// fn unpoisoned_alt(input: &str) -> IResult<&str, &str> {
///     uncut(
///         alt((
///             pair(tag("foo"), cut(tag("bar"))),
///             tag("baz"),
///         ))
///     )(input)
/// }
/// ```
pub(crate) fn uncut<I, O, E: ParseError<I>, F: Parser<I, O, E>>(
    mut parser: F,
) -> impl FnMut(I) -> IResult<I, O, E> {
    move |input: I| match parser.parse(input) {
        Err(nom::Err::Failure(e)) => Err(nom::Err::Error(e)),
        rest => rest,
    }
}

#[allow(dead_code)]
/// Map a parser error into another error
pub(crate) fn map_err<I: Clone, O, E1, E2, F, G>(
    mut parser: F,
    mut f: G,
) -> impl FnMut(I) -> IResult<I, O, E1>
where
    F: nom::Parser<I, O, E1>,
    G: FnMut(E1) -> E2,
    E1: FromInternalError<I, E2>,
{
    move |input: I| {
        let i = input.clone();
        match parser.parse(i) {
            Ok((remainder, value)) => Ok((remainder, value)),
            Err(nom::Err::Error(e)) => Err(nom::Err::Error(E1::from_internal_error(input, f(e)))),
            Err(nom::Err::Failure(e)) => {
                Err(nom::Err::Failure(E1::from_internal_error(input, f(e))))
            }
            Err(nom::Err::Incomplete(_)) => unreachable!(),
        }
    }
}

/// Use the `cut` parser with a custom error wrapper
pub(crate) fn cut_with<I, O, E1, E2, F, G>(
    mut parser: F,
    mut f: G,
) -> impl FnMut(I) -> IResult<I, O, E1>
where
    I: Clone,
    F: nom::Parser<I, O, E1>,
    G: FnMut(E1) -> E2,
    E1: FromInternalError<I, E2>,
{
    move |input: I| {
        let i = input.clone();
        match parser.parse(i) {
            Ok((remainder, value)) => Ok((remainder, value)),
            Err(nom::Err::Error(e) | nom::Err::Failure(e)) => {
                Err(nom::Err::Failure(E1::from_internal_error(input, f(e))))
            }
            Err(nom::Err::Incomplete(_)) => unreachable!(),
        }
    }
}

#[allow(dead_code)]
/// Fail with an internal error
pub(crate) fn fail_with<I, O, E1, E2, G>(mut f: G) -> impl FnMut(I) -> IResult<I, O, E1>
where
    E1: FromInternalError<I, E2>,
    G: FnMut() -> E2,
{
    move |input: I| Err(nom::Err::Failure(E1::from_internal_error(input, f())))
}
