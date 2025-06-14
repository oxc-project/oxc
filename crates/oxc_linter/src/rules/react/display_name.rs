use oxc_ast::{
    AstKind,
    ast::{Argument, ClassElement, Expression, ObjectPropertyKind, PropertyKey, Statement},
    match_member_expression,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_span::GetSpan;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};
use oxc_macros::declare_oxc_lint;

fn display_name_diagnostic(span: Span) -> OxcDiagnostic {
    // See <https://oxc.rs/docs/contribute/linter/adding-rules.html#diagnostics> for details
    OxcDiagnostic::warn("React component is missing a displayName property")
        .with_help("Add a displayName property to the component")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct DisplayName;

// See <https://github.com/oxc-project/oxc/issues/6050> for documentation details.
declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule allows you to name your component. This name is used by React in debugging messages.
    ///
    /// ### Why is this bad?
    ///
    /// When debugging React components, there will be missing identifiers for the components that lack a displayName property.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// var Hello = createReactClass({
    ///   render: function() {
    ///     return <div>Hello {this.props.name}</div>;
    ///   }
    /// });
    ///
    /// const Hello = React.memo(({ a }) => {
    ///   return <>{a}</>
    /// })
    ///
    /// export default ({ a }) => {
    ///   return <>{a}</>
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// var Hello = createReactClass({
    ///   displayName: 'Hello',
    ///   render: function() {
    ///     return <div>Hello {this.props.name}</div>;
    ///   }
    /// });
    ///
    /// const Hello = React.memo(function Hello({ a }) {
    ///   return <>{a}</>
    /// })
    /// ```
    ///
    DisplayName,
    react,
    style,
);

impl Rule for DisplayName {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::VariableDeclaration(decl) => {
                for decl in &decl.declarations {
                    if let Some(init) = &decl.init {
                        // Check for createReactClass
                        if let Expression::CallExpression(call) = init {
                            if let Expression::Identifier(ident) = &call.callee {
                                if ident.name == "createReactClass" {
                                    let mut found_display_name = false;
                                    // Check for displayName property in the object argument
                                    if let Some(arg) = call.arguments.first() {
                                        if let Argument::ObjectExpression(obj) = arg {
                                            for prop in &obj.properties {
                                                if let ObjectPropertyKind::ObjectProperty(prop) = prop {
                                                    if let PropertyKey::StaticIdentifier(key) = &prop.key {
                                                        if key.name == "displayName" {
                                                            // if let Expression::StringLiteral(value) = &prop.value {
                                                                found_display_name = true;
                                                            // }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    if !found_display_name {
                                        ctx.diagnostic(display_name_diagnostic(init.span()));
                                    }
                                }
                            }
                        }
                        // Check for functional components
                        if let Expression::ArrowFunctionExpression(_) = init {
                            // if let Some(name) = decl.id.get_identifier_name() {
                                let mut found_display_name = false;
                                // Check for displayName property
                                if let Some(symbol_id) = decl.id.get_binding_identifiers().first().and_then(|id| id.symbol_id.get()) {
                                    if let Some(display_name) = ctx.scoping().get_resolved_references(symbol_id).find(|r| ctx.reference_name(r) == "displayName") {
                                        let node = ctx.nodes().get_node(display_name.node_id());
                                        if let AstKind::StringLiteral(_) = node.kind() {
                                            found_display_name = true;
                                        }
                                    }
                                }
                                if !found_display_name {
                                    ctx.diagnostic(display_name_diagnostic(init.span()));
                                }
                            // }
                        }
                    }
                }
            }
            AstKind::Class(class) => {
                println!("Class: {:?}", class);

                if let Some(super_class) = &class.super_class {
                    println!("Super class: {:?}", super_class);
                    if let match_member_expression!(Expression) = super_class {
                        let member = super_class.to_member_expression();
                        if let Expression::Identifier(ident) = &member.object() {
                            println!("Ident: {:?}", ident);
                            if ident.name == "React" {
                                if let Some(prop_name) = member.static_property_name() {
                                    println!("Prop name: {:?}", prop_name);
                                    if prop_name == "Component" || prop_name == "PureComponent" {
                                        let mut found_display_name = false;
                                        // Check for static displayName property or getter
                                        for element in &class.body.body {
                                            println!("Element: {:?}", element);

                                            let key = element.property_key();

                                            if let Some(PropertyKey::StaticIdentifier(key)) = key {
                                                println!("Key: {:?}", key);

                                                if key.name == "displayName" {
                                                    found_display_name = true;
                                                }

                                                break;
                                            }




                                            if let ClassElement::MethodDefinition(method) = element {
                                                println!("Method: {:?}", method);
                                                if method.r#static {
                                                    println!("Static method: {:?}", method);
                                                    if let PropertyKey::StaticIdentifier(key) = &method.key {
                                                        if key.name == "displayName" {
                                                            // Check for both string literal and getter method
                                                            if let Some(body) = &method.value.body {
                                                                if let Some(stmt) = body.statements.first() {
                                                                    match stmt {
                                                                        Statement::ExpressionStatement(expr_stmt) => {
                                                                            if let Expression::StringLiteral(_) = &expr_stmt.expression {
                                                                                found_display_name = true;
                                                                            }
                                                                        }
                                                                        Statement::ReturnStatement(ret_stmt) => {
                                                                            if let Some(expr) = &ret_stmt.argument {
                                                                                if let Expression::StringLiteral(_) = expr {
                                                                                    found_display_name = true;
                                                                                }
                                                                            }
                                                                        }
                                                                        _ => {}
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        if !found_display_name {
                                            ctx.diagnostic(display_name_diagnostic(class.span()));
                                        }
                                    }
                                }
                            }
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

    let pass = vec![
        (
            "
			        var Hello = createReactClass({
			          displayName: 'Hello',
			          render: function() {
			            return <div>Hello {this.props.name}</div>;
			          }
			        });
			      ",
            Some(serde_json::json!([{ "ignoreTranspilerName": true }])),
            None,
        ),
        (
            "
			        var Hello = React.createClass({
			          displayName: 'Hello',
			          render: function() {
			            return <div>Hello {this.props.name}</div>;
			          }
			        });
			      ",
            Some(serde_json::json!([{ "ignoreTranspilerName": true }])),
            Some(
                serde_json::json!({ "settings": {        "react": {          "createClass": "createClass",        },      } }),
            ),
        ),
        // (
        //     "
		// 	        class Hello extends React.Component {
		// 	          render() {
		// 	            return <div>Hello {this.props.name}</div>;
		// 	          }
		// 	        }
		// 	        Hello.displayName = 'Hello'
		// 	      ",
        //     Some(serde_json::json!([{ "ignoreTranspilerName": true }])),
        //     None,
        // ),
        (
            "
			        class Hello {
			          render() {
			            return 'Hello World';
			          }
			        }
			      ",
            None,
            None,
        ),
        (
            "
			        class Hello extends Greetings {
			          static text = 'Hello World';
			          render() {
			            return Hello.text;
			          }
			        }
			      ",
            None,
            None,
        ),
        (
            "
			        class Hello {
			          method;
			        }
			      ",
            None,
            None,
        ),
        (
            "
			        class Hello extends React.Component {
			          static get displayName() {
			            return 'Hello';
			          }
			          render() {
			            return <div>Hello {this.props.name}</div>;
			          }
			        }
			      ",
            Some(serde_json::json!([{ "ignoreTranspilerName": true }])),
            None,
        ),
        (
            "
			        class Hello extends React.Component {
			          static displayName = 'Widget';
			          render() {
			            return <div>Hello {this.props.name}</div>;
			          }
			        }
			      ",
            Some(serde_json::json!([{ "ignoreTranspilerName": true }])),
            None,
        ),
        (
            "
			        var Hello = createReactClass({
			          render: function() {
			            return <div>Hello {this.props.name}</div>;
			          }
			        });
			      ",
            None,
            None,
        ),
        (
            "
			        class Hello extends React.Component {
			          render() {
			            return <div>Hello {this.props.name}</div>;
			          }
			        }
			      ",
            None,
            None,
        ),
        (
            "
			        export default class Hello {
			          render() {
			            return <div>Hello {this.props.name}</div>;
			          }
			        }
			      ",
            None,
            None,
        ),
        (
            "
			        var Hello;
			        Hello = createReactClass({
			          render: function() {
			            return <div>Hello {this.props.name}</div>;
			          }
			        });
			      ",
            None,
            None,
        ),
        (
            r#"
			        module.exports = createReactClass({
			          "displayName": "Hello",
			          "render": function() {
			            return <div>Hello {this.props.name}</div>;
			          }
			        });
			      "#,
            None,
            None,
        ),
        (
            "
			        var Hello = createReactClass({
			          displayName: 'Hello',
			          render: function() {
			            let { a, ...b } = obj;
			            let c = { ...d };
			            return <div />;
			          }
			        });
			      ",
            Some(serde_json::json!([{ "ignoreTranspilerName": true }])),
            None,
        ),
        (
            "
			        export default class {
			          render() {
			            return <div>Hello {this.props.name}</div>;
			          }
			        }
			      ",
            None,
            None,
        ),
        (
            "
			        export const Hello = React.memo(function Hello() {
			          return <p />;
			        })
			      ",
            None,
            None,
        ),
        (
            "
			        var Hello = function() {
			          return <div>Hello {this.props.name}</div>;
			        }
			      ",
            None,
            None,
        ),
        (
            "
			        function Hello() {
			          return <div>Hello {this.props.name}</div>;
			        }
			      ",
            None,
            None,
        ),
        (
            "
			        var Hello = () => {
			          return <div>Hello {this.props.name}</div>;
			        }
			      ",
            None,
            None,
        ),
        (
            "
			        module.exports = function Hello() {
			          return <div>Hello {this.props.name}</div>;
			        }
			      ",
            None,
            None,
        ),
        (
            "
			        function Hello() {
			          return <div>Hello {this.props.name}</div>;
			        }
			        Hello.displayName = 'Hello';
			      ",
            Some(serde_json::json!([{ "ignoreTranspilerName": true }])),
            None,
        ),
        (
            "
			        var Hello = () => {
			          return <div>Hello {this.props.name}</div>;
			        }
			        Hello.displayName = 'Hello';
			      ",
            Some(serde_json::json!([{ "ignoreTranspilerName": true }])),
            None,
        ),
        (
            "
			        var Hello = function() {
			          return <div>Hello {this.props.name}</div>;
			        }
			        Hello.displayName = 'Hello';
			      ",
            Some(serde_json::json!([{ "ignoreTranspilerName": true }])),
            None,
        ),
        (
            "
			        var Mixins = {
			          Greetings: {
			            Hello: function() {
			              return <div>Hello {this.props.name}</div>;
			            }
			          }
			        }
			        Mixins.Greetings.Hello.displayName = 'Hello';
			      ",
            Some(serde_json::json!([{ "ignoreTranspilerName": true }])),
            None,
        ),
        (
            "
			        var Hello = createReactClass({
			          render: function() {
			            return <div>{this._renderHello()}</div>;
			          },
			          _renderHello: function() {
			            return <span>Hello {this.props.name}</span>;
			          }
			        });
			      ",
            None,
            None,
        ),
        (
            "
			        var Hello = createReactClass({
			          displayName: 'Hello',
			          render: function() {
			            return <div>{this._renderHello()}</div>;
			          },
			          _renderHello: function() {
			            return <span>Hello {this.props.name}</span>;
			          }
			        });
			      ",
            Some(serde_json::json!([{ "ignoreTranspilerName": true }])),
            None,
        ),
        (
            "
			        const Mixin = {
			          Button() {
			            return (
			              <button />
			            );
			          }
			        };
			      ",
            None,
            None,
        ),
        (
            "
			        var obj = {
			          pouf: function() {
			            return any
			          }
			        };
			      ",
            Some(serde_json::json!([{ "ignoreTranspilerName": true }])),
            None,
        ),
        (
            "
			        var obj = {
			          pouf: function() {
			            return any
			          }
			        };
			      ",
            None,
            None,
        ),
        (
            "
			        export default {
			          renderHello() {
			            let {name} = this.props;
			            return <div>{name}</div>;
			          }
			        };
			      ",
            None,
            None,
        ),
        (
            "
			        import React, { createClass } from 'react';
			        export default createClass({
			          displayName: 'Foo',
			          render() {
			            return <h1>foo</h1>;
			          }
			        });
			      ",
            Some(serde_json::json!([{ "ignoreTranspilerName": true }])),
            Some(
                serde_json::json!({ "settings": {        "react": {          "createClass": "createClass",        },      } }),
            ),
        ),
        (
            r#"
			        import React, {Component} from "react";
			        function someDecorator(ComposedComponent) {
			          return class MyDecorator extends Component {
			            render() {return <ComposedComponent {...this.props} />;}
			          };
			        }
			        module.exports = someDecorator;
			      "#,
            None,
            None,
        ),
        (
            r#"
			        import React, {createElement} from "react";
			        const SomeComponent = (props) => {
			          const {foo, bar} = props;
			          return someComponentFactory({
			            onClick: () => foo(bar("x"))
			          });
			        };
			      "#,
            None,
            None,
        ),
        (
            "
			        const element = (
			          <Media query={query} render={() => {
			            renderWasCalled = true
			            return <div/>
			          }}/>
			        )
			      ",
            None,
            None,
        ),
        (
            "
			        const element = (
			          <Media query={query} render={function() {
			            renderWasCalled = true
			            return <div/>
			          }}/>
			        )
			      ",
            None,
            None,
        ),
        (
            "
			        module.exports = {
			          createElement: tagName => document.createElement(tagName)
			        };
			      ",
            None,
            None,
        ),
        (
            r#"
			        const { createElement } = document;
			        createElement("a");
			      "#,
            None,
            None,
        ),
        (
            "
			        import React from 'react'
			        import { string } from 'prop-types'

			        function Component({ world }) {
			          return <div>Hello {world}</div>
			        }

			        Component.propTypes = {
			          world: string,
			        }

			        export default React.memo(Component)
			      ",
            None,
            None,
        ),
        (
            "
			        import React from 'react'

			        const ComponentWithMemo = React.memo(function Component({ world }) {
			          return <div>Hello {world}</div>
			        })
			      ",
            None,
            None,
        ),
        (
            "
			        import React from 'react';

			        const Hello = React.memo(function Hello() {
			          return;
			        });
			      ",
            None,
            None,
        ),
        (
            "
			        import React from 'react'

			        const ForwardRefComponentLike = React.forwardRef(function ComponentLike({ world }, ref) {
			          return <div ref={ref}>Hello {world}</div>
			        })
			      ",
            None,
            None,
        ),
        (
            r#"
			        function F() {
			          let items = [];
			          let testData = [
			            {a: "test1", displayName: "test2"}, {a: "test1", displayName: "test2"}];
			          for (let item of testData) {
			              items.push({a: item.a, b: item.displayName});
			          }
			          return <div>{items}</div>;
			        }
			      "#,
            None,
            None,
        ),
        // NOTE: this test throws an unexpected token error.
        // (
        //     r#"
        // 	        import {Component} from "react";
        // 	        type LinkProps = {...{}};
        // 	        class Link extends Component<LinkProps> {}
        // 	      "#,
        //     None,
        //     None,
        // ),
        (
            r#"
			        const x = {
			          title: "URL",
			          dataIndex: "url",
			          key: "url",
			          render: url => (
			            <a href={url} target="_blank" rel="noopener noreferrer">
			              <p>lol</p>
			            </a>
			          )
			        }
			      "#,
            None,
            None,
        ),
        (
            "
			        const renderer = a => function Component(listItem) {
			          return <div>{a} {listItem}</div>;
			        };
			      ",
            None,
            None,
        ),
        (
            "
			        const Comp = React.forwardRef((props, ref) => <main />);
			        Comp.displayName = 'MyCompName';
			      ",
            None,
            None,
        ),
        (
            r#"
			        const Comp = React.forwardRef((props, ref) => <main data-as="yes" />) as SomeComponent;
			        Comp.displayName = 'MyCompNameAs';
			      "#,
            None,
            None,
        ),
        (
            "
			        function Test() {
			          const data = [
			            {
			              name: 'Bob',
			            },
			          ];

			          const columns = [
			            {
			              Header: 'Name',
			              accessor: 'name',
			              Cell: ({ value }) => <div>{value}</div>,
			            },
			          ];

			          return <ReactTable columns={columns} data={data} />;
			        }
			      ",
            None,
            None,
        ),
        (
            "
			        const f = (a) => () => {
			          if (a) {
			            return null;
			          }
			          return 1;
			        };
			      ",
            None,
            None,
        ),
        (
            "
			        class Test {
			          render() {
			            const data = [
			              {
			                name: 'Bob',
			              },
			            ];

			            const columns = [
			              {
			                Header: 'Name',
			                accessor: 'name',
			                Cell: ({ value }) => <div>{value}</div>,
			              },
			            ];

			            return <ReactTable columns={columns} data={data} />;
			          }
			        }
			      ",
            None,
            None,
        ),
        (
            "
			        export const demo = (a) => (b) => {
			          if (a == null) return null;
			          return b;
			        }
			      ",
            None,
            None,
        ),
        (
            "
			        let demo = null;
			        demo = (a) => {
			          if (a == null) return null;
			          return f(a);
			        };",
            None,
            None,
        ),
        (
            "
			        obj._property = (a) => {
			          if (a == null) return null;
			          return f(a);
			        };
			      ",
            None,
            None,
        ),
        (
            "
			        _variable = (a) => {
			          if (a == null) return null;
			          return f(a);
			        };
			      ",
            None,
            None,
        ),
        (
            "
			        demo = () => () => null;
			      ",
            None,
            None,
        ),
        (
            "
			        demo = {
			          property: () => () => null
			        }
			      ",
            None,
            None,
        ),
        (
            "
			        demo = function() {return function() {return null;};};
			      ",
            None,
            None,
        ),
        (
            "
			        demo = {
			          property: function() {return function() {return null;};}
			        }
			      ",
            None,
            None,
        ),
        (
            "
			        function MyComponent(props) {
			          return <b>{props.name}</b>;
			        }

			        const MemoizedMyComponent = React.memo(
			          MyComponent,
			          (prevProps, nextProps) => prevProps.name === nextProps.name
			        )
			      ",
            None,
            None,
        ),
        (
            "
			        import React from 'react'

			        const MemoizedForwardRefComponentLike = React.memo(
			          React.forwardRef(function({ world }, ref) {
			            return <div ref={ref}>Hello {world}</div>
			        })
			        )
			      ",
            None,
            Some(
                serde_json::json!({ "settings": {        "react": {          "version": "16.14.0",        },      } }),
            ),
        ),
        (
            "
			        import React from 'react'

			        const MemoizedForwardRefComponentLike = React.memo(
			          React.forwardRef(({ world }, ref) => {
			            return <div ref={ref}>Hello {world}</div>
			          })
			        )
			      ",
            None,
            Some(
                serde_json::json!({ "settings": {        "react": {          "version": "15.7.0",        },      } }),
            ),
        ),
        (
            "
			        import React from 'react'

			        const MemoizedForwardRefComponentLike = React.memo(
			          React.forwardRef(function ComponentLike({ world }, ref) {
			            return <div ref={ref}>Hello {world}</div>
			          })
			        )
			      ",
            None,
            Some(
                serde_json::json!({ "settings": {        "react": {          "version": "16.12.1",        },      } }),
            ),
        ),
        (
            "
			        export const ComponentWithForwardRef = React.memo(
			          React.forwardRef(function Component({ world }) {
			            return <div>Hello {world}</div>
			          })
			        )
			      ",
            None,
            Some(
                serde_json::json!({ "settings": {        "react": {          "version": "0.14.11",        },      } }),
            ),
        ),
        (
            "
			        import React from 'react'

			        const MemoizedForwardRefComponentLike = React.memo(
			          React.forwardRef(function({ world }, ref) {
			            return <div ref={ref}>Hello {world}</div>
			          })
			        )
			      ",
            None,
            Some(
                serde_json::json!({ "settings": {        "react": {          "version": "15.7.1",        },      } }),
            ),
        ),
        (
            r#"
			        import React from 'react';

			        const Hello = React.createContext();
			        Hello.displayName = "HelloContext"
			      "#,
            Some(serde_json::json!([{ "checkContextObjects": true }])),
            None,
        ),
        (
            r#"
			        import { createContext } from 'react';

			        const Hello = createContext();
			        Hello.displayName = "HelloContext"
			      "#,
            Some(serde_json::json!([{ "checkContextObjects": true }])),
            None,
        ),
        (
            r#"
			        import { createContext } from 'react';

			        const Hello = createContext();

			        const obj = {};
			        obj.displayName = "False positive";

			        Hello.displayName = "HelloContext"
			      "#,
            Some(serde_json::json!([{ "checkContextObjects": true }])),
            None,
        ),
        (
            r#"
			        import * as React from 'react';

			        const Hello = React.createContext();

			        const obj = {};
			        obj.displayName = "False positive";

			        Hello.displayName = "HelloContext";
			      "#,
            Some(serde_json::json!([{ "checkContextObjects": true }])),
            None,
        ),
        (
            r#"
			        const obj = {};
			        obj.displayName = "False positive";
			      "#,
            Some(serde_json::json!([{ "checkContextObjects": true }])),
            None,
        ),
        (
            "
			        import { createContext } from 'react';

			        const Hello = createContext();
			      ",
            Some(serde_json::json!([{ "checkContextObjects": true }])),
            Some(
                serde_json::json!({ "settings": {        "react": {          "version": "16.2.0",        },      } }),
            ),
        ),
        (
            r#"
			        import { createContext } from 'react';

			        const Hello = createContext();
			        Hello.displayName = "HelloContext";
			      "#,
            Some(serde_json::json!([{ "checkContextObjects": true }])),
            Some(
                serde_json::json!({ "settings": {        "react": {          "version": ">16.3.0",        },      } }),
            ),
        ),
        (
            r#"
			        import { createContext } from 'react';

			        let Hello;
			        Hello = createContext();
			        Hello.displayName = "HelloContext";
			      "#,
            Some(serde_json::json!([{ "checkContextObjects": true }])),
            None,
        ),
        (
            "
			        import { createContext } from 'react';

			        const Hello = createContext();
			      ",
            Some(serde_json::json!([{ "checkContextObjects": false }])),
            Some(
                serde_json::json!({ "settings": {        "react": {          "version": ">16.3.0",        },      } }),
            ),
        ),
        (
            r#"
			        import { createContext } from 'react';

			        var Hello;
			        Hello = createContext();
			        Hello.displayName = "HelloContext";
			      "#,
            Some(serde_json::json!([{ "checkContextObjects": true }])),
            None,
        ),
        (
            r#"
			        import { createContext } from 'react';

			        var Hello;
			        Hello = React.createContext();
			        Hello.displayName = "HelloContext";
			      "#,
            Some(serde_json::json!([{ "checkContextObjects": true }])),
            None,
        ),
    ];

    let fail = vec![
        (
            r#"
			        var Hello = createReactClass({
			          render: function() {
			            return React.createElement("div", {}, "text content");
			          }
			        });
			      "#,
            Some(serde_json::json!([{ "ignoreTranspilerName": true }])),
            None,
        ),
        (
            r#"
			        var Hello = React.createClass({
			          render: function() {
			            return React.createElement("div", {}, "text content");
			          }
			        });
			      "#,
            Some(serde_json::json!([{ "ignoreTranspilerName": true }])),
            Some(
                serde_json::json!({ "settings": {        "react": {          "createClass": "createClass",        },      } }),
            ),
        ),
        (
            "
			        var Hello = createReactClass({
			          render: function() {
			            return <div>Hello {this.props.name}</div>;
			          }
			        });
			      ",
            Some(serde_json::json!([{ "ignoreTranspilerName": true }])),
            None,
        ),
        (
            "
			        class Hello extends React.Component {
			          render() {
			            return <div>Hello {this.props.name}</div>;
			          }
			        }
			      ",
            Some(serde_json::json!([{ "ignoreTranspilerName": true }])),
            None,
        ),
        (
            "
			        function HelloComponent() {
			          return createReactClass({
			            render: function() {
			              return <div>Hello {this.props.name}</div>;
			            }
			          });
			        }
			        module.exports = HelloComponent();
			      ",
            Some(serde_json::json!([{ "ignoreTranspilerName": true }])),
            None,
        ),
        (
            "
			        module.exports = () => {
			          return <div>Hello {props.name}</div>;
			        }
			      ",
            None,
            None,
        ),
        (
            "
			        module.exports = function() {
			          return <div>Hello {props.name}</div>;
			        }
			      ",
            None,
            None,
        ),
        (
            "
			        module.exports = createReactClass({
			          render() {
			            return <div>Hello {this.props.name}</div>;
			          }
			        });
			      ",
            None,
            None,
        ),
        (
            "
			        var Hello = createReactClass({
			          _renderHello: function() {
			            return <span>Hello {this.props.name}</span>;
			          },
			          render: function() {
			            return <div>{this._renderHello()}</div>;
			          }
			        });
			      ",
            Some(serde_json::json!([{ "ignoreTranspilerName": true }])),
            None,
        ),
        (
            "
			        var Hello = Foo.createClass({
			          _renderHello: function() {
			            return <span>Hello {this.props.name}</span>;
			          },
			          render: function() {
			            return <div>{this._renderHello()}</div>;
			          }
			        });
			      ",
            Some(serde_json::json!([{ "ignoreTranspilerName": true }])),
            Some(
                serde_json::json!({ "settings": {        "react": {          "pragma": "Foo",          "createClass": "createClass",        },      } }),
            ),
        ),
        (
            "
			        /** @jsx Foo */
			        var Hello = Foo.createClass({
			          _renderHello: function() {
			            return <span>Hello {this.props.name}</span>;
			          },
			          render: function() {
			            return <div>{this._renderHello()}</div>;
			          }
			        });
			      ",
            Some(serde_json::json!([{ "ignoreTranspilerName": true }])),
            Some(
                serde_json::json!({ "settings": {        "react": {          "createClass": "createClass",        },      } }),
            ),
        ),
        (
            "
			        const Mixin = {
			          Button() {
			            return (
			              <button />
			            );
			          }
			        };
			      ",
            Some(serde_json::json!([{ "ignoreTranspilerName": true }])),
            None,
        ),
        (
            "
			        function Hof() {
			          return function () {
			            return <div />
			          }
			        }
			      ",
            None,
            None,
        ),
        (
            r#"
			        import React, { createElement } from "react";
			        export default (props) => {
			          return createElement("div", {}, "hello");
			        };
			      "#,
            None,
            None,
        ),
        (
            "
			        import React from 'react'

			        const ComponentWithMemo = React.memo(({ world }) => {
			          return <div>Hello {world}</div>
			        })
			      ",
            None,
            None,
        ),
        (
            "
			        import React from 'react'

			        const ComponentWithMemo = React.memo(function() {
			          return <div>Hello {world}</div>
			        })
			      ",
            None,
            None,
        ),
        (
            "
			        import React from 'react'

			        const ForwardRefComponentLike = React.forwardRef(({ world }, ref) => {
			          return <div ref={ref}>Hello {world}</div>
			        })
			      ",
            None,
            None,
        ),
        (
            "
			        import React from 'react'

			        const ForwardRefComponentLike = React.forwardRef(function({ world }, ref) {
			          return <div ref={ref}>Hello {world}</div>
			        })
			      ",
            None,
            None,
        ),
        (
            "
			        import React from 'react'

			        const MemoizedForwardRefComponentLike = React.memo(
			          React.forwardRef(({ world }, ref) => {
			            return <div ref={ref}>Hello {world}</div>
			          })
			        )
			      ",
            None,
            Some(
                serde_json::json!({ "settings": {        "react": {          "version": "15.6.0",        },      } }),
            ),
        ),
        (
            "
			        import React from 'react'

			        const MemoizedForwardRefComponentLike = React.memo(
			          React.forwardRef(function({ world }, ref) {
			            return <div ref={ref}>Hello {world}</div>
			          })
			        )
			      ",
            None,
            Some(
                serde_json::json!({ "settings": {        "react": {          "version": "0.14.2",        },      } }),
            ),
        ),
        (
            "
			        import React from 'react'

			        const MemoizedForwardRefComponentLike = React.memo(
			          React.forwardRef(function ComponentLike({ world }, ref) {
			            return <div ref={ref}>Hello {world}</div>
			          })
			        )
			      ",
            None,
            Some(
                serde_json::json!({ "settings": {        "react": {          "version": "15.0.1",        },      } }),
            ),
        ),
        (
            r#"
			        import React from "react";
			        const { createElement } = React;
			        export default (props) => {
			          return createElement("div", {}, "hello");
			        };
			      "#,
            None,
            None,
        ),
        (
            r#"
			        import React from "react";
			        const createElement = React.createElement;
			        export default (props) => {
			          return createElement("div", {}, "hello");
			        };
			      "#,
            None,
            None,
        ),
        (
            r#"
			        module.exports = function () {
			          function a () {}
			          const b = function b () {}
			          const c = function () {}
			          const d = () => {}
			          const obj = {
			            a: function a () {},
			            b: function b () {},
			            c () {},
			            d: () => {},
			          }
			          return React.createElement("div", {}, "text content");
			        }
			      "#,
            None,
            None,
        ),
        (
            r#"
			        module.exports = () => {
			          function a () {}
			          const b = function b () {}
			          const c = function () {}
			          const d = () => {}
			          const obj = {
			            a: function a () {},
			            b: function b () {},
			            c () {},
			            d: () => {},
			          }

			          return React.createElement("div", {}, "text content");
			        }
			      "#,
            None,
            None,
        ),
        (
            "
			        export default class extends React.Component {
			          render() {
			            function a () {}
			            const b = function b () {}
			            const c = function () {}
			            const d = () => {}
			            const obj = {
			              a: function a () {},
			              b: function b () {},
			              c () {},
			              d: () => {},
			            }
			            return <div>Hello {this.props.name}</div>;
			          }
			        }
			      ",
            None,
            None,
        ),
        (
            "
			        export default class extends React.PureComponent {
			          render() {
			            return <Card />;
			          }
			        }

			        const Card = (() => {
			          return React.memo(({ }) => (
			            <div />
			          ));
			        })();
			      ",
            None,
            None,
        ),
        (
            "
			        const renderer = a => listItem => (
			          <div>{a} {listItem}</div>
			        );
			      ",
            None,
            None,
        ),
        (
            "
			        const processData = (options?: { value: string }) => options?.value || 'no data';

			        export const Component = observer(() => {
			          const data = processData({ value: 'data' });
			          return <div>{data}</div>;
			        });

			        export const Component2 = observer(() => {
			          const data = processData();
			          return <div>{data}</div>;
			        });
			      ",
            None,
            Some(serde_json::json!({ "settings": { "componentWrapperFunctions": ["observer"] } })),
        ),
        (
            "
			        import React from 'react';

			        const Hello = React.createContext();
			      ",
            Some(serde_json::json!([{ "checkContextObjects": true }])),
            None,
        ),
        (
            "
			        import * as React from 'react';

			        const Hello = React.createContext();
			      ",
            Some(serde_json::json!([{ "checkContextObjects": true }])),
            None,
        ),
        (
            "
			        import { createContext } from 'react';

			        const Hello = createContext();
			      ",
            Some(serde_json::json!([{ "checkContextObjects": true }])),
            None,
        ),
        (
            "
			        import { createContext } from 'react';

			        var Hello;
			        Hello = createContext();
			      ",
            Some(serde_json::json!([{ "checkContextObjects": true }])),
            None,
        ),
        (
            "
			        import { createContext } from 'react';

			        var Hello;
			        Hello = React.createContext();
			      ",
            Some(serde_json::json!([{ "checkContextObjects": true }])),
            None,
        ),
    ];

    Tester::new(DisplayName::NAME, DisplayName::PLUGIN, pass, fail).test_and_snapshot();
}
