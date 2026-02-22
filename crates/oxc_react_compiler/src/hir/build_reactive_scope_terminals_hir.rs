/// Build reactive scope terminals in the HIR.
///
/// Port of `HIR/BuildReactiveScopeTerminalsHIR.ts` from the React Compiler.
///
/// This pass inserts `Terminal::Scope` and `Terminal::PrunedScope` terminals into
/// the CFG at reactive scope boundaries. It transforms a flat block sequence into
/// blocks connected by scope terminals.
///
/// The pass assumes that all program blocks are properly nested with respect to
/// fallthroughs (e.g. a valid javascript AST). Given a function whose reactive
/// scope ranges have been correctly aligned and merged, this pass rewrites blocks
/// to introduce ReactiveScopeTerminals and their fallthrough blocks.
use rustc_hash::{FxHashMap, FxHashSet};

use crate::compiler_error::GENERATED_SOURCE;
use crate::hir::hir_builder::{each_terminal_successor, mark_instruction_ids, mark_predecessors};
use crate::hir::hir_types::{
    BasicBlock, BlockId, BlockKind, GotoTerminal, GotoVariant, HIRFunction, Hir, Instruction,
    InstructionId, MutableRange, ReactiveScope, ReactiveScopeTerminal, ScopeId, Terminal,
    UnreachableTerminal,
};
use crate::hir::visitors::{
    each_instruction_lvalue, each_instruction_operand, each_terminal_operand,
};

// =====================================================================================
// get_scopes: collect all unique reactive scopes from identifiers
// =====================================================================================

/// Collect all unique reactive scopes from identifier annotations in the HIR.
///
/// Walks all instruction lvalues, operands, and terminal operands. Returns a
/// vector of scopes sorted in pre-order (ascending start, descending end for ties).
fn get_scopes(func: &HIRFunction) -> Vec<ReactiveScope> {
    let mut seen_ids: FxHashSet<ScopeId> = FxHashSet::default();
    let mut scopes: Vec<ReactiveScope> = Vec::new();

    for block in func.body.blocks.values() {
        for instr in &block.instructions {
            for operand in each_instruction_lvalue(instr) {
                visit_place_for_scope(operand, &mut seen_ids, &mut scopes);
            }
            for operand in each_instruction_operand(instr) {
                visit_place_for_scope(operand, &mut seen_ids, &mut scopes);
            }
        }
        for operand in each_terminal_operand(&block.terminal) {
            visit_place_for_scope(operand, &mut seen_ids, &mut scopes);
        }
    }

    scopes
}

fn visit_place_for_scope(
    place: &crate::hir::hir_types::Place,
    seen_ids: &mut FxHashSet<ScopeId>,
    scopes: &mut Vec<ReactiveScope>,
) {
    if let Some(scope) = &place.identifier.scope
        && scope.range.start != scope.range.end
        && seen_ids.insert(scope.id)
    {
        scopes.push(*scope.clone());
    }
}

// =====================================================================================
// range_pre_order_comparator / recursively_traverse_items
// =====================================================================================

/// Sort ranges in ascending order of start, breaking ties with descending order of end.
/// For overlapping ranges, this always orders nested inner ranges after outer ranges,
/// which is identical to the ordering of a pre-order tree traversal.
fn range_pre_order_comparator(a: MutableRange, b: MutableRange) -> std::cmp::Ordering {
    let start_cmp = a.start.0.cmp(&b.start.0);
    if start_cmp != std::cmp::Ordering::Equal {
        return start_cmp;
    }
    // Descending order of end for ties
    b.end.0.cmp(&a.end.0)
}

/// Process nested ranges by calling `enter` for outer ranges first, then inner ones.
/// When an inner range ends, `exit` is called.
fn recursively_traverse_items<T, TContext>(
    items: &mut [T],
    get_range: impl Fn(&T) -> MutableRange,
    context: &mut TContext,
    enter: impl Fn(&T, &mut TContext),
    exit: impl Fn(&T, &mut TContext),
) {
    items.sort_by(|a, b| range_pre_order_comparator(get_range(a), get_range(b)));
    let ranges: Vec<MutableRange> = items.iter().map(&get_range).collect();
    let mut active_indices: Vec<usize> = Vec::new();

    for i in 0..items.len() {
        let curr_range = ranges[i];
        // Pop active items that are disjoint from the current range
        while let Some(&active_idx) = active_indices.last() {
            let parent_range = get_range(&items[active_idx]);
            let disjoint = curr_range.start.0 >= parent_range.end.0;
            let nested = curr_range.end.0 <= parent_range.end.0;
            debug_assert!(disjoint || nested, "Invalid nesting: items overlap but are not nested");
            if disjoint {
                exit(&items[active_idx], context);
                active_indices.pop();
            } else {
                break;
            }
        }
        enter(&items[i], context);
        active_indices.push(i);
    }

    // Exit remaining active items
    while let Some(active_idx) = active_indices.pop() {
        exit(&items[active_idx], context);
    }
}

// =====================================================================================
// Terminal rewrite info types
// =====================================================================================

enum TerminalRewriteInfo {
    StartScope {
        block_id: BlockId,
        fallthrough_id: BlockId,
        instr_id: InstructionId,
        scope: ReactiveScope,
    },
    EndScope {
        instr_id: InstructionId,
        fallthrough_id: BlockId,
    },
}

impl TerminalRewriteInfo {
    fn instr_id(&self) -> InstructionId {
        match self {
            TerminalRewriteInfo::StartScope { instr_id, .. }
            | TerminalRewriteInfo::EndScope { instr_id, .. } => *instr_id,
        }
    }
}

struct ScopeTraversalContext<'a> {
    fallthroughs: FxHashMap<ScopeId, BlockId>,
    rewrites: &'a mut Vec<TerminalRewriteInfo>,
    env: &'a mut crate::hir::environment::Environment,
}

fn push_start_scope_terminal(scope: &ReactiveScope, context: &mut ScopeTraversalContext) {
    let block_id = context.env.next_block_id();
    let fallthrough_id = context.env.next_block_id();
    context.rewrites.push(TerminalRewriteInfo::StartScope {
        block_id,
        fallthrough_id,
        instr_id: scope.range.start,
        scope: scope.clone(),
    });
    context.fallthroughs.insert(scope.id, fallthrough_id);
}

fn push_end_scope_terminal(scope: &ReactiveScope, context: &mut ScopeTraversalContext) {
    let fallthrough_id = context.fallthroughs.get(&scope.id).copied();
    debug_assert!(fallthrough_id.is_some(), "Expected scope to exist");
    if let Some(fallthrough_id) = fallthrough_id {
        context
            .rewrites
            .push(TerminalRewriteInfo::EndScope { fallthrough_id, instr_id: scope.range.end });
    }
}

// =====================================================================================
// Rewrite context for block splitting
// =====================================================================================

struct RewriteContext {
    source_kind: BlockKind,
    source_instructions: Vec<Instruction>,
    source_phis: Vec<crate::hir::hir_types::Phi>,
    instr_slice_idx: usize,
    next_preds: FxHashSet<BlockId>,
    next_block_id: BlockId,
    rewrites: Vec<BasicBlock>,
}

/// Create a block rewrite by slicing a set of instructions from source.
fn handle_rewrite(terminal_info: &TerminalRewriteInfo, idx: usize, context: &mut RewriteContext) {
    let terminal: Terminal = match terminal_info {
        TerminalRewriteInfo::StartScope { fallthrough_id, block_id, instr_id, scope } => {
            Terminal::Scope(ReactiveScopeTerminal {
                id: *instr_id,
                block: *block_id,
                scope: scope.clone(),
                fallthrough: *fallthrough_id,
                loc: GENERATED_SOURCE,
            })
        }
        TerminalRewriteInfo::EndScope { instr_id, fallthrough_id } => {
            Terminal::Goto(GotoTerminal {
                id: *instr_id,
                block: *fallthrough_id,
                variant: GotoVariant::Break,
                loc: GENERATED_SOURCE,
            })
        }
    };

    let curr_block_id = context.next_block_id;
    let instructions = context.source_instructions[context.instr_slice_idx..idx].to_vec();
    let preds = std::mem::take(&mut context.next_preds);
    // Only the first rewrite should reuse source block phis
    let phis = if context.rewrites.is_empty() {
        std::mem::take(&mut context.source_phis)
    } else {
        Vec::new()
    };

    context.rewrites.push(BasicBlock {
        kind: context.source_kind,
        id: curr_block_id,
        instructions,
        preds,
        phis,
        terminal,
    });

    let mut new_preds = FxHashSet::default();
    new_preds.insert(curr_block_id);
    context.next_preds = new_preds;

    context.next_block_id = match terminal_info {
        TerminalRewriteInfo::StartScope { block_id, .. } => *block_id,
        TerminalRewriteInfo::EndScope { fallthrough_id, .. } => *fallthrough_id,
    };
    context.instr_slice_idx = idx;
}

// =====================================================================================
// reverse_postorder_blocks
// =====================================================================================

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

                let successors: Vec<BlockId> =
                    each_terminal_successor(&block.terminal).into_iter().rev().collect();
                let fallthrough = block.terminal.fallthrough();

                if !was_visited {
                    stack.push((block_id, is_used, Phase::PostVisit));
                }

                for &successor in &successors {
                    stack.push((successor, is_used, Phase::PreVisit));
                }

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
    }

    body.blocks = new_blocks;
}

// =====================================================================================
// fix_scope_and_identifier_ranges
// =====================================================================================

/// Fix scope and identifier ranges to account for renumbered instructions.
///
/// After inserting new blocks and renumbering instruction IDs, scope ranges must be
/// updated so they start at the scope terminal and end at the first instruction of
/// the fallthrough block.
fn fix_scope_and_identifier_ranges(func: &mut Hir) {
    // Collect the fixups we need to apply (scope terminal id -> fallthrough first instruction id)
    let fixups: Vec<(BlockId, InstructionId, InstructionId)> = func
        .blocks
        .values()
        .filter_map(|block| match &block.terminal {
            Terminal::Scope(t) => {
                let fallthrough_block = func.blocks.get(&t.fallthrough)?;
                let first_id = fallthrough_block
                    .instructions
                    .first()
                    .map_or(fallthrough_block.terminal.id(), |instr| instr.id);
                Some((block.id, t.id, first_id))
            }
            Terminal::PrunedScope(t) => {
                let fallthrough_block = func.blocks.get(&t.fallthrough)?;
                let first_id = fallthrough_block
                    .instructions
                    .first()
                    .map_or(fallthrough_block.terminal.id(), |instr| instr.id);
                Some((block.id, t.id, first_id))
            }
            _ => None,
        })
        .collect();

    for (block_id, terminal_id, end_id) in fixups {
        if let Some(block) = func.blocks.get_mut(&block_id) {
            match &mut block.terminal {
                Terminal::Scope(t) => {
                    t.scope.range.start = terminal_id;
                    t.scope.range.end = end_id;
                }
                Terminal::PrunedScope(t) => {
                    t.scope.range.start = terminal_id;
                    t.scope.range.end = end_id;
                }
                _ => {}
            }
        }
    }
}

// =====================================================================================
// Main entry point
// =====================================================================================

/// Build reactive scope terminals in the HIR.
///
/// This pass rewrites blocks to introduce `Terminal::Scope` (and `Terminal::PrunedScope`)
/// terminals and their fallthrough blocks at reactive scope boundaries.
pub fn build_reactive_scope_terminals_hir(func: &mut HIRFunction) {
    // ---------------------------------------------------------------------------------
    // Step 1: Traverse all blocks to build up a list of rewrites.
    // We also pre-allocate the fallthrough ID here as scope start terminals and
    // scope end terminals both require a fallthrough block.
    // ---------------------------------------------------------------------------------
    let mut queued_rewrites: Vec<TerminalRewriteInfo> = Vec::new();
    let mut scopes = get_scopes(func);

    {
        let mut context = ScopeTraversalContext {
            fallthroughs: FxHashMap::default(),
            rewrites: &mut queued_rewrites,
            env: &mut func.env,
        };

        recursively_traverse_items(
            &mut scopes,
            |s| s.range,
            &mut context,
            |scope, ctx| push_start_scope_terminal(scope, ctx),
            |scope, ctx| push_end_scope_terminal(scope, ctx),
        );
    }

    // ---------------------------------------------------------------------------------
    // Step 2: Traverse all blocks to apply rewrites. Here, we split blocks as described
    // to add scope terminals and fallthroughs.
    // ---------------------------------------------------------------------------------
    let mut rewritten_final_blocks: FxHashMap<BlockId, BlockId> = FxHashMap::default();
    let mut next_blocks: FxHashMap<BlockId, BasicBlock> = FxHashMap::default();

    // Reverse queued_rewrites to pop off the end as we traverse instructions in
    // ascending order.
    queued_rewrites.reverse();

    let block_ids: Vec<BlockId> = func.body.blocks.keys().copied().collect();
    for block_id in block_ids {
        let Some(block) = func.body.blocks.remove(&block_id) else { continue };

        let mut context = RewriteContext {
            source_kind: block.kind,
            source_instructions: block.instructions.clone(),
            source_phis: block.phis.clone(),
            instr_slice_idx: 0,
            next_preds: block.preds.clone(),
            next_block_id: block.id,
            rewrites: Vec::new(),
        };

        // Handle queued terminal rewrites at their nearest instruction ID.
        // Note that multiple terminal rewrites may map to the same instruction ID.
        let num_instructions = block.instructions.len();
        for i in 0..=num_instructions {
            let instr_id =
                if i < num_instructions { block.instructions[i].id } else { block.terminal.id() };

            while let Some(rewrite) = queued_rewrites.last() {
                if rewrite.instr_id().0 <= instr_id.0 {
                    // We need to pop first, then handle, to avoid borrow issues
                    let rewrite = queued_rewrites.pop();
                    if let Some(ref rewrite) = rewrite {
                        handle_rewrite(rewrite, i, &mut context);
                    }
                } else {
                    break;
                }
            }
        }

        if context.rewrites.is_empty() {
            next_blocks.insert(block.id, block);
        } else {
            let final_block = BasicBlock {
                id: context.next_block_id,
                kind: block.kind,
                preds: context.next_preds,
                terminal: block.terminal,
                instructions: context.source_instructions[context.instr_slice_idx..].to_vec(),
                phis: Vec::new(),
            };
            let final_block_id = final_block.id;
            context.rewrites.push(final_block);
            for b in context.rewrites {
                next_blocks.insert(b.id, b);
            }
            rewritten_final_blocks.insert(block.id, final_block_id);
        }
    }

    func.body.blocks = next_blocks;

    // ---------------------------------------------------------------------------------
    // Step 3: Repoint phis when they refer to a rewritten block.
    // In the TS version, originalBlocks and nextBlocks share phi objects by reference,
    // so modifying phis while iterating originalBlocks also updates nextBlocks.
    // In Rust, we directly update the phis in func.body.blocks.
    // ---------------------------------------------------------------------------------
    if !rewritten_final_blocks.is_empty() {
        for block in func.body.blocks.values_mut() {
            for phi in &mut block.phis {
                let keys_to_remap: Vec<BlockId> = phi
                    .operands
                    .keys()
                    .filter(|k| rewritten_final_blocks.contains_key(k))
                    .copied()
                    .collect();
                for original_id in keys_to_remap {
                    if let Some(&new_id) = rewritten_final_blocks.get(&original_id)
                        && let Some(value) = phi.operands.remove(&original_id)
                    {
                        phi.operands.insert(new_id, value);
                    }
                }
            }
        }
    }

    // ---------------------------------------------------------------------------------
    // Step 4: Fixup the HIR to restore RPO, ensure correct predecessors, and
    // renumber instructions.
    // ---------------------------------------------------------------------------------
    reverse_postorder_blocks(&mut func.body);
    mark_predecessors(&mut func.body);
    mark_instruction_ids(&mut func.body);

    // ---------------------------------------------------------------------------------
    // Step 5: Fix scope and identifier ranges to account for renumbered instructions
    // ---------------------------------------------------------------------------------
    fix_scope_and_identifier_ranges(&mut func.body);
}
