/// Dead code elimination optimization pass.
///
/// Port of `Optimization/DeadCodeElimination.ts` from the React Compiler.
///
/// Eliminates instructions whose values are unused. Uses a two-phase approach:
/// 1. Find/mark all referenced identifiers (fixpoint iteration in postorder)
/// 2. Prune unreferenced instructions and rewrite destructuring patterns
use rustc_hash::FxHashSet;

use crate::hir::{
    BlockKind, HIRFunction, Identifier, IdentifierId, InstructionKind, InstructionValue,
    visitors::{each_instruction_value_operand, each_terminal_operand},
};

/// State for tracking referenced identifiers during DCE.
struct DceState {
    named: FxHashSet<String>,
    identifiers: FxHashSet<IdentifierId>,
}

impl DceState {
    fn new() -> Self {
        Self { named: FxHashSet::default(), identifiers: FxHashSet::default() }
    }

    /// Mark the identifier as being referenced (not dead code).
    fn reference(&mut self, identifier: &Identifier) {
        self.identifiers.insert(identifier.id);
        if let Some(name) = &identifier.name {
            let name_str = name.value().to_string();
            self.named.insert(name_str);
        }
    }

    /// Check if any version of the given identifier is used somewhere.
    fn is_id_or_name_used(&self, identifier: &Identifier) -> bool {
        if self.identifiers.contains(&identifier.id) {
            return true;
        }
        if let Some(name) = &identifier.name {
            let name_str = name.value();
            return self.named.contains(name_str);
        }
        false
    }

    fn count(&self) -> usize {
        self.identifiers.len()
    }

    /// Check if this specific identifier ID is used.
    fn is_id_used(&self, identifier: &Identifier) -> bool {
        self.identifiers.contains(&identifier.id)
    }
}

/// Run dead code elimination on the given function.
pub fn dead_code_elimination(func: &mut HIRFunction) {
    // Phase 1: Find all referenced identifiers
    let state = find_referenced_identifiers(func);

    // Phase 2: Prune unreferenced instructions
    let block_ids: Vec<_> = func.body.blocks.keys().copied().collect();
    for block_id in block_ids {
        let Some(block) = func.body.blocks.get_mut(&block_id) else { continue };

        // Retain only instructions whose lvalue is referenced
        block.instructions.retain(|instr| state.is_id_or_name_used(&instr.lvalue.identifier));
    }

    // Prune unreferenced context variables
    func.context.retain(|ctx_var| state.is_id_or_name_used(&ctx_var.identifier));
}

fn find_referenced_identifiers(func: &HIRFunction) -> DceState {
    let has_loop = has_back_edge(func);
    let mut state = DceState::new();

    // Iterate in reverse postorder for better convergence
    let mut reversed_block_ids: Vec<_> = func.body.blocks.keys().copied().collect();
    reversed_block_ids.reverse();

    let mut size = state.count();
    loop {
        for &block_id in &reversed_block_ids {
            let Some(block) = func.body.blocks.get(&block_id) else { continue };

            // Terminal operands are always referenced
            for operand in each_terminal_operand(&block.terminal) {
                state.reference(&operand.identifier);
            }

            let instr_count = block.instructions.len();
            // Iterate instructions in reverse for better convergence
            for i in (0..instr_count).rev() {
                let instr = &block.instructions[i];
                let is_block_value = block.kind != BlockKind::Block && i == instr_count - 1;

                if is_block_value {
                    // The last instr of a value block is never eligible for pruning
                    state.reference(&instr.lvalue.identifier);
                    for place in each_instruction_value_operand(&instr.value) {
                        state.reference(&place.identifier);
                    }
                } else if state.is_id_or_name_used(&instr.lvalue.identifier)
                    || !pruneable_value(&instr.value, &state)
                {
                    state.reference(&instr.lvalue.identifier);

                    if let InstructionValue::StoreLocal(v) = &instr.value {
                        // For Let/Const declarations, mark initializer as referenced
                        // only if the ssa'ed lval is also referenced
                        if v.lvalue.kind == InstructionKind::Reassign
                            || state.is_id_used(&v.lvalue.place.identifier)
                        {
                            state.reference(&v.value.identifier);
                        }
                    } else {
                        for operand in each_instruction_value_operand(&instr.value) {
                            state.reference(&operand.identifier);
                        }
                    }
                }
            }
        }

        // Continue iterating only if there's a back-edge and new identifiers were found
        if state.count() <= size || !has_loop {
            break;
        }
        size = state.count();
    }

    state
}

/// Check if a value is pruneable (can be safely eliminated if unused).
fn pruneable_value(value: &InstructionValue, state: &DceState) -> bool {
    match value {
        // Side-effect-free values can always be pruned
        InstructionValue::Primitive(_)
        | InstructionValue::LoadLocal(_)
        | InstructionValue::LoadContext(_)
        | InstructionValue::LoadGlobal(_)
        | InstructionValue::PropertyLoad(_)
        | InstructionValue::ComputedLoad(_)
        | InstructionValue::BinaryExpression(_)
        | InstructionValue::UnaryExpression(_)
        | InstructionValue::TypeCastExpression(_)
        | InstructionValue::JsxText(_)
        | InstructionValue::RegExpLiteral(_)
        | InstructionValue::MetaProperty(_)
        | InstructionValue::TemplateLiteral(_) => true,

        // DeclareLocal with unreferenced lvalue can be pruned
        InstructionValue::DeclareLocal(v) => !state.is_id_or_name_used(&v.lvalue.place.identifier),

        // StoreLocal: only const/let declarations can be pruned, not reassignments
        InstructionValue::StoreLocal(v) => {
            v.lvalue.kind != InstructionKind::Reassign
                && !state.is_id_or_name_used(&v.lvalue.place.identifier)
        }

        // Most other values have side effects and cannot be pruned
        _ => false,
    }
}

/// Check if the function's CFG has any back-edges (loops).
fn has_back_edge(func: &HIRFunction) -> bool {
    let mut visited = FxHashSet::default();
    for &block_id in func.body.blocks.keys() {
        visited.insert(block_id);
        if let Some(block) = func.body.blocks.get(&block_id) {
            for pred in &block.preds {
                if !visited.contains(pred) {
                    return true;
                }
            }
        }
    }
    false
}
