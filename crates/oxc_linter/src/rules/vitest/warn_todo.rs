use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{
        JestFnKind, JestGeneralFnKind, PossibleJestNode, is_type_of_jest_fn_call,
        parse_general_jest_fn_call,
    },
};

fn warn_todo_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("The use of `.todo` is not recommended.")
        .with_help("Write an actual test and remove the `.todo` modifier before pushing/merging your changes.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct WarnTodo;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule warns about usage of `.todo` in `describe`, `it`, or `test` functions.
    ///
    /// ### Why is this bad?
    ///
    /// The tests you push should be complete. Any pending/`TODO` code should not be committed.
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
    correctness,
    version = "1.37.0",
);

impl Rule for WarnTodo {
    fn run_on_jest_node<'a, 'c>(
        &self,
        jest_node: &PossibleJestNode<'a, 'c>,
        ctx: &'c LintContext<'a>,
    ) {
        WarnTodo::run(jest_node, ctx);
    }
}

impl WarnTodo {
    fn run<'a>(possible_jest_node: &PossibleJestNode<'a, '_>, ctx: &LintContext<'a>) {
        if let AstKind::CallExpression(call_expr) = possible_jest_node.node.kind() {
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
            }

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
        (r#"describe("foo", function () {})"#),
        (r#"it("foo", function () {})"#),
        (r#"it.concurrent("foo", function () {})"#),
        (r#"test("foo", function () {})"#),
        (r#"test.concurrent("foo", function () {})"#),
        (r#"describe.only("foo", function () {})"#),
        (r#"it.only("foo", function () {})"#),
        (r#"it.each()("foo", function () {})"#),
    ];

    let fail = vec![
        (r#"describe.todo("foo", function () {})"#),
        (r#"it.todo("foo", function () {})"#),
        (r#"test.todo("foo", function () {})"#),
        (r#"describe.todo.each([])("foo", function () {})"#),
        (r#"it.todo.each([])("foo", function () {})"#),
        (r#"test.todo.each([])("foo", function () {})"#),
        (r#"describe.only.todo("foo", function () {})"#),
        (r#"it.only.todo("foo", function () {})"#),
        (r#"test.only.todo("foo", function () {})"#),
        // Issue #20955
        r#"import { test as vpTest } from "vite-plus/test";
        vpTest.todo(
          "vite-plus/test does not have expected vitest/warn-todo lint error",
          () => {},
        );
        "#,
    ];

    Tester::new(WarnTodo::NAME, WarnTodo::PLUGIN, pass, fail)
        .with_vitest_plugin(true)
        .test_and_snapshot();
}
