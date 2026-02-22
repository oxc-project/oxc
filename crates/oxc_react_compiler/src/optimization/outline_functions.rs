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

use crate::hir::{HIRFunction, IdentifierId, InstructionValue, LoadGlobal, NonLocalBinding};

/// Outline function expressions from reactive scopes.
pub(crate) fn outline_functions(func: &mut HIRFunction, fbt_operands: &FxHashSet<IdentifierId>) {
    let block_ids: Vec<_> = func.body.blocks.keys().copied().collect();

    for block_id in block_ids {
        let Some(block) = func.body.blocks.get_mut(&block_id) else {
            continue;
        };

        for instr in &mut block.instructions {
            // First pass: recurse into inner functions (FunctionExpression and ObjectMethod)
            match &mut instr.value {
                InstructionValue::FunctionExpression(func_expr) => {
                    outline_functions(&mut func_expr.lowered_func.func, fbt_operands);
                }
                InstructionValue::ObjectMethod(obj_method) => {
                    outline_functions(&mut obj_method.lowered_func.func, fbt_operands);
                }
                _ => {}
            }

            // Second pass: outline closureless anonymous FunctionExpressions
            let should_outline = matches!(&instr.value, InstructionValue::FunctionExpression(func_expr)
                if func_expr.lowered_func.func.context.is_empty()
                    && func_expr.lowered_func.func.id.is_none()
                    && !fbt_operands.contains(&instr.lvalue.identifier.id));

            if should_outline {
                let func_expr = match &mut instr.value {
                    InstructionValue::FunctionExpression(f) => f,
                    _ => continue,
                };
                let loc = func_expr.loc;

                // Generate a globally unique name from the function's nameHint
                let hint = func_expr.lowered_func.func.id.as_deref().or(func_expr
                    .lowered_func
                    .func
                    .name_hint
                    .as_deref());
                let id_name = func.env.generate_globally_unique_identifier_name(hint);
                let name_value = id_name.value().to_string();

                // Set the function's id to the generated name
                func_expr.lowered_func.func.id = Some(name_value.clone());

                // Take the lowered func out and outline it
                let lowered_func = std::mem::replace(
                    &mut func_expr.lowered_func.func,
                    Box::new(HIRFunction {
                        loc,
                        id: None,
                        name_hint: None,
                        fn_type: crate::hir::ReactFunctionType::Other,
                        env: func.env.clone(),
                        params: Vec::new(),
                        returns: instr.lvalue.clone(),
                        context: Vec::new(),
                        body: crate::hir::Hir {
                            entry: crate::hir::BlockId(0),
                            blocks: Default::default(),
                        },
                        generator: false,
                        is_async: false,
                        directives: Vec::new(),
                        aliasing_effects: None,
                    }),
                );
                func.env.outline_function(*lowered_func, None);

                // Replace the instruction value with a LoadGlobal
                instr.value = InstructionValue::LoadGlobal(LoadGlobal {
                    binding: NonLocalBinding::Global { name: name_value },
                    loc,
                });
            }
        }
    }
}
