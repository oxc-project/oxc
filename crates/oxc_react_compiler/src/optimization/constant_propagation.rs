/// Constant propagation / folding optimization pass.
///
/// Port of `Optimization/ConstantPropagation.ts` from the React Compiler.
///
/// Applies Sparse Conditional Constant Propagation (SCCP):
/// - Tracks known constant values for identifiers
/// - Replaces instructions whose operands are all constants with the result
/// - Prunes unreachable branches when conditions are known constants
/// - Uses fixpoint iteration to propagate constants through the CFG
use oxc_syntax::identifier::is_identifier_name;
use oxc_syntax::operator::{BinaryOperator, UnaryOperator};
use rustc_hash::FxHashMap;

use crate::hir::{
    BlockKind, GotoTerminal, GotoVariant, HIRFunction, IdentifierId, Instruction, InstructionValue,
    NonLocalBinding, Place, PrimitiveValue, PrimitiveValueKind, PropertyLoad, PropertyStore,
    Terminal,
    hir_builder::{mark_instruction_ids, mark_predecessors, remove_unnecessary_try_catch},
    merge_consecutive_blocks::merge_consecutive_blocks,
    types::PropertyLiteral,
};
use crate::ssa::eliminate_redundant_phi::eliminate_redundant_phi;

/// A constant value discovered during propagation.
#[derive(Debug, Clone)]
pub enum Constant {
    Primitive(PrimitiveValueKind),
    LoadGlobal(NonLocalBinding),
}

/// Map from identifier ID to its known constant value.
type Constants = FxHashMap<IdentifierId, Constant>;

/// Run constant propagation on the given function.
pub fn constant_propagation(func: &mut HIRFunction) {
    let mut constants: Constants = FxHashMap::default();
    constant_propagation_impl(func, &mut constants);
}

fn constant_propagation_impl(func: &mut HIRFunction, constants: &mut Constants) {
    loop {
        let have_terminals_changed = apply_constant_propagation(func, constants);
        if !have_terminals_changed {
            break;
        }

        // If terminals have changed, blocks may have become unreachable.
        // Re-run minification passes.
        remove_unnecessary_try_catch(&mut func.body);
        mark_instruction_ids(&mut func.body);
        mark_predecessors(&mut func.body);

        // Prune phi operands that can never be reached
        let block_ids: Vec<_> = func.body.blocks.keys().copied().collect();
        for block_id in &block_ids {
            if let Some(block) = func.body.blocks.get_mut(block_id) {
                let preds = block.preds.clone();
                for phi in &mut block.phis {
                    phi.operands.retain(|pred_block_id, _| preds.contains(pred_block_id));
                }
            }
        }

        // Eliminate newly redundant phis
        eliminate_redundant_phi(func, None);

        // Merge consecutive blocks
        merge_consecutive_blocks(func);
    }
}

fn apply_constant_propagation(func: &mut HIRFunction, constants: &mut Constants) -> bool {
    let mut has_changes = false;

    let block_ids: Vec<_> = func.body.blocks.keys().copied().collect();
    for block_id in block_ids {
        let Some(block) = func.body.blocks.get_mut(&block_id) else { continue };

        // Initialize phi values if all operands have the same known constant value
        for phi in &block.phis {
            if let Some(value) = evaluate_phi(phi, constants) {
                constants.insert(phi.place.identifier.id, value);
            }
        }

        // Evaluate instructions
        let instr_count = block.instructions.len();
        for i in 0..instr_count {
            // Skip the last instruction of sequence blocks (order of evaluation)
            if block.kind == BlockKind::Sequence && i == instr_count - 1 {
                continue;
            }

            let instr = &mut block.instructions[i];
            let lvalue_id = instr.lvalue.identifier.id;
            let value = evaluate_instruction(constants, instr);
            if let Some(constant) = value {
                constants.insert(lvalue_id, constant);
            }
        }

        // Evaluate terminal for constant conditions
        if let Terminal::If(if_term) = &block.terminal {
            let test_value = read(constants, &if_term.test);
            if let Some(Constant::Primitive(prim)) = test_value {
                let is_truthy = primitive_is_truthy(&prim);
                let target_block_id =
                    if is_truthy { if_term.consequent } else { if_term.alternate };
                has_changes = true;
                let id = if_term.id;
                let loc = if_term.loc;
                block.terminal = Terminal::Goto(GotoTerminal {
                    id,
                    block: target_block_id,
                    variant: GotoVariant::Break,
                    loc,
                });
            }
        }
    }

    has_changes
}

/// Evaluate phi nodes: if all operands map to the same constant, return it.
fn evaluate_phi(phi: &crate::hir::Phi, constants: &Constants) -> Option<Constant> {
    let mut value: Option<Constant> = None;
    for operand in phi.operands.values() {
        let operand_value = constants.get(&operand.identifier.id)?;

        if let Some(ref current) = value {
            if !constants_equal(current, operand_value) {
                return None;
            }
        } else {
            value = Some(operand_value.clone());
        }
    }
    value
}

fn evaluate_instruction(constants: &mut Constants, instr: &mut Instruction) -> Option<Constant> {
    // Handle FunctionExpression and ObjectMethod recursion first, since they need
    // exclusive mutable access to the nested function body.
    match &mut instr.value {
        InstructionValue::FunctionExpression(v) => {
            constant_propagation_impl(&mut v.lowered_func.func, constants);
            return None;
        }
        InstructionValue::ObjectMethod(v) => {
            constant_propagation_impl(&mut v.lowered_func.func, constants);
            return None;
        }
        _ => {}
    }

    match &instr.value {
        InstructionValue::Primitive(v) => Some(Constant::Primitive(v.value.clone())),
        InstructionValue::LoadGlobal(v) => Some(Constant::LoadGlobal(v.binding.clone())),
        InstructionValue::LoadLocal(v) => {
            // If the loaded variable has a known constant value, propagate it
            let place_value = constants.get(&v.place.identifier.id).cloned();
            if let Some(ref c) = place_value {
                instr.value = constant_to_instruction_value(c, instr.value.loc());
            }
            place_value
        }
        InstructionValue::StoreLocal(v) => {
            let place_value = read(constants, &v.value);
            if let Some(ref c) = place_value {
                constants.insert(v.lvalue.place.identifier.id, c.clone());
            }
            place_value
        }
        InstructionValue::ComputedLoad(v) => {
            let prop_const = read(constants, &v.property);
            if let Some(Constant::Primitive(prim)) = prop_const
                && let Some(prop_literal) = primitive_to_property_literal(&prim)
            {
                let object_clone = v.object.clone();
                let loc_val = v.loc;
                instr.value = InstructionValue::PropertyLoad(PropertyLoad {
                    object: object_clone,
                    property: prop_literal,
                    loc: loc_val,
                });
            }
            None
        }
        InstructionValue::ComputedStore(v) => {
            let prop_const = read(constants, &v.property);
            if let Some(Constant::Primitive(prim)) = prop_const
                && let Some(prop_literal) = primitive_to_property_literal(&prim)
            {
                let object_clone = v.object.clone();
                let value_clone = v.value.clone();
                let loc_val = v.loc;
                instr.value = InstructionValue::PropertyStore(PropertyStore {
                    object: object_clone,
                    property: prop_literal,
                    value: value_clone,
                    loc: loc_val,
                });
            }
            None
        }
        InstructionValue::UnaryExpression(v) => match v.operator {
            UnaryOperator::LogicalNot => {
                let operand = read(constants, &v.value)?;
                if let Constant::Primitive(prim) = operand {
                    let negated = !primitive_is_truthy(&prim);
                    let result = Constant::Primitive(PrimitiveValueKind::Boolean(negated));
                    let loc = v.loc;
                    instr.value = InstructionValue::Primitive(PrimitiveValue {
                        value: PrimitiveValueKind::Boolean(negated),
                        loc,
                    });
                    return Some(result);
                }
                None
            }
            UnaryOperator::UnaryNegation => {
                let operand = read(constants, &v.value)?;
                if let Constant::Primitive(PrimitiveValueKind::Number(n)) = operand {
                    let negated = -n;
                    let result = Constant::Primitive(PrimitiveValueKind::Number(negated));
                    let loc = v.loc;
                    instr.value = InstructionValue::Primitive(PrimitiveValue {
                        value: PrimitiveValueKind::Number(negated),
                        loc,
                    });
                    return Some(result);
                }
                None
            }
            UnaryOperator::Typeof => {
                let operand = read(constants, &v.value)?;
                if let Constant::Primitive(prim) = operand {
                    let type_str = match prim {
                        PrimitiveValueKind::Number(_) => "number",
                        PrimitiveValueKind::Boolean(_) => "boolean",
                        PrimitiveValueKind::String(_) => "string",
                        PrimitiveValueKind::Null => "object",
                        PrimitiveValueKind::Undefined => "undefined",
                    };
                    let result =
                        Constant::Primitive(PrimitiveValueKind::String(type_str.to_string()));
                    let loc = v.loc;
                    instr.value = InstructionValue::Primitive(PrimitiveValue {
                        value: PrimitiveValueKind::String(type_str.to_string()),
                        loc,
                    });
                    return Some(result);
                }
                None
            }
            _ => None,
        },
        InstructionValue::BinaryExpression(v) => {
            let lhs_const = read(constants, &v.left)?;
            let rhs_const = read(constants, &v.right)?;
            if let (Constant::Primitive(lhs), Constant::Primitive(rhs)) = (&lhs_const, &rhs_const) {
                let result = evaluate_binary(v.operator, lhs, rhs)?;
                let loc = v.loc;
                instr.value =
                    InstructionValue::Primitive(PrimitiveValue { value: result.clone(), loc });
                return Some(Constant::Primitive(result));
            }
            None
        }
        InstructionValue::TemplateLiteral(v) => {
            let loc = v.loc;
            // No subexpressions: fold quasis into a single string
            if v.subexprs.is_empty() {
                let mut result_string = String::new();
                for quasi in &v.quasis {
                    if let Some(ref cooked) = quasi.cooked {
                        result_string.push_str(cooked);
                    } else {
                        return None;
                    }
                }
                let result = Constant::Primitive(PrimitiveValueKind::String(result_string.clone()));
                instr.value = InstructionValue::Primitive(PrimitiveValue {
                    value: PrimitiveValueKind::String(result_string),
                    loc,
                });
                return Some(result);
            }

            // Must have quasis.len() == subexprs.len() + 1
            if v.subexprs.len() + 1 != v.quasis.len() {
                return None;
            }

            // Check all quasis have cooked values
            if v.quasis.iter().any(|q| q.cooked.is_none()) {
                return None;
            }

            // Build the result string
            let first_cooked = v.quasis[0].cooked.as_ref()?;
            let mut result_string = first_cooked.clone();
            let mut quasi_index = 1;

            for subexpr in &v.subexprs {
                let subexpr_value = read(constants, subexpr)?;
                if let Constant::Primitive(prim) = subexpr_value {
                    let expr_str = primitive_to_string(&prim);
                    result_string.push_str(&expr_str);
                } else {
                    return None;
                }
                let suffix = v.quasis[quasi_index].cooked.as_ref()?;
                result_string.push_str(suffix);
                quasi_index += 1;
            }

            let result = Constant::Primitive(PrimitiveValueKind::String(result_string.clone()));
            instr.value = InstructionValue::Primitive(PrimitiveValue {
                value: PrimitiveValueKind::String(result_string),
                loc,
            });
            Some(result)
        }
        // FunctionExpression and ObjectMethod are handled at the top of this function
        _ => None,
    }
}

fn read(constants: &Constants, place: &Place) -> Option<Constant> {
    constants.get(&place.identifier.id).cloned()
}

/// Check if two constants are equal (used in phi evaluation).
pub fn constants_equal(a: &Constant, b: &Constant) -> bool {
    match (a, b) {
        (Constant::Primitive(pa), Constant::Primitive(pb)) => match (pa, pb) {
            (PrimitiveValueKind::Number(a), PrimitiveValueKind::Number(b)) => {
                (a - b).abs() < f64::EPSILON
            }
            (PrimitiveValueKind::Boolean(a), PrimitiveValueKind::Boolean(b)) => a == b,
            (PrimitiveValueKind::String(a), PrimitiveValueKind::String(b)) => a == b,
            (PrimitiveValueKind::Null, PrimitiveValueKind::Null)
            | (PrimitiveValueKind::Undefined, PrimitiveValueKind::Undefined) => true,
            _ => false,
        },
        (Constant::LoadGlobal(a), Constant::LoadGlobal(b)) => match (a, b) {
            (NonLocalBinding::Global { name: a }, NonLocalBinding::Global { name: b }) => a == b,
            _ => false,
        },
        _ => false,
    }
}

fn primitive_is_truthy(value: &PrimitiveValueKind) -> bool {
    match value {
        PrimitiveValueKind::Boolean(b) => *b,
        PrimitiveValueKind::Number(n) => *n != 0.0 && !n.is_nan(),
        PrimitiveValueKind::String(s) => !s.is_empty(),
        PrimitiveValueKind::Null | PrimitiveValueKind::Undefined => false,
    }
}

/// Convert a primitive constant to a property literal for ComputedLoad -> PropertyLoad rewriting.
/// Returns None if the primitive is not a valid property key (valid identifier string or number).
fn primitive_to_property_literal(prim: &PrimitiveValueKind) -> Option<PropertyLiteral> {
    match prim {
        PrimitiveValueKind::String(s) if is_identifier_name(s) => {
            Some(PropertyLiteral::String(s.clone()))
        }
        PrimitiveValueKind::Number(n) => {
            // Convert f64 to i64 for property literal if it's an integer.
            // These casts intentionally truncate: we check the round-trip below.
            #[expect(
                clippy::cast_possible_truncation,
                clippy::cast_precision_loss,
                clippy::float_cmp
            )]
            let matches = (*n as i64) as f64 == *n;
            if matches {
                #[expect(clippy::cast_possible_truncation)]
                let i = *n as i64;
                Some(PropertyLiteral::Number(i))
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Convert a constant to an InstructionValue for rewriting LoadLocal instructions.
fn constant_to_instruction_value(
    constant: &Constant,
    loc: crate::compiler_error::SourceLocation,
) -> InstructionValue {
    match constant {
        Constant::Primitive(prim) => {
            InstructionValue::Primitive(PrimitiveValue { value: prim.clone(), loc })
        }
        Constant::LoadGlobal(binding) => {
            InstructionValue::LoadGlobal(crate::hir::LoadGlobal { binding: binding.clone(), loc })
        }
    }
}

/// Cast an f64 to i32 per the JavaScript ToInt32 specification.
#[expect(clippy::cast_possible_truncation)]
fn to_int32(n: f64) -> i32 {
    n as i32
}

/// Cast an f64 to u32 per the JavaScript ToUint32 specification.
#[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn to_uint32(n: f64) -> u32 {
    n as u32
}

/// Evaluate a binary expression with two primitive operands.
fn evaluate_binary(
    operator: BinaryOperator,
    lhs: &PrimitiveValueKind,
    rhs: &PrimitiveValueKind,
) -> Option<PrimitiveValueKind> {
    match operator {
        BinaryOperator::Addition => match (lhs, rhs) {
            (PrimitiveValueKind::Number(a), PrimitiveValueKind::Number(b)) => {
                Some(PrimitiveValueKind::Number(a + b))
            }
            (PrimitiveValueKind::String(a), PrimitiveValueKind::String(b)) => {
                let mut result = a.clone();
                result.push_str(b);
                Some(PrimitiveValueKind::String(result))
            }
            _ => None,
        },
        BinaryOperator::Subtraction => match (lhs, rhs) {
            (PrimitiveValueKind::Number(a), PrimitiveValueKind::Number(b)) => {
                Some(PrimitiveValueKind::Number(a - b))
            }
            _ => None,
        },
        BinaryOperator::Multiplication => match (lhs, rhs) {
            (PrimitiveValueKind::Number(a), PrimitiveValueKind::Number(b)) => {
                Some(PrimitiveValueKind::Number(a * b))
            }
            _ => None,
        },
        BinaryOperator::Division => match (lhs, rhs) {
            (PrimitiveValueKind::Number(a), PrimitiveValueKind::Number(b)) => {
                Some(PrimitiveValueKind::Number(a / b))
            }
            _ => None,
        },
        BinaryOperator::Remainder => match (lhs, rhs) {
            (PrimitiveValueKind::Number(a), PrimitiveValueKind::Number(b)) => {
                Some(PrimitiveValueKind::Number(a % b))
            }
            _ => None,
        },
        BinaryOperator::Exponential => match (lhs, rhs) {
            (PrimitiveValueKind::Number(a), PrimitiveValueKind::Number(b)) => {
                Some(PrimitiveValueKind::Number(a.powf(*b)))
            }
            _ => None,
        },
        BinaryOperator::BitwiseOR => match (lhs, rhs) {
            (PrimitiveValueKind::Number(a), PrimitiveValueKind::Number(b)) => {
                Some(PrimitiveValueKind::Number(f64::from(to_int32(*a) | to_int32(*b))))
            }
            _ => None,
        },
        BinaryOperator::BitwiseAnd => match (lhs, rhs) {
            (PrimitiveValueKind::Number(a), PrimitiveValueKind::Number(b)) => {
                Some(PrimitiveValueKind::Number(f64::from(to_int32(*a) & to_int32(*b))))
            }
            _ => None,
        },
        BinaryOperator::BitwiseXOR => match (lhs, rhs) {
            (PrimitiveValueKind::Number(a), PrimitiveValueKind::Number(b)) => {
                Some(PrimitiveValueKind::Number(f64::from(to_int32(*a) ^ to_int32(*b))))
            }
            _ => None,
        },
        BinaryOperator::ShiftLeft => match (lhs, rhs) {
            (PrimitiveValueKind::Number(a), PrimitiveValueKind::Number(b)) => {
                let shift = to_uint32(*b) & 0x1f;
                Some(PrimitiveValueKind::Number(f64::from(to_int32(*a) << shift)))
            }
            _ => None,
        },
        BinaryOperator::ShiftRight => match (lhs, rhs) {
            (PrimitiveValueKind::Number(a), PrimitiveValueKind::Number(b)) => {
                let shift = to_uint32(*b) & 0x1f;
                Some(PrimitiveValueKind::Number(f64::from(to_int32(*a) >> shift)))
            }
            _ => None,
        },
        BinaryOperator::ShiftRightZeroFill => match (lhs, rhs) {
            (PrimitiveValueKind::Number(a), PrimitiveValueKind::Number(b)) => {
                let shift = to_uint32(*b) & 0x1f;
                Some(PrimitiveValueKind::Number(f64::from(to_uint32(*a) >> shift)))
            }
            _ => None,
        },
        BinaryOperator::LessThan => match (lhs, rhs) {
            (PrimitiveValueKind::Number(a), PrimitiveValueKind::Number(b)) => {
                Some(PrimitiveValueKind::Boolean(a < b))
            }
            _ => None,
        },
        BinaryOperator::LessEqualThan => match (lhs, rhs) {
            (PrimitiveValueKind::Number(a), PrimitiveValueKind::Number(b)) => {
                Some(PrimitiveValueKind::Boolean(a <= b))
            }
            _ => None,
        },
        BinaryOperator::GreaterThan => match (lhs, rhs) {
            (PrimitiveValueKind::Number(a), PrimitiveValueKind::Number(b)) => {
                Some(PrimitiveValueKind::Boolean(a > b))
            }
            _ => None,
        },
        BinaryOperator::GreaterEqualThan => match (lhs, rhs) {
            (PrimitiveValueKind::Number(a), PrimitiveValueKind::Number(b)) => {
                Some(PrimitiveValueKind::Boolean(a >= b))
            }
            _ => None,
        },
        BinaryOperator::StrictEquality => {
            Some(PrimitiveValueKind::Boolean(primitives_strict_equal(lhs, rhs)))
        }
        BinaryOperator::StrictInequality => {
            Some(PrimitiveValueKind::Boolean(!primitives_strict_equal(lhs, rhs)))
        }
        BinaryOperator::Equality => {
            Some(PrimitiveValueKind::Boolean(primitives_loose_equal(lhs, rhs)))
        }
        BinaryOperator::Inequality => {
            Some(PrimitiveValueKind::Boolean(!primitives_loose_equal(lhs, rhs)))
        }
        BinaryOperator::In | BinaryOperator::Instanceof => None,
    }
}

/// JS strict equality (===) for primitive values.
fn primitives_strict_equal(a: &PrimitiveValueKind, b: &PrimitiveValueKind) -> bool {
    match (a, b) {
        (PrimitiveValueKind::Number(a), PrimitiveValueKind::Number(b)) => {
            // NaN !== NaN, and +0 === -0 per spec
            if a.is_nan() || b.is_nan() {
                return false;
            }
            #[expect(clippy::float_cmp)]
            let eq = a == b;
            eq
        }
        (PrimitiveValueKind::Boolean(a), PrimitiveValueKind::Boolean(b)) => a == b,
        (PrimitiveValueKind::String(a), PrimitiveValueKind::String(b)) => a == b,
        (PrimitiveValueKind::Null, PrimitiveValueKind::Null)
        | (PrimitiveValueKind::Undefined, PrimitiveValueKind::Undefined) => true,
        _ => false,
    }
}

/// JS loose equality (==) for primitive values.
/// Handles null == undefined and the basic same-type cases.
fn primitives_loose_equal(a: &PrimitiveValueKind, b: &PrimitiveValueKind) -> bool {
    match (a, b) {
        // null == undefined and undefined == null
        (PrimitiveValueKind::Null, PrimitiveValueKind::Undefined)
        | (PrimitiveValueKind::Undefined, PrimitiveValueKind::Null) => true,
        // Same-type comparisons use strict equality semantics
        (PrimitiveValueKind::Number(_), PrimitiveValueKind::Number(_))
        | (PrimitiveValueKind::Boolean(_), PrimitiveValueKind::Boolean(_))
        | (PrimitiveValueKind::String(_), PrimitiveValueKind::String(_))
        | (PrimitiveValueKind::Null, PrimitiveValueKind::Null)
        | (PrimitiveValueKind::Undefined, PrimitiveValueKind::Undefined) => {
            primitives_strict_equal(a, b)
        }
        _ => false,
    }
}

/// Convert a primitive to its string representation for template literal folding.
fn primitive_to_string(prim: &PrimitiveValueKind) -> String {
    match prim {
        PrimitiveValueKind::String(s) => s.clone(),
        PrimitiveValueKind::Number(n) => {
            if *n == 0.0 {
                "0".to_string()
            } else if n.is_infinite() {
                if n.is_sign_positive() { "Infinity".to_string() } else { "-Infinity".to_string() }
            } else if n.is_nan() {
                "NaN".to_string()
            } else {
                format!("{n}")
            }
        }
        PrimitiveValueKind::Boolean(b) => {
            if *b {
                "true".to_string()
            } else {
                "false".to_string()
            }
        }
        PrimitiveValueKind::Null => "null".to_string(),
        PrimitiveValueKind::Undefined => "undefined".to_string(),
    }
}
