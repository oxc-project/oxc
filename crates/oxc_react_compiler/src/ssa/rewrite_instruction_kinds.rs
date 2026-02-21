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
use rustc_hash::FxHashMap;

use crate::{
    compiler_error::{CompilerError, GENERATED_SOURCE},
    hir::{
        BlockKind, DeclarationId, HIRFunction, InstructionKind, InstructionValue, Place,
        ReactiveParam,
        visitors::each_pattern_operand,
    },
};

/// Rewrite instruction kinds based on whether variables are reassigned.
///
/// # Errors
/// Returns a `CompilerError` if any invariant is violated.
pub fn rewrite_instruction_kinds_based_on_reassignment(
    func: &mut HIRFunction,
) -> Result<(), CompilerError> {
    // Track first declaration of each variable by DeclarationId
    let mut declarations: FxHashMap<DeclarationId, InstructionKind> = FxHashMap::default();

    // Register params
    for param in &func.params {
        let place: &Place = match param {
            ReactiveParam::Place(p) => p,
            ReactiveParam::Spread(s) => &s.place,
        };
        if place.identifier.name.is_some() {
            declarations.insert(place.identifier.declaration_id, InstructionKind::Let);
        }
    }

    // Register context variables
    for place in &func.context {
        if place.identifier.name.is_some() {
            declarations.insert(place.identifier.declaration_id, InstructionKind::Let);
        }
    }

    // Process all blocks
    let block_ids: Vec<_> = func.body.blocks.keys().copied().collect();
    for block_id in block_ids {
        let block_kind = func.body.blocks.get(&block_id).map(|b| b.kind);
        let block = match func.body.blocks.get_mut(&block_id) {
            Some(b) => b,
            None => continue,
        };

        for instr in &mut block.instructions {
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
                    declarations.insert(decl_id, v.lvalue.kind);
                }
                InstructionValue::StoreLocal(v) => {
                    if v.lvalue.place.identifier.name.is_some() {
                        let decl_id = v.lvalue.place.identifier.declaration_id;
                        if let Some(existing_kind) = declarations.get_mut(&decl_id) {
                            // This is a reassignment
                            *existing_kind = InstructionKind::Let;
                            v.lvalue.kind = InstructionKind::Reassign;
                        } else {
                            // First definition
                            declarations.insert(decl_id, InstructionKind::Const);
                            v.lvalue.kind = InstructionKind::Const;
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
                            kind = Some(InstructionKind::Const);
                        } else if declarations.contains_key(decl_id) {
                            // Reassignment of existing declaration
                            if let Some(existing_kind) = declarations.get_mut(decl_id) {
                                *existing_kind = InstructionKind::Let;
                            }
                            kind = Some(InstructionKind::Reassign);
                        } else {
                            // First definition
                            if block_kind == Some(BlockKind::Value) {
                                return Err(CompilerError::invariant(
                                    "Handle reassignment in a value block where the original declaration was removed by DCE",
                                    None,
                                    GENERATED_SOURCE,
                                ));
                            }
                            declarations.insert(*decl_id, InstructionKind::Const);
                            kind = Some(InstructionKind::Const);
                        }
                    }

                    if let Some(k) = kind {
                        v.lvalue.kind = k;
                    } else {
                        return Err(CompilerError::invariant(
                            "Expected at least one operand in destructure",
                            None,
                            GENERATED_SOURCE,
                        ));
                    }
                }
                InstructionValue::PrefixUpdate(v) => {
                    let decl_id = v.lvalue.identifier.declaration_id;
                    if let Some(existing_kind) = declarations.get_mut(&decl_id) {
                        *existing_kind = InstructionKind::Let;
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
                    if let Some(existing_kind) = declarations.get_mut(&decl_id) {
                        *existing_kind = InstructionKind::Let;
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

    Ok(())
}
