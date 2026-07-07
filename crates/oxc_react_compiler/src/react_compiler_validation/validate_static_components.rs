// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

//! Validates against components that are created dynamically and whose identity
//! is not guaranteed to be stable (which would cause the component to reset on
//! each re-render).
//!
//! Port of ValidateStaticComponents.ts.

use rustc_hash::FxHashMap;

use crate::react_compiler_diagnostics::{
    CompilerDiagnostic, CompilerDiagnosticDetail, CompilerError, ErrorCategory, Span,
};
use crate::react_compiler_hir::{HirFunction, IdentifierId, InstructionValue, JsxTag};

/// Validates that components used in JSX are not dynamically created during render.
///
/// Returns a CompilerError containing all diagnostics found (may be empty).
/// Called via `env.logErrors()` pattern in Pipeline.ts.
pub fn validate_static_components(func: &HirFunction) -> CompilerError {
    let mut error = CompilerError::new();
    let mut known_dynamic_components: FxHashMap<IdentifierId, Option<Span>> = FxHashMap::default();

    for (_block_id, block) in &func.body.blocks {
        // Process phis: propagate dynamic component knowledge through phi nodes
        'phis: for phi in &block.phis {
            for (_pred, operand) in &phi.operands {
                if let Some(span) = known_dynamic_components.get(&operand.identifier) {
                    known_dynamic_components.insert(phi.place.identifier, *span);
                    continue 'phis;
                }
            }
        }

        // Process instructions
        for &instr_id in &block.instructions {
            let instr = &func.instructions[instr_id.0 as usize];
            let lvalue_id = instr.lvalue.identifier;
            let value = &instr.value;

            match value {
                InstructionValue::FunctionExpression { span, .. }
                | InstructionValue::NewExpression { span, .. }
                | InstructionValue::MethodCall { span, .. }
                | InstructionValue::CallExpression { span, .. } => {
                    known_dynamic_components.insert(lvalue_id, *span);
                }
                InstructionValue::LoadLocal { place, .. } => {
                    if let Some(span) = known_dynamic_components.get(&place.identifier) {
                        known_dynamic_components.insert(lvalue_id, *span);
                    }
                }
                InstructionValue::StoreLocal { lvalue, value: val, .. } => {
                    if let Some(span) = known_dynamic_components.get(&val.identifier) {
                        let span = *span;
                        known_dynamic_components.insert(lvalue_id, span);
                        known_dynamic_components.insert(lvalue.place.identifier, span);
                    }
                }
                InstructionValue::JsxExpression { tag: JsxTag::Place(tag_place), .. } => {
                    if let Some(location) = known_dynamic_components.get(&tag_place.identifier) {
                        let location = *location;
                        let diagnostic = CompilerDiagnostic::new(
                            ErrorCategory::StaticComponents,
                            "Cannot create components during render",
                            Some("Components created during render will reset their state each time they are created. Declare components outside of render".to_string()),
                        )
                        .with_detail(CompilerDiagnosticDetail::Error {
                            span: tag_place.span,
                            message: Some("This component is created during render".to_string()),
                        })
                        .with_detail(CompilerDiagnosticDetail::Error {
                            span: location,
                            message: Some(
                                "The component is created during render here".to_string(),
                            ),
                        });
                        error.push_diagnostic(diagnostic);
                    }
                }
                _ => {}
            }
        }
    }

    error
}
