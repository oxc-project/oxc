use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_empty_static_block_diagnostic(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("eslint(no-empty-static-block): Disallow empty static blocks")
        .with_help("Unexpected empty static block.")
        .with_label(span0)
}

#[derive(Debug, Default, Clone)]
pub struct NoEmptyStaticBlock;

declare_oxc_lint!(
    /// ### What it does
    /// Disallows the usages of empty static blocks
    ///
    /// ### Why is this bad?
    /// Empty block statements, while not technically errors, usually occur due to refactoring that wasn’t completed.
    /// They can cause confusion when reading code.
    ///
    /// ### Example
    /// ```javascript
    ///
    /// class Foo {
    ///     static {
    ///     }
    /// }
    ///
    /// ```
    NoEmptyStaticBlock,
    correctness
);

impl Rule for NoEmptyStaticBlock {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::StaticBlock(static_block) = node.kind() {
            if static_block.body.is_empty() {
                if ctx.semantic().trivias().has_comments_between(static_block.span) {
                    return;
                }
                ctx.diagnostic(no_empty_static_block_diagnostic(static_block.span));
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "class Foo { static { bar(); } }",
        "class Foo { static { /* comments */ } }",
        "class Foo { static {
			// comment
			} }",
        "class Foo { static { bar(); } static { bar(); } }",
    ];

    let fail = vec![
        "class Foo { static {} }",
        "class Foo { static { } }",
        "class Foo { static {

			 } }",
        "class Foo { static { bar(); } static {} }",
    ];

    Tester::new(NoEmptyStaticBlock::NAME, pass, fail).test_and_snapshot();
}
