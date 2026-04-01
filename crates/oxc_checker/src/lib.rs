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
mod inference;
mod instantiation;
mod keyof;
mod mapped_types;
mod type_display;
mod type_from_type_node;

pub use checker::Checker;
pub use global_types::{allocate_intrinsics, find_lib_source};
// Re-export from oxc_checker_host so downstream crates use the same trait/struct
pub use oxc_checker_host::{CheckerHost, IntrinsicIds};

#[cfg(test)]
mod tests;
