use oxc_ast::{
    AstKind,
    ast::{
        Argument, AssignmentExpression, CallExpression, Expression, MemberExpression,
        SimpleAssignmentTarget,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::AstNode;
use oxc_span::Span;

use crate::{
    context::LintContext,
    fixer::RuleFixer,
    rule::Rule,
    utils::{
        KnownMemberExpressionProperty, PossibleJestNode, get_node_name, parse_general_jest_fn_call,
    },
};

fn use_jest_spy_on(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Suggest using `jest.spyOn()` or `vi.spyOn()`.").with_label(span)
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
    /// ### Why is this bad?
    ///
    /// Directly overwriting properties with mock functions can lead to cleanup issues
    /// and test isolation problems. When you manually assign a mock to a property,
    /// you're responsible for restoring the original implementation, which is easy to
    /// forget and can cause tests to interfere with each other. Using `jest.spyOn()`
    /// provides automatic cleanup capabilities and makes your tests more reliable.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// Date.now = jest.fn();
    /// Date.now = jest.fn(() => 10);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// jest.spyOn(Date, 'now');
    /// jest.spyOn(Date, 'now').mockImplementation(() => 10);
    /// ```
    ///
    /// This rule is compatible with [eslint-plugin-vitest](https://github.com/vitest-dev/eslint-plugin-vitest/blob/main/docs/rules/prefer-spy-on.md),
    /// to use it, add the following configuration to your `.oxlintrc.json`:
    ///
    /// ```json
    /// {
    ///   "rules": {
    ///      "vitest/prefer-spy-on": "error"
    ///   }
    /// }
    /// ```
    PreferSpyOn,
    jest,
    style,
    suggestion
);

impl Rule for PreferSpyOn {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::AssignmentExpression(assign_expr) = node.kind() else {
            return;
        };

        let Some(left_assign) = &assign_expr
            .left
            .as_simple_assignment_target()
            .and_then(SimpleAssignmentTarget::as_member_expression)
        else {
            return;
        };

        match &assign_expr.right {
            Expression::CallExpression(call_expr) => {
                Self::check_and_fix(assign_expr, call_expr, left_assign, node, ctx);
            }
            _ => {
                if let Some(mem_expr) = assign_expr.right.as_member_expression() {
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

        let Some(first_fn_member_name) =
            jest_fn_call.members.first().and_then(KnownMemberExpressionProperty::name)
        else {
            return;
        };

        if first_fn_member_name != "fn" {
            return;
        }

        ctx.diagnostic_with_suggestion(
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
        let (framework_spy, arguments) = Self::get_test_fn_call(call_expr);

        let mut formatter = fixer.codegen();
        formatter.print_str(framework_spy);

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

        if let Some(expr) = arguments {
            formatter.print_expression(expr);
        }

        formatter.print_ascii_byte(b')');
        formatter.into_source_text()
    }

    fn get_test_fn_call<'a>(
        call_expr: &'a CallExpression<'a>,
    ) -> (&'a str, Option<&'a Expression<'a>>) {
        let node_name = get_node_name(&call_expr.callee);
        let is_test_fn = node_name == "jest.fn" || node_name == "vi.fn";

        if is_test_fn {
            let framework_spy = match node_name.as_str() {
                "vi.fn" => "vi.spyOn(",
                _ => "jest.spyOn(",
            };
            return (framework_spy, call_expr.arguments.first().and_then(Argument::as_expression));
        }

        match &call_expr.callee {
            expr if expr.is_member_expression() => {
                let mem_expr = expr.to_member_expression();
                if let Some(call_expr) = Self::find_mem_expr(mem_expr) {
                    return Self::get_test_fn_call(call_expr);
                }
                ("", None)
            }
            Expression::CallExpression(call_expr) => Self::get_test_fn_call(call_expr),
            _ => ("", None),
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

    let mut pass = vec![
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

    let mut fail = vec![
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

    let mut fix = vec![
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

    let vitest_pass = vec![
        ("Date.now = () => 10", None),
        ("window.fetch = vi.fn", None),
        ("Date.now = fn()", None),
        ("obj.mock = vi.something()", None),
        ("const mock = vi.fn()", None),
        ("mock = vi.fn()", None),
        ("const mockObj = { mock: vi.fn() }", None),
        ("mockObj = { mock: vi.fn() }", None),
        ("window[`${name}`] = vi[`fn${expression}`]()", None),
    ];

    let vitest_fail = vec![
        ("obj.a = vi.fn(); const test = 10;", None),
        ("Date['now'] = vi['fn']()", None),
        ("window[`${name}`] = vi[`fn`]()", None),
        ("obj['prop' + 1] = vi['fn']()", None),
        ("obj.one.two = vi.fn(); const test = 10;", None),
        ("obj.a = vi.fn(() => 10,)", None), // { "parserOptions": { "ecmaVersion": 2017 } }
        (
            "obj.a.b = vi.fn(() => ({})).mockReturnValue('default').mockReturnValueOnce('first call'); test();",
            None,
        ),
        ("window.fetch = vi.fn(() => ({})).one.two().three().four", None),
        ("foo[bar] = vi.fn().mockReturnValue(undefined)", None),
        (
            "
			        foo.bar = vi.fn().mockImplementation(baz => baz)
			        foo.bar = vi.fn(a => b).mockImplementation(baz => baz)
			      ",
            None,
        ),
    ];

    let vitest_fix = vec![
        (
            "obj.a = vi.fn(); const test = 10;",
            "vi.spyOn(obj, 'a').mockImplementation(); const test = 10;",
            None,
        ),
        ("Date['now'] = vi['fn']()", "vi.spyOn(Date, 'now').mockImplementation()", None),
        (
            "window[`${name}`] = vi[`fn`]()",
            "vi.spyOn(window, `${name}`).mockImplementation()",
            None,
        ),
        ("obj['prop' + 1] = vi['fn']()", "vi.spyOn(obj, 'prop' + 1).mockImplementation()", None),
        (
            "obj.one.two = vi.fn(); const test = 10;",
            "vi.spyOn(obj.one, 'two').mockImplementation(); const test = 10;",
            None,
        ),
        ("obj.a = vi.fn(() => 10,)", "vi.spyOn(obj, 'a').mockImplementation(() => 10)", None),
        (
            "obj.a.b = vi.fn(() => ({})).mockReturnValue('default').mockReturnValueOnce('first call'); test();",
            "vi.spyOn(obj.a, 'b').mockImplementation(() => ({})).mockReturnValue('default').mockReturnValueOnce('first call'); test();",
            None,
        ),
        (
            "window.fetch = vi.fn(() => ({})).one.two().three().four",
            "vi.spyOn(window, 'fetch').mockImplementation(() => ({})).one.two().three().four",
            None,
        ),
        (
            "foo[bar] = vi.fn().mockReturnValue(undefined)",
            "vi.spyOn(foo, bar).mockImplementation().mockReturnValue(undefined)",
            None,
        ),
        (
            "
			        foo.bar = vi.fn().mockImplementation(baz => baz)
			        foo.bar = vi.fn(a => b).mockImplementation(baz => baz)
			      ",
            "
			        vi.spyOn(foo, 'bar').mockImplementation(baz => baz)
			        vi.spyOn(foo, 'bar').mockImplementation(baz => baz)
			      ",
            None,
        ),
    ];

    pass.extend(vitest_pass);
    fail.extend(vitest_fail);
    fix.extend(vitest_fix);

    Tester::new(PreferSpyOn::NAME, PreferSpyOn::PLUGIN, pass, fail)
        .expect_fix(fix)
        .with_jest_plugin(true)
        .with_vitest_plugin(true)
        .test_and_snapshot();
}
