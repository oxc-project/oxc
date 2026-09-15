use oxc_ast::ast::{
    BinaryExpression, Expression, IdentifierReference, TSEnumDeclaration, UnaryExpression,
};
use oxc_ecmascript::{ToInt32, ToUint32};
use oxc_str::{CompactStr, Ident, JSStr};
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

enum SiblingLookup {
    Missing,
    Unknown,
    Value(ConstantValue),
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
        if decl.r#const {
            scoping.add_const_enum(id);
        }
    }

    // Sentinel: the first member with no initializer evaluates to `-1.0 + 1.0 = 0.0`.
    // Once a member fails to resolve, `prev_value` becomes `None` and stays there, so
    // subsequent auto-increment members correctly propagate "unknown" instead of restarting at 0.
    let mut prev_value: Option<ConstantValue> = Some(ConstantValue::Number(-1.0));

    for member in &decl.body.members {
        let value = if let Some(init) = &member.initializer {
            let ctx = EnumEvalCtx { scope_id, enum_symbol_id, scoping };
            evaluate_expression(init, &ctx)
        } else {
            match &prev_value {
                Some(ConstantValue::Number(n)) => Some(ConstantValue::Number(n + 1.0)),
                None | Some(ConstantValue::String(_)) => None,
            }
        };

        let member_name = member.id.static_name();
        let member_symbol =
            member_name.as_str().and_then(|name| scoping.get_binding(scope_id, name.into()));
        let is_string = matches!(value, Some(ConstantValue::String(_)))
            || (value.is_none()
                && member.initializer.as_ref().is_some_and(|init| {
                    is_string_expression(init, &EnumEvalCtx { scope_id, enum_symbol_id, scoping })
                }));
        if let Some(ref value) = value
            && let Some(symbol_id) = member_symbol
        {
            scoping.set_enum_member_value(symbol_id, value.clone());
        } else if is_string {
            // Preserve the kind even when the constant or member name cannot convert to UTF-8.
            scoping.add_string_enum_member(scope_id, member_name);
        }

        prev_value = value;
    }
}

/// Determine string kind once, alongside constant evaluation, before transformation rewrites references.
fn is_string_expression(expr: &Expression<'_>, ctx: &EnumEvalCtx<'_>) -> bool {
    match expr.without_parentheses() {
        Expression::StringLiteral(_) | Expression::TemplateLiteral(_) => true,
        Expression::BinaryExpression(expr) if expr.operator == BinaryOperator::Addition => {
            is_string_expression(&expr.left, ctx) || is_string_expression(&expr.right, ctx)
        }
        Expression::Identifier(ident) => {
            if let Some(scopes) =
                ctx.enum_symbol_id.and_then(|id| ctx.scoping.get_enum_body_scopes(id))
            {
                for &scope_id in scopes {
                    if scope_id != ctx.scope_id
                        && ctx.scoping.get_binding(scope_id, ident.name).is_some()
                    {
                        return ctx.scoping.is_string_enum_member(scope_id, ident.name.into());
                    }
                }
            }
            resolve_identifier_symbol(ident, ctx).is_some_and(|symbol_id| {
                ctx.scoping.is_string_enum_member(
                    ctx.scoping.symbol_scope_id(symbol_id),
                    ident.name.into(),
                )
            })
        }
        Expression::StaticMemberExpression(member) => {
            is_string_member(&member.object, member.property.name.into(), ctx)
        }
        Expression::ComputedMemberExpression(member) => {
            let Expression::StringLiteral(name) = &member.expression else { return false };
            is_string_member(&member.object, name.value, ctx)
        }
        _ => false,
    }
}

fn is_string_member(object: &Expression<'_>, name: JSStr<'_>, ctx: &EnumEvalCtx<'_>) -> bool {
    let Expression::Identifier(ident) = object else { return false };
    resolve_identifier_symbol(ident, ctx)
        .and_then(|symbol_id| ctx.scoping.get_enum_body_scopes(symbol_id))
        .is_some_and(|scopes| {
            scopes.iter().any(|&scope| ctx.scoping.is_string_enum_member(scope, name))
        })
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
            Some(ConstantValue::String(CompactStr::from(lit.value.as_str()?)))
        }
        Expression::TemplateLiteral(lit) => {
            if let Some(quasi) = lit.single_quasi() {
                Some(ConstantValue::String(CompactStr::from(quasi.as_str()?)))
            } else {
                let mut value = String::new();
                for (i, quasi) in lit.quasis.iter().enumerate() {
                    let cooked_or_raw = match quasi.value.cooked {
                        Some(cooked) => cooked.as_str()?,
                        None => quasi.value.raw.as_str(),
                    };
                    value.push_str(cooked_or_raw);
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
/// 1. Sibling enum fallback — `enum A { X = 1 } enum A { Y = X }` (merged declarations)
/// 2. Lexical binding — `enum A { X = 1, Y = X }` (X is resolved in the same scope)
/// 3. Unbound globals — `Infinity` and `NaN`
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
            match find_in_sibling_enum_scopes(
                ident.name.as_str(),
                ctx.scope_id,
                ctx.enum_symbol_id,
                ctx.scoping,
            ) {
                SiblingLookup::Value(value) => return Some(value),
                SiblingLookup::Unknown => return None,
                SiblingLookup::Missing => {}
            }

            if let Some(symbol_id) = resolve_identifier_symbol(ident, ctx) {
                return ctx.scoping.get_enum_member_value(symbol_id).cloned();
            }

            match ident.name.as_str() {
                "Infinity" => return Some(ConstantValue::Number(f64::INFINITY)),
                "NaN" => return Some(ConstantValue::Number(f64::NAN)),
                _ => {}
            }

            None
        }
        Expression::StaticMemberExpression(member_expr) => {
            let Expression::Identifier(obj_ident) = &member_expr.object else { return None };
            let obj_symbol_id = resolve_identifier_symbol(obj_ident, ctx)?;
            find_in_enum_body_scopes(member_expr.property.name, obj_symbol_id, ctx.scoping)
        }
        Expression::ComputedMemberExpression(member_expr) => {
            let Expression::Identifier(obj_ident) = &member_expr.object else { return None };
            let Expression::StringLiteral(prop_lit) = &member_expr.expression else {
                return None;
            };
            let obj_symbol_id = resolve_identifier_symbol(obj_ident, ctx)?;
            find_in_enum_body_scopes(
                Ident::from(prop_lit.value.as_str()?),
                obj_symbol_id,
                ctx.scoping,
            )
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
    member_name: Ident<'_>,
    enum_symbol_id: SymbolId,
    scoping: &Scoping,
) -> Option<ConstantValue> {
    let body_scopes = scoping.get_enum_body_scopes(enum_symbol_id)?;
    for &body_scope in body_scopes {
        if let Some(member_symbol_id) = scoping.get_binding(body_scope, member_name)
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
///
/// Returns [`SiblingLookup::Unknown`] when the member exists but could not be evaluated, so
/// callers do not fall back to a global with the same name.
fn find_in_sibling_enum_scopes(
    name: &str,
    current_scope_id: ScopeId,
    enum_symbol_id: Option<SymbolId>,
    scoping: &Scoping,
) -> SiblingLookup {
    let Some(enum_symbol_id) = enum_symbol_id else { return SiblingLookup::Missing };
    let Some(body_scopes) = scoping.get_enum_body_scopes(enum_symbol_id) else {
        return SiblingLookup::Missing;
    };

    for &body_scope in body_scopes {
        if body_scope != current_scope_id
            && let Some(member_sym) = scoping.get_binding(body_scope, name.into())
        {
            return scoping
                .get_enum_member_value(member_sym)
                .cloned()
                .map_or(SiblingLookup::Unknown, SiblingLookup::Value);
        }
    }
    SiblingLookup::Missing
}
