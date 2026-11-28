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
use oxc_index::IndexSlice;
use smallvec::SmallVec;

use crate::diagnostics;
use crate::react_compiler_hir::{FunctionId, HirFunction, IdentifierId, InstructionValue, JsxTag};
use oxc_span::Span;

/// Validates that components used in JSX are not dynamically created during render.
///
/// Returns the diagnostics found (may be empty).
/// Called via `env.logErrors()` pattern in Pipeline.ts.
pub fn validate_static_components<'a>(
    func: &HirFunction<'a>,
    functions: &IndexSlice<FunctionId, [HirFunction<'a>]>,
) -> Diagnostics {
    let mut error = Diagnostics::new();
    let mut known_dynamic_components: FxHashMap<IdentifierId, Option<Span>> = FxHashMap::default();
    let mut worklist: SmallVec<[_; 1]> = smallvec::smallvec![func];

    while let Some(func) = worklist.pop() {
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
                    InstructionValue::FunctionExpression { lowered_func, span, .. } => {
                        let callback = &functions[lowered_func.func];
                        let location = callback.diagnostic_span().or(*span);
                        known_dynamic_components.insert(lvalue_id, location);
                        // Unlike the TS implementation, preserved useMemo callbacks
                        // retain their scope but still create and render components.
                        // Share the map so captures of render-local components stay dynamic.
                        if callback.is_memo_callback {
                            worklist.push(callback);
                        }
                    }
                    InstructionValue::NewExpression { callee, span, .. }
                    | InstructionValue::CallExpression { callee, span, .. } => {
                        known_dynamic_components.insert(lvalue_id, callee.span.or(*span));
                    }
                    InstructionValue::MethodCall { property, span, .. } => {
                        known_dynamic_components.insert(lvalue_id, property.span.or(*span));
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
                        if let Some(location) = known_dynamic_components.get(&tag_place.identifier)
                        {
                            let location = *location;
                            let diagnostic = diagnostics::static_component_during_render(
                                tag_place.span,
                                location,
                            );
                            error.push(diagnostic);
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    error
}
