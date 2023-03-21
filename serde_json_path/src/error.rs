use std::ops::Deref;

use nom::error::{convert_error, VerboseError};

use crate::parser::ParserError;

/// A JSONPath error
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid JSONPath string:\n{message}")]
    InvalidJsonPathString { message: String },
}

impl<T> From<(T, VerboseError<T>)> for Error
where
    T: Deref<Target = str>,
{
    fn from((i, e): (T, VerboseError<T>)) -> Self {
        Self::InvalidJsonPathString {
            message: convert_error(i, e),
        }
    }
}

impl<I> From<(I, ParserError<I>)> for Error
where
    I: Deref<Target = str>,
{
    fn from((_, e): (I, ParserError<I>)) -> Self {
        Self::InvalidJsonPathString {
            message: e.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Error;

    #[test]
    fn test_send() {
        fn assert_send<T: Send>() {}
        assert_send::<Error>();
    }

    #[test]
    fn test_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<Error>();
    }
}
