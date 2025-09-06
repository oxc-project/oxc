use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

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
    /// ### Examples
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
    unicorn,
    style,
    conditional_fix
);

impl Rule for PreferDomNodeTextContent {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::StaticMemberExpression(member_expr) => {
                let (span, name) = member_expr.static_property_info();
                if name == "innerText" {
                    ctx.diagnostic_with_fix(
                        prefer_dom_node_text_content_diagnostic(span),
                        |fixer| fixer.replace(span, "textContent"),
                    );
                }
            }
            // `const {innerText} = node` or `({innerText: text} = node)`
            AstKind::IdentifierName(identifier) => {
                if identifier.name != "innerText" {
                    return;
                }

                let parent_node = ctx.nodes().parent_node(node.id());
                let grand_parent_node = ctx.nodes().parent_node(parent_node.id());

                if matches!(
                    parent_node.kind(),
                    AstKind::BindingProperty(_) | AstKind::AssignmentTargetPropertyProperty(_)
                ) && (matches!(grand_parent_node.kind(), AstKind::ObjectPattern(_))
                    || matches!(
                        grand_parent_node.kind(),
                        AstKind::IdentifierReference(_)
                            | AstKind::ObjectAssignmentTarget(_)
                            | AstKind::AssignmentTargetPropertyIdentifier(_)
                            | AstKind::ArrayAssignmentTarget(_)
                            | AstKind::ComputedMemberExpression(_)
                            | AstKind::StaticMemberExpression(_)
                            | AstKind::PrivateFieldExpression(_)
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

                let mut ancestor_kinds = ctx.nodes().ancestor_kinds(node.id());

                let Some(mut parent_node_kind) = ancestor_kinds.next() else { return };
                if matches!(parent_node_kind, AstKind::AssignmentTargetPropertyIdentifier(_)) {
                    let Some(next) = ancestor_kinds.next() else { return };
                    parent_node_kind = next;
                }
                let Some(grand_parent_node_kind) = ancestor_kinds.next() else { return };

                if matches!(parent_node_kind, AstKind::ObjectAssignmentTarget(_))
                    && matches!(
                        grand_parent_node_kind,
                        AstKind::ExpressionStatement(_)
                            | AstKind::AssignmentExpression(_)
                            | AstKind::ObjectAssignmentTarget(_)
                    )
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
    let fix: Vec<(&'static str, &'static str)> = vec![
        ("node.innerText;", "node.textContent;"),
        ("node?.innerText;", "node?.textContent;"),
        ("node.innerText = 'foo';", "node.textContent = 'foo';"),
        ("innerText.innerText = 'foo';", "innerText.textContent = 'foo';"),
    ];

    Tester::new(PreferDomNodeTextContent::NAME, PreferDomNodeTextContent::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
