//! This crate allows you to use JSONPath queries to extract nodelists from a [`serde_json::Value`].
//!
//! The crate intends to adhere to the [IETF JSONPath specification][ietf-spec]. Check out the
//! specification to read more about JSONPath, and to find many examples of its usage.
//!
//! [ietf-spec]: https://www.ietf.org/archive/id/draft-ietf-jsonpath-base-20.html
//!
//! Please note that the specification has not yet been published as an RFC; therefore, this crate
//! may evolve as JSONPath becomes standardized.
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
//! Finally, the [`#[function]`][function] attribute macro allows you to extend your JSONPath
//! queries to use custom functions.
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
//! # fn main() -> Result<(), serde_json_path::ParseError> {
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
//! method. There are several [selectors][ietf-selectors] in JSONPath whose combination can produce
//! useful and powerful queries.
//!
//! [ietf-selectors]: https://www.ietf.org/archive/id/draft-ietf-jsonpath-base-20.html#name-selectors-2
//!
//! #### Wildcards (`*`)
//!
//! Wildcards select everything under a current node. They work on both arrays, by selecting all
//! array elements, and on objects, by selecting all object key values:
//!
//! ```rust
//! # use serde_json::json;
//! # use serde_json_path::JsonPath;
//! # fn main() -> Result<(), serde_json_path::ParseError> {
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
//! the array in reverse order. Consider the following JSON object, and subsequent examples:
//! ```rust
//! # use serde_json::json;
//! # use serde_json_path::JsonPath;
//! # fn main() -> Result<(), serde_json_path::ParseError> {
//! let value = json!({ "foo": [1, 2, 3, 4, 5] });
//! # Ok(())
//! # }
//! ```
//! `start`, `end`, and `step` are all optional:
//! ```rust
//! # use serde_json::json;
//! # use serde_json_path::JsonPath;
//! # fn main() -> Result<(), serde_json_path::ParseError> {
//! # let value = json!({ "foo": [1, 2, 3, 4, 5] });
//! let path = JsonPath::parse("$.foo[:]")?;
//! let nodes = path.query(&value).all();
//! assert_eq!(nodes, vec![1, 2, 3, 4, 5]);
//! # Ok(())
//! # }
//! ```
//!
//! Omitting `end` will go to end of slice:
//! ```rust
//! # use serde_json::json;
//! # use serde_json_path::JsonPath;
//! # fn main() -> Result<(), serde_json_path::ParseError> {
//! # let value = json!({ "foo": [1, 2, 3, 4, 5] });
//! let path = JsonPath::parse("$.foo[2:]")?;
//! let nodes = path.query(&value).all();
//! assert_eq!(nodes, vec![3, 4, 5]);
//! # Ok(())
//! # }
//! ```
//!
//! Omitting `start` will start from beginning of slice:
//! ```rust
//! # use serde_json::json;
//! # use serde_json_path::JsonPath;
//! # fn main() -> Result<(), serde_json_path::ParseError> {
//! # let value = json!({ "foo": [1, 2, 3, 4, 5] });
//! let path = JsonPath::parse("$.foo[:2]")?;
//! let nodes = path.query(&value).all();
//! assert_eq!(nodes, vec![1, 2]);
//! # Ok(())
//! # }
//! ```
//!
//! You can specify the `step` size:
//! ```rust
//! # use serde_json::json;
//! # use serde_json_path::JsonPath;
//! # fn main() -> Result<(), serde_json_path::ParseError> {
//! # let value = json!({ "foo": [1, 2, 3, 4, 5] });
//! let path = JsonPath::parse("$.foo[::2]")?;
//! let nodes = path.query(&value).all();
//! assert_eq!(nodes, vec![1, 3, 5]);
//! # Ok(())
//! # }
//! ```
//!
//! Or use a negative `step` to go in reverse:
//! ```rust
//! # use serde_json::json;
//! # use serde_json_path::JsonPath;
//! # fn main() -> Result<(), serde_json_path::ParseError> {
//! # let value = json!({ "foo": [1, 2, 3, 4, 5] });
//! let path = JsonPath::parse("$.foo[::-1]")?;
//! let nodes = path.query(&value).all();
//! assert_eq!(nodes, vec![5, 4, 3, 2, 1]);
//! # Ok(())
//! # }
//! ```
//!
//! Finally, reverse indices can be used for `start` or `end`:
//! ```rust
//! # use serde_json::json;
//! # use serde_json_path::JsonPath;
//! # fn main() -> Result<(), serde_json_path::ParseError> {
//! # let value = json!({ "foo": [1, 2, 3, 4, 5] });
//! let path = JsonPath::parse("$.foo[-2:]")?;
//! let nodes = path.query(&value).all();
//! assert_eq!(nodes, vec![4, 5]);
//! # Ok(())
//! # }
//! ```
//!
//! #### Filter expressions (`?`)
//!
//! [Filter selectors][ietf-filter-selectors] allow you to use logical expressions to evaluate which
//! members in a JSON object or array will be selected. You can use the boolean `&&` and `||`
//! operators as well as parentheses to group logical expressions in your filters. The current node
//! (`@`) operator allows you to utilize the node being filtered in your filter logic:
//!
//! [ietf-filter-selectors]: https://www.ietf.org/archive/id/draft-ietf-jsonpath-base-20.html#name-filter-selector
//!
//! ```rust
//! # use serde_json::json;
//! # use serde_json_path::JsonPath;
//! # fn main() -> Result<(), serde_json_path::ParseError> {
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
//! # fn main() -> Result<(), serde_json_path::ParseError> {
//! let value = json!({
//!     "threshold": 40,
//!     "readings": [
//!         { "val": 35, "msg": "foo" },
//!         { "val": 40, "msg": "bar" },
//!         { "val": 42, "msg": "biz" },
//!         { "val": 48, "msg": "bop" },
//!     ]
//! });
//! let path = JsonPath::parse("$.readings[? @.val > $.threshold ].msg")?;
//! let nodes = path.query(&value).all();
//! assert_eq!(nodes, vec!["biz", "bop"]);
//! # Ok(())
//! # }
//! ```
//!
//! Filters also allow you to make use of [functions] in your queries:
//!
//! ```rust
//! # use serde_json::json;
//! # use serde_json_path::JsonPath;
//! # fn main() -> Result<(), serde_json_path::ParseError> {
//! let value = json!([
//!     "a short string",
//!     "a longer string",
//!     "an unnecessarily long string",
//! ]);
//! let path = JsonPath::parse("$[? length(@) < 20 ]")?;
//! let nodes = path.query(&value).all();
//! assert_eq!(nodes, vec!["a short string", "a longer string"]);
//! # Ok(())
//! # }
//! ```
//!
//! #### Descendant Operator (`..`)
//!
//! JSONPath query segments following a descendant operator (`..`) will visit the input node and each of its [descendants][ietf-descendants-def].
//!
//! [ietf-descendants-def]: https://www.ietf.org/archive/id/draft-ietf-jsonpath-base-20.html#section-1.1-6.28.1
//!
//! ```rust
//! # use serde_json::json;
//! # use serde_json_path::JsonPath;
//! # fn main() -> Result<(), serde_json_path::ParseError> {
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

#![warn(
    clippy::all,
    clippy::dbg_macro,
    clippy::todo,
    clippy::empty_enum,
    clippy::enum_glob_use,
    clippy::mem_forget,
    clippy::unused_self,
    clippy::filter_map_next,
    clippy::needless_continue,
    clippy::needless_borrow,
    clippy::match_wildcard_for_single_variants,
    clippy::if_let_mutex,
    clippy::mismatched_target_os,
    clippy::await_holding_lock,
    clippy::match_on_vec_items,
    clippy::imprecise_flops,
    clippy::suboptimal_flops,
    clippy::lossy_float_literal,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::fn_params_excessive_bools,
    clippy::exit,
    clippy::inefficient_to_string,
    clippy::linkedlist,
    clippy::macro_use_imports,
    clippy::option_option,
    clippy::verbose_file_reads,
    clippy::unnested_or_patterns,
    clippy::str_to_string,
    rust_2018_idioms,
    future_incompatible,
    nonstandard_style,
    missing_debug_implementations,
    missing_docs
)]
#![deny(unreachable_pub, private_in_public)]
#![allow(elided_lifetimes_in_paths, clippy::type_complexity)]
#![forbid(unsafe_code)]

mod error;
mod ext;
mod parser;
mod path;

#[doc(inline)]
pub use error::ParseError;
#[doc(inline)]
pub use ext::JsonPathExt;
#[doc(inline)]
pub use path::JsonPath;
#[doc(inline)]
pub use serde_json_path_core::node::{AtMostOneError, ExactlyOneError, NodeList};

pub use serde_json_path_core::spec::functions;

/// Register a function for use in JSONPath queries
///
/// The `#[function]` attribute macro allows you to define your own functions for use in your
/// JSONPath queries.
///
/// Note that `serde_json_path` already provides the functions defined in the IETF JSONPath
/// specification. You can find documentation for those in the [functions] module.
///
/// # Usage
///
/// ```
/// use serde_json::json;
/// use serde_json_path::JsonPath;
/// use serde_json_path::functions::{NodesType, ValueType};
///
/// // This will register a function called "first" that takes the first node from a nodelist and
/// // returns a reference to that node, if it contains any nodes, or nothing otherwise:
/// #[serde_json_path::function]
/// fn first(nodes: NodesType) -> ValueType {
///     match nodes.first() {
///         Some(n) => ValueType::Node(n),
///         None => ValueType::Nothing,
///     }
/// }
///
/// // You can now use this function in your JSONPath queries:
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let value = json!([
///     [1, 2, 3],
///     [4, 5, 6],
///     [7, 8, 9],
/// ]);
/// let path = JsonPath::parse("$[? first(@.*) == 4 ]")?;
/// let node = path.query(&value).exactly_one()?;
/// assert_eq!(node, &json!([4, 5, 6]));
/// # Ok(())
/// # }
/// ```
///
/// # Compile-time Checking
///
/// `#[function]` will ensure validity of your custom functions at compile time. The JSONPath type
/// system is outlined in more detail in the [functions] module, but consists of the three types:
/// [`NodesType`][functions::NodesType], [`ValueType`][functions::ValueType], and
/// [`LogicalType`][functions::LogicalType].
///
/// When defining your own JSONPath functions, you must use only these types as arguments to your
/// functions. In addition, your function must return one of these types. If you do not, the macro
/// will produce a helpful compiler error.
///
/// # Override function name using the `name` argument
///
/// By default, the function name available in your JSONPath queries will be that of the function
/// defined in your Rust code. However, you can specify the name that will be used from within your
/// JSONPath queries using the `name` argument:
///
/// ```
/// # use serde_json_path::JsonPath;
/// # use serde_json_path::functions::{ValueType, LogicalType};
/// #[serde_json_path::function(name = "match")]
/// fn match_func(node: ValueType, regexp: ValueType) -> LogicalType {
///     /* ... */
///     # LogicalType::False
/// }
/// // You can then call this in your queries using "match" (instead of "match_func"):
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let path = JsonPath::parse("$[? match(@.foo, 'ba[rz]')]")?;
/// # Ok(())
/// # }
/// ```
///
/// # How it works
///
/// In addition to type checking the function signature, the `#[function]` macro generates two
/// pieces of code: a validator and an evaluator.
///
/// The validator is used to ensure validity of function usage in your JSONPath query strings during
/// parsing, and will produce errors when calling, e.g., [`JsonPath::parse`], if you have misused a
/// function in your query.
///
/// The evaluator is used to evaluate the function when using [`JsonPath::query`] to evaluate a
/// JSONPath query against a [`serde_json::Value`].
///
/// The validator and evaluator are generated as lazily evaluated static closures and registered
/// into a single function registry using the [`inventory`] crate. This means you can define your
/// functions in any source file linked into your application.
///
/// [`inventory`]: https://docs.rs/inventory/latest/inventory/
#[doc(inline)]
#[cfg(feature = "functions")]
pub use serde_json_path_macros::function;
