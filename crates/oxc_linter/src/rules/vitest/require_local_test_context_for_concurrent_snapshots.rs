use oxc_ast::{ast::MemberExpression, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{is_type_of_jest_fn_call, JestFnKind, JestGeneralFnKind, PossibleJestNode},
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
fn is_test_or_describe_node(member_expr: &MemberExpression) -> bool {
    if let Some(id) = member_expr.object().get_identifier_reference() {
        if matches!(
            JestFnKind::from(id.name.as_str()),
            JestFnKind::General(JestGeneralFnKind::Describe | JestGeneralFnKind::Test)
        ) {
            if let Some(property_name) = member_expr.static_property_name() {
                return property_name == "concurrent";
            }
        }
    }
    false
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
    ///
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
    correctness,
    pending
);

impl Rule for RequireLocalTestContextForConcurrentSnapshots {
    fn run_on_jest_node<'a, 'c>(
        &self,
        jest_node: &PossibleJestNode<'a, 'c>,
        ctx: &'c LintContext<'a>,
    ) {
        Self::run(jest_node, ctx);
    }
}

impl RequireLocalTestContextForConcurrentSnapshots {
    fn run<'a>(possible_jest_node: &PossibleJestNode<'a, '_>, ctx: &LintContext<'a>) {
        let node = possible_jest_node.node;
        if let AstKind::CallExpression(call_expr) = node.kind() {
            if !is_type_of_jest_fn_call(call_expr, possible_jest_node, ctx, &[JestFnKind::Expect]) {
                return;
            }

            let Some(member_expr) = call_expr.callee.as_member_expression() else { return };

            let Some(property_name) = member_expr.static_property_name() else { return };

            if !is_snapshot_method(property_name) {
                return;
            }

            let test_or_describe_node_found =
                ctx.nodes().iter_parents(possible_jest_node.node.id()).any(|node| {
                    if let AstKind::CallExpression(ancestor_call_expr) = node.kind() {
                        if let Some(ancestor_member_expr) =
                            ancestor_call_expr.callee.as_member_expression()
                        {
                            return is_test_or_describe_node(ancestor_member_expr);
                        }
                    }

                    false
                });

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
    ];

    Tester::new(RequireLocalTestContextForConcurrentSnapshots::NAME, pass, fail)
        .with_vitest_plugin(true)
        .test_and_snapshot();
}
