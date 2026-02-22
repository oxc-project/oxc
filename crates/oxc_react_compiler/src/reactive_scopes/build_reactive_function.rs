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
        BasicBlock, BlockId, GotoVariant, HIRFunction, Hir, Instruction, InstructionId,
        InstructionValue, Place, ReactiveBlock, ReactiveBreakTerminal, ReactiveContinueTerminal,
        ReactiveDoWhileTerminal, ReactiveForInTerminal, ReactiveForOfTerminal, ReactiveForTerminal,
        ReactiveFunction, ReactiveIfTerminal, ReactiveInstruction, ReactiveInstructionStatement,
        ReactiveLabel, ReactiveLabelTerminal, ReactiveLogicalValue, ReactiveOptionalCallValue,
        ReactiveReturnTerminal, ReactiveSequenceValue, ReactiveStatement, ReactiveSwitchCase,
        ReactiveSwitchTerminal, ReactiveTerminal, ReactiveTerminalStatement,
        ReactiveTerminalTargetKind, ReactiveTernaryValue, ReactiveThrowTerminal,
        ReactiveTryTerminal, ReactiveValue, ReactiveWhileTerminal, Terminal,
    },
};

// =====================================================================================
// Context
// =====================================================================================

/// Context for the HIR -> Reactive conversion.
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

// =====================================================================================
// Value block result
// =====================================================================================

/// Result of visiting a value block: the final block id, the reactive value,
/// the place it is assigned to, and the instruction id.
struct ValueBlockResult {
    block: BlockId,
    value: ReactiveValue,
    place: Place,
    id: InstructionId,
}

/// Result of visiting a test block: the test value result plus the branch info.
struct TestBlockResult {
    test: ValueBlockResult,
    consequent: BlockId,
    alternate: BlockId,
    branch_loc: SourceLocation,
}

/// Result of visiting a value-block terminal.
struct ValueBlockTerminalResult {
    value: ReactiveValue,
    place: Place,
    fallthrough: BlockId,
    id: InstructionId,
}

// =====================================================================================
// Public entry point
// =====================================================================================

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

// =====================================================================================
// Helper: convert HIR Instruction to ReactiveInstruction
// =====================================================================================

fn hir_instr_to_reactive(instr: &Instruction) -> ReactiveInstruction {
    ReactiveInstruction {
        id: instr.id,
        lvalue: Some(instr.lvalue.clone()),
        value: ReactiveValue::Instruction(Box::new(instr.value.clone())),
        loc: instr.loc,
    }
}

// =====================================================================================
// Value block traversal
// =====================================================================================

/// Wraps a continuation result with preceding instructions. If there are no
/// instructions, returns the continuation as-is. Otherwise, wraps the continuation's
/// value in a SequenceExpression with the instructions prepended.
fn wrap_with_sequence(
    instructions: Vec<ReactiveInstruction>,
    continuation: ValueBlockResult,
    loc: SourceLocation,
) -> ValueBlockResult {
    if instructions.is_empty() {
        return continuation;
    }
    let sequence = ReactiveSequenceValue {
        instructions,
        id: continuation.id,
        value: Box::new(continuation.value),
        loc,
    };
    ValueBlockResult {
        block: continuation.block,
        value: ReactiveValue::Sequence(sequence),
        place: continuation.place,
        id: continuation.id,
    }
}

/// Extracts the result value from instructions at the end of a value block.
/// Value blocks generally end in a StoreLocal to assign the value of the
/// expression. These StoreLocal instructions can be pruned since we represent
/// value blocks as compound values in ReactiveFunction (no phis). However,
/// it's also possible to have a value block that ends in an AssignmentExpression,
/// which we need to keep. So we only prune StoreLocal for temporaries.
fn extract_value_block_result(
    instructions: &[Instruction],
    block_id: BlockId,
    loc: SourceLocation,
) -> ValueBlockResult {
    // The instructions list must be non-empty
    debug_assert!(
        !instructions.is_empty(),
        "Expected non-empty instructions in extract_value_block_result"
    );

    let instr = &instructions[instructions.len() - 1];
    let mut place = instr.lvalue.clone();
    let mut value = ReactiveValue::Instruction(Box::new(instr.value.clone()));

    // Special-case: if the last instruction is StoreLocal to a temporary (name is None),
    // extract the stored value instead.
    if let InstructionValue::StoreLocal(store) = &instr.value
        && store.lvalue.place.identifier.name.is_none()
    {
        place = store.lvalue.place.clone();
        value = ReactiveValue::Instruction(Box::new(InstructionValue::LoadLocal(
            crate::hir::LoadLocal { place: store.value.clone(), loc: store.value.loc },
        )));
    }

    if instructions.len() == 1 {
        return ValueBlockResult { block: block_id, place, value, id: instr.id };
    }

    // Wrap preceding instructions into a SequenceExpression
    let preceding: Vec<ReactiveInstruction> =
        instructions[..instructions.len() - 1].iter().map(hir_instr_to_reactive).collect();

    let sequence = ReactiveSequenceValue {
        instructions: preceding,
        id: instr.id,
        value: Box::new(value),
        loc,
    };
    ValueBlockResult {
        block: block_id,
        place,
        value: ReactiveValue::Sequence(sequence),
        id: instr.id,
    }
}

/// Visits a value block starting at `block_id` and returns the extracted reactive value.
///
/// A value block is a sub-graph of the CFG that represents an expression (e.g., the
/// test of a logical/ternary/optional expression). It walks through the blocks until
/// it reaches a branch or goto terminal, extracting the result.
///
/// `fallthrough` is an optional block id to stop at (used for sequence terminals).
fn visit_value_block(
    cx: &Context,
    block_id: BlockId,
    loc: SourceLocation,
    fallthrough: Option<BlockId>,
) -> ValueBlockResult {
    let Some(block) = cx.block(block_id) else {
        // Fallback: return an undefined value if the block is missing
        return ValueBlockResult {
            block: block_id,
            place: make_undefined_place(loc),
            value: make_undefined_value(loc),
            id: InstructionId::ZERO,
        };
    };

    // If we've reached the fallthrough block, this shouldn't happen
    if let Some(ft) = fallthrough
        && block_id == ft
    {
        // Invariant violation in TS; here we return undefined as fallback
        return ValueBlockResult {
            block: block_id,
            place: make_undefined_place(loc),
            value: make_undefined_value(loc),
            id: InstructionId::ZERO,
        };
    }

    match &block.terminal {
        Terminal::Branch(_) => {
            // A branch terminal means this block produces a value (for the test)
            // and then branches to consequent/alternate.
            if block.instructions.is_empty() {
                // The block has no instructions; the "value" is the test place itself.
                let Terminal::Branch(branch) = &block.terminal else { unreachable!() };
                return ValueBlockResult {
                    block: block.id,
                    place: branch.test.clone(),
                    value: ReactiveValue::Instruction(Box::new(InstructionValue::LoadLocal(
                        crate::hir::LoadLocal { place: branch.test.clone(), loc: branch.test.loc },
                    ))),
                    id: branch.id,
                };
            }
            extract_value_block_result(&block.instructions, block.id, loc)
        }
        Terminal::Goto(_) => {
            // A goto terminal: extract the value from the instructions
            if block.instructions.is_empty() {
                return ValueBlockResult {
                    block: block.id,
                    place: make_undefined_place(loc),
                    value: make_undefined_value(loc),
                    id: block.terminal.id(),
                };
            }
            extract_value_block_result(&block.instructions, block.id, loc)
        }
        Terminal::MaybeThrow(mt) => {
            // ReactiveFunction does not explicitly model maybe-throw semantics,
            // so maybe-throw terminals in value blocks flatten away.
            let continuation_id = mt.continuation;

            // Check if the continuation is just an empty goto (the common case where
            // a maybe-throw splits what would be a StoreLocal+goto).
            if let Some(cont_block) = cx.block(continuation_id)
                && cont_block.instructions.is_empty()
                && let Terminal::Goto(_) = &cont_block.terminal
            {
                return extract_value_block_result(&block.instructions, cont_block.id, loc);
            }

            let continuation = visit_value_block(cx, continuation_id, loc, fallthrough);
            let instructions: Vec<ReactiveInstruction> =
                block.instructions.iter().map(hir_instr_to_reactive).collect();
            wrap_with_sequence(instructions, continuation, loc)
        }
        // The value block ended in a value terminal (logical, ternary, optional, sequence).
        // Recurse to get the value of that terminal and stitch them together in a sequence.
        Terminal::Logical(_)
        | Terminal::Ternary(_)
        | Terminal::Optional(_)
        | Terminal::Sequence(_) => {
            let init = visit_value_block_terminal(cx, &block.terminal);
            let final_result = visit_value_block(cx, init.fallthrough, loc, fallthrough);

            let mut all_instructions: Vec<ReactiveInstruction> =
                block.instructions.iter().map(hir_instr_to_reactive).collect();
            all_instructions.push(ReactiveInstruction {
                id: init.id,
                loc,
                lvalue: Some(init.place),
                value: init.value,
            });

            wrap_with_sequence(all_instructions, final_result, loc)
        }
        _ => {
            // For other terminals (if, etc.), just extract from instructions
            if block.instructions.is_empty() {
                return ValueBlockResult {
                    block: block.id,
                    place: make_undefined_place(loc),
                    value: make_undefined_value(loc),
                    id: block.terminal.id(),
                };
            }
            extract_value_block_result(&block.instructions, block.id, loc)
        }
    }
}

/// Visits the test block of a value terminal (optional, logical, ternary) and
/// returns the result along with the branch terminal. Returns None if the test
/// block does not end in a branch terminal (unexpected).
fn visit_test_block(
    cx: &Context,
    test_block_id: BlockId,
    loc: SourceLocation,
) -> Option<TestBlockResult> {
    let test = visit_value_block(cx, test_block_id, loc, None);
    let test_block = cx.block(test.block)?;

    if let Terminal::Branch(branch) = &test_block.terminal {
        Some(TestBlockResult {
            test,
            consequent: branch.consequent,
            alternate: branch.alternate,
            branch_loc: branch.loc,
        })
    } else {
        // The TS code throws a Todo error here. We'll return None.
        None
    }
}

/// Visits a value-block terminal (logical, ternary, optional, sequence) and returns
/// the reactive value it produces.
fn visit_value_block_terminal(cx: &Context, terminal: &Terminal) -> ValueBlockTerminalResult {
    match terminal {
        Terminal::Sequence(seq) => {
            let block_result = visit_value_block(cx, seq.block, seq.loc, Some(seq.fallthrough));
            ValueBlockTerminalResult {
                value: block_result.value,
                place: block_result.place,
                fallthrough: seq.fallthrough,
                id: seq.id,
            }
        }
        Terminal::Optional(optional) => {
            // Visit the test block to get test value and branch info
            if let Some(test_result) = visit_test_block(cx, optional.test, optional.loc) {
                let consequent = visit_value_block(
                    cx,
                    test_result.consequent,
                    optional.loc,
                    Some(optional.fallthrough),
                );

                // Build a SequenceExpression wrapping the test + consequent
                let call = ReactiveSequenceValue {
                    instructions: vec![ReactiveInstruction {
                        id: test_result.test.id,
                        loc: test_result.branch_loc,
                        lvalue: Some(test_result.test.place),
                        value: test_result.test.value,
                    }],
                    id: consequent.id,
                    value: Box::new(consequent.value),
                    loc: optional.loc,
                };

                let place = consequent.place;
                ValueBlockTerminalResult {
                    place,
                    value: ReactiveValue::OptionalCall(ReactiveOptionalCallValue {
                        id: optional.id,
                        value: Box::new(ReactiveValue::Sequence(call)),
                        optional: optional.optional,
                        loc: optional.loc,
                    }),
                    fallthrough: optional.fallthrough,
                    id: optional.id,
                }
            } else {
                // Fallback if test block doesn't end in branch
                ValueBlockTerminalResult {
                    value: make_undefined_value(optional.loc),
                    place: make_undefined_place(optional.loc),
                    fallthrough: optional.fallthrough,
                    id: optional.id,
                }
            }
        }
        Terminal::Logical(logical) => {
            if let Some(test_result) = visit_test_block(cx, logical.test, logical.loc) {
                // The left (consequent) branch is the "short-circuit" path
                let left_final = visit_value_block(
                    cx,
                    test_result.consequent,
                    logical.loc,
                    Some(logical.fallthrough),
                );

                // Build SequenceExpression for the left side (test + left continuation)
                let left = ReactiveSequenceValue {
                    instructions: vec![ReactiveInstruction {
                        id: test_result.test.id,
                        loc: logical.loc,
                        lvalue: Some(test_result.test.place),
                        value: test_result.test.value,
                    }],
                    id: left_final.id,
                    value: Box::new(left_final.value),
                    loc: logical.loc,
                };

                // The right (alternate) branch is the "non-short-circuit" path
                let right = visit_value_block(
                    cx,
                    test_result.alternate,
                    logical.loc,
                    Some(logical.fallthrough),
                );

                let place = left_final.place;
                let value = ReactiveLogicalValue {
                    operator: logical.operator,
                    left: Box::new(ReactiveValue::Sequence(left)),
                    right: Box::new(right.value),
                    loc: logical.loc,
                };

                ValueBlockTerminalResult {
                    place,
                    value: ReactiveValue::Logical(value),
                    fallthrough: logical.fallthrough,
                    id: logical.id,
                }
            } else {
                ValueBlockTerminalResult {
                    value: make_undefined_value(logical.loc),
                    place: make_undefined_place(logical.loc),
                    fallthrough: logical.fallthrough,
                    id: logical.id,
                }
            }
        }
        Terminal::Ternary(ternary) => {
            if let Some(test_result) = visit_test_block(cx, ternary.test, ternary.loc) {
                let consequent = visit_value_block(
                    cx,
                    test_result.consequent,
                    ternary.loc,
                    Some(ternary.fallthrough),
                );
                let alternate = visit_value_block(
                    cx,
                    test_result.alternate,
                    ternary.loc,
                    Some(ternary.fallthrough),
                );

                let place = consequent.place;
                let value = ReactiveTernaryValue {
                    test: Box::new(test_result.test.value),
                    consequent: Box::new(consequent.value),
                    alternate: Box::new(alternate.value),
                    loc: ternary.loc,
                };

                ValueBlockTerminalResult {
                    place,
                    value: ReactiveValue::Ternary(value),
                    fallthrough: ternary.fallthrough,
                    id: ternary.id,
                }
            } else {
                ValueBlockTerminalResult {
                    value: make_undefined_value(ternary.loc),
                    place: make_undefined_place(ternary.loc),
                    fallthrough: ternary.fallthrough,
                    id: ternary.id,
                }
            }
        }
        _ => {
            // Unsupported terminal in value block context
            let loc = terminal.loc();
            let id = terminal.id();
            let fallthrough = terminal.fallthrough().unwrap_or(BlockId(0));
            ValueBlockTerminalResult {
                value: make_undefined_value(loc),
                place: make_undefined_place(loc),
                fallthrough,
                id,
            }
        }
    }
}

/// Converts the result of visit_value_block into a SequenceExpression that includes
/// the instruction with its lvalue. This is needed for for/for-of/for-in init/test
/// blocks where the instruction's lvalue assignment must be preserved.
fn value_block_result_to_sequence(
    result: ValueBlockResult,
    loc: SourceLocation,
) -> ReactiveSequenceValue {
    let mut instructions: Vec<ReactiveInstruction> = Vec::new();
    let mut inner_value = result.value;

    // Flatten nested SequenceExpressions
    while let ReactiveValue::Sequence(seq) = inner_value {
        instructions.extend(seq.instructions);
        inner_value = *seq.value;
    }

    // Only add the final instruction if the innermost value is not just a LoadLocal
    // of the same place we're storing to (which would be a no-op).
    let is_load_of_same_place = if let ReactiveValue::Instruction(ref iv) = inner_value {
        if let InstructionValue::LoadLocal(ref ll) = **iv {
            ll.place.identifier.id == result.place.identifier.id
        } else {
            false
        }
    } else {
        false
    };

    if !is_load_of_same_place {
        instructions.push(ReactiveInstruction {
            id: result.id,
            lvalue: Some(result.place.clone()),
            value: inner_value,
            loc,
        });
    }

    ReactiveSequenceValue {
        instructions,
        id: result.id,
        value: Box::new(make_undefined_value(loc)),
        loc,
    }
}

// =====================================================================================
// Helper: make undefined value / place
// =====================================================================================

fn make_undefined_value(loc: SourceLocation) -> ReactiveValue {
    ReactiveValue::Instruction(Box::new(InstructionValue::Primitive(crate::hir::PrimitiveValue {
        value: crate::hir::PrimitiveValueKind::Undefined,
        loc,
    })))
}

fn make_undefined_place(loc: SourceLocation) -> Place {
    Place {
        identifier: crate::hir::Identifier {
            id: crate::hir::IdentifierId(0),
            declaration_id: crate::hir::DeclarationId(0),
            name: None,
            mutable_range: crate::hir::MutableRange::default(),
            scope: None,
            type_: crate::hir::types::Type::Primitive,
            loc,
        },
        effect: crate::hir::Effect::Read,
        reactive: false,
        loc,
    }
}

// =====================================================================================
// Main block traversal
// =====================================================================================

/// Traverse a block and its successors, building a reactive block.
fn traverse_block(cx: &Context, block_id: BlockId) -> ReactiveBlock {
    let mut statements: ReactiveBlock = Vec::new();
    let mut current_id = block_id;

    while let Some(block) = cx.block(current_id) {
        // Add instructions as reactive statements
        for instr in &block.instructions {
            statements.push(ReactiveStatement::Instruction(ReactiveInstructionStatement {
                instruction: hir_instr_to_reactive(instr),
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
                    GotoVariant::Break | GotoVariant::Continue => {
                        ReactiveTerminalTargetKind::Unlabeled
                    }
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
                statements.push(ReactiveStatement::Terminal(Box::new(ReactiveTerminalStatement {
                    terminal: ReactiveTerminal::Return(ReactiveReturnTerminal {
                        value: ret.value.clone(),
                        id: ret.id,
                        loc: ret.loc,
                    }),
                    label: None,
                })));
                break;
            }
            Terminal::Throw(throw) => {
                statements.push(ReactiveStatement::Terminal(Box::new(ReactiveTerminalStatement {
                    terminal: ReactiveTerminal::Throw(ReactiveThrowTerminal {
                        value: throw.value.clone(),
                        id: throw.id,
                        loc: throw.loc,
                    }),
                    label: None,
                })));
                break;
            }
            Terminal::If(if_term) => {
                let consequent = traverse_block(cx, if_term.consequent);
                let alternate = traverse_block(cx, if_term.alternate);
                let alternate_opt = if alternate.is_empty() { None } else { Some(alternate) };

                statements.push(ReactiveStatement::Terminal(Box::new(ReactiveTerminalStatement {
                    terminal: ReactiveTerminal::If(Box::new(ReactiveIfTerminal {
                        test: if_term.test.clone(),
                        consequent,
                        alternate: alternate_opt,
                        id: if_term.id,
                        loc: if_term.loc,
                    })),
                    label: Some(ReactiveLabel { id: if_term.fallthrough, implicit: true }),
                })));

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

                statements.push(ReactiveStatement::Terminal(Box::new(ReactiveTerminalStatement {
                    terminal: ReactiveTerminal::Switch(Box::new(ReactiveSwitchTerminal {
                        test: switch_term.test.clone(),
                        cases,
                        id: switch_term.id,
                        loc: switch_term.loc,
                    })),
                    label: Some(ReactiveLabel { id: switch_term.fallthrough, implicit: true }),
                })));

                current_id = switch_term.fallthrough;
            }
            Terminal::While(while_term) => {
                let test_result = visit_value_block(cx, while_term.test, while_term.loc, None);
                let loop_body = traverse_block(cx, while_term.r#loop);

                statements.push(ReactiveStatement::Terminal(Box::new(ReactiveTerminalStatement {
                    terminal: ReactiveTerminal::While(Box::new(ReactiveWhileTerminal {
                        test: test_result.value,
                        r#loop: loop_body,
                        id: while_term.id,
                        loc: while_term.loc,
                    })),
                    label: Some(ReactiveLabel { id: while_term.fallthrough, implicit: true }),
                })));

                current_id = while_term.fallthrough;
            }
            Terminal::DoWhile(do_while) => {
                let loop_body = traverse_block(cx, do_while.r#loop);
                let test_result = visit_value_block(cx, do_while.test, do_while.loc, None);

                statements.push(ReactiveStatement::Terminal(Box::new(ReactiveTerminalStatement {
                    terminal: ReactiveTerminal::DoWhile(Box::new(ReactiveDoWhileTerminal {
                        r#loop: loop_body,
                        test: test_result.value,
                        id: do_while.id,
                        loc: do_while.loc,
                    })),
                    label: Some(ReactiveLabel { id: do_while.fallthrough, implicit: true }),
                })));

                current_id = do_while.fallthrough;
            }
            Terminal::For(for_term) => {
                let init_result = visit_value_block(cx, for_term.init, for_term.loc, None);
                let init_value = ReactiveValue::Sequence(value_block_result_to_sequence(
                    init_result,
                    for_term.loc,
                ));
                let test_result = visit_value_block(cx, for_term.test, for_term.loc, None);
                let update = for_term
                    .update
                    .map(|update_id| visit_value_block(cx, update_id, for_term.loc, None).value);
                let loop_body = traverse_block(cx, for_term.r#loop);

                statements.push(ReactiveStatement::Terminal(Box::new(ReactiveTerminalStatement {
                    terminal: ReactiveTerminal::For(Box::new(ReactiveForTerminal {
                        init: init_value,
                        test: test_result.value,
                        update,
                        r#loop: loop_body,
                        id: for_term.id,
                        loc: for_term.loc,
                    })),
                    label: Some(ReactiveLabel { id: for_term.fallthrough, implicit: true }),
                })));

                current_id = for_term.fallthrough;
            }
            Terminal::ForOf(for_of) => {
                let init_result = visit_value_block(cx, for_of.init, for_of.loc, None);
                let init_value = ReactiveValue::Sequence(value_block_result_to_sequence(
                    init_result,
                    for_of.loc,
                ));
                let test_result = visit_value_block(cx, for_of.test, for_of.loc, None);
                let test_value = ReactiveValue::Sequence(value_block_result_to_sequence(
                    test_result,
                    for_of.loc,
                ));
                let loop_body = traverse_block(cx, for_of.r#loop);

                statements.push(ReactiveStatement::Terminal(Box::new(ReactiveTerminalStatement {
                    terminal: ReactiveTerminal::ForOf(Box::new(ReactiveForOfTerminal {
                        init: init_value,
                        test: test_value,
                        r#loop: loop_body,
                        id: for_of.id,
                        loc: for_of.loc,
                    })),
                    label: Some(ReactiveLabel { id: for_of.fallthrough, implicit: true }),
                })));

                current_id = for_of.fallthrough;
            }
            Terminal::ForIn(for_in) => {
                let init_result = visit_value_block(cx, for_in.init, for_in.loc, None);
                let init_value = ReactiveValue::Sequence(value_block_result_to_sequence(
                    init_result,
                    for_in.loc,
                ));
                let loop_body = traverse_block(cx, for_in.r#loop);

                statements.push(ReactiveStatement::Terminal(Box::new(ReactiveTerminalStatement {
                    terminal: ReactiveTerminal::ForIn(Box::new(ReactiveForInTerminal {
                        init: init_value,
                        r#loop: loop_body,
                        id: for_in.id,
                        loc: for_in.loc,
                    })),
                    label: Some(ReactiveLabel { id: for_in.fallthrough, implicit: true }),
                })));

                current_id = for_in.fallthrough;
            }
            Terminal::Label(label) => {
                let block_body = traverse_block(cx, label.block);

                statements.push(ReactiveStatement::Terminal(Box::new(ReactiveTerminalStatement {
                    terminal: ReactiveTerminal::Label(Box::new(ReactiveLabelTerminal {
                        block: block_body,
                        id: label.id,
                        loc: label.loc,
                    })),
                    label: Some(ReactiveLabel { id: label.fallthrough, implicit: false }),
                })));

                current_id = label.fallthrough;
            }
            Terminal::Try(try_term) => {
                let block_body = traverse_block(cx, try_term.block);
                let handler_body = traverse_block(cx, try_term.handler);

                statements.push(ReactiveStatement::Terminal(Box::new(ReactiveTerminalStatement {
                    terminal: ReactiveTerminal::Try(Box::new(ReactiveTryTerminal {
                        block: block_body,
                        handler_binding: try_term.handler_binding.clone(),
                        handler: handler_body,
                        id: try_term.id,
                        loc: try_term.loc,
                    })),
                    label: Some(ReactiveLabel { id: try_term.fallthrough, implicit: true }),
                })));

                current_id = try_term.fallthrough;
            }
            Terminal::MaybeThrow(mt) => {
                // Continue to the continuation block
                current_id = mt.continuation;
            }
            Terminal::Scope(scope) => {
                let block_body = traverse_block(cx, scope.block);
                statements.push(ReactiveStatement::Scope(crate::hir::ReactiveScopeBlock {
                    scope: scope.scope.clone(),
                    instructions: block_body,
                }));
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
            // Value-block terminals: Logical, Ternary, Optional, Sequence
            // These produce expression-level values and are emitted as instructions,
            // matching the TS `visitValueBlockTerminal` behavior.
            Terminal::Sequence(seq) => {
                let result = visit_value_block_terminal(cx, &block.terminal);

                statements.push(ReactiveStatement::Instruction(ReactiveInstructionStatement {
                    instruction: ReactiveInstruction {
                        id: seq.id,
                        lvalue: Some(result.place),
                        value: result.value,
                        loc: seq.loc,
                    },
                }));

                current_id = result.fallthrough;
            }
            Terminal::Logical(logical) => {
                let result = visit_value_block_terminal(cx, &block.terminal);

                statements.push(ReactiveStatement::Instruction(ReactiveInstructionStatement {
                    instruction: ReactiveInstruction {
                        id: logical.id,
                        lvalue: Some(result.place),
                        value: result.value,
                        loc: logical.loc,
                    },
                }));

                current_id = result.fallthrough;
            }
            Terminal::Ternary(ternary) => {
                let result = visit_value_block_terminal(cx, &block.terminal);

                statements.push(ReactiveStatement::Instruction(ReactiveInstructionStatement {
                    instruction: ReactiveInstruction {
                        id: ternary.id,
                        lvalue: Some(result.place),
                        value: result.value,
                        loc: ternary.loc,
                    },
                }));

                current_id = result.fallthrough;
            }
            Terminal::Optional(optional) => {
                let result = visit_value_block_terminal(cx, &block.terminal);

                statements.push(ReactiveStatement::Instruction(ReactiveInstructionStatement {
                    instruction: ReactiveInstruction {
                        id: optional.id,
                        lvalue: Some(result.place),
                        value: result.value,
                        loc: optional.loc,
                    },
                }));

                current_id = result.fallthrough;
            }
            Terminal::Branch(branch) => {
                let consequent = traverse_block(cx, branch.consequent);
                let alternate = traverse_block(cx, branch.alternate);
                let alternate_opt = if alternate.is_empty() { None } else { Some(alternate) };

                statements.push(ReactiveStatement::Terminal(Box::new(ReactiveTerminalStatement {
                    terminal: ReactiveTerminal::If(Box::new(ReactiveIfTerminal {
                        test: branch.test.clone(),
                        consequent,
                        alternate: alternate_opt,
                        id: branch.id,
                        loc: branch.loc,
                    })),
                    label: Some(ReactiveLabel { id: branch.fallthrough, implicit: true }),
                })));

                current_id = branch.fallthrough;
            }
        }
    }

    statements
}
