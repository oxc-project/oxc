use oxc_ast::{
    AstKind,
    ast::{Argument, CallExpression, Expression, Statement},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{Atom, Span};

use crate::{context::LintContext, fixer::RuleFixer, rule::Rule, utils::get_node_name};

fn use_mock_shorthand(x0: &str, span1: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Prefer mock resolved/rejected shorthands for promises")
        .with_help(format!("Prefer {x0:?}"))
        .with_label(span1)
}

#[derive(Debug, Default, Clone)]
pub struct PreferMockPromiseShorthand;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// When working with mocks of functions that return promises, Jest provides some
    /// API sugar functions to reduce the amount of boilerplate you have to write.
    /// These methods should be preferred when possible.
    ///
    /// ### Why is this bad?
    ///
    /// Using generic mock functions like `mockImplementation(() => Promise.resolve())`
    /// or `mockReturnValue(Promise.reject())` is more verbose and less readable than
    /// Jest's specialized promise shorthands. The shorthand methods like
    /// `mockResolvedValue()` and `mockRejectedValue()` are more expressive and
    /// make the test intent clearer.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// jest.fn().mockImplementation(() => Promise.resolve(123));
    /// jest
    ///   .spyOn(fs.promises, 'readFile')
    ///   .mockReturnValue(Promise.reject(new Error('oh noes!')));
    ///
    /// myFunction
    ///   .mockReturnValueOnce(Promise.resolve(42))
    ///   .mockImplementationOnce(() => Promise.resolve(42))
    ///   .mockReturnValue(Promise.reject(new Error('too many calls!')));
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// jest.fn().mockResolvedValue(123);
    /// jest.spyOn(fs.promises, 'readFile').mockRejectedValue(new Error('oh noes!'));
    ///
    /// myFunction
    ///   .mockResolvedValueOnce(42)
    ///   .mockResolvedValueOnce(42)
    ///   .mockRejectedValue(new Error('too many calls!'));
    /// ```
    PreferMockPromiseShorthand,
    jest,
    style,
    conditional_fix
);

impl Rule for PreferMockPromiseShorthand {
    fn run<'a>(&self, node: &oxc_semantic::AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };
        let Some(mem_expr) = call_expr.callee.as_member_expression() else {
            return;
        };

        if call_expr.arguments.is_empty() {
            return;
        }

        let Some((property_span, property_name)) = mem_expr.static_property_info() else {
            return;
        };
        let Some(expr) = call_expr.arguments.first().and_then(Argument::as_expression) else {
            return;
        };
        let is_once = property_name.ends_with("Once");

        if property_name.eq("mockReturnValue") || property_name.eq("mockReturnValueOnce") {
            Self::report(is_once, property_span, None, expr, ctx);
        } else if property_name.eq("mockImplementation")
            || property_name.eq("mockImplementationOnce")
        {
            match expr {
                Expression::ArrowFunctionExpression(arrow_func) => {
                    if !arrow_func.params.is_empty() {
                        return;
                    }
                    let Some(stmt) = arrow_func.body.statements.first() else {
                        return;
                    };

                    if let Some(expr) = arrow_func.get_expression() {
                        Self::report(is_once, property_span, Some(arrow_func.span), expr, ctx);
                    } else if let Statement::ReturnStatement(return_stmt) = stmt {
                        let Some(arg_expr) = &return_stmt.argument else {
                            return;
                        };
                        Self::report(is_once, property_span, Some(arrow_func.span), arg_expr, ctx);
                    }
                }
                Expression::FunctionExpression(func_expr) => {
                    if !func_expr.params.is_empty() {
                        return;
                    }
                    let Some(func_body) = &func_expr.body else {
                        return;
                    };
                    let Some(stmt) = func_body.statements.first() else {
                        return;
                    };
                    let Statement::ReturnStatement(return_stmt) = stmt else {
                        return;
                    };
                    let Some(arg_expr) = &return_stmt.argument else {
                        return;
                    };
                    Self::report(is_once, property_span, Some(func_expr.span), arg_expr, ctx);
                }
                _ => (),
            }
        }
    }
}

impl PreferMockPromiseShorthand {
    fn report<'a>(
        is_once: bool,
        property_span: Span,
        arg_span: Option<Span>,
        arg_expr: &'a Expression<'a>,
        ctx: &LintContext<'a>,
    ) {
        let Expression::CallExpression(call_expr) = arg_expr else {
            return;
        };
        let arg_name = get_node_name(arg_expr);

        if !arg_name.eq("Promise.resolve") && !arg_name.eq("Promise.reject") {
            return;
        }

        let mock_promise_resolve =
            if is_once { "mockResolvedValueOnce" } else { "mockResolvedValue" };
        let mock_promise_reject =
            if is_once { "mockRejectedValueOnce" } else { "mockRejectedValue" };
        let prefer_name: &'static str =
            if arg_name.ends_with("reject") { mock_promise_reject } else { mock_promise_resolve };
        let fix_span = arg_span.unwrap_or(call_expr.span);

        // if arguments is more than one, just report it instead of fixing it.
        if call_expr.arguments.len() <= 1 {
            ctx.diagnostic_with_fix(
                use_mock_shorthand(Atom::from(prefer_name).as_str(), property_span),
                |fixer| {
                    let content = Self::fix(fixer, prefer_name, call_expr);
                    let span = Span::new(property_span.start, fix_span.end);
                    fixer.replace(span, content)
                },
            );
        } else {
            ctx.diagnostic(use_mock_shorthand(Atom::from(prefer_name).as_str(), property_span));
        }
    }

    fn fix<'a>(
        fixer: RuleFixer<'_, 'a>,
        prefer_name: &'a str,
        call_expr: &CallExpression<'a>,
    ) -> String {
        let mut content = fixer.codegen();
        content.print_str(prefer_name);
        content.print_ascii_byte(b'(');
        if call_expr.arguments.is_empty() {
            content.print_str("undefined");
        } else {
            for argument in &call_expr.arguments {
                if let Some(expr) = argument.as_expression() {
                    content.print_expression(expr);
                }
            }
        }
        content.into_source_text()
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let mut pass = vec![
        ("describe()", None),
        ("it()", None),
        ("describe.skip()", None),
        ("it.skip()", None),
        ("test()", None),
        ("test.skip()", None),
        ("var appliedOnly = describe.only; appliedOnly.apply(describe)", None),
        ("var calledOnly = it.only; calledOnly.call(it)", None),
        ("it.each()()", None),
        ("it.each`table`()", None),
        ("test.each()()", None),
        ("test.each`table`()", None),
        ("test.concurrent()", None),
        ("jest.fn().mockResolvedValue(42)", None),
        ("jest.fn(() => Promise.resolve(42))", None),
        ("jest.fn(() => Promise.reject(42))", None),
        ("aVariable.mockImplementation", None),
        ("aVariable.mockImplementation()", None),
        ("aVariable.mockImplementation([])", None),
        ("aVariable.mockImplementation(() => {})", None),
        ("aVariable.mockImplementation(() => [])", None),
        ("aVariable.mockReturnValue(() => Promise.resolve(1))", None),
        ("aVariable.mockReturnValue(Promise.resolve(1).then(() => 1))", None),
        ("aVariable.mockReturnValue(Promise.reject(1).then(() => 1))", None),
        ("aVariable.mockReturnValue(Promise.reject().then(() => 1))", None),
        ("aVariable.mockReturnValue(new Promise(resolve => resolve(1)))", None),
        ("aVariable.mockReturnValue(new Promise((_, reject) => reject(1)))", None),
        ("jest.spyOn(Thingy, 'method').mockImplementation(param => Promise.resolve(param));", None),
        (
            "
                aVariable.mockImplementation(() => {
                    const value = new Date();
                    return Promise.resolve(value);
                });
            ",
            None,
        ),
        (
            "
                aVariable.mockImplementation(() => {
                    return Promise.resolve(value)
                        .then(value => value + 1);
                });
            ",
            None,
        ),
        (
            "
                aVariable.mockImplementation(() => {
                    return Promise.all([1, 2, 3]);
                });
            ",
            None,
        ),
        ("aVariable.mockImplementation(() => Promise.all([1, 2, 3]));", None),
        ("aVariable.mockReturnValue(Promise.all([1, 2, 3]));", None),
    ];

    let mut fail = vec![
        ("aVariable.mockImplementation(() => Promise.reject(42, 25))", None),
        ("jest.fn().mockImplementation(() => Promise.reject(42))", None),
        ("aVariable.mockImplementation(() => Promise.resolve(42))", None),
        ("aVariable.mockImplementation(() => { return Promise.resolve(42); })", None),
        ("aVariable.mockImplementation(() => Promise.reject(42))", None),
        ("aVariable.mockImplementation(() => Promise.reject(42),)", None),
        ("aVariable.mockImplementationOnce(() => Promise.resolve(42))", None),
        ("aVariable.mockImplementationOnce(() => Promise.reject(42))", None),
        ("jest.fn().mockReturnValue(Promise.resolve(42))", None),
        ("jest.fn().mockReturnValue(Promise.reject(42))", None),
        ("aVariable.mockReturnValue(Promise.resolve(42))", None),
        ("aVariable.mockReturnValue(Promise.reject(42))", None),
        ("aVariable.mockReturnValueOnce(Promise.resolve(42))", None),
        ("aVariable.mockReturnValueOnce(Promise.reject(42))", None),
        (
            "
                aVariable.mockReturnValue(Promise.resolve({
                    target: 'world',
                    message: 'hello'
                }))
            ",
            None,
        ),
        (
            "
                aVariable
                    .mockImplementation(() => Promise.reject(42))
                    .mockImplementation(() => Promise.resolve(42))
                    .mockReturnValue(Promise.reject(42))
            ",
            None,
        ),
        (
            "
                aVariable
                    .mockReturnValueOnce(Promise.reject(42))
                    .mockImplementation(() => Promise.resolve(42))
                    .mockReturnValueOnce(Promise.reject(42))
            ",
            None,
        ),
        (
            "
                aVariable.mockReturnValueOnce(
                    Promise.reject(
                        new Error('oh noes!')
                    )
                )
            ",
            None,
        ),
        ("jest.fn().mockReturnValue(Promise.resolve(42), xyz)", None),
        ("jest.fn().mockImplementation(() => Promise.reject(42), xyz)", None),
        ("aVariable.mockReturnValueOnce(Promise.resolve(42, xyz))", None),
        ("aVariable.mockReturnValueOnce(Promise.resolve())", None),
        (
            "jest.spyOn(fs, \"readFile\").mockReturnValue(Promise.reject(new Error(\"oh noes!\")))",
            None,
        ),
    ];

    let mut fix = vec![
        (
            "jest.fn().mockImplementation(() => Promise.resolve(42))",
            "jest.fn().mockResolvedValue(42)",
            None,
        ),
        (
            "jest.fn().mockImplementation(() => Promise.reject(42))",
            "jest.fn().mockRejectedValue(42)",
            None,
        ),
        (
            "aVariable.mockImplementation(() => Promise.resolve(42))",
            "aVariable.mockResolvedValue(42)",
            None,
        ),
        (
            "aVariable.mockImplementation(() => {
                return Promise.resolve(42);
            });",
            "aVariable.mockResolvedValue(42);",
            None,
        ),
        (
            "aVariable.mockImplementation(() => Promise.reject(42))",
            "aVariable.mockRejectedValue(42)",
            None,
        ),
        (
            "aVariable.mockImplementation(() => Promise.reject(42),)",
            "aVariable.mockRejectedValue(42,)",
            None,
        ),
        (
            "aVariable.mockImplementationOnce(() => Promise.resolve(42))",
            "aVariable.mockResolvedValueOnce(42)",
            None,
        ),
        (
            "aVariable.mockImplementationOnce(() => Promise.reject(42))",
            "aVariable.mockRejectedValueOnce(42)",
            None,
        ),
        ("jest.fn().mockReturnValue(Promise.resolve(42))", "jest.fn().mockResolvedValue(42)", None),
        ("jest.fn().mockReturnValue(Promise.reject(42))", "jest.fn().mockRejectedValue(42)", None),
        ("aVariable.mockReturnValue(Promise.resolve(42))", "aVariable.mockResolvedValue(42)", None),
        ("aVariable.mockReturnValue(Promise.reject(42))", "aVariable.mockRejectedValue(42)", None),
        (
            "aVariable.mockReturnValueOnce(Promise.resolve(42))",
            "aVariable.mockResolvedValueOnce(42)",
            None,
        ),
        (
            "aVariable.mockReturnValueOnce(Promise.reject(42))",
            "aVariable.mockRejectedValueOnce(42)",
            None,
        ),
        // Todo: Fixed
        // (
        //     "aVariable.mockReturnValue(Promise.resolve({ target: 'world', message: 'hello' }))",
        //     "aVariable.mockResolvedValue({ target: 'world', message: 'hello' })",
        //     None,
        // ),
        (
            "
                aVariable
                    .mockImplementation(() => Promise.reject(42))
                    .mockImplementation(() => Promise.resolve(42))
                    .mockReturnValue(Promise.reject(42))
            ",
            "
                aVariable
                    .mockRejectedValue(42)
                    .mockResolvedValue(42)
                    .mockRejectedValue(42)
            ",
            None,
        ),
        (
            "
                aVariable
                    .mockReturnValueOnce(Promise.reject(42))
                    .mockImplementation(() => Promise.resolve(42))
                    .mockReturnValueOnce(Promise.reject(42))
            ",
            "
                aVariable
                    .mockRejectedValueOnce(42)
                    .mockResolvedValue(42)
                    .mockRejectedValueOnce(42)
            ",
            None,
        ),
        (
            "aVariable.mockReturnValueOnce(Promise.reject(new Error('oh noes!')))",
            "aVariable.mockRejectedValueOnce(new Error('oh noes!'))",
            None,
        ),
        (
            "jest.fn().mockReturnValue(Promise.resolve(42), xyz)",
            "jest.fn().mockResolvedValue(42, xyz)",
            None,
        ),
        (
            "jest.fn().mockImplementation(() => Promise.reject(42), xyz)",
            "jest.fn().mockRejectedValue(42, xyz)",
            None,
        ),
        (
            "aVariable.mockReturnValueOnce(Promise.resolve())",
            "aVariable.mockResolvedValueOnce(undefined)",
            None,
        ),
        // Todo: Fixed
        // (
        //     "jest.spyOn(fs, \"readFile\").mockReturnValue(Promise.reject(new Error(\"oh noes!\")))",
        //     "jest.spyOn(fs, \"readFile\").mockRejectedValue(new Error(\"oh noes!\"))",
        //     None,
        // ),
    ];

    let pass_vitest = vec![
        ("describe()", None),
        ("it()", None),
        ("describe.skip()", None),
        ("it.skip()", None),
        ("test()", None),
        ("test.skip()", None),
        ("var appliedOnly = describe.only; appliedOnly.apply(describe)", None),
        ("var calledOnly = it.only; calledOnly.call(it)", None),
        ("it.each()()", None),
        ("it.each`table`()", None),
        ("test.each()()", None),
        ("test.each`table`()", None),
        ("test.concurrent()", None),
        ("vi.fn().mockResolvedValue(42)", None),
        ("vi.fn(() => Promise.resolve(42))", None),
        ("vi.fn(() => Promise.reject(42))", None),
        ("aVariable.mockImplementation", None),
        ("aVariable.mockImplementation()", None),
        ("aVariable.mockImplementation([])", None),
        ("aVariable.mockImplementation(() => {})", None),
        ("aVariable.mockImplementation(() => [])", None),
        ("aVariable.mockReturnValue(() => Promise.resolve(1))", None),
        ("aVariable.mockReturnValue(Promise.resolve(1).then(() => 1))", None),
        ("aVariable.mockReturnValue(Promise.reject(1).then(() => 1))", None),
        ("aVariable.mockReturnValue(Promise.reject().then(() => 1))", None),
        ("aVariable.mockReturnValue(new Promise(resolve => resolve(1)))", None),
        ("aVariable.mockReturnValue(new Promise((_, reject) => reject(1)))", None),
        ("vi.spyOn(Thingy, 'method').mockImplementation(param => Promise.resolve(param));", None),
    ];

    let fail_vitest = vec![
        ("vi.fn().mockImplementation(() => Promise.resolve(42))", None),
        ("vi.fn().mockImplementation(() => Promise.reject(42))", None),
        ("aVariable.mockImplementation(() => Promise.resolve(42))", None),
        ("aVariable.mockImplementation(() => { return Promise.resolve(42) })", None),
        ("aVariable.mockImplementation(() => Promise.reject(42))", None),
        ("aVariable.mockImplementation(() => Promise.reject(42),)", None), // { "parserOptions": { "ecmaVersion": 2017 } },
        ("aVariable.mockImplementationOnce(() => Promise.resolve(42))", None),
        ("aVariable.mockImplementationOnce(() => Promise.reject(42))", None),
        ("vi.fn().mockReturnValue(Promise.resolve(42))", None),
        ("vi.fn().mockReturnValue(Promise.reject(42))", None),
        ("aVariable.mockReturnValue(Promise.resolve(42))", None),
        ("aVariable.mockReturnValue(Promise.reject(42))", None),
        ("aVariable.mockReturnValueOnce(Promise.resolve(42))", None),
        ("aVariable.mockReturnValueOnce(Promise.reject(42))", None),
        ("aVariable.mockReturnValue(Promise.resolve({ target: 'world', message: 'hello' }))", None),
        (
            "aVariable.mockImplementation(() => Promise.reject(42)).mockImplementation(() => Promise.resolve(42)).mockReturnValue(Promise.reject(42))",
            None,
        ),
        (
            "aVariable.mockReturnValueOnce(Promise.reject(42)).mockImplementation(() => Promise.resolve(42)).mockReturnValueOnce(Promise.reject(42))",
            None,
        ),
        ("aVariable.mockReturnValueOnce(Promise.reject(new Error('oh noes!')))", None),
        ("vi.fn().mockReturnValue(Promise.resolve(42), xyz)", None),
        ("vi.fn().mockImplementation(() => Promise.reject(42), xyz)", None),
        ("aVariable.mockReturnValueOnce(Promise.resolve(42, xyz))", None),
        ("aVariable.mockReturnValueOnce(Promise.resolve())", None),
    ];

    let fix_vitest = vec![
        (
            "vi.fn().mockImplementation(() => Promise.resolve(42))",
            "vi.fn().mockResolvedValue(42)",
            None,
        ),
        (
            "vi.fn().mockImplementation(() => Promise.reject(42))",
            "vi.fn().mockRejectedValue(42)",
            None,
        ),
        (
            "aVariable.mockImplementation(() => Promise.resolve(42))",
            "aVariable.mockResolvedValue(42)",
            None,
        ),
        (
            "aVariable.mockImplementation(() => { return Promise.resolve(42) })",
            "aVariable.mockResolvedValue(42)",
            None,
        ),
        (
            "aVariable.mockImplementation(() => Promise.reject(42))",
            "aVariable.mockRejectedValue(42)",
            None,
        ),
        (
            "aVariable.mockImplementation(() => Promise.reject(42),)",
            "aVariable.mockRejectedValue(42,)",
            None,
        ),
        (
            "aVariable.mockImplementationOnce(() => Promise.resolve(42))",
            "aVariable.mockResolvedValueOnce(42)",
            None,
        ),
        (
            "aVariable.mockImplementationOnce(() => Promise.reject(42))",
            "aVariable.mockRejectedValueOnce(42)",
            None,
        ),
        ("vi.fn().mockReturnValue(Promise.resolve(42))", "vi.fn().mockResolvedValue(42)", None),
        ("vi.fn().mockReturnValue(Promise.reject(42))", "vi.fn().mockRejectedValue(42)", None),
        ("aVariable.mockReturnValue(Promise.resolve(42))", "aVariable.mockResolvedValue(42)", None),
        ("aVariable.mockReturnValue(Promise.reject(42))", "aVariable.mockRejectedValue(42)", None),
        (
            "aVariable.mockReturnValueOnce(Promise.resolve(42))",
            "aVariable.mockResolvedValueOnce(42)",
            None,
        ),
        (
            "aVariable.mockReturnValueOnce(Promise.reject(42))",
            "aVariable.mockRejectedValueOnce(42)",
            None,
        ),
        // Todo: Fixed
        // (
        //     "aVariable.mockReturnValue(Promise.resolve({ target: 'world', message: 'hello' }))",
        //     "aVariable.mockResolvedValue({ target: 'world', message: 'hello' })",
        //     None,
        // ),
        (
            "aVariable.mockImplementation(() => Promise.reject(42)).mockImplementation(() => Promise.resolve(42)).mockReturnValue(Promise.reject(42))",
            "aVariable.mockRejectedValue(42).mockResolvedValue(42).mockRejectedValue(42)",
            None,
        ),
        (
            "aVariable.mockReturnValueOnce(Promise.reject(42)).mockImplementation(() => Promise.resolve(42)).mockReturnValueOnce(Promise.reject(42))",
            "aVariable.mockRejectedValueOnce(42).mockResolvedValue(42).mockRejectedValueOnce(42)",
            None,
        ),
        (
            "aVariable.mockReturnValueOnce(Promise.reject(new Error('oh noes!')))",
            "aVariable.mockRejectedValueOnce(new Error('oh noes!'))",
            None,
        ),
        (
            "vi.fn().mockReturnValue(Promise.resolve(42), xyz)",
            "vi.fn().mockResolvedValue(42, xyz)",
            None,
        ),
        (
            "vi.fn().mockImplementation(() => Promise.reject(42), xyz)",
            "vi.fn().mockRejectedValue(42, xyz)",
            None,
        ),
        (
            "aVariable.mockReturnValueOnce(Promise.resolve())",
            "aVariable.mockResolvedValueOnce(undefined)",
            None,
        ),
    ];

    pass.extend(pass_vitest);
    fail.extend(fail_vitest);
    fix.extend(fix_vitest);

    Tester::new(PreferMockPromiseShorthand::NAME, PreferMockPromiseShorthand::PLUGIN, pass, fail)
        .with_jest_plugin(true)
        .expect_fix(fix)
        .test_and_snapshot();
}
