use nom::{
    branch::alt,
    character::complete::{char, multispace0},
    combinator::{map, opt},
    sequence::{preceded, separated_pair, terminated},
};
use serde_json_path_core::spec::selector::slice::Slice;

use crate::parser::{primitive::int::parse_int, PResult};

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
fn parse_int_space_after(input: &str) -> PResult<isize> {
    terminated(parse_int, multispace0)(input)
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
fn parse_int_space_before(input: &str) -> PResult<isize> {
    preceded(multispace0, parse_int)(input)
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "trace", parent = None, ret, err))]
pub(crate) fn parse_array_slice(input: &str) -> PResult<Slice> {
    map(
        separated_pair(
            opt(parse_int_space_after),
            char(':'),
            preceded(
                multispace0,
                alt((
                    separated_pair(
                        opt(parse_int_space_after),
                        char(':'),
                        opt(parse_int_space_before),
                    ),
                    map(opt(parse_int_space_after), |i| (i, None)),
                )),
            ),
        ),
        |(start, (end, step))| Slice { start, end, step },
    )(input)
}

#[cfg(test)]
mod tests {
    use crate::parser::selector::slice::Slice;

    use super::parse_array_slice;

    #[test]
    fn valid_forward() {
        assert_eq!(
            parse_array_slice("1:5:1"),
            Ok(("", Slice::new().with_start(1).with_end(5).with_step(1)))
        );
        assert_eq!(
            parse_array_slice("1:10:3"),
            Ok(("", Slice::new().with_start(1).with_end(10).with_step(3)))
        );
        assert_eq!(
            parse_array_slice("1:5:-1"),
            Ok(("", Slice::new().with_start(1).with_end(5).with_step(-1)))
        );
        assert_eq!(
            parse_array_slice(":5:1"),
            Ok(("", Slice::new().with_end(5).with_step(1)))
        );
        assert_eq!(
            parse_array_slice("1::1"),
            Ok(("", Slice::new().with_start(1).with_step(1)))
        );
        assert_eq!(
            parse_array_slice("1:5"),
            Ok(("", Slice::new().with_start(1).with_end(5)))
        );
        assert_eq!(parse_array_slice("::"), Ok(("", Slice::new())));
    }

    #[test]
    fn optional_whitespace() {
        assert_eq!(
            parse_array_slice("1 :5:1"),
            Ok(("", Slice::new().with_start(1).with_end(5).with_step(1)))
        );
        assert_eq!(
            parse_array_slice("1: 5 :1"),
            Ok(("", Slice::new().with_start(1).with_end(5).with_step(1)))
        );
        assert_eq!(
            parse_array_slice("1: :1"),
            Ok(("", Slice::new().with_start(1).with_step(1)))
        );
        assert_eq!(
            parse_array_slice("1:5\n:1"),
            Ok(("", Slice::new().with_start(1).with_end(5).with_step(1)))
        );
        assert_eq!(
            parse_array_slice("1 : 5 :1"),
            Ok(("", Slice::new().with_start(1).with_end(5).with_step(1)))
        );
        assert_eq!(
            parse_array_slice("1:5: 1"),
            Ok(("", Slice::new().with_start(1).with_end(5).with_step(1)))
        );
    }
}
