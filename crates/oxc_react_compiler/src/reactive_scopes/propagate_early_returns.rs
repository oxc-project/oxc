/// Propagate early returns in reactive scopes.
///
/// Port of `ReactiveScopes/PropagateEarlyReturns.ts` from the React Compiler.
///
/// This pass ensures that reactive blocks honor the control flow behavior of the
/// original code including early return semantics. Specifically, if a reactive
/// scope early returned during the previous execution and the inputs to that block
/// have not changed, then the code should early return (with the same value) again.
///
/// The compilation strategy is as follows. For each top-level reactive scope
/// that contains (transitively) an early return:
///
/// - Label the scope
/// - Synthesize a new temporary (e.g. `t0`) and set it as a declaration of the scope.
///   This will represent the possibly-unset return value for that scope.
/// - Make the first instruction of the scope the declaration of that temporary,
///   assigning a sentinel value (can reuse the same symbol as we use for cache slots).
///   This assignment ensures that if we don't take an early return, that the value
///   is the sentinel.
/// - Replace all `return` statements with:
///   - An assignment of the temporary with the value being returned.
///   - A `break` to the reactive scope's label.
///
/// Finally, CodegenReactiveScope adds an if check following the reactive scope:
/// if the early return temporary value is *not* the sentinel value, we early return
/// it. Otherwise, execution continues.
use crate::hir::{
    CallArg, EarlyReturnValue, Effect, IdentifierName, InstructionId, InstructionKind,
    InstructionValue, LValue, LoadGlobal, MethodCall, NonLocalBinding, Place, PrimitiveValue,
    PrimitiveValueKind, PropertyLoad, ReactiveBlock, ReactiveBreakTerminal, ReactiveFunction,
    ReactiveInstruction, ReactiveInstructionStatement, ReactiveLabel, ReactiveLabelTerminal,
    ReactiveScopeBlock, ReactiveScopeDeclaration, ReactiveStatement, ReactiveTerminal,
    ReactiveTerminalStatement, ReactiveTerminalTargetKind, ReactiveValue, StoreLocal,
    environment::Environment, hir_builder::create_temporary_place, types::PropertyLiteral,
};

use super::codegen_reactive_function::EARLY_RETURN_SENTINEL;

/// State threaded through the traversal.
struct State {
    /// Are we within a reactive scope? We use this for two things:
    /// - When we find an early return, transform it to an assign+break
    ///   only if we're in a reactive scope
    /// - Annotate reactive scopes that contain early returns...but only
    ///   the outermost reactive scope, we can't do this for nested
    ///   scopes.
    within_reactive_scope: bool,

    /// Store early return information to bubble it back up to the outermost
    /// reactive scope.
    early_return_value: Option<Box<EarlyReturnValue>>,
}

/// Propagate early returns from reactive scopes.
pub fn propagate_early_returns(func: &mut ReactiveFunction, env: &mut Environment) {
    let mut state = State { within_reactive_scope: false, early_return_value: None };
    propagate_in_block(&mut func.body, &mut state, env);
}

fn propagate_in_block(block: &mut ReactiveBlock, state: &mut State, env: &mut Environment) {
    let mut i = 0;
    while i < block.len() {
        match &mut block[i] {
            ReactiveStatement::Scope(scope) => {
                visit_scope(scope, state, env);
                i += 1;
            }
            ReactiveStatement::PrunedScope(scope) => {
                propagate_in_block(&mut scope.instructions, state, env);
                i += 1;
            }
            ReactiveStatement::Terminal(term) => {
                let replacement = transform_terminal(term, state, env);
                match replacement {
                    Transformed::Keep => {
                        i += 1;
                    }
                    Transformed::ReplaceMany(stmts) => {
                        let count = stmts.len();
                        block.remove(i);
                        for (offset, stmt) in stmts.into_iter().enumerate() {
                            block.insert(i + offset, stmt);
                        }
                        // The replaced items are final (StoreLocal + Break), skip past them.
                        i += count;
                    }
                }
            }
            ReactiveStatement::Instruction(_) => {
                i += 1;
            }
        }
    }
}

enum Transformed {
    Keep,
    ReplaceMany(Vec<ReactiveStatement>),
}

fn visit_scope(
    scope_block: &mut ReactiveScopeBlock,
    parent_state: &mut State,
    env: &mut Environment,
) {
    // Exit early if an earlier pass has already created an early return,
    // which may happen in alternate compiler configurations.
    if scope_block.scope.early_return_value.is_some() {
        return;
    }

    let mut inner_state = State {
        within_reactive_scope: true,
        early_return_value: parent_state.early_return_value.take(),
    };
    propagate_in_block(&mut scope_block.instructions, &mut inner_state, env);

    if let Some(early_return_value) = inner_state.early_return_value {
        if parent_state.within_reactive_scope {
            // Not the outermost scope, but we save the early return information in case
            // there are other early returns within the same outermost scope
            parent_state.early_return_value = Some(early_return_value);
        } else {
            // This is the outermost scope wrapping an early return, store the early return
            // information
            scope_block.scope.early_return_value = Some(early_return_value.clone());
            scope_block.scope.declarations.insert(
                early_return_value.value.id,
                ReactiveScopeDeclaration {
                    identifier: early_return_value.value.clone(),
                    scope: scope_block.scope.clone(),
                },
            );

            let instructions = std::mem::take(&mut scope_block.instructions);
            let loc = early_return_value.loc;

            let sentinel_temp = create_temporary_place(env, loc);
            let symbol_temp = create_temporary_place(env, loc);
            let for_temp = create_temporary_place(env, loc);
            let arg_temp = create_temporary_place(env, loc);

            // 1. LoadGlobal("Symbol") -> symbolTemp
            let load_global_instr = ReactiveStatement::Instruction(ReactiveInstructionStatement {
                instruction: ReactiveInstruction {
                    id: InstructionId::ZERO,
                    lvalue: Some(symbol_temp.clone()),
                    value: ReactiveValue::Instruction(Box::new(InstructionValue::LoadGlobal(
                        LoadGlobal {
                            binding: NonLocalBinding::Global { name: "Symbol".to_string() },
                            loc,
                        },
                    ))),
                    loc,
                },
            });

            // 2. PropertyLoad(symbolTemp, "for") -> forTemp
            let property_load_instr =
                ReactiveStatement::Instruction(ReactiveInstructionStatement {
                    instruction: ReactiveInstruction {
                        id: InstructionId::ZERO,
                        lvalue: Some(for_temp.clone()),
                        value: ReactiveValue::Instruction(Box::new(
                            InstructionValue::PropertyLoad(PropertyLoad {
                                object: symbol_temp.clone(),
                                property: PropertyLiteral::String("for".to_string()),
                                loc,
                            }),
                        )),
                        loc,
                    },
                });

            // 3. Primitive(EARLY_RETURN_SENTINEL) -> argTemp
            let primitive_instr = ReactiveStatement::Instruction(ReactiveInstructionStatement {
                instruction: ReactiveInstruction {
                    id: InstructionId::ZERO,
                    lvalue: Some(arg_temp.clone()),
                    value: ReactiveValue::Instruction(Box::new(InstructionValue::Primitive(
                        PrimitiveValue {
                            value: PrimitiveValueKind::String(EARLY_RETURN_SENTINEL.to_string()),
                            loc,
                        },
                    ))),
                    loc,
                },
            });

            // 4. MethodCall(Symbol.for, sentinel_string) -> sentinelTemp
            let method_call_instr = ReactiveStatement::Instruction(ReactiveInstructionStatement {
                instruction: ReactiveInstruction {
                    id: InstructionId::ZERO,
                    lvalue: Some(sentinel_temp.clone()),
                    value: ReactiveValue::Instruction(Box::new(InstructionValue::MethodCall(
                        MethodCall {
                            receiver: symbol_temp,
                            property: for_temp,
                            args: vec![CallArg::Place(arg_temp)],
                            loc,
                        },
                    ))),
                    loc,
                },
            });

            // 5. StoreLocal(Let, earlyReturnValue.value, sentinelTemp)
            let store_local_instr = ReactiveStatement::Instruction(ReactiveInstructionStatement {
                instruction: ReactiveInstruction {
                    id: InstructionId::ZERO,
                    lvalue: None,
                    value: ReactiveValue::Instruction(Box::new(InstructionValue::StoreLocal(
                        StoreLocal {
                            lvalue: LValue {
                                kind: InstructionKind::Let,
                                place: Place {
                                    identifier: early_return_value.value.clone(),
                                    effect: Effect::ConditionallyMutate,
                                    reactive: true,
                                    loc,
                                },
                            },
                            value: sentinel_temp,
                            loc,
                        },
                    ))),
                    loc,
                },
            });

            // 6. Label terminal wrapping the original instructions
            let label_terminal = ReactiveStatement::Terminal(Box::new(ReactiveTerminalStatement {
                label: Some(ReactiveLabel { id: early_return_value.label, implicit: false }),
                terminal: ReactiveTerminal::Label(Box::new(ReactiveLabelTerminal {
                    block: instructions,
                    id: InstructionId::ZERO,
                    loc: crate::compiler_error::GENERATED_SOURCE,
                })),
            }));

            scope_block.instructions = vec![
                load_global_instr,
                property_load_instr,
                primitive_instr,
                method_call_instr,
                store_local_instr,
                label_terminal,
            ];
        }
    }
}

fn transform_terminal(
    stmt: &mut ReactiveTerminalStatement,
    state: &mut State,
    env: &mut Environment,
) -> Transformed {
    if state.within_reactive_scope
        && let ReactiveTerminal::Return(ret) = &stmt.terminal
    {
        let loc = ret.value.loc;
        let return_value = ret.value.clone();

        let early_return_value: Box<EarlyReturnValue> =
            if let Some(existing) = &state.early_return_value {
                existing.clone()
            } else {
                let mut temp_place = create_temporary_place(env, loc);
                // promoteTemporary: set the name to #t{declarationId}
                let decl_id = temp_place.identifier.declaration_id.0;
                temp_place.identifier.name = Some(IdentifierName::Promoted(format!("#t{decl_id}")));

                Box::new(EarlyReturnValue {
                    label: env.next_block_id(),
                    loc,
                    value: temp_place.identifier,
                })
            };
        state.early_return_value = Some(early_return_value.clone());

        // StoreLocal(Reassign, earlyReturnValue.value, return_value)
        let store_stmt = ReactiveStatement::Instruction(ReactiveInstructionStatement {
            instruction: ReactiveInstruction {
                id: InstructionId::ZERO,
                lvalue: None,
                value: ReactiveValue::Instruction(Box::new(InstructionValue::StoreLocal(
                    StoreLocal {
                        lvalue: LValue {
                            kind: InstructionKind::Reassign,
                            place: Place {
                                identifier: early_return_value.value.clone(),
                                effect: Effect::Capture,
                                reactive: true,
                                loc,
                            },
                        },
                        value: return_value,
                        loc,
                    },
                ))),
                loc,
            },
        });

        // Break targeting the scope's label
        let break_stmt = ReactiveStatement::Terminal(Box::new(ReactiveTerminalStatement {
            label: None,
            terminal: ReactiveTerminal::Break(ReactiveBreakTerminal {
                target: early_return_value.label,
                id: InstructionId::ZERO,
                target_kind: ReactiveTerminalTargetKind::Labeled,
                loc,
            }),
        }));

        return Transformed::ReplaceMany(vec![store_stmt, break_stmt]);
    }

    // Not a return inside a reactive scope -- traverse the terminal's children
    propagate_in_terminal(&mut stmt.terminal, state, env);
    Transformed::Keep
}

fn propagate_in_terminal(
    terminal: &mut ReactiveTerminal,
    state: &mut State,
    env: &mut Environment,
) {
    match terminal {
        ReactiveTerminal::If(t) => {
            propagate_in_block(&mut t.consequent, state, env);
            if let Some(alt) = &mut t.alternate {
                propagate_in_block(alt, state, env);
            }
        }
        ReactiveTerminal::Switch(t) => {
            for case in &mut t.cases {
                if let Some(block) = &mut case.block {
                    propagate_in_block(block, state, env);
                }
            }
        }
        ReactiveTerminal::While(t) => propagate_in_block(&mut t.r#loop, state, env),
        ReactiveTerminal::DoWhile(t) => propagate_in_block(&mut t.r#loop, state, env),
        ReactiveTerminal::For(t) => propagate_in_block(&mut t.r#loop, state, env),
        ReactiveTerminal::ForOf(t) => propagate_in_block(&mut t.r#loop, state, env),
        ReactiveTerminal::ForIn(t) => propagate_in_block(&mut t.r#loop, state, env),
        ReactiveTerminal::Label(t) => propagate_in_block(&mut t.block, state, env),
        ReactiveTerminal::Try(t) => {
            propagate_in_block(&mut t.block, state, env);
            propagate_in_block(&mut t.handler, state, env);
        }
        ReactiveTerminal::Break(_)
        | ReactiveTerminal::Continue(_)
        | ReactiveTerminal::Return(_)
        | ReactiveTerminal::Throw(_) => {}
    }
}
