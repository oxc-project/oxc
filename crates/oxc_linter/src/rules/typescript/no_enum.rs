use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_enum_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("TypeScript enums are not allowed.")
        .with_help("Use a union type or a const object instead of an enum.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoEnum;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows TypeScript enums.
    ///
    /// ### Why is this bad?
    ///
    /// TypeScript enums have several pitfalls:
    /// - They generate runtime code unlike other TS-only features
    /// - Numeric enums are not type-safe
    /// - String enums create an opaque type
    /// - `const enum` has interop issues
    ///
    /// Union types or const objects are safer alternatives.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// enum Color { Red, Green, Blue }
    /// const enum Direction { Up, Down }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// type Color = 'Red' | 'Green' | 'Blue';
    /// const Direction = { Up: 'Up', Down: 'Down' } as const;
    /// ```
    NoEnum,
    typescript,
    restriction
);

impl Rule for NoEnum {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::TSEnumDeclaration(decl) = node.kind() else {
            return;
        };

        ctx.diagnostic(no_enum_diagnostic(Span::new(
            decl.span.start,
            decl.span.start + if decl.r#const { 10 } else { 4 }, // "const enum" or "enum"
        )));
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "type Color = 'Red' | 'Green' | 'Blue';",
        "const Direction = { Up: 'Up', Down: 'Down' } as const;",
    ];

    let fail = vec![
        "enum Color { Red, Green, Blue }",
        "const enum Direction { Up, Down }",
        "enum Status { Active = 'active', Inactive = 'inactive' }",
    ];

    Tester::new(NoEnum::NAME, NoEnum::PLUGIN, pass, fail).test_and_snapshot();
}
