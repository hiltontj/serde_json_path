use nom::{error::ParseError, IResult, Parser};

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
