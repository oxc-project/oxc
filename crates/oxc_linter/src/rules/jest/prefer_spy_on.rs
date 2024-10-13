use oxc_ast::{
    ast::{
        Argument, AssignmentExpression, CallExpression, Expression, MemberExpression,
        SimpleAssignmentTarget,
    },
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::AstNode;
use oxc_span::Span;

use crate::{
    context::LintContext,
    fixer::RuleFixer,
    rule::Rule,
    utils::{get_node_name, parse_general_jest_fn_call, PossibleJestNode},
};

fn use_jest_spy_on(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Suggest using `jest.spyOn()`.")
        .with_help("Use jest.spyOn() instead")
        .with_label(span)
}

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
    fix
);

impl Rule for PreferSpyOn {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::AssignmentExpression(assign_expr) = node.kind() else {
            return;
        };

        let left = &assign_expr.left;
        let right = &assign_expr.right;

        let Some(left_assign) = left
            .as_simple_assignment_target()
            .and_then(SimpleAssignmentTarget::as_member_expression)
        else {
            return;
        };

        match right {
            Expression::CallExpression(call_expr) => {
                Self::check_and_fix(assign_expr, call_expr, left_assign, node, ctx);
            }
            _ => {
                if let Some(mem_expr) = right.as_member_expression() {
                    let Expression::CallExpression(call_expr) = mem_expr.object() else {
                        return;
                    };
                    Self::check_and_fix(assign_expr, call_expr, left_assign, node, ctx);
                }
            }
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
            use_jest_spy_on(Span::new(call_expr.span.start, first_fn_member.span.end)),
            |fixer| {
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
                    Self::build_code(call_expr, left_assign, has_mock_implementation, fixer);
                fixer.replace(Span::new(assign_expr.span.start, end), content)
            },
        );
    }

    fn build_code<'a>(
        call_expr: &'a CallExpression<'a>,
        left_assign: &MemberExpression,
        has_mock_implementation: bool,
        fixer: RuleFixer<'_, 'a>,
    ) -> String {
        let mut formatter = fixer.codegen();
        formatter.print_str("jest.spyOn(");

        match left_assign {
            MemberExpression::ComputedMemberExpression(cmp_mem_expr) => {
                formatter.print_expression(&cmp_mem_expr.object);
                formatter.print_ascii_byte(b',');
                formatter.print_ascii_byte(b' ');
                formatter.print_expression(&cmp_mem_expr.expression);
            }
            MemberExpression::StaticMemberExpression(static_mem_expr) => {
                let name = &static_mem_expr.property.name;
                formatter.print_expression(&static_mem_expr.object);
                formatter.print_ascii_byte(b',');
                formatter.print_ascii_byte(b' ');
                formatter.print_str(format!("\'{name}\'").as_str());
            }
            MemberExpression::PrivateFieldExpression(_) => (),
        }

        formatter.print_ascii_byte(b')');

        if has_mock_implementation {
            return formatter.into_source_text();
        }

        formatter.print_str(".mockImplementation(");

        if let Some(expr) = Self::get_jest_fn_call(call_expr) {
            formatter.print_expression(expr);
        }

        formatter.print_ascii_byte(b')');
        formatter.into_source_text()
    }

    fn get_jest_fn_call<'a>(call_expr: &'a CallExpression<'a>) -> Option<&'a Expression<'a>> {
        let is_jest_fn = get_node_name(&call_expr.callee) == "jest.fn";

        if is_jest_fn {
            return call_expr.arguments.first().and_then(Argument::as_expression);
        }

        match &call_expr.callee {
            expr if expr.is_member_expression() => {
                let mem_expr = expr.to_member_expression();
                if let Some(call_expr) = Self::find_mem_expr(mem_expr) {
                    return Self::get_jest_fn_call(call_expr);
                }
                None
            }
            Expression::CallExpression(call_expr) => Self::get_jest_fn_call(call_expr),
            _ => None,
        }
    }

    fn find_mem_expr<'a>(mut mem_expr: &'a MemberExpression<'a>) -> Option<&'a CallExpression<'a>> {
        loop {
            let object = mem_expr.object();
            if let Expression::CallExpression(call_expr) = object {
                return Some(call_expr);
            }
            if let Some(object_mem_expr) = object.as_member_expression() {
                mem_expr = object_mem_expr;
            } else {
                return None;
            }
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
