use oxc_ast::{ast::MemberExpression, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

fn prefer_dom_node_append_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Prefer `Node#append()` over `Node#appendChild()` for DOM nodes.")
        .with_help("Replace `Node#appendChild()` with `Node#append()`.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferDomNodeAppend;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces the use of, for example, `document.body.append(div);` over `document.body.appendChild(div);` for DOM nodes.
    ///
    /// ### Why is this bad?
    ///
    /// There are [some advantages of using `Node#append()`](https://developer.mozilla.org/en-US/docs/Web/API/ParentNode/append), like the ability to append multiple nodes and to append both [`DOMString`](https://developer.mozilla.org/en-US/docs/Web/API/DOMString) and DOM node objects.
    ///
    /// ### Examples
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// foo.appendChild(bar);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// foo.append(bar);
    /// ```
    PreferDomNodeAppend,
    unicorn,
    pedantic,
    fix
);

impl Rule for PreferDomNodeAppend {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        if call_expr.optional {
            return;
        }

        let Some(member_expr) = call_expr.callee.get_member_expr() else {
            return;
        };

        let span = match member_expr {
            MemberExpression::StaticMemberExpression(v) => {
                if !matches!(v.property.name.as_str(), "appendChild") {
                    return;
                }
                v.property.span
            }
            _ => return,
        };

        if call_expr.arguments.len() != 1 {
            return;
        }

        if call_expr.arguments[0].is_spread() {
            return;
        }

        ctx.diagnostic_with_fix(prefer_dom_node_append_diagnostic(span), |fixer| {
            fixer.replace(span, "append")
        });
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r"parent.append(child);",
        r"new parent.appendChild(child);",
        r"appendChild(child);",
        r"parent['appendChild'](child);",
        r"parent[appendChild](child);",
        r"parent.foo(child);",
        r"parent.appendChild(one, two);",
        r"parent.appendChild();",
        r"parent.appendChild(...argumentsArray)",
        r"parent.appendChild?.(child)",
    ];

    let fail = vec![
        r"node.appendChild(child);",
        r"document.body.appendChild(child);",
        r"node.appendChild(foo)",
        r"const foo = node.appendChild(child);",
        r"console.log(node.appendChild(child));",
        r"node.appendChild(child).appendChild(grandchild);",
        r#"node.appendChild(child) || "foo";"#,
        r"node.appendChild(child) + 0;",
        r"node.appendChild(child) + 0;",
        r"+node.appendChild(child);",
        r#"node.appendChild(child) ? "foo" : "bar";"#,
        r"if (node.appendChild(child)) {}",
        r"const foo = [node.appendChild(child)]",
        r"const foo = { bar: node.appendChild(child) }",
        r"function foo() { return node.appendChild(child); }",
        r"const foo = () => { return node.appendChild(child); }",
        r"foo(bar = node.appendChild(child))",
        r"node?.appendChild(child);",
        r"() => node?.appendChild(child)",
    ];

    let fix = vec![
        (
            r"node.appendChild(child).appendChild(grandchild);",
            r"node.append(child).append(grandchild);",
        ),
        (r"node?.appendChild(child);", r"node?.append(child);"),
        (
            r"function foo() { return node.appendChild(child); }",
            r"function foo() { return node.append(child); }",
        ),
        (r"const foo = [node.appendChild(child)]", r"const foo = [node.append(child)]"),
    ];

    Tester::new(PreferDomNodeAppend::NAME, PreferDomNodeAppend::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
