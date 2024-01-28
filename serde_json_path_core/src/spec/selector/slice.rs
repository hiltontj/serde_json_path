//! Slice selectors for selecting array slices in JSONPath
use serde_json::Value;

use crate::spec::{path::NormalizedPath, query::Queryable};

/// A slice selector
#[derive(Debug, PartialEq, Eq, Default, Clone, Copy)]
pub struct Slice {
    /// The start of the slice
    ///
    /// This can be negative to start the slice from a position relative to the end of the array
    /// being sliced.
    pub start: Option<isize>,
    /// The end of the slice
    ///
    /// This can be negative to end the slice at a position relative to the end of the array being
    /// sliced.
    pub end: Option<isize>,
    /// The step slice for the slice
    ///
    /// This can be negative to step in reverse order.
    pub step: Option<isize>,
}

impl std::fmt::Display for Slice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(start) = self.start {
            write!(f, "{start}")?;
        }
        write!(f, ":")?;
        if let Some(end) = self.end {
            write!(f, "{end}")?;
        }
        write!(f, ":")?;
        if let Some(step) = self.step {
            write!(f, "{step}")?;
        }
        Ok(())
    }
}

#[doc(hidden)]
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

    #[inline]
    fn bounds_on_forward_slice(&self, len: isize) -> (isize, isize) {
        let start_default = self.start.unwrap_or(0);
        let end_default = self.end.unwrap_or(len);
        let start = normalize_slice_index(start_default, len)
            .unwrap_or(0)
            .max(0);
        let end = normalize_slice_index(end_default, len).unwrap_or(0).max(0);
        let lower = start.min(len);
        let upper = end.min(len);
        (lower, upper)
    }

    #[inline]
    fn bounds_on_reverse_slice(&self, len: isize) -> Option<(isize, isize)> {
        let start_default = self.start.or_else(|| len.checked_sub(1))?;
        let end_default = self
            .end
            .or_else(|| len.checked_mul(-1).and_then(|l| l.checked_sub(1)))?;
        let start = normalize_slice_index(start_default, len)
            .unwrap_or(0)
            .max(-1);
        let end = normalize_slice_index(end_default, len).unwrap_or(0).max(-1);
        let lower = end.min(len.checked_sub(1).unwrap_or(len));
        let upper = start.min(len.checked_sub(1).unwrap_or(len));
        Some((lower, upper))
    }
}

impl Queryable for Slice {
    #[cfg_attr(feature = "trace", tracing::instrument(name = "Query Slice", level = "trace", parent = None, ret))]
    fn query<'b>(&self, current: &'b Value, _root: &'b Value) -> Vec<&'b Value> {
        if let Some(list) = current.as_array() {
            let mut query = Vec::new();
            let step = self.step.unwrap_or(1);
            if step == 0 {
                return vec![];
            }
            let Ok(len) = isize::try_from(list.len()) else {
                return vec![];
            };
            if step > 0 {
                let (lower, upper) = self.bounds_on_forward_slice(len);
                let mut i = lower;
                while i < upper {
                    if let Some(v) = usize::try_from(i).ok().and_then(|i| list.get(i)) {
                        query.push(v);
                    }
                    i += step;
                }
            } else {
                let Some((lower, upper)) = self.bounds_on_reverse_slice(len) else {
                    return vec![];
                };
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

    fn query_located<'b>(
        &self,
        current: &'b Value,
        _root: &'b Value,
        parent: NormalizedPath<'b>,
    ) -> Vec<(NormalizedPath<'b>, &'b Value)> {
        if let Some(list) = current.as_array() {
            let mut result = Vec::new();
            let step = self.step.unwrap_or(1);
            if step == 0 {
                return vec![];
            }
            let Ok(len) = isize::try_from(list.len()) else {
                return vec![];
            };
            if step > 0 {
                let (lower, upper) = self.bounds_on_forward_slice(len);
                let mut i = lower;
                while i < upper {
                    if let Some((i, v)) = usize::try_from(i)
                        .ok()
                        .and_then(|i| list.get(i).map(|v| (i, v)))
                    {
                        result.push((parent.clone_and_push(i), v));
                    }
                    i += step;
                }
            } else {
                let Some((lower, upper)) = self.bounds_on_reverse_slice(len) else {
                    return vec![];
                };
                let mut i = upper;
                while lower < i {
                    if let Some((i, v)) = usize::try_from(i)
                        .ok()
                        .and_then(|i| list.get(i).map(|v| (i, v)))
                    {
                        result.push((parent.clone_and_push(i), v));
                    }
                    i += step;
                }
            }
            result
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
