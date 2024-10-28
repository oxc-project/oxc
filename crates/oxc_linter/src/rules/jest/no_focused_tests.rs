use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{
        parse_general_jest_fn_call, JestFnKind, JestGeneralFnKind, MemberExpressionElement,
        ParsedGeneralJestFnCall, PossibleJestNode,
    },
};

fn no_focused_tests_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected focused test.")
        .with_help("Remove focus from test.")
        .with_label(span)
}

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
    ///
    /// This rule is compatible with [eslint-plugin-vitest](https://github.com/veritem/eslint-plugin-vitest/blob/main/docs/rules/no-focused-tests.md),
    /// to use it, add the following configuration to your `.eslintrc.json`:
    ///
    /// ```json
    /// {
    ///   "rules": {
    ///      "vitest/no-focused-tests": "error"
    ///   }
    /// }
    /// ```
    NoFocusedTests,
    correctness,
    fix
);

impl Rule for NoFocusedTests {
    fn run_on_jest_node<'a, 'c>(
        &self,
        jest_node: &PossibleJestNode<'a, 'c>,
        ctx: &'c LintContext<'a>,
    ) {
        run(jest_node, ctx);
    }
}

fn run<'a>(possible_jest_node: &PossibleJestNode<'a, '_>, ctx: &LintContext<'a>) {
    let node = possible_jest_node.node;
    let AstKind::CallExpression(call_expr) = node.kind() else {
        return;
    };
    let Some(jest_fn_call) = parse_general_jest_fn_call(call_expr, possible_jest_node, ctx) else {
        return;
    };
    let ParsedGeneralJestFnCall { kind, members, name, .. } = jest_fn_call;
    if !matches!(kind, JestFnKind::General(JestGeneralFnKind::Describe | JestGeneralFnKind::Test)) {
        return;
    }

    if name.starts_with('f') {
        ctx.diagnostic_with_fix(
            no_focused_tests_diagnostic(Span::new(
                call_expr.span.start,
                call_expr.span.start + u32::try_from(name.len()).unwrap_or(1),
            )),
            |fixer| fixer.delete_range(Span::sized(call_expr.span.start, 1)),
        );

        return;
    }

    let only_node = members.iter().find(|member| member.is_name_equal("only"));
    if let Some(only_node) = only_node {
        ctx.diagnostic_with_fix(no_focused_tests_diagnostic(only_node.span), |fixer| {
            let mut span = only_node.span.expand_left(1);
            if !matches!(only_node.element, MemberExpressionElement::IdentName(_)) {
                span = span.expand_right(1);
            }
            fixer.delete_range(span)
        });
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
    ];

    let mut fail = vec![
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

    let mut fix = vec![
        ("describe.only('foo', () => {})", "describe('foo', () => {})", None),
        ("describe['only']('foo', () => {})", "describe('foo', () => {})", None),
        ("fdescribe('foo', () => {})", "describe('foo', () => {})", None),
    ];

    let pass_vitest = vec![
        (r#"it("test", () => {});"#, None),
        (r#"describe("test group", () => {});"#, None),
        (r#"it("test", () => {});"#, None),
        (r#"describe("test group", () => {});"#, None),
    ];

    let fail_vitest = vec![
        (
            r#"
            import { it } from 'vitest'; 
            it.only("test", () => {});
            "#,
            None,
        ),
        (r#"describe.only("test", () => {});"#, None),
        (r#"test.only("test", () => {});"#, None),
        (r#"it.only.each([])("test", () => {});"#, None),
        (r#"test.only.each``("test", () => {});"#, None),
        (r#"it.only.each``("test", () => {});"#, None),
    ];

    let fix_vitest = vec![
        (r#"it.only("test", () => {});"#, r#"it("test", () => {});"#, None),
        (r#"describe.only("test", () => {});"#, r#"describe("test", () => {});"#, None),
        (r#"test.only("test", () => {});"#, r#"test("test", () => {});"#, None),
        (r#"it.only.each([])("test", () => {});"#, r#"it.each([])("test", () => {});"#, None),
        (r#"test.only.each``("test", () => {});"#, r#"test.each``("test", () => {});"#, None),
        (r#"it.only.each``("test", () => {});"#, r#"it.each``("test", () => {});"#, None),
    ];

    pass.extend(pass_vitest);
    fail.extend(fail_vitest);
    fix.extend(fix_vitest);

    Tester::new(NoFocusedTests::NAME, pass, fail)
        .with_jest_plugin(true)
        .expect_fix(fix)
        .test_and_snapshot();
}
