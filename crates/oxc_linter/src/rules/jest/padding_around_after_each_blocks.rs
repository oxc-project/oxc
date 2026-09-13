use oxc_ast::AstKind;
use oxc_macros::declare_oxc_lint;

use crate::utils::{
    JestGeneralFnKind, ParsedGeneralJestFnCall, parse_general_jest_fn_call,
    report_missing_padding_after_jest_block, report_missing_padding_before_jest_block,
};
use crate::{
    context::LintContext,
    rule::Rule,
    utils::PossibleJestNode,
};

#[derive(Debug, Default, Clone)]
pub struct PaddingAroundAfterEachBlocks;

// See <https://github.com/oxc-project/oxc/issues/6050> for documentation details.
declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule enforces a line of padding before and after 1 or more
    /// `afterEach` statements.
    ///
    /// ### Why is this bad?
    ///
    /// Inconsistent formatting of code can make the code more difficult to read
    /// and follow. This rule helps ensure that `afterEach` blocks are visually
    /// separated from the rest of the code, making them easier to identify while
    /// looking through test files.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// const something = 123;
    /// afterEach(() => {
    ///   // more stuff
    /// });
    /// describe('foo', () => {});
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// const something = 123;
    ///
    /// afterEach(() => {
    ///   // more stuff
    /// });
    ///
    /// describe('foo', () => {});
    /// ```
    PaddingAroundAfterEachBlocks,
    jest,
    style,
    fix,
    version = "next",
    short_description = "Enforce padding around afterEach blocks.",
);

impl Rule for PaddingAroundAfterEachBlocks {
    fn run_on_jest_node<'a, 'c>(
        &self,
        possible_jest_node: &PossibleJestNode<'a, 'c>,
        ctx: &'c LintContext<'a>,
    ) {
        let node = possible_jest_node.node;
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };
        let Some(jest_fn_call) = parse_general_jest_fn_call(call_expr, possible_jest_node, ctx)
        else {
            return;
        };
        let ParsedGeneralJestFnCall { kind, name, .. } = &jest_fn_call;
        let Some(kind) = kind.to_general() else {
            return;
        };
        if kind != JestGeneralFnKind::Hook {
            return;
        }
        if name != "afterEach" {
            return;
        }
        report_missing_padding_before_jest_block(node, ctx, name);
        report_missing_padding_after_jest_block(node, ctx, name);
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "const something = 123;\n\nafterEach(() => {\n//  // more stuff\n});\n\ndescribe('foo', () => {});",
        "const something = 123;\n\nafterEach(() => {\n//  // more stuff\n});",
    ];

    let fail = vec![
        "const something = 123;\nafterEach(() => {\n//  // more stuff\n});\ndescribe('foo', () => {});",
        "const something = 123;\n\nafterEach(() => {\n//  // more stuff\n});\ndescribe('foo', () => {});",
        "const something = 123;\nafterEach(() => {\n//  // more stuff\n});\n\ndescribe('foo', () => {});",
    ];

    let fix = vec![(
        "const something = 123;\nafterEach(() => {\n//  // more stuff\n});\ndescribe('foo', () => {});",
        "const something = 123;\n\nafterEach(() => {\n//  // more stuff\n});\n\ndescribe('foo', () => {});"
    ), (
        "
            const someText = 'abc';
            afterEach(() => {
            });
            describe('someText', () => {
              const something = 'abc';
              // A comment
              afterEach(() => {
                // stuff
              });
              afterEach(() => {
                // other stuff
              });
            });
            describe('someText', () => {
              const something = 'abc';
              afterEach(() => {
                // stuff
              });
            });
        ",
        "
            const someText = 'abc';

            afterEach(() => {
            });

            describe('someText', () => {
              const something = 'abc';

              // A comment
              afterEach(() => {
                // stuff
              });

              afterEach(() => {
                // other stuff
              });
            });
            describe('someText2', () => {
              const something = 'xyz';

              afterEach(() => {
                // stuff
              });
            });
        "
        )];

    Tester::new(
        PaddingAroundAfterEachBlocks::NAME,
        PaddingAroundAfterEachBlocks::PLUGIN,
        pass,
        fail,
    )
    .with_jest_plugin(true)
    .expect_fix(fix)
    .test_and_snapshot();
}
