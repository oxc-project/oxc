use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_const_enum_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected const enum")
        .with_help("Const enums are not supported by bundlers and are incompatible with the isolatedModules mode. Their use can lead to import nonexistent values (because const enums are erased).")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoConstEnum;

// Ported from <https://biomejs.dev/linter/rules/no-const-enum/>
declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow TypeScript `const enum`
    ///
    /// ### Why is this bad?
    ///
    /// Const enums are enums that should be inlined at use sites.
    /// Const enums are not supported by bundlers and are incompatible with the isolatedModules mode.
    /// Their use can lead to import nonexistent values (because const enums are erased).
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// const enum Color {
    ///     Red,
    ///     Green,
    ///     Blue
    /// }
    /// ```
    NoConstEnum,
    oxc,
    restriction,
    fix,
    version = "0.4.2",
    short_description = "Disallow TypeScript `const enum`",
);

impl Rule for NoConstEnum {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::TSEnumDeclaration(enum_decl) = node.kind() {
            if !enum_decl.r#const {
                return;
            }

            let span = Span::sized(enum_decl.span.start, 5);

            ctx.diagnostic_with_fix(no_const_enum_diagnostic(span), |fixer| {
                // const enum Color { Red, Green, Blue }
                // ^
                let start = span.start;

                // const  enum Color { Red, Green, Blue }
                //  ^^^^^^
                let Some(offset) = fixer.find_next_token_within(start, enum_decl.span.end, "enum")
                else {
                    return fixer.noop();
                };

                // the whole range up to `enum` is deleted, so bail rather than eat a comment
                if ctx.has_comments_between(Span::new(start, start + offset)) {
                    return fixer.noop();
                }

                fixer.delete_range(Span::sized(start, offset))
            });
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec!["enum Color { Red, Green, Blue }"];

    let fail = vec!["const enum Color { Red, Green, Blue }"];

    let fix = vec![
        // the `enum` inside the comment is not the keyword; deleting up to the real
        // one would swallow the comment, so no fix is offered
        ("const /* enum */ enum Color { Red }", "const /* enum */ enum Color { Red }", None),
        ("const enum Color { Red, Green, Blue }", "enum Color { Red, Green, Blue }", None),
        ("const   enum Color { Red, Green, Blue }", "enum Color { Red, Green, Blue }", None),
    ];

    Tester::new(NoConstEnum::NAME, NoConstEnum::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
