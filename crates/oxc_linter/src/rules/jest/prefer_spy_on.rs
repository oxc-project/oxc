use oxc_ast::{
    ast::{
        Argument, AssignmentExpression, AssignmentTarget, CallExpression, Expression,
        MemberExpression, SimpleAssignmentTarget,
    },
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_semantic::AstNode;
use oxc_span::Span;

use crate::{
    context::LintContext,
    fixer::Fix,
    rule::Rule,
    utils::{get_node_name, parse_general_jest_fn_call, PossibleJestNode},
};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-jest(prefer-spy-on): Suggest using `jest.spyOn()`.")]
#[diagnostic(severity(warning), help("Use jest.spyOn() instead"))]
struct UseJestSpyOn(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct PreferSpyOn;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// When mocking a function by overwriting a property you have to manually restore
    /// the original implementation when cleaning up. When using `jest.spyOn()` Jest
    /// keeps track of changes, and they can be restored with `jest.restoreAllMocks()`,
    /// `mockFn.mockRestore()` or by setting `restoreMocks` to `true` in the Jest
    /// config.
    ///
    /// Note: The mock created by `jest.spyOn()` still behaves the same as the original
    /// function. The original function can be overwritten with
    /// `mockFn.mockImplementation()` or by some of the
    /// [other mock functions](https://jestjs.io/docs/en/mock-function-api).
    ///
    /// ### Example
    ///
    /// ```javascript
    /// // invalid
    /// Date.now = jest.fn();
    /// Date.now = jest.fn(() => 10);
    ///
    /// // valid
    /// jest.spyOn(Date, 'now');
    /// jest.spyOn(Date, 'now').mockImplementation(() => 10);
    /// ```
    PreferSpyOn,
    style,
);

impl Rule for PreferSpyOn {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::AssignmentExpression(assign_expr) = node.kind() else {
            return;
        };

        let left = &assign_expr.left;
        let right = &assign_expr.right;

        let AssignmentTarget::SimpleAssignmentTarget(
            SimpleAssignmentTarget::MemberAssignmentTarget(left_assign),
        ) = left
        else {
            return;
        };

        match right {
            Expression::CallExpression(call_expr) => {
                Self::check_and_fix(assign_expr, call_expr, left_assign, node, ctx);
            }
            Expression::MemberExpression(mem_expr) => {
                let Expression::CallExpression(call_expr) = mem_expr.object() else {
                    return;
                };
                Self::check_and_fix(assign_expr, call_expr, left_assign, node, ctx);
            }
            _ => (),
        }
    }
}

impl PreferSpyOn {
    fn check_and_fix<'a>(
        assign_expr: &AssignmentExpression,
        call_expr: &'a CallExpression<'a>,
        left_assign: &MemberExpression,
        node: &AstNode<'a>,
        ctx: &LintContext<'a>,
    ) {
        let Some(jest_fn_call) =
            parse_general_jest_fn_call(call_expr, &PossibleJestNode { node, original: None }, ctx)
        else {
            return;
        };
        let Some(first_fn_member) = jest_fn_call.members.first() else {
            return;
        };

        if first_fn_member.name().unwrap() != "fn" {
            return;
        }

        ctx.diagnostic_with_fix(
            UseJestSpyOn(Span::new(call_expr.span.start, first_fn_member.span.end)),
            || {
                let (end, has_mock_implementation) = if jest_fn_call.members.len() > 1 {
                    let second = &jest_fn_call.members[1];
                    let has_mock_implementation = jest_fn_call
                        .members
                        .iter()
                        .any(|modifier| modifier.is_name_equal("mockImplementation"));

                    (second.span.start - 1, has_mock_implementation)
                } else {
                    (
                        first_fn_member.span.end + (call_expr.span.end - first_fn_member.span.end),
                        false,
                    )
                };
                let content =
                    Self::build_code(call_expr, left_assign, has_mock_implementation, ctx);
                Fix::new(content, Span::new(assign_expr.span.start, end))
            },
        );
    }

    fn build_code<'a>(
        call_expr: &'a CallExpression<'a>,
        left_assign: &MemberExpression,
        has_mock_implementation: bool,
        ctx: &LintContext,
    ) -> String {
        let mut formatter = ctx.codegen();
        formatter.print_str(b"jest.spyOn(");

        match left_assign {
            MemberExpression::ComputedMemberExpression(cmp_mem_expr) => {
                formatter.print_expression(&cmp_mem_expr.object);
                formatter.print(b',');
                formatter.print_hard_space();
                formatter.print_expression(&cmp_mem_expr.expression);
            }
            MemberExpression::StaticMemberExpression(static_mem_expr) => {
                let name = &static_mem_expr.property.name;
                formatter.print_expression(&static_mem_expr.object);
                formatter.print(b',');
                formatter.print_hard_space();
                formatter.print_str(format!("\'{name}\'").as_bytes());
            }
            MemberExpression::PrivateFieldExpression(_) => (),
        }

        formatter.print(b')');

        if has_mock_implementation {
            return formatter.into_source_text();
        }

        formatter.print_str(b".mockImplementation(");

        if let Some(Argument::Expression(expr)) = Self::get_jest_fn_call(call_expr) {
            formatter.print_expression(expr);
        }

        formatter.print(b')');
        formatter.into_source_text()
    }

    fn get_jest_fn_call<'a>(call_expr: &'a CallExpression<'a>) -> Option<&'a Argument<'a>> {
        let is_jest_fn = get_node_name(&call_expr.callee) == "jest.fn";

        if is_jest_fn {
            return call_expr.arguments.first();
        }

        match &call_expr.callee {
            Expression::MemberExpression(mem_expr) => {
                if let Some(call_expr) = Self::find_mem_expr(mem_expr) {
                    return Self::get_jest_fn_call(call_expr);
                }
                None
            }
            Expression::CallExpression(call_expr) => Self::get_jest_fn_call(call_expr),
            _ => None,
        }
    }

    fn find_mem_expr<'a>(mem_expr: &'a MemberExpression<'a>) -> Option<&'a CallExpression<'a>> {
        match mem_expr.object() {
            Expression::CallExpression(call_expr) => Some(call_expr),
            Expression::MemberExpression(mem_expr) => Self::find_mem_expr(mem_expr),
            _ => None,
        }
    }
}

#[test]
fn tests() {
    use crate::tester::Tester;

    let pass = vec![
        ("Date.now = () => 10", None),
        ("window.fetch = jest.fn", None),
        ("Date.now = fn()", None),
        ("obj.mock = jest.something()", None),
        ("const mock = jest.fn()", None),
        ("mock = jest.fn()", None),
        ("const mockObj = { mock: jest.fn() }", None),
        ("mockObj = { mock: jest.fn() }", None),
        ("window[`${name}`] = jest[`fn${expression}`]()", None),
    ];

    let fail = vec![
        ("obj.a = jest.fn(); const test = 10;", None),
        ("Date['now'] = jest['fn']()", None),
        ("window[`${name}`] = jest[`fn`]()", None),
        ("obj['prop' + 1] = jest['fn']()", None),
        ("obj.one.two = jest.fn(); const test = 10;", None),
        ("obj.a = jest.fn(() => 10,)", None),
        (
            "obj.a.b = jest.fn(() => ({})).mockReturnValue('default').mockReturnValueOnce('first call'); test();",
            None,
        ),
        ("window.fetch = jest.fn(() => ({})).one.two().three().four", None),
        ("foo[bar] = jest.fn().mockReturnValue(undefined)", None),
        (
            "
                foo.bar = jest.fn().mockImplementation(baz => baz)
                foo.bar = jest.fn(a => b).mockImplementation(baz => baz)
            ",
            None,
        ),
    ];

    let fix = vec![
        (
            "obj.a = jest.fn(); const test = 10;",
            "jest.spyOn(obj, 'a').mockImplementation(); const test = 10;",
            None,
        ),
        ("Date['now'] = jest['fn']()", "jest.spyOn(Date, 'now').mockImplementation()", None),
        (
            "window[`${name}`] = jest[`fn`]()",
            "jest.spyOn(window, `${name}`).mockImplementation()",
            None,
        ),
        (
            "obj['prop' + 1] = jest['fn']()",
            "jest.spyOn(obj, 'prop' + 1).mockImplementation()",
            None,
        ),
        (
            "obj.one.two = jest.fn(); const test = 10;",
            "jest.spyOn(obj.one, 'two').mockImplementation(); const test = 10;",
            None,
        ),
        ("obj.a = jest.fn(() => 10,)", "jest.spyOn(obj, 'a').mockImplementation(() => 10)", None),
        (
            "obj.a.b = jest.fn(() => ({})).mockReturnValue('default').mockReturnValueOnce('first call'); test();",
            "jest.spyOn(obj.a, 'b').mockImplementation(() => ({})).mockReturnValue('default').mockReturnValueOnce('first call'); test();",
            None,
        ),
        (
            "window.fetch = jest.fn(() => ({})).one.two().three().four",
            "jest.spyOn(window, 'fetch').mockImplementation(() => ({})).one.two().three().four",
            None,
        ),
        (
            "foo[bar] = jest.fn().mockReturnValue(undefined)",
            "jest.spyOn(foo, bar).mockImplementation().mockReturnValue(undefined)",
            None,
        ),
        (
            "
                foo.bar = jest.fn().mockImplementation(baz => baz)
                foo.bar = jest.fn(a => b).mockImplementation(baz => baz)
            ",
            "
                jest.spyOn(foo, 'bar').mockImplementation(baz => baz)
                jest.spyOn(foo, 'bar').mockImplementation(baz => baz)
            ",
            None,
        ),
    ];

    Tester::new(PreferSpyOn::NAME, pass, fail)
        .with_jest_plugin(true)
        .expect_fix(fix)
        .test_and_snapshot();
}
