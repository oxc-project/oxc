/// Dead code elimination optimization pass.
///
/// Port of `Optimization/DeadCodeElimination.ts` from the React Compiler.
///
/// Eliminates instructions whose values are unused. Uses a two-phase approach:
/// 1. Find/mark all referenced identifiers (fixpoint iteration in postorder)
/// 2. Prune unreferenced instructions and rewrite destructuring patterns
use rustc_hash::FxHashSet;

use crate::hir::{
    ArrayPatternElement, BlockKind, DeclareLocal, HIRFunction, Identifier, IdentifierId,
    InstructionKind, InstructionValue, ObjectPatternProperty, Pattern,
    visitors::{each_instruction_value_operand, each_pattern_operand, each_terminal_operand},
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

    // Phase 2: Rewrite destructure patterns to remove unused lvalues, then prune instructions
    let block_ids: Vec<_> = func.body.blocks.keys().copied().collect();
    for block_id in block_ids {
        let Some(block) = func.body.blocks.get_mut(&block_id) else { continue };

        // Prune unused phi nodes (TS lines 45-49)
        block.phis.retain(|phi| state.is_id_or_name_used(&phi.place.identifier));

        // Retain only instructions whose lvalue is referenced
        block.instructions.retain(|instr| state.is_id_or_name_used(&instr.lvalue.identifier));

        // Rewrite retained instructions (but skip block-value instructions)
        let instr_count = block.instructions.len();
        for i in 0..instr_count {
            let is_block_value = block.kind != BlockKind::Block && i == instr_count - 1;
            if !is_block_value {
                rewrite_instruction(&mut block.instructions[i], &state);
            }
        }
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

            // Mark phi operands as referenced if the phi output is used.
            // Port of TS lines 175-181: if a phi's output place is referenced,
            // then all its operand identifiers must also be referenced.
            for phi in &block.phis {
                if state.is_id_or_name_used(&phi.place.identifier) {
                    for operand in phi.operands.values() {
                        state.reference(&operand.identifier);
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

/// Rewrite destructure patterns to remove unused lvalues.
///
/// Port of `rewriteInstruction` from `DeadCodeElimination.ts` (lines 187-252).
///
/// For array patterns, unused items before the end are replaced with holes;
/// trailing unused items are dropped entirely.
///
/// For object patterns, unused properties can be pruned as long as there is no
/// used rest/spread element. If a rest element exists and is used, nothing can
/// be pruned because removing properties would change the set of properties
/// copied into the rest value.
fn rewrite_instruction(instr: &mut crate::hir::Instruction, state: &DceState) {
    if let InstructionValue::Destructure(v) = &mut instr.value {
        match &mut v.lvalue.pattern {
            Pattern::Array(arr) => {
                // For arrays, we can prune items prior to the end by replacing
                // them with a hole. Items at the end can simply be dropped.
                let mut last_entry_index: usize = 0;
                for i in 0..arr.items.len() {
                    let should_remove = match &arr.items[i] {
                        ArrayPatternElement::Place(p) => !state.is_id_or_name_used(&p.identifier),
                        ArrayPatternElement::Spread(s) => {
                            !state.is_id_or_name_used(&s.place.identifier)
                        }
                        ArrayPatternElement::Hole => true,
                    };
                    if should_remove {
                        arr.items[i] = ArrayPatternElement::Hole;
                    } else {
                        last_entry_index = i;
                    }
                }
                if !arr.items.is_empty() {
                    arr.items.truncate(last_entry_index + 1);
                }
            }
            Pattern::Object(obj) => {
                // For objects we can prune any unused properties so long as there
                // is no used rest element. If a rest element exists and is used,
                // then nothing can be pruned because it would change the set of
                // properties which are copied into the rest value.
                let mut next_properties: Option<Vec<ObjectPatternProperty>> = None;
                for prop in &obj.properties {
                    match prop {
                        ObjectPatternProperty::Property(p) => {
                            if state.is_id_or_name_used(&p.place.identifier) {
                                next_properties.get_or_insert_with(Vec::new).push(prop.clone());
                            }
                        }
                        ObjectPatternProperty::Spread(s) => {
                            if state.is_id_or_name_used(&s.place.identifier) {
                                // Used rest element — don't prune anything
                                next_properties = None;
                                break;
                            }
                        }
                    }
                }
                if let Some(new_props) = next_properties {
                    obj.properties = new_props;
                }
            }
        }
    } else if let InstructionValue::StoreLocal(v) = &instr.value {
        // Port of TS lines 253-270: StoreLocal -> DeclareLocal rewrite.
        // If this is a const/let declaration where the variable is accessed later,
        // but the value is always overwritten before being read (i.e. the initializer
        // value is never read), we rewrite to a DeclareLocal so that the initializer
        // value can be DCE'd.
        if v.lvalue.kind != InstructionKind::Reassign
            && !state.is_id_used(&v.lvalue.place.identifier)
        {
            let lvalue = v.lvalue.clone();
            let loc = v.loc;
            instr.value = InstructionValue::DeclareLocal(DeclareLocal { lvalue, loc });
        }
    }
}

/// Check if a value is pruneable (can be safely eliminated if unused).
fn pruneable_value(value: &InstructionValue, state: &DceState) -> bool {
    match value {
        // Side-effect-free values can always be pruned (read-only)
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
        | InstructionValue::TemplateLiteral(_)
        | InstructionValue::ArrayExpression(_)
        | InstructionValue::ObjectExpression(_)
        | InstructionValue::ObjectMethod(_)
        | InstructionValue::FunctionExpression(_)
        | InstructionValue::JsxExpression(_)
        | InstructionValue::JsxFragment(_) => true,

        // DeclareLocal with unreferenced lvalue can be pruned
        InstructionValue::DeclareLocal(v) => !state.is_id_or_name_used(&v.lvalue.place.identifier),

        // StoreLocal: Reassignments can be pruned if the specific instance being
        // assigned is never read. Declarations are pruneable only if the named
        // variable is never read later.
        InstructionValue::StoreLocal(v) => {
            if v.lvalue.kind == InstructionKind::Reassign {
                !state.is_id_used(&v.lvalue.place.identifier)
            } else {
                !state.is_id_or_name_used(&v.lvalue.place.identifier)
            }
        }

        // Destructure: pruneable if all pattern operands are unused
        InstructionValue::Destructure(v) => {
            let mut is_id_or_name_used = false;
            let mut is_id_used = false;
            for place in each_pattern_operand(&v.lvalue.pattern) {
                if state.is_id_used(&place.identifier) {
                    is_id_or_name_used = true;
                    is_id_used = true;
                } else if state.is_id_or_name_used(&place.identifier) {
                    is_id_or_name_used = true;
                }
            }
            if v.lvalue.kind == InstructionKind::Reassign {
                // Reassignments can be pruned if the specific instance being assigned is never read
                !is_id_used
            } else {
                // Otherwise pruneable only if none of the identifiers are read from later
                !is_id_or_name_used
            }
        }

        // Updates are pruneable if the specific instance being assigned is never read
        InstructionValue::PostfixUpdate(v) => !state.is_id_used(&v.lvalue.identifier),
        InstructionValue::PrefixUpdate(v) => !state.is_id_used(&v.lvalue.identifier),

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
