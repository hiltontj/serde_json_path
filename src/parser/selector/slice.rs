use nom::{
    branch::alt,
    character::complete::{char, multispace0},
    combinator::{map, opt},
    sequence::{preceded, separated_pair, terminated},
};
use serde_json::Value;

use crate::parser::{primitive::int::parse_int, PResult, QueryValue};

#[derive(Debug, PartialEq, Default)]
pub struct Slice {
    start: Option<isize>,
    end: Option<isize>,
    step: Option<isize>,
}

#[cfg(test)]
impl Slice {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_start(mut self, start: isize) -> Self {
        self.start = Some(start);
        self
    }

    pub fn with_end(mut self, end: isize) -> Self {
        self.end = Some(end);
        self
    }

    pub fn with_step(mut self, step: isize) -> Self {
        self.step = Some(step);
        self
    }
}

impl QueryValue for Slice {
    fn query_value<'b>(&self, current: &'b Value, _root: &'b Value) -> Vec<&'b Value> {
        if let Some(list) = current.as_array() {
            let mut query = Vec::new();
            let step = self.step.unwrap_or(1);
            if step == 0 {
                return vec![];
            }
            let len = if let Ok(l) = isize::try_from(list.len()) {
                l
            } else {
                return vec![];
            };
            if step > 0 {
                let start_default = self.start.unwrap_or(0);
                let end_default = self.end.unwrap_or(len);
                let start = normalize_slice_index(start_default, len)
                    .unwrap_or(0)
                    .max(0);
                let end = normalize_slice_index(end_default, len).unwrap_or(0).max(0);
                let lower = start.min(len);
                let upper = end.min(len);
                let mut i = lower;
                while i < upper {
                    if let Some(v) = usize::try_from(i).ok().and_then(|i| list.get(i)) {
                        query.push(v);
                    }
                    i += step;
                }
            } else {
                let start_default = self.start.unwrap_or(len - 1); // TODO - not checked sub
                let end_default = self.end.unwrap_or(-len - 1); // TODO - not checked sub
                let start = normalize_slice_index(start_default, len)
                    .unwrap_or(0)
                    .max(-1);
                let end = normalize_slice_index(end_default, len).unwrap_or(0).max(-1);
                let lower = end.min(len.checked_sub(1).unwrap_or(len));
                let upper = start.min(len.checked_sub(1).unwrap_or(len));
                let mut i = upper;
                while lower < i {
                    if let Some(v) = usize::try_from(i).ok().and_then(|i| list.get(i)) {
                        query.push(v);
                    }
                    i += step;
                }
            }
            query
        } else {
            vec![]
        }
    }
}

fn normalize_slice_index(index: isize, len: isize) -> Option<isize> {
    if index >= 0 {
        Some(index)
    } else {
        index.checked_abs().and_then(|i| len.checked_sub(i))
    }
}

fn parse_int_space_after(input: &str) -> PResult<isize> {
    terminated(parse_int, multispace0)(input)
}

fn parse_int_space_before(input: &str) -> PResult<isize> {
    preceded(multispace0, parse_int)(input)
}

/// Parse an Array Slice Selector
///
/// See [Section 2.5.4](https://www.ietf.org/archive/id/draft-ietf-jsonpath-base-09.html#name-array-slice-selector)
/// in the JSONPath standard.
pub fn parse_array_slice(input: &str) -> PResult<Slice> {
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
