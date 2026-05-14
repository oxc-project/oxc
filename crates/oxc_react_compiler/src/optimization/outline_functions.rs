/// Outline function expressions from reactive scopes.
///
/// Port of `Optimization/OutlineFunctions.ts` from the React Compiler.
///
/// Moves function expressions out of reactive scopes when safe to do so,
/// reducing the number of values that need to be memoized. When a function
/// expression has no context (closureless), is anonymous, and is not an fbt
/// operand, it can be outlined: the function body is extracted to a top-level
/// function and the original instruction is replaced with a `LoadGlobal`.
use rustc_hash::FxHashSet;

use crate::hir::environment::Environment;
use crate::hir::{HIRFunction, IdentifierId, InstructionValue, LoadGlobal, NonLocalBinding};

/// Outline function expressions from reactive scopes.
///
/// All outlined functions are registered in the TOP-LEVEL function's environment,
/// so that globally unique names are generated consistently across all nesting
/// levels. This mirrors the TypeScript version, where a single `env` object is
/// shared (by reference) across all nested function scopes.
pub(crate) fn outline_functions(func: &mut HIRFunction, fbt_operands: &FxHashSet<IdentifierId>) {
    // We extract the env by value (clone), use it as the "shared" name generator,
    // then write the result back. This matches the TS behaviour where all functions
    // share the same env object.
    let mut shared_env = func.env.clone();
    outline_functions_inner(func, &mut shared_env, fbt_operands);
    // Write the updated env (with all the new names registered) back to func.env.
    func.env = shared_env;
}

/// Inner implementation that uses an explicitly provided `shared_env` for:
///   1. Generating globally unique names
///   2. Registering outlined functions
///
/// `shared_env` is passed by mutable reference so all nesting levels share the
/// same name counter, exactly like the TypeScript version where `fn.env` is the
/// same object at every nesting level.
fn outline_functions_inner(
    func: &mut HIRFunction,
    shared_env: &mut Environment,
    fbt_operands: &FxHashSet<IdentifierId>,
) {
    let block_ids: Vec<_> = func.body.blocks.keys().copied().collect();

    for block_id in block_ids {
        let Some(block) = func.body.blocks.get_mut(&block_id) else {
            continue;
        };

        for instr in &mut block.instructions {
            // First pass: recurse into inner functions using the SAME shared_env.
            // In the TypeScript version `fn.env` is the same object reference at
            // every nesting level, so name generation is always global. We replicate
            // that by explicitly threading `shared_env` down through the recursion.
            match &mut instr.value {
                InstructionValue::FunctionExpression(func_expr) => {
                    outline_functions_inner(
                        &mut func_expr.lowered_func.func,
                        shared_env,
                        fbt_operands,
                    );
                }
                InstructionValue::ObjectMethod(obj_method) => {
                    outline_functions_inner(
                        &mut obj_method.lowered_func.func,
                        shared_env,
                        fbt_operands,
                    );
                }
                _ => {}
            }

            // Second pass: outline closureless anonymous FunctionExpressions
            let should_outline = matches!(&instr.value, InstructionValue::FunctionExpression(func_expr)
                if func_expr.lowered_func.func.context.is_empty()
                    && func_expr.lowered_func.func.id.is_none()
                    && !fbt_operands.contains(&instr.lvalue.identifier.id));

            if should_outline {
                let InstructionValue::FunctionExpression(func_expr) = &mut instr.value else {
                    continue;
                };
                let loc = func_expr.loc;

                // Generate a globally unique name using the SHARED env, so all
                // nested functions draw from the same counter/namespace.
                let hint = func_expr.lowered_func.func.id.as_deref().or(func_expr
                    .lowered_func
                    .func
                    .name_hint
                    .as_deref());
                let id_name = shared_env.generate_globally_unique_identifier_name(hint);
                let name_value = id_name.value().to_string();

                // Set the function's id to the generated name
                func_expr.lowered_func.func.id = Some(name_value.clone());

                // Take the lowered func out and register it in the shared env
                let lowered_func = std::mem::replace(
                    &mut func_expr.lowered_func.func,
                    Box::new(HIRFunction {
                        loc,
                        id: None,
                        name_hint: None,
                        fn_type: crate::hir::ReactFunctionType::Other,
                        env: shared_env.clone(),
                        params: Vec::new(),
                        returns: instr.lvalue.clone(),
                        context: Vec::new(),
                        body: crate::hir::Hir {
                            entry: crate::hir::BlockId(0),
                            blocks: indexmap::IndexMap::default(),
                        },
                        generator: false,
                        is_async: false,
                        directives: Vec::new(),
                        aliasing_effects: None,
                    }),
                );
                shared_env.outline_function(*lowered_func, None);

                // Replace the instruction value with a LoadGlobal.
                //
                // Note: do NOT clear `instr.lvalue.identifier.scope` here (nor on
                // operand Places elsewhere in the function). The TS reference
                // outliner leaves the lvalue's scope intact — it only replaces
                // the RHS. Clearing scope across all uses would over-prune any
                // surrounding reactive scope that legitimately wraps a ternary /
                // logical / call site that consumes the outlined helper, and
                // breaks fixtures like `functionexpr-conditional-access-2.tsx`
                // where the ternary `props == null ? _temp : f` must remain in
                // its own scope keyed on the original (`f`, `props`) deps.
                instr.value = InstructionValue::LoadGlobal(LoadGlobal {
                    binding: NonLocalBinding::Global { name: name_value },
                    loc,
                });
            }
        }
    }
}
