use rustc_hash::FxHashSet;

use oxc_ast::{
    AstKind,
    ast::{CallExpression, Expression, IdentifierReference, NewExpression},
};
use oxc_semantic::SymbolId;

use crate::context::LintContext;

// https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise
pub const PROMISE_STATIC_METHODS: [&str; 7] =
    ["all", "allSettled", "any", "race", "reject", "resolve", "withResolvers"];

pub fn is_promise(call_expr: &CallExpression) -> Option<String> {
    let member_expr = call_expr.callee.get_member_expr()?;
    let prop_name = member_expr.static_property_name()?;

    // hello.then(), hello.catch(), hello.finally()
    if matches!(prop_name, "then" | "catch" | "finally") {
        return Some(prop_name.into());
    }

    if member_expr.object().is_specific_id("Promise") && PROMISE_STATIC_METHODS.contains(&prop_name)
    {
        return Some(prop_name.into());
    }

    None
}

/// Like [`is_promise`], but avoids obvious false positives for non-Promise receivers.
///
/// This is intentionally conservative: if we cannot prove the receiver is non-Promise,
/// we keep the original behavior.
pub fn is_promise_with_context<'a>(
    call_expr: &CallExpression<'a>,
    ctx: &LintContext<'a>,
) -> Option<String> {
    let prop_name = is_promise(call_expr)?;

    // Promise static methods are already explicit (`Promise.resolve`, etc.).
    if !matches!(prop_name.as_str(), "then" | "catch" | "finally") {
        return Some(prop_name);
    }

    let member_expr = call_expr.callee.get_member_expr()?;
    let mut visited = FxHashSet::<SymbolId>::default();
    if matches!(
        classify_receiver(member_expr.object().get_inner_expression(), ctx, &mut visited),
        ReceiverKind::NotPromise
    ) {
        return None;
    }

    Some(prop_name)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ReceiverKind {
    PromiseLike,
    NotPromise,
    Unknown,
}

fn classify_receiver<'a>(
    expr: &Expression<'a>,
    ctx: &LintContext<'a>,
    visited: &mut FxHashSet<SymbolId>,
) -> ReceiverKind {
    match expr.get_inner_expression() {
        Expression::CallExpression(call_expr) => {
            if is_promise(call_expr).is_some() {
                ReceiverKind::PromiseLike
            } else {
                // Arbitrary call return types are unknown.
                ReceiverKind::Unknown
            }
        }
        Expression::NewExpression(new_expr) => {
            if is_promise_constructor(new_expr) {
                ReceiverKind::PromiseLike
            } else {
                ReceiverKind::NotPromise
            }
        }
        Expression::Identifier(ident) => classify_identifier_receiver(ident, ctx, visited),
        // These expression kinds are never Promise instances.
        Expression::ObjectExpression(_)
        | Expression::ArrayExpression(_)
        | Expression::FunctionExpression(_)
        | Expression::ArrowFunctionExpression(_)
        | Expression::ClassExpression(_) => ReceiverKind::NotPromise,
        _ => ReceiverKind::Unknown,
    }
}

fn classify_identifier_receiver<'a>(
    ident: &IdentifierReference<'a>,
    ctx: &LintContext<'a>,
    visited: &mut FxHashSet<SymbolId>,
) -> ReceiverKind {
    let Some(symbol_id) = ctx.scoping().get_reference(ident.reference_id()).symbol_id() else {
        return ReceiverKind::Unknown;
    };

    if !visited.insert(symbol_id) {
        return ReceiverKind::Unknown;
    }

    let declaration = ctx.nodes().get_node(ctx.scoping().symbol_declaration(symbol_id));
    match declaration.kind() {
        AstKind::VariableDeclarator(var_decl) => var_decl
            .init
            .as_ref()
            .map_or(ReceiverKind::Unknown, |init| classify_receiver(init, ctx, visited)),
        _ => ReceiverKind::Unknown,
    }
}

pub fn is_promise_constructor(new_expr: &NewExpression) -> bool {
    new_expr.callee.is_specific_id("Promise")
}

pub fn get_promise_constructor_inline_executor<'a>(
    new_expr: &'a NewExpression<'a>,
) -> Option<&'a Expression<'a>> {
    if !is_promise_constructor(new_expr) {
        return None;
    }
    if new_expr.arguments.len() != 1 {
        return None;
    }
    new_expr.arguments[0]
        .as_expression()
        .and_then(|expr| if expr.is_function() { Some(expr) } else { None })
}
