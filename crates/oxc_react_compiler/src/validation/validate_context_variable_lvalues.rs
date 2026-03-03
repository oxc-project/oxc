/// Validate context variable lvalues.
///
/// Port of `Validation/ValidateContextVariableLValues.ts` from the React Compiler.
///
/// Validates that all store/load references to a given named identifier align
/// with the "kind" of that variable (normal variable or context variable).
use rustc_hash::FxHashMap;

use crate::{
    compiler_error::{CompilerError, GENERATED_SOURCE},
    hir::{HIRFunction, IdentifierId, InstructionValue},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IdentifierKindTag {
    Local,
    Context,
}

/// Validate that context variable lvalues are consistent.
///
/// # Errors
/// Returns a `CompilerError` if inconsistent variable kinds are found.
pub fn validate_context_variable_lvalues(func: &HIRFunction) -> Result<(), CompilerError> {
    let mut identifier_kinds: FxHashMap<IdentifierId, IdentifierKindTag> = FxHashMap::default();

    for block in func.body.blocks.values() {
        for instr in &block.instructions {
            match &instr.value {
                InstructionValue::DeclareContext(v) => {
                    visit(
                        &mut identifier_kinds,
                        v.lvalue_place.identifier.id,
                        IdentifierKindTag::Context,
                    )?;
                }
                InstructionValue::StoreContext(v) => {
                    visit(
                        &mut identifier_kinds,
                        v.lvalue_place.identifier.id,
                        IdentifierKindTag::Context,
                    )?;
                }
                InstructionValue::LoadContext(v) => {
                    visit(
                        &mut identifier_kinds,
                        v.place.identifier.id,
                        IdentifierKindTag::Context,
                    )?;
                }
                InstructionValue::StoreLocal(v) => {
                    if v.lvalue.place.identifier.name.is_some() {
                        visit(
                            &mut identifier_kinds,
                            v.lvalue.place.identifier.id,
                            IdentifierKindTag::Local,
                        )?;
                    }
                }
                InstructionValue::DeclareLocal(v) => {
                    if v.lvalue.place.identifier.name.is_some() {
                        visit(
                            &mut identifier_kinds,
                            v.lvalue.place.identifier.id,
                            IdentifierKindTag::Local,
                        )?;
                    }
                }
                InstructionValue::LoadLocal(v) => {
                    if v.place.identifier.name.is_some() {
                        visit(
                            &mut identifier_kinds,
                            v.place.identifier.id,
                            IdentifierKindTag::Local,
                        )?;
                    }
                }
                // NOTE: The TS implementation recurses into FunctionExpression
                // and ObjectMethod with a shared identifierKinds map. We cannot
                // do the same because our Rust implementation uses different
                // IdentifierIds for inner functions (each function gets its own
                // HirBuilder with separate ID space). Sharing the map would
                // cause false conflicts between outer and inner function IDs.
                _ => {}
            }
        }
    }
    Ok(())
}

fn visit(
    kinds: &mut FxHashMap<IdentifierId, IdentifierKindTag>,
    id: IdentifierId,
    expected: IdentifierKindTag,
) -> Result<(), CompilerError> {
    if let Some(&existing) = kinds.get(&id) {
        if existing != expected {
            return Err(CompilerError::invariant(
                "Expected all references to a variable to be consistently local or context references",
                Some(&format!(
                    "Identifier #{} is referenced as a {} variable, but was previously referenced as a {} variable.",
                    id.0,
                    kind_tag_name(expected),
                    kind_tag_name(existing),
                )),
                GENERATED_SOURCE,
            ));
        }
    } else {
        kinds.insert(id, expected);
    }
    Ok(())
}

fn kind_tag_name(tag: IdentifierKindTag) -> &'static str {
    match tag {
        IdentifierKindTag::Local => "local",
        IdentifierKindTag::Context => "context",
    }
}
