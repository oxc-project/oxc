use oxc_ast::ast::{
    BinaryExpression, Expression, IdentifierReference, TSEnumDeclaration, TSEnumMemberName,
    UnaryExpression,
};
use oxc_ecmascript::{ToInt32, ToUint32};
use oxc_str::CompactStr;
use oxc_syntax::{
    constant_value::ConstantValue,
    number::ToJsString,
    operator::{BinaryOperator, UnaryOperator},
    scope::ScopeId,
    symbol::SymbolId,
};

use crate::scoping::Scoping;

/// Immutable context shared across recursive evaluation calls.
struct EnumEvalCtx<'s> {
    scope_id: ScopeId,
    enum_symbol_id: Option<SymbolId>,
    scoping: &'s Scoping,
}

/// Evaluate all enum member values in a `TSEnumDeclaration` and store them in `Scoping`.
///
/// Runs during semantic analysis so the transformer can look up pre-computed values
/// to emit constant literals and decide whether const enum declarations can be removed.
///
/// ```ts
/// enum Color { Red, Green, Blue }
/// // Red → 0, Green → 1, Blue → 2 (auto-increment)
///
/// enum Flags { A = 1 << 0, B = 1 << 1, C = A | B }
/// // A → 1, B → 2, C → 3 (constant-folded expressions)
///
/// enum Mixed { X = "hello", Y }
/// // X → "hello", Y → None (can't auto-increment after string)
/// ```
pub fn evaluate_enum_members(decl: &TSEnumDeclaration<'_>, scoping: &mut Scoping) {
    let Some(scope_id) = decl.body.scope_id.get() else { return };

    let enum_symbol_id = decl.id.symbol_id.get();
    if let Some(id) = enum_symbol_id {
        scoping.add_enum_body_scope(id, scope_id);
    }

    let mut prev_value: Option<ConstantValue> = None;

    for member in &decl.body.members {
        let value = if let Some(init) = &member.initializer {
            let ctx = EnumEvalCtx { scope_id, enum_symbol_id, scoping };
            evaluate_expression(init, &ctx)
        } else {
            match &prev_value {
                None => Some(ConstantValue::Number(0.0)),
                Some(ConstantValue::Number(n)) => Some(ConstantValue::Number(n + 1.0)),
                Some(ConstantValue::String(_)) => None,
            }
        };

        if let Some(ref val) = value {
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

fn evaluate_expression(expr: &Expression<'_>, ctx: &EnumEvalCtx<'_>) -> Option<ConstantValue> {
    match expr {
        Expression::Identifier(_)
        | Expression::ComputedMemberExpression(_)
        | Expression::StaticMemberExpression(_)
        | Expression::PrivateFieldExpression(_) => evaluate_ref(expr, ctx),
        Expression::BinaryExpression(expr) => eval_binary_expression(expr, ctx),
        Expression::UnaryExpression(expr) => eval_unary_expression(expr, ctx),
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
                        match evaluate_expression(&lit.expressions[i], ctx)? {
                            ConstantValue::String(s) => value.push_str(&s),
                            ConstantValue::Number(n) => value.push_str(&n.to_js_string()),
                        }
                    }
                }
                Some(ConstantValue::String(CompactStr::from(value.as_str())))
            }
        }
        Expression::ParenthesizedExpression(expr) => evaluate_expression(&expr.expression, ctx),
        _ => None,
    }
}

/// Resolve an identifier or member expression to a previously evaluated enum value.
///
/// Uses a three-level lookup strategy for bare identifiers:
/// 1. Resolved reference — `enum A { X = 1, Y = X }` (X is resolved in the same scope)
/// 2. Scope binding fallback — handles cases where references aren't yet resolved
/// 3. Sibling enum fallback — `enum A { X = 1 } enum A { Y = X }` (merged declarations)
///
/// Also handles cross-enum member access:
/// ```ts
/// enum A { X = 1 }
/// enum B { Y = A.X + 1 }   // StaticMemberExpression
/// enum C { Z = A["X"] + 1 } // ComputedMemberExpression
/// ```
fn evaluate_ref(expr: &Expression<'_>, ctx: &EnumEvalCtx<'_>) -> Option<ConstantValue> {
    match expr {
        Expression::Identifier(ident) => {
            if ident.name == "Infinity" {
                return Some(ConstantValue::Number(f64::INFINITY));
            }
            if ident.name == "NaN" {
                return Some(ConstantValue::Number(f64::NAN));
            }

            if let Some(ref_id) = ident.reference_id.get()
                && let Some(symbol_id) = ctx.scoping.get_reference(ref_id).symbol_id()
            {
                return ctx.scoping.get_enum_member_value(symbol_id).cloned();
            }

            // Fallback: look up as a binding in the current enum body scope.
            if let Some(symbol_id) =
                ctx.scoping.get_binding(ctx.scope_id, ident.name.as_str().into())
                && let Some(value) = ctx.scoping.get_enum_member_value(symbol_id)
            {
                return Some(value.clone());
            }

            // Sibling enum fallback for merged enums.
            find_in_sibling_enum_scopes(
                ident.name.as_str(),
                ctx.scope_id,
                ctx.enum_symbol_id,
                ctx.scoping,
            )
        }
        Expression::StaticMemberExpression(member_expr) => {
            let Expression::Identifier(obj_ident) = &member_expr.object else { return None };
            let obj_symbol_id = resolve_identifier_symbol(obj_ident, ctx)?;
            find_in_enum_body_scopes(member_expr.property.name.as_str(), obj_symbol_id, ctx.scoping)
        }
        Expression::ComputedMemberExpression(member_expr) => {
            let Expression::Identifier(obj_ident) = &member_expr.object else { return None };
            let Expression::StringLiteral(prop_lit) = &member_expr.expression else {
                return None;
            };
            let obj_symbol_id = resolve_identifier_symbol(obj_ident, ctx)?;
            find_in_enum_body_scopes(prop_lit.value.as_str(), obj_symbol_id, ctx.scoping)
        }
        _ => None,
    }
}

/// Resolve an identifier to its symbol, trying the resolved reference first,
/// then falling back to scope binding lookup.
fn resolve_identifier_symbol(
    ident: &IdentifierReference<'_>,
    ctx: &EnumEvalCtx<'_>,
) -> Option<SymbolId> {
    if let Some(ref_id) = ident.reference_id.get()
        && let Some(symbol_id) = ctx.scoping.get_reference(ref_id).symbol_id()
    {
        return Some(symbol_id);
    }
    ctx.scoping.find_binding(ctx.scope_id, ident.name.as_str().into())
}

fn eval_binary_expression(
    expr: &BinaryExpression<'_>,
    ctx: &EnumEvalCtx<'_>,
) -> Option<ConstantValue> {
    let left = evaluate_expression(&expr.left, ctx)?;
    let right = evaluate_expression(&expr.right, ctx)?;

    if matches!(expr.operator, BinaryOperator::Addition)
        && (matches!(left, ConstantValue::String(_)) || matches!(right, ConstantValue::String(_)))
    {
        let mut result = match &left {
            ConstantValue::String(s) => s.to_string(),
            ConstantValue::Number(v) => v.to_js_string(),
        };
        match &right {
            ConstantValue::String(s) => result.push_str(s),
            ConstantValue::Number(v) => result.push_str(&v.to_js_string()),
        }
        return Some(ConstantValue::String(CompactStr::from(result.as_str())));
    }

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
    ctx: &EnumEvalCtx<'_>,
) -> Option<ConstantValue> {
    let value = evaluate_expression(&expr.argument, ctx)?;

    // Babel uses JS coercion for unary on strings: `+"s"` → `"s"`, `-"s"` → NaN, `~"s"` → -1.
    // TypeScript would leave these unevaluated (computed members). We align with Babel.
    let value = match value {
        ConstantValue::Number(v) => v,
        ConstantValue::String(_) => {
            return match expr.operator {
                UnaryOperator::UnaryPlus => Some(value),
                UnaryOperator::UnaryNegation => Some(ConstantValue::Number(f64::NAN)),
                UnaryOperator::BitwiseNot => Some(ConstantValue::Number(-1.0)),
                _ => None,
            };
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
///
/// Handles merged enums where a single enum symbol has multiple body scopes:
/// ```ts
/// enum A { X = 1 }
/// enum A { Y = 2 }
/// // The symbol `A` has two body scopes — this searches both.
/// ```
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

/// For merged enum declarations, find a bare identifier in a *sibling* body scope.
///
/// This handles the case where a later declaration references a member from an
/// earlier declaration without qualification:
/// ```ts
/// enum Foo { A = 1 }
/// enum Foo { B = A + 1 }  // `A` is in the first Foo's scope, not the current one
/// ```
fn find_in_sibling_enum_scopes(
    name: &str,
    current_scope_id: ScopeId,
    enum_symbol_id: Option<SymbolId>,
    scoping: &Scoping,
) -> Option<ConstantValue> {
    let body_scopes = scoping.get_enum_body_scopes(enum_symbol_id?)?;
    for &body_scope in body_scopes {
        if body_scope != current_scope_id
            && let Some(member_sym) = scoping.get_binding(body_scope, name.into())
            && let Some(value) = scoping.get_enum_member_value(member_sym)
        {
            return Some(value.clone());
        }
    }
    None
}
