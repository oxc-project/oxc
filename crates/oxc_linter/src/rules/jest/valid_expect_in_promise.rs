use oxc_ast::{
    AstKind,
    ast::{
        ArrayExpressionElement, BindingPatternKind, CallExpression, Expression, Statement,
        VariableDeclarator,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{PossibleJestNode, get_node_name, parse_general_jest_fn_call},
};

fn valid_expect_in_promise_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "Expect in promise should be awaited or returned to ensure the test waits for it.",
    )
    .with_help("Return or await the promise to ensure Jest waits for the assertion")
    .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct ValidExpectInPromise;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Ensures that promises containing `expect` assertions are properly handled
    /// by being returned or awaited in tests.
    ///
    /// ### Why is this bad?
    ///
    /// When a promise containing `expect` assertions is not returned or awaited,
    /// the test may complete before the assertions run, leading to false positives
    /// where tests pass even when the assertions would fail.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// it('test', () => {
    ///   somePromise.then(value => {
    ///     expect(value).toBe(true);
    ///   });
    /// });
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// it('test', () => {
    ///   return somePromise.then(value => {
    ///     expect(value).toBe(true);
    ///   });
    /// });
    ///
    /// it('test', async () => {
    ///   await somePromise.then(value => {
    ///     expect(value).toBe(true);
    ///   });
    /// });
    /// ```
    ValidExpectInPromise,
    jest,
    correctness
);

impl Rule for ValidExpectInPromise {
    fn run_on_jest_node<'a, 'c>(
        &self,
        possible_jest_node: &PossibleJestNode<'a, 'c>,
        ctx: &'c LintContext<'a>,
    ) {
        run(possible_jest_node, ctx);
    }
}

fn run<'a>(possible_jest_node: &PossibleJestNode<'a, '_>, ctx: &LintContext<'a>) {
    let node = possible_jest_node.node;
    let AstKind::CallExpression(call_expr) = node.kind() else {
        return;
    };

    let Some(jest_fn_call) = parse_general_jest_fn_call(call_expr, possible_jest_node, ctx) else {
        return;
    };

    // Only process test and hook functions
    if !matches!(
        jest_fn_call.kind,
        crate::utils::JestFnKind::General(
            crate::utils::JestGeneralFnKind::Test | crate::utils::JestGeneralFnKind::Hook
        )
    ) {
        return;
    }

    // Get the callback function (last argument or second-to-last if there's a timeout)
    let Some(callback) = get_test_callback(call_expr, &jest_fn_call) else {
        return;
    };

    // Check if callback has a done parameter (skip if it does)
    if has_done_callback(callback, call_expr) {
        return;
    }

    // Get the callback body
    let Some(body) = get_callback_body(callback) else {
        return;
    };

    // Find all promise chains with expect calls in the body
    check_body_for_unhandled_promises(body, ctx);
}

fn get_test_callback<'a>(
    call_expr: &'a CallExpression<'a>,
    _jest_fn_call: &crate::utils::ParsedGeneralJestFnCall,
) -> Option<&'a Expression<'a>> {
    let args = &call_expr.arguments;

    // Find the callback function (last function argument)
    for arg in args.iter().rev() {
        if let Some(expr) = arg.as_expression()
            && is_function_expression(expr)
        {
            return Some(expr);
        }
    }

    None
}

fn is_function_expression(expr: &Expression) -> bool {
    matches!(expr, Expression::FunctionExpression(_) | Expression::ArrowFunctionExpression(_))
}

fn has_done_callback(callback: &Expression, call_expr: &CallExpression) -> bool {
    // Check if this is a .each call - in that case, first param is data row, not done
    let is_jest_each = get_node_name(&call_expr.callee).ends_with("each");

    // For .each, done would be the second parameter (first is data row)
    // For regular test/it, done is the first parameter
    let expected_param_count = if is_jest_each { 2 } else { 1 };

    match callback {
        Expression::FunctionExpression(func) => {
            let param_count = func.params.items.len();
            // Must have exactly the expected number of params, and last one must be identifier
            if param_count != expected_param_count {
                return false;
            }
            // Check if last param (the done callback) is a binding identifier
            func.params.items.last().is_some_and(|p| p.pattern.kind.is_binding_identifier())
        }
        Expression::ArrowFunctionExpression(func) => {
            let params = &func.params.items;
            let param_count = params.len();
            // Must have exactly the expected number of params
            if param_count != expected_param_count {
                return false;
            }
            // Check if last param (the done callback) is a binding identifier
            params.last().is_some_and(|p| p.pattern.kind.is_binding_identifier())
        }
        _ => false,
    }
}

fn get_callback_body<'a>(callback: &'a Expression<'a>) -> Option<CallbackBody<'a>> {
    match callback {
        Expression::FunctionExpression(func) => {
            func.body.as_ref().map(|body| CallbackBody::Block(body.as_ref()))
        }
        Expression::ArrowFunctionExpression(func) => {
            // Arrow functions with expression bodies implicitly return the expression
            // So we don't need to check those - they're always "returned"
            if func.expression {
                Some(CallbackBody::ExpressionBody)
            } else {
                Some(CallbackBody::Block(&func.body))
            }
        }
        _ => None,
    }
}

#[derive(Clone, Copy)]
enum CallbackBody<'a> {
    Block(&'a oxc_ast::ast::FunctionBody<'a>),
    ExpressionBody, // Arrow function with implicit return - no checking needed
}

fn check_body_for_unhandled_promises<'a>(body: CallbackBody<'a>, ctx: &LintContext<'a>) {
    let block = match body {
        CallbackBody::Block(block) => block,
        CallbackBody::ExpressionBody => {
            // Arrow function with implicit return - the expression is returned, so it's valid
            return;
        }
    };

    // Collect variable declarations that hold promises with expects
    let mut promise_vars: Vec<(&str, Span)> = Vec::new();
    // Track which promise vars are properly handled (awaited/returned)
    let mut handled_vars: Vec<&str> = Vec::new();
    // Track if we've hit a return statement (anything after is unreachable)
    let mut has_returned = false;

    for stmt in &block.statements {
        check_statement_for_promises(
            stmt,
            &mut promise_vars,
            &mut handled_vars,
            ctx,
            &mut has_returned,
        );
    }

    // Report any unhandled promise variables
    for (var_name, span) in &promise_vars {
        if !handled_vars.contains(var_name) {
            ctx.diagnostic(valid_expect_in_promise_diagnostic(*span));
        }
    }
}

fn check_statement_for_promises<'a>(
    stmt: &'a Statement<'a>,
    promise_vars: &mut Vec<(&'a str, Span)>,
    handled_vars: &mut Vec<&'a str>,
    ctx: &LintContext<'a>,
    has_returned: &mut bool,
) {
    // After a return statement, any await is unreachable, so don't collect handled_vars
    if *has_returned {
        return;
    }

    match stmt {
        Statement::ExpressionStatement(expr_stmt) => {
            let expr = &expr_stmt.expression;

            // Check for direct promise chain with expect (not assigned, not returned, not awaited)
            if let Some(promise_span) = find_promise_chain_with_expect(expr, ctx) {
                // Check if this is wrapped in await
                if !matches!(expr, Expression::AwaitExpression(_)) {
                    ctx.diagnostic(valid_expect_in_promise_diagnostic(promise_span));
                }
            }

            // Recursively find all awaited identifiers in the expression
            find_awaited_identifiers_in_expression(expr, handled_vars);

            // Check for expect(promise).resolves/rejects pattern
            collect_expect_resolves_rejects_identifiers(expr, handled_vars);

            // Check for assignments that might reassign promise vars
            if let Expression::AssignmentExpression(assign) = expr
                && let Some(name) = get_assignment_target_name(&assign.left)
                && let Some(promise_span) = find_promise_chain_with_expect(&assign.right, ctx)
            {
                // New promise assignment - add to tracking
                promise_vars.push((name, promise_span));
            }
        }
        Statement::ReturnStatement(ret_stmt) => {
            // Mark that we've hit a return - nothing after is reachable
            *has_returned = true;

            // If returning, mark any identifiers as handled
            if let Some(arg) = &ret_stmt.argument {
                collect_awaited_identifiers(arg, handled_vars);

                // Also check for Promise.all/resolve etc.
                if let Expression::CallExpression(call) = arg
                    && is_promise_method_call(call)
                {
                    collect_promise_method_args(call, handled_vars);
                }
            }
        }
        Statement::VariableDeclaration(var_decl) => {
            for decl in &var_decl.declarations {
                check_variable_declarator(decl, promise_vars, ctx);
            }
        }
        Statement::BlockStatement(block) => {
            for inner_stmt in &block.body {
                check_statement_for_promises(
                    inner_stmt,
                    promise_vars,
                    handled_vars,
                    ctx,
                    has_returned,
                );
            }
        }
        _ => {}
    }
}

/// Recursively walks an expression to find all awaited identifiers
fn find_awaited_identifiers_in_expression<'a>(
    expr: &'a Expression<'a>,
    handled: &mut Vec<&'a str>,
) {
    match expr {
        Expression::AwaitExpression(await_expr) => {
            collect_awaited_identifiers(&await_expr.argument, handled);
        }
        Expression::CallExpression(call) => {
            // Check callee
            find_awaited_identifiers_in_expression(&call.callee, handled);
            // Check arguments
            for arg in &call.arguments {
                if let Some(arg_expr) = arg.as_expression() {
                    find_awaited_identifiers_in_expression(arg_expr, handled);
                }
            }
        }
        Expression::StaticMemberExpression(member) => {
            find_awaited_identifiers_in_expression(&member.object, handled);
        }
        Expression::ComputedMemberExpression(member) => {
            find_awaited_identifiers_in_expression(&member.object, handled);
            find_awaited_identifiers_in_expression(&member.expression, handled);
        }
        Expression::ArrayExpression(arr) => {
            for elem in &arr.elements {
                if let ArrayExpressionElement::SpreadElement(spread) = elem {
                    find_awaited_identifiers_in_expression(&spread.argument, handled);
                } else if let Some(expr) = elem.as_expression() {
                    find_awaited_identifiers_in_expression(expr, handled);
                }
            }
        }
        Expression::ObjectExpression(obj) => {
            for prop in &obj.properties {
                if let oxc_ast::ast::ObjectPropertyKind::ObjectProperty(p) = prop {
                    find_awaited_identifiers_in_expression(&p.value, handled);
                }
            }
        }
        Expression::AssignmentExpression(assign) => {
            find_awaited_identifiers_in_expression(&assign.right, handled);
        }
        Expression::SequenceExpression(seq) => {
            for expr in &seq.expressions {
                find_awaited_identifiers_in_expression(expr, handled);
            }
        }
        Expression::ConditionalExpression(cond) => {
            find_awaited_identifiers_in_expression(&cond.consequent, handled);
            find_awaited_identifiers_in_expression(&cond.alternate, handled);
        }
        Expression::BinaryExpression(bin) => {
            find_awaited_identifiers_in_expression(&bin.left, handled);
            find_awaited_identifiers_in_expression(&bin.right, handled);
        }
        Expression::LogicalExpression(log) => {
            find_awaited_identifiers_in_expression(&log.left, handled);
            find_awaited_identifiers_in_expression(&log.right, handled);
        }
        Expression::UnaryExpression(unary) => {
            find_awaited_identifiers_in_expression(&unary.argument, handled);
        }
        Expression::ParenthesizedExpression(paren) => {
            find_awaited_identifiers_in_expression(&paren.expression, handled);
        }
        _ => {}
    }
}

fn check_variable_declarator<'a>(
    decl: &'a VariableDeclarator<'a>,
    promise_vars: &mut Vec<(&'a str, Span)>,
    ctx: &LintContext<'a>,
) {
    // Skip destructuring patterns
    let BindingPatternKind::BindingIdentifier(id) = &decl.id.kind else {
        return;
    };

    let Some(init) = &decl.init else {
        return;
    };

    // If the init is already awaited, the promise is handled
    if matches!(init, Expression::AwaitExpression(_)) {
        return;
    }

    // Check if init is a promise chain with expect
    if let Some(promise_span) = find_promise_chain_with_expect(init, ctx) {
        promise_vars.push((id.name.as_str(), promise_span));
    }
}

fn find_promise_chain_with_expect<'a>(
    expr: &'a Expression<'a>,
    ctx: &LintContext<'a>,
) -> Option<Span> {
    // Unwrap await expression
    let expr = if let Expression::AwaitExpression(await_expr) = expr {
        &await_expr.argument
    } else {
        expr
    };

    // Check if this is a promise chain call (.then/.catch/.finally)
    let Expression::CallExpression(call_expr) = expr else {
        return None;
    };

    if !is_promise_chain_call(call_expr) {
        return None;
    }

    // Check if this promise chain or any nested chain contains expect
    if contains_expect_in_callbacks(call_expr, ctx) {
        return Some(call_expr.span);
    }

    None
}

fn is_promise_chain_call(call_expr: &CallExpression) -> bool {
    // Check for .then(), .catch(), .finally()
    let callee = &call_expr.callee;

    if let Expression::StaticMemberExpression(member) = callee {
        let method_name = member.property.name.as_str();
        return matches!(method_name, "then" | "catch" | "finally");
    }

    if let Expression::ComputedMemberExpression(member) = callee
        && let Expression::StringLiteral(lit) = &member.expression
    {
        return matches!(lit.value.as_str(), "then" | "catch" | "finally");
    }

    false
}

/// Returns the maximum number of callback arguments for a promise method
/// .then() takes 2 (onFulfilled, onRejected), .catch() and .finally() take 1
fn get_max_callback_args(call_expr: &CallExpression) -> usize {
    let callee = &call_expr.callee;

    if let Expression::StaticMemberExpression(member) = callee {
        let method_name = member.property.name.as_str();
        return match method_name {
            "then" => 2,
            "catch" | "finally" => 1,
            _ => 0,
        };
    }

    if let Expression::ComputedMemberExpression(member) = callee
        && let Expression::StringLiteral(lit) = &member.expression
    {
        return match lit.value.as_str() {
            "then" => 2,
            "catch" | "finally" => 1,
            _ => 0,
        };
    }

    0
}

fn contains_expect_in_callbacks<'a>(
    call_expr: &'a CallExpression<'a>,
    ctx: &LintContext<'a>,
) -> bool {
    // Determine how many arguments to check based on the method
    // .then() takes 2 callbacks, .catch() and .finally() take 1
    let max_args = get_max_callback_args(call_expr);

    // Check callbacks in this call (only up to max_args)
    for (i, arg) in call_expr.arguments.iter().enumerate() {
        if i >= max_args {
            break;
        }
        if let Some(expr) = arg.as_expression()
            && callback_contains_expect(expr, ctx)
        {
            return true;
        }
    }

    // Check the chain (the object being called on)
    if let Expression::StaticMemberExpression(member) = &call_expr.callee
        && let Expression::CallExpression(parent_call) = &member.object
        && is_promise_chain_call(parent_call)
        && contains_expect_in_callbacks(parent_call, ctx)
    {
        return true;
    }

    if let Expression::ComputedMemberExpression(member) = &call_expr.callee
        && let Expression::CallExpression(parent_call) = &member.object
        && is_promise_chain_call(parent_call)
        && contains_expect_in_callbacks(parent_call, ctx)
    {
        return true;
    }

    false
}

fn callback_contains_expect<'a>(expr: &'a Expression<'a>, ctx: &LintContext<'a>) -> bool {
    match expr {
        Expression::FunctionExpression(func) => {
            if let Some(body) = &func.body {
                body_contains_expect(body, ctx)
            } else {
                false
            }
        }
        Expression::ArrowFunctionExpression(func) => body_contains_expect(&func.body, ctx),
        _ => false,
    }
}

fn body_contains_expect<'a>(
    body: &'a oxc_ast::ast::FunctionBody<'a>,
    ctx: &LintContext<'a>,
) -> bool {
    for stmt in &body.statements {
        if statement_contains_expect(stmt, ctx) {
            return true;
        }
    }
    false
}

fn statement_contains_expect<'a>(stmt: &'a Statement<'a>, ctx: &LintContext<'a>) -> bool {
    match stmt {
        Statement::ExpressionStatement(expr_stmt) => {
            expression_contains_expect(&expr_stmt.expression, ctx)
        }
        Statement::ReturnStatement(ret) => {
            ret.argument.as_ref().is_some_and(|arg| expression_contains_expect(arg, ctx))
        }
        Statement::BlockStatement(block) => {
            block.body.iter().any(|s| statement_contains_expect(s, ctx))
        }
        Statement::IfStatement(if_stmt) => {
            statement_contains_expect(&if_stmt.consequent, ctx)
                || if_stmt.alternate.as_ref().is_some_and(|alt| statement_contains_expect(alt, ctx))
        }
        Statement::VariableDeclaration(var_decl) => var_decl
            .declarations
            .iter()
            .any(|d| d.init.as_ref().is_some_and(|init| expression_contains_expect(init, ctx))),
        _ => false,
    }
}

fn expression_contains_expect<'a>(expr: &'a Expression<'a>, ctx: &LintContext<'a>) -> bool {
    match expr {
        Expression::CallExpression(call) => {
            // Check if this is an expect call
            if is_expect_call(call, ctx) {
                return true;
            }
            // Check arguments
            call.arguments
                .iter()
                .any(|arg| arg.as_expression().is_some_and(|e| expression_contains_expect(e, ctx)))
        }
        Expression::AwaitExpression(await_expr) => {
            expression_contains_expect(&await_expr.argument, ctx)
        }
        Expression::ChainExpression(chain) => {
            if let oxc_ast::ast::ChainElement::CallExpression(call) = &chain.expression {
                is_expect_call(call, ctx)
            } else {
                false
            }
        }
        _ => false,
    }
}

fn is_expect_call<'a>(call_expr: &'a CallExpression<'a>, ctx: &LintContext<'a>) -> bool {
    // Check if callee is expect(...) or expect.xxx or expect(...).xxx
    let callee = call_expr.callee.get_inner_expression();

    match callee {
        Expression::Identifier(ident) => ident.name == "expect",
        Expression::StaticMemberExpression(member) => {
            // Could be expect.assertions, expect.hasAssertions, or expect().toBe etc.
            expression_is_expect(&member.object, ctx)
        }
        Expression::CallExpression(inner_call) => expression_is_expect(&inner_call.callee, ctx),
        _ => false,
    }
}

fn expression_is_expect<'a>(expr: &'a Expression<'a>, _ctx: &LintContext<'a>) -> bool {
    match expr.get_inner_expression() {
        Expression::Identifier(ident) => ident.name == "expect",
        Expression::CallExpression(call) => {
            matches!(call.callee.get_inner_expression(), Expression::Identifier(ident) if ident.name == "expect")
        }
        Expression::StaticMemberExpression(member) => {
            matches!(member.object.get_inner_expression(), Expression::Identifier(ident) if ident.name == "expect")
        }
        _ => false,
    }
}

fn collect_awaited_identifiers<'a>(expr: &'a Expression<'a>, handled: &mut Vec<&'a str>) {
    match expr {
        Expression::Identifier(ident) => {
            handled.push(ident.name.as_str());
        }
        Expression::AwaitExpression(await_expr) => {
            collect_awaited_identifiers(&await_expr.argument, handled);
        }
        Expression::CallExpression(call) => {
            // Check for Promise.all/resolve etc.
            if is_promise_method_call(call) {
                collect_promise_method_args(call, handled);
            }
        }
        _ => {}
    }
}

fn is_promise_method_call(call: &CallExpression) -> bool {
    if let Expression::StaticMemberExpression(member) = &call.callee
        && let Expression::Identifier(obj) = &member.object
        && obj.name == "Promise"
    {
        let method = member.property.name.as_str();
        return matches!(method, "all" | "allSettled" | "race" | "any" | "resolve" | "reject");
    }
    false
}

fn collect_promise_method_args<'a>(call: &'a CallExpression<'a>, handled: &mut Vec<&'a str>) {
    for arg in &call.arguments {
        if let Some(expr) = arg.as_expression() {
            match expr {
                Expression::Identifier(ident) => {
                    handled.push(ident.name.as_str());
                }
                Expression::ArrayExpression(arr) => {
                    for elem in &arr.elements {
                        if let ArrayExpressionElement::Identifier(ident) = elem {
                            handled.push(ident.name.as_str());
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

fn get_assignment_target_name<'a>(
    target: &'a oxc_ast::ast::AssignmentTarget<'a>,
) -> Option<&'a str> {
    match target {
        oxc_ast::ast::AssignmentTarget::AssignmentTargetIdentifier(ident) => {
            Some(ident.name.as_str())
        }
        _ => None,
    }
}

/// Check for expect(promise).resolves/rejects patterns and collect the promise identifier
fn collect_expect_resolves_rejects_identifiers<'a>(
    expr: &'a Expression<'a>,
    handled: &mut Vec<&'a str>,
) {
    // Walk up call expression chains like expect(x).resolves.toBe(1)
    let mut current = expr;

    loop {
        match current {
            Expression::CallExpression(call) => {
                // Check if this is an expect().resolves/rejects chain
                if let Some(ident) = find_expect_resolves_rejects_arg(call) {
                    handled.push(ident);
                    return;
                }
                // Continue walking up the chain
                match &call.callee {
                    Expression::StaticMemberExpression(member) => {
                        current = &member.object;
                    }
                    _ => return,
                }
            }
            _ => return,
        }
    }
}

/// Find identifier passed to expect() if it has resolves/rejects modifier
fn find_expect_resolves_rejects_arg<'a>(call: &'a CallExpression<'a>) -> Option<&'a str> {
    // Pattern: expect(promise).resolves.toBe() or expect(promise).rejects.toBe()
    // We need to find the expect() call and check if there's a resolves/rejects member

    let mut current = &call.callee;
    let mut has_resolves_or_rejects = false;

    // Walk up the member expression chain
    loop {
        match current.get_inner_expression() {
            Expression::StaticMemberExpression(member) => {
                let prop_name = member.property.name.as_str();
                if matches!(prop_name, "resolves" | "rejects") {
                    has_resolves_or_rejects = true;
                }
                current = &member.object;
            }
            Expression::CallExpression(inner_call) => {
                // Check if this is expect()
                if let Expression::Identifier(ident) = inner_call.callee.get_inner_expression()
                    && ident.name == "expect"
                    && has_resolves_or_rejects
                    && let Some(arg) = inner_call.arguments.first()
                {
                    // Found expect().resolves/rejects - get the argument
                    if let Some(Expression::Identifier(arg_ident)) = arg.as_expression() {
                        return Some(arg_ident.name.as_str());
                    }
                    // Also check for await promise
                    if let Some(Expression::AwaitExpression(await_expr)) = arg.as_expression()
                        && let Expression::Identifier(arg_ident) = &await_expr.argument
                    {
                        return Some(arg_ident.name.as_str());
                    }
                }
                return None;
            }
            _ => return None,
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("test('something', () => Promise.resolve().then(() => expect(1).toBe(2)));", None, None),
        ("Promise.resolve().then(() => expect(1).toBe(2))", None, None),
        ("const x = Promise.resolve().then(() => expect(1).toBe(2))", None, None),
        (
            "it('is valid', () => {
			  const promise = loadNumber().then(number => {
			    expect(typeof number).toBe('number');
			    return number + 1;
			  });
			  expect(promise).resolves.toBe(1);
			});",
            None,
            None,
        ),
        (
            "it('is valid', () => {
			  const promise = loadNumber().then(number => {
			    expect(typeof number).toBe('number');
			    return number + 1;
			  });
			  expect(promise).resolves.not.toBe(2);
			});",
            None,
            None,
        ),
        (
            "it('is valid', () => {
			  const promise = loadNumber().then(number => {
			    expect(typeof number).toBe('number');
			    return number + 1;
			  });
			  expect(promise).rejects.toBe(1);
			});",
            None,
            None,
        ),
        (
            "it('is valid', () => {
			  const promise = loadNumber().then(number => {
			    expect(typeof number).toBe('number');
			    return number + 1;
			  });
			  expect(promise).rejects.not.toBe(2);
			});",
            None,
            None,
        ),
        (
            "it('is valid', async () => {
			  const promise = loadNumber().then(number => {
			    expect(typeof number).toBe('number');
			    return number + 1;
			  });
			  expect(await promise).toBeGreaterThan(1);
			});",
            None,
            None,
        ),
        (
            "it('is valid', async () => {
			  const promise = loadNumber().then(number => {
			    expect(typeof number).toBe('number');
			    return number + 1;
			  });
			  expect(await promise).resolves.toBeGreaterThan(1);
			});",
            None,
            None,
        ),
        (
            "it('is valid', async () => {
			  const promise = loadNumber().then(number => {
			    expect(typeof number).toBe('number');
			    return number + 1;
			  });
			  expect(1).toBeGreaterThan(await promise);
			});",
            None,
            None,
        ),
        (
            "it('is valid', async () => {
			  const promise = loadNumber().then(number => {
			    expect(typeof number).toBe('number');
			    return number + 1;
			  });
			  expect.this.that.is(await promise);
			});",
            None,
            None,
        ),
        (
            "it('is valid', async () => {
			  expect(await loadNumber().then(number => {
			    expect(typeof number).toBe('number');
			    return number + 1;
			  })).toBeGreaterThan(1);
			});",
            None,
            None,
        ),
        (
            "it('is valid', async () => {
			  const promise = loadNumber().then(number => {
			    expect(typeof number).toBe('number');
			    return number + 1;
			  });
			  expect([await promise]).toHaveLength(1);
			});",
            None,
            None,
        ),
        (
            "it('is valid', async () => {
			  const promise = loadNumber().then(number => {
			    expect(typeof number).toBe('number');
			    return number + 1;
			  });
			  expect([,,await promise,,]).toHaveLength(1);
			});",
            None,
            None,
        ),
        (
            "it('is valid', async () => {
			  const promise = loadNumber().then(number => {
			    expect(typeof number).toBe('number');
			    return number + 1;
			  });
			  expect([[await promise]]).toHaveLength(1);
			});",
            None,
            None,
        ),
        (
            "it('is valid', async () => {
			  const promise = loadNumber().then(number => {
			    expect(typeof number).toBe('number');
			    return number + 1;
			  });
			  logValue(await promise);
			});",
            None,
            None,
        ),
        (
            "it('is valid', async () => {
			  const promise = loadNumber().then(number => {
			    expect(typeof number).toBe('number');
			    return 1;
			  });
			  expect.assertions(await promise);
			});",
            None,
            None,
        ),
        (
            "it('is valid', async () => {
			  await loadNumber().then(number => {
			    expect(typeof number).toBe('number');
			  });
			});",
            None,
            None,
        ),
        (
            "it('it1', () => new Promise((done) => {
			  test()
			    .then(() => {
			      expect(someThing).toEqual(true);
			      done();
			    });
			}));",
            None,
            None,
        ),
        (
            "it('it1', () => {
			  return new Promise(done => {
			    test().then(() => {
			      expect(someThing).toEqual(true);
			      done();
			    });
			  });
			});",
            None,
            None,
        ),
        (
            "it('passes', () => {
			  Promise.resolve().then(() => {
			    grabber.grabSomething();
			  });
			});",
            None,
            None,
        ),
        (
            "it('passes', async () => {
			  const grabbing = Promise.resolve().then(() => {
			    grabber.grabSomething();
			  });
			  await grabbing;
			  expect(grabber.grabbedItems).toHaveLength(1);
			});",
            None,
            None,
        ),
        (
            "const myFn = () => {
			  Promise.resolve().then(() => {
			    expect(true).toBe(false);
			  });
			};",
            None,
            None,
        ),
        (
            "const myFn = () => {
			  Promise.resolve().then(() => {
			    subject.invokeMethod();
			  });
			};",
            None,
            None,
        ),
        (
            "const myFn = () => {
			  Promise.resolve().then(() => {
			    expect(true).toBe(false);
			  });
			};
			it('it1', () => {
			  return somePromise.then(() => {
			    expect(someThing).toEqual(true);
			  });
			});",
            None,
            None,
        ),
        (
            "it('it1', () => new Promise((done) => {
			  test()
			    .finally(() => {
			      expect(someThing).toEqual(true);
			      done();
			    });
			}));",
            None,
            None,
        ),
        (
            "it('it1', () => {
			  return somePromise.then(() => {
			    expect(someThing).toEqual(true);
			  });
			});",
            None,
            None,
        ),
        (
            "it('it1', () => {
			  return somePromise.finally(() => {
			    expect(someThing).toEqual(true);
			  });
			});",
            None,
            None,
        ),
        (
            "it('it1', function() {
			  return somePromise.catch(function() {
			    expect(someThing).toEqual(true);
			  });
			});",
            None,
            None,
        ),
        (
            "xtest('it1', function() {
			  return somePromise.catch(function() {
			    expect(someThing).toEqual(true);
			  });
			});",
            None,
            None,
        ),
        (
            "it('it1', function() {
			  return somePromise.then(function() {
			    doSomeThingButNotExpect();
			  });
			});",
            None,
            None,
        ),
        (
            "it('it1', function() {
			  return getSomeThing().getPromise().then(function() {
			    expect(someThing).toEqual(true);
			  });
			});",
            None,
            None,
        ),
        (
            "it('it1', function() {
			  return Promise.resolve().then(function() {
			    expect(someThing).toEqual(true);
			  });
			});",
            None,
            None,
        ),
        (
            "it('it1', function () {
			  return Promise.resolve().then(function () {
			    /*fulfillment*/
			    expect(someThing).toEqual(true);
			  }, function () {
			    /*rejection*/
			    expect(someThing).toEqual(true);
			  });
			});",
            None,
            None,
        ),
        (
            "it('it1', function () {
			  Promise.resolve().then(/*fulfillment*/ function () {
			  }, undefined, /*rejection*/ function () {
			    expect(someThing).toEqual(true)
			  })
			});",
            None,
            None,
        ),
        (
            "it('it1', function () {
			  return Promise.resolve().then(function () {
			    /*fulfillment*/
			  }, function () {
			    /*rejection*/
			    expect(someThing).toEqual(true);
			  });
			});",
            None,
            None,
        ),
        (
            "it('it1', function () {
			  return somePromise.then()
			});",
            None,
            None,
        ),
        (
            "it('it1', async () => {
			  await Promise.resolve().then(function () {
			    expect(someThing).toEqual(true)
			  });
			});",
            None,
            None,
        ),
        (
            "it('it1', async () => {
			  await somePromise.then(() => {
			    expect(someThing).toEqual(true)
			  });
			});",
            None,
            None,
        ),
        (
            "it('it1', async () => {
			  await getSomeThing().getPromise().then(function () {
			    expect(someThing).toEqual(true)
			  });
			});",
            None,
            None,
        ),
        (
            "it('it1', () => {
			  return somePromise.then(() => {
			    expect(someThing).toEqual(true);
			  })
			  .then(() => {
			    expect(someThing).toEqual(true);
			  })
			});",
            None,
            None,
        ),
        (
            "it('it1', () => {
			  return somePromise.then(() => {
			    return value;
			  })
			  .then(value => {
			    expect(someThing).toEqual(value);
			  })
			});",
            None,
            None,
        ),
        (
            "it('it1', () => {
			  return somePromise.then(() => {
			    expect(someThing).toEqual(true);
			  })
			  .then(() => {
			    console.log('this is silly');
			  })
			});",
            None,
            None,
        ),
        (
            "it('it1', () => {
			  return somePromise.then(() => {
			    expect(someThing).toEqual(true);
			  })
			  .catch(() => {
			    expect(someThing).toEqual(false);
			  })
			});",
            None,
            None,
        ),
        (
            "test('later return', () => {
			  const promise = something().then(value => {
			    expect(value).toBe('red');
			  });
			  return promise;
			});",
            None,
            None,
        ),
        (
            "test('later return', async () => {
			  const promise = something().then(value => {
			    expect(value).toBe('red');
			  });
			  await promise;
			});",
            None,
            None,
        ),
        (
            "test.only('later return', () => {
			  const promise = something().then(value => {
			    expect(value).toBe('red');
			  });
			  return promise;
			});",
            None,
            None,
        ),
        (
            "test('that we bailout if destructuring is used', () => {
			  const [promise] = something().then(value => {
			    expect(value).toBe('red');
			  });
			});",
            None,
            None,
        ),
        (
            "test('that we bailout if destructuring is used', async () => {
			  const [promise] = await something().then(value => {
			    expect(value).toBe('red');
			  });
			});",
            None,
            None,
        ),
        (
            "test('that we bailout if destructuring is used', () => {
			  const [promise] = [
			    something().then(value => {
			      expect(value).toBe('red');
			    })
			  ];
			});",
            None,
            None,
        ),
        (
            "test('that we bailout if destructuring is used', () => {
			  const {promise} = {
			    promise: something().then(value => {
			      expect(value).toBe('red');
			    })
			  };
			});",
            None,
            None,
        ),
        (
            "test('that we bailout in complex cases', () => {
			  promiseSomething({
			    timeout: 500,
			    promise: something().then(value => {
			      expect(value).toBe('red');
			    })
			  });
			});",
            None,
            None,
        ),
        (
            "it('shorthand arrow', () =>
			  something().then(value => {
			    expect(() => {
			      value();
			    }).toThrow();
			  })
			);",
            None,
            None,
        ),
        (
            "it('crawls for files based on patterns', () => {
			  const promise = nodeCrawl({}).then(data => {
			    expect(childProcess.spawn).lastCalledWith('find');
			  });
			  return promise;
			});",
            None,
            None,
        ),
        (
            "it('is a test', async () => {
			  const value = await somePromise().then(response => {
			    expect(response).toHaveProperty('data');
			    return response.data;
			  });
			  expect(value).toBe('hello world');
			});",
            None,
            None,
        ),
        (
            "it('is a test', async () => {
			  return await somePromise().then(response => {
			    expect(response).toHaveProperty('data');
			    return response.data;
			  });
			});",
            None,
            None,
        ),
        (
            "it('is a test', async () => {
			  return somePromise().then(response => {
			    expect(response).toHaveProperty('data');
			    return response.data;
			  });
			});",
            None,
            None,
        ),
        (
            "it('is a test', async () => {
			  await somePromise().then(response => {
			    expect(response).toHaveProperty('data');
			    return response.data;
			  });
			});",
            None,
            None,
        ),
        (
            "it(
			  'test function',
			  () => {
			    return Builder
			      .getPromiseBuilder()
			      .get().build()
			      .then((data) => {
			        expect(data).toEqual('Hi');
			      });
			  }
			);",
            None,
            None,
        ),
        (
            "notATestFunction(
			  'not a test function',
			  () => {
			    Builder
			      .getPromiseBuilder()
			      .get()
			      .build()
			      .then((data) => {
			        expect(data).toEqual('Hi');
			      });
			  }
			);",
            None,
            None,
        ),
        (
            "it('is valid', async () => {
			  const promiseOne = loadNumber().then(number => {
			    expect(typeof number).toBe('number');
			  });
			  const promiseTwo = loadNumber().then(number => {
			    expect(typeof number).toBe('number');
			  });
			  await promiseTwo;
			  await promiseOne;
			});",
            None,
            None,
        ),
        (
            r#"it("it1", () => somePromise.then(() => {
			  expect(someThing).toEqual(true)
			}))"#,
            None,
            None,
        ),
        (r#"it("it1", () => somePromise.then(() => expect(someThing).toEqual(true)))"#, None, None),
        (
            "it('promise test with done', (done) => {
			  const promise = getPromise();
			  promise.then(() => expect(someThing).toEqual(true));
			});",
            None,
            None,
        ),
        (
            "it('name of done param does not matter', (nameDoesNotMatter) => {
			  const promise = getPromise();
			  promise.then(() => expect(someThing).toEqual(true));
			});",
            None,
            None,
        ),
        // NOTE: it.each([])(...) with only one param is NOT a done callback - first param is data row
        // So this case is now correctly moved to fail array below
        (
            "it.each``('name of done param does not matter', ({}, nameDoesNotMatter) => {
			  const promise = getPromise();
			  promise.then(() => expect(someThing).toEqual(true));
			});",
            None,
            None,
        ),
        (
            "test('valid-expect-in-promise', async () => {
			  const text = await fetch('url')
			      .then(res => res.text())
			      .then(text => text);
			  expect(text).toBe('text');
			});",
            None,
            None,
        ),
        (
            "test('promise test', async function () {
			  let somePromise = getPromise().then((data) => {
			    expect(data).toEqual('foo');
			  }), x = 1;
			  await somePromise;
			});",
            None,
            None,
        ),
        (
            "test('promise test', async function () {
			  let x = 1, somePromise = getPromise().then((data) => {
			    expect(data).toEqual('foo');
			  });
			  await somePromise;
			});",
            None,
            None,
        ),
        (
            "test('promise test', async function () {
			  let somePromise = getPromise().then((data) => {
			    expect(data).toEqual('foo');
			  });
			  await somePromise;
			  somePromise = getPromise().then((data) => {
			    expect(data).toEqual('foo');
			  });
			  await somePromise;
			});",
            None,
            None,
        ),
        (
            "test('promise test', async function () {
			  let somePromise = getPromise().then((data) => {
			    expect(data).toEqual('foo');
			  });
			  await somePromise;
			  somePromise = getPromise().then((data) => {
			    expect(data).toEqual('foo');
			  });
			  return somePromise;
			});",
            None,
            None,
        ),
        (
            "test('promise test', async function () {
			  let somePromise = getPromise().then((data) => {
			    expect(data).toEqual('foo');
			  });
			  {}
			  await somePromise;
			});",
            None,
            None,
        ),
        (
            "test('promise test', async function () {
			  const somePromise = getPromise().then((data) => {
			    expect(data).toEqual('foo');
			  });
			  {
			    await somePromise;
			  }
			});",
            None,
            None,
        ),
        (
            "test('promise test', async function () {
			  let somePromise = getPromise().then((data) => {
			    expect(data).toEqual('foo');
			  });
			  {
			    await somePromise;
			    somePromise = getPromise().then((data) => {
			      expect(data).toEqual('foo');
			    });
			    await somePromise;
			  }
			});",
            None,
            None,
        ),
        (
            "test('promise test', async function () {
			  let somePromise = getPromise().then((data) => {
			    expect(data).toEqual('foo');
			  });
			  await somePromise;
			  {
			    somePromise = getPromise().then((data) => {
			      expect(data).toEqual('foo');
			    });
			    await somePromise;
			  }
			});",
            None,
            None,
        ),
        (
            "test('promise test', async function () {
			  let somePromise = getPromise().then((data) => {
			    expect(data).toEqual('foo');
			  });
			  somePromise = somePromise.then((data) => {
			    expect(data).toEqual('foo');
			  });
			  await somePromise;
			});",
            None,
            None,
        ),
        (
            "test('promise test', async function () {
			  let somePromise = getPromise().then((data) => {
			    expect(data).toEqual('foo');
			  });
			  somePromise = somePromise
			    .then((data) => data)
			    .then((data) => data)
			    .then((data) => {
			      expect(data).toEqual('foo');
			    });
			  await somePromise;
			});",
            None,
            None,
        ),
        (
            "test('promise test', async function () {
			  let somePromise = getPromise().then((data) => {
			    expect(data).toEqual('foo');
			  });
			  somePromise = somePromise
			    .then((data) => data)
			    .then((data) => data)
			  await somePromise;
			});",
            None,
            None,
        ),
        (
            "test('promise test', async function () {
			  let somePromise = getPromise().then((data) => {
			    expect(data).toEqual('foo');
			  });
			  await somePromise;
			  {
			    somePromise = getPromise().then((data) => {
			      expect(data).toEqual('foo');
			    });
			    {
			      await somePromise;
			    }
			  }
			});",
            None,
            None,
        ),
        (
            "test('promise test', async function () {
			  const somePromise = getPromise().then((data) => {
			    expect(data).toEqual('foo');
			  });
			  await Promise.all([somePromise]);
			});",
            None,
            None,
        ),
        (
            "test('promise test', async function () {
			  const somePromise = getPromise().then((data) => {
			    expect(data).toEqual('foo');
			  });
			  return Promise.all([somePromise]);
			});",
            None,
            None,
        ),
        (
            "test('promise test', async function () {
			  const somePromise = getPromise().then((data) => {
			    expect(data).toEqual('foo');
			  });
			  return Promise.resolve(somePromise);
			});",
            None,
            None,
        ),
        (
            "test('promise test', async function () {
			  const somePromise = getPromise().then((data) => {
			    expect(data).toEqual('foo');
			  });
			  return Promise.reject(somePromise);
			});",
            None,
            None,
        ),
        (
            "test('promise test', async function () {
			  const somePromise = getPromise().then((data) => {
			    expect(data).toEqual('foo');
			  });
			  await Promise.resolve(somePromise);
			});",
            None,
            None,
        ),
        (
            "test('promise test', async function () {
			  const somePromise = getPromise().then((data) => {
			    expect(data).toEqual('foo');
			  });
			  await Promise.reject(somePromise);
			});",
            None,
            None,
        ),
        (
            "test('later return', async () => {
			  const onePromise = something().then(value => {
			    console.log(value);
			  });
			  const twoPromise = something().then(value => {
			    expect(value).toBe('red');
			  });
			  return Promise.all([onePromise, twoPromise]);
			});",
            None,
            None,
        ),
        (
            "test('later return', async () => {
			  const onePromise = something().then(value => {
			    console.log(value);
			  });
			  const twoPromise = something().then(value => {
			    expect(value).toBe('red');
			  });
			  return Promise.allSettled([onePromise, twoPromise]);
			});",
            None,
            None,
        ),
    ];

    let fail = vec![
        (
            "const myFn = () => {
			  Promise.resolve().then(() => {
			    expect(true).toBe(false);
			  });
			};
			it('it1', () => {
			  somePromise.then(() => {
			    expect(someThing).toEqual(true);
			  });
			});",
            None,
            None,
        ),
        (
            "it('it1', () => {
			  somePromise.then(() => {
			    expect(someThing).toEqual(true);
			  });
			});",
            None,
            None,
        ),
        (
            "it('it1', () => {
			  somePromise.finally(() => {
			    expect(someThing).toEqual(true);
			  });
			});",
            None,
            None,
        ),
        (
            "
			       it('it1', () => {
			         somePromise['then'](() => {
			           expect(someThing).toEqual(true);
			         });
			       });
			      ",
            None,
            None,
        ),
        (
            "it('it1', function() {
			  getSomeThing().getPromise().then(function() {
			    expect(someThing).toEqual(true);
			  });
			});",
            None,
            None,
        ),
        (
            "it('it1', function() {
			  Promise.resolve().then(function() {
			    expect(someThing).toEqual(true);
			  });
			});",
            None,
            None,
        ),
        (
            "it('it1', function() {
			  somePromise.catch(function() {
			    expect(someThing).toEqual(true)
			  })
			})",
            None,
            None,
        ),
        (
            "xtest('it1', function() {
			  somePromise.catch(function() {
			    expect(someThing).toEqual(true)
			  })
			})",
            None,
            None,
        ),
        (
            "it('it1', function() {
			  somePromise.then(function() {
			    expect(someThing).toEqual(true)
			  })
			})",
            None,
            None,
        ),
        (
            "it('it1', function () {
			  Promise.resolve().then(/*fulfillment*/ function () {
			    expect(someThing).toEqual(true);
			  }, /*rejection*/ function () {
			    expect(someThing).toEqual(true);
			  })
			})",
            None,
            None,
        ),
        (
            "it('it1', function () {
			  Promise.resolve().then(/*fulfillment*/ function () {
			  }, /*rejection*/ function () {
			    expect(someThing).toEqual(true)
			  })
			});",
            None,
            None,
        ),
        (
            "it('test function', () => {
			  Builder.getPromiseBuilder()
			    .get()
			    .build()
			    .then(data => expect(data).toEqual('Hi'));
			});",
            None,
            None,
        ),
        (
            "
			        it('test function', async () => {
			          Builder.getPromiseBuilder()
			            .get()
			            .build()
			            .then(data => expect(data).toEqual('Hi'));
			        });
			      ",
            None,
            None,
        ),
        (
            "it('it1', () => {
			  somePromise.then(() => {
			    doSomeOperation();
			    expect(someThing).toEqual(true);
			  })
			});",
            None,
            None,
        ),
        (
            "it('is a test', () => {
			  somePromise
			    .then(() => {})
			    .then(() => expect(someThing).toEqual(value))
			});",
            None,
            None,
        ),
        (
            "it('is a test', () => {
			  somePromise
			    .then(() => expect(someThing).toEqual(value))
			    .then(() => {})
			});",
            None,
            None,
        ),
        (
            "it('is a test', () => {
			  somePromise.then(() => {
			    return value;
			  })
			  .then(value => {
			    expect(someThing).toEqual(value);
			  })
			});",
            None,
            None,
        ),
        (
            "it('is a test', () => {
			  somePromise.then(() => {
			    expect(someThing).toEqual(true);
			  })
			  .then(() => {
			    console.log('this is silly');
			  })
			});",
            None,
            None,
        ),
        (
            "it('is a test', () => {
			  somePromise.then(() => {
			    // return value;
			  })
			  .then(value => {
			    expect(someThing).toEqual(value);
			  })
			});",
            None,
            None,
        ),
        (
            "it('is a test', () => {
			  somePromise.then(() => {
			    return value;
			  })
			  .then(value => {
			    expect(someThing).toEqual(value);
			  })
			  return anotherPromise.then(() => expect(x).toBe(y));
			});",
            None,
            None,
        ),
        (
            "it('is a test', () => {
			  somePromise
			    .then(() => 1)
			    .then(x => x + 1)
			    .catch(() => -1)
			    .then(v => expect(v).toBe(2));
			  return anotherPromise.then(() => expect(x).toBe(y));
			});",
            None,
            None,
        ),
        (
            "it('is a test', () => {
			  somePromise
			    .then(() => 1)
			    .then(v => expect(v).toBe(2))
			    .then(x => x + 1)
			    .catch(() => -1);
			  return anotherPromise.then(() => expect(x).toBe(y));
			});",
            None,
            None,
        ),
        (
            "it('it1', () => {
			  somePromise.finally(() => {
			    doSomeOperation();
			    expect(someThing).toEqual(true);
			  })
			});",
            None,
            None,
        ),
        (
            r#"test('invalid return', () => {
			  const promise = something().then(value => {
			    const foo = "foo";
			    return expect(value).toBe('red');
			  });
			});"#,
            None,
            None,
        ),
        (
            "fit('it1', () => {
			  somePromise.then(() => {
			    doSomeOperation();
			    expect(someThing).toEqual(true);
			  })
			});",
            None,
            None,
        ),
        (
            "it.skip('it1', () => {
			  somePromise.then(() => {
			    doSomeOperation();
			    expect(someThing).toEqual(true);
			  })
			});",
            None,
            None,
        ),
        (
            "test('later return', async () => {
			  const promise = something().then(value => {
			    expect(value).toBe('red');
			  });
			  promise;
			});",
            None,
            None,
        ),
        (
            "test('later return', async () => {
			  const promise = something().then(value => {
			    expect(value).toBe('red');
			  });
			  return;
			  await promise;
			});",
            None,
            None,
        ),
        (
            "test('later return', async () => {
			  const promise = something().then(value => {
			    expect(value).toBe('red');
			  });
			  return 1;
			  await promise;
			});",
            None,
            None,
        ),
        (
            "test('later return', async () => {
			  const promise = something().then(value => {
			    expect(value).toBe('red');
			  });
			  return [];
			  await promise;
			});",
            None,
            None,
        ),
        (
            "test('later return', async () => {
			  const promise = something().then(value => {
			    expect(value).toBe('red');
			  });
			  return Promise.all([anotherPromise]);
			  await promise;
			});",
            None,
            None,
        ),
        (
            "test('later return', async () => {
			  const promise = something().then(value => {
			    expect(value).toBe('red');
			  });
			  return {};
			  await promise;
			});",
            None,
            None,
        ),
        (
            "test('later return', async () => {
			  const promise = something().then(value => {
			    expect(value).toBe('red');
			  });
			  return Promise.all([]);
			  await promise;
			});",
            None,
            None,
        ),
        (
            "test('later return', async () => {
			  const promise = something().then(value => {
			    expect(value).toBe('red');
			  });
			  await 1;
			});",
            None,
            None,
        ),
        (
            "test('later return', async () => {
			  const promise = something().then(value => {
			    expect(value).toBe('red');
			  });
			  await [];
			});",
            None,
            None,
        ),
        (
            "test('later return', async () => {
			  const promise = something().then(value => {
			    expect(value).toBe('red');
			  });
			  await Promise.all([anotherPromise]);
			});",
            None,
            None,
        ),
        (
            "test('later return', async () => {
			  const promise = something().then(value => {
			    expect(value).toBe('red');
			  });
			  await {};
			});",
            None,
            None,
        ),
        (
            "test('later return', async () => {
			  const promise = something().then(value => {
			    expect(value).toBe('red');
			  });
			  await Promise.all([]);
			});",
            None,
            None,
        ),
        (
            "test('later return', async () => {
			  const promise = something().then(value => {
			    expect(value).toBe('red');
			  }), x = 1;
			});",
            None,
            None,
        ),
        (
            "test('later return', async () => {
			  const x = 1, promise = something().then(value => {
			    expect(value).toBe('red');
			  });
			});",
            None,
            None,
        ),
        (
            "import { test } from '@jest/globals';
			test('later return', async () => {
			  const x = 1, promise = something().then(value => {
			    expect(value).toBe('red');
			  });
			});",
            None,
            None,
        ),
        (
            "it('promise test', () => {
			  const somePromise = getThatPromise();
			  somePromise.then((data) => {
			    expect(data).toEqual('foo');
			  });
			  expect(somePromise).toBeDefined();
			  return somePromise;
			});",
            None,
            None,
        ),
        (
            "test('promise test', function () {
			  let somePromise = getThatPromise();
			  somePromise.then((data) => {
			    expect(data).toEqual('foo');
			  });
			  expect(somePromise).toBeDefined();
			  return somePromise;
			});",
            None,
            None,
        ),
        // TODO: The following edge cases are not currently detected and may be added in future:
        // - Variable reassignment before await (somePromise = null; await somePromise;)
        // - Reassignment to new promise before awaiting original
        // - Destructuring assignment to existing variables ([promise] = ...)
        // - Destructuring object assignment ({ somePromise } = {})
        (
            "test('that we error on this', () => {
			  const promise = something().then(value => {
			    expect(value).toBe('red');
			  });
			  log(promise);
			});",
            None,
            None,
        ),
        (
            "it('is valid', async () => {
			  const promise = loadNumber().then(number => {
			    expect(typeof number).toBe('number');
			    return number + 1;
			  });
			  expect(promise).toBeInstanceOf(Promise);
			});",
            None,
            None,
        ),
        (
            "it('is valid', async () => {
			  const promise = loadNumber().then(number => {
			    expect(typeof number).toBe('number');
			    return number + 1;
			  });
			  expect(anotherPromise).resolves.toBe(1);
			});",
            None,
            None,
        ),
        (
            "import { it as promiseThatThis } from '@jest/globals';
			promiseThatThis('is valid', async () => {
			  const promise = loadNumber().then(number => {
			    expect(typeof number).toBe('number');
			    return number + 1;
			  });
			  expect(anotherPromise).resolves.toBe(1);
			});",
            None,
            None,
        ),
        // it.each([]) with single param - first param is data row, not done callback
        (
            "it.each([])('name of done param does not matter', (nameDoesNotMatter) => {
			  const promise = getPromise();
			  promise.then(() => expect(someThing).toEqual(true));
			});",
            None,
            None,
        ),
        // NOTE: The following case requires globalAliases configuration support which is a
        // more advanced feature. Skipping for now:
        // - promiseThatThis('is valid', ...) with globalAliases: { xit: ["promiseThatThis"] }
    ];

    Tester::new(ValidExpectInPromise::NAME, ValidExpectInPromise::PLUGIN, pass, fail)
        .with_jest_plugin(true)
        .test_and_snapshot();
}
