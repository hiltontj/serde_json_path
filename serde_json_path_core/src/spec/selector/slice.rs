//! Slice selectors for selecting array slices in JSONPath
use serde_json::Value;

use crate::{
    node::LocatedNode,
    path::NormalizedPath,
    spec::{integer::Integer, query::Queryable},
};

/// A slice selector
#[derive(Debug, PartialEq, Eq, Default, Clone, Copy)]
pub struct Slice {
    /// The start of the slice
    ///
    /// This can be negative to start the slice from a position relative to the end of the array
    /// being sliced.
    pub start: Option<Integer>,
    /// The end of the slice
    ///
    /// This can be negative to end the slice at a position relative to the end of the array being
    /// sliced.
    pub end: Option<Integer>,
    /// The step slice for the slice
    ///
    /// This can be negative to step in reverse order.
    pub step: Option<Integer>,
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

    /// Set the slice `start`
    ///
    /// # Panics
    ///
    /// This will panic if the provided value is outside the range [-(2<sup>53</sup>) + 1, (2<sup>53</sup>) - 1].
    pub fn with_start(mut self, start: i64) -> Self {
        self.start = Some(Integer::from_i64_opt(start).expect("valid start"));
        self
    }

    /// Set the slice `end`
    ///
    /// # Panics
    ///
    /// This will panic if the provided value is outside the range [-(2<sup>53</sup>) + 1, (2<sup>53</sup>) - 1].
    pub fn with_end(mut self, end: i64) -> Self {
        self.end = Some(Integer::from_i64_opt(end).expect("valid end"));
        self
    }

    /// Set the slice `step`
    ///
    /// # Panics
    ///
    /// This will panic if the provided value is outside the range [-(2<sup>53</sup>) + 1, (2<sup>53</sup>) - 1].
    pub fn with_step(mut self, step: i64) -> Self {
        self.step = Some(Integer::from_i64_opt(step).expect("valid step"));
        self
    }

    #[inline]
    fn bounds_on_forward_slice(&self, len: Integer) -> (Integer, Integer) {
        let start_default = self.start.unwrap_or_default();
        let end_default = self.end.unwrap_or(len);
        let start = normalize_slice_index(start_default, len)
            .unwrap_or_default()
            .max(Integer::default());
        let end = normalize_slice_index(end_default, len)
            .unwrap_or_default()
            .max(Integer::default());
        let lower = start.min(len);
        let upper = end.min(len);
        (lower, upper)
    }

    #[inline]
    fn bounds_on_reverse_slice(&self, len: Integer) -> Option<(Integer, Integer)> {
        let start_default = self
            .start
            .or_else(|| Integer::from_i64_opt(1).and_then(|i| len.checked_sub(i)))?;
        let end_default = self.end.or_else(|| {
            let l = len.checked_mul(Integer::from_i64_opt(-1).unwrap())?;
            l.checked_sub(Integer::from_i64_opt(1).unwrap())
        })?;
        let start = normalize_slice_index(start_default, len)
            .unwrap_or_default()
            .max(Integer::from_i64_opt(-1).unwrap());
        let end = normalize_slice_index(end_default, len)
            .unwrap_or_default()
            .max(Integer::from_i64_opt(-1).unwrap());
        let lower = end.min(
            len.checked_sub(Integer::from_i64_opt(1).unwrap())
                .unwrap_or(len),
        );
        let upper = start.min(
            len.checked_sub(Integer::from_i64_opt(1).unwrap())
                .unwrap_or(len),
        );
        Some((lower, upper))
    }
}

impl Queryable for Slice {
    #[cfg_attr(feature = "trace", tracing::instrument(name = "Query Slice", level = "trace", parent = None, ret))]
    fn query<'b>(&self, current: &'b Value, _root: &'b Value) -> Vec<&'b Value> {
        if let Some(list) = current.as_array() {
            let mut query = Vec::new();
            let step = self.step.unwrap_or(Integer::from_i64_opt(1).unwrap());
            if step == 0 {
                return vec![];
            }
            let Ok(len) = Integer::try_from(list.len()) else {
                return vec![];
            };
            if step > 0 {
                let (lower, upper) = self.bounds_on_forward_slice(len);
                let mut i = lower;
                while i < upper {
                    if let Some(v) = usize::try_from(i).ok().and_then(|i| list.get(i)) {
                        query.push(v);
                    }
                    i = if let Some(i) = i.checked_add(step) {
                        i
                    } else {
                        break;
                    };
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
                    i = if let Some(i) = i.checked_add(step) {
                        i
                    } else {
                        break;
                    };
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
    ) -> Vec<LocatedNode<'b>> {
        if let Some(list) = current.as_array() {
            let mut result = Vec::new();
            let step = self.step.unwrap_or(Integer::from_i64_opt(1).unwrap());
            if step == 0 {
                return vec![];
            }
            let Ok(len) = Integer::try_from(list.len()) else {
                return vec![];
            };
            if step > 0 {
                let (lower, upper) = self.bounds_on_forward_slice(len);
                let mut i = lower;
                while i < upper {
                    if let Some((i, node)) = usize::try_from(i)
                        .ok()
                        .and_then(|i| list.get(i).map(|v| (i, v)))
                    {
                        result.push(LocatedNode {
                            loc: parent.clone_and_push(i),
                            node,
                        });
                    }
                    i = if let Some(i) = i.checked_add(step) {
                        i
                    } else {
                        break;
                    };
                }
            } else {
                let Some((lower, upper)) = self.bounds_on_reverse_slice(len) else {
                    return vec![];
                };
                let mut i = upper;
                while lower < i {
                    if let Some((i, node)) = usize::try_from(i)
                        .ok()
                        .and_then(|i| list.get(i).map(|v| (i, v)))
                    {
                        result.push(LocatedNode {
                            loc: parent.clone_and_push(i),
                            node,
                        });
                    }
                    i = if let Some(i) = i.checked_add(step) {
                        i
                    } else {
                        break;
                    };
                }
            }
            result
        } else {
            vec![]
        }
    }
}

fn normalize_slice_index(index: Integer, len: Integer) -> Option<Integer> {
    if index >= 0 {
        Some(index)
    } else {
        index.checked_abs().and_then(|i| len.checked_sub(i))
    }
}
