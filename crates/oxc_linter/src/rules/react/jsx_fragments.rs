use oxc_ast::{
    AstKind,
    ast::{JSXElementName, JSXMemberExpressionObject, JSXOpeningElement},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{AstNode, context::LintContext, rule::Rule};

fn jsx_fragments_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Shorthand form for React fragments is preferred")
        .with_help("Use <></> instead of <React.Fragment></React.Fragment>")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct JsxFragments;

// See <https://github.com/oxc-project/oxc/issues/6050> for documentation details.
declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces the shorthand form for React fragments
    ///
    /// ### Why is this bad?
    ///
    /// Shorthand form is much more succinct and readable than the fully qualified element name.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// <React.Fragment><Foo /></React.Fragment>
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// <><Foo /></>
    /// ```
    ///
    /// ```jsx
    /// <React.Fragment key="key"><Foo /></React.Fragment>
    /// ```
    JsxFragments,
    react,
    style,
    fix
);

impl Rule for JsxFragments {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::JSXElement(jsx_elem) => {
                let Some(closing_element) = &jsx_elem.closing_element else {
                    return;
                };
                if !is_jsx_fragment(&jsx_elem.opening_element)
                    || jsx_elem.opening_element.attributes.len() > 0
                {
                    return;
                }
                ctx.diagnostic_with_fix(
                    jsx_fragments_diagnostic(jsx_elem.opening_element.name.span()),
                    |fixer| {
                        let before_opening_tag = ctx.source_range(Span::new(
                            jsx_elem.span().start,
                            jsx_elem.opening_element.span().start,
                        ));
                        let between_opening_tag_and_closing_tag = ctx.source_range(Span::new(
                            jsx_elem.opening_element.span().end,
                            closing_element.span().start,
                        ));
                        let after_closing_tag = ctx.source_range(Span::new(
                            closing_element.span().end,
                            jsx_elem.span().end,
                        ));
                        let mut replacement = String::new();
                        replacement.push_str(&before_opening_tag);
                        replacement.push_str("<>");
                        replacement.push_str(&between_opening_tag_and_closing_tag);
                        replacement.push_str("</>");
                        replacement.push_str(&after_closing_tag);
                        fixer.replace(jsx_elem.span(), replacement)
                    },
                );
            }
            _ => {}
        }
    }

    fn should_run(&self, ctx: &crate::context::ContextHost) -> bool {
        ctx.source_type().is_jsx()
    }
}

fn is_jsx_fragment(elem: &JSXOpeningElement) -> bool {
    match &elem.name {
        JSXElementName::IdentifierReference(ident) => ident.name == "Fragment",
        JSXElementName::MemberExpression(mem_expr) => {
            if let JSXMemberExpressionObject::IdentifierReference(ident) = &mem_expr.object {
                ident.name == "React" && mem_expr.property.name == "Fragment"
            } else {
                false
            }
        }
        JSXElementName::NamespacedName(_)
        | JSXElementName::Identifier(_)
        | JSXElementName::ThisExpression(_) => false,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "<><Foo /></>",
        "<Fragment key=\"key\"><Foo /></Fragment>",
        "<React.Fragment key=\"key\"><Foo /></React.Fragment>",
        "<Fragment />",
        "<React.Fragment />",
    ];

    let fail = vec!["<Fragment><Foo /></Fragment>", "<React.Fragment><Foo /></React.Fragment>"];

    let fix = vec![
        ("<Fragment><Foo /></Fragment>", "<><Foo /></>"),
        ("<React.Fragment><Foo /></React.Fragment>", "<><Foo /></>"),
    ];
    Tester::new(JsxFragments::NAME, JsxFragments::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
