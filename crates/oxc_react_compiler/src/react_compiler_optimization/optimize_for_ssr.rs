// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

//! Optimizes the code for running in an SSR environment.
//!
//! Assumes that setState will not be called during render during initial mount,
//! which allows inlining useState/useReducer.
//!
//! Optimizations:
//! - Inline useState/useReducer
//! - Remove effects (useEffect, useLayoutEffect, useInsertionEffect)
//! - Remove event handlers (functions that call setState or startTransition)
//! - Remove known event handler props and ref props from builtin JSX tags
//! - Inline useEffectEvent to its argument
//!
//! Ported from TypeScript `src/Optimization/OptimizeForSSR.ts`.

use oxc_allocator::Vec as ArenaVec;
use rustc_hash::FxHashMap;

use crate::react_compiler_hir::environment::Environment;
use crate::react_compiler_hir::object_shape::HookKind;
use crate::react_compiler_hir::visitors::{each_instruction_value_operand, each_terminal_operand};
use crate::react_compiler_hir::{
    ArrayPatternElement, HirFunction, IdentifierId, InstructionValue, JsxAttribute, JsxTag, LValue,
    Pattern, Place, PlaceOrSpread, PrimitiveValue, is_array_type, is_plain_object_type,
    is_primitive_type, is_set_state_type, is_start_transition_type,
};
use oxc_span::Span;

/// Optimizes a function for SSR by inlining state hooks, removing effects,
/// removing event handlers, and stripping known event handler / ref JSX props.
///
/// Corresponds to TS `optimizeForSSR(fn: HIRFunction): void`.
pub fn optimize_for_ssr<'a>(func: &mut HirFunction<'a>, env: &Environment<'a>) {
    // Phase 1: Identify useState/useReducer calls that can be safely inlined.
    //
    // For useState(initialValue) where initialValue is primitive/object/array,
    // store a LoadLocal of the initial value.
    //
    // For useReducer(reducer, initialArg) store a LoadLocal of initialArg.
    // For useReducer(reducer, initialArg, init) store a CallExpression of init(initialArg).
    //
    // Any use of the hook return other than the expected destructuring pattern
    // prevents inlining (we delete from inlined_state if we see the identifier used
    // as an operand elsewhere).
    let mut inlined_state: FxHashMap<IdentifierId, InlinedStateReplacement> = FxHashMap::default();

    for (_block_id, block) in &func.body.blocks {
        for &instr_id in &block.instructions {
            let instr = &func.instructions[instr_id.index()];
            match &instr.value {
                InstructionValue::Destructure { value, lvalue, .. } => {
                    if inlined_state.contains_key(&env.identifiers[value.identifier].id)
                        && let Pattern::Array(arr) = &lvalue.pattern
                        && !arr.items.is_empty()
                        && let ArrayPatternElement::Place(_) = &arr.items[0]
                    {
                        // Allow destructuring of inlined states
                        continue;
                    }
                }
                InstructionValue::MethodCall { property, args, .. }
                | InstructionValue::CallExpression { callee: property, args, .. } => {
                    // Determine callee based on instruction kind
                    let callee_id = property.identifier;
                    let hook_kind = get_hook_kind(env, callee_id);
                    match hook_kind {
                        Some(HookKind::UseReducer) => {
                            if args.len() == 2 {
                                if let (PlaceOrSpread::Place(_), PlaceOrSpread::Place(arg)) =
                                    (&args[0], &args[1])
                                {
                                    let lvalue_id = env.identifiers[instr.lvalue.identifier].id;
                                    inlined_state.insert(
                                        lvalue_id,
                                        InlinedStateReplacement::LoadLocal {
                                            place: *arg,
                                            span: arg.span,
                                        },
                                    );
                                }
                            } else if args.len() == 3
                                && let (
                                    PlaceOrSpread::Place(_),
                                    PlaceOrSpread::Place(arg),
                                    PlaceOrSpread::Place(initializer),
                                ) = (&args[0], &args[1], &args[2])
                            {
                                let lvalue_id = env.identifiers[instr.lvalue.identifier].id;
                                let call_span = instr.value.span().copied();
                                inlined_state.insert(
                                    lvalue_id,
                                    InlinedStateReplacement::CallExpression {
                                        callee: *initializer,
                                        arg: *arg,
                                        span: call_span,
                                    },
                                );
                            }
                        }
                        Some(HookKind::UseState) if args.len() == 1 => {
                            if let PlaceOrSpread::Place(arg) = &args[0] {
                                let arg_type = &env.types[env.identifiers[arg.identifier].type_];
                                if is_primitive_type(arg_type)
                                    || is_plain_object_type(arg_type)
                                    || is_array_type(arg_type)
                                {
                                    let lvalue_id = env.identifiers[instr.lvalue.identifier].id;
                                    inlined_state.insert(
                                        lvalue_id,
                                        InlinedStateReplacement::LoadLocal {
                                            place: *arg,
                                            span: arg.span,
                                        },
                                    );
                                }
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }

            // Any use of useState/useReducer return besides destructuring prevents inlining
            if !inlined_state.is_empty() {
                let operands = each_instruction_value_operand(&instr.value, env);
                for operand in &operands {
                    let id = env.identifiers[operand.identifier].id;
                    inlined_state.remove(&id);
                }
            }
        }
        if !inlined_state.is_empty() {
            let operands = each_terminal_operand(&block.terminal);
            for operand in &operands {
                let id = env.identifiers[operand.identifier].id;
                inlined_state.remove(&id);
            }
        }
    }

    // Phase 2: Apply transformations
    //
    // - Replace FunctionExpression with Primitive(undefined) if it calls setState/startTransition
    // - Remove known event handler props and ref props from builtin JSX tags
    // - Replace Destructure of inlined state with StoreLocal
    // - Replace useEffectEvent(fn) with LoadLocal(fn)
    // - Replace useEffect/useLayoutEffect/useInsertionEffect with Primitive(undefined)
    // - Replace useState/useReducer with their inlined replacement
    for (_block_id, block) in &mut func.body.blocks {
        for &instr_id in &block.instructions {
            let instr = &mut func.instructions[instr_id.index()];
            match &instr.value {
                InstructionValue::FunctionExpression { lowered_func, span, .. } => {
                    let inner_func = &env.functions[lowered_func.func];
                    if has_known_non_render_call(inner_func, env) {
                        let span = *span;
                        instr.value =
                            InstructionValue::Primitive { value: PrimitiveValue::Undefined, span };
                    }
                }
                InstructionValue::JsxExpression { tag: JsxTag::Builtin(builtin), .. } => {
                    // Only optimize non-custom-element builtin tags
                    if !builtin.name.contains('-') {
                        // Retain only props that are not known event handlers and not "ref"
                        if let InstructionValue::JsxExpression { props, .. } = &mut instr.value {
                            props.retain(|prop| match prop {
                                JsxAttribute::SpreadAttribute { .. } => true,
                                JsxAttribute::Attribute { name, .. } => {
                                    !is_known_event_handler(name) && name != "ref"
                                }
                            });
                        }
                    }
                }
                InstructionValue::Destructure { value, lvalue, span } => {
                    let value_id = env.identifiers[value.identifier].id;
                    if inlined_state.contains_key(&value_id) {
                        // Invariant: destructuring pattern must be ArrayPattern with at least one Identifier item
                        if let Pattern::Array(arr) = &lvalue.pattern
                            && !arr.items.is_empty()
                            && let ArrayPatternElement::Place(first_place) = &arr.items[0]
                        {
                            let span = *span;
                            let kind = lvalue.kind;
                            let store = InstructionValue::StoreLocal {
                                lvalue: LValue { place: *first_place, kind },
                                value: *value,
                                span,
                            };
                            instr.value = store;
                        }
                    }
                }
                InstructionValue::MethodCall { property, args, span, .. }
                | InstructionValue::CallExpression { callee: property, args, span, .. } => {
                    let callee_id = property.identifier;
                    let hook_kind = get_hook_kind(env, callee_id);
                    match hook_kind {
                        Some(HookKind::UseEffectEvent) => {
                            if args.len() == 1
                                && let PlaceOrSpread::Place(arg) = &args[0]
                            {
                                let span = *span;
                                instr.value = InstructionValue::LoadLocal { place: *arg, span };
                            }
                        }
                        Some(
                            HookKind::UseEffect
                            | HookKind::UseLayoutEffect
                            | HookKind::UseInsertionEffect,
                        ) => {
                            let span = *span;
                            instr.value = InstructionValue::Primitive {
                                value: PrimitiveValue::Undefined,
                                span,
                            };
                        }
                        Some(HookKind::UseReducer | HookKind::UseState) => {
                            let lvalue_id = env.identifiers[instr.lvalue.identifier].id;
                            if let Some(replacement) = inlined_state.get(&lvalue_id) {
                                instr.value = match replacement {
                                    InlinedStateReplacement::LoadLocal { place, span } => {
                                        InstructionValue::LoadLocal { place: *place, span: *span }
                                    }
                                    InlinedStateReplacement::CallExpression {
                                        callee,
                                        arg,
                                        span,
                                    } => InstructionValue::CallExpression {
                                        callee: *callee,
                                        args: ArenaVec::from_array_in(
                                            [PlaceOrSpread::Place(*arg)],
                                            &env.allocator,
                                        ),
                                        span: *span,
                                    },
                                };
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }
}

/// Replacement values for inlined useState/useReducer calls.
#[derive(Debug, Clone)]
enum InlinedStateReplacement {
    /// Replace with `LoadLocal { place }` — used for useState and useReducer(reducer, initialArg)
    LoadLocal { place: Place, span: Option<Span> },
    /// Replace with `CallExpression { callee, args: [arg] }` — used for useReducer(reducer, initialArg, init)
    CallExpression { callee: Place, arg: Place, span: Option<Span> },
}

/// Returns true if the function body contains a call to setState or startTransition.
/// This identifies functions that are event handlers and can be replaced with undefined
/// during SSR.
///
/// Corresponds to TS `hasKnownNonRenderCall(fn: HIRFunction): boolean`.
fn has_known_non_render_call(func: &HirFunction, env: &Environment) -> bool {
    for (_block_id, block) in &func.body.blocks {
        for &instr_id in &block.instructions {
            let instr = &func.instructions[instr_id.index()];
            if let InstructionValue::CallExpression { callee, .. } = &instr.value {
                let callee_type = &env.types[env.identifiers[callee.identifier].type_];
                if is_set_state_type(callee_type) || is_start_transition_type(callee_type) {
                    return true;
                }
            }
        }
    }
    false
}

/// Returns true if the prop name matches the known event handler pattern `on[A-Z]`.
fn is_known_event_handler(prop: &str) -> bool {
    if prop.len() < 3 {
        return false;
    }
    if !prop.starts_with("on") {
        return false;
    }
    let third_char = prop.as_bytes()[2];
    third_char.is_ascii_uppercase()
}

/// Get the hook kind for an identifier, if its type represents a hook.
fn get_hook_kind(env: &Environment, identifier_id: IdentifierId) -> Option<HookKind> {
    env.get_hook_kind_for_id(identifier_id).ok().flatten().cloned()
}
