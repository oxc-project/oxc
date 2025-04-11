use oxc_ast::ast::CallExpression;

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

    if member_expr.object().is_specific_id("Promise")
        && PROMISE_STATIC_METHODS.binary_search(&prop_name).is_ok()
    {
        return Some(prop_name.into());
    }

    None
}
