use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn explicit_member_accessibility_diagnostic(span: Span, kind: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Missing accessibility modifier on {kind}."))
        .with_help("Add an explicit `public`, `private`, or `protected` modifier.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct ExplicitMemberAccessibility;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Requires explicit accessibility modifiers on class properties and methods.
    ///
    /// ### Why is this bad?
    ///
    /// Without explicit accessibility modifiers, class members default to `public`.
    /// Being explicit about accessibility makes the intent clearer and helps
    /// prevent accidentally exposing internal implementation details.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// class Foo {
    ///   bar: string;
    ///   baz() {}
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// class Foo {
    ///   public bar: string;
    ///   private baz() {}
    /// }
    /// ```
    ExplicitMemberAccessibility,
    typescript,
    style,
    pending
);

impl Rule for ExplicitMemberAccessibility {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if !ctx.source_type().is_typescript() {
            return;
        }

        match node.kind() {
            AstKind::MethodDefinition(method) => {
                // Skip constructors
                if method.kind.is_constructor() {
                    return;
                }
                if method.accessibility.is_none() {
                    ctx.diagnostic(explicit_member_accessibility_diagnostic(
                        Span::new(method.span.start, method.span.start + 1),
                        "method",
                    ));
                }
            }
            AstKind::PropertyDefinition(prop) => {
                if prop.accessibility.is_none() {
                    ctx.diagnostic(explicit_member_accessibility_diagnostic(
                        Span::new(prop.span.start, prop.span.start + 1),
                        "property",
                    ));
                }
            }
            _ => {}
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "class Foo { public bar: string; }",
        "class Foo { private baz() {} }",
        "class Foo { protected qux = 1; }",
        "class Foo { constructor() {} }",
    ];

    let fail =
        vec!["class Foo { bar: string; }", "class Foo { baz() {} }", "class Foo { qux = 1; }"];

    Tester::new(ExplicitMemberAccessibility::NAME, ExplicitMemberAccessibility::PLUGIN, pass, fail)
        .test_and_snapshot();
}
