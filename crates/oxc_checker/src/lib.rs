//! TypeScript type checker for oxc.
//!
//! This crate provides the `Checker`, which runs after semantic analysis
//! to resolve types and emit type-checking diagnostics.

mod checker;
mod expression_type;
mod type_display;
mod type_from_type_node;

pub use checker::Checker;

#[cfg(test)]
mod tests;
