use oxc_ast::{ast::ModifierKind, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, fixer::Fix, rule::Rule, AstNode};

fn no_const_enum_diagnostic(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("oxc(no-const-enum): Unexpected const enum")
        .with_help("Const enums are not supported by bundlers and are incompatible with the isolatedModules mode. Their use can lead to import nonexistent values (because const enums are erased).")
        .with_labels([span0.into()])
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
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::TSEnumDeclaration(enum_decl) = node.kind() {
            let Some(const_enum) =
                enum_decl.modifiers.find(|modifier| matches!(modifier.kind, ModifierKind::Const))
            else {
                return;
            };

            ctx.diagnostic_with_fix(no_const_enum_diagnostic(const_enum.span), || {
                // const enum Color { Red, Green, Blue }
                // ^
                let start = const_enum.span.start;

                // const enum Color { Red, Green, Blue }
                // ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
                let text = Span::new(start, enum_decl.span.end).source_text(ctx.source_text());

                // const  enum Color { Red, Green, Blue }
                //  ^^^^^^
                let offset = u32::try_from(text.find("enum").unwrap_or(1)).unwrap_or(1); // 1 is the default offset

                let end = start + offset;
                Fix::delete(Span::new(start, end))
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
