use oxc_ast::{
    ast::{Argument, CallExpression, Expression, MemberExpression},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    context::LintContext,
    fixer::Fix,
    rule::Rule,
    utils::{is_type_of_jest_fn_call, JestFnKind, JestGeneralFnKind},
    AstNode,
};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-jest(prefer-todo): Suggest using `test.todo`.")]
#[diagnostic(severity(warning))]
pub struct EmptyTest(#[label] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-jest(prefer-todo): Suggest using `test.todo`.")]
#[diagnostic(severity(warning))]
struct UmImplementedTestDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct PreferTodo;

declare_oxc_lint!(
    /// ### What it does
    /// When test cases are empty then it is better to mark them as `test.todo` as it
    /// will be highlighted in the summary output.
    ///
    /// ### Why is this bad?
    ///
    /// This rule triggers a warning if empty test cases are used without 'test.todo'.
    ///
    /// ### Example
    ///
    /// ```javascript
    /// test('i need to write this test'); // invalid
    /// test('i need to write this test', () => {}); // invalid
    /// test.skip('i need to write this test', () => {}); // invalid
    ///
    /// test.todo('i need to write this test');
    /// ```
    PreferTodo,
    restriction,
);

impl Rule for PreferTodo {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::CallExpression(call_expr) = node.kind() {
            let counts = call_expr.arguments.len();

            if counts < 1
                || should_filter_case(call_expr)
                || !is_string_type(&call_expr.arguments[0])
                || !is_type_of_jest_fn_call(
                    call_expr,
                    node,
                    ctx,
                    &[JestFnKind::General(JestGeneralFnKind::Test)],
                )
            {
                return;
            }

            if counts == 1 && !filter_todo_case(call_expr) {
                let (content, span) = get_fix_content(call_expr);
                ctx.diagnostic_with_fix(UmImplementedTestDiagnostic(span), || {
                    Fix::new(content, span)
                });
            }

            if is_empty_function(call_expr) {
                let (_, span) = get_fix_content(call_expr);
                ctx.diagnostic_with_fix(EmptyTest(span), || {
                    let (content, span) = build_code(call_expr, ctx);
                    Fix::new(content, span)
                });
            }
        }
    }
}

fn filter_todo_case(expr: &CallExpression) -> bool {
    if let Expression::MemberExpression(mem_expr) = &expr.callee {
        if let Some(name) = mem_expr.static_property_name() {
            return name == "todo";
        }
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
    matches!(
        arg,
        Argument::Expression(Expression::StringLiteral(_) | Expression::TemplateLiteral(_))
    )
}

fn is_empty_function(expr: &CallExpression) -> bool {
    let mut is_empty = false;

    for argument in &expr.arguments {
        match argument {
            Argument::Expression(Expression::ArrowExpression(arrow)) => {
                is_empty = arrow.body.is_empty();
                break;
            }
            Argument::Expression(Expression::FunctionExpression(func)) => {
                let Some(func_body) = &func.body else {
                    continue;
                };
                if func_body.is_empty() {
                    is_empty = true;
                    break;
                }
            }
            _ => continue,
        }
    }

    is_empty
}

fn get_fix_content<'a>(expr: &'a CallExpression<'a>) -> (&'a str, Span) {
    match &expr.callee {
        Expression::Identifier(ident) => {
            (".todo", Span { start: ident.span.end, end: ident.span.end })
        }
        Expression::MemberExpression(mem_expr) => {
            if let Some((span, _)) = mem_expr.static_property_info() {
                return ("todo", span);
            }
            ("", expr.span)
        }
        _ => ("", expr.span),
    }
}

fn build_code(expr: &CallExpression, ctx: &LintContext) -> (String, Span) {
    let mut formatter = ctx.formatter();

    if let Expression::Identifier(ident) = &expr.callee {
        formatter.print_str(ident.name.as_bytes());
        formatter.print_str(b".todo(");
    } else if let Expression::MemberExpression(mem_expr) = &expr.callee {
        match &mem_expr.0 {
            MemberExpression::ComputedMemberExpression(expr) => {
                if let Expression::Identifier(ident) = &expr.object {
                    formatter.print_str(ident.name.as_bytes());
                    formatter.print_str(b"[");
                    formatter.print_str(b"'todo'");
                    formatter.print_str(b"](");
                }
            }
            MemberExpression::StaticMemberExpression(expr) => {
                if let Expression::Identifier(ident) = &expr.object {
                    formatter.print_str(ident.name.as_bytes());
                    formatter.print_str(b".todo(");
                }
            }
            MemberExpression::PrivateFieldExpression(_) => {}
        }
    }

    if let Argument::Expression(Expression::StringLiteral(ident)) = &expr.arguments[0] {
        // Todo: this punctuation should read from the config
        formatter.print(b'\'');
        formatter.print_str(ident.value.as_bytes());
        formatter.print(b'\'');
        formatter.print(b')');
    } else if let Argument::Expression(Expression::TemplateLiteral(temp)) = &expr.arguments[0] {
        formatter.print(b'`');
        for q in &temp.quasis {
            formatter.print_str(q.value.raw.as_bytes());
        }
        formatter.print(b'`');
        formatter.print(b')');
    }

    (formatter.into_code(), expr.span)
}

#[test]
fn tests() {
    use crate::tester::Tester;

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
    ];

    Tester::new(PreferTodo::NAME, pass, fail).expect_fix(fix).test_and_snapshot();
}
