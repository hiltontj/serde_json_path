use std::ops::Deref;

use crate::parser::Error;

/// Error type for JSONPath query string parsing errors
#[derive(Debug, thiserror::Error)]
#[error("{err}")]
pub struct ParseError {
    err: Box<ErrorImpl>,
}

impl ParseError {
    /// Get the 1-indexed error position
    pub fn position(&self) -> usize {
        self.err.position
    }

    /// Get the error message
    pub fn message(&self) -> &str {
        &self.err.message
    }
}

#[derive(Debug, thiserror::Error)]
#[error("at position {position}, {message}")]
struct ErrorImpl {
    position: usize,
    message: Box<str>,
}

impl<I> From<(I, Error<I>)> for ParseError
where
    I: Deref<Target = str> + std::fmt::Debug,
{
    fn from((input, pe): (I, Error<I>)) -> Self {
        #[cfg(feature = "trace")]
        tracing::trace!(input = %input.to_string(), parser_error = ?pe);
        let position = pe.calculate_position(input);
        let message = pe.to_string().into();
        Self {
            err: Box::new(ErrorImpl { position, message }),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ParseError;
    #[cfg(feature = "trace")]
    use test_log::test;

    #[test]
    fn test_send() {
        fn assert_send<T: Send>() {}
        assert_send::<ParseError>();
    }

    #[test]
    fn test_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<ParseError>();
    }
}
