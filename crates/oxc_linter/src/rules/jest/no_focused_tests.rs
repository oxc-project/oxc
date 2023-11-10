use oxc_ast::AstKind;
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
        parse_general_jest_fn_call, JestFnKind, JestGeneralFnKind, MemberExpressionElement,
        ParsedGeneralJestFnCall,
    },
    AstNode,
};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-jest(no-focused-tests): Unexpected focused test.")]
#[diagnostic(severity(warning), help("Remove focus from test."))]
struct NoFocusedTestsDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoFocusedTests;

declare_oxc_lint!(
    /// ### What it does
    /// This rule reminds you to remove `.only` from your tests by raising a warning
    /// whenever you are using the exclusivity feature.
    ///
    /// ### Why is this bad?
    ///
    /// Jest has a feature that allows you to focus tests by appending `.only` or
    /// prepending `f` to a test-suite or a test-case. This feature is really helpful to
    /// debug a failing test, so you don’t have to execute all of your tests. After you
    /// have fixed your test and before committing the changes you have to remove
    /// `.only` to ensure all tests are executed on your build system.
    ///
    /// ### Example
    ///
    /// ```javascript
    /// describe.only('foo', () => {});
    /// it.only('foo', () => {});
    /// describe['only']('bar', () => {});
    /// it['only']('bar', () => {});
    /// test.only('foo', () => {});
    /// test['only']('bar', () => {});
    /// fdescribe('foo', () => {});
    /// fit('foo', () => {});
    /// fit.each`
    /// table
    /// `();
    /// ```
    NoFocusedTests,
    correctness
);

impl Rule for NoFocusedTests {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else { return };
        let Some(jest_fn_call) = parse_general_jest_fn_call(call_expr, node, ctx) else { return };
        let ParsedGeneralJestFnCall { kind, members, name } = jest_fn_call;
        if !matches!(
            kind,
            JestFnKind::General(JestGeneralFnKind::Describe | JestGeneralFnKind::Test)
        ) {
            return;
        }

        if name.starts_with('f') {
            ctx.diagnostic_with_fix(NoFocusedTestsDiagnostic(call_expr.span), || {
                let start = call_expr.span.start;
                Fix::delete(Span { start, end: start + 1 })
            });

            return;
        }

        let only_node = members.iter().find(|member| member.is_name_equal("only"));
        if let Some(only_node) = only_node {
            ctx.diagnostic_with_fix(NoFocusedTestsDiagnostic(call_expr.span), || {
                let span = only_node.span;
                let start = span.start - 1;
                let end = if matches!(only_node.element, MemberExpressionElement::IdentName(_)) {
                    span.end
                } else {
                    span.end + 1
                };
                Fix::delete(Span { start, end })
            });
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
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
    ];

    let fail = vec![
        ("describe.only()", None),
        // TODO: this need set setting like `settings: { jest: { globalAliases: { describe: ['context'] } } },`
        // ("context.only()", None),
        ("describe.only.each()()", None),
        ("describe.only.each`table`()", None),
        ("describe[\"only\"]()", None),
        ("it.only()", None),
        ("it.concurrent.only.each``()", None),
        ("it.only.each()()", None),
        ("it.only.each`table`()", None),
        ("it[\"only\"]()", None),
        ("test.only()", None),
        ("test.concurrent.only.each()()", None),
        ("test.only.each()()", None),
        ("test.only.each`table`()", None),
        ("test[\"only\"]()", None),
        ("fdescribe()", None),
        ("fit()", None),
        ("fit.each()()", None),
        ("fit.each`table`()", None),
    ];

    let fix = vec![
        ("describe.only('foo', () => {})", "describe('foo', () => {})", None),
        ("describe['only']('foo', () => {})", "describe('foo', () => {})", None),
        ("fdescribe('foo', () => {})", "describe('foo', () => {})", None),
    ];

    Tester::new(NoFocusedTests::NAME, pass, fail)
        .with_jest_plugin(true)
        .expect_fix(fix)
        .test_and_snapshot();
}
