//! TypeScript type checker for oxc.
//!
//! This crate provides the `Checker`, which runs after semantic analysis
//! to resolve types and emit type-checking diagnostics.

mod assignability;
mod check_expression;
mod checker;
mod declared_type;
mod expression_type;
mod global_types;
mod type_display;
mod type_from_type_node;

pub use checker::Checker;
pub use global_types::GlobalTypes;

#[cfg(test)]
mod tests;
