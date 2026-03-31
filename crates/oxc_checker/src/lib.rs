//! TypeScript type checker for oxc.
//!
//! This crate provides the `Checker`, which runs after semantic analysis
//! to resolve types and emit type-checking diagnostics.

#![warn(clippy::wildcard_enum_match_arm)]

mod assignability;
mod check_expression;
mod checker;
mod conditional_types;
mod declared_type;
mod expression_type;
mod flow;
mod flow_analysis;
mod flow_builder;
mod global_types;
mod host;
mod instantiation;
mod keyof;
mod mapped_types;
mod type_display;
mod type_from_type_node;

pub use checker::Checker;
pub use global_types::{GlobalTypes, allocate_intrinsics};
pub use host::CheckerHost;

#[cfg(test)]
mod tests;
