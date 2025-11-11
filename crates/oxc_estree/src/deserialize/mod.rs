//! ESTree to oxc AST deserialization utilities.
//!
//! Due to circular dependency constraints, the actual conversion functions that return
//! oxc_ast types are implemented in oxc_linter which has access to both crates.
//! This module provides the building blocks:
//! - Context tracking for identifier disambiguation
//! - Error types for conversion failures
//! - Identifier and literal type detection logic
//! - Pattern vs AssignmentTarget determination

mod context;
mod converter;
mod error;
mod identifier;
mod literals;
mod patterns;
mod types;

pub use context::ConversionContext;
pub use converter::EstreeConverter;
pub use error::{ConversionError, ConversionResult};
pub use identifier::{IdentifierKind, convert_identifier, get_identifier_span};
pub use literals::{
    LiteralKind, convert_literal, get_boolean_value, get_literal_span, get_numeric_value,
    get_string_value,
};
pub use patterns::{PatternTargetKind, determine_pattern_kind};
pub use types::{EstreeIdentifier, EstreeLiteral, EstreeNode, EstreeNodeType};
