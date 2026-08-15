// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

//! For simplicity the majority of compiler passes do not treat hooks specially. However, hooks are
//! different from regular functions in two key ways:
//! - They can introduce reactivity even when their arguments are non-reactive (accounted for in
//!   InferReactivePlaces)
//! - They cannot be called conditionally
//!
//! The `use` operator is similar:
//! - It can access context, and therefore introduce reactivity
//! - It can be called conditionally, but _it must be called if the component needs the return value_.
//!   This is because React uses the fact that use was called to remember that the component needs the
//!   value, and that changes to the input should invalidate the component itself.
//!
//! This pass accounts for the "can't call conditionally" aspect of both hooks and use. Though the
//! reasoning is slightly different for each, the result is that we can't memoize scopes that call
//! hooks or use since this would make them called conditionally in the output.
//!
//! Opaque tagged templates also have to execute unconditionally. A tag is an arbitrary function
//! call that may have side effects or return a fresh value or stable alias. Inferred result types
//! cannot prove otherwise because consumers such as coercive binary operators may constrain an
//! opaque result to `Primitive`. A configured read-only, pure signature with a known safe return is
//! sufficient evidence to retain memoization.
//!
//! The pass finds and removes any scopes that transitively contain one of these instructions. By
//! running all the reactive scope inference first, we know that the reactive scopes accurately
//! describe the set of values which "construct together", and remove _all_ that memoization in order
//! to ensure the instruction does not inadvertently become conditional.
//!
//! Analogous to TS `ReactiveScopes/FlattenScopesWithHooksOrUseHIR.ts`.

use oxc_diagnostics::OxcDiagnostic;

use crate::diagnostics;
use crate::react_compiler_hir::environment::Environment;
use crate::react_compiler_hir::{
    BlockId, HirFunction, InstructionValue, Terminal, Type, is_use_operator_type,
};
use crate::react_compiler_inference::is_known_pure_tagged_template;

/// Flattens reactive scopes that contain hook calls, `use()` calls, or tagged templates.
///
/// Hooks and `use` must be called unconditionally. Tagged templates without a known pure signature
/// must also be recomputed because their inferred result type cannot establish that the call is pure
/// or that its result is stable. Any reactive scope containing such an instruction must therefore
/// be flattened to avoid making its evaluation conditional.
pub fn flatten_scopes_with_hooks_or_use_hir(
    func: &mut HirFunction,
    env: &Environment,
) -> Result<(), OxcDiagnostic> {
    let mut active_scopes: Vec<ActiveScope> = Vec::new();
    let mut prune: Vec<BlockId> = Vec::new();

    // Collect block ids to allow mutation during iteration
    let block_ids: Vec<BlockId> = func.body.blocks.keys().copied().collect();

    for block_id in &block_ids {
        // Remove scopes whose fallthrough matches this block
        active_scopes.retain(|scope| scope.fallthrough != *block_id);

        let block = &func.body.blocks[block_id];

        // Check instructions that must execute unconditionally.
        for instr_id in &block.instructions {
            let instr = &func.instructions[instr_id.index()];
            match &instr.value {
                InstructionValue::CallExpression { callee, .. } => {
                    let callee_ty = &env.types[env.identifiers[callee.identifier].type_];
                    if is_hook_or_use(env, callee_ty)? {
                        // All active scopes must be pruned
                        prune.extend(active_scopes.iter().map(|s| s.block));
                        active_scopes.clear();
                    }
                }
                InstructionValue::TaggedTemplateExpression { tag, subexprs, .. } => {
                    if !is_known_pure_tagged_template(env, tag, subexprs.len()) {
                        prune.extend(active_scopes.iter().map(|s| s.block));
                        active_scopes.clear();
                    }
                }
                InstructionValue::MethodCall { property, .. } => {
                    let property_ty = &env.types[env.identifiers[property.identifier].type_];
                    if is_hook_or_use(env, property_ty)? {
                        prune.extend(active_scopes.iter().map(|s| s.block));
                        active_scopes.clear();
                    }
                }
                _ => {}
            }
        }

        // Track scope terminals
        if let Terminal::Scope { fallthrough, .. } = &block.terminal {
            active_scopes.push(ActiveScope { block: *block_id, fallthrough: *fallthrough });
        }
    }

    // Apply pruning: convert Scope terminals to Label or PrunedScope
    for id in prune {
        let block = &func.body.blocks[&id];
        let terminal = &block.terminal;

        let (scope_block, fallthrough, eval_id, span, scope) = match terminal {
            Terminal::Scope { block, fallthrough, id, span, scope } => {
                (*block, *fallthrough, *id, *span, *scope)
            }
            _ => {
                return Err(diagnostics::expected_scope_terminal(id.index()));
            }
        };

        // Check if the scope body is a single-instruction block that goes directly
        // to fallthrough — if so, use Label instead of PrunedScope
        let body = &func.body.blocks[&scope_block];
        let new_terminal = if body.instructions.len() == 1
            && matches!(&body.terminal, Terminal::Goto { block, .. } if *block == fallthrough)
        {
            // This was a scope just for a hook call, which doesn't need memoization.
            // Flatten it away. We rely on PruneUnusedLabels to do the actual flattening.
            Terminal::Label { block: scope_block, block_span: None, fallthrough, id: eval_id, span }
        } else {
            Terminal::PrunedScope { block: scope_block, fallthrough, scope, id: eval_id, span }
        };

        let block_mut = func.body.blocks.get_mut(&id).unwrap();
        block_mut.terminal = new_terminal;
    }
    Ok(())
}

struct ActiveScope {
    block: BlockId,
    fallthrough: BlockId,
}

fn is_hook_or_use(env: &Environment, ty: &Type) -> Result<bool, OxcDiagnostic> {
    Ok(env.get_hook_kind_for_type(ty)?.is_some() || is_use_operator_type(ty))
}
