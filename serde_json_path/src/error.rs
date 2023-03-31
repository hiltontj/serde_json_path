use std::ops::Deref;

use crate::parser::ParserError;

/// Error type for the `serde_json_path` crate
#[derive(Debug, thiserror::Error)]
#[error("{err}")]
pub struct Error {
    err: Box<ErrorImpl>,
}

impl Error {
    /// Get the error position
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

impl<I> From<(I, ParserError<I>)> for Error
where
    I: Deref<Target = str> + std::fmt::Debug,
{
    fn from((input, pe): (I, ParserError<I>)) -> Self {
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
    use crate::{Error, JsonPath};
    #[cfg(feature = "trace")]
    use test_log::test;

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

    #[test]
    fn test_errors() {
        let e = JsonPath::parse("$['test]").unwrap_err();
        println!("{e}");
    }
}
