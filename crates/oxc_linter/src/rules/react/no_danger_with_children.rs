use oxc_ast::{
    AstKind,
    ast::{Argument, Expression, JSXAttributeItem, JSXAttributeName, JSXChild, ObjectPropertyKind},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_danger_with_children_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Only set one of `children` or `props.dangerouslySetInnerHTML`")
            .with_help("`dangerouslySetInnerHTML` is not compatible with also passing children and React will throw a warning at runtime.")
            .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoDangerWithChildren;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows when a DOM element is using both `children` and `dangerouslySetInnerHTML` properties.
    ///
    /// ### Why is this bad?
    ///
    /// React will throw a warning if this rule is ignored and both `children` and `dangerouslySetInnerHTML` are used.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// <div dangerouslySetInnerHTML={{ __html: "HTML" }}>Children</div>
    /// React.createElement(
    ///     "div",
    ///     { dangerouslySetInnerHTML: { __html: "HTML" } },
    ///     "Children"
    /// );
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// <div>Children</div>
    /// <div dangerouslySetInnerHTML={{ __html: "HTML" }} />
    /// ```
    NoDangerWithChildren,
    react,
    correctness
);

impl Rule for NoDangerWithChildren {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::JSXElement(jsx) => {
                // Either children are passed in as a prop like `children={}` or they are nested between the tags.
                let has_children = has_jsx_prop(ctx, node, "children")
                    || (!jsx.children.is_empty() && !is_line_break(&jsx.children[0]));
                if !has_children {
                    return;
                }

                let has_danger_prop = has_jsx_prop(ctx, node, "dangerouslySetInnerHTML");
                if has_danger_prop {
                    ctx.diagnostic(no_danger_with_children_diagnostic(jsx.span));
                }
            }
            AstKind::CallExpression(call_expr) => {
                // Calls with zero or one arguments are safe since they are not proper createElement calls.
                if call_expr.arguments.len() <= 1 {
                    return;
                }
                let Expression::StaticMemberExpression(callee) = &call_expr.callee else {
                    return;
                };

                // Only accept calls like `.createElement(...)`
                if callee.property.name != "createElement" {
                    return;
                }

                let Some(props) = call_expr.arguments.get(1).and_then(Argument::as_expression)
                else {
                    return;
                };

                // If there are three arguments, then it is a JSX element with children.
                // If it's just two arguments, it only has children if the props object has a children property.
                let has_children = if call_expr.arguments.len() == 2 {
                    match props {
                        Expression::ObjectExpression(obj_expr) => {
                            is_object_with_prop_name(&obj_expr.properties, "children")
                        }
                        Expression::Identifier(ident) => {
                            does_object_var_have_prop_name(ctx, node, &ident.name, "children")
                        }
                        _ => false,
                    }
                } else {
                    true
                };

                if !has_children {
                    return;
                }

                let has_danger_prop = match props {
                    Expression::ObjectExpression(obj_expr) => {
                        is_object_with_prop_name(&obj_expr.properties, "dangerouslySetInnerHTML")
                    }
                    Expression::Identifier(ident) => does_object_var_have_prop_name(
                        ctx,
                        node,
                        &ident.name,
                        "dangerouslySetInnerHTML",
                    ),
                    _ => false,
                };

                if has_danger_prop && has_children {
                    ctx.diagnostic(no_danger_with_children_diagnostic(call_expr.span));
                }
            }
            _ => (),
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "<div>Children</div>",
        "<div {...props} />",
        r#"<div dangerouslySetInnerHTML={{ __html: "HTML" }} />"#,
        r#"<div children="Children" />"#,
        r#"
        const props = { dangerouslySetInnerHTML: { __html: "HTML" } };
        <div {...props} />
        "#,
        r#"
        const moreProps = { className: "eslint" };
        const props = { children: "Children", ...moreProps };
        <div {...props} />
        "#,
        r#"
        const otherProps = { children: "Children" };
        const { a, b, ...props } = otherProps;
        <div {...props} />
        "#,
        "<Hello>Children</Hello>",
        r#"<Hello dangerouslySetInnerHTML={{ __html: "HTML" }} />"#,
        r#"
        <Hello dangerouslySetInnerHTML={{ __html: "HTML" }}>
        </Hello>
        "#,
        r#"React.createElement("div", { dangerouslySetInnerHTML: { __html: "HTML" } });"#,
        r#"React.createElement("div", {}, "Children");"#,
        r#"React.createElement("Hello", { dangerouslySetInnerHTML: { __html: "HTML" } });"#,
        r#"React.createElement("Hello", {}, "Children");"#,
        "<Hello {...undefined}>Children</Hello>",
        r#"React.createElement("Hello", undefined, "Children")"#,
        "
        const props = {...props, scratch: {mode: 'edit'}};
        const component = shallow(<TaskEditableTitle {...props} />);
        ",
    ];

    let fail = vec![
        r#"
        <div dangerouslySetInnerHTML={{ __html: "HTML" }}>
            Children
        </div>
        "#,
        r#"<div dangerouslySetInnerHTML={{ __html: "HTML" }} children="Children" />"#,
        r#"
        const props = { dangerouslySetInnerHTML: { __html: "HTML" } };
        <div {...props}>Children</div>
        "#,
        r#"
        const props = { children: "Children", dangerouslySetInnerHTML: { __html: "HTML" } };
        <div {...props} />
        "#,
        r#"
        <Hello dangerouslySetInnerHTML={{ __html: "HTML" }}>
            Children
        </Hello>
        "#,
        r#"<Hello dangerouslySetInnerHTML={{ __html: "HTML" }} children="Children" />"#,
        r#"<Hello dangerouslySetInnerHTML={{ __html: "HTML" }}> </Hello>"#,
        r#"
        React.createElement(
            "div",
            { dangerouslySetInnerHTML: { __html: "HTML" } },
            "Children"
        );
        "#,
        r#"
        React.createElement(
            "div",
            {
                dangerouslySetInnerHTML: { __html: "HTML" },
                children: "Children",
            }
        );
        "#,
        r#"
        React.createElement(
            "Hello",
            { dangerouslySetInnerHTML: { __html: "HTML" } },
            "Children"
        );
        "#,
        r#"
        React.createElement(
            "Hello",
            {
                dangerouslySetInnerHTML: { __html: "HTML" },
                children: "Children",
            }
        );
        "#,
        r#"
        const props = { dangerouslySetInnerHTML: { __html: "HTML" } };
        React.createElement("div", props, "Children");
        "#,
        r#"
        const props = { children: "Children", dangerouslySetInnerHTML: { __html: "HTML" } };
        React.createElement("div", props);
        "#,
        r#"
        const moreProps = { children: "Children" };
        const otherProps = { ...moreProps };
        const props = { ...otherProps, dangerouslySetInnerHTML: { __html: "HTML" } };
        React.createElement("div", props);
        "#,
    ];

    Tester::new(NoDangerWithChildren::NAME, NoDangerWithChildren::PLUGIN, pass, fail)
        .test_and_snapshot();
}

fn is_whitespace(s: &str) -> bool {
    s.chars().all(char::is_whitespace)
}

fn is_line_break(child: &JSXChild) -> bool {
    let JSXChild::Text(text) = child else {
        return false;
    };
    let is_multi_line = text.value.contains('\n');
    is_multi_line && is_whitespace(text.value.as_str())
}

/// Given a JSX element, find the JSXAttributeItem with the given name.
/// If there are spread props, it will search within those as well.
fn has_jsx_prop(ctx: &LintContext, node: &AstNode, prop_name: &'static str) -> bool {
    let AstKind::JSXElement(jsx) = node.kind() else {
        return false;
    };

    jsx.opening_element.attributes.iter().any(|attr| match attr {
        JSXAttributeItem::Attribute(attr) => {
            let JSXAttributeName::Identifier(ident) = &attr.name else {
                return false;
            };
            ident.name == prop_name
        }
        JSXAttributeItem::SpreadAttribute(attr) => {
            let Some(ident) = attr.argument.get_identifier_reference() else {
                return false;
            };
            does_object_var_have_prop_name(ctx, node, ident.name.as_str(), prop_name)
        }
    })
}

/// Given a variable name, finds the variable and checks if it is an object that has a property
/// by the given name, either by directly being set or by being spread into the object.
fn does_object_var_have_prop_name(
    ctx: &LintContext,
    node: &AstNode,
    name: &str,
    prop_name: &str,
) -> bool {
    let Some(symbol) = &find_var_in_scope(ctx, node, name) else {
        return false;
    };

    let AstKind::VariableDeclarator(var_decl) = symbol.kind() else {
        return false;
    };

    let Some(init) = &var_decl.init else {
        return false;
    };

    let Expression::ObjectExpression(obj_expr) = init else {
        return false;
    };

    obj_expr.properties.iter().any(|prop| match prop {
        ObjectPropertyKind::ObjectProperty(obj_prop) => {
            obj_prop.key.static_name().is_some_and(|key| key == prop_name)
        }
        ObjectPropertyKind::SpreadProperty(spread_prop) => {
            let Some(ident) = spread_prop.argument.get_identifier_reference() else {
                return false;
            };
            // If the next symbol is the same as the current symbol, then there is a cycle,
            // for example: `const props = {...props}`, so we will stop searching.
            if let Some(next_symbol) = find_var_in_scope(ctx, node, ident.name.as_str())
                && next_symbol.id() == symbol.id()
            {
                return false;
            }

            does_object_var_have_prop_name(ctx, symbol, ident.name.as_str(), prop_name)
        }
    })
}

/// Given the name of an identifier, find the variable declaration in the current scope.
fn find_var_in_scope<'c>(
    ctx: &'c LintContext,
    node: &AstNode,
    name: &str,
) -> Option<&'c AstNode<'c>> {
    ctx.scoping()
        .find_binding(node.scope_id(), name)
        .map(|symbol_id| ctx.semantic().symbol_declaration(symbol_id))
}

/// Returns whether a given object has a property with the given name.
fn is_object_with_prop_name(
    obj_props: &oxc_allocator::Vec<'_, ObjectPropertyKind<'_>>,
    prop_name: &str,
) -> bool {
    obj_props.iter().any(|prop| {
        let ObjectPropertyKind::ObjectProperty(obj_prop) = prop else {
            return false;
        };
        obj_prop.key.static_name().is_some_and(|key| key == prop_name)
    })
}
