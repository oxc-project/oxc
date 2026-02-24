use oxc_ast::{AstKind, ast::BindingIdentifier};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::AstNode;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule};

fn no_class_assign_diagnostic(name: &str, decl_span: Span, assign_span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Unexpected re-assignment of class {name}"))
        .with_help("Use a different variable name instead of re-assigning the class declaration.")
        .with_labels([
            decl_span.label(format!("{name} is declared as class here")),
            assign_span.label(format!("{name} is re-assigned here")),
        ])
}

#[derive(Debug, Default, Clone)]
pub struct NoClassAssign;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow reassigning class variables.
    ///
    /// This rule can be disabled for TypeScript code, as the TypeScript compiler
    /// enforces this check.
    ///
    /// ### Why is this bad?
    ///
    /// `ClassDeclaration` creates a variable that can be re-assigned, but the re-assignment is a
    /// mistake in most cases.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// class A { }
    /// A = 0;
    /// ```
    ///
    /// ```javascript
    /// A = 0;
    /// class A { }
    /// ```
    ///
    /// ```javascript
    /// class A {
    ///   b() {
    ///     A = 0;
    ///   }
    /// }
    /// ```
    ///
    /// ```javascript
    /// let A = class A {
    ///   b() {
    ///     A = 0;
    ///     // `let A` is shadowed by the class name.
    ///   }
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// let A = class A { }
    /// A = 0; // A is a variable.
    /// ```
    ///
    /// ```javascript
    /// let A = class {
    ///   b() {
    ///     A = 0; // A is a variable.
    ///   }
    /// }
    /// ```
    ///
    /// ```javascript
    /// class A {
    ///   b(A) {
    ///     A = 0; // A is a parameter.
    ///   }
    /// }
    /// ```
    NoClassAssign,
    eslint,
    correctness
);

impl Rule for NoClassAssign {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::Class(class) = node.kind() else {
            return;
        };

        let Some(symbol_id) = class.id.as_ref().map(BindingIdentifier::symbol_id) else {
            return;
        };

        let symbol_table = ctx.scoping();
        // This should always be considered a class (since we got it from a class declaration),
        // but we check in debug mode just to be sure.
        debug_assert!(symbol_table.symbol_flags(symbol_id).is_class());

        for reference in symbol_table.get_resolved_references(symbol_id) {
            if reference.is_write() {
                ctx.diagnostic(no_class_assign_diagnostic(
                    symbol_table.symbol_name(symbol_id),
                    symbol_table.symbol_span(symbol_id),
                    ctx.semantic().reference_span(reference),
                ));
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "class A { } foo(A);",
        "let A = class A { }; foo(A);",
        "class A { b(A) { A = 0; } }",
        "class A { b() { let A; A = 0; } }",
        "let A = class { b() { A = 0; } }",
        "let A = class B { foo() { A = 0; } }",
        "let A = class A {}; A = 1",
        "var x = 0; x = 1;",
        "let x = 0; x = 1;",
        "const x = 0; x = 1;",
        "function x() {} x = 1;",
        "function foo(x) { x = 1; }",
        "try {} catch (x) { x = 1; }",
        "if (foo) { class A {} } else { class A {} } A = 1;",
        // Sequence expression
        "(class A {}, A = 1)",
        // Class expressions
        "let A = class { }; A = 1;",
        "let A = class B { }; A = 1;",
    ];

    let fail = vec![
        "class A { } A = 0;",
        "class A { } ({A} = 0);",
        "class A { } ({b: A = 0} = {});",
        "A = 0; class A { }",
        "class A { b() { A = 0; } }",
        "let A = class A { b() { A = 0; } }",
        "class A { } A = 0; A = 1;",
        "if (foo) { class A {} A = 1; }",
    ];

    Tester::new(NoClassAssign::NAME, NoClassAssign::PLUGIN, pass, fail).test_and_snapshot();
}
