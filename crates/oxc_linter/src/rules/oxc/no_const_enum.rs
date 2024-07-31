use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_const_enum_diagnostic(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected const enum")
        .with_help("Const enums are not supported by bundlers and are incompatible with the isolatedModules mode. Their use can lead to import nonexistent values (because const enums are erased).")
        .with_label(span0)
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
    /// ### Example
    /// ```javascript
    /// const enum Color {
    ///    Red,
    ///    Green,
    ///    Blue
    /// }
    /// ```
    NoConstEnum,
    restriction,
);

impl Rule for NoConstEnum {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a, '_>) {
        if let AstKind::TSEnumDeclaration(enum_decl) = node.kind() {
            if !enum_decl.r#const {
                return;
            }

            let span = Span::new(enum_decl.span.start, enum_decl.span.start + 5);

            ctx.diagnostic_with_fix(no_const_enum_diagnostic(span), |fixer| {
                // const enum Color { Red, Green, Blue }
                // ^
                let start = span.start;

                // const enum Color { Red, Green, Blue }
                // ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
                let text = fixer.source_range(Span::new(start, enum_decl.span.end));

                // const  enum Color { Red, Green, Blue }
                //  ^^^^^^
                let offset = u32::try_from(text.find("enum").unwrap_or(1)).unwrap_or(1); // 1 is the default offset

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
        ("const enum Color { Red, Green, Blue }", "enum Color { Red, Green, Blue }", None),
        ("const   enum Color { Red, Green, Blue }", "enum Color { Red, Green, Blue }", None),
    ];

    Tester::new(NoConstEnum::NAME, pass, fail).expect_fix(fix).test_and_snapshot();
}
