use oxc_ast::{ast::Expression, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{
        collect_possible_jest_call_node, parse_general_jest_fn_call, JestGeneralFnKind,
        KnownMemberExpressionProperty, ParsedGeneralJestFnCall, PossibleJestNode,
    },
};

fn no_test_prefixes_diagnostic(x1: &str, span2: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Use {x1:?} instead.")).with_label(span2)
}

#[derive(Debug, Default, Clone)]
pub struct NoTestPrefixes;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Require using `.only` and `.skip` over `f` and `x`.
    ///
    /// ### Why is this bad?
    ///
    /// Jest allows you to choose how you want to define focused and skipped tests,
    /// with multiple permutations for each:
    /// - only & skip: it.only, test.only, describe.only, it.skip, test.skip, describe.skip.
    /// - 'f' & 'x': fit, fdescribe, xit, xtest, xdescribe.
    ///
    /// This rule enforces usages from the only & skip list.
    ///
    /// ### Example
    /// ```javascript
    /// fit('foo'); // invalid
    /// fdescribe('foo'); // invalid
    /// xit('foo'); // invalid
    /// xtest('foo'); // invalid
    /// xdescribe('foo'); // invalid
    /// ```
    ///
    /// This rule is compatible with [eslint-plugin-vitest](https://github.com/veritem/eslint-plugin-vitest/blob/main/docs/rules/no-test-prefixes.md),
    /// to use it, add the following configuration to your `.eslintrc.json`:
    ///
    /// ```json
    /// {
    ///   "rules": {
    ///      "vitest/no-test-prefixes": "error"
    ///   }
    /// }
    /// ```
    NoTestPrefixes,
    style,
    fix
);

impl Rule for NoTestPrefixes {
    fn run_once(&self, ctx: &LintContext) {
        for node in &collect_possible_jest_call_node(ctx) {
            run(node, ctx);
        }
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
    let ParsedGeneralJestFnCall { kind, name, .. } = &jest_fn_call;
    let Some(kind) = kind.to_general() else {
        return;
    };

    if !matches!(kind, JestGeneralFnKind::Describe | JestGeneralFnKind::Test) {
        return;
    }

    if !name.starts_with('f') && !name.starts_with('x') {
        return;
    }

    let span = match &call_expr.callee {
        Expression::TaggedTemplateExpression(tagged_template_expr) => {
            tagged_template_expr.tag.span()
        }
        Expression::CallExpression(child_call_expr) => child_call_expr.callee.span(),
        _ => call_expr.callee.span(),
    };

    let preferred_node_name = get_preferred_node_names(&jest_fn_call);

    ctx.diagnostic_with_fix(no_test_prefixes_diagnostic(&preferred_node_name, span), |fixer| {
        fixer.replace(span, preferred_node_name)
    });
}

fn get_preferred_node_names(jest_fn_call: &ParsedGeneralJestFnCall) -> String {
    let ParsedGeneralJestFnCall { members, name, .. } = jest_fn_call;

    let preferred_modifier = if name.starts_with('f') { "only" } else { "skip" };
    let member_names = members
        .iter()
        .filter_map(KnownMemberExpressionProperty::name)
        .collect::<Vec<_>>()
        .join(".");
    let name_slice = &name[1..];

    if member_names.is_empty() {
        format!("{name_slice}.{preferred_modifier}")
    } else {
        format!("{name_slice}.{preferred_modifier}.{member_names}")
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let mut pass = vec![
        ("describe('foo', function () {})", None),
        ("it('foo', function () {})", None),
        ("it.concurrent('foo', function () {})", None),
        ("test('foo', function () {})", None),
        ("test.concurrent('foo', function () {})", None),
        ("describe.only('foo', function () {})", None),
        ("it.only('foo', function () {})", None),
        ("it.each()('foo', function () {})", None),
        ("it.each``('foo', function () {})", None),
        ("test.only('foo', function () {})", None),
        ("test.each()('foo', function () {})", None),
        ("test.each``('foo', function () {})", None),
        ("describe.skip('foo', function () {})", None),
        ("it.skip('foo', function () {})", None),
        ("test.skip('foo', function () {})", None),
        ("foo()", None),
        ("[1,2,3].forEach()", None),
    ];

    let mut fail = vec![
        ("fdescribe('foo', function () {})", None),
        ("xdescribe.each([])('foo', function () {})", None),
        ("fit('foo', function () {})", None),
        ("xdescribe('foo', function () {})", None),
        ("xit('foo', function () {})", None),
        ("xtest('foo', function () {})", None),
        ("xit.each``('foo', function () {})", None),
        ("xtest.each``('foo', function () {})", None),
        ("xit.each([])('foo', function () {})", None),
        ("xtest.each([])('foo', function () {})", None),
        (
            "
                import { xit } from '@jest/globals';
                xit('foo', function () {})
            ",
            None,
        ),
        (
            "
                import { xit as skipThis } from '@jest/globals';
                skipThis('foo', function () {})
            ",
            None,
        ),
        (
            "
                import { fit as onlyThis } from '@jest/globals';
                onlyThis('foo', function () {})
            ",
            None,
        ),
    ];

    let pass_vitest = vec![
        ("describe(\"foo\", function () {})", None),
        ("it(\"foo\", function () {})", None),
        ("it.concurrent(\"foo\", function () {})", None),
        ("test(\"foo\", function () {})", None),
        ("test.concurrent(\"foo\", function () {})", None),
        ("describe.only(\"foo\", function () {})", None),
        ("it.only(\"foo\", function () {})", None),
        ("it.each()(\"foo\", function () {})", None),
    ];

    let fail_vitest = vec![
        ("fdescribe(\"foo\", function () {})", None),
        ("xdescribe.each([])(\"foo\", function () {})", None),
        ("fit(\"foo\", function () {})", None),
        ("xdescribe(\"foo\", function () {})", None),
        ("xit(\"foo\", function () {})", None),
        ("xtest(\"foo\", function () {})", None),
        ("xit.each``(\"foo\", function () {})", None),
        ("xtest.each``(\"foo\", function () {})", None),
        ("xit.each([])(\"foo\", function () {})", None),
        ("xtest.each([])(\"foo\", function () {})", None),
    ];

    pass.extend(pass_vitest);
    fail.extend(fail_vitest);

    let fix = vec![
        ("xdescribe('foo', () => {})", "describe.skip('foo', () => {})"),
        ("fdescribe('foo', () => {})", "describe.only('foo', () => {})"),
        ("xtest('foo', () => {})", "test.skip('foo', () => {})"),
        // NOTE(@DonIsaac): is this intentional?
        // ("ftest('foo', () => {})", "test.only('foo', () => {})"),
        ("xit('foo', () => {})", "it.skip('foo', () => {})"),
        ("fit('foo', () => {})", "it.only('foo', () => {})"),
    ];

    Tester::new(NoTestPrefixes::NAME, pass, fail)
        .with_jest_plugin(true)
        .with_vitest_plugin(true)
        .expect_fix(fix)
        .test_and_snapshot();
}
