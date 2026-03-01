//! Functions to prepare tokens for sending to JS side.
//!
//! Provides 2 different implementations:
//!
//! 1. Serialize tokens to JSON (`json.rs`).
//! 2. Update `Kind` of tokens in place so they can be deserialized on JS side (`raw_transfer.rs`).
//!
//! Both implementations share the same logic by utilizing the `Context` trait and AST visitor `Visitor`.
//! `Context` trait is implemented by `JsonContext` and `RawContext`.
//! The implementations only differ in how they process tokens sent to them by the visitor.
//!
//! Both implementations also convert UTF-8 spans to UTF-16.

mod context;
mod json;
mod jsx_state;
mod options;
mod raw_transfer;
mod token_type;
mod u32_string;
mod visitor;

pub use json::{to_estree_tokens_json, to_estree_tokens_pretty_json};
pub use jsx_state::{JSXState, JSXStateJS, JSXStateTS};
pub use options::{
    ESTreeTokenConfig, ESTreeTokenOptions, ESTreeTokenOptionsJS, ESTreeTokenOptionsTS,
};
pub use raw_transfer::update_tokens;
