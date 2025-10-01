use oxc_ast::ast::{CallExpression, Expression, NewExpression};

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
