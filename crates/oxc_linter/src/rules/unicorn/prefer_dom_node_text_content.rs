use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

fn prefer_dom_node_text_content_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Prefer `.textContent` over `.innerText`.")
        .with_help("Replace `.innerText` with `.textContent`.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferDomNodeTextContent;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces the use of `.textContent` over `.innerText` for DOM nodes.
    ///
    /// ### Why is this bad?
    ///
    /// There are some disadvantages of using .innerText.
    /// - `.innerText` is much more performance-heavy as it requires layout information to return the result.
    /// - `.innerText` is defined only for HTMLElement objects, while `.textContent` is defined for all Node objects.
    /// - `.innerText` is not standard, for example, it is not present in Firefox.
    ///
    /// ### Example
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// const text = foo.innerText;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// const text = foo.textContent;
    /// ```
    PreferDomNodeTextContent,
    style,
    conditional_fix
);

impl Rule for PreferDomNodeTextContent {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::MemberExpression(member_expr) => {
                if let Some((span, name)) = member_expr.static_property_info() {
                    if name == "innerText" && !member_expr.is_computed() {
                        ctx.diagnostic_with_fix(
                            prefer_dom_node_text_content_diagnostic(span),
                            |fixer| fixer.replace(span, "textContent"),
                        );
                    }
                }
            }
            // `const {innerText} = node` or `({innerText: text} = node)`
            AstKind::IdentifierName(identifier) => {
                if identifier.name != "innerText" {
                    return;
                }

                let mut ancestor_kinds =
                    ctx.nodes().iter_parents(node.id()).skip(1).map(AstNode::kind);
                let (Some(parent_node_kind), Some(grand_parent_node_kind)) =
                    (ancestor_kinds.next(), ancestor_kinds.next())
                else {
                    return;
                };

                if matches!(parent_node_kind, AstKind::PropertyKey(_))
                    && (matches!(grand_parent_node_kind, AstKind::ObjectPattern(_))
                        || matches!(
                            grand_parent_node_kind,
                            AstKind::ObjectAssignmentTarget(_)
                                | AstKind::SimpleAssignmentTarget(_)
                                | AstKind::AssignmentTarget(_)
                        ))
                {
                    ctx.diagnostic(prefer_dom_node_text_content_diagnostic(identifier.span));
                }
            }
            // `({innerText} = node)`
            AstKind::IdentifierReference(identifier_ref) => {
                if identifier_ref.name != "innerText" {
                    return;
                }

                let mut ancestor_kinds =
                    ctx.nodes().iter_parents(node.id()).skip(1).map(AstNode::kind);
                let (Some(parent_node_kind), Some(grand_parent_node_kind)) =
                    (ancestor_kinds.next(), ancestor_kinds.next())
                else {
                    return;
                };

                if matches!(
                    parent_node_kind,
                    AstKind::ObjectAssignmentTarget(_)
                        | AstKind::AssignmentTarget(_)
                        | AstKind::SimpleAssignmentTarget(_)
                ) && matches!(grand_parent_node_kind, AstKind::AssignmentTargetPattern(_))
                {
                    ctx.diagnostic(prefer_dom_node_text_content_diagnostic(identifier_ref.span));
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
        ("innerText;", None),
        ("node.textContent;", None),
        ("node[innerText];", None),
        ("innerText = true;", None),
        ("node['innerText'];", None),
        ("innerText.textContent", None),
        ("const [innerText] = node;", None),
        ("[innerText] = node;", None),
        ("const {[innerText]: text} = node;", None),
        ("({[innerText]: text} = node);", None),
        ("const foo = {innerText}", None),
        ("const foo = {innerText: text}", None),
    ];

    let fail = vec![
        ("node.innerText;", None),
        ("node?.innerText;", None),
        ("node.innerText = 'foo';", None),
        ("innerText.innerText;", None),
        ("const {innerText} = node;", None),
        ("const {innerText,} = node;", None),
        ("const {innerText: text} = node;", None),
        ("const {innerText = \"default text\"} = node;", None),
        ("const {innerText: text = \"default text\"} = node;", None),
        ("({innerText} = node);", None),
        ("({innerText: text} = node);", None),
        ("({innerText = \"default text\"} = node);", None),
        ("({innerText: text = \"default text\"} = node);", None),
        ("function foo({innerText}) {return innerText}", None),
        ("for (const [{innerText}] of elements);", None),
    ];

    // TODO: implement a fixer for destructuring assignment cases
    let fix = vec![
        ("node.innerText;", "node.textContent;"),
        ("node?.innerText;", "node?.textContent;"),
        ("node.innerText = 'foo';", "node.textContent = 'foo';"),
        ("innerText.innerText = 'foo';", "innerText.textContent = 'foo';"),
    ];

    Tester::new(PreferDomNodeTextContent::NAME, pass, fail).expect_fix(fix).test_and_snapshot();
}
