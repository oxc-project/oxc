/// Build a reactive function from the HIR control-flow graph.
///
/// Port of `ReactiveScopes/BuildReactiveFunction.ts` from the React Compiler.
///
/// Converts from HIR (lower-level CFG) to ReactiveFunction, a tree representation
/// that is closer to an AST. This pass restores the original control flow constructs,
/// including break/continue to labeled statements.
use rustc_hash::FxHashSet;

use crate::{
    compiler_error::{CompilerError, SourceLocation},
    hir::{
        BasicBlock, BlockId, BlockMap, GotoVariant, HIRFunction, Hir, Instruction, InstructionId,
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
// Control flow target (matches TS ControlFlowTarget)
// =====================================================================================

#[derive(Debug)]
enum ControlFlowTarget {
    If {
        block: BlockId,
        id: u32,
    },
    Switch {
        block: BlockId,
        id: u32,
    },
    Case {
        block: BlockId,
        id: u32,
    },
    Loop {
        block: BlockId,
        continue_block: BlockId,
        loop_block: Option<BlockId>,
        owns_loop: bool,
        id: u32,
    },
}

impl ControlFlowTarget {
    fn block(&self) -> BlockId {
        match self {
            ControlFlowTarget::If { block, .. }
            | ControlFlowTarget::Switch { block, .. }
            | ControlFlowTarget::Case { block, .. }
            | ControlFlowTarget::Loop { block, .. } => *block,
        }
    }

    fn id(&self) -> u32 {
        match self {
            ControlFlowTarget::If { id, .. }
            | ControlFlowTarget::Switch { id, .. }
            | ControlFlowTarget::Case { id, .. }
            | ControlFlowTarget::Loop { id, .. } => *id,
        }
    }

    fn is_loop(&self) -> bool {
        matches!(self, ControlFlowTarget::Loop { .. })
    }
}

// =====================================================================================
// Context
// =====================================================================================

/// Context for the HIR -> Reactive conversion.
struct Context {
    blocks: BlockMap,
    next_schedule_id: u32,
    /// Used to track which blocks *have been* generated already in order to
    /// abort if a block is generated a second time. This is an error catching
    /// mechanism for debugging purposes.
    emitted: FxHashSet<BlockId>,
    scope_fallthroughs: FxHashSet<BlockId>,
    /// A set of blocks that are already scheduled to be emitted by eg a parent.
    /// This allows child nodes to avoid re-emitting the same block and emit eg
    /// a break instead.
    scheduled: FxHashSet<BlockId>,
    catch_handlers: FxHashSet<BlockId>,
    /// Represents which control flow operations are currently in scope, with the innermost
    /// scope last. The last ControlFlowTarget on the stack indicates where control will
    /// implicitly transfer.
    control_flow_stack: Vec<ControlFlowTarget>,
}

impl Context {
    fn new(hir: &Hir) -> Self {
        Self {
            blocks: hir.blocks.clone(),
            next_schedule_id: 0,
            emitted: FxHashSet::default(),
            scope_fallthroughs: FxHashSet::default(),
            scheduled: FxHashSet::default(),
            catch_handlers: FxHashSet::default(),
            control_flow_stack: Vec::new(),
        }
    }

    fn block(&self, id: BlockId) -> Option<&BasicBlock> {
        self.blocks.get(&id)
    }

    fn reachable(&self, id: BlockId) -> bool {
        if let Some(block) = self.blocks.get(&id) {
            !matches!(block.terminal, Terminal::Unreachable(_))
        } else {
            false
        }
    }

    fn schedule_catch_handler(&mut self, block: BlockId) {
        self.catch_handlers.insert(block);
    }

    /// Record that the given block will be emitted (eg by the codegen of a parent node)
    /// so that child nodes can avoid re-emitting it.
    fn schedule(&mut self, block: BlockId, target_type: &str) -> u32 {
        let id = self.next_schedule_id;
        self.next_schedule_id += 1;
        debug_assert!(
            !self.scheduled.contains(&block),
            "Break block is already scheduled: bb{block:?}"
        );
        self.scheduled.insert(block);
        let target = match target_type {
            "switch" => ControlFlowTarget::Switch { block, id },
            "case" => ControlFlowTarget::Case { block, id },
            // "if" and any other type
            _ => ControlFlowTarget::If { block, id },
        };
        self.control_flow_stack.push(target);
        id
    }

    fn schedule_loop(
        &mut self,
        fallthrough_block: BlockId,
        continue_block: BlockId,
        loop_block: Option<BlockId>,
    ) -> u32 {
        let id = self.next_schedule_id;
        self.next_schedule_id += 1;
        self.scheduled.insert(fallthrough_block);
        debug_assert!(
            !self.scheduled.contains(&continue_block),
            "Continue block is already scheduled: bb{continue_block:?}"
        );
        self.scheduled.insert(continue_block);
        let mut owns_loop = false;
        if let Some(lb) = loop_block {
            owns_loop = !self.scheduled.contains(&lb);
            self.scheduled.insert(lb);
        }
        self.control_flow_stack.push(ControlFlowTarget::Loop {
            block: fallthrough_block,
            continue_block,
            loop_block,
            owns_loop,
            id,
        });
        id
    }

    /// Removes a block that was scheduled; must be called after that block is emitted.
    fn unschedule(&mut self, schedule_id: u32) {
        let last = self.control_flow_stack.pop();
        debug_assert!(
            last.as_ref().is_some_and(|t| t.id() == schedule_id),
            "Can only unschedule the last target"
        );
        if let Some(target) = last {
            match &target {
                ControlFlowTarget::Loop {
                    block, continue_block, loop_block, owns_loop, ..
                } => {
                    // Always remove the fallthrough block from scheduled.
                    // In the TS reference, the condition is `last.ownsBlock !== null`
                    // which is always true since ownsBlock is a boolean (never null),
                    // so the block is always removed from scheduled regardless of ownsBlock.
                    self.scheduled.remove(block);
                    self.scheduled.remove(continue_block);
                    if *owns_loop && let Some(lb) = loop_block {
                        self.scheduled.remove(lb);
                    }
                }
                _ => {
                    self.scheduled.remove(&target.block());
                }
            }
        }
    }

    /// Helper to unschedule multiple scheduled blocks. The ids should be in
    /// the order in which they were scheduled, ie most recently scheduled last.
    fn unschedule_all(&mut self, schedule_ids: &[u32]) {
        for &id in schedule_ids.iter().rev() {
            self.unschedule(id);
        }
    }

    /// Check if the given block is scheduled or not.
    fn is_scheduled(&self, block: BlockId) -> bool {
        self.scheduled.contains(&block) || self.catch_handlers.contains(&block)
    }

    /// Given the current control flow stack, determines how a `break` to the given block
    /// must be emitted.
    fn get_break_target(&self, block: BlockId) -> Option<(BlockId, ReactiveTerminalTargetKind)> {
        let mut has_preceding_loop = false;
        for i in (0..self.control_flow_stack.len()).rev() {
            let target = &self.control_flow_stack[i];
            if target.block() == block {
                let kind = if target.is_loop() {
                    // breaking out of a loop requires an explicit break,
                    // but only requires a label if breaking past the innermost loop.
                    if has_preceding_loop {
                        ReactiveTerminalTargetKind::Labeled
                    } else {
                        ReactiveTerminalTargetKind::Unlabeled
                    }
                } else if i == self.control_flow_stack.len() - 1 {
                    // breaking to the last break point, which is where control will transfer
                    // implicitly
                    ReactiveTerminalTargetKind::Implicit
                } else {
                    // breaking somewhere else requires an explicit break
                    ReactiveTerminalTargetKind::Labeled
                };
                return Some((target.block(), kind));
            }
            if target.is_loop() {
                has_preceding_loop = true;
            }
        }
        None
    }

    /// Given the current control flow stack, determines how a `continue` to the given block
    /// must be emitted.
    fn get_continue_target(&self, block: BlockId) -> Option<(BlockId, ReactiveTerminalTargetKind)> {
        let mut has_preceding_loop = false;
        for i in (0..self.control_flow_stack.len()).rev() {
            let target = &self.control_flow_stack[i];
            if let ControlFlowTarget::Loop { continue_block, block: fallthrough_block, .. } = target
                && *continue_block == block
            {
                let kind = if has_preceding_loop {
                    // continuing to a loop that is not the innermost loop always requires
                    // a label
                    ReactiveTerminalTargetKind::Labeled
                } else if i == self.control_flow_stack.len() - 1 {
                    // continuing to the last break point, which is where control will
                    // transfer to naturally
                    ReactiveTerminalTargetKind::Implicit
                } else {
                    // the continue is inside some conditional logic, requires an explicit
                    // continue
                    ReactiveTerminalTargetKind::Unlabeled
                };
                return Some((*fallthrough_block, kind));
            }
            if target.is_loop() {
                has_preceding_loop = true;
            }
        }
        None
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
///
/// # Errors
/// Returns a `CompilerError` if the HIR contains unsupported terminal types
/// or invariant violations during the conversion.
pub fn build_reactive_function(func: &HIRFunction) -> Result<ReactiveFunction, CompilerError> {
    let mut cx = Context::new(&func.body);
    let entry_block = cx.block(func.body.entry).cloned();
    let body =
        if let Some(block) = entry_block { traverse_block(&mut cx, block)? } else { Vec::new() };

    Ok(ReactiveFunction {
        loc: func.loc,
        id: func.id.clone(),
        name_hint: func.name_hint.clone(),
        params: func.params.clone(),
        generator: func.generator,
        is_async: func.is_async,
        body,
        directives: func.directives.clone(),
    })
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
) -> Result<ValueBlockResult, CompilerError> {
    let Some(block) = cx.block(block_id) else {
        // Fallback: return an undefined value if the block is missing
        return Ok(ValueBlockResult {
            block: block_id,
            place: make_undefined_place(loc),
            value: make_undefined_value(loc),
            id: InstructionId::ZERO,
        });
    };

    // If we've reached the fallthrough block, this shouldn't happen
    if let Some(ft) = fallthrough
        && block_id == ft
    {
        // Invariant violation in TS; here we return undefined as fallback
        return Ok(ValueBlockResult {
            block: block_id,
            place: make_undefined_place(loc),
            value: make_undefined_value(loc),
            id: InstructionId::ZERO,
        });
    }

    match &block.terminal {
        Terminal::Branch(_) => {
            // A branch terminal means this block produces a value (for the test)
            // and then branches to consequent/alternate.
            if block.instructions.is_empty() {
                // The block has no instructions; the "value" is the test place itself.
                let Terminal::Branch(branch) = &block.terminal else { unreachable!() };
                return Ok(ValueBlockResult {
                    block: block.id,
                    place: branch.test.clone(),
                    value: ReactiveValue::Instruction(Box::new(InstructionValue::LoadLocal(
                        crate::hir::LoadLocal { place: branch.test.clone(), loc: branch.test.loc },
                    ))),
                    id: branch.id,
                });
            }
            Ok(extract_value_block_result(&block.instructions, block.id, loc))
        }
        Terminal::Goto(_) => {
            // A goto terminal: extract the value from the instructions
            if block.instructions.is_empty() {
                return Ok(ValueBlockResult {
                    block: block.id,
                    place: make_undefined_place(loc),
                    value: make_undefined_value(loc),
                    id: block.terminal.id(),
                });
            }
            Ok(extract_value_block_result(&block.instructions, block.id, loc))
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
                return Ok(extract_value_block_result(&block.instructions, cont_block.id, loc));
            }

            let continuation = visit_value_block(cx, continuation_id, loc, fallthrough)?;
            let instructions: Vec<ReactiveInstruction> =
                block.instructions.iter().map(hir_instr_to_reactive).collect();
            Ok(wrap_with_sequence(instructions, continuation, loc))
        }
        // The value block ended in a value terminal (logical, ternary, optional, sequence).
        // Recurse to get the value of that terminal and stitch them together in a sequence.
        Terminal::Logical(_)
        | Terminal::Ternary(_)
        | Terminal::Optional(_)
        | Terminal::Sequence(_) => {
            let init = visit_value_block_terminal(cx, &block.terminal)?;
            let final_result = visit_value_block(cx, init.fallthrough, loc, fallthrough)?;

            let mut all_instructions: Vec<ReactiveInstruction> =
                block.instructions.iter().map(hir_instr_to_reactive).collect();
            all_instructions.push(ReactiveInstruction {
                id: init.id,
                loc,
                lvalue: Some(init.place),
                value: init.value,
            });

            Ok(wrap_with_sequence(all_instructions, final_result, loc))
        }
        // Scope and PrunedScope terminals can appear inside value blocks when the
        // reactive scope inference inserts scope boundaries into the middle of a
        // value block chain (e.g., inside the test block of an optional chain).
        // We "see through" the scope by recursing into the scope's inner block,
        // then wrapping any preceding instructions from the current block.
        Terminal::Scope(scope) => {
            let inner_block_id = scope.block;
            let continuation = visit_value_block(cx, inner_block_id, loc, fallthrough)?;
            let instructions: Vec<ReactiveInstruction> =
                block.instructions.iter().map(hir_instr_to_reactive).collect();
            Ok(wrap_with_sequence(instructions, continuation, loc))
        }
        Terminal::PrunedScope(scope) => {
            let inner_block_id = scope.block;
            let continuation = visit_value_block(cx, inner_block_id, loc, fallthrough)?;
            let instructions: Vec<ReactiveInstruction> =
                block.instructions.iter().map(hir_instr_to_reactive).collect();
            Ok(wrap_with_sequence(instructions, continuation, loc))
        }
        _ => {
            // For other terminals (if, etc.), just extract from instructions
            if block.instructions.is_empty() {
                return Ok(ValueBlockResult {
                    block: block.id,
                    place: make_undefined_place(loc),
                    value: make_undefined_value(loc),
                    id: block.terminal.id(),
                });
            }
            Ok(extract_value_block_result(&block.instructions, block.id, loc))
        }
    }
}

/// Visits the test block of a value terminal (optional, logical, ternary) and
/// returns the result along with the branch terminal.
///
/// # Errors
/// Returns a `CompilerError` if the test block does not end in a branch terminal.
fn visit_test_block(
    cx: &Context,
    test_block_id: BlockId,
    loc: SourceLocation,
) -> Result<TestBlockResult, CompilerError> {
    let test = visit_value_block(cx, test_block_id, loc, None)?;
    let test_block = cx
        .block(test.block)
        .ok_or_else(|| CompilerError::invariant("Expected test block to exist", None, loc))?;

    if let Terminal::Branch(branch) = &test_block.terminal {
        Ok(TestBlockResult {
            test,
            consequent: branch.consequent,
            alternate: branch.alternate,
            branch_loc: branch.loc,
        })
    } else {
        Err(CompilerError::todo(
            "Unexpected terminal kind for test block",
            None,
            test_block.terminal.loc(),
        ))
    }
}

/// Visits a value-block terminal (logical, ternary, optional, sequence) and returns
/// the reactive value it produces.
fn visit_value_block_terminal(
    cx: &Context,
    terminal: &Terminal,
) -> Result<ValueBlockTerminalResult, CompilerError> {
    match terminal {
        Terminal::Sequence(seq) => {
            let block_result = visit_value_block(cx, seq.block, seq.loc, Some(seq.fallthrough))?;
            Ok(ValueBlockTerminalResult {
                value: block_result.value,
                place: block_result.place,
                fallthrough: seq.fallthrough,
                id: seq.id,
            })
        }
        Terminal::Optional(optional) => {
            let test_result = visit_test_block(cx, optional.test, optional.loc)?;
            let consequent = visit_value_block(
                cx,
                test_result.consequent,
                optional.loc,
                Some(optional.fallthrough),
            )?;

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
            Ok(ValueBlockTerminalResult {
                place,
                value: ReactiveValue::OptionalCall(ReactiveOptionalCallValue {
                    id: optional.id,
                    value: Box::new(ReactiveValue::Sequence(call)),
                    optional: optional.optional,
                    loc: optional.loc,
                }),
                fallthrough: optional.fallthrough,
                id: optional.id,
            })
        }
        Terminal::Logical(logical) => {
            let test_result = visit_test_block(cx, logical.test, logical.loc)?;
            // The left (consequent) branch is the "short-circuit" path
            let left_final = visit_value_block(
                cx,
                test_result.consequent,
                logical.loc,
                Some(logical.fallthrough),
            )?;

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
            )?;

            let place = left_final.place;
            let value = ReactiveLogicalValue {
                operator: logical.operator,
                left: Box::new(ReactiveValue::Sequence(left)),
                right: Box::new(right.value),
                loc: logical.loc,
            };

            Ok(ValueBlockTerminalResult {
                place,
                value: ReactiveValue::Logical(value),
                fallthrough: logical.fallthrough,
                id: logical.id,
            })
        }
        Terminal::Ternary(ternary) => {
            let test_result = visit_test_block(cx, ternary.test, ternary.loc)?;
            let consequent = visit_value_block(
                cx,
                test_result.consequent,
                ternary.loc,
                Some(ternary.fallthrough),
            )?;
            let alternate = visit_value_block(
                cx,
                test_result.alternate,
                ternary.loc,
                Some(ternary.fallthrough),
            )?;

            let place = consequent.place;
            let value = ReactiveTernaryValue {
                test: Box::new(test_result.test.value),
                consequent: Box::new(consequent.value),
                alternate: Box::new(alternate.value),
                loc: ternary.loc,
            };

            Ok(ValueBlockTerminalResult {
                place,
                value: ReactiveValue::Ternary(value),
                fallthrough: ternary.fallthrough,
                id: ternary.id,
            })
        }
        Terminal::MaybeThrow(_) => Err(CompilerError::invariant(
            "Unexpected maybe-throw in visit_value_block_terminal - should be handled in visit_value_block",
            None,
            terminal.loc(),
        )),
        Terminal::Label(_) => Err(CompilerError::todo(
            "Support labeled statements combined with value blocks (conditional, logical, optional chaining, etc)",
            None,
            terminal.loc(),
        )),
        _ => Err(CompilerError::todo(
            "Unsupported terminal as a value block terminal (conditional, logical, optional chaining, etc)",
            None,
            terminal.loc(),
        )),
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
// Break / Continue visitors
// =====================================================================================

/// Visits a goto-break to a scheduled block and returns the appropriate
/// reactive break terminal statement, or None if the break is implicit
/// to a scope fallthrough.
fn visit_break(
    cx: &Context,
    block: BlockId,
    id: InstructionId,
    loc: SourceLocation,
) -> Result<Option<ReactiveStatement>, CompilerError> {
    // If the target is a scope fallthrough, the break is implicit —
    // control will naturally fall through to the next block. Suppress it.
    if cx.scope_fallthroughs.contains(&block) {
        return Ok(None);
    }
    let (target_block, target_kind) = cx
        .get_break_target(block)
        .ok_or_else(|| CompilerError::invariant("Expected a break target", None, loc))?;
    if cx.scope_fallthroughs.contains(&target_block) {
        debug_assert!(
            target_kind == ReactiveTerminalTargetKind::Implicit,
            "Expected reactive scope to implicitly break to fallthrough"
        );
        return Ok(None);
    }
    Ok(Some(ReactiveStatement::Terminal(Box::new(ReactiveTerminalStatement {
        terminal: ReactiveTerminal::Break(ReactiveBreakTerminal {
            target: target_block,
            id,
            target_kind,
            loc,
        }),
        label: None,
    }))))
}

/// Visits a goto-continue to a scheduled block and returns the appropriate
/// reactive continue terminal statement.
fn visit_continue(
    cx: &Context,
    block: BlockId,
    id: InstructionId,
    loc: SourceLocation,
) -> Result<ReactiveStatement, CompilerError> {
    let (target_block, target_kind) = cx.get_continue_target(block).ok_or_else(|| {
        CompilerError::invariant(
            &format!("Expected continue target to be scheduled for bb{block:?}"),
            None,
            loc,
        )
    })?;
    Ok(ReactiveStatement::Terminal(Box::new(ReactiveTerminalStatement {
        terminal: ReactiveTerminal::Continue(ReactiveContinueTerminal {
            target: target_block,
            id,
            target_kind,
            loc,
        }),
        label: None,
    })))
}

// =====================================================================================
// Main block traversal
// =====================================================================================

/// Traverse a block and its successors, building a reactive block.
fn traverse_block(cx: &mut Context, block: BasicBlock) -> Result<ReactiveBlock, CompilerError> {
    let mut statements: ReactiveBlock = Vec::new();
    visit_block(cx, block, &mut statements)?;
    Ok(statements)
}

/// Visit a single block, appending its reactive statements to `statements`,
/// then continue visiting successor blocks (goto/fallthrough) iteratively.
///
/// The fallthrough/continuation pattern is converted from tail recursion to a
/// loop to avoid stack overflows when hundreds of Scope/PrunedScope wrappers
/// create deep chains.
fn visit_block(
    cx: &mut Context,
    mut block: BasicBlock,
    statements: &mut ReactiveBlock,
) -> Result<(), CompilerError> {
    loop {
        if cx.emitted.contains(&block.id) {
            // The block was already emitted via an inline-continuation path
            // (e.g., Goto(Break) fallback). Skip to avoid duplicating output.
            break;
        }
        cx.emitted.insert(block.id);

        // Add instructions as reactive statements
        for instr in &block.instructions {
            statements.push(ReactiveStatement::Instruction(ReactiveInstructionStatement {
                instruction: hir_instr_to_reactive(instr),
            }));
        }

        let mut schedule_ids: Vec<u32> = Vec::new();

        // Process terminal
        match &block.terminal {
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
                let fallthrough_id =
                    if cx.reachable(if_term.fallthrough) && !cx.is_scheduled(if_term.fallthrough) {
                        Some(if_term.fallthrough)
                    } else {
                        None
                    };
                let alternate_id =
                    (if_term.alternate != if_term.fallthrough).then_some(if_term.alternate);

                if let Some(ft) = fallthrough_id {
                    let sid = cx.schedule(ft, "if");
                    schedule_ids.push(sid);
                }

                let consequent = {
                    let consequent_block = cx.block(if_term.consequent).cloned();
                    if let Some(b) = consequent_block { traverse_block(cx, b)? } else { Vec::new() }
                };

                let alternate = if let Some(alt_id) = alternate_id {
                    let alt_block = cx.block(alt_id).cloned();
                    if let Some(b) = alt_block { Some(traverse_block(cx, b)?) } else { None }
                } else {
                    None
                };

                cx.unschedule_all(&schedule_ids);

                statements.push(ReactiveStatement::Terminal(Box::new(ReactiveTerminalStatement {
                    terminal: ReactiveTerminal::If(Box::new(ReactiveIfTerminal {
                        test: if_term.test.clone(),
                        consequent,
                        alternate,
                        id: if_term.id,
                        loc: if_term.loc,
                    })),
                    label: fallthrough_id.map(|ft| ReactiveLabel { id: ft, implicit: false }),
                })));

                if let Some(ft) = fallthrough_id
                    && let Some(ft_block) = cx.block(ft).cloned()
                {
                    block = ft_block;
                    continue;
                }
                break;
            }
            Terminal::Switch(switch_term) => {
                let fallthrough_id = if cx.reachable(switch_term.fallthrough)
                    && !cx.is_scheduled(switch_term.fallthrough)
                {
                    Some(switch_term.fallthrough)
                } else {
                    None
                };
                if let Some(ft) = fallthrough_id {
                    let sid = cx.schedule(ft, "switch");
                    schedule_ids.push(sid);
                }

                // Process cases in reverse order like the TS version
                let mut cases: Vec<ReactiveSwitchCase> = Vec::new();
                let reversed_cases: Vec<_> = switch_term.cases.iter().rev().collect();
                for case in &reversed_cases {
                    let test = case.test.clone();
                    if cx.is_scheduled(case.block) {
                        // Skip cases that point to already-scheduled blocks (fallthrough)
                        debug_assert!(
                            case.block == switch_term.fallthrough,
                            "Unexpected 'switch' where a case is already scheduled and block is not the fallthrough"
                        );
                        continue;
                    }
                    let consequent_block = cx.block(case.block).cloned();
                    let consequent = if let Some(b) = consequent_block {
                        traverse_block(cx, b)?
                    } else {
                        Vec::new()
                    };
                    let sid = cx.schedule(case.block, "case");
                    schedule_ids.push(sid);
                    cases.push(ReactiveSwitchCase { test, block: Some(consequent) });
                }
                cases.reverse();

                cx.unschedule_all(&schedule_ids);

                statements.push(ReactiveStatement::Terminal(Box::new(ReactiveTerminalStatement {
                    terminal: ReactiveTerminal::Switch(Box::new(ReactiveSwitchTerminal {
                        test: switch_term.test.clone(),
                        cases,
                        id: switch_term.id,
                        loc: switch_term.loc,
                    })),
                    label: fallthrough_id.map(|ft| ReactiveLabel { id: ft, implicit: false }),
                })));

                if let Some(ft) = fallthrough_id
                    && let Some(ft_block) = cx.block(ft).cloned()
                {
                    block = ft_block;
                    continue;
                }
                break;
            }
            Terminal::DoWhile(do_while) => {
                let fallthrough_id =
                    (!cx.is_scheduled(do_while.fallthrough)).then_some(do_while.fallthrough);
                let loop_id = if !cx.is_scheduled(do_while.r#loop)
                    && do_while.r#loop != do_while.fallthrough
                {
                    Some(do_while.r#loop)
                } else {
                    None
                };
                let sid =
                    cx.schedule_loop(do_while.fallthrough, do_while.test, Some(do_while.r#loop));
                schedule_ids.push(sid);

                let loop_body = if let Some(lid) = loop_id {
                    let lb = cx.block(lid).cloned();
                    if let Some(b) = lb { traverse_block(cx, b)? } else { Vec::new() }
                } else {
                    Vec::new()
                };

                let test_result = visit_value_block(cx, do_while.test, do_while.loc, None)?;

                cx.unschedule_all(&schedule_ids);

                statements.push(ReactiveStatement::Terminal(Box::new(ReactiveTerminalStatement {
                    terminal: ReactiveTerminal::DoWhile(Box::new(ReactiveDoWhileTerminal {
                        r#loop: loop_body,
                        test: test_result.value,
                        id: do_while.id,
                        loc: do_while.loc,
                    })),
                    label: fallthrough_id.map(|ft| ReactiveLabel { id: ft, implicit: false }),
                })));

                if let Some(ft) = fallthrough_id
                    && let Some(ft_block) = cx.block(ft).cloned()
                {
                    block = ft_block;
                    continue;
                }
                break;
            }
            Terminal::While(while_term) => {
                let fallthrough_id = if cx.reachable(while_term.fallthrough)
                    && !cx.is_scheduled(while_term.fallthrough)
                {
                    Some(while_term.fallthrough)
                } else {
                    None
                };
                let loop_id = if !cx.is_scheduled(while_term.r#loop)
                    && while_term.r#loop != while_term.fallthrough
                {
                    Some(while_term.r#loop)
                } else {
                    None
                };
                let sid = cx.schedule_loop(
                    while_term.fallthrough,
                    while_term.test,
                    Some(while_term.r#loop),
                );
                schedule_ids.push(sid);

                let test_result = visit_value_block(cx, while_term.test, while_term.loc, None)?;

                let loop_body = if let Some(lid) = loop_id {
                    let lb = cx.block(lid).cloned();
                    if let Some(b) = lb { traverse_block(cx, b)? } else { Vec::new() }
                } else {
                    Vec::new()
                };

                cx.unschedule_all(&schedule_ids);

                statements.push(ReactiveStatement::Terminal(Box::new(ReactiveTerminalStatement {
                    terminal: ReactiveTerminal::While(Box::new(ReactiveWhileTerminal {
                        test: test_result.value,
                        r#loop: loop_body,
                        id: while_term.id,
                        loc: while_term.loc,
                    })),
                    label: fallthrough_id.map(|ft| ReactiveLabel { id: ft, implicit: false }),
                })));

                if let Some(ft) = fallthrough_id
                    && let Some(ft_block) = cx.block(ft).cloned()
                {
                    block = ft_block;
                    continue;
                }
                break;
            }
            Terminal::For(for_term) => {
                let loop_id = if !cx.is_scheduled(for_term.r#loop)
                    && for_term.r#loop != for_term.fallthrough
                {
                    Some(for_term.r#loop)
                } else {
                    None
                };
                let fallthrough_id =
                    (!cx.is_scheduled(for_term.fallthrough)).then_some(for_term.fallthrough);
                let sid = cx.schedule_loop(
                    for_term.fallthrough,
                    for_term.update.unwrap_or(for_term.test),
                    Some(for_term.r#loop),
                );
                schedule_ids.push(sid);

                let init_result = visit_value_block(cx, for_term.init, for_term.loc, None)?;
                let init_value = ReactiveValue::Sequence(value_block_result_to_sequence(
                    init_result,
                    for_term.loc,
                ));
                let test_result = visit_value_block(cx, for_term.test, for_term.loc, None)?;
                let update = for_term
                    .update
                    .map(|update_id| visit_value_block(cx, update_id, for_term.loc, None))
                    .transpose()?
                    .map(|r| r.value);

                let loop_body = if let Some(lid) = loop_id {
                    let lb = cx.block(lid).cloned();
                    if let Some(b) = lb { traverse_block(cx, b)? } else { Vec::new() }
                } else {
                    Vec::new()
                };

                cx.unschedule_all(&schedule_ids);

                statements.push(ReactiveStatement::Terminal(Box::new(ReactiveTerminalStatement {
                    terminal: ReactiveTerminal::For(Box::new(ReactiveForTerminal {
                        init: init_value,
                        test: test_result.value,
                        update,
                        r#loop: loop_body,
                        id: for_term.id,
                        loc: for_term.loc,
                    })),
                    label: fallthrough_id.map(|ft| ReactiveLabel { id: ft, implicit: false }),
                })));

                if let Some(ft) = fallthrough_id
                    && let Some(ft_block) = cx.block(ft).cloned()
                {
                    block = ft_block;
                    continue;
                }
                break;
            }
            Terminal::ForOf(for_of) => {
                let loop_id =
                    if !cx.is_scheduled(for_of.r#loop) && for_of.r#loop != for_of.fallthrough {
                        Some(for_of.r#loop)
                    } else {
                        None
                    };
                let fallthrough_id =
                    (!cx.is_scheduled(for_of.fallthrough)).then_some(for_of.fallthrough);
                let sid = cx.schedule_loop(for_of.fallthrough, for_of.init, Some(for_of.r#loop));
                schedule_ids.push(sid);

                let init_result = visit_value_block(cx, for_of.init, for_of.loc, None)?;
                let init_value = ReactiveValue::Sequence(value_block_result_to_sequence(
                    init_result,
                    for_of.loc,
                ));
                let test_result = visit_value_block(cx, for_of.test, for_of.loc, None)?;
                let test_value = ReactiveValue::Sequence(value_block_result_to_sequence(
                    test_result,
                    for_of.loc,
                ));

                let loop_body = if let Some(lid) = loop_id {
                    let lb = cx.block(lid).cloned();
                    if let Some(b) = lb { traverse_block(cx, b)? } else { Vec::new() }
                } else {
                    Vec::new()
                };

                cx.unschedule_all(&schedule_ids);

                statements.push(ReactiveStatement::Terminal(Box::new(ReactiveTerminalStatement {
                    terminal: ReactiveTerminal::ForOf(Box::new(ReactiveForOfTerminal {
                        init: init_value,
                        test: test_value,
                        r#loop: loop_body,
                        id: for_of.id,
                        loc: for_of.loc,
                    })),
                    label: fallthrough_id.map(|ft| ReactiveLabel { id: ft, implicit: false }),
                })));

                if let Some(ft) = fallthrough_id
                    && let Some(ft_block) = cx.block(ft).cloned()
                {
                    block = ft_block;
                    continue;
                }
                break;
            }
            Terminal::ForIn(for_in) => {
                let loop_id =
                    if !cx.is_scheduled(for_in.r#loop) && for_in.r#loop != for_in.fallthrough {
                        Some(for_in.r#loop)
                    } else {
                        None
                    };
                let fallthrough_id =
                    (!cx.is_scheduled(for_in.fallthrough)).then_some(for_in.fallthrough);
                let sid = cx.schedule_loop(for_in.fallthrough, for_in.init, Some(for_in.r#loop));
                schedule_ids.push(sid);

                let init_result = visit_value_block(cx, for_in.init, for_in.loc, None)?;
                let init_value = ReactiveValue::Sequence(value_block_result_to_sequence(
                    init_result,
                    for_in.loc,
                ));

                let loop_body = if let Some(lid) = loop_id {
                    let lb = cx.block(lid).cloned();
                    if let Some(b) = lb { traverse_block(cx, b)? } else { Vec::new() }
                } else {
                    Vec::new()
                };

                cx.unschedule_all(&schedule_ids);

                statements.push(ReactiveStatement::Terminal(Box::new(ReactiveTerminalStatement {
                    terminal: ReactiveTerminal::ForIn(Box::new(ReactiveForInTerminal {
                        init: init_value,
                        r#loop: loop_body,
                        id: for_in.id,
                        loc: for_in.loc,
                    })),
                    label: fallthrough_id.map(|ft| ReactiveLabel { id: ft, implicit: false }),
                })));

                if let Some(ft) = fallthrough_id
                    && let Some(ft_block) = cx.block(ft).cloned()
                {
                    block = ft_block;
                    continue;
                }
                break;
            }
            Terminal::Branch(branch) => {
                let consequent = if cx.is_scheduled(branch.consequent) {
                    let break_stmt = visit_break(cx, branch.consequent, branch.id, branch.loc)?;
                    if let Some(s) = break_stmt { vec![s] } else { Vec::new() }
                } else {
                    let consequent_block = cx.block(branch.consequent).cloned();
                    if let Some(b) = consequent_block { traverse_block(cx, b)? } else { Vec::new() }
                };

                let alternate = if cx.is_scheduled(branch.alternate) {
                    // Invariant: alternate should not be scheduled for branch terminals
                    Vec::new()
                } else {
                    let alternate_block = cx.block(branch.alternate).cloned();
                    if let Some(b) = alternate_block { traverse_block(cx, b)? } else { Vec::new() }
                };
                let alternate_opt = if alternate.is_empty() { None } else { Some(alternate) };

                statements.push(ReactiveStatement::Terminal(Box::new(ReactiveTerminalStatement {
                    terminal: ReactiveTerminal::If(Box::new(ReactiveIfTerminal {
                        test: branch.test.clone(),
                        consequent,
                        alternate: alternate_opt,
                        id: branch.id,
                        loc: branch.loc,
                    })),
                    label: None,
                })));
                break;
            }
            Terminal::Label(label) => {
                let fallthrough_id =
                    if cx.reachable(label.fallthrough) && !cx.is_scheduled(label.fallthrough) {
                        Some(label.fallthrough)
                    } else {
                        None
                    };
                if let Some(ft) = fallthrough_id {
                    let sid = cx.schedule(ft, "if");
                    schedule_ids.push(sid);
                }

                let block_body = {
                    let label_block = cx.block(label.block).cloned();
                    if let Some(b) = label_block { traverse_block(cx, b)? } else { Vec::new() }
                };

                cx.unschedule_all(&schedule_ids);

                statements.push(ReactiveStatement::Terminal(Box::new(ReactiveTerminalStatement {
                    terminal: ReactiveTerminal::Label(Box::new(ReactiveLabelTerminal {
                        block: block_body,
                        id: label.id,
                        loc: label.loc,
                    })),
                    label: fallthrough_id.map(|ft| ReactiveLabel { id: ft, implicit: false }),
                })));

                if let Some(ft) = fallthrough_id
                    && let Some(ft_block) = cx.block(ft).cloned()
                {
                    block = ft_block;
                    continue;
                }
                break;
            }
            Terminal::Goto(goto) => {
                match goto.variant {
                    GotoVariant::Break => {
                        // If the target is a scope fallthrough, suppress the break.
                        if cx.scope_fallthroughs.contains(&goto.block) {
                            break;
                        }
                        // If the target is not scheduled and hasn't been emitted,
                        // treat it as an inline continuation rather than a break.
                        // This happens when build_reactive_scope_terminals_hir
                        // splits a Goto(Continue) across a scope boundary,
                        // creating an intermediate block.
                        if !cx.is_scheduled(goto.block)
                            && !cx.emitted.contains(&goto.block)
                            && let Some(next_block) = cx.block(goto.block).cloned()
                        {
                            block = next_block;
                            continue;
                        }
                        let break_stmt = visit_break(cx, goto.block, goto.id, goto.loc)?;
                        if let Some(s) = break_stmt {
                            statements.push(s);
                        }
                    }
                    GotoVariant::Continue => {
                        let continue_stmt = visit_continue(cx, goto.block, goto.id, goto.loc)?;
                        statements.push(continue_stmt);
                    }
                    GotoVariant::Try => {
                        // noop - try gotos flatten away
                    }
                }
                break;
            }
            Terminal::MaybeThrow(mt) => {
                // ReactiveFunction does not explicitly model maybe-throw semantics,
                // so these terminals flatten away
                if !cx.is_scheduled(mt.continuation)
                    && let Some(cont_block) = cx.block(mt.continuation).cloned()
                {
                    block = cont_block;
                    continue;
                }
                break;
            }
            Terminal::Try(try_term) => {
                let fallthrough_id = if cx.reachable(try_term.fallthrough)
                    && !cx.is_scheduled(try_term.fallthrough)
                {
                    Some(try_term.fallthrough)
                } else {
                    None
                };
                if let Some(ft) = fallthrough_id {
                    let sid = cx.schedule(ft, "if");
                    schedule_ids.push(sid);
                }
                cx.schedule_catch_handler(try_term.handler);

                let block_body = {
                    let try_block = cx.block(try_term.block).cloned();
                    if let Some(b) = try_block { traverse_block(cx, b)? } else { Vec::new() }
                };
                let handler_body = {
                    let handler_block = cx.block(try_term.handler).cloned();
                    if let Some(b) = handler_block { traverse_block(cx, b)? } else { Vec::new() }
                };

                cx.unschedule_all(&schedule_ids);

                statements.push(ReactiveStatement::Terminal(Box::new(ReactiveTerminalStatement {
                    terminal: ReactiveTerminal::Try(Box::new(ReactiveTryTerminal {
                        block: block_body,
                        handler_binding: try_term.handler_binding.clone(),
                        handler: handler_body,
                        id: try_term.id,
                        loc: try_term.loc,
                    })),
                    label: fallthrough_id.map(|ft| ReactiveLabel { id: ft, implicit: false }),
                })));

                if let Some(ft) = fallthrough_id
                    && let Some(ft_block) = cx.block(ft).cloned()
                {
                    block = ft_block;
                    continue;
                }
                break;
            }
            Terminal::Scope(scope) => {
                let fallthrough_id =
                    (!cx.is_scheduled(scope.fallthrough)).then_some(scope.fallthrough);
                if let Some(ft) = fallthrough_id {
                    let sid = cx.schedule(ft, "if");
                    schedule_ids.push(sid);
                    cx.scope_fallthroughs.insert(ft);
                }

                let block_body = {
                    let scope_block = cx.block(scope.block).cloned();
                    if let Some(b) = scope_block { traverse_block(cx, b)? } else { Vec::new() }
                };

                cx.unschedule_all(&schedule_ids);

                statements.push(ReactiveStatement::Scope(crate::hir::ReactiveScopeBlock {
                    scope: scope.scope.clone(),
                    instructions: block_body,
                }));

                if let Some(ft) = fallthrough_id
                    && let Some(ft_block) = cx.block(ft).cloned()
                {
                    block = ft_block;
                    continue;
                }
                break;
            }
            Terminal::PrunedScope(scope) => {
                let fallthrough_id =
                    (!cx.is_scheduled(scope.fallthrough)).then_some(scope.fallthrough);
                if let Some(ft) = fallthrough_id {
                    let sid = cx.schedule(ft, "if");
                    schedule_ids.push(sid);
                    cx.scope_fallthroughs.insert(ft);
                }

                let block_body = {
                    let scope_block = cx.block(scope.block).cloned();
                    if let Some(b) = scope_block { traverse_block(cx, b)? } else { Vec::new() }
                };

                cx.unschedule_all(&schedule_ids);

                statements.push(ReactiveStatement::PrunedScope(
                    crate::hir::PrunedReactiveScopeBlock {
                        scope: scope.scope.clone(),
                        instructions: block_body,
                    },
                ));

                if let Some(ft) = fallthrough_id
                    && let Some(ft_block) = cx.block(ft).cloned()
                {
                    block = ft_block;
                    continue;
                }
                break;
            }
            // Value-block terminals: Logical, Ternary, Optional, Sequence
            // These produce expression-level values and are emitted as instructions.
            Terminal::Sequence(seq) => {
                let fallthrough_id = (!cx.is_scheduled(seq.fallthrough)).then_some(seq.fallthrough);
                if let Some(ft) = fallthrough_id {
                    let sid = cx.schedule(ft, "if");
                    schedule_ids.push(sid);
                }

                let result = visit_value_block_terminal(cx, &block.terminal)?;

                cx.unschedule_all(&schedule_ids);

                statements.push(ReactiveStatement::Instruction(ReactiveInstructionStatement {
                    instruction: ReactiveInstruction {
                        id: seq.id,
                        lvalue: Some(result.place),
                        value: result.value,
                        loc: seq.loc,
                    },
                }));

                if let Some(ft) = fallthrough_id
                    && let Some(ft_block) = cx.block(ft).cloned()
                {
                    block = ft_block;
                    continue;
                }
                break;
            }
            Terminal::Logical(logical) => {
                let fallthrough_id =
                    (!cx.is_scheduled(logical.fallthrough)).then_some(logical.fallthrough);
                if let Some(ft) = fallthrough_id {
                    let sid = cx.schedule(ft, "if");
                    schedule_ids.push(sid);
                }

                let result = visit_value_block_terminal(cx, &block.terminal)?;

                cx.unschedule_all(&schedule_ids);

                statements.push(ReactiveStatement::Instruction(ReactiveInstructionStatement {
                    instruction: ReactiveInstruction {
                        id: logical.id,
                        lvalue: Some(result.place),
                        value: result.value,
                        loc: logical.loc,
                    },
                }));

                if let Some(ft) = fallthrough_id
                    && let Some(ft_block) = cx.block(ft).cloned()
                {
                    block = ft_block;
                    continue;
                }
                break;
            }
            Terminal::Ternary(ternary) => {
                let fallthrough_id =
                    (!cx.is_scheduled(ternary.fallthrough)).then_some(ternary.fallthrough);
                if let Some(ft) = fallthrough_id {
                    let sid = cx.schedule(ft, "if");
                    schedule_ids.push(sid);
                }

                let result = visit_value_block_terminal(cx, &block.terminal)?;

                cx.unschedule_all(&schedule_ids);

                statements.push(ReactiveStatement::Instruction(ReactiveInstructionStatement {
                    instruction: ReactiveInstruction {
                        id: ternary.id,
                        lvalue: Some(result.place),
                        value: result.value,
                        loc: ternary.loc,
                    },
                }));

                if let Some(ft) = fallthrough_id
                    && let Some(ft_block) = cx.block(ft).cloned()
                {
                    block = ft_block;
                    continue;
                }
                break;
            }
            Terminal::Optional(optional) => {
                let fallthrough_id =
                    (!cx.is_scheduled(optional.fallthrough)).then_some(optional.fallthrough);
                if let Some(ft) = fallthrough_id {
                    let sid = cx.schedule(ft, "if");
                    schedule_ids.push(sid);
                }

                let result = visit_value_block_terminal(cx, &block.terminal)?;

                cx.unschedule_all(&schedule_ids);

                statements.push(ReactiveStatement::Instruction(ReactiveInstructionStatement {
                    instruction: ReactiveInstruction {
                        id: optional.id,
                        lvalue: Some(result.place),
                        value: result.value,
                        loc: optional.loc,
                    },
                }));

                if let Some(ft) = fallthrough_id
                    && let Some(ft_block) = cx.block(ft).cloned()
                {
                    block = ft_block;
                    continue;
                }
                break;
            }
            // Terminals that end the block without continuation
            Terminal::Unreachable(_) | Terminal::Unsupported(_) => {
                break;
            }
        }
    }

    Ok(())
}
