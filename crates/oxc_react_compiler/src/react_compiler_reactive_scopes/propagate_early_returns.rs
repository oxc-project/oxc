// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

//! PropagateEarlyReturns — ensures reactive blocks honor early return semantics.
//!
//! When a scope contains an early return, creates a sentinel-based check so that
//! cached scopes can properly replay the early return behavior.
//!
//! Corresponds to `src/ReactiveScopes/PropagateEarlyReturns.ts`.

use std::mem::replace;

use oxc_allocator::{Box as ArenaBox, Vec as ArenaVec};
use oxc_diagnostics::OxcDiagnostic;
use oxc_str::{Ident, format_ident};

use crate::react_compiler_hir::{
    BlockId, Effect, EvaluationOrder, IdentifierId, IdentifierName, InstructionKind,
    InstructionValue, LValue, NonLocalBinding, Place, PlaceOrSpread, PrimitiveValue,
    PropertyLiteral, ReactiveFunction, ReactiveInstruction, ReactiveLabel, ReactiveScopeBlock,
    ReactiveScopeDeclaration, ReactiveScopeEarlyReturn, ReactiveStatement, ReactiveTerminal,
    ReactiveTerminalStatement, ReactiveTerminalTargetKind, ReactiveValue, environment::Environment,
};
use crate::react_compiler_reactive_scopes::visitors::{
    ReactiveFunctionTransform, Transformed, transform_reactive_function,
};
use oxc_span::Span;

/// The sentinel string used to detect early returns.
/// TS: `EARLY_RETURN_SENTINEL` from CodegenReactiveFunction.
const EARLY_RETURN_SENTINEL: &str = "react.early_return_sentinel";

// =============================================================================
// Public entry point
// =============================================================================

/// Propagate early return semantics through reactive scopes.
/// TS: `propagateEarlyReturns`
pub fn propagate_early_returns<'a>(func: &mut ReactiveFunction<'a>, env: &mut Environment<'a>) {
    let mut transform = Transform { env };
    let mut state = State { within_reactive_scope: false, early_return_value: None };
    // The TS version doesn't produce errors from this pass, so we ignore the Result.
    let _ = transform_reactive_function(func, &mut transform, &mut state);
}

// =============================================================================
// State
// =============================================================================

#[derive(Debug, Clone)]
struct EarlyReturnInfo {
    value: IdentifierId,
    span: Option<Span>,
    label: BlockId,
}

struct State {
    within_reactive_scope: bool,
    early_return_value: Option<EarlyReturnInfo>,
}

// =============================================================================
// Transform implementation (ReactiveFunctionTransform)
// =============================================================================

/// TS: `class Transform extends ReactiveFunctionTransform<State>`
struct Transform<'a, 'e> {
    env: &'e mut Environment<'a>,
}

impl<'a, 'e> ReactiveFunctionTransform<'a> for Transform<'a, 'e> {
    type State = State;

    fn env(&self) -> &Environment<'a> {
        self.env
    }

    /// TS: `override visitScope`
    fn visit_scope(
        &mut self,
        scope_block: &mut ReactiveScopeBlock<'a>,
        parent_state: &mut State,
    ) -> Result<(), OxcDiagnostic> {
        let scope_id = scope_block.scope;

        // Exit early if an earlier pass has already created an early return
        if self.env.scopes[scope_id].early_return_value.is_some() {
            return Ok(());
        }

        let mut inner_state = State {
            within_reactive_scope: true,
            early_return_value: parent_state.early_return_value.clone(),
        };
        self.traverse_scope(scope_block, &mut inner_state)?;

        if let Some(early_return_value) = inner_state.early_return_value {
            if !parent_state.within_reactive_scope {
                // This is the outermost scope wrapping an early return
                apply_early_return_to_scope(scope_block, self.env, &early_return_value);
            } else {
                // Not outermost — bubble up
                parent_state.early_return_value = Some(early_return_value);
            }
        }

        Ok(())
    }

    /// TS: `override transformTerminal`
    fn transform_terminal(
        &mut self,
        stmt: &mut ReactiveTerminalStatement<'a>,
        state: &mut State,
    ) -> Result<Transformed<ReactiveStatement<'a>>, OxcDiagnostic> {
        if state.within_reactive_scope
            && let ReactiveTerminal::Return { value, .. } = &stmt.terminal
        {
            let span = value.span;

            let early_return_value = if let Some(ref existing) = state.early_return_value {
                existing.clone()
            } else {
                // Create a new early return identifier
                let identifier_id = create_temporary_place_id(self.env, span);
                promote_temporary(self.env, identifier_id);
                let label = self.env.next_block_id();
                EarlyReturnInfo { value: identifier_id, span, label }
            };

            state.early_return_value = Some(early_return_value.clone());

            let return_value = *value;
            let alloc = self.env.allocator;

            return Ok(Transformed::ReplaceMany(vec![
                // StoreLocal: reassign the early return value
                ReactiveStatement::Instruction(ReactiveInstruction {
                    id: EvaluationOrder::UNSET,
                    lvalue: None,
                    value: ReactiveValue::Instruction(InstructionValue::StoreLocal {
                        lvalue: LValue {
                            kind: InstructionKind::Reassign,
                            place: Place {
                                identifier: early_return_value.value,
                                effect: Effect::Capture,
                                reactive: true,
                                span,
                            },
                        },
                        value: return_value,
                        span,
                    }),
                    span,
                }),
                // Break to the label
                ReactiveStatement::Terminal(ArenaBox::new_in(
                    ReactiveTerminalStatement {
                        terminal: ReactiveTerminal::Break {
                            target: early_return_value.label,
                            id: EvaluationOrder::UNSET,
                            target_kind: ReactiveTerminalTargetKind::Labeled,
                        },
                        label: None,
                    },
                    &alloc,
                )),
            ]));
        }

        // Default: traverse into the terminal's sub-blocks
        self.visit_terminal(stmt, state)?;
        Ok(Transformed::Keep)
    }
}

// =============================================================================
// Apply early return transformation to the outermost scope
// =============================================================================

fn apply_early_return_to_scope<'a>(
    scope_block: &mut ReactiveScopeBlock<'a>,
    env: &mut Environment<'a>,
    early_return: &EarlyReturnInfo,
) {
    let scope_id = scope_block.scope;
    let span = early_return.span;

    // Set early return value on the scope
    env.scopes[scope_id].early_return_value = Some(ReactiveScopeEarlyReturn {
        value: early_return.value,
        span: early_return.span,
        label: early_return.label,
    });

    // Add the early return identifier as a scope declaration
    env.scopes[scope_id].declarations.push((
        early_return.value,
        ReactiveScopeDeclaration { identifier: early_return.value, scope: scope_id },
    ));

    // Create temporary places for the sentinel initialization
    let sentinel_temp = create_temporary_place_id(env, span);
    let symbol_temp = create_temporary_place_id(env, span);
    let for_temp = create_temporary_place_id(env, span);
    let arg_temp = create_temporary_place_id(env, span);

    let alloc = env.allocator;
    let original_instructions = replace(&mut scope_block.instructions, ArenaVec::new_in(&alloc));

    scope_block.instructions = ArenaVec::from_iter_in(
        [
            // LoadGlobal Symbol
            ReactiveStatement::Instruction(ReactiveInstruction {
                id: EvaluationOrder::UNSET,
                lvalue: Some(Place {
                    identifier: symbol_temp,
                    effect: Effect::Unknown,
                    reactive: false,
                    span: None, // GeneratedSource
                }),
                value: ReactiveValue::Instruction(InstructionValue::LoadGlobal {
                    binding: NonLocalBinding::Global { name: Ident::from("Symbol") },
                    span,
                }),
                span,
            }),
            // PropertyLoad Symbol.for
            ReactiveStatement::Instruction(ReactiveInstruction {
                id: EvaluationOrder::UNSET,
                lvalue: Some(Place {
                    identifier: for_temp,
                    effect: Effect::Unknown,
                    reactive: false,
                    span: None, // GeneratedSource
                }),
                value: ReactiveValue::Instruction(InstructionValue::PropertyLoad {
                    object: Place {
                        identifier: symbol_temp,
                        effect: Effect::Unknown,
                        reactive: false,
                        span: None, // GeneratedSource
                    },
                    property: PropertyLiteral::String(Ident::from("for")),
                    span,
                }),
                span,
            }),
            // Primitive: the sentinel string
            ReactiveStatement::Instruction(ReactiveInstruction {
                id: EvaluationOrder::UNSET,
                lvalue: Some(Place {
                    identifier: arg_temp,
                    effect: Effect::Unknown,
                    reactive: false,
                    span: None, // GeneratedSource
                }),
                value: ReactiveValue::Instruction(InstructionValue::Primitive {
                    value: PrimitiveValue::String(EARLY_RETURN_SENTINEL.into()),
                    span,
                }),
                span,
            }),
            // MethodCall: Symbol.for("react.early_return_sentinel")
            ReactiveStatement::Instruction(ReactiveInstruction {
                id: EvaluationOrder::UNSET,
                lvalue: Some(Place {
                    identifier: sentinel_temp,
                    effect: Effect::Unknown,
                    reactive: false,
                    span: None, // GeneratedSource
                }),
                value: ReactiveValue::Instruction(InstructionValue::MethodCall {
                    receiver: Place {
                        identifier: symbol_temp,
                        effect: Effect::Unknown,
                        reactive: false,
                        span: None, // GeneratedSource
                    },
                    property: Place {
                        identifier: for_temp,
                        effect: Effect::Unknown,
                        reactive: false,
                        span: None, // GeneratedSource
                    },
                    args: ArenaVec::from_array_in(
                        [PlaceOrSpread::Place(Place {
                            identifier: arg_temp,
                            effect: Effect::Unknown,
                            reactive: false,
                            span: None, // GeneratedSource
                        })],
                        &alloc,
                    ),
                    span,
                }),
                span,
            }),
            // StoreLocal: let earlyReturnValue = sentinel
            ReactiveStatement::Instruction(ReactiveInstruction {
                id: EvaluationOrder::UNSET,
                lvalue: None,
                value: ReactiveValue::Instruction(InstructionValue::StoreLocal {
                    lvalue: LValue {
                        kind: InstructionKind::Let,
                        place: Place {
                            identifier: early_return.value,
                            effect: Effect::ConditionallyMutate,
                            reactive: true,
                            span,
                        },
                    },
                    value: Place {
                        identifier: sentinel_temp,
                        effect: Effect::Unknown,
                        reactive: false,
                        span: None, // GeneratedSource
                    },
                    span,
                }),
                span,
            }),
            // Label terminal wrapping the original instructions
            ReactiveStatement::Terminal(ArenaBox::new_in(
                ReactiveTerminalStatement {
                    label: Some(ReactiveLabel { id: early_return.label, implicit: false }),
                    terminal: ReactiveTerminal::Label {
                        block: original_instructions,
                        id: EvaluationOrder::UNSET,
                    },
                },
                &alloc,
            )),
        ],
        &alloc,
    );
}

// =============================================================================
// Helper: create a temporary place identifier
// =============================================================================

fn create_temporary_place_id(env: &mut Environment, span: Option<Span>) -> IdentifierId {
    let id = env.next_identifier_id();
    env.identifiers[id].span = span;
    id
}

fn promote_temporary<'a>(env: &mut Environment<'a>, identifier_id: IdentifierId) {
    let decl_id = env.identifiers[identifier_id].declaration_id;
    env.identifiers[identifier_id].name =
        Some(IdentifierName::Promoted(format_ident!(env.allocator, "#t{}", decl_id.index())));
}
