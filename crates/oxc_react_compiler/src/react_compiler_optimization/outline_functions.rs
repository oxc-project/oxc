// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

//! Port of OutlineFunctions from TypeScript (`Optimization/OutlineFunctions.ts`).
//!
//! Extracts anonymous function expressions that do not close over any local
//! variables into top-level outlined functions. The original instruction is
//! replaced with a `LoadGlobal` referencing the outlined function's generated name.
//!
//! Conditional on `env.config.enable_function_outlining`.

use std::mem::replace;

use rustc_hash::FxHashSet;

use oxc_str::Ident;

use crate::react_compiler_hir::environment::Environment;
use crate::react_compiler_hir::{
    FunctionId, HirFunction, IdentifierId, InstructionValue, NonLocalBinding,
};
use crate::react_compiler_ssa::enter_ssa::placeholder_function;

/// Outline anonymous function expressions that have no captured context variables.
///
/// Ported from TS `outlineFunctions` in `Optimization/OutlineFunctions.ts`.
pub fn outline_functions<'a>(
    func: &mut HirFunction<'a>,
    env: &mut Environment<'a>,
    fbt_operands: &FxHashSet<IdentifierId>,
) {
    // Collect per-instruction actions to maintain depth-first name allocation order.
    // Each entry: (instr index, function_id to recurse into, should_outline)
    enum Action {
        /// Recurse into an inner function (FunctionExpression or ObjectMethod)
        Recurse(FunctionId),
        /// Recurse then outline a FunctionExpression
        RecurseAndOutline { instr_idx: usize, function_id: FunctionId },
    }

    let mut actions: Vec<Action> = Vec::new();

    for block in func.body.blocks.values() {
        for &instr_id in &block.instructions {
            let instr = &func.instructions[instr_id.index()];
            let lvalue_id = instr.lvalue.identifier;

            match &instr.value {
                InstructionValue::FunctionExpression { lowered_func, .. } => {
                    let inner_func = &env.functions[lowered_func.func.index()];

                    // Check outlining conditions (TS only checks func.id === null, not name):
                    // 1. No captured context variables
                    // 2. Anonymous (no explicit id on the inner function)
                    // 3. Not an fbt operand
                    if inner_func.context.is_empty()
                        && inner_func.id.is_none()
                        && !fbt_operands.contains(&lvalue_id)
                    {
                        actions.push(Action::RecurseAndOutline {
                            instr_idx: instr_id.index(),
                            function_id: lowered_func.func,
                        });
                    } else {
                        actions.push(Action::Recurse(lowered_func.func));
                    }
                }
                InstructionValue::ObjectMethod { lowered_func, .. } => {
                    // Recurse into object methods (but don't outline them)
                    actions.push(Action::Recurse(lowered_func.func));
                }
                _ => {}
            }
        }
    }

    // Process actions sequentially: for each instruction, recurse first (depth-first),
    // then generate name and outline. This matches TS ordering where inner functions
    // get names allocated before outer ones.
    for action in actions {
        match action {
            Action::Recurse(function_id) => {
                let mut inner_func =
                    replace(&mut env.functions[function_id.index()], placeholder_function());
                outline_functions(&mut inner_func, env, fbt_operands);
                env.functions[function_id.index()] = inner_func;
            }
            Action::RecurseAndOutline { instr_idx, function_id } => {
                // First recurse into the inner function (depth-first)
                let mut inner_func =
                    replace(&mut env.functions[function_id.index()], placeholder_function());
                outline_functions(&mut inner_func, env, fbt_operands);
                env.functions[function_id.index()] = inner_func;

                // Then generate the name and outline (after recursion, matching TS order)
                let inner = &env.functions[function_id.index()];
                let hint: Option<Ident<'a>> = inner.id.or(inner.name_hint);
                let generated_name = env.generate_globally_unique_identifier_name(hint.as_deref());

                // Set the id on the inner function
                env.functions[function_id.index()].id = Some(generated_name);

                // Outline the function
                let outlined_func = env.functions[function_id.index()].clone();
                env.outline_function(outlined_func, None);

                // Replace the instruction value with LoadGlobal
                let span = func.instructions[instr_idx].value.span().cloned();
                func.instructions[instr_idx].value = InstructionValue::LoadGlobal {
                    binding: NonLocalBinding::Global { name: generated_name },
                    span,
                };
            }
        }
    }
}
