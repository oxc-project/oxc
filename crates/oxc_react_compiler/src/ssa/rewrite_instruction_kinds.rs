/// Rewrite instruction kinds based on reassignment analysis.
///
/// Port of `SSA/RewriteInstructionKindsBasedOnReassignment.ts` from the React Compiler.
///
/// This pass rewrites the InstructionKind of instructions which declare/assign variables,
/// converting the first declaration to a Const/Let depending on whether it is subsequently
/// reassigned, and ensuring that subsequent reassignments are marked as Reassign.
///
/// NOTE: a `let` which was reassigned in the source may be converted to a `const` if the
/// reassignment is not used and was removed by dead code elimination.
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    compiler_error::{CompilerError, GENERATED_SOURCE},
    hir::{
        BlockId, BlockKind, DeclarationId, HIRFunction, InstructionKind, InstructionValue, Place,
        ReactiveParam, hir_builder::compute_rpo_order, visitors::each_pattern_operand,
    },
};

/// Location of the original declaration in the block/instruction list,
/// used for back-propagating `InstructionKind::Let` when a reassignment is found.
/// In the TS version, this back-propagation happens implicitly via JS reference
/// semantics (the `declarations` Map stores references to the actual LValue objects).
/// In Rust, we need to explicitly track locations and do a second pass.
struct DeclLocation {
    block_id: BlockId,
    instr_index: usize,
    /// Which variant the original declaration was
    decl_type: DeclType,
}

enum DeclType {
    DeclareLocal,
    StoreLocal,
    Destructure,
}

/// Rewrite instruction kinds based on whether variables are reassigned.
///
/// # Errors
/// Returns a `CompilerError` if any invariant is violated.
pub fn rewrite_instruction_kinds_based_on_reassignment(
    func: &mut HIRFunction,
) -> Result<(), CompilerError> {
    // Track first declaration of each variable by DeclarationId.
    // We store the location so we can back-propagate Let to the original declaration.
    let mut declarations: FxHashMap<DeclarationId, DeclLocation> = FxHashMap::default();
    // Track which declarations need back-propagation to Let
    let mut needs_let: FxHashSet<DeclarationId> = FxHashSet::default();

    // Register params — these are always Let (they can be reassigned)
    for param in &func.params {
        let place: &Place = match param {
            ReactiveParam::Place(p) => p,
            ReactiveParam::Spread(s) => &s.place,
        };
        if place.identifier.name.is_some() {
            declarations.insert(
                place.identifier.declaration_id,
                DeclLocation {
                    block_id: BlockId(u32::MAX), // sentinel — params aren't in blocks
                    instr_index: 0,
                    decl_type: DeclType::StoreLocal,
                },
            );
        }
    }

    // Register context variables
    for place in &func.context {
        if place.identifier.name.is_some() {
            declarations.insert(
                place.identifier.declaration_id,
                DeclLocation {
                    block_id: BlockId(u32::MAX),
                    instr_index: 0,
                    decl_type: DeclType::StoreLocal,
                },
            );
        }
    }

    // Pass 1: Determine kinds and mark reassignments.
    // Iterate blocks in reverse-postorder (RPO), matching the TypeScript
    // reference which uses `for (const [, block] of fn.body.blocks)` on a Map
    // that stores blocks in RPO order.  RPO guarantees that dominator blocks are
    // processed before the blocks they dominate, so a DeclareLocal is always
    // seen before any StoreLocal/reassignment in a branch or successor block.
    //
    // Sorting by block_id can produce a different order than RPO (e.g. when
    // inner functions are analysed via lower_with_mutation_aliasing, the blocks
    // may not have monotonically-increasing ids in RPO order), which causes
    // false "Expected variable not to be defined prior to declaration" errors.
    // Using compute_rpo_order here ensures correctness regardless of whether
    // reverse_postorder_blocks has been called before this pass.
    let block_ids = compute_rpo_order(func.body.entry, &func.body.blocks);
    for block_id in &block_ids {
        let block_kind = func.body.blocks.get(block_id).map(|b| b.kind);
        let Some(block) = func.body.blocks.get_mut(block_id) else { continue };

        for (instr_index, instr) in block.instructions.iter_mut().enumerate() {
            match &mut instr.value {
                InstructionValue::DeclareLocal(v) => {
                    let decl_id = v.lvalue.place.identifier.declaration_id;
                    if declarations.contains_key(&decl_id) {
                        return Err(CompilerError::invariant(
                            "Expected variable not to be defined prior to declaration",
                            None,
                            v.lvalue.place.loc,
                        ));
                    }
                    declarations.insert(
                        decl_id,
                        DeclLocation {
                            block_id: *block_id,
                            instr_index,
                            decl_type: DeclType::DeclareLocal,
                        },
                    );
                }
                InstructionValue::StoreLocal(v) => {
                    if v.lvalue.place.identifier.name.is_some() {
                        let decl_id = v.lvalue.place.identifier.declaration_id;
                        if let std::collections::hash_map::Entry::Vacant(e) =
                            declarations.entry(decl_id)
                        {
                            // First definition
                            e.insert(DeclLocation {
                                block_id: *block_id,
                                instr_index,
                                decl_type: DeclType::StoreLocal,
                            });
                            v.lvalue.kind = InstructionKind::Const;
                        } else {
                            // This is a reassignment — mark original as needing Let
                            needs_let.insert(decl_id);
                            v.lvalue.kind = InstructionKind::Reassign;
                        }
                    }
                }
                InstructionValue::Destructure(v) => {
                    let mut kind: Option<InstructionKind> = None;
                    let pattern_places: Vec<(DeclarationId, bool)> =
                        each_pattern_operand(&v.lvalue.pattern)
                            .iter()
                            .map(|p| (p.identifier.declaration_id, p.identifier.name.is_some()))
                            .collect();

                    for (decl_id, has_name) in &pattern_places {
                        if !has_name {
                            // Unnamed/temporary operand — must be consistent (Const)
                            if kind.is_some() && kind != Some(InstructionKind::Const) {
                                return Err(CompilerError::invariant(
                                    "Expected consistent kind for destructuring",
                                    Some("other places were not `Const` but this operand is const"),
                                    GENERATED_SOURCE,
                                ));
                            }
                            kind = Some(InstructionKind::Const);
                        } else if declarations.contains_key(decl_id) {
                            // Reassignment of existing declaration — must be consistent (Reassign)
                            if kind.is_some() && kind != Some(InstructionKind::Reassign) {
                                return Err(CompilerError::invariant(
                                    "Expected consistent kind for destructuring",
                                    Some(
                                        "other places were not `Reassign` but this operand is reassigned",
                                    ),
                                    GENERATED_SOURCE,
                                ));
                            }
                            needs_let.insert(*decl_id);
                            kind = Some(InstructionKind::Reassign);
                        } else {
                            // First definition — must be consistent (Const)
                            if block_kind == Some(BlockKind::Value) {
                                return Err(CompilerError::invariant(
                                    "Handle reassignment in a value block where the original declaration was removed by DCE",
                                    None,
                                    GENERATED_SOURCE,
                                ));
                            }
                            if kind.is_some() && kind != Some(InstructionKind::Const) {
                                return Err(CompilerError::invariant(
                                    "Expected consistent kind for destructuring",
                                    Some("other places were not `Const` but this operand is const"),
                                    GENERATED_SOURCE,
                                ));
                            }
                            declarations.insert(
                                *decl_id,
                                DeclLocation {
                                    block_id: *block_id,
                                    instr_index,
                                    decl_type: DeclType::Destructure,
                                },
                            );
                            kind = Some(InstructionKind::Const);
                        }
                    }

                    if let Some(k) = kind {
                        v.lvalue.kind = k;
                    } else {
                        return Err(CompilerError::invariant(
                            "Expected at least one operand",
                            None,
                            GENERATED_SOURCE,
                        ));
                    }
                }
                InstructionValue::PrefixUpdate(v) => {
                    let decl_id = v.lvalue.identifier.declaration_id;
                    if declarations.contains_key(&decl_id) {
                        needs_let.insert(decl_id);
                    } else {
                        return Err(CompilerError::invariant(
                            "Expected variable to have been defined before update",
                            None,
                            v.lvalue.loc,
                        ));
                    }
                }
                InstructionValue::PostfixUpdate(v) => {
                    let decl_id = v.lvalue.identifier.declaration_id;
                    if declarations.contains_key(&decl_id) {
                        needs_let.insert(decl_id);
                    } else {
                        return Err(CompilerError::invariant(
                            "Expected variable to have been defined before update",
                            None,
                            v.lvalue.loc,
                        ));
                    }
                }
                _ => {}
            }
        }
    }

    // Pass 2: Back-propagate Let to original declarations.
    // In TS, `declaration.kind = InstructionKind.Let` mutates the original LValue
    // via reference. In Rust, we stored the location and now update it explicitly.
    for decl_id in &needs_let {
        if let Some(loc) = declarations.get(decl_id) {
            // Skip params/context (sentinel block_id) — they're already Let
            if loc.block_id == BlockId(u32::MAX) {
                continue;
            }
            if let Some(block) = func.body.blocks.get_mut(&loc.block_id)
                && let Some(instr) = block.instructions.get_mut(loc.instr_index)
            {
                match (&mut instr.value, &loc.decl_type) {
                    (InstructionValue::DeclareLocal(v), DeclType::DeclareLocal) => {
                        v.lvalue.kind = InstructionKind::Let;
                    }
                    (InstructionValue::StoreLocal(v), DeclType::StoreLocal) => {
                        v.lvalue.kind = InstructionKind::Let;
                    }
                    (InstructionValue::Destructure(v), DeclType::Destructure) => {
                        v.lvalue.kind = InstructionKind::Let;
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}
