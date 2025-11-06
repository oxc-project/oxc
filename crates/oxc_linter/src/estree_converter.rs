//! ESTree to oxc AST conversion bridge.
//!
//! This module provides the conversion from ESTree AST (from custom parsers)
//! to oxc AST. It uses utilities from `oxc_estree::deserialize` but implements
//! the actual AST construction here since it has access to `oxc_allocator` and `oxc_ast`.

use oxc_allocator::Allocator;
use oxc_ast::ast::Program;
use oxc_estree::deserialize::{ConversionError, ConversionResult, EstreeConverter};
use oxc_span::Span;

/// Convert ESTree AST (from raw transfer buffer) to oxc AST Program.
///
/// This is the main entry point for converting an ESTree AST from a custom parser
/// to an oxc AST program. The ESTree AST is read from a raw transfer buffer.
///
/// # Arguments
///
/// * `buffer` - Raw transfer buffer containing ESTree AST
/// * `estree_offset` - Offset where ESTree data starts in the buffer
/// * `source_text` - Original source code (needed for span conversion)
/// * `allocator` - Arena allocator for AST node allocation
///
/// # Returns
///
/// Returns a `Program` allocated in the arena, or an error if conversion fails.
pub fn convert_estree_to_oxc_program<'a>(
    buffer: &[u8],
    estree_offset: u32,
    source_text: &'a str,
    allocator: &'a Allocator,
) -> ConversionResult<Program<'a>> {
    // TODO: Implement raw transfer buffer reading and conversion
    // This should:
    // 1. Read ESTree AST from buffer starting at estree_offset
    // 2. Use EstreeConverter utilities from oxc_estree
    // 3. Convert ESTree nodes to oxc AST nodes
    // 4. Allocate all nodes via allocator
    // 5. Return Program
    
    // For now, return error indicating not yet implemented
    Err(ConversionError::UnsupportedNodeType {
        node_type: "Raw transfer conversion not yet implemented".to_string(),
        span: (0, 0),
    })
}

/// Convert ESTree JSON (fallback) to oxc AST Program.
///
/// This is a fallback for platforms without raw transfer support.
/// Uses JSON deserialization and conversion.
pub fn convert_estree_json_to_oxc_program<'a>(
    estree_json: &str,
    source_text: &'a str,
    allocator: &'a Allocator,
) -> ConversionResult<Program<'a>> {
    // Parse JSON
    let estree: serde_json::Value = serde_json::from_str(estree_json)
        .map_err(|e| ConversionError::JsonParseError {
            message: format!("Failed to parse ESTree JSON: {}", e),
        })?;

    // Validate and convert
    let converter = EstreeConverter::new(source_text);
    converter.validate_program(&estree)?;

    // TODO: Implement full conversion
    // This should:
    // 1. Use EstreeConverter utilities
    // 2. Convert ESTree nodes to oxc AST nodes
    // 3. Allocate all nodes via allocator
    // 4. Return Program
    
    Err(ConversionError::UnsupportedNodeType {
        node_type: "JSON conversion not yet implemented".to_string(),
        span: (0, 0),
    })
}

/// Convert ESTree span (character offsets) to oxc span (byte offsets).
///
/// ESTree uses character offsets (for UTF-16 compatibility),
/// while oxc uses byte offsets.
pub fn convert_span(source_text: &str, start: usize, end: usize) -> Span {
    // Fast path for ASCII-only files
    if source_text.is_ascii() {
        return Span::new(
            start.min(source_text.len()) as u32,
            end.min(source_text.len()) as u32,
        );
    }

    // Slow path for UTF-8 files
    let start_byte = source_text
        .char_indices()
        .nth(start)
        .map(|(byte_offset, _)| byte_offset)
        .unwrap_or(source_text.len());
    let end_byte = source_text
        .char_indices()
        .nth(end)
        .map(|(byte_offset, _)| byte_offset)
        .unwrap_or(source_text.len());

    Span::new(start_byte as u32, end_byte as u32)
}

