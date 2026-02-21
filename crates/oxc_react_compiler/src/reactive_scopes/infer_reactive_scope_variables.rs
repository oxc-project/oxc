/// Infer reactive scope variables.
///
/// Port of `ReactiveScopes/InferReactiveScopeVariables.ts` from the React Compiler.
///
/// For each mutable variable, infers a reactive scope which will construct that
/// variable. Variables that co-mutate are assigned to the same reactive scope.
///
/// This is the 1st of 4 passes that determine how to break a function into
/// discrete reactive scopes:
/// 1. InferReactiveScopeVariables (this pass)
/// 2. AlignReactiveScopesToBlockScopes
/// 3. MergeOverlappingReactiveScopes
/// 4. BuildReactiveBlocks
use crate::{
    hir::{
        HIRFunction, Identifier, IdentifierId, InstructionId,
        visitors::each_instruction_operand,
    },
    utils::disjoint_set::DisjointSet,
};

/// Check if an identifier is mutable at the given instruction.
pub fn is_mutable(identifier: &Identifier, at_instruction: InstructionId) -> bool {
    let range = &identifier.mutable_range;
    at_instruction >= range.start && at_instruction < range.end
}

/// Infer reactive scope variables for the function.
pub fn infer_reactive_scope_variables(func: &HIRFunction) {
    // Phase 1: Find groups of co-mutating identifiers using disjoint sets
    let mut co_mutations: DisjointSet<IdentifierId> = DisjointSet::new();

    let block_ids: Vec<_> = func.body.blocks.keys().copied().collect();
    for &block_id in &block_ids {
        let Some(block) = func.body.blocks.get(&block_id) else { continue };

        for instr in &block.instructions {
            // Collect all mutable operands at this instruction
            let mut mutable_operands: Vec<IdentifierId> = Vec::new();

            // Check the lvalue
            if is_mutable(&instr.lvalue.identifier, instr.id) {
                mutable_operands.push(instr.lvalue.identifier.id);
            }

            // Check all operands
            for place in each_instruction_operand(instr) {
                if is_mutable(&place.identifier, instr.id) {
                    mutable_operands.push(place.identifier.id);
                }
            }

            // All mutable operands of the same instruction must be in the same scope
            if !mutable_operands.is_empty() {
                co_mutations.union(&mutable_operands);
            }
        }
    }

    // Phase 2: Assign scope IDs to each group
    // (In the full implementation, this would create ReactiveScope objects
    // and assign them to identifiers. For now, we just group them.)
    let _sets = co_mutations.build_sets();
}

/// Find all sets of disjoint mutable values in the function.
pub fn find_disjoint_mutable_values(func: &HIRFunction) -> DisjointSet<IdentifierId> {
    let mut co_mutations: DisjointSet<IdentifierId> = DisjointSet::new();

    for block in func.body.blocks.values() {
        for instr in &block.instructions {
            let mut mutable_operands: Vec<IdentifierId> = Vec::new();

            if is_mutable(&instr.lvalue.identifier, instr.id) {
                mutable_operands.push(instr.lvalue.identifier.id);
            }

            for place in each_instruction_operand(instr) {
                if is_mutable(&place.identifier, instr.id) {
                    mutable_operands.push(place.identifier.id);
                }
            }

            if !mutable_operands.is_empty() {
                co_mutations.union(&mutable_operands);
            }
        }
    }

    co_mutations
}
