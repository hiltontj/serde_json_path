//! Representation of itegers in the JSONPath specification
//!
//! The JSONPath specification defines some rules for integers used in query strings (see [here][spec]).
//!
//! [spec]: https://www.rfc-editor.org/rfc/rfc9535.html#name-overview

use std::{
    num::{ParseIntError, TryFromIntError},
    str::FromStr,
};

/// An integer for internet JSON ([RFC7493][ijson])
///
/// The value must be within the range [-(2<sup>53</sup>)+1, (2<sup>53</sup>)-1]).
///
/// [ijson]: https://www.rfc-editor.org/rfc/rfc7493#section-2.2
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Integer(i64);

/// The maximum allowed value, 2^53 - 1
const MAX: i64 = 9_007_199_254_740_992 - 1;
/// The minimum allowed value (-2^53) + 1
const MIN: i64 = -9_007_199_254_740_992 + 1;

impl Integer {
    fn try_new(value: i64) -> Result<Self, IntegerError> {
        if value < MIN || value > MAX {
            return Err(IntegerError::OutOfBounds);
        } else {
            return Ok(Self(value));
        }
    }

    fn check(&self) -> bool {
        self.0 >= MIN && self.0 <= MAX
    }

    /// Get an [`Integer`] from an `i64`
    ///
    /// This will produce `None` if the inputted value is out of the valid range
    /// [-(2<sup>53</sup>)+1, (2<sup>53</sup>)-1]).
    pub fn from_i64_opt(value: i64) -> Option<Self> {
        Self::try_new(value).ok()
    }

    /// Take the absolute value, producing `None` if the resulting value is outside
    /// the valid range [-(2<sup>53</sup>)+1, (2<sup>53</sup>)-1]).
    pub fn checked_abs(mut self) -> Option<Self> {
        self.0 = self.0.checked_abs()?;
        self.check().then_some(self)
    }

    /// Add the two values, producing `None` if the resulting value is outside the
    /// valid range [-(2<sup>53</sup>)+1, (2<sup>53</sup>)-1]).
    pub fn checked_add(mut self, rhs: Self) -> Option<Self> {
        self.0 = self.0.checked_add(rhs.0)?;
        self.check().then_some(self)
    }

    /// Subtract the `rhs` from `self`, producing `None` if the resulting value is
    /// outside the valid range [-(2<sup>53</sup>)+1, (2<sup>53</sup>)-1]).
    pub fn checked_sub(mut self, rhs: Self) -> Option<Self> {
        self.0 = self.0.checked_sub(rhs.0)?;
        self.check().then_some(self)
    }

    /// Multiply the two values, producing `None` if the resulting value is outside
    /// the valid range [-(2<sup>53</sup>)+1, (2<sup>53</sup>)-1]).
    pub fn checked_mul(mut self, rhs: Self) -> Option<Self> {
        self.0 = self.0.checked_mul(rhs.0)?;
        self.check().then_some(self)
    }
}

impl Default for Integer {
    fn default() -> Self {
        Self(0)
    }
}

impl TryFrom<i64> for Integer {
    type Error = IntegerError;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}

impl TryFrom<usize> for Integer {
    type Error = IntegerError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        i64::try_from(value)
            .map_err(|_| IntegerError::OutOfBounds)
            .and_then(Self::try_new)
    }
}

impl FromStr for Integer {
    type Err = IntegerError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<i64>().map_err(Into::into).and_then(Self::try_new)
    }
}

impl std::fmt::Display for Integer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<Integer> for usize {
    type Error = TryFromIntError;

    fn try_from(value: Integer) -> Result<Self, Self::Error> {
        Self::try_from(value.0)
    }
}

impl PartialEq<i64> for Integer {
    fn eq(&self, other: &i64) -> bool {
        self.0.eq(other)
    }
}

impl PartialOrd<i64> for Integer {
    fn partial_cmp(&self, other: &i64) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(other)
    }
}

/// An error for the [`Integer`] type
#[derive(Debug, thiserror::Error)]
pub enum IntegerError {
    /// The provided value was outside the valid range [-(2**53)+1, (2**53)-1])
    #[error("the provided integer was outside the valid range, see https://www.rfc-editor.org/rfc/rfc9535.html#section-2.1-4.1")]
    OutOfBounds,
    /// Integer parsing error
    #[error(transparent)]
    Parse(#[from] ParseIntError),
}
