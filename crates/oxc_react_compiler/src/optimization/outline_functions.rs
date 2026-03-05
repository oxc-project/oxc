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
use crate::hir::visitors::map_instruction_operands;
use crate::hir::{HIRFunction, IdentifierId, InstructionValue, LoadGlobal, NonLocalBinding, Place};

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
    let mut outlined_ids: FxHashSet<IdentifierId> = FxHashSet::default();
    outline_functions_inner(func, &mut shared_env, fbt_operands, &mut outlined_ids);
    // Write the updated env (with all the new names registered) back to func.env.
    func.env = shared_env;

    // Second pass: clear scopes for all operand Places that reference outlined identifiers.
    // In the TS reference, Identifiers are shared objects so clearing the scope in one place
    // is visible everywhere. In Rust, each Place owns its own Identifier clone, so we must
    // explicitly propagate the scope removal to all uses.
    if !outlined_ids.is_empty() {
        clear_scopes_for_outlined_ids(func, &outlined_ids);
    }
}

/// Walk the entire HIR and clear scopes for any Place whose IdentifierId is in `outlined_ids`.
fn clear_scopes_for_outlined_ids(func: &mut HIRFunction, outlined_ids: &FxHashSet<IdentifierId>) {
    let clear_scope_if_outlined = &mut |mut place: Place| -> Place {
        if outlined_ids.contains(&place.identifier.id) {
            place.identifier.scope = None;
        }
        place
    };

    for block in func.body.blocks.values_mut() {
        for instr in &mut block.instructions {
            // Clear scope on lvalue if outlined
            if outlined_ids.contains(&instr.lvalue.identifier.id) {
                instr.lvalue.identifier.scope = None;
            }
            // Clear scope on all operand places via map (takes ownership of Place then returns it)
            map_instruction_operands(instr, clear_scope_if_outlined);
            // Recurse into nested functions
            match &mut instr.value {
                InstructionValue::FunctionExpression(fe) => {
                    clear_scopes_for_outlined_ids(&mut fe.lowered_func.func, outlined_ids);
                }
                InstructionValue::ObjectMethod(om) => {
                    clear_scopes_for_outlined_ids(&mut om.lowered_func.func, outlined_ids);
                }
                _ => {}
            }
        }
    }
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
    outlined_ids: &mut FxHashSet<IdentifierId>,
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
                        outlined_ids,
                    );
                }
                InstructionValue::ObjectMethod(obj_method) => {
                    outline_functions_inner(
                        &mut obj_method.lowered_func.func,
                        shared_env,
                        fbt_operands,
                        outlined_ids,
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

                // Track the outlined identifier ID so we can clear its scope everywhere.
                // In the TS reference, Identifiers are shared objects — clearing scope on the
                // FunctionExpression lvalue propagates to all uses automatically. In Rust, each
                // Place owns its own Identifier clone, so we do a second pass (in the caller).
                outlined_ids.insert(instr.lvalue.identifier.id);

                // Replace the instruction value with a LoadGlobal.
                // Clear the lvalue scope: a LoadGlobal is always a stable global reference
                // (mutableRange.start = 0 in the TS reference, excluded from scope unions),
                // so it does not need its own reactive scope / sentinel check.
                instr.value = InstructionValue::LoadGlobal(LoadGlobal {
                    binding: NonLocalBinding::Global { name: name_value },
                    loc,
                });
                instr.lvalue.identifier.scope = None;
            }
        }
    }
}
