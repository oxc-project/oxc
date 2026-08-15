//! Shared pure helpers reused across language formatters.
//!
//! These implement formatting behavior common to multiple language formatters.
//! Our output currently targets Prettier compatibility,
//! but the domain is defined by what it is (a pure, language-agnostic behavior), not by Prettier.
//!
//! Membership is decided by three gates, NOT by "is it shared".
//! Every item must pass all:
//! 1. pure functions only (no option/config types),
//! 2. language differences arrive as explicit parameters, never hidden defaults or baked-in language rules,
//! 3. nothing here is re-aliased as a language's public config type.
//!
//! Import discipline: files here import only `std` + `cow_utils`,
//! never `oxc_*` crates or engine IR types via `crate::`.

mod bom;
mod front_matter;
mod gap;
mod number;
pub mod string;
mod suppression;

pub use bom::split_bom;
pub use front_matter::{FrontMatter, blank_front_matter, parse_front_matter};
pub use gap::{Gap, classify_gap};
pub use number::{format_trimmed_number, is_simple_number};
pub use string::normalize_string;
pub use suppression::is_suppression_marker;
