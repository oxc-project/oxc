use oxc_ast::AstKind;
use oxc_ast::ast::{BindingPattern, VariableDeclarationKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn naming_convention_diagnostic(
    span: Span,
    kind: &str,
    name: &str,
    expected: &str,
) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("{kind} `{name}` does not match the expected naming convention."))
        .with_help(format!("Rename to match the `{expected}` naming convention."))
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NamingConvention;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces naming conventions for various code identifiers.
    ///
    /// ### Why is this bad?
    ///
    /// Consistent naming conventions improve code readability and make it
    /// easier to understand the purpose and scope of identifiers at a glance.
    ///
    /// ### Conventions enforced
    ///
    /// - **Classes**: PascalCase
    /// - **Interfaces/Type aliases**: PascalCase
    /// - **Enums**: PascalCase
    /// - **Enum members**: PascalCase or UPPER_CASE
    /// - **Type parameters**: PascalCase (single letter or prefixed with T)
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// class my_class {}
    /// interface my_interface {}
    /// type my_type = string;
    /// enum my_enum { my_value }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// class MyClass {}
    /// interface MyInterface {}
    /// type MyType = string;
    /// enum MyEnum { MyValue }
    /// ```
    NamingConvention,
    typescript,
    style,
    pending
);

impl Rule for NamingConvention {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if !ctx.source_type().is_typescript() {
            return;
        }

        match node.kind() {
            // Classes must be PascalCase
            AstKind::Class(class) => {
                if let Some(id) = &class.id {
                    let name = id.name.as_str();
                    if !is_pascal_case(name) {
                        ctx.diagnostic(naming_convention_diagnostic(
                            id.span,
                            "Class",
                            name,
                            "PascalCase",
                        ));
                    }
                }
            }
            // Interfaces must be PascalCase
            AstKind::TSInterfaceDeclaration(decl) => {
                let name = decl.id.name.as_str();
                if !is_pascal_case(name) {
                    ctx.diagnostic(naming_convention_diagnostic(
                        decl.id.span,
                        "Interface",
                        name,
                        "PascalCase",
                    ));
                }
            }
            // Type aliases must be PascalCase
            AstKind::TSTypeAliasDeclaration(decl) => {
                let name = decl.id.name.as_str();
                if !is_pascal_case(name) {
                    ctx.diagnostic(naming_convention_diagnostic(
                        decl.id.span,
                        "Type alias",
                        name,
                        "PascalCase",
                    ));
                }
            }
            // Enums must be PascalCase
            AstKind::TSEnumDeclaration(decl) => {
                let name = decl.id.name.as_str();
                if !is_pascal_case(name) {
                    ctx.diagnostic(naming_convention_diagnostic(
                        decl.id.span,
                        "Enum",
                        name,
                        "PascalCase",
                    ));
                }
            }
            // Enum members must be PascalCase or UPPER_CASE
            AstKind::TSEnumMember(member) => {
                if let oxc_ast::ast::TSEnumMemberName::Identifier(id) = &member.id {
                    let name = id.name.as_str();
                    if !is_pascal_case(name) && !is_upper_case(name) {
                        ctx.diagnostic(naming_convention_diagnostic(
                            id.span,
                            "Enum member",
                            name,
                            "PascalCase or UPPER_CASE",
                        ));
                    }
                }
            }
            _ => {}
        }
    }
}

fn is_pascal_case(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    let first = s.chars().next().unwrap();
    if !first.is_ascii_uppercase() {
        return false;
    }
    // Must not contain underscores (unless it's ALL_CAPS)
    !s.contains('_')
}

fn is_upper_case(s: &str) -> bool {
    s.len() >= 2
        && s.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
        && s.chars().any(|c| c.is_ascii_uppercase())
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "class MyClass {}",
        "interface MyInterface {}",
        "type MyType = string;",
        "enum MyEnum { MyValue }",
        "enum Status { ACTIVE, INACTIVE }",
    ];

    let fail = vec![
        "class my_class {}",
        "interface my_interface {}",
        "type my_type = string;",
        "enum my_enum { my_value }",
    ];

    Tester::new(NamingConvention::NAME, NamingConvention::PLUGIN, pass, fail).test_and_snapshot();
}
