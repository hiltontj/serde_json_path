use std::ops::Deref;

use nom::error::{convert_error, VerboseError};

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
