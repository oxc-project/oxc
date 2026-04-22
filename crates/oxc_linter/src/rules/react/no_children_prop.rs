use oxc_ast::{
    AstKind,
    ast::{Argument, JSXAttributeName, ObjectPropertyKind},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{AstNode, context::LintContext, rule::Rule, utils::is_create_element_call};

fn no_children_prop_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Avoid passing children using a prop.")
        .with_help("The canonical way to pass children in React is to use JSX elements")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoChildrenProp;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Checks that children are not passed using a prop.
    ///
    /// ### Why is this bad?
    ///
    /// Children should always be actual children, not passed in as a prop.
    /// When using JSX, the children should be nested between the opening and closing tags.
    /// When not using JSX, the children should be passed as additional arguments to `React.createElement`.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// <div children='Children' />
    ///
    /// <MyComponent children={<AnotherComponent />} />
    /// <MyComponent children={['Child 1', 'Child 2']} />
    /// React.createElement("div", { children: 'Children' })
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
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
    /// ```
    NoChildrenProp,
    react,
    correctness,
    version = "0.0.14",
);

impl Rule for NoChildrenProp {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::JSXAttribute(attr) => {
                let JSXAttributeName::Identifier(attr_ident) = &attr.name else {
                    return;
                };
                if attr_ident.name == "children" {
                    ctx.diagnostic(no_children_prop_diagnostic(attr_ident.span));
                }
            }
            AstKind::CallExpression(call_expr) => {
                if is_create_element_call(call_expr)
                    && let Some(Argument::ObjectExpression(obj_expr)) = call_expr.arguments.get(1)
                    && let Some(span) = obj_expr.properties.iter().find_map(|prop| {
                        if let ObjectPropertyKind::ObjectProperty(prop) = prop
                            && prop.key.is_specific_static_name("children")
                        {
                            return Some(prop.key.span());
                        }

                        None
                    })
                {
                    ctx.diagnostic(no_children_prop_diagnostic(span));
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
        "<div />;",
        "<div></div>;",
        r#"React.createElement("div", {});"#,
        r#"React.createElement("div", undefined);"#,
        r#"<div className="class-name"></div>;"#,
        r#"React.createElement("div", {className: "class-name"});"#,
        "<div>Children</div>;",
        r#"React.createElement("div", "Children");"#,
        r#"React.createElement("div", {}, "Children");"#,
        r#"React.createElement("div", undefined, "Children");"#,
        r#"<div className="class-name">Children</div>;"#,
        r#"React.createElement("div", {className: "class-name"}, "Children");"#,
        "<div><div /></div>;",
        r#"React.createElement("div", React.createElement("div"));"#,
        r#"React.createElement("div", {}, React.createElement("div"));"#,
        r#"React.createElement("div", undefined, React.createElement("div"));"#,
        "<div><div /><div /></div>;",
        r#"React.createElement("div", React.createElement("div"), React.createElement("div"));"#,
        r#"React.createElement("div", {}, React.createElement("div"), React.createElement("div"));"#,
        r#"React.createElement("div", undefined, React.createElement("div"), React.createElement("div"));"#,
        r#"React.createElement("div", [React.createElement("div"), React.createElement("div")]);"#,
        r#"React.createElement("div", {}, [React.createElement("div"), React.createElement("div")]);"#,
        r#"React.createElement("div", undefined, [React.createElement("div"), React.createElement("div")]);"#,
        "<MyComponent />",
        "React.createElement(MyComponent);",
        "React.createElement(MyComponent, {});",
        "React.createElement(MyComponent, undefined);",
        "<MyComponent>Children</MyComponent>;",
        r#"React.createElement(MyComponent, "Children");"#,
        r#"React.createElement(MyComponent, {}, "Children");"#,
        r#"React.createElement(MyComponent, undefined, "Children");"#,
        r#"<MyComponent className="class-name"></MyComponent>;"#,
        r#"React.createElement(MyComponent, {className: "class-name"});"#,
        r#"<MyComponent className="class-name">Children</MyComponent>;"#,
        r#"React.createElement(MyComponent, {className: "class-name"}, "Children");"#,
        r#"<MyComponent className="class-name" {...props} />;"#,
        r#"foo(MyComponent, {...props, children: "Children"})"#,
        r#"React.createElement(MyComponent, {className: "class-name", ...props});"#,
    ];

    #[rustfmt::skip]
    let fail = vec![
        "<div children />;",
        r#"<div children="Children" />;"#,
        "<div children={<div />} />;",
        "<div children={[<div />, <div />]} />;",
        r#"<div children="Children">Children</div>;"#,
        r#"React.createElement("div", {children: "Children"});"#,
        r#"React.createElement("div", {children: "Children"}, "Children");"#,
        r#"React.createElement("div", {children: React.createElement("div")});"#,
        r#"React.createElement("div", {children: [React.createElement("div"), React.createElement("div")]});"#,
        r#"<MyComponent children="Children" />"#,
        r#"React.createElement(MyComponent, {children: "Children"});"#,
        r#"<MyComponent className="class-name" children="Children" />;"#,
        r#"React.createElement(MyComponent, {children: "Children", className: "class-name"});"#,
        r#"<MyComponent {...props} children="Children" />;"#,
        r#"React.createElement(MyComponent, {...props, children: "Children"})"#,
    ];

    Tester::new(NoChildrenProp::NAME, NoChildrenProp::PLUGIN, pass, fail).test_and_snapshot();
}
