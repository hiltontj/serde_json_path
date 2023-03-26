use std::ops::Deref;

use crate::parser::ParserError;

/// A JSONPath error
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// An invalid JSONPath query string
    #[error("invalid JSONPath string:\n{message}")]
    InvalidJsonPathString {
        /// The error message
        message: String,
    },
}

// TODO - use error kind for more detailed error message output?
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
