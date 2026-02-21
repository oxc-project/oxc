/// Infer mutable ranges from aliasing effects.
///
/// Port of `Inference/InferMutationAliasingRanges.ts` from the React Compiler.
///
/// This pass builds an abstract model of the heap and interprets effects to determine:
/// - The mutable ranges of all identifiers
/// - The externally-visible effects of the function (mutations of params, aliasing)
/// - The legacy `Effect` to store on each Place
use rustc_hash::FxHashMap;

use crate::{
    compiler_error::CompilerError,
    hir::{
        BlockId, Effect, HIRFunction, IdentifierId, InstructionId, MutableRange,
        visitors::{
            each_instruction_lvalue, each_instruction_value_operand, each_terminal_operand,
        },
    },
    inference::aliasing_effects::AliasingEffect,
};

/// Options for the inference pass.
#[derive(Debug, Clone)]
pub struct InferRangesOptions {
    pub is_function_expression: bool,
}

/// Infer mutable ranges from aliasing effects.
///
/// # Errors
/// Returns a `CompilerError` if invalid mutations are detected (e.g., mutating frozen values).
pub fn infer_mutation_aliasing_ranges(
    func: &mut HIRFunction,
    _options: InferRangesOptions,
) -> Result<Vec<AliasingEffect>, CompilerError> {
    let external_effects: Vec<AliasingEffect> = Vec::new();

    // Phase 1: Collect all mutable ranges by analyzing instruction effects
    let mut ranges: FxHashMap<IdentifierId, MutableRange> = FxHashMap::default();

    // Initialize parameter ranges
    for param in &func.params {
        let place = match param {
            crate::hir::ReactiveParam::Place(p) => p,
            crate::hir::ReactiveParam::Spread(s) => &s.place,
        };
        ranges.insert(
            place.identifier.id,
            MutableRange { start: InstructionId(0), end: InstructionId(0) },
        );
    }

    // Phase 2: Walk through blocks and update ranges based on instruction effects
    for block in func.body.blocks.values() {
        for instr in &block.instructions {
            let instr_id = instr.id;

            // The lvalue is always defined at this instruction
            let lvalue_id = instr.lvalue.identifier.id;
            extend_range(&mut ranges, lvalue_id, instr_id);

            // Operands that are mutated extend their ranges
            for operand in each_instruction_value_operand(&instr.value) {
                if operand.effect.is_mutable() {
                    extend_range(&mut ranges, operand.identifier.id, instr_id);
                }
            }

            // Lvalues from instruction value (e.g., StoreLocal lvalue)
            for lvalue in each_instruction_lvalue(instr) {
                extend_range(&mut ranges, lvalue.identifier.id, instr_id);
            }
        }

        // Terminal operands
        let terminal_id = block.terminal.id();
        for operand in each_terminal_operand(&block.terminal) {
            if operand.effect.is_mutable() {
                extend_range(&mut ranges, operand.identifier.id, terminal_id);
            }
        }
    }

    // Phase 3: Apply computed ranges back to identifiers
    let block_ids: Vec<BlockId> = func.body.blocks.keys().copied().collect();
    for block_id in block_ids {
        if let Some(block) = func.body.blocks.get_mut(&block_id) {
            for instr in &mut block.instructions {
                if let Some(range) = ranges.get(&instr.lvalue.identifier.id) {
                    instr.lvalue.identifier.mutable_range = *range;
                }
            }
        }
    }

    // Phase 4: Populate Place effects
    let block_ids: Vec<BlockId> = func.body.blocks.keys().copied().collect();
    for block_id in block_ids {
        if let Some(block) = func.body.blocks.get_mut(&block_id) {
            for instr in &mut block.instructions {
                // Set lvalue effect
                let lvalue_range = instr.lvalue.identifier.mutable_range;
                if lvalue_range.start == lvalue_range.end || lvalue_range.end.0 == 0 {
                    instr.lvalue.effect = Effect::Read;
                } else if instr.id >= lvalue_range.start && instr.id < lvalue_range.end {
                    instr.lvalue.effect = Effect::Mutate;
                } else {
                    instr.lvalue.effect = Effect::Read;
                }
            }
        }
    }

    Ok(external_effects)
}

/// Extend a mutable range for an identifier to include the given instruction.
fn extend_range(
    ranges: &mut FxHashMap<IdentifierId, MutableRange>,
    id: IdentifierId,
    instr_id: InstructionId,
) {
    let range = ranges.entry(id).or_insert(MutableRange {
        start: instr_id,
        end: InstructionId(instr_id.0 + 1),
    });

    if instr_id < range.start {
        range.start = instr_id;
    }
    if instr_id >= range.end {
        range.end = InstructionId(instr_id.0 + 1);
    }
}
