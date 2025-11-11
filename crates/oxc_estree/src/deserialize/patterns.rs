//! Pattern and AssignmentTarget conversion for ESTree to oxc AST.

use super::context::ConversionContext;
use super::error::{ConversionError, ConversionResult};
use super::types::{EstreeNode, EstreeNodeType};

/// Determines whether an ESTree Pattern should be converted to a Pattern or AssignmentTarget.
///
/// ESTree uses `Pattern` for both binding and assignment contexts,
/// but oxc distinguishes `Pattern` (for bindings) from `AssignmentTarget` (for assignments).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatternTargetKind {
    Pattern,
    AssignmentTarget,
}

/// Determine if an ESTree Pattern should be converted to a Pattern or AssignmentTarget.
pub fn determine_pattern_kind(
    estree_node: &serde_json::Value,
    context: &ConversionContext,
) -> ConversionResult<PatternTargetKind> {
    // Check context to determine if this is a binding or assignment
    if context.is_assignment_context() {
        Ok(PatternTargetKind::AssignmentTarget)
    } else if context.is_binding_context() {
        Ok(PatternTargetKind::Pattern)
    } else {
        // Default: try to determine from node structure
        // If it's an Identifier in assignment context, it's an AssignmentTarget
        use super::types::EstreeNode;
        use serde_json::Value;
        if <Value as EstreeNode>::get_type(estree_node) == Some(EstreeNodeType::Identifier)
            && context.is_assignment_context()
        {
            Ok(PatternTargetKind::AssignmentTarget)
        } else {
            // Default to pattern (binding)
            Ok(PatternTargetKind::Pattern)
        }
    }
}
