use oxc_ast::{
    ast::{BindingPatternKind, FormalParameters, PropertyKey},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::AstNode;
use oxc_span::Span;

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{
        is_type_of_jest_fn_call, parse_expect_jest_fn_call, JestFnKind, JestGeneralFnKind,
        PossibleJestNode,
    },
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
        let mut function_args: Vec<String> = vec![];
        let nodes = ctx.nodes();

        for node in nodes.iter() {
            if let AstKind::Function(func) = node.kind() {
                Self::collect_functions_params(&func.params, &mut function_args);
            } else if let AstKind::ArrowFunctionExpression(arrow_func) = node.kind() {
                Self::collect_functions_params(&arrow_func.params, &mut function_args);
            }
        }

        for node in nodes.iter() {
            Self::check(node, &mut function_args, ctx);
        }
    }
}

impl RequireLocalTestContextForConcurrentSnapshots {
    fn collect_functions_params(params: &FormalParameters, function_args: &mut Vec<String>) {
        if !params.is_empty() {
            for params in &params.items {
                match &params.pattern.kind {
                    BindingPatternKind::BindingIdentifier(ident) => {
                        function_args.push(ident.name.to_string());
                    }
                    BindingPatternKind::ObjectPattern(obj_pat) => {
                        for prop in &obj_pat.properties {
                            if let PropertyKey::StaticIdentifier(ident) = &prop.key {
                                function_args.push(ident.name.to_string());
                            }
                        }
                    }
                    _ => (),
                }
            }
        }
    }

    fn check<'a>(node: &AstNode<'a>, function_args: &mut [String], ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };
        let Some(callee_name) = call_expr.callee_name() else {
            return;
        };
        let Some(expect_fn_call) =
            parse_expect_jest_fn_call(call_expr, &PossibleJestNode { node, original: None }, ctx)
        else {
            return;
        };

        if function_args.contains(&expect_fn_call.name.to_string()) {
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
        (r#"it.concurrent("something", ({ expect }) => { expect(1).toMatchSnapshot() })"#, None),
        (
            r#"it.concurrent("something", ({ expect }) => { expect(1).toMatchInlineSnapshot("1") })"#,
            None,
        ),
        (
            r#"describe.concurrent("something", () => { it("something", ({ expect }) => { expect(1).toMatchSnapshot() }) })"#,
            None,
        ),
        (
            r#"describe.concurrent("something", () => { it("something", ({ expect }) => { expect(1).toMatchInlineSnapshot() }) })"#,
            None,
        ),
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
