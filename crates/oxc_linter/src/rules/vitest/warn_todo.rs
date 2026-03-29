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
    /// This rule triggers warnings when `.todo` is used in `describe`, `it`, or `test` functions.
    /// It is recommended to use this with your CI pipeline to annotate PR diffs.
    ///
    /// ### Why is this bad?
    ///
    /// The test that you push should be completed, any pending/"TODO" code should not be committed.
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

    /*
     * Currently the responsible to set what frameworks are active or not is not `with_vitest_plugin` or oxlint config.
     * The code that set what test framewors are active is ContextHost::sniff_for_frameworks, and the current detection lead to a
     * a false negative. To detect if the current source code belongs to vitest is based if a `vitest` import exist, if not, assumes
     * we are on a possible jest test. On top of that, the method `frameworks::is_jestlike_file` most of the times is going to be true, at least in
     * our current situation. So this lead that the ContextHost can have jest and vitest active **at same time**.
     *
     * This detection isn't compatible on how `parse_general_jest_fn_call` handle if a node is valid or not. To make it simple:
     *
     * - Jest file: ctx.frameworks().is_jest() is true && ctx.frameworks().is_vitest() is false
     * - Vitest file: ctx.frameworks().is_jest() is true && ctx.frameworks().is_vitest is true
     *
     * And if you are dealing with non compatible modifiers or methods, that only exists in vitest, it will fail as in jest doesn't exist.
     *
     * In case of dealing with syntax that only exists in vitest, add an import of `vitest` to force the ContextHost to detect we are dealing with vitest.
     * This probably will allow reuse allow of the methods that rely on this false negative detection.
     */
    macro_rules! vitest_context {
        ($test: literal) => {
            concat!("import * as vi from 'vitest'\n\n", $test)
        };
    }

    let pass = vec![
        (vitest_context!(r#"describe("foo", function () {})"#)),
        (vitest_context!(r#"it("foo", function () {})"#)),
        (vitest_context!(r#"it.concurrent("foo", function () {})"#)),
        (vitest_context!(r#"test("foo", function () {})"#)),
        (vitest_context!(r#"test.concurrent("foo", function () {})"#)),
        (vitest_context!(r#"describe.only("foo", function () {})"#)),
        (vitest_context!(r#"it.only("foo", function () {})"#)),
        (vitest_context!(r#"it.each()("foo", function () {})"#)),
    ];

    let fail = vec![
        (vitest_context!(r#"describe.todo("foo", function () {})"#)),
        (vitest_context!(r#"it.todo("foo", function () {})"#)),
        (vitest_context!(r#"test.todo("foo", function () {})"#)),
        (vitest_context!(r#"describe.todo.each([])("foo", function () {})"#)),
        (vitest_context!(r#"it.todo.each([])("foo", function () {})"#)),
        (vitest_context!(r#"test.todo.each([])("foo", function () {})"#)),
        (vitest_context!(r#"describe.only.todo("foo", function () {})"#)),
        (vitest_context!(r#"it.only.todo("foo", function () {})"#)),
        (vitest_context!(r#"test.only.todo("foo", function () {})"#)),
    ];

    Tester::new(WarnTodo::NAME, WarnTodo::PLUGIN, pass, fail)
        .with_vitest_plugin(true)
        .test_and_snapshot();
}
