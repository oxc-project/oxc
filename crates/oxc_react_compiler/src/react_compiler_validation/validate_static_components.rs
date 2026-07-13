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

use oxc_diagnostics::Diagnostics;

use crate::diagnostics::ErrorCategory;
use crate::react_compiler_hir::{HirFunction, IdentifierId, InstructionValue, JsxTag};
use oxc_span::Span;

/// Validates that components used in JSX are not dynamically created during render.
///
/// Returns the diagnostics found (may be empty).
/// Called via `env.logErrors()` pattern in Pipeline.ts.
pub fn validate_static_components(func: &HirFunction) -> Diagnostics {
    let mut error = Diagnostics::new();
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
            let instr = &func.instructions[instr_id.index()];
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
                        let diagnostic = ErrorCategory::StaticComponents
                            .diagnostic("Cannot create components during render")
                            .with_help("Components created during render will reset their state each time they are created. Declare components outside of render")
                            .with_labels(
                                tag_place
                                    .span
                                    .map(|s| s.label("This component is created during render")),
                            )
                            .and_labels(location.map(
                                |s| s.label("The component is created during render here"),
                            ));
                        error.push(diagnostic);
                    }
                }
                _ => {}
            }
        }
    }

    error
}
