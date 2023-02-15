use nom::bytes::complete::is_not;
use nom::character::complete::char;
use nom::combinator::verify;
use nom::{
    branch::alt,
    bytes::complete::take_while_m_n,
    combinator::{map, map_opt, map_res, value},
    multi::fold_many0,
    sequence::{delimited, preceded},
};

use crate::parser::PResult;

fn parse_unicode(input: &str) -> PResult<char> {
    let parse_hex = take_while_m_n(1, 6, |c: char| c.is_ascii_hexdigit());
    let parse_delimited_hex = preceded(char('u'), delimited(char('{'), parse_hex, char('}')));
    let parse_u32 = map_res(parse_delimited_hex, move |hex| u32::from_str_radix(hex, 16));
    map_opt(parse_u32, std::char::from_u32)(input)
}

#[derive(Copy, Clone)]
enum Quotes {
    Single,
    Double,
}

fn parse_escaped_quote(quoted_with: Quotes) -> impl Fn(&str) -> PResult<char> {
    move |input: &str| match quoted_with {
        Quotes::Single => value('\u{0027}', char('\''))(input),
        Quotes::Double => value('\u{0022}', char('"'))(input),
    }
}

fn parse_escaped_char(quoted_with: Quotes) -> impl Fn(&str) -> PResult<char> {
    move |input: &str| {
        preceded(
            char('\\'),
            alt((
                parse_unicode,
                value('\u{0008}', char('b')),
                value('\u{0009}', char('t')),
                value('\u{000A}', char('n')),
                value('\u{000C}', char('f')),
                value('\u{000D}', char('r')),
                value('\u{002F}', char('/')),
                value('\u{005C}', char('\\')),
                parse_escaped_quote(quoted_with),
            )),
        )(input)
    }
}

fn parse_literal(quoted_with: Quotes) -> impl Fn(&str) -> PResult<&str> {
    move |input: &str| {
        let not_quote_slash = match quoted_with {
            Quotes::Single => is_not("'\\"),
            Quotes::Double => is_not("\"\\"),
        };
        verify(not_quote_slash, |s: &str| !s.is_empty())(input)
    }
}

fn parse_fragment(quoted_with: Quotes) -> impl Fn(&str) -> PResult<StringFragment<'_>> {
    move |input: &str| {
        alt((
            map(parse_literal(quoted_with), StringFragment::Literal),
            map(parse_escaped_char(quoted_with), StringFragment::EscapedChar),
        ))(input)
    }
}

enum StringFragment<'a> {
    Literal(&'a str),
    EscapedChar(char),
}

fn parse_internal(quoted_with: Quotes) -> impl Fn(&str) -> PResult<String> {
    move |input: &str| {
        fold_many0(
            parse_fragment(quoted_with),
            String::new,
            |mut string, fragment| {
                match fragment {
                    StringFragment::Literal(s) => string.push_str(s),
                    StringFragment::EscapedChar(c) => string.push(c),
                }
                string
            },
        )(input)
    }
}

fn parse_single_quoted(input: &str) -> PResult<String> {
    delimited(char('\''), parse_internal(Quotes::Single), char('\''))(input)
}

fn parse_double_quoted(input: &str) -> PResult<String> {
    delimited(char('"'), parse_internal(Quotes::Double), char('"'))(input)
}

pub fn parse_string_literal(input: &str) -> PResult<String> {
    alt((parse_single_quoted, parse_double_quoted))(input)
}

#[cfg(test)]
mod tests {
    use super::parse_string_literal;

    #[test]
    fn valid_double_quoted_selectors() {
        assert_eq!(
            parse_string_literal("\"test\""),
            Ok(("", String::from("test")))
        );
        assert_eq!(
            parse_string_literal("\"test\n\""),
            Ok(("", String::from("test\n")))
        );
        assert_eq!(
            parse_string_literal("\"test\\ntest\""),
            Ok(("", String::from("test\ntest")))
        );
        assert_eq!(
            parse_string_literal("\"test\\\"\""),
            Ok(("", String::from("test\"")))
        );
        assert_eq!(
            parse_string_literal("\"tes't\""),
            Ok(("", String::from("tes't")))
        );
    }

    #[test]
    fn valid_single_quoted_selectors() {
        assert_eq!(
            parse_string_literal("'test'"),
            Ok(("", String::from("test")))
        );
        assert_eq!(
            parse_string_literal(r#"'te"st'"#),
            Ok(("", String::from("te\"st")))
        );
        assert_eq!(
            parse_string_literal(r#"'te\'st'"#),
            Ok(("", String::from("te'st")))
        );
    }
}
