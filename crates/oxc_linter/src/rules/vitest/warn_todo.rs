use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{
        JestFnKind, JestGeneralFnKind, PossibleJestNode, collect_possible_jest_call_node,
        is_type_of_jest_fn_call, parse_general_jest_fn_call,
    },
};

fn warn_todo_diagnostic(span: Span) -> OxcDiagnostic {
    // See <https://oxc.rs/docs/contribute/linter/adding-rules.html#diagnostics> for details
    OxcDiagnostic::warn("The use of `.todo` is not recommended.")
        .with_help("Remove the `.todo` modifier before push your changes.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct WarnTodo;

// See <https://github.com/oxc-project/oxc/issues/6050> for documentation details.
declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule should be used to trigger warnings when .todo is used in describe, it, or test functions.
    /// It is recommended to use this with GitHub Actions to annotate PR diffs.
    ///
    /// ### Why is this bad?
    ///
    /// The test that your push should be completed, any pending code should not be commit.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// describe.todo('foo', () => {})
    /// it.todo('foo', () => {})
    /// test.todo('foo', () => {})
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// describe([])('foo', () => {})
    /// it([])('foo', () => {})
    /// test([])('foo', () => {})
    /// ```
    WarnTodo,
    vitest,
    correctness
);

impl Rule for WarnTodo {
    fn run_once(&self, ctx: &LintContext) {
        let mut possibles_jest_nodes = collect_possible_jest_call_node(ctx);
        possibles_jest_nodes.sort_unstable_by_key(|n| n.node.id());

        for possible_jest_node in &possibles_jest_nodes {
            self.run(possible_jest_node, ctx);
        }
    }
}

impl WarnTodo {
    fn run<'a>(&self, possible_jest_node: &PossibleJestNode<'a, '_>, ctx: &LintContext<'a>) {
        let node = possible_jest_node.node;
        if let AstKind::CallExpression(call_expr) = node.kind() {
            if !is_type_of_jest_fn_call(
                call_expr,
                possible_jest_node,
                ctx,
                &[
                    JestFnKind::General(JestGeneralFnKind::Describe),
                    JestFnKind::General(JestGeneralFnKind::Test),
                ],
            ) {
                return;
            };

            let Some(parsed_vi_fn_call) =
                parse_general_jest_fn_call(call_expr, possible_jest_node, ctx)
            else {
                return;
            };

            let warn_modifier =
                parsed_vi_fn_call.members.iter().find(|member| member.is_name_equal("todo"));

            if let Some(modifier) = warn_modifier {
                ctx.diagnostic(warn_todo_diagnostic(modifier.span));
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r#"describe("foo", function () {})"#,
        r#"it("foo", function () {})"#,
        r#"it.concurrent("foo", function () {})"#,
        r#"test("foo", function () {})"#,
        r#"test.concurrent("foo", function () {})"#,
        r#"describe.only("foo", function () {})"#,
        r#"it.only("foo", function () {})"#,
        r#"it.each()("foo", function () {})"#,
    ];

    let fail = vec![
        r#"describe.todo("foo", function () {})"#,
        r#"it.todo("foo", function () {})"#,
        r#"test.todo("foo", function () {})"#,
        r#"describe.todo.each([])("foo", function () {})"#,
        r#"it.todo.each([])("foo", function () {})"#,
        r#"test.todo.each([])("foo", function () {})"#,
        r#"describe.only.todo("foo", function () {})"#,
        r#"it.only.todo("foo", function () {})"#,
        r#"test.only.todo("foo", function () {})"#,
    ];

    Tester::new(WarnTodo::NAME, WarnTodo::PLUGIN, pass, fail)
        .with_vitest_plugin(true)
        .test_and_snapshot();
}
