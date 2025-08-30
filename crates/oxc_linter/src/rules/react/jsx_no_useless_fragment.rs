use crate::{
    AstNode,
    context::{ContextHost, LintContext},
    fixer::{RuleFix, RuleFixer},
    rule::Rule,
    utils::is_jsx_fragment,
};
use oxc_allocator::Vec as ArenaVec;
use oxc_ast::{
    AstKind,
    ast::{
        JSXAttributeItem, JSXAttributeName, JSXChild, JSXElement, JSXElementName, JSXExpression,
        JSXFragment,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::NodeId;
use oxc_span::{GetSpan, Span};

fn needs_more_children(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Fragments should contain more than one child.").with_label(span)
}

fn child_of_html_element(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Passing a fragment to a HTML element is useless.").with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct JsxNoUselessFragment {
    /// Allow fragments with a single expression child.
    pub allow_expressions: bool,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow unnecessary fragments.
    ///
    /// ### Why is this bad?
    ///
    /// Fragments are a useful tool when you need to group multiple children without adding a node to the DOM tree. However, sometimes you might end up with a fragment with a single child. When this child is an element, string, or expression, it's not necessary to use a fragment.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// <>foo</>
    /// <div><>foo</></div>
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// <>foo <div></div></>
    /// <div>foo</div>
    /// ```
    JsxNoUselessFragment,
    react,
    pedantic,
    suggestion
);

impl Rule for JsxNoUselessFragment {
    fn from_configuration(value: serde_json::Value) -> Self {
        let value = value.as_array().and_then(|arr| arr.first()).and_then(|val| val.as_object());

        Self {
            allow_expressions: value
                .and_then(|val| val.get("allowExpressions").and_then(serde_json::Value::as_bool))
                .unwrap_or(Self::default().allow_expressions),
        }
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::JSXElement(jsx_elem) => {
                if !is_jsx_fragment(&jsx_elem.opening_element) {
                    return;
                }
                self.check_element(node, jsx_elem, ctx);
            }
            AstKind::JSXFragment(jsx_elem) => {
                self.check_fragment(node, jsx_elem, ctx);
            }
            _ => {}
        }
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.source_type().is_jsx()
    }
}

impl JsxNoUselessFragment {
    fn check_element(&self, node: &AstNode, elem: &JSXElement, ctx: &LintContext) {
        if jsx_elem_has_key_attr(elem) {
            return;
        }

        if has_less_than_two_children(&elem.children)
            && !is_fragment_with_only_text_and_is_not_child(node.id(), &elem.children, ctx)
            && !(self.allow_expressions && is_fragment_with_single_expression(&elem.children))
        {
            let span = elem.opening_element.span;
            let diagnostic = needs_more_children(span);
            if can_fix(node, &elem.children, ctx) {
                ctx.diagnostic_with_suggestion(diagnostic, |fixer| {
                    fix_fragment_element(elem, ctx, fixer)
                });
            } else {
                ctx.diagnostic(diagnostic);
            }
        }

        if is_child_of_html_element(node, ctx) {
            let span = elem.opening_element.span;
            let diagnostic = child_of_html_element(span);
            if can_fix(node, &elem.children, ctx) {
                ctx.diagnostic_with_suggestion(diagnostic, |fixer| {
                    fix_fragment_element(elem, ctx, fixer)
                });
            } else {
                ctx.diagnostic(diagnostic);
            }
        }
    }

    fn check_fragment(&self, node: &AstNode, elem: &JSXFragment, ctx: &LintContext) {
        if has_less_than_two_children(&elem.children)
            && !is_fragment_with_only_text_and_is_not_child(node.id(), &elem.children, ctx)
            && !(self.allow_expressions && is_fragment_with_single_expression(&elem.children))
        {
            let span = elem.opening_fragment.span;
            let diagnostic = needs_more_children(span);
            if can_fix(node, &elem.children, ctx) {
                ctx.diagnostic_with_suggestion(diagnostic, |fixer| {
                    fix_jsx_fragment(elem, ctx, fixer)
                });
            } else {
                ctx.diagnostic(diagnostic);
            }
        }

        if is_child_of_html_element(node, ctx) {
            let span = elem.opening_fragment.span;
            let diagnostic = child_of_html_element(span);
            if can_fix(node, &elem.children, ctx) {
                ctx.diagnostic_with_suggestion(diagnostic, |fixer| {
                    fix_jsx_fragment(elem, ctx, fixer)
                });
            } else {
                ctx.diagnostic(diagnostic);
            }
        }
    }
}

fn fix_fragment_element<'a>(
    elem: &JSXElement,
    ctx: &LintContext<'a>,
    fixer: RuleFixer<'_, 'a>,
) -> RuleFix<'a> {
    let replacement = if let Some(closing_elem) = &elem.closing_element {
        trim_like_react(
            Span::new(elem.opening_element.span.end, closing_elem.span.start)
                .source_text(ctx.source_text()),
        )
    } else {
        ""
    };

    fixer.replace(elem.span(), trim_like_react(replacement))
}

fn fix_jsx_fragment<'a>(
    elem: &JSXFragment,
    ctx: &LintContext<'a>,
    fixer: RuleFixer<'_, 'a>,
) -> RuleFix<'a> {
    fixer.replace(
        elem.span(),
        trim_like_react(
            Span::new(elem.opening_fragment.span.end, elem.closing_fragment.span.start)
                .source_text(ctx.source_text()),
        ),
    )
}

fn trim_like_react(text: &str) -> &str {
    let bytes = text.as_bytes();
    let len = bytes.len();

    if len == 0 {
        return text;
    }

    // Find leading whitespace
    let mut leading_end = 0;
    let mut has_leading_newline = false;

    for &byte in bytes {
        if byte.is_ascii_whitespace() {
            if byte == b'\n' {
                has_leading_newline = true;
            }
            leading_end += 1;
        } else {
            break;
        }
    }

    // Find trailing whitespace
    let mut trailing_start = len;
    let mut has_trailing_newline = false;

    for &byte in bytes.iter().rev() {
        if byte.is_ascii_whitespace() {
            if byte == b'\n' {
                has_trailing_newline = true;
            }
            trailing_start -= 1;
        } else {
            break;
        }
    }

    // Apply React-like trimming rules
    let start = if has_leading_newline { leading_end } else { 0 };
    let end = if has_trailing_newline { trailing_start } else { len };

    // Handle edge cases
    if start >= end {
        return "";
    }

    &text[start..end]
}

fn can_fix(node: &AstNode, children: &ArenaVec<JSXChild<'_>>, ctx: &LintContext) -> bool {
    let parent = ctx.nodes().parent_kind(node.id());

    if !matches!(parent, AstKind::JSXElement(_) | AstKind::JSXFragment(_)) {
        // const a = <></>
        if children.is_empty() {
            return false;
        }

        // const a = <>cat {meow}</>
        if children.iter().all(|child| {
            is_whitespace_only_text(child) || matches!(child, JSXChild::ExpressionContainer(_))
        }) {
            return false;
        }
    }

    // Not safe to fix `<Eeee><>foo</></Eeee>` because `Eeee` might require its children be a ReactElement.
    if let AstKind::JSXElement(el) = parent {
        if !el
            .opening_element
            .name
            .get_identifier_name()
            .is_some_and(|ident| ident.chars().all(char::is_lowercase))
            && !is_jsx_fragment(&el.opening_element)
        {
            return false;
        }
    }

    true
}

fn is_whitespace_only_text(child: &JSXChild) -> bool {
    match child {
        JSXChild::Text(text) => text.value.trim().is_empty(),
        _ => false,
    }
}

fn jsx_elem_has_key_attr(elem: &JSXElement) -> bool {
    elem.opening_element.attributes.iter().any(|attr| {
        let JSXAttributeItem::Attribute(attr) = attr else {
            return false;
        };

        let JSXAttributeName::Identifier(ident) = &attr.name else {
            return false;
        };

        ident.name == "key"
    })
}

fn is_fragment_with_single_expression(children: &oxc_allocator::Vec<'_, JSXChild<'_>>) -> bool {
    let children = children.iter().filter(|v| is_padding_spaces(v)).collect::<Vec<_>>();

    children.len() == 1 && matches!(children[0], JSXChild::ExpressionContainer(_))
}

fn is_padding_spaces(v: &JSXChild<'_>) -> bool {
    if let JSXChild::Text(v) = v {
        return !(v.value.trim().is_empty() && v.value.contains('\n'));
    }

    true
}

fn is_child_of_html_element(node: &AstNode, ctx: &LintContext) -> bool {
    if let AstKind::JSXElement(elem) = ctx.nodes().parent_kind(node.id()) {
        if is_html_element(&elem.opening_element.name) {
            return true;
        }
    }

    false
}

fn is_html_element(elem_name: &JSXElementName) -> bool {
    let JSXElementName::Identifier(ident) = elem_name else {
        return false;
    };

    ident.name.starts_with(char::is_lowercase)
}

fn has_less_than_two_children(children: &oxc_allocator::Vec<'_, JSXChild<'_>>) -> bool {
    let non_padding_children = children.iter().filter(|v| is_padding_spaces(v)).collect::<Vec<_>>();

    if non_padding_children.len() < 2 {
        return !non_padding_children.iter().any(|v| {
            if let JSXChild::ExpressionContainer(v) = v {
                if let JSXExpression::CallExpression(_) = v.expression {
                    return true;
                }
                return false;
            }

            false
        });
    }
    false
}

fn is_fragment_with_only_text_and_is_not_child<'a>(
    id: NodeId,
    node: &oxc_allocator::Vec<'a, JSXChild<'a>>,
    ctx: &LintContext,
) -> bool {
    if node.len() != 1 {
        return false;
    }

    if let Some(JSXChild::Text(_)) = node.first() {
        let parent = ctx.nodes().parent_kind(id);
        return !matches!(parent, AstKind::JSXElement(_) | AstKind::JSXFragment(_));
    }

    false
}

#[test]
fn test() {
    use serde_json::json;

    use crate::tester::Tester;

    let pass = vec![
        (r"<><Foo /><Bar /></>", None),
        (r"<>foo<div /></>", None),
        (r"<> <div /></>", None),
        (r#"<>{"moo"} </>"#, None),
        (r"<NotFragment />", None),
        (r"<React.NotFragment />", None),
        (r"<NotReact.Fragment />", None),
        (r"<Foo><><div /><div /></></Foo>", None),
        (r#"<div p={<>{"a"}{"b"}</>} />"#, None),
        (r"<Fragment key={item.id}>{item.value}</Fragment>", None),
        (r"<Fooo content={<>eeee ee eeeeeee eeeeeeee</>} />", None),
        (r"<>{foos.map(foo => foo)}</>", None),
        (r"<>{moo}</>", Some(json!([{ "allowExpressions": true }]))),
        (
            r"
        <>
            {moo}
        </>
        ",
            Some(json!([{ "allowExpressions": true }])),
        ),
        (r"{1 && <>{1}</>}", Some(json!([{"allowExpressions": true}]))),
    ];

    let fail = vec![
        (r"<></>", None),
        (r"<>{}</>", None),
        (r"<p>moo<>foo</></p>", None),
        (r"<>{meow}</>", None),
        (r"<p><>{meow}</></p>", None),
        (r"<><div/></>", None),
        (
            r"
            <>
              <div/>
            </>
        ",
            None,
        ),
        (r"<Fragment />", None),
        (
            r"
                <React.Fragment>
                  <Foo />
                </React.Fragment>
            ",
            None,
        ),
        (r"<Eeee><>foo</></Eeee>", None),
        (r"<div><>foo</></div>", None),
        (r#"<div><>{"a"}{"b"}</></div>"#, None),
        (r#"<div><>{"a"}{"b"}</></div>"#, None),
        (
            r#"
            <section>
              <Eeee />
              <Eeee />
              <>{"a"}{"b"}</>
            </section>"#,
            None,
        ),
        (r#"<div><Fragment>{"a"}{"b"}</Fragment></div>"#, None),
        (
            r"
            <section>
              git<>
                <b>hub</b>.
              </>

              git<> <b>hub</b></>
            </section>
            ",
            None,
        ),
        (r#"<div>a <>{""}{""}</> a</div>"#, None),
        (
            r"
            const Comp = () => (
              <html>
                <React.Fragment />
              </html>
            );
        ",
            None,
        ),
        (r"<><Foo>{moo}</Foo></>", None),
    ];

    let fix = vec![
        (r"<></>", r"<></>", None),
        (r"<>{}</>", r"<>{}</>", None),
        (r"<p>moo<>foo</></p>", r"<p>moofoo</p>", None),
        (r"<>{meow}</>", r"<>{meow}</>", None),
        (r"<p><>{meow}</></p>", r"<p>{meow}</p>", None),
        (r"<><div/></>", r"<div/>", None),
        (
            r"<>
              <div/>
            </>",
            r"<div/>",
            None,
        ),
        (r"<Fragment />", r"<Fragment />", None),
        (
            r"<React.Fragment>
                  <Foo />
                </React.Fragment>",
            r"<Foo />",
            None,
        ),
        (r"<Eeee><>foo</></Eeee>", r"<Eeee><>foo</></Eeee>", None),
        (r"<div><>foo</></div>", r"<div>foo</div>", None),
        (r#"<div><>{"a"}{"b"}</></div>"#, r#"<div>{"a"}{"b"}</div>"#, None),
        (
            r#"
            <section>
              <Eeee />
              <Eeee />
              <>{"a"}{"b"}</>
            </section>
            "#,
            r#"
            <section>
              <Eeee />
              <Eeee />
              {"a"}{"b"}
            </section>
            "#,
            None,
        ),
        (r#"<div><Fragment>{"a"}{"b"}</Fragment></div>"#, r#"<div>{"a"}{"b"}</div>"#, None),
        (
            r"
            <section>
                git<>
                    <b>hub</b>.
                </>

                git<> <b>hub</b></>
            </section>",
            r"
            <section>
                git<b>hub</b>.

                git <b>hub</b>
            </section>",
            None,
        ),
        (r#"<div>a <>{""}{""}</> a</div>"#, r#"<div>a {""}{""} a</div>"#, None),
        (
            r"
            const Comp = () => (
              <html>
                <React.Fragment />
              </html>
            );
            ",
            r"
            const Comp = () => (
              <html>
                
              </html>
            );
            ",
            None,
        ),
        (r"<><Foo>{moo}</Foo></>", r"<Foo>{moo}</Foo>", Some(json!([{"allowExpressions": true}]))),
    ];

    Tester::new(JsxNoUselessFragment::NAME, JsxNoUselessFragment::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
