// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

//! Rewrites InstructionKind of instructions which declare/assign variables,
//! converting the first declaration to Const/Let depending on whether it is
//! subsequently reassigned, and ensuring that subsequent reassignments are
//! marked as Reassign.
//!
//! Ported from TypeScript `src/SSA/RewriteInstructionKindsBasedOnReassignment.ts`.
//!
//! Note that declarations which were const in the original program cannot become
//! `let`, but the inverse is not true: a `let` which was reassigned in the source
//! may be converted to a `const` if the reassignment is not used and was removed
//! by dead code elimination.

use std::borrow::Cow;

use rustc_hash::FxHashMap;

use oxc_diagnostics::OxcDiagnostic;

use crate::diagnostics::ErrorCategory;
use crate::react_compiler_hir::visitors::each_pattern_operand;
use crate::react_compiler_hir::{
    BlockKind, DeclarationId, HirFunction, InstructionKind, InstructionValue, ParamPattern, Place,
};
use oxc_span::Span;

use crate::react_compiler_hir::environment::Environment;

/// Create an invariant diagnostic (matches TS CompilerError.invariant).
/// When a span is provided, it is attached as a labelled span
/// (matching TS CompilerError.invariant which uses .withDetails()).
fn invariant_error(reason: &str, description: Option<String>) -> OxcDiagnostic {
    invariant_error_with_span(reason, description, None)
}

fn invariant_error_with_span(
    reason: &str,
    description: Option<String>,
    span: Option<Span>,
) -> OxcDiagnostic {
    let mut diagnostic =
        ErrorCategory::Invariant.diagnostic(reason).with_labels(span.map(|s| s.label(reason)));
    if let Some(description) = description {
        diagnostic = diagnostic.with_help(description);
    }
    diagnostic
}

/// Format an InstructionKind variant name (matches TS `${kind}` interpolation).
fn format_kind(kind: Option<InstructionKind>) -> &'static str {
    match kind {
        Some(InstructionKind::Const) => "Const",
        Some(InstructionKind::Let) => "Let",
        Some(InstructionKind::Reassign) => "Reassign",
        Some(InstructionKind::Catch) => "Catch",
        Some(InstructionKind::HoistedConst) => "HoistedConst",
        Some(InstructionKind::HoistedLet) => "HoistedLet",
        Some(InstructionKind::HoistedFunction) => "HoistedFunction",
        Some(InstructionKind::Function) => "Function",
        None => "null",
    }
}

/// Format a Place like TS `printPlace()`: `<effect> <name>$<id>[<range>]{reactive}`
fn format_place(place: &Place, env: &Environment) -> String {
    let ident = &env.identifiers[place.identifier.index()];
    let name = ident.name.as_ref().map_or("", |name| name.value());
    let scope = ident
        .scope
        .map_or(Cow::Borrowed(""), |scope_id| Cow::Owned(format!("_@{}", scope_id.index())));
    let mutable_range = if ident.mutable_range.end > ident.mutable_range.start + 1 {
        Cow::Owned(format!(
            "[{}:{}]",
            ident.mutable_range.start.index(),
            ident.mutable_range.end.index()
        ))
    } else {
        Cow::Borrowed("")
    };
    let reactive = if place.reactive { "{reactive}" } else { "" };
    format!(
        "{} {}${}{}{}{}",
        place.effect,
        name,
        place.identifier.index(),
        scope,
        mutable_range,
        reactive
    )
}

/// Index into a collected list of declaration mutations to apply.
///
/// We use a two-phase approach: first collect which declarations exist,
/// then apply mutations. This is because in the TS code, `declarations`
/// map stores references to LValue/LValuePattern and mutates `kind` through them.
/// In Rust, we track instruction indices and apply changes in a second pass.
enum DeclarationLoc {
    /// An LValue from DeclareLocal or StoreLocal — identified by (block_index, instr_index_in_block)
    Instruction { block_index: usize, instr_local_index: usize },
    /// A parameter or context variable (seeded as Let, may be upgraded to Let on reassignment — already Let)
    ParamOrContext,
}

pub fn rewrite_instruction_kinds_based_on_reassignment(
    func: &mut HirFunction,
    env: &Environment,
) -> Result<(), OxcDiagnostic> {
    // Phase 1: Collect all information about which declarations need updates.
    //
    // Track: for each DeclarationId, the location of its first declaration,
    // and whether it needs to be changed to Let (because of reassignment).
    let mut declarations: FxHashMap<DeclarationId, DeclarationLoc> = FxHashMap::default();
    // Track which (block_index, instr_local_index) should have their lvalue.kind set to Reassign
    let mut reassign_spans: Vec<(usize, usize)> = Vec::new();
    // Track which declaration locations need to be set to Let
    let mut let_spans: Vec<(usize, usize)> = Vec::new();
    // Track which (block_index, instr_local_index) should have their lvalue.kind set to Const
    let mut const_spans: Vec<(usize, usize)> = Vec::new();
    // Track which (block_index, instr_local_index) Destructure instructions get a specific kind
    let mut destructure_kind_spans: Vec<(usize, usize, InstructionKind)> = Vec::new();

    // Seed with parameters
    for param in &func.params {
        let place: &Place = match param {
            ParamPattern::Place(p) => p,
            ParamPattern::Spread(s) => &s.place,
        };
        let ident = &env.identifiers[place.identifier.index()];
        if ident.name.is_some() {
            declarations.insert(ident.declaration_id, DeclarationLoc::ParamOrContext);
        }
    }

    // Seed with context variables
    for place in &func.context {
        let ident = &env.identifiers[place.identifier.index()];
        if ident.name.is_some() {
            declarations.insert(ident.declaration_id, DeclarationLoc::ParamOrContext);
        }
    }

    // Process all blocks
    let block_keys: Vec<_> = func.body.blocks.keys().cloned().collect();
    for (block_index, block_id) in block_keys.iter().enumerate() {
        let block = &func.body.blocks[block_id];
        let block_kind = block.kind;
        for (local_idx, instr_id) in block.instructions.iter().enumerate() {
            let instr = &func.instructions[instr_id.index()];
            match &instr.value {
                InstructionValue::DeclareLocal { lvalue, .. } => {
                    let decl_id = env.identifiers[lvalue.place.identifier.index()].declaration_id;
                    if declarations.contains_key(&decl_id) {
                        return Err(invariant_error_with_span(
                            "Expected variable not to be defined prior to declaration",
                            Some(format!(
                                "{} was already defined",
                                format_place(&lvalue.place, env),
                            )),
                            lvalue.place.span,
                        ));
                    }
                    declarations.insert(
                        decl_id,
                        DeclarationLoc::Instruction { block_index, instr_local_index: local_idx },
                    );
                }
                InstructionValue::StoreLocal { lvalue, .. } => {
                    let ident = &env.identifiers[lvalue.place.identifier.index()];
                    if ident.name.is_some() {
                        let decl_id = ident.declaration_id;
                        if let Some(existing) = declarations.get(&decl_id) {
                            // Reassignment: mark existing declaration as Let, current as Reassign
                            match existing {
                                DeclarationLoc::Instruction {
                                    block_index: bi,
                                    instr_local_index: ili,
                                } => {
                                    let_spans.push((*bi, *ili));
                                }
                                DeclarationLoc::ParamOrContext => {
                                    // Already Let, no-op
                                }
                            }
                            reassign_spans.push((block_index, local_idx));
                        } else {
                            // First store — mark as Const
                            // Mirrors TS: Diagnostics.invariant(!declarations.has(...))
                            if declarations.contains_key(&decl_id) {
                                return Err(invariant_error_with_span(
                                    "Expected variable not to be defined prior to declaration",
                                    Some(format!(
                                        "{} was already defined",
                                        format_place(&lvalue.place, env),
                                    )),
                                    lvalue.place.span,
                                ));
                            }
                            declarations.insert(
                                decl_id,
                                DeclarationLoc::Instruction {
                                    block_index,
                                    instr_local_index: local_idx,
                                },
                            );
                            const_spans.push((block_index, local_idx));
                        }
                    }
                }
                InstructionValue::Destructure { lvalue, .. } => {
                    let mut kind: Option<InstructionKind> = None;
                    for place in each_pattern_operand(&lvalue.pattern) {
                        let ident = &env.identifiers[place.identifier.index()];
                        if ident.name.is_none() {
                            if !(kind.is_none() || kind == Some(InstructionKind::Const)) {
                                return Err(invariant_error_with_span(
                                    "Expected consistent kind for destructuring",
                                    Some(format!(
                                        "other places were `{}` but '{}' is const",
                                        format_kind(kind),
                                        format_place(&place, env),
                                    )),
                                    place.span,
                                ));
                            }
                            kind = Some(InstructionKind::Const);
                        } else {
                            let decl_id = ident.declaration_id;
                            if let Some(existing) = declarations.get(&decl_id) {
                                // Reassignment
                                if !(kind.is_none() || kind == Some(InstructionKind::Reassign)) {
                                    return Err(invariant_error_with_span(
                                        "Expected consistent kind for destructuring",
                                        Some(format!(
                                            "Other places were `{}` but '{}' is reassigned",
                                            format_kind(kind),
                                            format_place(&place, env),
                                        )),
                                        place.span,
                                    ));
                                }
                                kind = Some(InstructionKind::Reassign);
                                match existing {
                                    DeclarationLoc::Instruction {
                                        block_index: bi,
                                        instr_local_index: ili,
                                    } => {
                                        let_spans.push((*bi, *ili));
                                    }
                                    DeclarationLoc::ParamOrContext => {
                                        // Already Let
                                    }
                                }
                            } else {
                                // New declaration
                                if block_kind == BlockKind::Value {
                                    return Err(invariant_error_with_span(
                                        "TODO: Handle reassignment in a value block where the original declaration was removed by dead code elimination (DCE)",
                                        None,
                                        place.span,
                                    ));
                                }
                                declarations.insert(
                                    decl_id,
                                    DeclarationLoc::Instruction {
                                        block_index,
                                        instr_local_index: local_idx,
                                    },
                                );
                                if !(kind.is_none() || kind == Some(InstructionKind::Const)) {
                                    return Err(invariant_error_with_span(
                                        "Expected consistent kind for destructuring",
                                        Some(format!(
                                            "Other places were `{}` but '{}' is const",
                                            format_kind(kind),
                                            format_place(&place, env),
                                        )),
                                        place.span,
                                    ));
                                }
                                kind = Some(InstructionKind::Const);
                            }
                        }
                    }
                    let kind =
                        kind.ok_or_else(|| invariant_error("Expected at least one operand", None))?;
                    destructure_kind_spans.push((block_index, local_idx, kind));
                }
                InstructionValue::PostfixUpdate { lvalue, .. }
                | InstructionValue::PrefixUpdate { lvalue, .. } => {
                    let ident = &env.identifiers[lvalue.identifier.index()];
                    let decl_id = ident.declaration_id;
                    let Some(existing) = declarations.get(&decl_id) else {
                        return Err(invariant_error_with_span(
                            "Expected variable to have been defined",
                            Some(format!("No declaration for {}", format_place(lvalue, env),)),
                            lvalue.span,
                        ));
                    };
                    match existing {
                        DeclarationLoc::Instruction { block_index: bi, instr_local_index: ili } => {
                            let_spans.push((*bi, *ili));
                        }
                        DeclarationLoc::ParamOrContext => {
                            // Already Let
                        }
                    }
                }
                _ => {}
            }
        }
    }

    // Phase 2: Apply all collected mutations.

    // Helper: given (block_index, instr_local_index), get the InstructionId
    // and mutate the instruction's lvalue kind.
    for (bi, ili) in const_spans {
        let block_id = &block_keys[bi];
        let instr_id = func.body.blocks[block_id].instructions[ili];
        let instr = &mut func.instructions[instr_id.index()];
        if let InstructionValue::StoreLocal { lvalue, .. } = &mut instr.value {
            lvalue.kind = InstructionKind::Const;
        }
    }

    for (bi, ili) in reassign_spans {
        let block_id = &block_keys[bi];
        let instr_id = func.body.blocks[block_id].instructions[ili];
        let instr = &mut func.instructions[instr_id.index()];
        if let InstructionValue::StoreLocal { lvalue, .. } = &mut instr.value {
            lvalue.kind = InstructionKind::Reassign;
        }
    }

    // Apply destructure_kind_spans BEFORE let_spans: a Destructure that first
    // declares a variable gets kind=Const here, but if a later instruction
    // reassigns that variable the Destructure must become Let.  Applying
    // let_spans afterwards allows it to override the Const set here, matching
    // the TS behaviour where `declaration.kind = Let` mutates the original
    // lvalue reference after the Destructure's own `lvalue.kind = kind`.
    for (bi, ili, kind) in destructure_kind_spans {
        let block_id = &block_keys[bi];
        let instr_id = func.body.blocks[block_id].instructions[ili];
        let instr = &mut func.instructions[instr_id.index()];
        if let InstructionValue::Destructure { lvalue, .. } = &mut instr.value {
            lvalue.kind = kind;
        }
    }

    for (bi, ili) in let_spans {
        let block_id = &block_keys[bi];
        let instr_id = func.body.blocks[block_id].instructions[ili];
        let instr = &mut func.instructions[instr_id.index()];
        match &mut instr.value {
            InstructionValue::DeclareLocal { lvalue, .. }
            | InstructionValue::StoreLocal { lvalue, .. } => {
                lvalue.kind = InstructionKind::Let;
            }
            InstructionValue::Destructure { lvalue, .. } => {
                lvalue.kind = InstructionKind::Let;
            }
            _ => {}
        }
    }

    Ok(())
}
