/// Conservative instruction reordering optimization pass.
///
/// Port of `Optimization/InstructionReordering.ts` (~516 LoC) from the React
/// Compiler. Gated by `Environment.config().enable_instruction_reordering`
/// (boolean, default `false`; matches upstream `Environment.ts:427`:
/// `enableInstructionReordering: z.boolean().default(false)`).
///
/// # Goal (mirrors upstream doc-comment lines 31-71)
///
/// Move instructions closer to where their produced values are consumed,
/// grouping them in a way that lets `MergeReactiveScopesThatAlwaysInvalidateTogether`
/// merge more scopes. Two candidate scopes can only merge if there are no
/// intervening instructions that are used by later code; reordering moves
/// those intervening instructions later (or earlier) so the candidates end
/// up adjacent.
///
/// # Algorithm
///
/// Build a per-block dependency graph and re-emit instructions in an order
/// that minimises intervening non-reorderable nodes.
///
/// **Nodes.** Each lvalue identifier gets a node with `(instruction?,
/// dependencies, reorderability, depth)`. `Destructure [x, y] = z` produces
/// three nodes: one for the destructure instruction itself (lvalue `z`) and
/// one each for `x` and `y` — the latter two have `instruction: null` and
/// depend on the destructure node.
///
/// **Dependency edges:**
/// 1. Non-reorderable instructions chain to the previous non-reorderable
///    instruction in the same block (preserves mutation ordering).
/// 2. Every operand of the instruction value adds a dep on its identifier id
///    (if that id has a local or shared node).
/// 3. Named variables: all reads and writes of the same `value.name.Named(s)`
///    are serialized — each access depends on the previous one and updates
///    the `named` map.
/// 4. Each lvalue from `each_instruction_value_lvalue` (Destructure targets,
///    StoreLocal targets, etc.) becomes its own lvalue-only node that
///    depends on the instruction's main lvalue. Named lvalues participate
///    in the named-serialization rule too.
///
/// **Reorderability** (`getReorderability` upstream lines 484-516):
/// - `JsxExpression`, `JsxFragment`, `JsxText`, `LoadGlobal`, `Primitive`,
///   `TemplateLiteral`, `BinaryExpression`, `UnaryExpression` → always
///   Reorderable.
/// - `LoadLocal`: Reorderable iff its name is `Named(s)`, the last write
///   of `s` happens strictly before this instruction id, AND its lvalue id
///   is in the `singleUseIdentifiers` set. Otherwise Nonreorderable.
/// - Everything else → Nonreorderable.
///
/// **Emission.** For statement blocks (kind `Block`/`Catch`) we emit
/// transitive deps of each terminal operand first (depth-first, sorted by
/// transitive weight), then anything left over: reorderable leftovers go to
/// the shared map for cross-block emission, non-reorderable leftovers get
/// flushed in reverse-insertion order. For expression blocks (Value/Loop/
/// Sequence) we must preserve the final-instruction-as-value semantics:
/// emit the last non-reorderable instruction first (to preserve mutation
/// order), then the block's last instruction (the value), then terminal
/// operand deps, then save reorderable leftovers.
///
/// **Weight (`getDepth`)**: depth(node) = (1 if reorderable else 10) + sum
/// of depth over deps. Dependencies are sorted bDepth - aDepth (highest
/// depth first) when emitting.
///
/// # Pipeline placement
///
/// Runs between `DeadCodeElimination` and `PruneMaybeThrows`, mirroring
/// upstream `Pipeline.ts:242-245`:
///
/// ```text
/// if (env.config.enableInstructionReordering) {
///   instructionReordering(hir);
///   log({kind: 'hir', name: 'InstructionReordering', value: hir});
/// }
///
/// pruneMaybeThrows(hir);
/// ```
use indexmap::IndexMap;
use rustc_hash::{FxBuildHasher, FxHashMap, FxHashSet};

use crate::{
    compiler_error::{CompilerError, GENERATED_SOURCE},
    hir::{
        BlockId, HIRFunction, IdentifierId, IdentifierName, Instruction, InstructionId,
        InstructionValue,
        hir_builder::mark_instruction_ids,
        visitors::{
            each_instruction_lvalue, each_instruction_value_lvalue, each_instruction_value_operand,
            each_terminal_operand,
        },
    },
};

/// Phi operands keyed by their *source* block id. For each block, the
/// vector contains every successor-phi operand whose `BlockId` key is the
/// current block — i.e. the values that flow out of the current block into
/// a successor's phi. These are emission roots from the current block's
/// perspective.
type PhiOperandsBySource = FxHashMap<BlockId, Vec<(IdentifierId, Option<IdentifierName>)>>;

/// Run the `InstructionReordering` optimization on a HIR function.
///
/// Mutates `func` in place. After reordering, re-numbers instruction IDs
/// (matches upstream `markInstructionIds(fn.body)` at the end of the pass).
///
/// # Errors
///
/// Returns a `CompilerError` with category `Invariant` if any reorderable
/// nodes remain in the shared map after all blocks have been visited —
/// that would mean a deferred reorderable producer was never emitted, and
/// the instruction would be silently dropped from the output. Matches
/// upstream `CompilerError.invariant(shared.size === 0, ...)` at TS lines
/// 79-92.
pub fn instruction_reordering(func: &mut HIRFunction) -> Result<(), CompilerError> {
    // Shared nodes are emitted when first used (across blocks). TS line 74.
    let mut shared: Nodes = IndexMap::with_hasher(FxBuildHasher);
    let phi_operands_by_source = collect_phi_operands_by_source(func);
    let references = find_referenced_range_of_temporaries(func, &phi_operands_by_source);

    // Collect block ids first so we can mutate blocks under iteration.
    let block_ids: Vec<_> = func.body.blocks.keys().copied().collect();
    for block_id in block_ids {
        let phi_roots = phi_operands_by_source.get(&block_id).cloned().unwrap_or_default();
        reorder_block(&mut func.body.blocks[&block_id], &mut shared, &references, &phi_roots);
    }

    // Upstream invariant (TS lines 79-92): all shared reorderable nodes
    // must have been emitted by the time we finish. If this fires the
    // compiler would silently drop instructions in release builds — fail
    // fast instead.
    if !shared.is_empty() {
        let loc = shared
            .values()
            .find_map(|n| n.instruction.as_ref().map(|i| i.loc))
            .unwrap_or(GENERATED_SOURCE);
        let leftover_summary = format!(
            "{} reorderable node(s) were deferred to the shared map but never \
             emitted by a later block",
            shared.len()
        );
        return Err(CompilerError::invariant(
            "InstructionReordering: expected all reorderable nodes to have been emitted",
            Some(&leftover_summary),
            loc,
        ));
    }

    mark_instruction_ids(&mut func.body);
    Ok(())
}

/// Walk all blocks' phis and group each phi operand by its *source* block
/// id (the `BlockId` key in `phi.operands`). The returned map answers
/// "what values flow out of block X into successor phis?" — the
/// `IndexMap` reorderer treats those operands as references that must
/// survive reordering of block X.
fn collect_phi_operands_by_source(func: &HIRFunction) -> PhiOperandsBySource {
    let mut out: PhiOperandsBySource = FxHashMap::default();
    for block in func.body.blocks.values() {
        for phi in &block.phis {
            for (src_block, place) in &phi.operands {
                out.entry(*src_block)
                    .or_default()
                    .push((place.identifier.id, place.identifier.name.clone()));
            }
        }
    }
    out
}

// =====================================================================================
// Reference scan: collect single-use temporaries and last-assignment ids
// =====================================================================================

/// Inclusive start and end. (TS `References` at lines 107-112.)
struct References {
    /// Identifier ids referenced exactly once as a read. Used by
    /// `LoadLocal` reorderability check to know that the load's lvalue is
    /// consumed at a single later site (so the load can move).
    single_use_identifiers: FxHashSet<IdentifierId>,
    /// For each named variable, the largest `InstructionId` that writes
    /// to it (an lvalue with `name == Named(s)`). Used by `LoadLocal`
    /// reorderability to ensure we don't move a load earlier than the
    /// most recent write of its source.
    last_assignments: FxHashMap<String, InstructionId>,
}

#[derive(Clone, Copy)]
enum ReferenceKind {
    Read,
    Write,
}

fn find_referenced_range_of_temporaries(
    func: &HIRFunction,
    phi_operands_by_source: &PhiOperandsBySource,
) -> References {
    let mut counts: FxHashMap<IdentifierId, u32> = FxHashMap::default();
    let mut last_assignments: FxHashMap<String, InstructionId> = FxHashMap::default();

    let reference = |instr: InstructionId,
                     id: IdentifierId,
                     name: Option<&IdentifierName>,
                     kind: ReferenceKind,
                     counts: &mut FxHashMap<IdentifierId, u32>,
                     last_assignments: &mut FxHashMap<String, InstructionId>| {
        if let Some(IdentifierName::Named(value)) = name {
            // TS lines 126-141: named writes update the last-assignment map
            // with the max of (previous, current). Named reads/writes do
            // NOT contribute to `singleUseIdentifiers` — the `return`
            // upstream short-circuits before the read-count update.
            if matches!(kind, ReferenceKind::Write) {
                let previous = last_assignments.get(value).copied();
                let new_id = match previous {
                    Some(prev) => InstructionId(prev.0.max(instr.0)),
                    None => instr,
                };
                last_assignments.insert(value.clone(), new_id);
            }
            return;
        }
        // TS lines 142-145: for reads on non-named (temporary) identifiers,
        // count usages. Writes on temporaries (or `Promoted` names) do not
        // update the count.
        if matches!(kind, ReferenceKind::Read) {
            *counts.entry(id).or_insert(0) += 1;
        }
    };

    for block in func.body.blocks.values() {
        for instr in &block.instructions {
            // value lvalues are *read* in this context per upstream lines
            // 148-150: `eachInstructionValueLValue(instr.value)` then
            // `ReferenceKind.Read`. Yes this is surprising — upstream
            // treats those as identifiers that *appear* in the value
            // (Destructure pattern operands, StoreLocal target, etc.) and
            // want to count their reads.
            for operand in each_instruction_value_lvalue(&instr.value) {
                reference(
                    instr.id,
                    operand.identifier.id,
                    operand.identifier.name.as_ref(),
                    ReferenceKind::Read,
                    &mut counts,
                    &mut last_assignments,
                );
            }
            for lvalue in each_instruction_lvalue(instr) {
                reference(
                    instr.id,
                    lvalue.identifier.id,
                    lvalue.identifier.name.as_ref(),
                    ReferenceKind::Write,
                    &mut counts,
                    &mut last_assignments,
                );
            }
        }
        for operand in each_terminal_operand(&block.terminal) {
            reference(
                block.terminal.id(),
                operand.identifier.id,
                operand.identifier.name.as_ref(),
                ReferenceKind::Read,
                &mut counts,
                &mut last_assignments,
            );
        }
        // Oxc port extension over TS: any value flowing out of this block
        // into a successor's phi must also count as a read. The TS pass
        // runs on phi-free code internally, but the Rust HIR keeps phis
        // in place across this pass. Without this step, a `LoadLocal`
        // whose only consumer is a phi would be misclassified — `count`
        // would stay 0, the lvalue would never enter `singleUseIdentifiers`,
        // so the load stays Nonreorderable (safe). The risk we *do* close
        // here is the opposite mismatch: a value used by one instruction
        // *and* one phi would otherwise be counted as `count == 1` and
        // incorrectly classified Reorderable.
        if let Some(phi_operands) = phi_operands_by_source.get(&block.id) {
            for (id, name) in phi_operands {
                reference(
                    block.terminal.id(),
                    *id,
                    name.as_ref(),
                    ReferenceKind::Read,
                    &mut counts,
                    &mut last_assignments,
                );
            }
        }
    }

    // TS lines 160-166: keep only single-read identifiers.
    let single_use_identifiers: FxHashSet<IdentifierId> =
        counts.into_iter().filter_map(|(id, n)| if n == 1 { Some(id) } else { None }).collect();

    References { single_use_identifiers, last_assignments }
}

// =====================================================================================
// Per-block dependency graph + reordering
// =====================================================================================

type Nodes = IndexMap<IdentifierId, Node, FxBuildHasher>;

struct Node {
    /// `None` for lvalue-only nodes (e.g. each Destructure target). Some
    /// otherwise.
    instruction: Option<Instruction>,
    dependencies: FxHashSet<IdentifierId>,
    /// Only meaningful for nodes with `instruction.is_some()`. Lvalue-only
    /// nodes inherit reorderability from the instruction they depend on
    /// (TS upstream stores `Reorderability` only when `instruction != null`
    /// but defaults the field anyway; we follow that with a default).
    reorderability: Reorderability,
    /// Memoized transitive depth (set on first request, cycle-safe).
    depth: Option<u32>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Reorderability {
    Reorderable,
    Nonreorderable,
}

fn reorder_block(
    block: &mut crate::hir::BasicBlock,
    shared: &mut Nodes,
    references: &References,
    phi_operand_roots: &[(IdentifierId, Option<IdentifierName>)],
) {
    // Locals MUST preserve insertion order — TS `Map` does. The
    // expression-block leftover sweep iterates `locals` insertion order,
    // and the statement-block leftover sweep iterates in REVERSE
    // insertion order. Using `FxHashMap` here would break both.
    let mut locals: Nodes = IndexMap::with_hasher(FxBuildHasher);
    let mut named: FxHashMap<String, IdentifierId> = FxHashMap::default();
    let mut previous: Option<IdentifierId> = None;

    // Take the instructions out of the block so we can move them into
    // node ownership. TS retains references; Rust must own.
    let instructions = std::mem::take(&mut block.instructions);

    // Capture the lvalue id of the *original* last instruction BEFORE we
    // move instructions into nodes (and before `emit` deletes them). For
    // expression blocks this id is the block's value; mirrors TS
    // `block.instructions.at(-1)!.lvalue.identifier.id` (line 282/289).
    // We cannot recover this after the fact: if the last instruction is
    // non-reorderable, the "emit last non-reorderable instruction first"
    // step removes it from `locals`, and scanning for the now-last main
    // node would pick up an earlier reorderable instruction instead —
    // producing the wrong block value.
    let original_last_lvalue_id: Option<IdentifierId> =
        instructions.last().map(|i| i.lvalue.identifier.id);

    for instr in instructions {
        let lvalue_id = instr.lvalue.identifier.id;
        let reorderability = get_reorderability(&instr, references);

        // Get-or-create the main node for this lvalue id. TS lines 183-193.
        // If a previous iteration created an lvalue-only node for this id
        // (instruction == None, from value-lvalue handling below), we fill
        // in `instruction` and `reorderability` now.
        let node = locals.entry(lvalue_id).or_insert_with(|| Node {
            instruction: None,
            dependencies: FxHashSet::default(),
            reorderability,
            depth: None,
        });
        node.instruction = Some(instr);
        node.reorderability = reorderability;

        // TS lines 198-203: non-reorderable instructions chain through the
        // most recent prior non-reorderable instruction.
        if matches!(reorderability, Reorderability::Nonreorderable) {
            if let Some(prev) = previous {
                node.dependencies.insert(prev);
            }
            previous = Some(lvalue_id);
        }

        // Collect operands first so we can release the &mut node borrow
        // before we look up other entries in `locals`.
        let value_ref = node.instruction.as_ref().unwrap();
        let operands: Vec<(IdentifierId, Option<IdentifierName>)> =
            each_instruction_value_operand(&value_ref.value)
                .into_iter()
                .map(|place| (place.identifier.id, place.identifier.name.clone()))
                .collect();
        let value_lvalue_ops: Vec<(IdentifierId, Option<IdentifierName>)> =
            each_instruction_value_lvalue(&value_ref.value)
                .into_iter()
                .map(|place| (place.identifier.id, place.identifier.name.clone()))
                .collect();

        // TS lines 206-219: operand-derived dependencies.
        let mut new_deps: Vec<IdentifierId> = Vec::new();
        let mut new_named_writes: Vec<(String, IdentifierId)> = Vec::new();
        for (operand_id, operand_name) in operands {
            if let Some(IdentifierName::Named(name_value)) = &operand_name {
                if let Some(&prev) = named.get(name_value) {
                    new_deps.push(prev);
                }
                new_named_writes.push((name_value.clone(), lvalue_id));
            } else if locals.contains_key(&operand_id) || shared.contains_key(&operand_id) {
                new_deps.push(operand_id);
            }
        }

        // TS lines 226-246: lvalue-only nodes for each value-level lvalue,
        // depending on the main instruction node. Named lvalues also
        // participate in the named-serialization rule.
        for (lv_id, lv_name) in value_lvalue_ops {
            // Ensure an lvalue-only node exists for `lv_id` and add a
            // dependency on the main lvalue id (i.e. on the instruction
            // itself).
            let lv_node = locals.entry(lv_id).or_insert_with(|| Node {
                instruction: None,
                dependencies: FxHashSet::default(),
                reorderability: Reorderability::Nonreorderable,
                depth: None,
            });
            lv_node.dependencies.insert(lvalue_id);

            if let Some(IdentifierName::Named(name_value)) = &lv_name {
                if let Some(&prev) = named.get(name_value) {
                    new_deps.push(prev);
                }
                new_named_writes.push((name_value.clone(), lvalue_id));
            }
        }

        // Now reborrow the main node and apply the collected deps + named
        // writes.
        let main_node = locals.get_mut(&lvalue_id).expect("main node just inserted");
        for dep in new_deps {
            main_node.dependencies.insert(dep);
        }
        for (n, id) in new_named_writes {
            named.insert(n, id);
        }
    }

    let mut next_instructions: Vec<Instruction> = Vec::new();

    if block.kind.is_expression() {
        // TS lines 261-328: expression block reorder.
        // 1. Emit the last non-reorderable instruction first so its chain
        //    of mutating effects is preserved.
        if let Some(prev) = previous {
            emit(&mut locals, shared, &mut next_instructions, prev);
        }
        // 2. Emit the block's last instruction (value of the expression).
        //    Use the lvalue id of the *original* last instruction —
        //    captured before the mem::take above — to mirror upstream
        //    `block.instructions.at(-1)!.lvalue.identifier.id` (TS line
        //    282/289). Recomputing from `locals` after step 1 is wrong:
        //    if the original last instruction was non-reorderable, step 1
        //    already emitted (and removed) it, and the now-last main node
        //    would be an *earlier* reorderable instruction, which step 2
        //    would then emit AFTER the real block value.
        if let Some(id) = original_last_lvalue_id {
            emit(&mut locals, shared, &mut next_instructions, id);
        }
        // 3. Emit dependencies of the terminal operands.
        let terminal_operand_ids: Vec<IdentifierId> =
            each_terminal_operand(&block.terminal).into_iter().map(|p| p.identifier.id).collect();
        for id in terminal_operand_ids {
            emit(&mut locals, shared, &mut next_instructions, id);
        }
        // 3b. Oxc port extension: emit dependencies of phi operands flowing
        //     out of this block. Without this step, a reorderable value
        //     consumed only by a successor's phi would be deferred to
        //     `shared` and then lost when no later block ever asks for it.
        for (id, _) in phi_operand_roots {
            emit(&mut locals, shared, &mut next_instructions, *id);
        }
        // 4. Anything left in `locals` is globally reorderable — save it
        //    to `shared` so a later block can emit it.
        let leftover_ids: Vec<IdentifierId> = locals.keys().copied().collect();
        for id in leftover_ids {
            if let Some(node) = locals.shift_remove(&id) {
                if node.instruction.is_none() {
                    continue;
                }
                debug_assert!(
                    matches!(node.reorderability, Reorderability::Reorderable),
                    "InstructionReordering: expected leftover instruction to be reorderable",
                );
                shared.insert(id, node);
            }
        }
    } else {
        // TS lines 329-378: statement block reorder.
        // 1. Emit transitive deps of every terminal operand. Reorderable
        //    deps that are not transitively required by any terminal
        //    operand can be moved freely.
        let terminal_operand_ids: Vec<IdentifierId> =
            each_terminal_operand(&block.terminal).into_iter().map(|p| p.identifier.id).collect();
        for id in terminal_operand_ids {
            emit(&mut locals, shared, &mut next_instructions, id);
        }
        // 1b. Oxc port extension: emit dependencies of phi operands. See
        //     the analogous step in the expression-block branch above.
        for (id, _) in phi_operand_roots {
            emit(&mut locals, shared, &mut next_instructions, *id);
        }
        // 2. Iterate the remaining locals in REVERSE insertion order
        //    (TS: `Array.from(locals.keys()).reverse()`). Reorderable
        //    leftovers are saved to `shared`; non-reorderable leftovers
        //    are flushed in place.
        let leftover_ids: Vec<IdentifierId> = locals.keys().rev().copied().collect();
        for id in leftover_ids {
            // Skip ids that emit() already removed from locals.
            if !locals.contains_key(&id) {
                continue;
            }
            // Read reorderability without removing yet — emit() will
            // remove it when called for non-reorderable leftovers.
            let is_reorderable = matches!(
                locals.get(&id).map(|n| n.reorderability),
                Some(Reorderability::Reorderable),
            );
            if is_reorderable {
                if let Some(node) = locals.shift_remove(&id) {
                    if node.instruction.is_none() {
                        continue;
                    }
                    shared.insert(id, node);
                }
            } else {
                emit(&mut locals, shared, &mut next_instructions, id);
            }
        }
    }

    block.instructions = next_instructions;
}

// =====================================================================================
// Depth + emission
// =====================================================================================

/// `getDepth` (TS lines 385-400). Memoised; cycle-safe via initial 0 marker
/// before recursion.
fn get_depth(nodes: &mut Nodes, id: IdentifierId) -> u32 {
    let Some(entry_index) = nodes.get_index_of(&id) else { return 0 };
    if let Some(d) = nodes[entry_index].depth {
        return d;
    }
    // Mark with 0 first to break cycles.
    nodes[entry_index].depth = Some(0);

    let mut depth: u32 = match nodes[entry_index].reorderability {
        Reorderability::Reorderable => 1,
        Reorderability::Nonreorderable => 10,
    };
    let deps: Vec<IdentifierId> = nodes[entry_index].dependencies.iter().copied().collect();
    for dep in deps {
        depth = depth.saturating_add(get_depth(nodes, dep));
    }
    nodes[entry_index].depth = Some(depth);
    depth
}

/// `emit` (TS lines 452-477). Walks the dep graph depth-first, sorting
/// deps by descending transitive depth, removes nodes from `locals`/
/// `shared` once emitted (to dedup across the recursion).
fn emit(
    locals: &mut Nodes,
    shared: &mut Nodes,
    instructions: &mut Vec<Instruction>,
    id: IdentifierId,
) {
    // Snapshot the node's dependencies + take its instruction. Use
    // `shift_remove` to preserve insertion order in the remaining map.
    let (mut deps, instr_opt) = {
        // Look up in locals first, then shared.
        let from_locals = locals.shift_remove(&id);
        let node = match from_locals {
            Some(n) => Some(n),
            None => shared.shift_remove(&id),
        };
        match node {
            Some(n) => {
                let deps: Vec<IdentifierId> = n.dependencies.into_iter().collect();
                (deps, n.instruction)
            }
            None => return,
        }
    };

    // Sort deps by descending transitive depth (TS lines 466-470:
    // `bDepth - aDepth`). Depth is computed over `locals` per TS.
    deps.sort_by_key(|b| std::cmp::Reverse(get_depth(locals, *b)));

    for dep in deps {
        emit(locals, shared, instructions, dep);
    }

    if let Some(instr) = instr_opt {
        instructions.push(instr);
    }
}

// =====================================================================================
// Reorderability classification
// =====================================================================================

/// TS `getReorderability` lines 483-516.
fn get_reorderability(instr: &Instruction, references: &References) -> Reorderability {
    match &instr.value {
        InstructionValue::JsxExpression(_)
        | InstructionValue::JsxFragment(_)
        | InstructionValue::JsxText(_)
        | InstructionValue::LoadGlobal(_)
        | InstructionValue::Primitive(_)
        | InstructionValue::TemplateLiteral(_)
        | InstructionValue::BinaryExpression(_)
        | InstructionValue::UnaryExpression(_) => Reorderability::Reorderable,
        InstructionValue::LoadLocal(load) => {
            // TS lines 498-510: only reorder a LoadLocal of a named source
            // when the last write of that name happened strictly before
            // this instruction's id AND the load's lvalue is single-use.
            if let Some(IdentifierName::Named(name_value)) = &load.place.identifier.name
                && let Some(&last_assignment) = references.last_assignments.get(name_value)
                && last_assignment.0 < instr.id.0
                && references.single_use_identifiers.contains(&instr.lvalue.identifier.id)
            {
                return Reorderability::Reorderable;
            }
            Reorderability::Nonreorderable
        }
        _ => Reorderability::Nonreorderable,
    }
}

// =====================================================================================
// Unit tests
//
// These tests construct minimal BasicBlocks directly (bypassing the full
// HIR builder) and call `reorder_block` to verify the two correctness
// gaps closed in this commit:
//
// 1. `expression_block_preserves_original_final_value`: a Value block
//    where the original last instruction is non-reorderable AND an
//    unrelated reorderable instruction sits earlier. Pre-fix the pass
//    would recompute the "last main id" after emitting the non-reorderable
//    instruction first and incorrectly emit the unrelated reorderable
//    instruction AFTER the real block value. Post-fix the original final
//    instruction id is captured up-front and the order is preserved.
//
// 2. `shared_leak_when_reorderable_value_feeds_only_a_phi`: a statement
//    block holding a reorderable Primitive whose lvalue is consumed only
//    by a successor phi. Pre-fix the leftover sweep dumped it into
//    `shared` and `instruction_reordering` then released the function
//    with that producer never emitted — release builds silently lost it.
//    Post-fix the phi-operand-roots step calls `emit` for that lvalue,
//    so the instruction survives the pass.
// =====================================================================================
#[cfg(test)]
mod tests {
    use rustc_hash::FxHashSet;

    use super::*;
    use crate::compiler_error::GENERATED_SOURCE;
    use crate::hir::{
        BasicBlock, BlockId, BlockKind, Effect, GotoTerminal, GotoVariant, Instruction,
        InstructionId, InstructionValue, Place, PrimitiveValue, PrimitiveValueKind, Terminal,
        hir_builder::make_temporary_identifier,
    };

    /// Build a `Place` for the given identifier id (no name, default effect).
    fn place(id: u32) -> Place {
        Place {
            identifier: make_temporary_identifier(IdentifierId(id), GENERATED_SOURCE),
            effect: Effect::Unknown,
            reactive: false,
            loc: GENERATED_SOURCE,
        }
    }

    /// Build a Primitive instruction (always Reorderable).
    fn primitive_instr(lvalue_id: u32, instr_id: u32) -> Instruction {
        Instruction {
            id: InstructionId(instr_id),
            lvalue: place(lvalue_id),
            value: InstructionValue::Primitive(PrimitiveValue {
                value: PrimitiveValueKind::Number(f64::from(instr_id)),
                loc: GENERATED_SOURCE,
            }),
            effects: None,
            loc: GENERATED_SOURCE,
        }
    }

    /// Build an `Unsupported` instruction (always Nonreorderable — falls
    /// through to the `_` arm in `get_reorderability`). Uses
    /// `InstructionValue::UnsupportedNode` which the pass treats as
    /// Nonreorderable.
    fn nonreorderable_instr(lvalue_id: u32, instr_id: u32) -> Instruction {
        use crate::hir::UnsupportedNode;
        Instruction {
            id: InstructionId(instr_id),
            lvalue: place(lvalue_id),
            value: InstructionValue::UnsupportedNode(UnsupportedNode { loc: GENERATED_SOURCE }),
            effects: None,
            loc: GENERATED_SOURCE,
        }
    }

    fn empty_references() -> References {
        References {
            single_use_identifiers: FxHashSet::default(),
            last_assignments: FxHashMap::default(),
        }
    }

    /// Build a `Goto` terminal pointing at the given target block.
    fn goto_terminal(id: u32, target: u32) -> Terminal {
        Terminal::Goto(GotoTerminal {
            id: InstructionId(id),
            block: BlockId(target),
            variant: GotoVariant::Break,
            loc: GENERATED_SOURCE,
        })
    }

    /// Build a Value (expression) basic block with the given instructions
    /// and a goto terminal to a sink block.
    fn value_block(block_id: u32, instructions: Vec<Instruction>, terminal_id: u32) -> BasicBlock {
        BasicBlock {
            kind: BlockKind::Value,
            id: BlockId(block_id),
            instructions,
            terminal: goto_terminal(terminal_id, block_id + 1),
            preds: FxHashSet::default(),
            phis: Vec::new(),
        }
    }

    /// Build a statement (Block) basic block.
    fn block(block_id: u32, instructions: Vec<Instruction>, terminal_id: u32) -> BasicBlock {
        BasicBlock {
            kind: BlockKind::Block,
            id: BlockId(block_id),
            instructions,
            terminal: goto_terminal(terminal_id, block_id + 1),
            preds: FxHashSet::default(),
            phis: Vec::new(),
        }
    }

    /// **Gap 1 regression.** Expression block where the original final
    /// instruction is non-reorderable AND an earlier unrelated reorderable
    /// instruction exists. The reordered output must preserve the original
    /// final instruction as the block's last emitted instruction.
    ///
    /// Layout:
    ///   id=1 lvalue=$10  Primitive   (Reorderable, unrelated)
    ///   id=2 lvalue=$20  Nonreorderable
    ///   id=3 lvalue=$30  Nonreorderable  <-- original last, becomes block value
    ///
    /// Pre-fix expected (buggy):
    ///   The pass emits the previous-chain last non-reorderable (id=3, lvalue=$30),
    ///   then `last_main_id` scans `locals.iter().rev()` and finds id=2 ($20),
    ///   then id=1 ($10). Emission order would interleave the unrelated
    ///   reorderable $10 AFTER the block value $30. Specifically: last
    ///   emitted would be $10, not $30.
    ///
    /// Post-fix expected: the original final lvalue ($30) is captured
    /// up-front; emitting it after the previous-chain emission is a no-op
    /// (it's already gone), and the block ends with $30 as the last
    /// emitted instruction.
    #[test]
    fn expression_block_preserves_original_final_value() {
        let instructions =
            vec![primitive_instr(10, 1), nonreorderable_instr(20, 2), nonreorderable_instr(30, 3)];
        let mut block_a = value_block(0, instructions, 4);

        let mut shared: Nodes = IndexMap::with_hasher(FxBuildHasher);
        let references = empty_references();
        reorder_block(&mut block_a, &mut shared, &references, &[]);

        // The last emitted instruction must be the one with lvalue $30,
        // the original final instruction. If the buggy implementation
        // returns, the last emitted lvalue would be $10 (the unrelated
        // reorderable primitive) instead.
        let last_lvalue = block_a
            .instructions
            .last()
            .expect("expression block should emit at least one instruction")
            .lvalue
            .identifier
            .id
            .0;
        assert_eq!(
            last_lvalue,
            30,
            "expression block must end with the original final instruction's lvalue id; \
             got block order = {:?}",
            block_a.instructions.iter().map(|i| i.lvalue.identifier.id.0).collect::<Vec<_>>(),
        );

        // The unrelated reorderable primitive ($10) has no consumers — it
        // gets deferred to `shared`. Confirm that's where it landed (and
        // that we don't have stragglers in the block).
        assert!(
            shared.contains_key(&IdentifierId(10)),
            "unused reorderable instruction should be deferred to `shared` for \
             cross-block emission, got shared keys = {:?}",
            shared.keys().map(|k| k.0).collect::<Vec<_>>(),
        );
    }

    /// **Gap 2 regression.** A reorderable Primitive whose lvalue is only
    /// consumed by a successor block's phi must NOT be silently dropped.
    /// The phi operand is supplied to `reorder_block` via
    /// `phi_operand_roots`; the pass must emit the producing instruction
    /// instead of dumping it into `shared`.
    ///
    /// Pre-fix: without the phi-roots step, `$10 = Primitive(...)` has no
    /// terminal-operand consumer, no in-block consumer, so the leftover
    /// sweep moves it into `shared`. The `instruction_reordering` driver
    /// then finishes (next block, the phi-consuming one, doesn't look at
    /// `shared[$10]` either) and the producer is lost. The
    /// `debug_assert!(shared.is_empty())` did not fire in release builds.
    ///
    /// Post-fix: the phi root for $10 makes the pass call `emit($10)`,
    /// which moves the instruction into the block's emitted instructions.
    #[test]
    fn statement_block_phi_root_prevents_shared_leak() {
        // Single reorderable instruction; no in-block consumer.
        let mut producer_block = block(0, vec![primitive_instr(10, 1)], 2);

        let mut shared: Nodes = IndexMap::with_hasher(FxBuildHasher);
        let references = empty_references();

        // Phi root: tells the reorderer that lvalue $10 is consumed by a
        // successor's phi and must be emitted in this block.
        let phi_roots: Vec<(IdentifierId, Option<IdentifierName>)> = vec![(IdentifierId(10), None)];
        reorder_block(&mut producer_block, &mut shared, &references, &phi_roots);

        // The producing instruction must be emitted in this block.
        let emitted_ids: Vec<u32> =
            producer_block.instructions.iter().map(|i| i.lvalue.identifier.id.0).collect();
        assert!(
            emitted_ids.contains(&10),
            "phi-consumed reorderable producer must be emitted in its source block; \
             got {emitted_ids:?}",
        );
        // And `shared` must NOT hold a stale copy of the leaked node.
        assert!(
            !shared.contains_key(&IdentifierId(10)),
            "phi-consumed producer must not leak into shared",
        );
    }

    /// Sanity check: an expression block whose original final instruction
    /// is *reorderable* (i.e. no previous non-reorderable chain) still
    /// works correctly — the original-last-lvalue capture must not break
    /// this case. The block's value is the reorderable final instruction.
    #[test]
    fn expression_block_with_reorderable_final_value() {
        let instructions = vec![nonreorderable_instr(20, 1), primitive_instr(10, 2)];
        let mut block_a = value_block(0, instructions, 3);

        let mut shared: Nodes = IndexMap::with_hasher(FxBuildHasher);
        let references = empty_references();
        reorder_block(&mut block_a, &mut shared, &references, &[]);

        // Should end with $10 (the original final instruction).
        let last_lvalue = block_a
            .instructions
            .last()
            .expect("expression block should emit at least one instruction")
            .lvalue
            .identifier
            .id
            .0;
        assert_eq!(last_lvalue, 10);
    }
}
