//! Core converter for ESTree to oxc AST conversion.
//!
//! Due to circular dependency constraints, the actual conversion functions that return
//! oxc_ast types are implemented in oxc_linter which has access to both crates.
//! This module provides foundational types and utilities.

use super::context::ConversionContext;
use super::error::{ConversionError, ConversionResult, Span};
use super::types::{EstreeNode, EstreeNodeType};

/// Main converter for ESTree to oxc AST.
///
/// This converter provides utilities for single-pass conversion from ESTree AST to oxc AST.
/// The actual AST construction happens in the calling code (oxc_linter) which has access to oxc_ast.
pub struct EstreeConverter<'a> {
    source_text: &'a str,
    context: ConversionContext,
}

impl<'a> EstreeConverter<'a> {
    /// Create a new converter.
    pub fn new(source_text: &'a str) -> Self {
        Self { source_text, context: ConversionContext::new() }
    }

    /// Validate that an ESTree node is a Program node.
    pub fn validate_program(&self, estree: &serde_json::Value) -> ConversionResult<()> {
        use super::types::EstreeNode;
        use serde_json::Value;
        let node_type = <Value as EstreeNode>::get_type(estree).ok_or_else(|| {
            ConversionError::MissingField {
                field: "type".to_string(),
                node_type: "unknown".to_string(),
                span: (0, 0),
            }
        })?;

        if !matches!(node_type, EstreeNodeType::Program) {
            return Err(ConversionError::UnsupportedNodeType {
                node_type: format!("{:?}", node_type),
                span: (0, 0),
            });
        }

        Ok(())
    }

    /// Convert ESTree character offsets to oxc byte offsets.
    ///
    /// ESTree uses character offsets (for UTF-16 compatibility),
    /// while oxc uses byte offsets. This function converts between them.
    fn char_offset_to_byte_offset(&self, char_offset: usize) -> usize {
        // Fast path for ASCII-only files
        if self.source_text.is_ascii() {
            return char_offset.min(self.source_text.len());
        }

        // Slow path for UTF-8 files
        self.source_text
            .char_indices()
            .nth(char_offset)
            .map(|(byte_offset, _)| byte_offset)
            .unwrap_or(self.source_text.len())
    }
}
