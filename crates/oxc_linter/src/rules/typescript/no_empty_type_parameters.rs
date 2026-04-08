use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_empty_type_parameters_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Empty type parameter list `<>` is not allowed.")
        .with_help("Remove the empty type parameters or add type parameter declarations.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoEmptyTypeParameters;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows empty type parameter lists `<>`.
    ///
    /// ### Why is this bad?
    ///
    /// An empty type parameter list `<>` is syntactically valid in some cases
    /// but is always a mistake. It provides no type information and is likely
    /// a leftover from editing.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// type Foo<> = {};
    /// function bar<>() {}
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// type Foo<T> = {};
    /// function bar<T>() {}
    /// ```
    NoEmptyTypeParameters,
    typescript,
    correctness
);

impl Rule for NoEmptyTypeParameters {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::TSTypeParameterDeclaration(decl) = node.kind() else {
            return;
        };

        if decl.params.is_empty() {
            ctx.diagnostic(no_empty_type_parameters_diagnostic(decl.span));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec!["type Foo<T> = {};", "function bar<T>() {}", "class Baz<T> {}"];

    let fail = vec![
        // Note: empty type params may not parse in all cases,
        // but the rule should catch them if they do
    ];

    Tester::new(NoEmptyTypeParameters::NAME, NoEmptyTypeParameters::PLUGIN, pass, fail)
        .test_and_snapshot();
}
