//! This crate allows you to use JSONPath queries to extract nodelists from a [`serde_json::Value`].
//!
//! The crate intends to adhere to the [IETF JSONPath specification][jp_spec]. Check out the
//! specification to read more about JSONPath, and to find many examples of its usage.
//!
//! Please note that the specification has not yet been published as an RFC; therefore, this crate
//! may evolve as JSONPath becomes standardized. See [Unimplemented Features](#unimplemented-features)
//! for more details on which parts of the specification are not implemented by this crate.
//!
//! # Features
//!
//! This crate provides two key abstractions:
//!
//! * The [`JsonPath`] struct, which represents a parsed JSONPath query.
//! * The [`NodeList`] struct, which represents the result of a JSONPath query performed on a
//!   [`serde_json::Value`].
//!
//! In addition, the [`JsonPathExt`] trait is provided, which extends the [`serde_json::Value`]
//! type with the [`json_path`][JsonPathExt::json_path] method for performing JSONPath queries.
//!
//! # Usage
//!
//! ## Parsing
//!
//! JSONPath query strings can be parsed using the [`JsonPath`] type:
//!
//! ```rust
//! use serde_json_path::JsonPath;
//!
//! # fn main() -> Result<(), serde_json_path::Error> {
//! let path = JsonPath::parse("$.foo.bar")?;
//! # Ok(())
//! # }
//! ```
//!
//! You can then use the parsed JSONPath to query a [`serde_json::Value`]. Every JSONPath query
//! produces a [`NodeList`], which provides several accessor methods that you can use depending on
//! the nature of your query and its expected output.
//!
//! ## Querying for single nodes
//!
//! For queries that are expected to return a single node, use either the
//! [`exactly_one`][NodeList::exactly_one] or the [`at_most_one`][NodeList::at_most_one] method.
//! For more lenient single node access, use the [`first`][NodeList::first],
//! [`last`][NodeList::last], or [`get`][NodeList::get] methods.
//!
//! ```rust
//! use serde_json::json;
//! # use serde_json_path::JsonPath;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let value = json!({ "foo": { "bar": ["baz", 42] } });
//! let path = JsonPath::parse("$.foo.bar[0]")?;
//! let node = path.query(&value).exactly_one()?;
//! assert_eq!(node, "baz");
//! # Ok(())
//! # }
//! ```
//!
//! JSONPath allows access via reverse indices:
//!
//! ```rust
//! # use serde_json::json;
//! # use serde_json_path::JsonPath;
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let value = json!([1, 2, 3, 4, 5]);
//! let path = JsonPath::parse("$[-1]")?;
//! let node = path.query(&value).at_most_one()?;
//! assert_eq!(node, Some(&json!(5)));
//! # Ok(())
//! # }
//! ```
//!
//! Keep in mind, that for simple queries, the [`serde_json::Value::pointer`] method may suffice.
//!
//! ## Querying for multiple nodes
//!
//! For queries that are expected to return zero or many nodes, use the [`all`][NodeList::all]
//! method. There are several [selectors][jp_selectors] in JSONPath whose combination can produce
//! useful and powerful queries.
//!
//! #### Wildcards (`*`)
//!
//! Wildcards select everything under a current node. They work on both arrays, by selecting all
//! array elements, and on objects, by selecting all object key values:
//!
//! ```rust
//! # use serde_json::json;
//! # use serde_json_path::JsonPath;
//! # fn main() -> Result<(), serde_json_path::Error> {
//! let value = json!({ "foo": { "bar": ["baz", "bop"] } });
//! let path = JsonPath::parse("$.foo.bar[*]")?;
//! let nodes = path.query(&value).all();
//! assert_eq!(nodes, vec!["baz", "bop"]);
//! # Ok(())
//! # }
//! ```
//!
//! #### Slice selectors (`start:end:step`)
//!
//! Extract slices from JSON arrays using optional `start`, `end`, and `step` values. Reverse
//! indices can be used for `start` and `end`, and a negative `step` can be used to traverse
//! the array in reverse order.
//!
//! ```rust
//! # use serde_json::json;
//! # use serde_json_path::JsonPath;
//! # fn main() -> Result<(), serde_json_path::Error> {
//! let value = json!({ "foo": ["bar", "baz", "bop"] });
//! let path = JsonPath::parse("$['foo'][1:]")?;
//! let nodes = path.query(&value).all();
//! assert_eq!(nodes, vec!["baz", "bop"]);
//! # Ok(())
//! # }
//! ```
//!
//! #### Filter expressions (`?`)
//!
//! Filter selectors allow you to perform comparisons and check for existence of nodes. You can
//! combine these checks using the boolean `&&` and `||` operators and group using parentheses.
//! The current node (`@`) operator allows you to apply the filtering logic based on the current
//! node being filtered:
//!
//! ```rust
//! # use serde_json::json;
//! # use serde_json_path::JsonPath;
//! # fn main() -> Result<(), serde_json_path::Error> {
//! let value = json!({ "foo": [1, 2, 3, 4, 5] });
//! let path = JsonPath::parse("$.foo[?@ > 2 && @ < 5]")?;
//! let nodes = path.query(&value).all();
//! assert_eq!(nodes, vec![3, 4]);
//! # Ok(())
//! # }
//! ```
//!
//! You can form relative paths on the current node, as well as absolute paths on the root (`$`)
//! node when writing filters:
//!
//! ```rust
//! # use serde_json::json;
//! # use serde_json_path::JsonPath;
//! # fn main() -> Result<(), serde_json_path::Error> {
//! let value = json!([
//!     { "title": "Great Expectations", "price": 10 },
//!     { "title": "Tale of Two Cities", "price": 8 },
//!     { "title": "David Copperfield", "price": 17 }
//! ]);
//! let path = JsonPath::parse("$[?@.price > $[0].price].title")?;
//! let nodes = path.query(&value).all();
//! assert_eq!(nodes, vec!["David Copperfield"]);
//! # Ok(())
//! # }
//! ```
//! #### Recursive descent (`..`)
//!
//! ```rust
//! # use serde_json::json;
//! # use serde_json_path::JsonPath;
//! # fn main() -> Result<(), serde_json_path::Error> {
//! let value = json!({
//!     "foo": {
//!         "bar": {
//!             "baz": 1
//!         },
//!         "baz": 2
//!     }
//! });
//! let path = JsonPath::parse("$.foo..baz")?;
//! let nodes = path.query(&value).all();
//! assert_eq!(nodes, vec![2, 1]);
//! # Ok(())
//! # }
//! ```
//!
//! As seen there, one of the useful features of JSONPath is its ability to extract deeply nested values.
//! Here is another example:
//!
//! ```rust
//! # use serde_json::json;
//! # use serde_json_path::JsonPath;
//! # fn main() -> Result<(), serde_json_path::Error> {
//! let value = json!({
//!     "foo": [
//!         { "bar": 1 },
//!         { "bar": 2 },
//!         { "bar": 3 }
//!     ]
//! });
//! let path = JsonPath::parse("$.foo.*.bar")?;
//! let nodes = path.query(&value).all();
//! assert_eq!(nodes, vec![1, 2, 3]);
//! # Ok(())
//! # }
//! ```
//!
//! See the [integration tests][tests] in the repository for more examples based on those found in
//! the JSONPath specification.
//!
//! # Unimplemented Features
//!
//! Parts of the JSONPath specification that are not implemented by this crate are listed here.
//!
//! * [Function Extensions][func_ext]: this is a planned feature for the crate, but has not yet
//!   been implemented.
//! * [Normalized Paths][norm_path]: this is not a planned feature for the crate.
//!
//! [tests]: https://github.com/hiltontj/serde_json_path/blob/main/tests/spec_examples.rs
//! [jp_spec]: https://www.ietf.org/archive/id/draft-ietf-jsonpath-base-10.html
//! [jp_selectors]: https://www.ietf.org/archive/id/draft-ietf-jsonpath-base-10.html#name-selectors-2
//! [func_ext]: https://www.ietf.org/archive/id/draft-ietf-jsonpath-base-10.html#name-function-extensions-2
//! [norm_path]: https://www.ietf.org/archive/id/draft-ietf-jsonpath-base-10.html#name-normalized-paths
mod error;
mod ext;
mod node;
mod parser;
mod path;

pub use parser::selector::function::{Evaluator, Function, JsonPathType};

pub use error::Error;
pub use ext::JsonPathExt;
pub use node::{AtMostOneError, ExactlyOneError, NodeList};
pub use path::JsonPath;

pub use serde_json_path_macros::function;
