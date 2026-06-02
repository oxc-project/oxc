//! Language-agnostic helpers shared by all formatter front-ends (JS/TS, JSON,
//! and future languages). Anything Prettier-spec-defined but context-free
//! belongs here.

mod comments;
mod number;
pub mod string;

pub use comments::is_suppression_marker;
pub use number::{NumberFormatOptions, format_trimmed_number, is_simple_number};
pub use string::{Quote, normalize_string};
