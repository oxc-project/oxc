use oxc_ast::ast::{
    BinaryExpression, Expression, IdentifierReference, TSEnumDeclaration, TSEnumMemberName,
    UnaryExpression,
};
use oxc_ecmascript::{ToInt32, ToUint32};
use oxc_span::CompactStr;
use oxc_syntax::{
    constant_value::ConstantValue,
    number::ToJsString,
    operator::{BinaryOperator, UnaryOperator},
    scope::ScopeId,
    symbol::{SymbolFlags, SymbolId},
};

use crate::scoping::Scoping;

/// Evaluate all enum member values in a `TSEnumDeclaration` and store them in `Scoping`.
///
/// This mirrors the constant-folding logic in `crates/oxc_transformer/src/typescript/enum.rs`
/// but runs during semantic analysis so that the computed values are available earlier in the
/// pipeline.
pub fn evaluate_enum_members(decl: &TSEnumDeclaration<'_>, scoping: &mut Scoping) {
    let Some(scope_id) = decl.body.scope_id.get() else { return };

    // Store enum declaration → body scope mapping for cross-enum references
    if let Some(enum_symbol_id) = decl.id.symbol_id.get() {
        scoping.add_enum_body_scope(enum_symbol_id, scope_id);
    }

    let mut prev_value: Option<ConstantValue> = None;

    for member in &decl.body.members {
        let value = if let Some(init) = &member.initializer {
            evaluate_expression(init, scope_id, scoping)
        } else {
            // Auto-increment: first member defaults to 0, subsequent members increment
            // the previous numeric value by 1. If the previous value was a string, we
            // cannot auto-increment.
            match &prev_value {
                None => Some(ConstantValue::Number(0.0)),
                Some(ConstantValue::Number(n)) => Some(ConstantValue::Number(n + 1.0)),
                Some(ConstantValue::String(_)) => None,
            }
        };

        if let Some(ref val) = value {
            // Extract the member name, guarding against ComputedTemplateString which
            // cannot be statically resolved.
            let member_name = match &member.id {
                TSEnumMemberName::Identifier(ident) => Some(ident.name.as_str()),
                TSEnumMemberName::String(lit) | TSEnumMemberName::ComputedString(lit) => {
                    Some(lit.value.as_str())
                }
                TSEnumMemberName::ComputedTemplateString(_) => None,
            };
            if let Some(name) = member_name
                && let Some(symbol_id) = scoping.get_binding(scope_id, name.into())
            {
                scoping.set_enum_member_value(symbol_id, val.clone());
            }
        }

        prev_value = value;
    }
}

fn evaluate_expression(
    expr: &Expression<'_>,
    scope_id: ScopeId,
    scoping: &Scoping,
) -> Option<ConstantValue> {
    match expr {
        Expression::Identifier(_)
        | Expression::ComputedMemberExpression(_)
        | Expression::StaticMemberExpression(_)
        | Expression::PrivateFieldExpression(_) => evaluate_ref(expr, scope_id, scoping),
        Expression::BinaryExpression(expr) => eval_binary_expression(expr, scope_id, scoping),
        Expression::UnaryExpression(expr) => eval_unary_expression(expr, scope_id, scoping),
        Expression::NumericLiteral(lit) => Some(ConstantValue::Number(lit.value)),
        Expression::StringLiteral(lit) => {
            Some(ConstantValue::String(CompactStr::from(lit.value.as_str())))
        }
        Expression::TemplateLiteral(lit) => {
            if let Some(quasi) = lit.single_quasi() {
                Some(ConstantValue::String(CompactStr::from(quasi.as_str())))
            } else {
                let mut value = String::new();
                for (i, quasi) in lit.quasis.iter().enumerate() {
                    let cooked_or_raw = quasi.value.cooked.as_ref().unwrap_or(&quasi.value.raw);
                    value.push_str(cooked_or_raw.as_str());
                    if i < lit.expressions.len() {
                        match evaluate_expression(&lit.expressions[i], scope_id, scoping)? {
                            ConstantValue::String(s) => value.push_str(&s),
                            ConstantValue::Number(n) => value.push_str(&n.to_js_string()),
                        }
                    }
                }
                Some(ConstantValue::String(CompactStr::from(value.as_str())))
            }
        }
        Expression::ParenthesizedExpression(expr) => {
            evaluate_expression(&expr.expression, scope_id, scoping)
        }
        _ => None,
    }
}

fn evaluate_ref(
    expr: &Expression<'_>,
    scope_id: ScopeId,
    scoping: &Scoping,
) -> Option<ConstantValue> {
    match expr {
        Expression::Identifier(ident) => {
            if ident.name == "Infinity" {
                return Some(ConstantValue::Number(f64::INFINITY));
            }
            if ident.name == "NaN" {
                return Some(ConstantValue::Number(f64::NAN));
            }

            // Try to resolve via the reference to an already-evaluated enum member.
            // First attempt: use the resolved reference (if the reference was already
            // resolved by the time this evaluator runs).
            if let Some(ref_id) = ident.reference_id.get()
                && let Some(symbol_id) = scoping.get_reference(ref_id).symbol_id()
            {
                return scoping.get_enum_member_value(symbol_id).cloned();
            }

            // Fallback: look up the identifier name as a binding in the current
            // enum body scope. Handles `enum A { X = 1, Y = X + 1 }`.
            if let Some(symbol_id) = scoping.get_binding(scope_id, ident.name.as_str().into())
                && let Some(value) = scoping.get_enum_member_value(symbol_id)
            {
                return Some(value.clone());
            }

            // Sibling enum fallback: for merged enums like `enum x { y = 0 } enum x { z = y + 1 }`,
            // the bare `y` is in the first declaration's body scope. Search all body scopes
            // of the same enum (identified by walking up to the parent enum symbol).
            find_in_sibling_enum_scopes(ident.name.as_str(), scope_id, scoping)
        }
        Expression::StaticMemberExpression(member_expr) => {
            // Handle cross-enum references like `A.X`
            let Expression::Identifier(obj_ident) = &member_expr.object else { return None };
            let obj_symbol_id = resolve_identifier_symbol(obj_ident, scope_id, scoping)?;
            find_in_enum_body_scopes(member_expr.property.name.as_str(), obj_symbol_id, scoping)
        }
        Expression::ComputedMemberExpression(member_expr) => {
            // Handle cross-enum references like `A["X"]`
            let Expression::Identifier(obj_ident) = &member_expr.object else { return None };
            let Expression::StringLiteral(prop_lit) = &member_expr.expression else {
                return None;
            };
            let obj_symbol_id = resolve_identifier_symbol(obj_ident, scope_id, scoping)?;
            find_in_enum_body_scopes(prop_lit.value.as_str(), obj_symbol_id, scoping)
        }
        _ => None,
    }
}

/// Resolve an identifier to its symbol, trying the resolved reference first,
/// then falling back to scope binding lookup. The fallback is needed because
/// references may not yet be resolved during semantic analysis.
fn resolve_identifier_symbol(
    ident: &IdentifierReference<'_>,
    scope_id: ScopeId,
    scoping: &Scoping,
) -> Option<SymbolId> {
    if let Some(ref_id) = ident.reference_id.get()
        && let Some(symbol_id) = scoping.get_reference(ref_id).symbol_id()
    {
        return Some(symbol_id);
    }
    scoping.find_binding(scope_id, ident.name.as_str().into())
}

fn eval_binary_expression(
    expr: &BinaryExpression<'_>,
    scope_id: ScopeId,
    scoping: &Scoping,
) -> Option<ConstantValue> {
    let left = evaluate_expression(&expr.left, scope_id, scoping)?;
    let right = evaluate_expression(&expr.right, scope_id, scoping)?;

    // String concatenation when `+` is used and either operand is a string.
    if matches!(expr.operator, BinaryOperator::Addition)
        && (matches!(left, ConstantValue::String(_)) || matches!(right, ConstantValue::String(_)))
    {
        let left_string = match &left {
            ConstantValue::String(s) => s.to_string(),
            ConstantValue::Number(v) => v.to_js_string(),
        };
        let right_string = match &right {
            ConstantValue::String(s) => s.to_string(),
            ConstantValue::Number(v) => v.to_js_string(),
        };
        let mut result = left_string;
        result.push_str(&right_string);
        return Some(ConstantValue::String(CompactStr::from(result.as_str())));
    }

    // All other operators require numeric operands.
    let left = match left {
        ConstantValue::Number(v) => v,
        ConstantValue::String(_) => return None,
    };
    let right = match right {
        ConstantValue::Number(v) => v,
        ConstantValue::String(_) => return None,
    };

    match expr.operator {
        BinaryOperator::ShiftRight => Some(ConstantValue::Number(f64::from(
            left.to_int_32().wrapping_shr(right.to_uint_32()),
        ))),
        BinaryOperator::ShiftRightZeroFill => Some(ConstantValue::Number(f64::from(
            left.to_uint_32().wrapping_shr(right.to_uint_32()),
        ))),
        BinaryOperator::ShiftLeft => Some(ConstantValue::Number(f64::from(
            left.to_int_32().wrapping_shl(right.to_uint_32()),
        ))),
        BinaryOperator::BitwiseXOR => {
            Some(ConstantValue::Number(f64::from(left.to_int_32() ^ right.to_int_32())))
        }
        BinaryOperator::BitwiseOR => {
            Some(ConstantValue::Number(f64::from(left.to_int_32() | right.to_int_32())))
        }
        BinaryOperator::BitwiseAnd => {
            Some(ConstantValue::Number(f64::from(left.to_int_32() & right.to_int_32())))
        }
        BinaryOperator::Multiplication => Some(ConstantValue::Number(left * right)),
        BinaryOperator::Division => Some(ConstantValue::Number(left / right)),
        BinaryOperator::Addition => Some(ConstantValue::Number(left + right)),
        BinaryOperator::Subtraction => Some(ConstantValue::Number(left - right)),
        BinaryOperator::Remainder => Some(ConstantValue::Number(left % right)),
        BinaryOperator::Exponential => Some(ConstantValue::Number(left.powf(right))),
        _ => None,
    }
}

fn eval_unary_expression(
    expr: &UnaryExpression<'_>,
    scope_id: ScopeId,
    scoping: &Scoping,
) -> Option<ConstantValue> {
    let value = evaluate_expression(&expr.argument, scope_id, scoping)?;

    let value = match value {
        ConstantValue::Number(v) => v,
        ConstantValue::String(_) => {
            let result = if expr.operator == UnaryOperator::UnaryNegation {
                ConstantValue::Number(f64::NAN)
            } else if expr.operator == UnaryOperator::BitwiseNot {
                ConstantValue::Number(-1.0)
            } else {
                value
            };
            return Some(result);
        }
    };

    match expr.operator {
        UnaryOperator::UnaryPlus => Some(ConstantValue::Number(value)),
        UnaryOperator::UnaryNegation => Some(ConstantValue::Number(-value)),
        UnaryOperator::BitwiseNot => Some(ConstantValue::Number(f64::from(!value.to_int_32()))),
        _ => None,
    }
}

/// Search all body scopes of an enum for a member by name.
/// Handles merged enums where `get_enum_body_scopes` returns multiple scopes.
fn find_in_enum_body_scopes(
    member_name: &str,
    enum_symbol_id: SymbolId,
    scoping: &Scoping,
) -> Option<ConstantValue> {
    let body_scopes = scoping.get_enum_body_scopes(enum_symbol_id)?;
    for &body_scope in body_scopes {
        if let Some(member_symbol_id) = scoping.get_binding(body_scope, member_name.into())
            && let Some(value) = scoping.get_enum_member_value(member_symbol_id)
        {
            return Some(value.clone());
        }
    }
    None
}

/// For sibling enum declarations (`enum x { y } enum x { z = y + 1 }`),
/// find a bare identifier by searching sibling body scopes of the *same* enum.
/// Only searches the enum that owns `current_scope_id`, avoiding false positives
/// when two different enums in the same parent scope share a member name.
fn find_in_sibling_enum_scopes(
    name: &str,
    current_scope_id: ScopeId,
    scoping: &Scoping,
) -> Option<ConstantValue> {
    let parent_scope = scoping.scope_parent_id(current_scope_id)?;

    // Find which enum symbol owns the current scope
    for &sym_id in scoping.get_bindings(parent_scope).values() {
        let flags = scoping.symbol_flags(sym_id);
        if !(flags.is_const_enum() || flags.contains(SymbolFlags::RegularEnum)) {
            continue;
        }
        let Some(body_scopes) = scoping.get_enum_body_scopes(sym_id) else { continue };
        // Only search if this enum owns the current scope
        if !body_scopes.contains(&current_scope_id) {
            continue;
        }
        // Found our enum — search all its body scopes for the member
        for &body_scope in body_scopes {
            if body_scope != current_scope_id
                && let Some(member_sym) = scoping.get_binding(body_scope, name.into())
                && let Some(value) = scoping.get_enum_member_value(member_sym)
            {
                return Some(value.clone());
            }
        }
        return None; // Found our enum but member not in sibling scopes
    }
    None
}
