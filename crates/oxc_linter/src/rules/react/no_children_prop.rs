use oxc_ast::{
    ast::{Argument, Expression, JSXAttributeItem, JSXAttributeName, ObjectPropertyKind},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, react_ast_utils::is_create_element_call, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-react/no-children-prop: Avoid passing children using a prop.")]
#[diagnostic(
    severity(warning),
    help("The canonical way to pass children in React is to use JSX elements")
)]
struct NoChildrenPropDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoChildrenProp;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Children should always be actual children, not passed in as a prop.
    ///
    /// When using JSX, the children should be nested between the opening and closing tags.
    ///
    /// When not using JSX, the children should be passed as additional arguments to `React.createElement`.
    ///
    /// ### Example
    /// ```javascript
    /// // Bad
    /// <div children='Children' />
    ///
    /// <MyComponent children={<AnotherComponent />} />
    /// <MyComponent children={['Child 1', 'Child 2']} />
    /// React.createElement("div", { children: 'Children' })
    ///
    /// // Good
    ///
    /// <div>Children</div>
    /// <MyComponent>Children</MyComponent>
    ///
    /// <MyComponent>
    ///   <span>Child 1</span>
    ///   <span>Child 2</span>
    /// </MyComponent>
    ///
    /// React.createElement("div", {}, 'Children')
    /// React.createElement("div", 'Child 1', 'Child 2')
    ///
    ///
    /// ```
    NoChildrenProp,
    correctness
);

impl Rule for NoChildrenProp {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::JSXAttributeItem(JSXAttributeItem::Attribute(attr)) => {
                let JSXAttributeName::Identifier(attr_ident) = &attr.name else { return };
                if attr_ident.name == "children" {
                    ctx.diagnostic(NoChildrenPropDiagnostic(attr.span));
                }
            }
            AstKind::CallExpression(call_expr) => {
                if is_create_element_call(call_expr) {
                    if let Some(Argument::Expression(Expression::ObjectExpression(obj_expr))) =
                        call_expr.arguments.get(1)
                    {
                        if let Some(span) = obj_expr.properties.iter().find_map(|prop| {
                            if let ObjectPropertyKind::ObjectProperty(prop) = prop {
                                if prop.key.is_specific_static_name("children") {
                                    return Some(prop.span);
                                }
                            }

                            None
                        }) {
                            ctx.diagnostic(NoChildrenPropDiagnostic(span));
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    #[rustfmt::skip]
    let pass = vec![
        ("<div />;", None),
        ("<div></div>;", None),
        (r#"React.createElement("div", {});"#, None),
        (r#"React.createElement("div", undefined);"#, None),
        (r#"<div className="class-name"></div>;"#, None),
        (r#"React.createElement("div", {className: "class-name"});"#, None),
        ("<div>Children</div>;", None),
        (r#"React.createElement("div", "Children");"#, None),
        (r#"React.createElement("div", {}, "Children");"#, None),
        (r#"React.createElement("div", undefined, "Children");"#, None),
        (r#"<div className="class-name">Children</div>;"#, None),
        (r#"React.createElement("div", {className: "class-name"}, "Children");"#, None),
        ("<div><div /></div>;", None),
        (r#"React.createElement("div", React.createElement("div"));"#, None),
        (r#"React.createElement("div", {}, React.createElement("div"));"#, None),
        (r#"React.createElement("div", undefined, React.createElement("div"));"#, None),
        ("<div><div /><div /></div>;", None),
        (r#"React.createElement("div", React.createElement("div"), React.createElement("div"));"#, None),
        (r#"React.createElement("div", {}, React.createElement("div"), React.createElement("div"));"#, None),
        (r#"React.createElement("div", undefined, React.createElement("div"), React.createElement("div"));"#, None),
        (r#"React.createElement("div", [React.createElement("div"), React.createElement("div")]);"#, None),
        (r#"React.createElement("div", {}, [React.createElement("div"), React.createElement("div")]);"#, None),
        (r#"React.createElement("div", undefined, [React.createElement("div"), React.createElement("div")]);"#, None),
        ("<MyComponent />", None),
        ("React.createElement(MyComponent);", None),
        ("React.createElement(MyComponent, {});", None),
        ("React.createElement(MyComponent, undefined);", None),
        ("<MyComponent>Children</MyComponent>;", None),
        (r#"React.createElement(MyComponent, "Children");"#, None),
        (r#"React.createElement(MyComponent, {}, "Children");"#, None),
        (r#"React.createElement(MyComponent, undefined, "Children");"#, None),
        (r#"<MyComponent className="class-name"></MyComponent>;"#, None),
        (r#"React.createElement(MyComponent, {className: "class-name"});"#, None),
        (r#"<MyComponent className="class-name">Children</MyComponent>;"#, None),
        (r#"React.createElement(MyComponent, {className: "class-name"}, "Children");"#, None),
        (r#"<MyComponent className="class-name" {...props} />;"#, None),
        (r#"foo(MyComponent, {...props, children: "Children"})"#, None),
        (r#"React.createElement(MyComponent, {className: "class-name", ...props});"#, None),
    ];

    #[rustfmt::skip]
    let fail = vec![
        (r#"<div children />;"#, None),
        (r#"<div children="Children" />;"#, None),
        (r#"<div children={<div />} />;"#, None),
        (r#"<div children={[<div />, <div />]} />;"#, None),
        (r#"<div children="Children">Children</div>;"#, None),
        (r#"React.createElement("div", {children: "Children"});"#, None),
        (r#"React.createElement("div", {children: "Children"}, "Children");"#, None),
        (r#"React.createElement("div", {children: React.createElement("div")});"#, None),
        (r#"React.createElement("div", {children: [React.createElement("div"), React.createElement("div")]});"#, None),
        (r#"<MyComponent children="Children" />"#, None),
        (r#"React.createElement(MyComponent, {children: "Children"});"#, None),
        (r#"<MyComponent className="class-name" children="Children" />;"#, None),
        (r#"React.createElement(MyComponent, {children: "Children", className: "class-name"});"#, None),
        (r#"<MyComponent {...props} children="Children" />;"#, None),
        (r#"React.createElement(MyComponent, {...props, children: "Children"})"#, None),
    ];

    Tester::new(NoChildrenProp::NAME, pass, fail).test_and_snapshot();
}
