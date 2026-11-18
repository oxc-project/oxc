use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::AstNode;
use oxc_span::{GetSpan, Span};
use oxc_str::JSStr;

use crate::{
    context::{ContextHost, LintContext},
    rule::Rule,
    utils::{
        JestFnKind, JestGeneralFnKind, PossibleJestNode, is_type_of_jest_fn_call,
        parse_general_jest_fn_call,
    },
};

#[inline]
fn is_snapshot_method(property_name: &str) -> bool {
    matches!(
        property_name,
        "toMatchSnapshot"
            | "toMatchInlineSnapshot"
            | "toMatchFileSnapshot"
            | "toThrowErrorMatchingSnapshot"
            | "toThrowErrorMatchingInlineSnapshot"
    )
}

#[inline]
fn is_concurrent_test_or_describe<'a>(node: &AstNode<'a>, ctx: &LintContext<'a>) -> bool {
    let AstKind::CallExpression(call_expr) = node.kind() else { return false };
    let Some(jest_node) = PossibleJestNode::from_call_expression(node, call_expr, ctx) else {
        return false;
    };
    let Some(call) = parse_general_jest_fn_call(call_expr, &jest_node, ctx) else {
        return false;
    };

    matches!(call.kind, JestFnKind::General(JestGeneralFnKind::Describe | JestGeneralFnKind::Test))
        && call.members.iter().any(|member| member.is_name_equal("concurrent"))
}

fn require_local_test_context_for_concurrent_snapshots_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Require local Test Context for concurrent snapshot tests")
        .with_help("Use local Test Context instead")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct RequireLocalTestContextForConcurrentSnapshots;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// The rule is intended to ensure that concurrent snapshot tests are executed
    /// within a properly configured local test context.
    ///
    /// ### Why is this bad?
    ///
    /// Running snapshot tests concurrently without a proper context can lead to
    /// unreliable or inconsistent snapshots. Ensuring that concurrent tests are
    /// correctly configured with the appropriate context helps maintain accurate
    /// and stable snapshots, avoiding potential conflicts or failures.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// test.concurrent('myLogic', () => {
    ///     expect(true).toMatchSnapshot();
    /// })
    ///
    /// describe.concurrent('something', () => {
    ///     test('myLogic', () => {
    ///         expect(true).toMatchInlineSnapshot();
    ///     })
    /// })
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// test.concurrent('myLogic', ({ expect }) => {
    ///     expect(true).toMatchSnapshot();
    /// })
    ///
    /// test.concurrent('myLogic', (context) => {
    ///     context.expect(true).toMatchSnapshot();
    /// })
    /// ```
    RequireLocalTestContextForConcurrentSnapshots,
    vitest,
    correctness,
    recommended,
    version = "0.8.0",
    short_description = "The rule is intended to ensure that concurrent snapshot tests are executed within a properly configured local test context.",
);

impl Rule for RequireLocalTestContextForConcurrentSnapshots {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(_) = node.kind() else { return };
        Self::run(node, ctx);
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.frameworks().is_test()
    }
}

impl RequireLocalTestContextForConcurrentSnapshots {
    fn run<'a>(node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::CallExpression(call_expr) = node.kind() {
            let Some(possible_jest_node) =
                PossibleJestNode::from_call_expression(node, call_expr, ctx)
            else {
                return;
            };
            if !is_type_of_jest_fn_call(call_expr, &possible_jest_node, ctx, &[JestFnKind::Expect])
            {
                return;
            }

            let Some(member_expr) = call_expr.callee.as_member_expression() else { return };

            let Some(property_name) = member_expr.static_property_name().and_then(JSStr::as_str)
            else {
                return;
            };

            if !is_snapshot_method(property_name) {
                return;
            }

            let test_or_describe_node_found = ctx
                .nodes()
                .ancestors(node.id())
                .any(|node| is_concurrent_test_or_describe(node, ctx));

            if test_or_describe_node_found {
                ctx.diagnostic(require_local_test_context_for_concurrent_snapshots_diagnostic(
                    call_expr.span(),
                ));
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r#"it("something", () => { expect(true).toBe(true) })"#,
        r#"it.concurrent("something", () => { expect(true).toBe(true) })"#,
        r#"it("something", () => { expect(1).toMatchSnapshot() })"#,
        r#"it.concurrent("something", ({ expect }) => { expect(1).toMatchSnapshot() })"#,
        r#"it.concurrent("something", ({ expect }) => { expect(1).toMatchInlineSnapshot("1") })"#,
        r#"describe.concurrent("something", () => { it("something", () => { expect(true).toBe(true) }) })"#,
        r#"describe.concurrent("something", () => { it("something", ({ expect }) => { expect(1).toMatchSnapshot() }) })"#,
        r#"describe.concurrent("something", () => { it("something", ({ expect }) => { expect(1).toMatchInlineSnapshot() }) })"#,
        r#"describe("something", () => { it("something", (context) => { context.expect(1).toMatchInlineSnapshot() }) })"#,
        r#"describe("something", () => { it("something", (context) => { expect(1).toMatchInlineSnapshot() }) })"#,
        r#"it.concurrent("something", (context) => { context.expect(1).toMatchSnapshot() })"#,
        r"import { test } from 'vitest';
            const customTest = test.extend({});
            customTest.concurrent('foo', ({ expect }) => {
                expect(true).toMatchSnapshot();
            });",
        r"import { test as baseTest } from 'vitest';
            const customTest = baseTest.extend({});
            customTest.concurrent('foo', context => {
                context.expect(true).toMatchSnapshot();
            });",
        r"import { test } from 'vitest';
            const customTest = test.extend({});
            customTest('foo', () => { expect(true).toMatchSnapshot(); });",
        r"import { test as baseTest } from 'other';
            const customTest = baseTest.extend({});
            customTest.concurrent('foo', () => { expect(true).toMatchSnapshot(); });",
        r"function test() {}
            test.concurrent('foo', () => { expect(true).toMatchSnapshot(); });",
        r"import { test } from 'vitest';
            const customTest = test.extend({});
            function helper(customTest) {
                customTest.concurrent('foo', () => { expect(true).toMatchSnapshot(); });
            }",
    ];

    let fail = vec![
        r#"it.concurrent("should fail", () => { expect(true).toMatchSnapshot() })"#,
        r#"it.concurrent("should fail", () => { expect(true).toMatchInlineSnapshot("true") })"#,
        r#"describe.concurrent("failing", () => { it("should fail", () => { expect(true).toMatchSnapshot() }) })"#,
        r#"describe.concurrent("failing", () => { it("should fail", () => { expect(true).toMatchInlineSnapshot("true") }) })"#,
        r#"it.concurrent("something", (context) => { expect(true).toMatchSnapshot() })"#,
        r#"it.concurrent("something", () => {
                        expect(true).toMatchSnapshot();

                         expect(true).toMatchSnapshot();
                    })"#,
        r#"it.concurrent("something", () => {
                         expect(true).toBe(true);

                         expect(true).toMatchSnapshot();
                    })"#,
        r#"it.concurrent("should fail", () => { expect(true).toMatchFileSnapshot("./test/basic.output.html") })"#,
        r#"it.concurrent("should fail", () => { expect(foo()).toThrowErrorMatchingSnapshot() })"#,
        r#"it.concurrent("should fail", () => { expect(foo()).toThrowErrorMatchingInlineSnapshot("bar") })"#,
        r"import { test as customTest } from 'vitest';
            customTest.concurrent('foo', () => { expect(true).toMatchSnapshot(); });",
        r"import { test } from 'vitest';
            const customTest = test.extend({});
            customTest.concurrent('foo', () => { expect(true).toMatchSnapshot(); });",
        r"import { test as baseTest } from 'vitest';
            const extendedTest = baseTest.extend({});
            const customTest = extendedTest.extend({});
            customTest.concurrent.only('foo', () => {
                expect(true).toMatchInlineSnapshot();
            });",
        r"import { test } from 'vitest';
            const customTest = test.extend({});
            customTest.concurrent.each([1])('foo', () => {
                expect(true).toMatchSnapshot();
            });",
        r"import { describe as customDescribe, test } from 'vitest';
            const customTest = test.extend({});
            customDescribe.concurrent('suite', () => {
                customTest('foo', () => { expect(true).toMatchSnapshot(); });
            });",
        r"import { test } from 'vitest';
            const customTest = test.extend({});
            customTest.only.concurrent('foo', () => {
                expect(true).toMatchSnapshot();
            });",
    ];

    Tester::new(
        RequireLocalTestContextForConcurrentSnapshots::NAME,
        RequireLocalTestContextForConcurrentSnapshots::PLUGIN,
        pass,
        fail,
    )
    .with_vitest_plugin(true)
    .test_and_snapshot();
}
