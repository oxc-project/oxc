use oxc_ast::{
    AstKind,
    ast::{Argument, CallExpression, Expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{JestFnKind, JestGeneralFnKind, PossibleJestNode, is_type_of_jest_fn_call},
};

fn prefer_todo_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Suggest using `test.todo`.").with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferTodo;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// When test cases are empty then it is better to mark them as `test.todo` as it
    /// will be highlighted in the summary output.
    ///
    /// ### Why is this bad?
    ///
    /// This rule triggers a warning if empty test cases are used without 'test.todo'.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// test('i need to write this test'); // invalid
    /// test('i need to write this test', () => {}); // invalid
    /// test.skip('i need to write this test', () => {}); // invalid
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// test.todo('i need to write this test');
    /// ```
    ///
    /// This rule is compatible with [eslint-plugin-vitest](https://github.com/vitest-dev/eslint-plugin-vitest/blob/main/docs/rules/prefer-todo.md),
    /// to use it, add the following configuration to your `.oxlintrc.json`:
    ///
    /// ```json
    /// {
    ///   "rules": {
    ///      "vitest/prefer-todo": "error"
    ///   }
    /// }
    /// ```
    PreferTodo,
    jest,
    style,
    fix
);

impl Rule for PreferTodo {
    fn run_on_jest_node<'a, 'c>(
        &self,
        possible_jest_node: &PossibleJestNode<'a, 'c>,
        ctx: &'c LintContext<'a>,
    ) {
        let node = possible_jest_node.node;
        if let AstKind::CallExpression(call_expr) = node.kind() {
            let counts = call_expr.arguments.len();

            if counts < 1
                || should_filter_case(call_expr)
                || !is_string_type(&call_expr.arguments[0])
                || !is_type_of_jest_fn_call(
                    call_expr,
                    possible_jest_node,
                    ctx,
                    &[JestFnKind::General(JestGeneralFnKind::Test)],
                )
            {
                return;
            }

            if (counts == 1 && !filter_todo_case(call_expr))
                || (counts > 1 && is_empty_function(call_expr))
            {
                let span = call_expr.callee.span();
                ctx.diagnostic_with_fix(prefer_todo_diagnostic(span), |fixer| {
                    let fix = if let Expression::Identifier(ident) = &call_expr.callee {
                        fixer.insert_text_after_range(ident.span, ".todo")
                    } else {
                        match &call_expr.callee {
                            Expression::StaticMemberExpression(mem_expr) => {
                                fixer.replace(mem_expr.property.span, "todo")
                            }
                            Expression::ComputedMemberExpression(mem_expr) => {
                                fixer.replace(mem_expr.expression.span(), "'todo'")
                            }
                            _ => return fixer.delete_range(call_expr.span),
                        }
                    };

                    if counts == 1 {
                        return fix;
                    }

                    let multi_fixer = fixer.for_multifix();
                    let mut multi_fix = multi_fixer.new_fix_with_capacity(2);
                    multi_fix.push(fix);
                    multi_fix.push(multi_fixer.delete_range(Span::new(
                        call_expr.arguments[0].span().end,
                        call_expr.span.end - 1,
                    )));
                    multi_fix.with_message("Replace with `test.todo` or `it.todo`.")
                });
            }
        }
    }
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
    let result = match &expr.callee {
        Expression::Identifier(ident) => ident.name.starts_with('x') || ident.name.starts_with('f'),
        _ => false,
    };
    result || filter_todo_case(expr)
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

#[test]
fn tests() {
    use crate::tester::Tester;

    // Note: Both Jest and Vitest share the same unit tests

    let pass = vec![
        ("test()", None),
        ("test.concurrent()", None),
        ("test.todo('i need to write this test correct');", None),
        ("test(obj)", None),
        ("test.concurrent(obj)", None),
        ("fit('foo')", None),
        ("fit.concurrent('foo')", None),
        ("xit('foo')", None),
        ("test('foo', 1)", None),
        ("test('stub', () => expect(1).toBe(1));", None),
        ("test.concurrent('stub', () => expect(1).toBe(1));", None),
        (
            "
                supportsDone && params.length < test.length
                    ? done => test(...params, done)
                    : () => test(...params);
            ",
            None,
        ),
    ];

    let fail = vec![
        ("test('i need to write this test');", None),
        ("test('i need to write this test',);", None),
        ("test(`i need to write this test`);", None),
        ("it('foo', function () {})", None),
        ("it('foo', () => {})", None),
        ("test.skip('i need to write this test', () => {});", None),
        ("test.skip('i need to write this test', function() {});", None),
        ("test[`skip`]('i need to write this test', function() {});", None),
        ("test[`skip`]('i need to write this test', function() {});", None),
    ];

    let fix = vec![
        (
            "test.skip('i need to write this test');",
            "test.todo('i need to write this test');",
            None,
        ),
        ("test('i need to write this test',);", "test.todo('i need to write this test',);", None),
        ("test(`i need to write this test`);", "test.todo(`i need to write this test`);", None),
        ("it.skip('foo', function () {})", "it.todo('foo')", None),
        ("it(`i need to write this test`, () => {})", "it.todo(`i need to write this test`)", None),
        ("it('foo', function () {})", "it.todo('foo')", None),
        ("it('foo', () => {})", "it.todo('foo')", None),
        (
            "test.skip('i need to write this test', () => {});",
            "test.todo('i need to write this test');",
            None,
        ),
        (
            "test.skip('i need to write this test', function() {});",
            "test.todo('i need to write this test');",
            None,
        ),
        (
            "test['skip']('i need to write this test', function() {});",
            "test['todo']('i need to write this test');",
            None,
        ),
        (
            "test['skip']('i need to write this test', () => {});",
            "test['todo']('i need to write this test');",
            None,
        ),
        (
            "test['skip']('i need to write this test');",
            "test['todo']('i need to write this test');",
            None,
        ),
    ];

    Tester::new(PreferTodo::NAME, PreferTodo::PLUGIN, pass, fail)
        .with_jest_plugin(true)
        .expect_fix(fix)
        .test_and_snapshot();
}
