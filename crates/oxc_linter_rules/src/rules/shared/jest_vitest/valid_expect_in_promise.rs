use oxc_ast::{
    AstKind,
    ast::{
        Argument, CallExpression, Expression, FunctionBody, MemberExpression,
        SimpleAssignmentTarget, Statement,
    },
};
use oxc_ast_visit::{Visit, walk};
use oxc_diagnostics::OxcDiagnostic;
use oxc_span::{GetSpan, Span};
use oxc_str::CompactStr;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    context::LintContext,
    utils::{JestGeneralFnKind, PossibleJestNode, get_node_name_vec, parse_general_jest_fn_call},
};

fn expect_in_unhandled_promise(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Expect in a promise chain must be awaited or returned")
        .with_help("Either `await` the promise, `return` it, or use `expect().resolves`/`expect().rejects`.")
        .with_label(span)
}

fn expect_in_promise_after_return(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Expect in a promise chain is unreachable after a `return` statement")
        .with_help("Move the promise before the `return` and ensure it is awaited or returned.")
        .with_label(span)
}

pub const DOCUMENTATION: &str = r"### What it does

Ensures that `expect` calls inside promise chains (`.then()`, `.catch()`,
`.finally()`) are properly awaited or returned from the test.

### Why is this bad?

When `expect` is called inside a promise callback that is not awaited or
returned, the test may pass even if the assertion fails because the test
completes before the promise resolves. This leads to silently passing
tests with broken assertions.

### Examples

Examples of **incorrect** code for this rule:
```javascript
test('promise test', async () => {
  something().then((value) => {
    expect(value).toBe('red')
  })
})

test('promises test', () => {
  const onePromise = something().then((value) => {
    expect(value).toBe('red')
  })
  const twoPromise = something().then((value) => {
    expect(value).toBe('blue')
  })

  return Promise.any([onePromise, twoPromise])
})
```

Examples of **correct** code for this rule:
```javascript
test('promise test', async () => {
  await something().then((value) => {
    expect(value).toBe('red')
  })
})

test('promises test', () => {
  const onePromise = something().then((value) => {
    expect(value).toBe('red')
  })
  const twoPromise = something().then((value) => {
    expect(value).toBe('blue')
  })

  return Promise.all([onePromise, twoPromise])
})
```
";

pub fn run<'a>(possible_jest_node: &PossibleJestNode<'a, '_>, ctx: &LintContext<'a>) {
    let node = possible_jest_node.node;

    let AstKind::CallExpression(call_expr) = node.kind() else {
        return;
    };

    if call_expr.arguments.len() < 2 {
        return;
    }

    let Some(parsed_jest_fn) = parse_general_jest_fn_call(call_expr, possible_jest_node, ctx)
    else {
        return;
    };

    let is_test_block = parsed_jest_fn
        .kind
        .to_general()
        .is_some_and(|test_kind| matches!(test_kind, JestGeneralFnKind::Test));

    if !is_test_block {
        return;
    }

    let Some(callback) = call_expr.arguments.get(1) else {
        return;
    };

    let Some(callback_body) = get_checkable_callback_body(callback) else {
        return;
    };

    let mut pending_promises: FxHashMap<CompactStr, Span> = FxHashMap::default();
    let mut return_found = false;

    process_statements(&callback_body.statements, &mut pending_promises, &mut return_found, ctx);

    for &span in pending_promises.values() {
        ctx.diagnostic(expect_in_unhandled_promise(span));
    }
}

fn process_statements<'a>(
    statements: &'a oxc_allocator::Vec<'a, Statement<'a>>,
    pending_promises: &mut FxHashMap<CompactStr, Span>,
    return_found: &mut bool,
    ctx: &LintContext<'a>,
) {
    for statement in statements {
        let mut scanner = PromiseExpectScanner::new();
        scanner.visit_statement(statement);

        // After a return, any statement with expect-in-promise is unreachable.
        if *return_found {
            if scanner.found_expect_in_promise {
                ctx.diagnostic(expect_in_promise_after_return(statement.span()));
            }
            continue;
        }

        let is_assignment = matches!(
            statement,
            Statement::ExpressionStatement(e)
                if matches!(e.expression, Expression::AssignmentExpression(_))
        );
        let is_block = matches!(statement, Statement::BlockStatement(_));

        // Assignments handle resolution manually (reassignment may invalidate old promises).
        // Blocks recurse into `process_statements` instead.
        if !is_assignment && !is_block {
            resolve_pending_promises(pending_promises, &scanner.resolved_names);
        }

        match statement {
            Statement::VariableDeclaration(decl) => {
                for declarator in &decl.declarations {
                    let Some(init) = &declarator.init else { continue };
                    let Some(ident) = declarator.id.get_binding_identifier() else { continue };
                    // Per-declarator scan: `let x = 1, promise = getPromise().then(...)` —
                    // only track the declarator that actually has expect-in-promise.
                    let mut init_scanner = PromiseExpectScanner::new();
                    init_scanner.visit_expression(init);
                    if init_scanner.found_expect_in_promise {
                        pending_promises
                            .insert(CompactStr::from(ident.name.as_str()), declarator.span);
                    }
                }
            }
            Statement::ExpressionStatement(expr_stmt) => {
                if let Expression::AssignmentExpression(assign_expr) = &expr_stmt.expression {
                    if let Some(name) = assign_expr
                        .left
                        .as_simple_assignment_target()
                        .and_then(SimpleAssignmentTarget::get_identifier_name)
                    {
                        if !expression_contains_identifier(&assign_expr.right, name)
                            && let Some(old_span) =
                                pending_promises.remove(CompactStr::from(name).as_str())
                        {
                            ctx.diagnostic(expect_in_unhandled_promise(old_span));
                        }
                        if scanner.found_expect_in_promise {
                            pending_promises.insert(CompactStr::from(name), expr_stmt.span);
                        }
                    } else if scanner.found_expect_in_promise {
                        ctx.diagnostic(expect_in_unhandled_promise(expr_stmt.span));
                    }
                } else if scanner.found_expect_in_promise
                    && is_top_level_promise_chain(&expr_stmt.expression)
                {
                    ctx.diagnostic(expect_in_unhandled_promise(expr_stmt.span));
                }
            }
            Statement::ReturnStatement(return_stmt) => {
                if let Some(name) = return_stmt.argument.as_ref().and_then(|arg| ident_name_of(arg))
                {
                    pending_promises.remove(name);
                }
                *return_found = true;
            }
            Statement::BlockStatement(block) => {
                process_statements(&block.body, pending_promises, return_found, ctx);
            }
            _ => {}
        }
    }
}

fn resolve_pending_promises(
    pending_promises: &mut FxHashMap<CompactStr, Span>,
    resolved_names: &FxHashSet<CompactStr>,
) {
    for name in resolved_names {
        pending_promises.remove(name.as_str());
    }
}

fn ident_name_of<'a>(expr: &'a Expression<'a>) -> Option<&'a str> {
    if let Expression::Identifier(ident) = expr { Some(ident.name.as_str()) } else { None }
}

/// Walks down the callee chain of `expect(x).resolves.not.toBe(2)` to find
/// the arguments of the innermost `expect(...)` call.
fn find_expect_args<'a>(
    call_expr: &'a CallExpression<'a>,
) -> Option<&'a oxc_allocator::Vec<'a, Argument<'a>>> {
    if let Expression::Identifier(ident) = &call_expr.callee
        && ident.name == "expect"
    {
        return Some(&call_expr.arguments);
    }
    find_inner_expect(&call_expr.callee)
}

fn find_inner_expect<'a>(
    expr: &'a Expression<'a>,
) -> Option<&'a oxc_allocator::Vec<'a, Argument<'a>>> {
    match expr {
        Expression::CallExpression(call) => find_expect_args(call),
        _ => find_inner_expect(expr.as_member_expression()?.object()),
    }
}

/// Returns `true` if the expression contains a reference to the given identifier name.
/// Used to check if `somePromise = somePromise.then(...)` continues the same chain.
fn expression_contains_identifier(expr: &Expression, name: &str) -> bool {
    let mut finder = IdentifierFinder { name, found: false };
    finder.visit_expression(expr);
    finder.found
}

struct IdentifierFinder<'b> {
    name: &'b str,
    found: bool,
}

impl<'a> Visit<'a> for IdentifierFinder<'_> {
    fn visit_identifier_reference(&mut self, ident: &oxc_ast::ast::IdentifierReference<'a>) {
        if ident.name == self.name {
            self.found = true;
        }
    }
}

/// Checks whether the expression statement's outermost call is itself a promise
/// chain (`.then/.catch/.finally`). This prevents false positives when a `.then()`
/// with `expect()` is nested deep inside unrelated structures like:
///
/// ```js
/// promiseSomething({
///   promise: something().then(value => { expect(value).toBe('red'); })
/// });
/// ```
///
/// Here the scanner finds `expect` inside `.then()`, but the statement's top-level
/// expression is `promiseSomething(...)` — not a promise chain we can track for
/// return/await. Since there's no way to ensure this buried promise is handled,
/// we bail out and don't report.
fn is_top_level_promise_chain(expr: &Expression) -> bool {
    let Expression::CallExpression(call_expr) = expr else {
        return false;
    };
    is_promise_call_expression(call_expr)
}

fn get_checkable_callback_body<'a>(callback: &'a Argument<'a>) -> Option<&'a FunctionBody<'a>> {
    match callback {
        Argument::ArrowFunctionExpression(arrow) => {
            if arrow.expression || !arrow.params.items.is_empty() {
                return None;
            }
            Some(&arrow.body)
        }
        Argument::FunctionExpression(func) => {
            if !func.params.items.is_empty() {
                return None;
            }
            func.body.as_ref().map(AsRef::as_ref)
        }
        _ => None,
    }
}

struct PromiseExpectScanner {
    /// Whether we are currently inside a promise chain callback.
    in_promise_chain: bool,
    /// Whether we are currently inside an `await` expression.
    in_await: bool,
    /// Set to `true` once we find an `expect()` inside a promise chain callback
    /// that is NOT already inside an `await`.
    found_expect_in_promise: bool,
    /// Identifiers that were properly resolved (awaited, expect().resolves, etc.)
    resolved_names: FxHashSet<CompactStr>,
}

impl PromiseExpectScanner {
    fn new() -> Self {
        Self {
            in_promise_chain: false,
            in_await: false,
            found_expect_in_promise: false,
            resolved_names: FxHashSet::default(),
        }
    }

    fn resolve_ident(&mut self, expr: &Expression) {
        if let Some(name) = ident_name_of(expr) {
            self.resolved_names.insert(CompactStr::from(name));
        }
    }

    fn collect_resolved_from_promise_wrapper(&mut self, call_expr: &CallExpression) {
        let Some(member) = call_expr.callee.as_member_expression() else { return };
        let Expression::Identifier(obj) = member.object() else { return };
        if obj.name != "Promise" {
            return;
        }

        let first_arg = call_expr.arguments.first().and_then(|a| a.as_expression());

        match member.static_property_name() {
            Some("all" | "allSettled" | "race" | "any") => {
                if let Some(Expression::ArrayExpression(arr)) = first_arg {
                    for elem in &arr.elements {
                        if let Some(expr) = elem.as_expression() {
                            self.resolve_ident(expr);
                        }
                    }
                }
            }
            Some("resolve" | "reject") => {
                if let Some(expr) = first_arg {
                    self.resolve_ident(expr);
                }
            }
            _ => {}
        }
    }
}

fn is_promise_call_expression(call_expr: &CallExpression<'_>) -> bool {
    call_expr
        .callee
        .as_member_expression()
        .and_then(MemberExpression::static_property_name)
        .is_some_and(|prop| matches!(prop, "then" | "catch" | "finally"))
}

impl<'a> Visit<'a> for PromiseExpectScanner {
    fn visit_call_expression(&mut self, call_expr: &CallExpression<'a>) {
        // Check for `expect(promise).resolves/rejects` — resolves the promise variable
        let callee_name = get_node_name_vec(&call_expr.callee);
        let is_expect_node = callee_name.first().is_some_and(|n| n == "expect");
        let is_expecting_promise =
            is_expect_node && callee_name.iter().any(|n| n == "resolves" || n == "rejects");

        if is_expecting_promise
            && let Some(expr) = find_expect_args(call_expr)
                .and_then(|args| args.first())
                .and_then(|arg| arg.as_expression())
        {
            self.resolve_ident(expr);
        }

        self.collect_resolved_from_promise_wrapper(call_expr);

        if is_promise_call_expression(call_expr) {
            let was_in_chain = self.in_promise_chain;

            // Only flag as promise chain if not already inside an await
            self.in_promise_chain = !self.in_await;

            self.visit_expression(&call_expr.callee);

            // Only walk the first 2 arguments of .then/.catch/.finally
            // (.then takes at most 2 callbacks; 3rd+ args are non-standard)
            for arg in call_expr.arguments.iter().take(2) {
                self.visit_argument(arg);
            }
            self.in_promise_chain = was_in_chain;
            return;
        }

        if self.in_promise_chain && is_expect_node {
            self.found_expect_in_promise = true;
            return;
        }

        walk::walk_call_expression(self, call_expr);
    }

    fn visit_await_expression(&mut self, await_expr: &oxc_ast::ast::AwaitExpression<'a>) {
        self.resolve_ident(&await_expr.argument);
        let was_in_await = self.in_await;
        self.in_await = true;
        walk::walk_await_expression(self, await_expr);
        self.in_await = was_in_await;
    }
}
