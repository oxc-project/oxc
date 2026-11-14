use oxc_ast::{
    AstKind,
    ast::{Argument, CallExpression, Expression, MemberExpression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_semantic::AstNode;
use oxc_span::{GetSpan, Span};

use crate::{
    context::LintContext,
    fixer::{RuleFix, RuleFixer},
    utils::{JestFnKind, JestGeneralFnKind, PossibleJestNode, is_type_of_jest_fn_call},
};

fn prefer_todo_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Suggest using `test.todo`.").with_label(span)
}

pub const DOCUMENTATION: &str = r"### What it does

When test cases are empty then it is better to mark them as `test.todo` as it
will be highlighted in the summary output.

### Why is this bad?

This rule triggers a warning if empty test cases are used without 'test.todo'.

### Examples

Examples of **incorrect** code for this rule:
```javascript
test('i need to write this test'); // invalid
test.skip('i need to write this test', () => {}); // invalid
```

Examples of **correct** code for this rule:
```javascript
test.todo('i need to write this test');
```
";

pub fn run<'a>(node: &AstNode<'a>, ctx: &LintContext<'a>) {
    let AstKind::CallExpression(call_expr) = node.kind() else {
        return;
    };

    let counts = call_expr.arguments.len();

    if counts < 1 || should_filter_case(call_expr) || !is_string_type(&call_expr.arguments[0]) {
        return;
    }

    let is_empty = counts > 1 && is_empty_function(call_expr);
    if (counts == 1 && !filter_todo_case(call_expr)) || is_empty {
        let Some(possible_jest_node) = PossibleJestNode::from_call_expression(node, call_expr, ctx)
        else {
            return;
        };
        if !is_type_of_jest_fn_call(
            call_expr,
            &possible_jest_node,
            ctx,
            &[JestFnKind::General(JestGeneralFnKind::Test)],
        ) {
            return;
        }

        let span = call_expr.callee.span();
        let diagnostic = prefer_todo_diagnostic(span);

        if is_safe_fix_target(call_expr) {
            ctx.diagnostic_with_fix(diagnostic, |fixer| build_fix(fixer, call_expr));
        } else {
            ctx.diagnostic(diagnostic);
        }
    }
}

/// Only the following rewrites preserve runtime behavior:
///
/// - `test('title')` / `it('title')` -> `test.todo('title')`
///   Vitest/Jest already treat a one-argument test call as a todo.
/// - `test.skip('title')` / `test.skip('title', () => {})` -> `test.todo('title')`
///   Skipped tests run no hooks, so removing the empty body is safe.
///
/// Bare `test('title', () => {})` and non-`skip` modifiers (`.only`,
/// `.fails`, `.concurrent`, `.each`, `.runIf`, etc.) can change which tests
/// run or whether hooks execute, so they must not be auto-fixed.
fn is_safe_fix_target(call_expr: &CallExpression) -> bool {
    if let Some(name) =
        call_expr.callee.as_member_expression().and_then(MemberExpression::static_property_name)
    {
        return name == "skip";
    }

    matches!(&call_expr.callee, Expression::Identifier(_)) && call_expr.arguments.len() == 1
}

fn build_fix<'a>(fixer: RuleFixer<'_, 'a>, call_expr: &CallExpression<'a>) -> RuleFix {
    let fix = match &call_expr.callee {
        Expression::Identifier(ident) => fixer.insert_text_after_range(ident.span, ".todo"),
        Expression::StaticMemberExpression(mem_expr) => {
            fixer.replace(mem_expr.property.span, "todo")
        }
        Expression::ComputedMemberExpression(mem_expr) => {
            fixer.replace(mem_expr.expression.span(), "'todo'")
        }
        _ => {
            // Only reachable in practice if `is_safe_fix_target` returns true
            // for an unexpected shape (e.g. a `skip` call expression). Fall back
            // to a no-op to avoid deleting code.
            return fixer.noop();
        }
    };

    if call_expr.arguments.len() == 1 {
        return fix;
    }

    let multi_fixer = fixer.for_multifix();
    let mut multi_fix = multi_fixer.new_fix_with_capacity(2);
    multi_fix.push(fix);
    multi_fix.push(
        multi_fixer
            .delete_range(Span::new(call_expr.arguments[0].span().end, call_expr.span.end - 1)),
    );
    multi_fix.with_message("Replace with `test.todo` or `it.todo`.")
}

fn filter_todo_case(expr: &CallExpression) -> bool {
    if let Some(mem_expr) = expr.callee.as_member_expression()
        && let Some(name) = mem_expr.static_property_name()
    {
        return name == "todo";
    }
    false
}

fn should_filter_case(expr: &CallExpression) -> bool {
    if let Expression::Identifier(ident) = &expr.callee {
        return ident.name.starts_with('x') || ident.name.starts_with('f');
    }

    let Some(mem_expr) = expr.callee.as_member_expression() else {
        // `test.each(...)()`, `test.runIf(...)()` and similar call-chain
        // callees are not targetable with a behavior-preserving fix.
        return true;
    };
    // Only `.skip` members can be safely rewritten. All other modifiers
    // (`.only`, `.fails`, `.concurrent`, dynamic computed members, etc.) are
    // intentionally skipped.
    mem_expr.static_property_name().is_none_or(|name| name != "skip")
}

fn is_string_type(arg: &Argument) -> bool {
    matches!(arg, Argument::StringLiteral(_) | Argument::TemplateLiteral(_))
}

fn is_empty_function(expr: &CallExpression) -> bool {
    match &expr.arguments[1] {
        Argument::ArrowFunctionExpression(arrow) => arrow.body.is_empty(),
        Argument::FunctionExpression(func) => {
            let Some(func_body) = &func.body else {
                return false;
            };
            func_body.is_empty()
        }
        _ => false,
    }
}
