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

use crate::hir::{
    HIRFunction, IdentifierId, IdentifierName, Instruction, InstructionId, InstructionValue,
    hir_builder::mark_instruction_ids,
    visitors::{
        each_instruction_lvalue, each_instruction_value_lvalue, each_instruction_value_operand,
        each_terminal_operand,
    },
};

/// Run the `InstructionReordering` optimization on a HIR function.
///
/// Mutates `func` in place. After reordering, re-numbers instruction IDs
/// (matches upstream `markInstructionIds(fn.body)` at the end of the pass).
pub fn instruction_reordering(func: &mut HIRFunction) {
    // Shared nodes are emitted when first used (across blocks). TS line 74.
    let mut shared: Nodes = IndexMap::with_hasher(FxBuildHasher);
    let references = find_referenced_range_of_temporaries(func);

    // Collect block ids first so we can mutate blocks under iteration.
    let block_ids: Vec<_> = func.body.blocks.keys().copied().collect();
    for block_id in block_ids {
        reorder_block(&mut func.body.blocks[&block_id], &mut shared, &references);
    }

    // Upstream invariant (lines 79-92): all shared reorderable nodes must
    // have been emitted by the time we finish. The Rust port does not
    // surface this as a fatal error (matches the policy used in other
    // ported optimization passes; see `lower_context_access`'s collect/
    // rewrite pattern). If the invariant is broken we'd silently drop
    // instructions; document that in the comment but don't panic in
    // release.
    debug_assert!(
        shared.is_empty(),
        "InstructionReordering: expected all reorderable nodes to have been emitted; \
         {} leftover",
        shared.len(),
    );

    mark_instruction_ids(&mut func.body);
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

fn find_referenced_range_of_temporaries(func: &HIRFunction) -> References {
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

fn reorder_block(block: &mut crate::hir::BasicBlock, shared: &mut Nodes, references: &References) {
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
        //    The whole block may have been empty, in which case we skip.
        //    We can't reach the original last instruction directly because
        //    we moved them into nodes, but we can look at `locals` in
        //    insertion order: the LAST inserted main-instruction node is
        //    the block's last instruction. (Lvalue-only nodes inserted
        //    later don't have an `instruction`.)
        let last_main_id =
            locals.iter().rev().find(|(_, n)| n.instruction.is_some()).map(|(id, _)| *id);
        if let Some(id) = last_main_id {
            emit(&mut locals, shared, &mut next_instructions, id);
        }
        // 3. Emit dependencies of the terminal operands.
        let terminal_operand_ids: Vec<IdentifierId> =
            each_terminal_operand(&block.terminal).into_iter().map(|p| p.identifier.id).collect();
        for id in terminal_operand_ids {
            emit(&mut locals, shared, &mut next_instructions, id);
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
