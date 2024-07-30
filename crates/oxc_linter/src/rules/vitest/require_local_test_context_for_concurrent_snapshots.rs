use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::AstNode;
use oxc_span::Span;

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{is_type_of_jest_fn_call, JestFnKind, JestGeneralFnKind, PossibleJestNode},
};

fn require_local_test_context(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("require local Test Context for concurrent snapshot tests".to_string())
        .with_help("Use local Test Context instead")
        .with_label(span0)
}

#[derive(Debug, Default, Clone)]
pub struct RequireLocalTestContextForConcurrentSnapshots;

declare_oxc_lint!(
    /// ### Examples
    ///
    /// ```javascript
    /// // invalid
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
    /// ```javascript
    /// // valid
    /// test.concurrent('myLogic', ({ expect }) => {
    ///     expect(true).toMatchSnapshot();
    /// })
    ///
    /// test.concurrent('myLogic', (context) => {
    ///     context.expect(true).toMatchSnapshot();
    /// })
    /// ```
    RequireLocalTestContextForConcurrentSnapshots,
    style,
);

impl Rule for RequireLocalTestContextForConcurrentSnapshots {
    fn run_once(&self, ctx: &LintContext) {
        for node in ctx.nodes().iter() {
            Self::check(node, ctx);
        }
    }
}

impl RequireLocalTestContextForConcurrentSnapshots {
    fn check<'a>(node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };
        let Some(callee_name) = call_expr.callee_name() else {
            return;
        };
        let is_not_assertion = !is_type_of_jest_fn_call(
            call_expr,
            &PossibleJestNode { node, original: None },
            ctx,
            &[JestFnKind::Expect],
        );

        if is_not_assertion {
            return;
        }

        if ![
            "toMatchSnapshot",
            "toMatchInlineSnapshot",
            "toMatchFileSnapshot",
            "toThrowErrorMatchingSnapshot",
            "toThrowErrorMatchingInlineSnapshot",
        ]
        .contains(&callee_name)
        {
            return;
        }

        let mut is_inside_sequential_describe_or_test = false;

        for parent_node_id in ctx.nodes().ancestors(node.id()) {
            let parent_node = ctx.nodes().get_node(parent_node_id);
            if let AstKind::CallExpression(parent_call_expr) = parent_node.kind() {
                if let Some(callee_name) = parent_call_expr.callee_name() {
                    let is_describe_or_test = is_type_of_jest_fn_call(
                        parent_call_expr,
                        &PossibleJestNode { node: parent_node, original: None },
                        ctx,
                        &[
                            JestFnKind::General(JestGeneralFnKind::Describe),
                            JestFnKind::General(JestGeneralFnKind::Test),
                        ],
                    );

                    if is_describe_or_test
                        && parent_call_expr.callee.is_member_expression()
                        && callee_name.eq("concurrent")
                    {
                        is_inside_sequential_describe_or_test = true;
                        break;
                    }
                }
            }
        }

        if !is_inside_sequential_describe_or_test {
            return;
        }

        ctx.diagnostic(require_local_test_context(call_expr.span));
    }
}

#[test]
fn tests() {
    use crate::tester::Tester;

    let pass = vec![
        (r#"it("something", () => { expect(true).toBe(true) })"#, None),
        (r#"it.concurrent("something", () => { expect(true).toBe(true) })"#, None),
        (r#"it("something", () => { expect(1).toMatchSnapshot() })"#, None),
        // Todo: currently the expect will recognize expect is a jest function.
        // (r#"it.concurrent("something", ({ expect }) => { expect(1).toMatchSnapshot() })"#, None),
        // (
        //     r#"it.concurrent("something", ({ expect }) => { expect(1).toMatchInlineSnapshot("1") })"#,
        //     None,
        // ),
        // (
        //     r#"describe.concurrent("something", () => { it("something", ({ expect }) => { expect(1).toMatchSnapshot() }) })"#,
        //     None,
        // ),
        // (
        //     r#"describe.concurrent("something", () => { it("something", ({ expect }) => { expect(1).toMatchInlineSnapshot() }) })"#,
        //     None,
        // ),
        (
            r#"describe.concurrent("something", () => { it("something", () => { expect(true).toBe(true) }) })"#,
            None,
        ),
        (
            r#"describe("something", () => { it("something", (context) => { context.expect(1).toMatchInlineSnapshot() }) })"#,
            None,
        ),
        (
            r#"describe("something", () => { it("something", (context) => { expect(1).toMatchInlineSnapshot() }) })"#,
            None,
        ),
        (
            r#"it.concurrent("something", (context) => { context.expect(1).toMatchSnapshot() })"#,
            None,
        ),
    ];

    let fail = vec![
        (r#"it.concurrent("should fail", () => { expect(true).toMatchSnapshot() })"#, None),
        (
            r#"it.concurrent("should fail", () => { expect(true).toMatchInlineSnapshot("true") })"#,
            None,
        ),
        (
            r#"describe.concurrent("failing", () => { it("should fail", () => { expect(true).toMatchSnapshot() }) })"#,
            None,
        ),
        (
            r#"describe.concurrent("failing", () => { it("should fail", () => { expect(true).toMatchInlineSnapshot("true") }) })"#,
            None,
        ),
        (r#"it.concurrent("something", (context) => { expect(true).toMatchSnapshot() })"#, None),
        (
            r#"
                it.concurrent("something", () => {
                    expect(true).toMatchSnapshot();

                    expect(true).toMatchSnapshot();
                })
            "#,
            None,
        ),
        (
            r#"
                it.concurrent("something", () => {
                    expect(true).toBe(true);

                    expect(true).toMatchSnapshot();
                })
            "#,
            None,
        ),
        (
            r#"it.concurrent("should fail", () => { expect(true).toMatchFileSnapshot("./test/basic.output.html") })"#,
            None,
        ),
        (
            r#"it.concurrent("should fail", () => { expect(foo()).toThrowErrorMatchingSnapshot() })"#,
            None,
        ),
        (
            r#"it.concurrent("should fail", () => { expect(foo()).toThrowErrorMatchingInlineSnapshot("bar") })"#,
            None,
        ),
    ];

    Tester::new(RequireLocalTestContextForConcurrentSnapshots::NAME, pass, fail)
        .with_vitest_plugin(true)
        .test_and_snapshot();
}
