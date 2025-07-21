use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_enum_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected enum")
        .with_help("Enums are not erasable syntax and are incompatible with TypeScript's --erasableSyntaxOnly flag. Consider using union types or const assertions instead.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoEnum;

// Ported from <https://github.com/JoshuaKGoldberg/eslint-plugin-erasable-syntax-only/blob/main/docs/rules/enums.md/>
declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow TypeScript `enum`
    ///
    /// ### Why is this bad?
    ///
    /// TypeScript 5.8 introduces the `--erasableSyntaxOnly` flag. When this flag is enabled,
    /// TypeScript will only allow you to use constructs that can be erased from a file, and
    /// will issue an error if it encounters any constructs that cannot be erased.
    ///
    /// Enums are not erasable syntax because they generate runtime code and cannot be
    /// completely removed during compilation. This makes them incompatible with the
    /// `--erasableSyntaxOnly` flag.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// enum Color {
    ///     Red,
    ///     Green,
    ///     Blue
    /// }
    ///
    /// const enum Status {
    ///     Active,
    ///     Inactive
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// // Use union types instead
    /// type Color = 'red' | 'green' | 'blue';
    ///
    /// // Use const assertions
    /// const Color = {
    ///     Red: 'red',
    ///     Green: 'green',
    ///     Blue: 'blue'
    /// } as const;
    ///
    /// // Use namespace with const assertions
    /// namespace Color {
    ///     export const Red = 'red';
    ///     export const Green = 'green';
    ///     export const Blue = 'blue';
    /// }
    /// ```
    NoEnum,
    typescript,
    restriction,
    pending
);

impl Rule for NoEnum {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::TSEnumDeclaration(enum_decl) = node.kind() {
            ctx.diagnostic(no_enum_diagnostic(enum_decl.span));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![];

    let fail = vec![
        // Regular enum
        "enum Color { Red, Green, Blue }",
        // Enum with string values
        "enum Status { Active = 'active', Inactive = 'inactive' }",
        // Enum with mixed values
        "enum Mixed { A, B = 'b', C = 2 }",
        // Exported enum
        "export enum Color { Red, Green, Blue }",
        // Enum with explicit numeric values
        "enum Priority { Low = 1, Medium = 2, High = 3 }",
        // Const enum (should also be caught)
        "const enum Direction { Up, Down, Left, Right }",
        // Enum with computed values
        "enum Computed { A = 1 << 1, B = 1 << 2 }",
        // Ambient enum declaration
        "declare enum AmbientEnum { A, B, C }",
        // Const enum
        "const enum ConstEnum { A = 1, B = 2, C = 3 }",
    ];

    Tester::new(NoEnum::NAME, NoEnum::PLUGIN, pass, fail).test_and_snapshot();
}
