/// Extract scope declarations from destructuring patterns.
///
/// Port of `ReactiveScopes/ExtractScopeDeclarationsFromDestructuring.ts` from the React Compiler.
///
/// When a destructuring pattern produces variables that belong to different
/// reactive scopes, this pass splits the destructuring into individual
/// assignments so each can be placed in the correct scope.
///
/// Destructuring statements may sometimes define some variables which are declared
/// by the scope, and others that are only used locally within the scope. This pass
/// finds destructuring instructions that contain mixed values, and rewrites them
/// to ensure that any scope variable assignments are extracted first to a temporary
/// and reassigned in a separate instruction.
use rustc_hash::FxHashSet;

use crate::hir::{
    DeclarationId, Identifier, IdentifierId, IdentifierName, InstructionKind, InstructionValue,
    LValue, Place, ReactiveBlock, ReactiveFunction, ReactiveInstruction,
    ReactiveInstructionStatement, ReactiveStatement, ReactiveTerminal, ReactiveValue, StoreLocal,
    environment::Environment, hir_builder::make_temporary_identifier,
    visitors::each_pattern_operand,
};

/// Extract scope declarations from destructuring patterns.
pub fn extract_scope_declarations_from_destructuring(
    func: &mut ReactiveFunction,
    env: &mut Environment,
) {
    let mut declared: FxHashSet<DeclarationId> = FxHashSet::default();

    // Initialize with function parameters
    for param in &func.params {
        let place = match param {
            crate::hir::ReactiveParam::Place(p) => p,
            crate::hir::ReactiveParam::Spread(s) => &s.place,
        };
        declared.insert(place.identifier.declaration_id);
    }

    extract_in_block(&mut func.body, &mut declared, env);
}

fn extract_in_block(
    block: &mut ReactiveBlock,
    declared: &mut FxHashSet<DeclarationId>,
    env: &mut Environment,
) {
    let mut i = 0;
    while i < block.len() {
        match &mut block[i] {
            ReactiveStatement::Scope(scope) => {
                // When entering a scope, add scope declarations to declared
                for declaration in scope.scope.declarations.values() {
                    declared.insert(declaration.identifier.declaration_id);
                }
                extract_in_block(&mut scope.instructions, declared, env);
                i += 1;
            }
            ReactiveStatement::PrunedScope(scope) => {
                // Pruned scopes do NOT add declarations to the declared set
                // (only active scopes do, matching the TS visitScope override)
                extract_in_block(&mut scope.instructions, declared, env);
                i += 1;
            }
            ReactiveStatement::Terminal(term) => {
                extract_in_terminal(&mut term.terminal, declared, env);
                i += 1;
            }
            ReactiveStatement::Instruction(instr_stmt) => {
                // Check if this is a Destructure instruction
                let transform_result =
                    try_transform_destructure(&mut instr_stmt.instruction, declared, env);

                match transform_result {
                    DestructureTransformResult::Keep => {
                        // Update declared with lvalues from the instruction
                        update_declared_from_instruction(&instr_stmt.instruction, declared);
                        i += 1;
                    }
                    DestructureTransformResult::ReplaceMany(instructions) => {
                        // Update declared from all replacement instructions
                        for instr in &instructions {
                            update_declared_from_instruction(instr, declared);
                        }
                        // Remove original statement and insert replacements
                        block.remove(i);
                        let count = instructions.len();
                        for (j, instr) in instructions.into_iter().enumerate() {
                            block.insert(
                                i + j,
                                ReactiveStatement::Instruction(ReactiveInstructionStatement {
                                    instruction: instr,
                                }),
                            );
                        }
                        i += count;
                    }
                }
            }
        }
    }
}

fn extract_in_terminal(
    terminal: &mut ReactiveTerminal,
    declared: &mut FxHashSet<DeclarationId>,
    env: &mut Environment,
) {
    match terminal {
        ReactiveTerminal::If(t) => {
            extract_in_block(&mut t.consequent, declared, env);
            if let Some(alt) = &mut t.alternate {
                extract_in_block(alt, declared, env);
            }
        }
        ReactiveTerminal::Switch(t) => {
            for case in &mut t.cases {
                if let Some(block) = &mut case.block {
                    extract_in_block(block, declared, env);
                }
            }
        }
        ReactiveTerminal::While(t) => extract_in_block(&mut t.r#loop, declared, env),
        ReactiveTerminal::DoWhile(t) => extract_in_block(&mut t.r#loop, declared, env),
        ReactiveTerminal::For(t) => extract_in_block(&mut t.r#loop, declared, env),
        ReactiveTerminal::ForOf(t) => extract_in_block(&mut t.r#loop, declared, env),
        ReactiveTerminal::ForIn(t) => extract_in_block(&mut t.r#loop, declared, env),
        ReactiveTerminal::Label(t) => extract_in_block(&mut t.block, declared, env),
        ReactiveTerminal::Try(t) => {
            extract_in_block(&mut t.block, declared, env);
            extract_in_block(&mut t.handler, declared, env);
        }
        ReactiveTerminal::Break(_)
        | ReactiveTerminal::Continue(_)
        | ReactiveTerminal::Return(_)
        | ReactiveTerminal::Throw(_) => {}
    }
}

enum DestructureTransformResult {
    /// Keep the instruction as-is (possibly mutated in place for all-reassignment case).
    Keep,
    /// Replace the instruction with multiple instructions.
    ReplaceMany(Vec<ReactiveInstruction>),
}

/// Try to transform a destructure instruction. Returns the transform result.
fn try_transform_destructure(
    instr: &mut ReactiveInstruction,
    declared: &FxHashSet<DeclarationId>,
    env: &mut Environment,
) -> DestructureTransformResult {
    // Only handle Destructure instruction values
    let ReactiveValue::Instruction(ref mut value) = instr.value else {
        return DestructureTransformResult::Keep;
    };
    let InstructionValue::Destructure(ref mut destructure) = **value else {
        return DestructureTransformResult::Keep;
    };

    // Determine which pattern operands are reassigned vs declared
    let mut reassigned: FxHashSet<IdentifierId> = FxHashSet::default();
    let mut has_declaration = false;

    for place in each_pattern_operand(&destructure.lvalue.pattern) {
        if declared.contains(&place.identifier.declaration_id) {
            reassigned.insert(place.identifier.id);
        } else {
            has_declaration = true;
        }
    }

    if reassigned.is_empty() {
        // All are new declarations, nothing to do
        return DestructureTransformResult::Keep;
    }

    if !has_declaration {
        // All are reassignments: change destructure's lvalue.kind to Reassign
        destructure.lvalue.kind = InstructionKind::Reassign;
        return DestructureTransformResult::Keep;
    }

    // Mixed case: replace reassigned items in the destructuring pattern with
    // temporaries and emit separate StoreLocal(Reassign) instructions
    let destructure_loc = destructure.loc;
    let instr_id = instr.id;
    let instr_loc = instr.loc;

    // Collect the operands that need to be replaced (reassigned ones)
    // We need to map pattern operands: for each reassigned place, replace with temporary
    let mut renamed: Vec<(Place, Place)> = Vec::new();

    crate::hir::visitors::map_pattern_operands(
        &mut destructure.lvalue.pattern,
        &mut |place: Place| {
            if !reassigned.contains(&place.identifier.id) {
                return place;
            }
            // Clone place to temporary and promote it
            let mut temporary = clone_place_to_temporary(env, &place);
            promote_temporary(&mut temporary);
            renamed.push((place, temporary.clone()));
            temporary
        },
    );

    // Build the result instructions: first the original destructure (now with temporaries),
    // then StoreLocal(Reassign) for each renamed pair
    let mut instructions: Vec<ReactiveInstruction> = Vec::new();

    // The original instruction (destructure now has temporary places in the pattern)
    // We need to take the original instruction as-is since we already mutated its pattern
    instructions.push(ReactiveInstruction {
        id: instr.id,
        lvalue: instr.lvalue.clone(),
        value: instr.value.clone(),
        loc: instr.loc,
    });

    // Emit StoreLocal(Reassign, original, temporary) for each renamed pair
    for (original, temporary) in renamed {
        instructions.push(ReactiveInstruction {
            id: instr_id,
            lvalue: None,
            value: ReactiveValue::Instruction(Box::new(InstructionValue::StoreLocal(StoreLocal {
                lvalue: LValue { kind: InstructionKind::Reassign, place: original },
                value: temporary,
                loc: destructure_loc,
            }))),
            loc: instr_loc,
        });
    }

    DestructureTransformResult::ReplaceMany(instructions)
}

/// Update the declared set with lvalue places from an instruction (non-Reassign only).
///
/// Port of the `eachInstructionLValueWithKind` loop in the TS Visitor.transformInstruction.
fn update_declared_from_instruction(
    instr: &ReactiveInstruction,
    declared: &mut FxHashSet<DeclarationId>,
) {
    let ReactiveValue::Instruction(ref value) = instr.value else {
        return;
    };
    match &**value {
        InstructionValue::DeclareLocal(v) => {
            if v.lvalue.kind != InstructionKind::Reassign {
                declared.insert(v.lvalue.place.identifier.declaration_id);
            }
        }
        InstructionValue::DeclareContext(v) => {
            if v.lvalue_kind != InstructionKind::Reassign {
                declared.insert(v.lvalue_place.identifier.declaration_id);
            }
        }
        InstructionValue::StoreLocal(v) => {
            if v.lvalue.kind != InstructionKind::Reassign {
                declared.insert(v.lvalue.place.identifier.declaration_id);
            }
        }
        InstructionValue::StoreContext(v) => {
            if v.lvalue_kind != InstructionKind::Reassign {
                declared.insert(v.lvalue_place.identifier.declaration_id);
            }
        }
        InstructionValue::Destructure(v) => {
            let kind = v.lvalue.kind;
            if kind != InstructionKind::Reassign {
                for place in each_pattern_operand(&v.lvalue.pattern) {
                    declared.insert(place.identifier.declaration_id);
                }
            }
        }
        _ => {}
    }
}

/// Clone a place to a new temporary place.
///
/// Port of `clonePlaceToTemporary` from HIRBuilder.ts.
/// Creates a temporary Place with `GENERATED_SOURCE` as its loc (matching the TS
/// `createTemporaryPlace`), then copies `effect`, `identifier.type`, and `reactive`
/// from the source place.
fn clone_place_to_temporary(env: &mut Environment, place: &Place) -> Place {
    let id = env.next_identifier_id();
    let temp_identifier = make_temporary_identifier(id, place.loc);
    Place {
        identifier: Identifier { type_: place.identifier.type_.clone(), ..temp_identifier },
        effect: place.effect,
        reactive: place.reactive,
        loc: crate::compiler_error::GENERATED_SOURCE,
    }
}

/// Promote a temporary identifier to a named identifier.
///
/// Port of `promoteTemporary` from HIR.ts.
fn promote_temporary(place: &mut Place) {
    let decl_id = place.identifier.declaration_id.0;
    place.identifier.name = Some(IdentifierName::Promoted(format!("#t{decl_id}")));
}
