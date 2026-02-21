/// Build a reactive function from the HIR control-flow graph.
///
/// Port of `ReactiveScopes/BuildReactiveFunction.ts` from the React Compiler.
///
/// Converts from HIR (lower-level CFG) to ReactiveFunction, a tree representation
/// that is closer to an AST. This pass restores the original control flow constructs,
/// including break/continue to labeled statements.
use rustc_hash::FxHashMap;

use crate::{
    compiler_error::SourceLocation,
    hir::{
        BlockId, GotoVariant, Hir, HIRFunction,
        InstructionValue, Terminal,
        ReactiveBlock, ReactiveBreakTerminal, ReactiveContinueTerminal,
        ReactiveDoWhileTerminal, ReactiveForInTerminal, ReactiveForOfTerminal,
        ReactiveForTerminal, ReactiveFunction, ReactiveIfTerminal, ReactiveInstruction,
        ReactiveInstructionStatement, ReactiveLabel, ReactiveLabelTerminal,
        ReactiveReturnTerminal, ReactiveSequenceValue, ReactiveStatement,
        ReactiveSwitchCase, ReactiveSwitchTerminal, ReactiveTerminal,
        ReactiveTerminalStatement, ReactiveTerminalTargetKind,
        ReactiveThrowTerminal, ReactiveTryTerminal, ReactiveValue,
        ReactiveWhileTerminal, BasicBlock,
    },
};

/// Context for the HIR → Reactive conversion.
struct Context {
    blocks: FxHashMap<BlockId, BasicBlock>,
}

impl Context {
    fn new(hir: &Hir) -> Self {
        Self { blocks: hir.blocks.clone() }
    }

    fn block(&self, id: BlockId) -> Option<&BasicBlock> {
        self.blocks.get(&id)
    }
}

/// Convert an HIR function to a ReactiveFunction.
pub fn build_reactive_function(func: &HIRFunction) -> ReactiveFunction {
    let cx = Context::new(&func.body);
    let body = traverse_block(&cx, func.body.entry);

    ReactiveFunction {
        loc: func.loc,
        id: func.id.clone(),
        name_hint: func.name_hint.clone(),
        params: func.params.clone(),
        generator: func.generator,
        is_async: func.is_async,
        body,
        directives: func.directives.clone(),
    }
}

/// Traverse a block and its successors, building a reactive block.
fn traverse_block(cx: &Context, block_id: BlockId) -> ReactiveBlock {
    let mut statements: ReactiveBlock = Vec::new();
    let mut current_id = block_id;

    loop {
        let Some(block) = cx.block(current_id) else { break };

        // Add instructions as reactive statements
        for instr in &block.instructions {
            statements.push(ReactiveStatement::Instruction(ReactiveInstructionStatement {
                instruction: ReactiveInstruction {
                    id: instr.id,
                    lvalue: Some(instr.lvalue.clone()),
                    value: ReactiveValue::Instruction(Box::new(instr.value.clone())),
                    loc: instr.loc,
                },
            }));
        }

        // Process terminal
        match &block.terminal {
            Terminal::Goto(goto) => {
                if goto.variant == GotoVariant::Break {
                    // Continue to the target block
                    current_id = goto.block;
                    continue;
                }
                // Break/Continue gotos become terminal statements
                let target_kind = match goto.variant {
                    GotoVariant::Break => ReactiveTerminalTargetKind::Unlabeled,
                    GotoVariant::Continue => ReactiveTerminalTargetKind::Unlabeled,
                    GotoVariant::Try => ReactiveTerminalTargetKind::Implicit,
                };
                if goto.variant == GotoVariant::Continue {
                    statements.push(ReactiveStatement::Terminal(Box::new(
                        ReactiveTerminalStatement {
                            terminal: ReactiveTerminal::Continue(ReactiveContinueTerminal {
                                target: goto.block,
                                id: goto.id,
                                target_kind,
                                loc: goto.loc,
                            }),
                            label: None,
                        },
                    )));
                } else {
                    statements.push(ReactiveStatement::Terminal(Box::new(
                        ReactiveTerminalStatement {
                            terminal: ReactiveTerminal::Break(ReactiveBreakTerminal {
                                target: goto.block,
                                id: goto.id,
                                target_kind,
                                loc: goto.loc,
                            }),
                            label: None,
                        },
                    )));
                }
                break;
            }
            Terminal::Return(ret) => {
                statements.push(ReactiveStatement::Terminal(Box::new(
                    ReactiveTerminalStatement {
                        terminal: ReactiveTerminal::Return(ReactiveReturnTerminal {
                            value: ret.value.clone(),
                            id: ret.id,
                            loc: ret.loc,
                        }),
                        label: None,
                    },
                )));
                break;
            }
            Terminal::Throw(throw) => {
                statements.push(ReactiveStatement::Terminal(Box::new(
                    ReactiveTerminalStatement {
                        terminal: ReactiveTerminal::Throw(ReactiveThrowTerminal {
                            value: throw.value.clone(),
                            id: throw.id,
                            loc: throw.loc,
                        }),
                        label: None,
                    },
                )));
                break;
            }
            Terminal::If(if_term) => {
                let consequent = traverse_block(cx, if_term.consequent);
                let alternate = traverse_block(cx, if_term.alternate);
                let alternate_opt = if alternate.is_empty() { None } else { Some(alternate) };

                statements.push(ReactiveStatement::Terminal(Box::new(
                    ReactiveTerminalStatement {
                        terminal: ReactiveTerminal::If(Box::new(ReactiveIfTerminal {
                            test: if_term.test.clone(),
                            consequent,
                            alternate: alternate_opt,
                            id: if_term.id,
                            loc: if_term.loc,
                        })),
                        label: Some(ReactiveLabel {
                            id: if_term.fallthrough,
                            implicit: true,
                        }),
                    },
                )));

                current_id = if_term.fallthrough;
            }
            Terminal::Switch(switch_term) => {
                let cases: Vec<ReactiveSwitchCase> = switch_term
                    .cases
                    .iter()
                    .map(|case| ReactiveSwitchCase {
                        test: case.test.clone(),
                        block: Some(traverse_block(cx, case.block)),
                    })
                    .collect();

                statements.push(ReactiveStatement::Terminal(Box::new(
                    ReactiveTerminalStatement {
                        terminal: ReactiveTerminal::Switch(Box::new(ReactiveSwitchTerminal {
                            test: switch_term.test.clone(),
                            cases,
                            id: switch_term.id,
                            loc: switch_term.loc,
                        })),
                        label: Some(ReactiveLabel {
                            id: switch_term.fallthrough,
                            implicit: true,
                        }),
                    },
                )));

                current_id = switch_term.fallthrough;
            }
            Terminal::While(while_term) => {
                let test_block = traverse_block(cx, while_term.test);
                let test_value = block_to_reactive_value(&test_block, while_term.loc);
                let loop_body = traverse_block(cx, while_term.r#loop);

                statements.push(ReactiveStatement::Terminal(Box::new(
                    ReactiveTerminalStatement {
                        terminal: ReactiveTerminal::While(Box::new(ReactiveWhileTerminal {
                            test: test_value,
                            r#loop: loop_body,
                            id: while_term.id,
                            loc: while_term.loc,
                        })),
                        label: Some(ReactiveLabel {
                            id: while_term.fallthrough,
                            implicit: true,
                        }),
                    },
                )));

                current_id = while_term.fallthrough;
            }
            Terminal::DoWhile(do_while) => {
                let loop_body = traverse_block(cx, do_while.r#loop);
                let test_block = traverse_block(cx, do_while.test);
                let test_value = block_to_reactive_value(&test_block, do_while.loc);

                statements.push(ReactiveStatement::Terminal(Box::new(
                    ReactiveTerminalStatement {
                        terminal: ReactiveTerminal::DoWhile(Box::new(ReactiveDoWhileTerminal {
                            r#loop: loop_body,
                            test: test_value,
                            id: do_while.id,
                            loc: do_while.loc,
                        })),
                        label: Some(ReactiveLabel {
                            id: do_while.fallthrough,
                            implicit: true,
                        }),
                    },
                )));

                current_id = do_while.fallthrough;
            }
            Terminal::For(for_term) => {
                let init_block = traverse_block(cx, for_term.init);
                let init_value = block_to_reactive_value(&init_block, for_term.loc);
                let test_block = traverse_block(cx, for_term.test);
                let test_value = block_to_reactive_value(&test_block, for_term.loc);
                let update = for_term.update.map(|update_id| {
                    let update_block = traverse_block(cx, update_id);
                    block_to_reactive_value(&update_block, for_term.loc)
                });
                let loop_body = traverse_block(cx, for_term.r#loop);

                statements.push(ReactiveStatement::Terminal(Box::new(
                    ReactiveTerminalStatement {
                        terminal: ReactiveTerminal::For(Box::new(ReactiveForTerminal {
                            init: init_value,
                            test: test_value,
                            update,
                            r#loop: loop_body,
                            id: for_term.id,
                            loc: for_term.loc,
                        })),
                        label: Some(ReactiveLabel {
                            id: for_term.fallthrough,
                            implicit: true,
                        }),
                    },
                )));

                current_id = for_term.fallthrough;
            }
            Terminal::ForOf(for_of) => {
                let init_block = traverse_block(cx, for_of.init);
                let init_value = block_to_reactive_value(&init_block, for_of.loc);
                let test_block = traverse_block(cx, for_of.test);
                let test_value = block_to_reactive_value(&test_block, for_of.loc);
                let loop_body = traverse_block(cx, for_of.r#loop);

                statements.push(ReactiveStatement::Terminal(Box::new(
                    ReactiveTerminalStatement {
                        terminal: ReactiveTerminal::ForOf(Box::new(ReactiveForOfTerminal {
                            init: init_value,
                            test: test_value,
                            r#loop: loop_body,
                            id: for_of.id,
                            loc: for_of.loc,
                        })),
                        label: Some(ReactiveLabel {
                            id: for_of.fallthrough,
                            implicit: true,
                        }),
                    },
                )));

                current_id = for_of.fallthrough;
            }
            Terminal::ForIn(for_in) => {
                let init_block = traverse_block(cx, for_in.init);
                let init_value = block_to_reactive_value(&init_block, for_in.loc);
                let loop_body = traverse_block(cx, for_in.r#loop);

                statements.push(ReactiveStatement::Terminal(Box::new(
                    ReactiveTerminalStatement {
                        terminal: ReactiveTerminal::ForIn(Box::new(ReactiveForInTerminal {
                            init: init_value,
                            r#loop: loop_body,
                            id: for_in.id,
                            loc: for_in.loc,
                        })),
                        label: Some(ReactiveLabel {
                            id: for_in.fallthrough,
                            implicit: true,
                        }),
                    },
                )));

                current_id = for_in.fallthrough;
            }
            Terminal::Label(label) => {
                let block_body = traverse_block(cx, label.block);

                statements.push(ReactiveStatement::Terminal(Box::new(
                    ReactiveTerminalStatement {
                        terminal: ReactiveTerminal::Label(Box::new(ReactiveLabelTerminal {
                            block: block_body,
                            id: label.id,
                            loc: label.loc,
                        })),
                        label: Some(ReactiveLabel {
                            id: label.fallthrough,
                            implicit: false,
                        }),
                    },
                )));

                current_id = label.fallthrough;
            }
            Terminal::Try(try_term) => {
                let block_body = traverse_block(cx, try_term.block);
                let handler_body = traverse_block(cx, try_term.handler);

                statements.push(ReactiveStatement::Terminal(Box::new(
                    ReactiveTerminalStatement {
                        terminal: ReactiveTerminal::Try(Box::new(ReactiveTryTerminal {
                            block: block_body,
                            handler_binding: try_term.handler_binding.clone(),
                            handler: handler_body,
                            id: try_term.id,
                            loc: try_term.loc,
                        })),
                        label: Some(ReactiveLabel {
                            id: try_term.fallthrough,
                            implicit: true,
                        }),
                    },
                )));

                current_id = try_term.fallthrough;
            }
            Terminal::MaybeThrow(mt) => {
                // Continue to the continuation block
                current_id = mt.continuation;
            }
            Terminal::Scope(scope) => {
                let block_body = traverse_block(cx, scope.block);
                statements.push(ReactiveStatement::Scope(
                    crate::hir::ReactiveScopeBlock {
                        scope: scope.scope.clone(),
                        instructions: block_body,
                    },
                ));
                current_id = scope.fallthrough;
            }
            Terminal::PrunedScope(scope) => {
                let block_body = traverse_block(cx, scope.block);
                statements.push(ReactiveStatement::PrunedScope(
                    crate::hir::PrunedReactiveScopeBlock {
                        scope: scope.scope.clone(),
                        instructions: block_body,
                    },
                ));
                current_id = scope.fallthrough;
            }
            // Terminals that end the block without continuation
            Terminal::Unreachable(_) | Terminal::Unsupported(_) => {
                break;
            }
            // Terminals handled specially (Logical, Ternary, Optional, etc.)
            Terminal::Logical(logical) => {
                let test_block = traverse_block(cx, logical.test);
                let _test_value = block_to_reactive_value(&test_block, logical.loc);
                // The logical expression is lowered into the fallthrough
                // For now, emit as-is
                current_id = logical.fallthrough;
                // Logical test lowering handled in full implementation
            }
            Terminal::Ternary(ternary) => {
                let test_block = traverse_block(cx, ternary.test);
                let _test_value = block_to_reactive_value(&test_block, ternary.loc);
                current_id = ternary.fallthrough;
            }
            Terminal::Optional(optional) => {
                let test_block = traverse_block(cx, optional.test);
                let _test_value = block_to_reactive_value(&test_block, optional.loc);
                current_id = optional.fallthrough;
            }
            Terminal::Sequence(seq) => {
                let block_body = traverse_block(cx, seq.block);
                statements.extend(block_body);
                current_id = seq.fallthrough;
            }
            Terminal::Branch(branch) => {
                let consequent = traverse_block(cx, branch.consequent);
                let alternate = traverse_block(cx, branch.alternate);
                let alternate_opt = if alternate.is_empty() { None } else { Some(alternate) };

                statements.push(ReactiveStatement::Terminal(Box::new(
                    ReactiveTerminalStatement {
                        terminal: ReactiveTerminal::If(Box::new(ReactiveIfTerminal {
                            test: branch.test.clone(),
                            consequent,
                            alternate: alternate_opt,
                            id: branch.id,
                            loc: branch.loc,
                        })),
                        label: Some(ReactiveLabel {
                            id: branch.fallthrough,
                            implicit: true,
                        }),
                    },
                )));

                current_id = branch.fallthrough;
            }
        }
    }

    statements
}

/// Convert a reactive block to a single reactive value (for expressions).
fn block_to_reactive_value(block: &ReactiveBlock, loc: SourceLocation) -> ReactiveValue {
    if block.is_empty() {
        return ReactiveValue::Instruction(Box::new(InstructionValue::Primitive(
            crate::hir::PrimitiveValue {
                value: crate::hir::PrimitiveValueKind::Undefined,
                loc,
            },
        )));
    }

    // If there's a single instruction, return its value
    if block.len() == 1
        && let ReactiveStatement::Instruction(stmt) = &block[0] {
            return stmt.instruction.value.clone();
        }

    // Otherwise wrap in a sequence
    let instructions: Vec<ReactiveInstruction> = block
        .iter()
        .filter_map(|stmt| {
            if let ReactiveStatement::Instruction(s) = stmt {
                Some(s.instruction.clone())
            } else {
                None
            }
        })
        .collect();

    if let Some(last) = instructions.last() {
        ReactiveValue::Sequence(ReactiveSequenceValue {
            instructions: instructions[..instructions.len() - 1].to_vec(),
            id: last.id,
            value: Box::new(last.value.clone()),
            loc,
        })
    } else {
        ReactiveValue::Instruction(Box::new(InstructionValue::Primitive(
            crate::hir::PrimitiveValue {
                value: crate::hir::PrimitiveValueKind::Undefined,
                loc,
            },
        )))
    }
}
