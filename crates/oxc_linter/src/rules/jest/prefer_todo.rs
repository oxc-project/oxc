use oxc_ast::{
    ast::{Argument, CallExpression, Expression},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    context::LintContext,
    fixer::{RuleFix, RuleFixer},
    rule::Rule,
    utils::{
        collect_possible_jest_call_node, is_type_of_jest_fn_call, JestFnKind, JestGeneralFnKind,
        PossibleJestNode,
    },
};

fn empty_test(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Suggest using `test.todo`.").with_label(span)
}

fn un_implemented_test_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Suggest using `test.todo`.").with_label(span)
}

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
    fix
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
            let span = call_expr
                .callee
                .as_member_expression()
                .map_or(call_expr.callee.span(), GetSpan::span);
            ctx.diagnostic_with_fix(un_implemented_test_diagnostic(span), |fixer| {
                if let Expression::Identifier(ident) = &call_expr.callee {
                    return fixer.replace(Span::empty(ident.span.end), ".todo");
                }
                if let Some(mem_expr) = call_expr.callee.as_member_expression() {
                    if let Some((span, _)) = mem_expr.static_property_info() {
                        return fixer.replace(span, "todo");
                    }
                }
                fixer.delete_range(call_expr.span)
            });
        }

        if counts > 1 && is_empty_function(call_expr) {
            ctx.diagnostic_with_fix(empty_test(call_expr.span), |fixer| {
                build_code(fixer, call_expr)
            });
        }
    }
}

fn filter_todo_case(expr: &CallExpression) -> bool {
    if let Some(mem_expr) = expr.callee.as_member_expression() {
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

fn build_code<'a>(fixer: RuleFixer<'_, 'a>, expr: &CallExpression<'a>) -> RuleFix<'a> {
    let mut formatter = fixer.codegen();

    match &expr.callee {
        Expression::Identifier(ident) => {
            formatter.print_str(ident.name.as_str());
            formatter.print_str(".todo(");
        }
        Expression::ComputedMemberExpression(expr) => {
            if let Expression::Identifier(ident) = &expr.object {
                formatter.print_str(ident.name.as_str());
                formatter.print_str("[");
                formatter.print_str("'todo'");
                formatter.print_str("](");
            }
        }
        Expression::StaticMemberExpression(expr) => {
            if let Expression::Identifier(ident) = &expr.object {
                formatter.print_str(ident.name.as_str());
                formatter.print_str(".todo(");
            }
        }
        _ => {}
    }

    if let Argument::StringLiteral(ident) = &expr.arguments[0] {
        // Todo: this punctuation should read from the config
        formatter.print_ascii_byte(b'\'');
        formatter.print_str(ident.value.as_str());
        formatter.print_ascii_byte(b'\'');
        formatter.print_ascii_byte(b')');
    } else if let Argument::TemplateLiteral(temp) = &expr.arguments[0] {
        formatter.print_ascii_byte(b'`');
        for q in &temp.quasis {
            formatter.print_str(q.value.raw.as_str());
        }
        formatter.print_ascii_byte(b'`');
        formatter.print_ascii_byte(b')');
    }

    fixer.replace(expr.span, formatter)
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
