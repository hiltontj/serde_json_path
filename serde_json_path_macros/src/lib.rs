//! Macros for [`serde_json_path`]
//!
//! [`serde_json_path`]: https://crates.io/crates/serde_json_path

pub use serde_json_path_macros_internal::function;
pub use serde_json_path_macros_internal::register;

#[doc(hidden)]
pub use ::inventory;

#[doc(hidden)]
pub use ::serde_json_path_core;
