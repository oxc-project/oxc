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
    utils::{
        collect_possible_jest_call_node, is_type_of_jest_fn_call, JestFnKind, JestGeneralFnKind,
        PossibleJestNode,
    },
};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-jest(prefer-todo): Suggest using `test.todo`.")]
#[diagnostic(severity(warning))]
struct EmptyTest(#[label] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-jest(prefer-todo): Suggest using `test.todo`.")]
#[diagnostic(severity(warning))]
struct UnImplementedTestDiagnostic(#[label] pub Span);

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
    style,
);

impl Rule for PreferTodo {
    fn run_once(&self, ctx: &LintContext) {
        for possible_jest_node in &collect_possible_jest_call_node(ctx) {
            run(possible_jest_node, ctx);
        }
    }
}

fn run<'a>(possible_jest_node: &PossibleJestNode<'a, '_>, ctx: &LintContext<'a>) {
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

        if counts == 1 && !filter_todo_case(call_expr) {
            let (content, span) = get_fix_content(call_expr);
            ctx.diagnostic_with_fix(UnImplementedTestDiagnostic(span), || Fix::new(content, span));
        }

        if counts > 1 && is_empty_function(call_expr) {
            ctx.diagnostic_with_fix(EmptyTest(call_expr.span), || {
                let (content, span) = build_code(call_expr, ctx);
                Fix::new(content, span)
            });
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
    match &expr.arguments[1] {
        Argument::Expression(Expression::ArrowFunctionExpression(arrow)) => arrow.body.is_empty(),
        Argument::Expression(Expression::FunctionExpression(func)) => {
            let Some(func_body) = &func.body else {
                return false;
            };
            func_body.is_empty()
        }
        _ => false,
    }
}

fn get_fix_content<'a>(expr: &'a CallExpression<'a>) -> (&'a str, Span) {
    match &expr.callee {
        Expression::Identifier(ident) => (".todo", Span::new(ident.span.end, ident.span.end)),
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
    let mut formatter = ctx.codegen();

    if let Expression::Identifier(ident) = &expr.callee {
        formatter.print_str(ident.name.as_bytes());
        formatter.print_str(b".todo(");
    } else if let Expression::MemberExpression(mem_expr) = &expr.callee {
        match &**mem_expr {
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

    (formatter.into_source_text(), expr.span)
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

    Tester::new(PreferTodo::NAME, pass, fail)
        .with_jest_plugin(true)
        .expect_fix(fix)
        .test_and_snapshot();
}
