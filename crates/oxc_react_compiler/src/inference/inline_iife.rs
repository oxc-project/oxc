/// Inline immediately-invoked function expressions (IIFEs).
///
/// Port of `Inference/InlineImmediatelyInvokedFunctionExpressions.ts` from the React Compiler.
///
/// Inlines IIFEs to allow more fine-grained memoization of the values they produce.
/// The implementation relies on HIR's labeled blocks to represent the inlined function body.
///
/// For functions with a single return, we avoid the labeled block and fully inline
/// the code. The original return is replaced with an assignment to the IIFE's call
/// expression lvalue.
///
/// For functions with multiple returns, we wrap the inlined body in a Label terminal
/// and replace all returns with store-to-temporary + goto-to-continuation.
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    compiler_error::{GENERATED_SOURCE, SourceLocation},
    hir::{
        BasicBlock, BlockId, BlockKind, DeclareLocal, FunctionExpressionValue, GotoTerminal,
        GotoVariant, HIRFunction, Hir, IdentifierId, IdentifierName, Instruction, InstructionId,
        InstructionKind, InstructionValue, LValue, LabelTerminal, LoadLocal, Place, StoreLocal,
        Terminal, UnreachableTerminal,
        environment::Environment,
        hir_builder::{
            create_temporary_place, each_terminal_successor, mark_instruction_ids,
            mark_predecessors,
        },
        merge_consecutive_blocks::merge_consecutive_blocks,
        visitors::each_instruction_value_operand,
    },
};

/// Inline immediately-invoked function expressions in the given function.
pub fn inline_immediately_invoked_function_expressions(func: &mut HIRFunction) {
    // Track all function expressions that are assigned to a temporary.
    // Maps callee identifier id -> the FunctionExpressionValue.
    let mut functions: FxHashMap<IdentifierId, FunctionExpressionValue> = FxHashMap::default();
    // Functions that are inlined
    let mut inlined_functions: FxHashSet<IdentifierId> = FxHashSet::default();

    // Iterate the *existing* blocks from the outer component to find IIFEs
    // and inline them. During iteration we will modify `func` (by inlining the CFG
    // of IIFEs) so we explicitly copy references to just the original
    // function's blocks first. As blocks are split to make room for IIFE calls,
    // the split portions of the blocks will be added to this queue.
    let mut queue: Vec<BlockId> = func.body.blocks.keys().copied().collect();

    let mut queue_idx = 0;
    'queue: while queue_idx < queue.len() {
        enum Action {
            TrackFunction {
                lvalue_id: IdentifierId,
                func_expr: FunctionExpressionValue,
            },
            InlineIife {
                callee_id: IdentifierId,
                body: FunctionExpressionValue,
                instr_lvalue: Place,
                terminal_id: InstructionId,
                terminal_loc: SourceLocation,
                block_kind: BlockKind,
                remaining_instructions: Vec<Instruction>,
                original_terminal: Box<Terminal>,
                is_single_return: bool,
            },
            RemoveFromFunctions {
                ids: Vec<IdentifierId>,
            },
            Skip,
        }

        let block_id = queue[queue_idx];
        queue_idx += 1;

        // We can't handle labels inside expressions yet, so we don't inline IIFEs
        // if they are in an expression block.
        let block_kind = match func.body.blocks.get(&block_id) {
            Some(block) => block.kind,
            None => continue,
        };
        if !block_kind.is_statement() {
            continue;
        }

        let mut ii = 0;
        while let Some(block) = func.body.blocks.get(&block_id) {
            let num_instructions = block.instructions.len();
            if ii >= num_instructions {
                break;
            }

            let action = {
                let block = &func.body.blocks[&block_id];
                let instr = &block.instructions[ii];
                match &instr.value {
                    InstructionValue::FunctionExpression(v) => {
                        if instr.lvalue.identifier.name.is_none() {
                            Action::TrackFunction {
                                lvalue_id: instr.lvalue.identifier.id,
                                func_expr: v.clone(),
                            }
                        } else {
                            Action::Skip
                        }
                    }
                    InstructionValue::CallExpression(v) => {
                        if v.args.is_empty() {
                            let callee_id = v.callee.identifier.id;
                            match functions.get(&callee_id) {
                                Some(body) => {
                                    if !body.lowered_func.func.params.is_empty()
                                        || body.lowered_func.func.is_async
                                        || body.lowered_func.func.generator
                                    {
                                        Action::Skip
                                    } else {
                                        let is_single_return = has_single_exit_return_terminal(
                                            &body.lowered_func.func,
                                        );
                                        Action::InlineIife {
                                            callee_id,
                                            body: body.clone(),
                                            instr_lvalue: instr.lvalue.clone(),
                                            terminal_id: block.terminal.id(),
                                            terminal_loc: block.terminal.loc(),
                                            block_kind: block.kind,
                                            remaining_instructions: block.instructions[ii + 1..]
                                                .to_vec(),
                                            original_terminal: Box::new(block.terminal.clone()),
                                            is_single_return,
                                        }
                                    }
                                }
                                None => Action::Skip,
                            }
                        } else {
                            Action::Skip
                        }
                    }
                    _ => {
                        let ids: Vec<IdentifierId> = each_instruction_value_operand(&instr.value)
                            .into_iter()
                            .map(|place| place.identifier.id)
                            .collect();
                        Action::RemoveFromFunctions { ids }
                    }
                }
            };

            match action {
                Action::TrackFunction { lvalue_id, func_expr } => {
                    functions.insert(lvalue_id, func_expr);
                }
                Action::InlineIife {
                    callee_id,
                    body,
                    instr_lvalue,
                    terminal_id,
                    terminal_loc,
                    block_kind,
                    remaining_instructions,
                    original_terminal,
                    is_single_return,
                } => {
                    inlined_functions.insert(callee_id);

                    // Create the continuation block
                    let continuation_block_id = func.env.next_block_id();
                    let continuation_block = BasicBlock {
                        id: continuation_block_id,
                        instructions: remaining_instructions,
                        kind: block_kind,
                        phis: Vec::new(),
                        preds: FxHashSet::default(),
                        terminal: *original_terminal,
                    };
                    func.body.blocks.insert(continuation_block_id, continuation_block);

                    // Trim the original block
                    if let Some(block) = func.body.blocks.get_mut(&block_id) {
                        block.instructions.truncate(ii);
                    }

                    let iife_entry = body.lowered_func.func.body.entry;
                    let mut iife_blocks = body.lowered_func.func.body.blocks;

                    if is_single_return {
                        // Single return path
                        if let Some(block) = func.body.blocks.get_mut(&block_id) {
                            block.terminal = Terminal::Goto(GotoTerminal {
                                id: terminal_id,
                                block: iife_entry,
                                variant: GotoVariant::Break,
                                loc: terminal_loc,
                            });
                        }

                        // Rewrite return blocks in the IIFE body
                        let iife_block_ids: Vec<BlockId> = iife_blocks.keys().copied().collect();
                        for iife_block_id in &iife_block_ids {
                            if let Some(iife_block) = iife_blocks.get_mut(iife_block_id)
                                && let Terminal::Return(ret) = &iife_block.terminal
                            {
                                let ret_loc = ret.loc;
                                let ret_id = ret.id;
                                let ret_value = ret.value.clone();

                                iife_block.instructions.push(Instruction {
                                    id: InstructionId::ZERO,
                                    loc: ret_loc,
                                    lvalue: instr_lvalue.clone(),
                                    value: InstructionValue::LoadLocal(LoadLocal {
                                        place: ret_value,
                                        loc: ret_loc,
                                    }),
                                    effects: None,
                                });
                                iife_block.terminal = Terminal::Goto(GotoTerminal {
                                    id: ret_id,
                                    block: continuation_block_id,
                                    variant: GotoVariant::Break,
                                    loc: ret_loc,
                                });
                            }
                        }

                        // Move all IIFE body blocks into func.body.blocks
                        for (id, mut iife_block) in iife_blocks {
                            iife_block.preds.clear();
                            func.body.blocks.insert(id, iife_block);
                        }
                    } else {
                        // Multi return path: use Label terminal.
                        // Borrow env and body.blocks as separate fields to satisfy
                        // the borrow checker.
                        let mut result = instr_lvalue;

                        {
                            let env = &mut func.env;
                            let blocks = &mut func.body.blocks;
                            if let Some(block) = blocks.get_mut(&block_id) {
                                block.terminal = Terminal::Label(LabelTerminal {
                                    id: InstructionId::ZERO,
                                    block: iife_entry,
                                    fallthrough: continuation_block_id,
                                    loc: terminal_loc,
                                });

                                // Declare the IIFE temporary on the current block
                                declare_temporary(env, block, &result);
                            }
                        }

                        // Promote the temporary with a name as we require this to persist
                        if result.identifier.name.is_none() {
                            promote_temporary(&mut result);
                        }

                        // Rewrite blocks from the lambda to replace return with store + goto
                        let iife_block_ids: Vec<BlockId> = iife_blocks.keys().copied().collect();
                        for iife_block_id in &iife_block_ids {
                            if let Some(iife_block) = iife_blocks.get_mut(iife_block_id) {
                                iife_block.preds.clear();
                                rewrite_block(
                                    &mut func.env,
                                    iife_block,
                                    continuation_block_id,
                                    &result,
                                );
                            }
                        }

                        // Move all IIFE body blocks into func.body.blocks
                        for (id, iife_block) in iife_blocks {
                            func.body.blocks.insert(id, iife_block);
                        }
                    }

                    // Ensure we visit the continuation block
                    queue.push(continuation_block_id);
                    continue 'queue;
                }
                Action::RemoveFromFunctions { ids } => {
                    for id in ids {
                        functions.remove(&id);
                    }
                }
                Action::Skip => {}
            }

            ii += 1;
        }
    }

    if !inlined_functions.is_empty() {
        // Remove instructions that define lambdas which we inlined
        for block in func.body.blocks.values_mut() {
            block
                .instructions
                .retain(|instr| !inlined_functions.contains(&instr.lvalue.identifier.id));
        }

        // If terminals have changed then blocks may have become newly unreachable.
        // Re-run minification of the graph (incl reordering instruction ids)
        reverse_postorder_blocks(&mut func.body);
        mark_instruction_ids(&mut func.body);
        mark_predecessors(&mut func.body);
        merge_consecutive_blocks(func);
    }
}

/// Returns true if the function has a single exit terminal (throw/return) which is a return.
fn has_single_exit_return_terminal(func: &HIRFunction) -> bool {
    let mut has_return = false;
    let mut exit_count = 0u32;
    for block in func.body.blocks.values() {
        match &block.terminal {
            Terminal::Return(_) => {
                has_return = true;
                exit_count += 1;
            }
            Terminal::Throw(_) => {
                exit_count += 1;
            }
            _ => {}
        }
    }
    exit_count == 1 && has_return
}

/// Rewrites the block so that all `return` terminals are replaced:
/// * Add a StoreLocal <return_value> = <terminal.value>
/// * Replace the terminal with a Goto to <return_target>
fn rewrite_block(
    env: &mut Environment,
    block: &mut BasicBlock,
    return_target: BlockId,
    return_value: &Place,
) {
    if let Terminal::Return(ret) = &block.terminal {
        let terminal_loc = ret.loc;
        let terminal_value = ret.value.clone();

        block.instructions.push(Instruction {
            id: InstructionId::ZERO,
            loc: terminal_loc,
            lvalue: create_temporary_place(env, terminal_loc),
            value: InstructionValue::StoreLocal(StoreLocal {
                lvalue: LValue { kind: InstructionKind::Reassign, place: return_value.clone() },
                value: terminal_value,
                loc: terminal_loc,
            }),
            effects: None,
        });
        block.terminal = Terminal::Goto(GotoTerminal {
            id: InstructionId::ZERO,
            block: return_target,
            variant: GotoVariant::Break,
            loc: terminal_loc,
        });
    }
}

/// Adds a DeclareLocal instruction to the block for the given result place.
fn declare_temporary(env: &mut Environment, block: &mut BasicBlock, result: &Place) {
    block.instructions.push(Instruction {
        id: InstructionId::ZERO,
        loc: GENERATED_SOURCE,
        lvalue: create_temporary_place(env, result.loc),
        value: InstructionValue::DeclareLocal(DeclareLocal {
            lvalue: LValue { place: result.clone(), kind: InstructionKind::Let },
            loc: result.loc,
        }),
        effects: None,
    });
}

/// Promotes a temporary identifier to a named identifier.
/// Port of `promoteTemporary` from HIR.ts.
fn promote_temporary(place: &mut Place) {
    let decl_id = place.identifier.declaration_id.0;
    place.identifier.name = Some(IdentifierName::Promoted(format!("#t{decl_id}")));
}

/// Reorder blocks in reverse postorder, removing unreachable blocks.
/// Port of `reversePostorderBlocks` from HIRBuilder.ts.
fn reverse_postorder_blocks(body: &mut Hir) {
    enum Phase {
        PreVisit,
        PostVisit,
    }

    let mut visited: FxHashSet<BlockId> = FxHashSet::default();
    let mut used: FxHashSet<BlockId> = FxHashSet::default();
    let mut used_fallthroughs: FxHashSet<BlockId> = FxHashSet::default();
    let mut postorder: Vec<BlockId> = Vec::new();

    // Iterative DFS using an explicit stack to avoid deep recursion.
    // The TS version uses recursion; we replicate the same traversal order iteratively.
    let mut stack: Vec<(BlockId, bool, Phase)> = Vec::new();
    stack.push((body.entry, true, Phase::PreVisit));

    while let Some((block_id, is_used, phase)) = stack.pop() {
        match phase {
            Phase::PreVisit => {
                let was_used = used.contains(&block_id);
                let was_visited = visited.contains(&block_id);
                visited.insert(block_id);
                if is_used {
                    used.insert(block_id);
                }
                if was_visited && (was_used || !is_used) {
                    continue;
                }

                let Some(block) = body.blocks.get(&block_id) else {
                    continue;
                };

                // Visit successors in reverse order (matching TS which reverses successors
                // before iterating, ensuring correct output order after final reversal).
                let successors: Vec<BlockId> =
                    each_terminal_successor(&block.terminal).into_iter().rev().collect();
                let fallthrough = block.terminal.fallthrough();

                // Push post-visit marker first (will be processed after children)
                if !was_visited {
                    stack.push((block_id, is_used, Phase::PostVisit));
                }

                // Push successors onto stack (reverse order so first successor is on top)
                for &successor in &successors {
                    stack.push((successor, is_used, Phase::PreVisit));
                }

                // Push fallthrough last (highest priority, processed first from stack)
                if let Some(ft) = fallthrough {
                    if is_used {
                        used_fallthroughs.insert(ft);
                    }
                    stack.push((ft, false, Phase::PreVisit));
                }
            }
            Phase::PostVisit => {
                postorder.push(block_id);
            }
        }
    }

    postorder.reverse();

    let mut new_blocks: FxHashMap<BlockId, BasicBlock> = FxHashMap::default();
    for block_id in &postorder {
        if used.contains(block_id) {
            if let Some(block) = body.blocks.remove(block_id) {
                new_blocks.insert(*block_id, block);
            }
        } else if used_fallthroughs.contains(block_id)
            && let Some(block) = body.blocks.remove(block_id)
        {
            new_blocks.insert(
                *block_id,
                BasicBlock {
                    id: block.id,
                    kind: block.kind,
                    instructions: Vec::new(),
                    phis: block.phis,
                    preds: block.preds,
                    terminal: Terminal::Unreachable(UnreachableTerminal {
                        id: block.terminal.id(),
                        loc: block.terminal.loc(),
                    }),
                },
            );
        }
        // otherwise this block is unreachable, drop it
    }

    body.blocks = new_blocks;
}
