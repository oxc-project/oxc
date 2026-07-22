// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

//! Constant propagation/folding pass.
//!
//! Applies Sparse Conditional Constant Propagation to the given function.
//! We use abstract interpretation to record known constant values for identifiers,
//! with lack of a value indicating that the identifier does not have a known
//! constant value.
//!
//! Instructions which can be compile-time evaluated *and* whose operands are known
//! constants are replaced with the resulting constant value.
//!
//! This pass also exploits SSA form, tracking constant values of local variables.
//! For example, in `let x = 4; let y = x + 1` we know that `x = 4` in the binary
//! expression and can replace it with `Constant 5`.
//!
//! This pass also visits conditionals (currently only IfTerminal) and can prune
//! unreachable branches when the condition is a known truthy/falsey constant.
//! The pass uses fixpoint iteration, looping until no additional updates can be
//! performed.
//!
//! Analogous to TS `Optimization/ConstantPropagation.ts`.

use std::mem::replace;

use rustc_hash::FxHashMap;

use oxc_allocator::Allocator;
use oxc_ecmascript::{StringToNumber, ToInt32, ToUint32};
use oxc_str::{Ident, Str};
use oxc_syntax::identifier::is_identifier_name;
use oxc_syntax::keyword::is_reserved_keyword;
use oxc_syntax::number::ToJsString;

use crate::react_compiler_hir::environment::Environment;
use crate::react_compiler_hir::{
    BlockKind, FloatValue, FunctionId, GotoVariant, HirFunction, IdentifierId, InstructionId,
    InstructionValue, ManualMemoDependencyRoot, NonLocalBinding, Phi, Place, PrimitiveValue,
    PropertyLiteral, Terminal, UnaryOperator,
};
use crate::react_compiler_lowering::{
    get_reverse_postordered_blocks, mark_instruction_ids, mark_predecessors,
    remove_dead_do_while_statements, remove_unnecessary_try_catch, remove_unreachable_for_updates,
};
use crate::react_compiler_ssa::eliminate_redundant_phi;
use crate::react_compiler_ssa::enter_ssa::placeholder_function;
use oxc_ast::ast::{BinaryOperator, UpdateOperator};
use oxc_span::Span;

use crate::react_compiler_optimization::merge_consecutive_blocks::merge_consecutive_blocks;

// =============================================================================
// Constant type — mirrors TS `type Constant = Primitive | LoadGlobal`
// The span is preserved so that when we replace an instruction value with the
// constant, we use the span from the original definition site (matching TS).
// =============================================================================

#[derive(Debug, Clone)]
enum Constant<'a> {
    Primitive { value: PrimitiveValue<'a>, span: Option<Span> },
    LoadGlobal { binding: NonLocalBinding<'a>, span: Option<Span> },
}

impl<'a> Constant<'a> {
    fn into_instruction_value(self) -> InstructionValue<'a> {
        match self {
            Constant::Primitive { value, span } => InstructionValue::Primitive { value, span },
            Constant::LoadGlobal { binding, span } => {
                InstructionValue::LoadGlobal { binding, span }
            }
        }
    }
}

/// Map of known constant values. Uses FxHashMap (not FxIndexMap) since iteration
/// order does not affect correctness — this map is only used for lookups.
type Constants<'a> = FxHashMap<IdentifierId, Constant<'a>>;

// =============================================================================
// Public entry point
// =============================================================================

pub fn constant_propagation<'a>(func: &mut HirFunction<'a>, env: &mut Environment<'a>) {
    let mut constants: Constants<'a> = FxHashMap::default();
    constant_propagation_impl(func, env, &mut constants);
}

fn constant_propagation_impl<'a>(
    func: &mut HirFunction<'a>,
    env: &mut Environment<'a>,
    constants: &mut Constants<'a>,
) {
    loop {
        let have_terminals_changed = apply_constant_propagation(func, env, constants);
        if !have_terminals_changed {
            break;
        }
        /*
         * If terminals have changed then blocks may have become newly unreachable.
         * Re-run minification of the graph (incl reordering instruction ids)
         */
        func.body.blocks = get_reverse_postordered_blocks(&func.body, env.allocator);
        remove_unreachable_for_updates(&mut func.body);
        remove_dead_do_while_statements(&mut func.body);
        remove_unnecessary_try_catch(&mut func.body);
        mark_instruction_ids(&mut func.body, &mut func.instructions);
        mark_predecessors(&mut func.body);

        // Now that predecessors are updated, prune phi operands that can never be reached
        for (_block_id, block) in func.body.blocks.iter_mut() {
            for phi in &mut block.phis {
                phi.operands.retain(|pred, _operand| block.preds.contains(pred));
            }
        }

        /*
         * By removing some phi operands, there may be phis that were not previously
         * redundant but now are
         */
        eliminate_redundant_phi(func, env);

        /*
         * Finally, merge together any blocks that are now guaranteed to execute
         * consecutively
         */
        merge_consecutive_blocks(func, &mut env.functions, env.allocator);

        // TODO: port assertConsistentIdentifiers(fn) and assertTerminalSuccessorsExist(fn)
        // from TS HIR validation. These are debug assertions that verify structural
        // invariants after the CFG cleanup helpers run.
    }
}

fn apply_constant_propagation<'a>(
    func: &mut HirFunction<'a>,
    env: &mut Environment<'a>,
    constants: &mut Constants<'a>,
) -> bool {
    let mut has_changes = false;

    let block_ids: Vec<_> = func.body.blocks.keys().copied().collect();
    for block_id in block_ids {
        let block = &func.body.blocks[&block_id];

        // Initialize phi values if all operands have the same known constant value
        let phi_updates: Vec<(IdentifierId, Constant)> = block
            .phis
            .iter()
            .filter_map(|phi| {
                let value = evaluate_phi(phi, constants)?;
                Some((phi.place.identifier, value))
            })
            .collect();
        for (id, value) in phi_updates {
            constants.insert(id, value);
        }

        let block = &func.body.blocks[&block_id];
        let instr_ids = block.instructions.iter().copied().collect::<Vec<_>>();
        let block_kind = block.kind;
        let instr_count = instr_ids.len();

        for (i, instr_id) in instr_ids.iter().enumerate() {
            if block_kind == BlockKind::Sequence && i == instr_count - 1 {
                /*
                 * evaluating the last value of a value block can break order of evaluation,
                 * skip these instructions
                 */
                continue;
            }
            let result = evaluate_instruction(constants, func, env, *instr_id);
            if let Some(value) = result {
                let lvalue_id = func.instructions[instr_id.index()].lvalue.identifier;
                constants.insert(lvalue_id, value);
            }
        }

        let block = &func.body.blocks[&block_id];
        match &block.terminal {
            Terminal::If { test, consequent, alternate, id, span, .. } => {
                let test_value = read(constants, test);
                if let Some(Constant::Primitive { value: ref prim, .. }) = test_value {
                    has_changes = true;
                    let target_block_id = if is_truthy(prim) { *consequent } else { *alternate };
                    let terminal = Terminal::Goto {
                        variant: GotoVariant::Break,
                        block: target_block_id,
                        id: *id,
                        span: *span,
                    };
                    func.body.blocks.get_mut(&block_id).unwrap().terminal = terminal;
                }
            }
            Terminal::Unreachable { .. }
            | Terminal::Throw { .. }
            | Terminal::Return { .. }
            | Terminal::Goto { .. }
            | Terminal::Branch { .. }
            | Terminal::Switch { .. }
            | Terminal::DoWhile { .. }
            | Terminal::While { .. }
            | Terminal::For { .. }
            | Terminal::ForOf { .. }
            | Terminal::ForIn { .. }
            | Terminal::Logical { .. }
            | Terminal::Ternary { .. }
            | Terminal::Optional { .. }
            | Terminal::Label { .. }
            | Terminal::Sequence { .. }
            | Terminal::MaybeThrow { .. }
            | Terminal::Try { .. }
            | Terminal::Scope { .. }
            | Terminal::PrunedScope { .. } => {
                // no-op
            }
        }
    }

    has_changes
}

// =============================================================================
// Phi evaluation
// =============================================================================

fn evaluate_phi<'a>(phi: &Phi, constants: &Constants<'a>) -> Option<Constant<'a>> {
    let mut value: Option<Constant> = None;
    for (_pred, operand) in &phi.operands {
        let operand_value = constants.get(&operand.identifier)?;

        match &value {
            None => {
                // first iteration of the loop
                value = Some(operand_value.clone());
                continue;
            }
            Some(current) => match (current, operand_value) {
                (Constant::Primitive { value: a, .. }, Constant::Primitive { value: b, .. }) => {
                    // Use JS strict equality semantics: NaN !== NaN
                    if !js_strict_equal(a, b) {
                        return None;
                    }
                }
                (
                    Constant::LoadGlobal { binding: a, .. },
                    Constant::LoadGlobal { binding: b, .. },
                ) => {
                    // different global values, can't constant propagate
                    if a.name() != b.name() {
                        return None;
                    }
                }
                // found different kinds of constants, can't constant propagate
                (Constant::Primitive { .. }, Constant::LoadGlobal { .. })
                | (Constant::LoadGlobal { .. }, Constant::Primitive { .. }) => {
                    return None;
                }
            },
        }
    }
    value
}

// =============================================================================
// Instruction evaluation
// =============================================================================

fn evaluate_instruction<'a>(
    constants: &mut Constants<'a>,
    func: &mut HirFunction<'a>,
    env: &mut Environment<'a>,
    instr_id: InstructionId,
) -> Option<Constant<'a>> {
    let instr = &func.instructions[instr_id.index()];
    match &instr.value {
        InstructionValue::Primitive { value, span } => {
            Some(Constant::Primitive { value: *value, span: *span })
        }
        InstructionValue::LoadGlobal { binding, span } => {
            Some(Constant::LoadGlobal { binding: *binding, span: *span })
        }
        InstructionValue::ComputedLoad { object, property, span } => {
            let prop_value = read(constants, property);
            if let Some(Constant::Primitive { value: ref prim, .. }) = prop_value {
                match prim {
                    PrimitiveValue::String(s) if is_valid_identifier(s.as_str()) => {
                        let object = *object;
                        let span = *span;
                        let new_property = PropertyLiteral::String(Ident::from(s.as_str()));
                        func.instructions[instr_id.index()].value =
                            InstructionValue::PropertyLoad { object, property: new_property, span };
                    }
                    PrimitiveValue::Number(n) => {
                        let object = *object;
                        let span = *span;
                        let new_property = PropertyLiteral::Number(*n);
                        func.instructions[instr_id.index()].value =
                            InstructionValue::PropertyLoad { object, property: new_property, span };
                    }
                    PrimitiveValue::Null
                    | PrimitiveValue::Undefined
                    | PrimitiveValue::Boolean(_)
                    | PrimitiveValue::String(_) => {}
                }
            }
            None
        }
        InstructionValue::ComputedStore { object, property, value, span } => {
            let prop_value = read(constants, property);
            if let Some(Constant::Primitive { value: ref prim, .. }) = prop_value {
                match prim {
                    PrimitiveValue::String(s) if is_valid_identifier(s.as_str()) => {
                        let object = *object;
                        let store_value = *value;
                        let span = *span;
                        let new_property = PropertyLiteral::String(Ident::from(s.as_str()));
                        func.instructions[instr_id.index()].value =
                            InstructionValue::PropertyStore {
                                object,
                                property: new_property,
                                value: store_value,
                                span,
                            };
                    }
                    PrimitiveValue::Number(n) => {
                        let object = *object;
                        let store_value = *value;
                        let span = *span;
                        let new_property = PropertyLiteral::Number(*n);
                        func.instructions[instr_id.index()].value =
                            InstructionValue::PropertyStore {
                                object,
                                property: new_property,
                                value: store_value,
                                span,
                            };
                    }
                    PrimitiveValue::Null
                    | PrimitiveValue::Undefined
                    | PrimitiveValue::Boolean(_)
                    | PrimitiveValue::String(_) => {}
                }
            }
            None
        }
        InstructionValue::PostfixUpdate { lvalue, operation, value, span } => {
            let previous = read(constants, value);
            if let Some(Constant::Primitive { value: PrimitiveValue::Number(n), span: prev_span }) =
                previous
            {
                let prev_val = n.value();
                let next_val = match operation {
                    UpdateOperator::Increment => prev_val + 1.0,
                    UpdateOperator::Decrement => prev_val - 1.0,
                };
                // Store the updated value for the lvalue
                let lvalue_id = lvalue.identifier;
                constants.insert(
                    lvalue_id,
                    Constant::Primitive {
                        value: PrimitiveValue::Number(FloatValue::new(next_val)),
                        span: *span,
                    },
                );
                // But return the value prior to the update (preserving its original span)
                return Some(Constant::Primitive {
                    value: PrimitiveValue::Number(n),
                    span: prev_span,
                });
            }
            None
        }
        InstructionValue::PrefixUpdate { lvalue, operation, value, span } => {
            let previous = read(constants, value);
            if let Some(Constant::Primitive { value: PrimitiveValue::Number(n), .. }) = previous {
                let prev_val = n.value();
                let next_val = match operation {
                    UpdateOperator::Increment => prev_val + 1.0,
                    UpdateOperator::Decrement => prev_val - 1.0,
                };
                let result = Constant::Primitive {
                    value: PrimitiveValue::Number(FloatValue::new(next_val)),
                    span: *span,
                };
                // Store and return the updated value
                let lvalue_id = lvalue.identifier;
                constants.insert(lvalue_id, result.clone());
                return Some(result);
            }
            None
        }
        InstructionValue::UnaryExpression { operator, value, span } => match operator {
            UnaryOperator::Not => {
                let operand = read(constants, value);
                if let Some(Constant::Primitive { value: ref prim, .. }) = operand {
                    let negated = !is_truthy(prim);
                    let span = *span;
                    let result =
                        Constant::Primitive { value: PrimitiveValue::Boolean(negated), span };
                    func.instructions[instr_id.index()].value = InstructionValue::Primitive {
                        value: PrimitiveValue::Boolean(negated),
                        span,
                    };
                    return Some(result);
                }
                None
            }
            UnaryOperator::Minus => {
                let operand = read(constants, value);
                if let Some(Constant::Primitive { value: PrimitiveValue::Number(n), .. }) = operand
                {
                    let negated = -n.value();
                    let span = *span;
                    let result = Constant::Primitive {
                        value: PrimitiveValue::Number(FloatValue::new(negated)),
                        span,
                    };
                    func.instructions[instr_id.index()].value = InstructionValue::Primitive {
                        value: PrimitiveValue::Number(FloatValue::new(negated)),
                        span,
                    };
                    return Some(result);
                }
                None
            }
            UnaryOperator::Plus
            | UnaryOperator::BitwiseNot
            | UnaryOperator::TypeOf
            | UnaryOperator::Void => None,
        },
        InstructionValue::BinaryExpression { operator, left, right, span } => {
            let lhs_value = read(constants, left);
            let rhs_value = read(constants, right);
            if let (
                Some(Constant::Primitive { value: lhs, .. }),
                Some(Constant::Primitive { value: rhs, .. }),
            ) = (&lhs_value, &rhs_value)
            {
                let result = evaluate_binary_op(*operator, lhs, rhs, env.allocator);
                if let Some(prim) = result {
                    let span = *span;
                    func.instructions[instr_id.index()].value =
                        InstructionValue::Primitive { value: prim, span };
                    return Some(Constant::Primitive { value: prim, span });
                }
            }
            None
        }
        InstructionValue::PropertyLoad { object, property, span } => {
            let object_value = read(constants, object);
            if let Some(Constant::Primitive { value: PrimitiveValue::String(ref s), .. }) =
                object_value
                && let PropertyLiteral::String(prop_name) = property
                && prop_name == "length"
            {
                // Use UTF-16 code unit count to match JS .length semantics
                let len = s.as_str().encode_utf16().count() as f64;
                let span = *span;
                let result = Constant::Primitive {
                    value: PrimitiveValue::Number(FloatValue::new(len)),
                    span,
                };
                func.instructions[instr_id.index()].value = InstructionValue::Primitive {
                    value: PrimitiveValue::Number(FloatValue::new(len)),
                    span,
                };
                return Some(result);
            }
            None
        }
        InstructionValue::TemplateLiteral { subexprs, quasis, span } => {
            if subexprs.is_empty() {
                // No subexpressions: join all cooked quasis
                let mut result_string = String::new();
                for q in quasis {
                    result_string.push_str(q.cooked.as_ref()?);
                }
                let span = *span;
                let value =
                    PrimitiveValue::String(Str::from_str_in(&result_string, &env.allocator));
                let result = Constant::Primitive { value, span };
                func.instructions[instr_id.index()].value =
                    InstructionValue::Primitive { value, span };
                return Some(result);
            }

            if subexprs.len() != quasis.len() - 1 {
                return None;
            }

            if quasis.iter().any(|q| q.cooked.is_none()) {
                return None;
            }

            let mut quasi_index = 0usize;
            let mut result_string =
                quasis[quasi_index].cooked.as_ref().unwrap().as_str().to_string();
            quasi_index += 1;

            for sub_expr in subexprs {
                let sub_expr_value = read(constants, sub_expr);
                let sub_prim = match sub_expr_value {
                    Some(Constant::Primitive { ref value, .. }) => value,
                    _ => return None,
                };

                let expression_str = match sub_prim {
                    PrimitiveValue::Null => "null".to_string(),
                    PrimitiveValue::Boolean(b) => b.to_string(),
                    PrimitiveValue::Number(n) => n.value().to_js_string(),
                    PrimitiveValue::String(s) => s.as_str().to_string(),
                    // TS rejects undefined subexpression values
                    PrimitiveValue::Undefined => return None,
                };

                let suffix = quasis[quasi_index].cooked?;
                quasi_index += 1;

                result_string.push_str(&expression_str);
                result_string.push_str(&suffix);
            }

            let span = *span;
            let value = PrimitiveValue::String(Str::from_str_in(&result_string, &env.allocator));
            let result = Constant::Primitive { value, span };
            func.instructions[instr_id.index()].value = InstructionValue::Primitive { value, span };
            Some(result)
        }
        InstructionValue::LoadLocal { place, .. } => {
            let place_value = read(constants, place);
            if let Some(ref constant) = place_value {
                // Replace the LoadLocal with the constant value (including the constant's original span)
                func.instructions[instr_id.index()].value =
                    constant.clone().into_instruction_value();
            }
            place_value
        }
        InstructionValue::StoreLocal { lvalue, value, .. } => {
            let place_value = read(constants, value);
            if let Some(ref constant) = place_value {
                let lvalue_id = lvalue.place.identifier;
                constants.insert(lvalue_id, constant.clone());
            }
            place_value
        }
        InstructionValue::FunctionExpression { lowered_func, .. } => {
            let func_id = lowered_func.func;
            process_inner_function(func_id, env, constants);
            None
        }
        InstructionValue::ObjectMethod { lowered_func, .. } => {
            let func_id = lowered_func.func;
            process_inner_function(func_id, env, constants);
            None
        }
        InstructionValue::StartMemoize { deps, .. } => {
            if let Some(deps) = deps {
                // Two-phase: collect which deps are constant, then mutate
                let const_dep_indices: Vec<usize> = deps
                    .iter()
                    .enumerate()
                    .filter_map(|(i, dep)| {
                        if let ManualMemoDependencyRoot::NamedLocal { value, .. } = &dep.root {
                            let pv = read(constants, value);
                            if matches!(pv, Some(Constant::Primitive { .. })) {
                                return Some(i);
                            }
                        }
                        None
                    })
                    .collect();
                for idx in const_dep_indices {
                    if let InstructionValue::StartMemoize { deps: Some(ref mut deps), .. } =
                        func.instructions[instr_id.index()].value
                        && let ManualMemoDependencyRoot::NamedLocal { constant, .. } =
                            &mut deps[idx].root
                    {
                        *constant = true;
                    }
                }
            }
            None
        }
        // All other instruction kinds: no constant folding
        InstructionValue::LoadContext { .. }
        | InstructionValue::DeclareLocal { .. }
        | InstructionValue::DeclareContext { .. }
        | InstructionValue::StoreContext { .. }
        | InstructionValue::Destructure { .. }
        | InstructionValue::JSXText { .. }
        | InstructionValue::NewExpression { .. }
        | InstructionValue::CallExpression { .. }
        | InstructionValue::MethodCall { .. }
        | InstructionValue::TypeCastExpression { .. }
        | InstructionValue::JsxExpression { .. }
        | InstructionValue::ObjectExpression { .. }
        | InstructionValue::ArrayExpression { .. }
        | InstructionValue::JsxFragment { .. }
        | InstructionValue::RegExpLiteral { .. }
        | InstructionValue::MetaProperty { .. }
        | InstructionValue::PropertyStore { .. }
        | InstructionValue::PropertyDelete { .. }
        | InstructionValue::ComputedDelete { .. }
        | InstructionValue::StoreGlobal { .. }
        | InstructionValue::TaggedTemplateExpression { .. }
        | InstructionValue::Await { .. }
        | InstructionValue::GetIterator { .. }
        | InstructionValue::IteratorNext { .. }
        | InstructionValue::NextPropertyOf { .. }
        | InstructionValue::Debugger { .. }
        | InstructionValue::FinishMemoize { .. } => None,
    }
}

// =============================================================================
// Inner function processing
// =============================================================================

fn process_inner_function<'a>(
    func_id: FunctionId,
    env: &mut Environment<'a>,
    constants: &mut Constants<'a>,
) {
    let mut inner = replace(&mut env.functions[func_id], placeholder_function(env.allocator));
    constant_propagation_impl(&mut inner, env, constants);
    env.functions[func_id] = inner;
}

// =============================================================================
// Helper: read constant for a place
// =============================================================================

fn read<'a>(constants: &Constants<'a>, place: &Place) -> Option<Constant<'a>> {
    constants.get(&place.identifier).cloned()
}

// =============================================================================
// Helper: is_valid_identifier
// =============================================================================

/// Check if a string is a valid JavaScript identifier that is not a reserved
/// word (matching Babel's `isValidIdentifier` default behavior).
fn is_valid_identifier(s: &str) -> bool {
    is_identifier_name(s) && !is_reserved_keyword(s)
}

// =============================================================================
// Helper: is_truthy for PrimitiveValue
// =============================================================================

fn is_truthy(value: &PrimitiveValue) -> bool {
    match value {
        PrimitiveValue::Null => false,
        PrimitiveValue::Undefined => false,
        PrimitiveValue::Boolean(b) => *b,
        PrimitiveValue::Number(n) => {
            let v = n.value();
            v != 0.0 && !v.is_nan()
        }
        PrimitiveValue::String(s) => !s.as_str().is_empty(),
    }
}

// =============================================================================
// Binary operation evaluation
// =============================================================================

fn evaluate_binary_op<'a>(
    operator: BinaryOperator,
    lhs: &PrimitiveValue,
    rhs: &PrimitiveValue,
    allocator: &'a Allocator,
) -> Option<PrimitiveValue<'a>> {
    match operator {
        BinaryOperator::Addition => match (lhs, rhs) {
            (PrimitiveValue::Number(l), PrimitiveValue::Number(r)) => {
                Some(PrimitiveValue::Number(FloatValue::new(l.value() + r.value())))
            }
            (PrimitiveValue::String(l), PrimitiveValue::String(r)) => Some(PrimitiveValue::String(
                Str::from_strs_array_in([l.as_str(), r.as_str()], &allocator),
            )),
            _ => None,
        },
        BinaryOperator::Subtraction => match (lhs, rhs) {
            (PrimitiveValue::Number(l), PrimitiveValue::Number(r)) => {
                Some(PrimitiveValue::Number(FloatValue::new(l.value() - r.value())))
            }
            _ => None,
        },
        BinaryOperator::Multiplication => match (lhs, rhs) {
            (PrimitiveValue::Number(l), PrimitiveValue::Number(r)) => {
                Some(PrimitiveValue::Number(FloatValue::new(l.value() * r.value())))
            }
            _ => None,
        },
        BinaryOperator::Division => match (lhs, rhs) {
            (PrimitiveValue::Number(l), PrimitiveValue::Number(r)) => {
                Some(PrimitiveValue::Number(FloatValue::new(l.value() / r.value())))
            }
            _ => None,
        },
        BinaryOperator::Remainder => match (lhs, rhs) {
            (PrimitiveValue::Number(l), PrimitiveValue::Number(r)) => {
                Some(PrimitiveValue::Number(FloatValue::new(l.value() % r.value())))
            }
            _ => None,
        },
        BinaryOperator::Exponential => match (lhs, rhs) {
            (PrimitiveValue::Number(l), PrimitiveValue::Number(r)) => {
                Some(PrimitiveValue::Number(FloatValue::new(l.value().powf(r.value()))))
            }
            _ => None,
        },
        BinaryOperator::BitwiseOR => match (lhs, rhs) {
            (PrimitiveValue::Number(l), PrimitiveValue::Number(r)) => {
                let result = l.value().to_int_32() | r.value().to_int_32();
                Some(PrimitiveValue::Number(FloatValue::new(result as f64)))
            }
            _ => None,
        },
        BinaryOperator::BitwiseAnd => match (lhs, rhs) {
            (PrimitiveValue::Number(l), PrimitiveValue::Number(r)) => {
                let result = l.value().to_int_32() & r.value().to_int_32();
                Some(PrimitiveValue::Number(FloatValue::new(result as f64)))
            }
            _ => None,
        },
        BinaryOperator::BitwiseXOR => match (lhs, rhs) {
            (PrimitiveValue::Number(l), PrimitiveValue::Number(r)) => {
                let result = l.value().to_int_32() ^ r.value().to_int_32();
                Some(PrimitiveValue::Number(FloatValue::new(result as f64)))
            }
            _ => None,
        },
        BinaryOperator::ShiftLeft => match (lhs, rhs) {
            (PrimitiveValue::Number(l), PrimitiveValue::Number(r)) => {
                let result = l.value().to_int_32() << (r.value().to_uint_32() & 0x1f);
                Some(PrimitiveValue::Number(FloatValue::new(result as f64)))
            }
            _ => None,
        },
        BinaryOperator::ShiftRight => match (lhs, rhs) {
            (PrimitiveValue::Number(l), PrimitiveValue::Number(r)) => {
                let result = l.value().to_int_32() >> (r.value().to_uint_32() & 0x1f);
                Some(PrimitiveValue::Number(FloatValue::new(result as f64)))
            }
            _ => None,
        },
        BinaryOperator::ShiftRightZeroFill => match (lhs, rhs) {
            (PrimitiveValue::Number(l), PrimitiveValue::Number(r)) => {
                let result = l.value().to_uint_32() >> (r.value().to_uint_32() & 0x1f);
                Some(PrimitiveValue::Number(FloatValue::new(result as f64)))
            }
            _ => None,
        },
        BinaryOperator::LessThan => match (lhs, rhs) {
            (PrimitiveValue::Number(l), PrimitiveValue::Number(r)) => {
                Some(PrimitiveValue::Boolean(l.value() < r.value()))
            }
            _ => None,
        },
        BinaryOperator::LessEqualThan => match (lhs, rhs) {
            (PrimitiveValue::Number(l), PrimitiveValue::Number(r)) => {
                Some(PrimitiveValue::Boolean(l.value() <= r.value()))
            }
            _ => None,
        },
        BinaryOperator::GreaterThan => match (lhs, rhs) {
            (PrimitiveValue::Number(l), PrimitiveValue::Number(r)) => {
                Some(PrimitiveValue::Boolean(l.value() > r.value()))
            }
            _ => None,
        },
        BinaryOperator::GreaterEqualThan => match (lhs, rhs) {
            (PrimitiveValue::Number(l), PrimitiveValue::Number(r)) => {
                Some(PrimitiveValue::Boolean(l.value() >= r.value()))
            }
            _ => None,
        },
        BinaryOperator::StrictEquality => Some(PrimitiveValue::Boolean(js_strict_equal(lhs, rhs))),
        BinaryOperator::StrictInequality => {
            Some(PrimitiveValue::Boolean(!js_strict_equal(lhs, rhs)))
        }
        BinaryOperator::Equality => Some(PrimitiveValue::Boolean(js_abstract_equal(lhs, rhs))),
        BinaryOperator::Inequality => Some(PrimitiveValue::Boolean(!js_abstract_equal(lhs, rhs))),
        BinaryOperator::In | BinaryOperator::Instanceof => None,
    }
}

// =============================================================================
// JavaScript equality semantics
// =============================================================================

fn js_strict_equal(lhs: &PrimitiveValue, rhs: &PrimitiveValue) -> bool {
    match (lhs, rhs) {
        (PrimitiveValue::Null, PrimitiveValue::Null) => true,
        (PrimitiveValue::Undefined, PrimitiveValue::Undefined) => true,
        (PrimitiveValue::Boolean(a), PrimitiveValue::Boolean(b)) => a == b,
        (PrimitiveValue::Number(a), PrimitiveValue::Number(b)) => {
            let av = a.value();
            let bv = b.value();
            // NaN !== NaN in JS
            if av.is_nan() || bv.is_nan() {
                return false;
            }
            av == bv
        }
        (PrimitiveValue::String(a), PrimitiveValue::String(b)) => a == b,
        // Different types => false
        _ => false,
    }
}

fn js_abstract_equal(lhs: &PrimitiveValue, rhs: &PrimitiveValue) -> bool {
    match (lhs, rhs) {
        (PrimitiveValue::Null, PrimitiveValue::Null) => true,
        (PrimitiveValue::Undefined, PrimitiveValue::Undefined) => true,
        (PrimitiveValue::Null, PrimitiveValue::Undefined)
        | (PrimitiveValue::Undefined, PrimitiveValue::Null) => true,
        (PrimitiveValue::Boolean(a), PrimitiveValue::Boolean(b)) => a == b,
        (PrimitiveValue::Number(a), PrimitiveValue::Number(b)) => {
            let av = a.value();
            let bv = b.value();
            if av.is_nan() || bv.is_nan() {
                return false;
            }
            av == bv
        }
        (PrimitiveValue::String(a), PrimitiveValue::String(b)) => a == b,
        // Cross-type coercions for primitives
        (PrimitiveValue::Number(n), PrimitiveValue::String(s))
        | (PrimitiveValue::String(s), PrimitiveValue::Number(n)) => {
            // String is coerced to number using JS ToNumber semantics.
            let sv = s.as_str().string_to_number();
            let nv = n.value();
            if nv.is_nan() || sv.is_nan() { false } else { nv == sv }
        }
        (PrimitiveValue::Boolean(b), other) => {
            let num = if *b { 1.0 } else { 0.0 };
            js_abstract_equal(&PrimitiveValue::Number(FloatValue::new(num)), other)
        }
        (other, PrimitiveValue::Boolean(b)) => {
            let num = if *b { 1.0 } else { 0.0 };
            js_abstract_equal(other, &PrimitiveValue::Number(FloatValue::new(num)))
        }
        // null/undefined vs number/string => false
        _ => false,
    }
}
