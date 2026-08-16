use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use oxc_ast::{
    AstKind,
    ast::{Expression, JSXAttributeValue, JSXOpeningElement},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    AstNode,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
    utils::is_react_component_name,
};

fn bind_call_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("JSX props should not use `.bind()`")
        .with_help(
            "Bind the handler outside of render, e.g. in the constructor or with a class property.",
        )
        .with_label(span)
}

fn arrow_func_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("JSX props should not use arrow functions")
        .with_help("Extract the arrow function into a stable reference, e.g. a class method or a `useCallback` hook.")
        .with_label(span)
}

fn func_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("JSX props should not use functions")
        .with_help("Extract the function into a stable reference, e.g. a class method or a `useCallback` hook.")
        .with_label(span)
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
struct ConfigElement0 {
    #[serde(rename = "ignoreDOMComponents")]
    ignore_dom_components: bool,
    allow_bind: bool,
    allow_functions: bool,
    ignore_refs: bool,
    allow_arrow_functions: bool,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, JsonSchema)]
pub struct JsxNoBind(ConfigElement0);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows `.bind()` calls, arrow functions, and function expressions in
    /// JSX props, including when they are assigned to a local `const` or
    /// declared as a local function and then passed as a prop.
    ///
    /// ### Why is this bad?
    ///
    /// A `.bind()` call or a function created during render produces a brand
    /// new function on every render. Child components receiving it as a prop
    /// see a changed reference each time, which defeats memoization
    /// (`React.memo`, `shouldComponentUpdate`, `PureComponent`) and causes
    /// unnecessary re-renders.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// <Foo onClick={this._handleClick.bind(this)} />
    /// <Foo onClick={() => console.log('Hello!')} />
    /// <Foo onClick={function () { console.log('Hello!'); }} />
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// <Foo onClick={this._handleClick} />
    /// ```
    JsxNoBind,
    react,
    perf,
    config = JsxNoBind,
    version = "next",
    short_description = "Disallow `.bind()` or arrow functions in JSX props.",
);

#[derive(Debug, Clone, Copy)]
enum ViolationType {
    BindCall,
    ArrowFunc,
    Func,
}

impl ConfigElement0 {
    fn get_violation_type(&self, expr: &Expression) -> Option<ViolationType> {
        match expr.get_inner_expression() {
            Expression::CallExpression(call) if !self.allow_bind => {
                if let Expression::StaticMemberExpression(member) = &call.callee
                    && member.property.name == "bind"
                {
                    Some(ViolationType::BindCall)
                } else {
                    None
                }
            }
            Expression::ConditionalExpression(cond) => self
                .get_violation_type(&cond.test)
                .or_else(|| self.get_violation_type(&cond.consequent))
                .or_else(|| self.get_violation_type(&cond.alternate)),
            Expression::ArrowFunctionExpression(_) if !self.allow_arrow_functions => {
                Some(ViolationType::ArrowFunc)
            }
            Expression::FunctionExpression(_) if !self.allow_functions => Some(ViolationType::Func),
            _ => None,
        }
    }
}

fn is_dom_component(element: &JSXOpeningElement) -> bool {
    element.name.get_identifier_name().is_some_and(|name| !is_react_component_name(&name))
}

impl Rule for JsxNoBind {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXAttribute(attr) = node.kind() else {
            return;
        };
        let config = &self.0;

        if config.ignore_refs && attr.name.as_identifier().is_some_and(|name| name.name == "ref") {
            return;
        }
        let Some(JSXAttributeValue::ExpressionContainer(container)) = &attr.value else {
            return;
        };
        let Some(expr) = container.expression.as_expression() else {
            return;
        };

        if config.ignore_dom_components
            && let AstKind::JSXOpeningElement(opening) = ctx.nodes().parent_kind(node.id())
            && is_dom_component(opening)
        {
            return;
        }

        let expr = expr.get_inner_expression();

        // A function created during render may be assigned to a `const` (or
        // declared as a local function) and then passed as a prop, so resolve
        // identifiers to their declaration.
        if let Expression::Identifier(ident) = expr {
            let Some(symbol_id) = ctx.scoping().get_reference(ident.reference_id()).symbol_id()
            else {
                return;
            };
            // Functions created at the root scope are not re-created on each
            // render; the original rule likewise only tracks declarations
            // inside a block.
            if ctx.scoping().symbol_scope_id(symbol_id) == ctx.scoping().root_scope_id() {
                return;
            }
            let decl_node = ctx.nodes().get_node(ctx.scoping().symbol_declaration(symbol_id));
            let violation_type = match decl_node.kind() {
                AstKind::VariableDeclarator(decl) => {
                    let AstKind::VariableDeclaration(parent_decl) =
                        ctx.nodes().parent_kind(decl_node.id())
                    else {
                        return;
                    };
                    // Only `const` is supported, matching the original rule.
                    if !parent_decl.kind.is_const() {
                        return;
                    }
                    decl.init.as_ref().and_then(|init| config.get_violation_type(init))
                }
                AstKind::Function(func) if func.is_declaration() && !config.allow_functions => {
                    Some(ViolationType::Func)
                }
                _ => None,
            };
            if let Some(violation_type) = violation_type {
                report(violation_type, attr.span, ctx);
            }
            return;
        }

        if let Some(violation_type) = config.get_violation_type(expr) {
            report(violation_type, attr.span, ctx);
        }
    }
}

fn report(violation_type: ViolationType, span: Span, ctx: &LintContext<'_>) {
    let diagnostic = match violation_type {
        ViolationType::BindCall => bind_call_diagnostic(span),
        ViolationType::ArrowFunc => arrow_func_diagnostic(span),
        ViolationType::Func => func_diagnostic(span),
    };
    ctx.diagnostic(diagnostic);
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("<div onClick={this._handleClick}></div>", None),
        ("<div onClick={this._handleClick}></div>", Some(serde_json::json!([{}]))),
        ("<Foo onClick={this._handleClick} />", None),
        ("<Foo onClick={this._handleClick} />", Some(serde_json::json!([{}]))),
        ("<div meaningOfLife={42}></div>", None),
        ("<div onClick={getHandler()}></div>", None),
        (
            "<div ref={c => this._input = c}></div>",
            Some(serde_json::json!([{ "ignoreRefs": true }])),
        ),
        (
            "<div ref={this._refCallback.bind(this)}></div>",
            Some(serde_json::json!([{ "ignoreRefs": true }])),
        ),
        (
            "<div ref={function (c) {this._input = c}}></div>",
            Some(serde_json::json!([{ "ignoreRefs": true }])),
        ),
        (
            "<div onClick={this._handleClick.bind(this)}></div>",
            Some(serde_json::json!([{ "allowBind": true }])),
        ),
        (
            r#"<div onClick={() => alert("1337")}></div>"#,
            Some(serde_json::json!([{ "allowArrowFunctions": true }])),
        ),
        (
            r#"<div onClick={async () => alert("1337")}></div>"#,
            Some(serde_json::json!([{ "allowArrowFunctions": true }])),
        ),
        (
            r#"<div onClick={function () { alert("1337") }}></div>"#,
            Some(serde_json::json!([{ "allowFunctions": true }])),
        ),
        (
            r#"<div onClick={function * () { alert("1337") }}></div>"#,
            Some(serde_json::json!([{ "allowFunctions": true }])),
        ),
        (
            r#"<div onClick={async function () { alert("1337") }}></div>"#,
            Some(serde_json::json!([{ "allowFunctions": true }])),
        ),
        (
            "
                    class Hello extends Component {
                      render() {
                        return <div>Hello</div>;
                      }
                    }
                    export default connect()(Hello);
                  ",
            Some(serde_json::json!([{ "allowBind": true }])),
        ),
        (
            r#"
                    var DocumentRow = Backbone.View.extend({
                      tagName: "li",
                      render: function() {
                        this.onTap.bind(this);
                      }
                    });
                  "#,
            None,
        ),
        (
            "
                    const foo = {
                      render: function() {
                        this.onTap.bind(this);
                        return true;
                      }
                    };
                  ",
            None,
        ),
        (
            "
                    const foo = {
                      render() {
                        this.onTap.bind(this);
                        return true;
                      }
                    };
                  ",
            None,
        ),
        (
            "
                    class Hello extends Component {
                      render() {
                        const click = this.onTap.bind(this);
                        return <div onClick={onClick}>Hello</div>;
                      }
                    };
                  ",
            None,
        ),
        (
            "
                    class Hello extends Component {
                      render() {
                        foo.onClick = this.onTap.bind(this);
                        return <div onClick={onClick}>Hello</div>;
                      }
                    };
                  ",
            None,
        ),
        (
            r#"
                    class Hello extends Component {
                      render() {
                        return (<div>{
                          this.props.list.map(this.wrap.bind(this, "span"))
                        }</div>);
                      }
                    };
                  "#,
            None,
        ),
        (
            "
                    class Hello extends Component {
                      render() {
                        const click = () => true;
                        return <div onClick={onClick}>Hello</div>;
                      }
                    };
                  ",
            None,
        ),
        (
            r#"
                    class Hello extends Component {
                      render() {
                        return (<div>{
                          this.props.list.map(item => <item hello="true"/>)
                        }</div>);
                      }
                    };
                  "#,
            None,
        ),
        (
            r#"
                    var Hello = React.createClass({
                      render: function() {
                        return (<div>{
                          this.props.list.map(this.wrap.bind(this, "span"))
                        }</div>);
                      }
                    });
                  "#,
            None,
        ),
        (
            "
                    var Hello = React.createClass({
                      render: function() {
                        const click = () => true
                        return <div onClick={onClick}>Hello</div>;
                      }
                    });
                  ",
            None,
        ),
        (
            r#"
                    class Hello23 extends React.Component {
                      renderDiv = () => {
                        const onClick = this.doSomething.bind(this, "no")
                        return <div onClick={click}>Hello</div>;
                      }
                    };
                  "#,
            None,
        ),
        (
            r#"
                    class Hello23 extends React.Component {
                      renderDiv = async () => {
                        return (<div>{
                          this.props.list.map(this.wrap.bind(this, "span"))
                        }</div>);
                      }
                    };
                  "#,
            None,
        ),
        (
            "
                    class Hello extends Component {
                      render() {
                        let click;
                        return <div onClick={onClick}>Hello</div>;
                      }
                    }
                  ",
            None,
        ),
        (
            "<div onClick={this._handleClick.bind(this)}></div>",
            Some(serde_json::json!([{ "ignoreDOMComponents": true }])),
        ),
        (
            r#"<div onClick={() => alert("1337")}></div>"#,
            Some(serde_json::json!([{ "ignoreDOMComponents": true }])),
        ),
        (
            r#"<div onClick={function () { alert("1337") }}></div>"#,
            Some(serde_json::json!([{ "ignoreDOMComponents": true }])),
        ),
        (
            "
                    function click() { return true; }
                    class Hello23 extends React.Component {
                      renderDiv() {
                        return <div onClick={click}>Hello</div>;
                      }
                    };
                  ",
            None,
        ),
    ];

    let fail = vec![
        ("<div onClick={this._handleClick.bind(this)}></div>", None),
("<div onClick={someGlobalFunction.bind(this)}></div>", None),
("<div onClick={window.lol.bind(this)}></div>", None),
("<div ref={this._refCallback.bind(this)}></div>", None),
("
                    var Hello = createReactClass({
                      render: function() {
                        const click = this.someMethod.bind(this);
                        return <div onClick={click}>Hello {this.state.name}</div>;
                      }
                    });
                  ", None),
("
                    class Hello23 extends React.Component {
                      render() {
                        const click = this.someMethod.bind(this);
                        return <div onClick={click}>Hello {this.state.name}</div>;
                      }
                    };
                  ", None),
(r#"
                    class Hello23 extends React.Component {
                      renderDiv() {
                        const click = this.doSomething.bind(this, "no")
                        return <div onClick={click}>Hello</div>;
                      }
                    };
                  "#, None),
(r#"
                    class Hello23 extends React.Component {
                      renderDiv = () => {
                        const click = this.doSomething.bind(this, "no")
                        return <div onClick={click}>Hello</div>;
                      }
                    };
                  "#, None),
(r#"
                    class Hello23 extends React.Component {
                      renderDiv = async () => {
                        const click = this.doSomething.bind(this, "no")
                        return <div onClick={click}>Hello</div>;
                      }
                    };
                  "#, None),
("
                    const foo = {
                      render: ({onClick}) => (
                        <div onClick={onClick.bind(this)}>Hello</div>
                      )
                    };
                  ", None),
(r#"
                    var Hello = React.createClass({
                      render: function() {
                      return <div onClick={this.doSomething.bind(this, "hey")} />
                      }
                    });
                  "#, None),
(r#"
                    var Hello = React.createClass({
                      render: function() {
                        const doThing = this.doSomething.bind(this, "hey")
                        return <div onClick={doThing} />
                      }
                    });
                  "#, None),
(r#"
                    class Hello23 extends React.Component {
                      renderDiv = () => {
                        const click = () => true
                        const renderStuff = () => {
                          const click = this.doSomething.bind(this, "hey")
                          return <div onClick={click} />
                        }
                        return <div onClick={click}>Hello</div>;
                      }
                    };
                  "#, None),
("
                    const foo = {
                      render: ({onClick}) => (
                        <div onClick={(returningBoolean()) ? onClick.bind(this) : onClick.bind(this)}>Hello</div>
                      )
                    };
                  ", None),
("
                    const foo = {
                      render: ({onClick}) => (
                        <div onClick={(returningBoolean()) ? onClick.bind(this) : handleClick()}>Hello</div>
                      )
                    };
                  ", None),
("
                    const foo = {
                      render: ({onClick}) => (
                        <div onClick={(returningBoolean()) ? handleClick() : this.onClick.bind(this)}>Hello</div>
                      )
                    };
                  ", None),
("
                    const foo = {
                      render: ({onClick}) => (
                        <div onClick={returningBoolean.bind(this) ? handleClick() : onClick()}>Hello</div>
                      )
                    };
                  ", None),
(r#"<div onClick={() => alert("1337")}></div>"#, None),
(r#"<div onClick={async () => alert("1337")}></div>"#, None),
("<div onClick={() => 42}></div>", None),
("<div onClick={param => { first(); second(); }}></div>", None),
("<div ref={c => this._input = c}></div>", None),
("
                    class Hello23 extends React.Component {
                      renderDiv = () => {
                        const click = () => true
                        return <div onClick={click}>Hello</div>;
                      }
                    };
                  ", None),
("
                    class Hello23 extends React.Component {
                      renderDiv = async () => {
                        const click = () => true
                        return <div onClick={click}>Hello</div>;
                      }
                    };
                  ", None),
("
                    class Hello23 extends React.Component {
                      renderDiv = async () => {
                        const click = async () => true
                        return <div onClick={click}>Hello</div>;
                      }
                    };
                  ", None),
("
                    var Hello = React.createClass({
                      render: function() {
                      return <div onClick={() => true} />
                      }
                    });
                  ", None),
("
                    var Hello = React.createClass({
                      render: function() {
                      return <div onClick={async () => true} />
                      }
                    });
                  ", None),
("
                    var Hello = React.createClass({
                      render: function() {
                        const doThing = () => true
                        return <div onClick={doThing} />
                      }
                    });
                  ", None),
("
                    var Hello = React.createClass({
                      render: function() {
                        const doThing = async () => true
                        return <div onClick={doThing} />
                      }
                    });
                  ", None),
(r#"<div onClick={function () { alert("1337") }}></div>"#, None),
(r#"<div onClick={function * () { alert("1337") }}></div>"#, None),
(r#"<div onClick={async function () { alert("1337") }}></div>"#, None),
("<div ref={function (c) { this._input = c }}></div>", None),
("
                    class Hello23 extends React.Component {
                      renderDiv = () => {
                        const click = function () { return true }
                        return <div onClick={click}>Hello</div>;
                      }
                    };
                  ", None),
("
                    class Hello23 extends React.Component {
                      renderDiv = () => {
                        const click = function * () { return true }
                        return <div onClick={click}>Hello</div>;
                      }
                    };
                  ", None),
("
                    class Hello23 extends React.Component {
                      renderDiv = async () => {
                        const click = function () { return true }
                        return <div onClick={click}>Hello</div>;
                      }
                    };
                  ", None),
("
                    class Hello23 extends React.Component {
                      renderDiv = async () => {
                        const click = async function () { return true }
                        return <div onClick={click}>Hello</div>;
                      }
                    };
                  ", None),
("
                    var Hello = React.createClass({
                      render: function() {
                      return <div onClick={function () { return true }} />
                      }
                    });
                  ", None),
("
                    var Hello = React.createClass({
                      render: function() {
                      return <div onClick={function * () { return true }} />
                      }
                    });
                  ", None),
("
                    var Hello = React.createClass({
                      render: function() {
                      return <div onClick={async function () { return true }} />
                      }
                    });
                  ", None),
("
                    var Hello = React.createClass({
                      render: function() {
                        const doThing = function () { return true }
                        return <div onClick={doThing} />
                      }
                    });
                  ", None),
("
                    var Hello = React.createClass({
                      render: function() {
                        const doThing = async function () { return true }
                        return <div onClick={doThing} />
                      }
                    });
                  ", None),
("
                    var Hello = React.createClass({
                      render: function() {
                        const doThing = function * () { return true }
                        return <div onClick={doThing} />
                      }
                    });
                  ", None),
("
                    class Hello23 extends React.Component {
                      renderDiv() {
                        function click() { return true; }
                        return <div onClick={click}>Hello</div>;
                      }
                    };
                  ", None),
("<Foo onClick={this._handleClick.bind(this)} />", Some(serde_json::json!([{ "ignoreDOMComponents": true }])))
    ];

    Tester::new(JsxNoBind::NAME, JsxNoBind::PLUGIN, pass, fail).test_and_snapshot();
}
