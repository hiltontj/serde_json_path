use nom::{error::ParseError, IResult, Parser};

/// Prevent a `cut` parser from poisoning an alt branch
pub(crate) fn uncut<I, O, E: ParseError<I>, F: Parser<I, O, E>>(
    mut parser: F,
) -> impl FnMut(I) -> IResult<I, O, E> {
    move |input: I| match parser.parse(input) {
        Err(nom::Err::Failure(e)) => Err(nom::Err::Error(e)),
        rest => rest,
    }
}
