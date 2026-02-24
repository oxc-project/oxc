use oxc_ast::{
    AstKind,
    ast::{
        ArrowFunctionExpression, CallExpression, Class, ClassBody, ClassElement, Function,
        JSXElement, JSXFragment,
    },
};
use oxc_ast_visit::{Visit, walk};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_syntax::scope::ScopeFlags;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    AstNode,
    context::{ContextHost, LintContext},
    rule::{DefaultRuleConfig, Rule},
    utils::is_es6_component,
};

fn prefer_function_component_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Class component should be written as a function component.")
        .with_help("Convert the class component to a function component.")
        .with_label(span)
}

#[derive(Debug, Clone, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct PreferFunctionComponent {
    /// If `true`, error boundary classes (those implementing `componentDidCatch`
    /// or `static getDerivedStateFromError`) are allowed as class components.
    allow_error_boundary: bool,
    /// If `true`, classes that contain JSX but do not extend `Component` or
    /// `PureComponent` are allowed.
    allow_jsx_utility_class: bool,
}

impl Default for PreferFunctionComponent {
    fn default() -> Self {
        Self { allow_error_boundary: true, allow_jsx_utility_class: false }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces that React components are written as function components
    /// instead of class components.
    ///
    /// ### Why is this bad?
    ///
    /// Function components are simpler, easier to read, and support React
    /// hooks. Class components are a legacy pattern that is discouraged in
    /// modern React.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// class Foo extends React.Component {
    ///   render() {
    ///     return <div>{this.props.foo}</div>;
    ///   }
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// const Foo = function(props) {
    ///   return <div>{props.foo}</div>;
    /// };
    /// ```
    PreferFunctionComponent,
    react,
    restriction,
    config = PreferFunctionComponent,
);

impl Rule for PreferFunctionComponent {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::Class(class) = node.kind() else {
            return;
        };

        if !is_es6_component(node) {
            // Not extending Component/PureComponent. Only flag if it contains
            // JSX and allowJsxUtilityClass is false.
            if self.allow_jsx_utility_class {
                return;
            }
            if !class_body_contains_jsx(&class.body) {
                return;
            }
        }

        // Check error boundary exemption
        if self.allow_error_boundary && is_error_boundary(class) {
            return;
        }

        let span = class.id.as_ref().map_or(class.span, |id| id.span);
        ctx.diagnostic(prefer_function_component_diagnostic(span));
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.source_type().is_jsx()
    }
}

/// Returns `true` if the class has `componentDidCatch` method or
/// `static getDerivedStateFromError` method.
fn is_error_boundary(class: &Class) -> bool {
    class.body.body.iter().any(|element| {
        let key_name = match element {
            ClassElement::MethodDefinition(method) => {
                method.key.static_name().map(|n| (n, method.r#static))
            }
            ClassElement::PropertyDefinition(prop) => {
                prop.key.static_name().map(|n| (n, prop.r#static))
            }
            _ => None,
        };

        matches!(key_name, Some((ref name, false)) if name == "componentDidCatch")
            || matches!(key_name, Some((ref name, true)) if name == "getDerivedStateFromError")
    })
}

/// Visitor that searches for JSX elements within a class body.
/// Walks into direct class method bodies but stops at nested
/// functions/arrows within those methods.
struct ClassJsxFinder {
    found: bool,
    depth: u32,
}

impl ClassJsxFinder {
    fn new() -> Self {
        Self { found: false, depth: 0 }
    }
}

impl<'a> Visit<'a> for ClassJsxFinder {
    fn visit_jsx_element(&mut self, _elem: &JSXElement<'a>) {
        self.found = true;
    }

    fn visit_jsx_fragment(&mut self, _frag: &JSXFragment<'a>) {
        self.found = true;
    }

    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if crate::utils::is_create_element_call(call) {
            self.found = true;
        }
        if !self.found {
            walk::walk_call_expression(self, call);
        }
    }

    fn visit_function(&mut self, func: &Function<'a>, flags: ScopeFlags) {
        // depth 0 = class method body, walk into it
        // depth > 0 = nested function inside a method, skip
        if self.depth == 0 {
            self.depth += 1;
            walk::walk_function(self, func, flags);
            self.depth -= 1;
        }
    }

    fn visit_arrow_function_expression(&mut self, arrow: &ArrowFunctionExpression<'a>) {
        if self.depth == 0 {
            self.depth += 1;
            walk::walk_arrow_function_expression(self, arrow);
            self.depth -= 1;
        }
    }
}

/// Returns `true` if the class body contains JSX in any of its methods.
fn class_body_contains_jsx(body: &ClassBody) -> bool {
    let mut finder = ClassJsxFinder::new();
    finder.visit_class_body(body);
    finder.found
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // Already a stateless function
        (
            "const Foo = function(props) {
               return <div>{props.foo}</div>;
             };",
            None,
        ),
        // Already a stateless (arrow) function
        ("const Foo = ({foo}) => <div>{foo}</div>;", None),
        // class without JSX
        (
            "class Foo {
               render() {
                 return 'hello'
               }
             };",
            None,
        ),
        // object with JSX
        ("const foo = { foo: <h>hello</h> };", None),
        // Error boundary with componentDidCatch (allowed by default)
        (
            "class Foo extends React.Component {
               componentDidCatch(error, errorInfo) {
                 logErrorToMyService(error, errorInfo);
               }
               render() {
                 return <div>{this.props.foo}</div>;
               }
             }",
            None,
        ),
        // Error boundary with componentDidCatch extending PureComponent
        (
            "class Foo extends React.PureComponent {
               componentDidCatch(error, errorInfo) {
                 logErrorToMyService(error, errorInfo);
               }
               render() {
                 return <div>{this.props.foo}</div>;
               }
             }",
            None,
        ),
        // Error boundary in expression context
        (
            "const Foo = class extends React.Component {
               componentDidCatch(error, errorInfo) {
                 logErrorToMyService(error, errorInfo);
               }
               render() {
                 return <div>{this.props.foo}</div>;
               }
             };",
            None,
        ),
        // Error boundary with static getDerivedStateFromError
        (
            "class Foo extends React.Component {
               constructor(props) {
                 super(props);
                 this.state = { hasError: false };
               }
               static getDerivedStateFromError(error) {
                 return { hasError: true };
               }
               render() {
                 return <div>{this.state.hasError ? 'Error' : this.props.foo}</div>;
               }
             }",
            None,
        ),
        // getDerivedStateFromError extending PureComponent
        (
            "class Foo extends React.PureComponent {
               constructor(props) {
                 super(props);
                 this.state = { hasError: false };
               }
               static getDerivedStateFromError(error) {
                 return { hasError: true };
               }
               render() {
                 return <div>{this.state.hasError ? 'Error' : this.props.foo}</div>;
               }
             }",
            None,
        ),
        // getDerivedStateFromError in expression context
        (
            "const Foo = class extends React.Component {
               constructor(props) {
                 super(props);
                 this.state = { hasError: false };
               }
               static getDerivedStateFromError(error) {
                 return { hasError: true };
               }
               render() {
                 return <div>{this.state.hasError ? 'Error' : this.props.foo}</div>;
               }
             };",
            None,
        ),
        // Error boundary with allowJsxUtilityClass: true (still valid)
        (
            "class Foo extends React.Component {
               componentDidCatch(error, errorInfo) {
                 logErrorToMyService(error, errorInfo);
               }
               render() {
                 return <div>{this.props.foo}</div>;
               }
             }",
            Some(serde_json::json!([{ "allowJsxUtilityClass": true }])),
        ),
        // JSX utility class with allowJsxUtilityClass: true
        (
            "class Foo {
               getBar() {
                 return <Bar />;
               }
             };",
            Some(serde_json::json!([{ "allowJsxUtilityClass": true }])),
        ),
        // validForAllOptions with allowJsxUtilityClass: true
        (
            "const Foo = function(props) {
               return <div>{props.foo}</div>;
             };",
            Some(serde_json::json!([{ "allowJsxUtilityClass": true }])),
        ),
        (
            "const Foo = ({foo}) => <div>{foo}</div>;",
            Some(serde_json::json!([{ "allowJsxUtilityClass": true }])),
        ),
        (
            "class Foo {
               render() {
                 return 'hello'
               }
             };",
            Some(serde_json::json!([{ "allowJsxUtilityClass": true }])),
        ),
        (
            "const foo = { foo: <h>hello</h> };",
            Some(serde_json::json!([{ "allowJsxUtilityClass": true }])),
        ),
        // validForAllOptions with allowErrorBoundary: false
        (
            "const Foo = function(props) {
               return <div>{props.foo}</div>;
             };",
            Some(serde_json::json!([{ "allowErrorBoundary": false }])),
        ),
        (
            "const Foo = ({foo}) => <div>{foo}</div>;",
            Some(serde_json::json!([{ "allowErrorBoundary": false }])),
        ),
        (
            "class Foo {
               render() {
                 return 'hello'
               }
             };",
            Some(serde_json::json!([{ "allowErrorBoundary": false }])),
        ),
        (
            "const foo = { foo: <h>hello</h> };",
            Some(serde_json::json!([{ "allowErrorBoundary": false }])),
        ),
        // validForAllOptions with both options
        (
            "const Foo = function(props) {
               return <div>{props.foo}</div>;
             };",
            Some(
                serde_json::json!([{ "allowJsxUtilityClass": true, "allowErrorBoundary": false }]),
            ),
        ),
        (
            "const Foo = ({foo}) => <div>{foo}</div>;",
            Some(
                serde_json::json!([{ "allowJsxUtilityClass": true, "allowErrorBoundary": false }]),
            ),
        ),
        (
            "class Foo {
               render() {
                 return 'hello'
               }
             };",
            Some(
                serde_json::json!([{ "allowJsxUtilityClass": true, "allowErrorBoundary": false }]),
            ),
        ),
        (
            "const foo = { foo: <h>hello</h> };",
            Some(
                serde_json::json!([{ "allowJsxUtilityClass": true, "allowErrorBoundary": false }]),
            ),
        ),
    ];

    // Note: Many of these test cases are duped for the purpose of testing different config options.

    let fail = vec![
        // Extending from react
        (
            "import { Component } from 'react';

             class Foo extends Component {
               render() {
                 return <div>{this.props.foo}</div>;
               }
             }",
            None,
        ),
        // For now, we are not going to check for other libraries like preact and inferno.
        // This rule only supports React.
        //
        // Extending from preact
        // (
        //     "import { Component } from 'preact';
        //
        //      class Foo extends Component {
        //        render() {
        //          return <div>{this.props.foo}</div>;
        //        }
        //      };",
        //     None,
        // ),
        // Extending from inferno
        // (
        //     "import { Component } from 'inferno';
        //
        //      class Foo extends Component {
        //        render() {
        //          return <div>{this.props.foo}</div>;
        //        }
        //      };",
        //     None,
        // ),
        // Extending from another class (not Component)
        // (
        //     "import Document from 'next/document';
        //
        //      class Foo extends Document {
        //        render() {
        //          return <div>{this.props.foo}</div>;
        //        }
        //      };",
        //     None,
        // ),
        (
            "class Foo extends React.Component {
               render() {
                 return <div>{this.props.foo}</div>;
               }
             };",
            None,
        ),
        (
            "class Foo extends React.PureComponent {
               render() {
                 return <div>{this.props.foo}</div>;
               }
             };",
            None,
        ),
        (
            "const Foo = class extends React.Component {
               render() {
                 return <div>{this.props.foo}</div>;
               }
             };",
            None,
        ),
        (
            "export default class extends React.Component {
               render() {
                 return <div>{this.props.foo}</div>;
               }
             };",
            None,
        ),
        // Does not contain JSX and extends React.Component.
        (
            "class Foo extends React.Component {
               render() {
                 return null;
               }
             };",
            None,
        ),
        // Does not contain JSX and extends Component.
        (
            "import { Component } from 'react';

             class Foo extends Component {
               render() {
                 return null;
               }
             }",
            None,
        ),
        // Does not contain JSX and extends React.Component in an expression context.
        (
            "const Foo = class extends React.Component {
               render() {
                 return null;
               }
             };",
            None,
        ),
        // Does not contain JSX and extends Component in an expression context.
        (
            "import { Component } from 'react';

             const Foo = class extends Component {
               render() {
                 return null;
               }
             };",
            None,
        ),
        // Does not contain JSX and extends Component in an expression context.
        (
            "import { Component } from 'react';

             const Foo = class Bar extends Component {
               render() {
                 return null;
               }
             };",
            None,
        ),
        // Does not contain JSX and extends Component in a default export expression context.
        (
            "import { Component } from 'react';

             export default class extends Component {
               render() {
                 return null;
               }
             };",
            None,
        ),
        // Does not contain JSX and extends Component in a default export expression context.
        (
            "import { Component } from 'react';

             export default class Foo extends Component {
               render() {
                 return null;
               }
             };",
            None,
        ),
        // Error boundaries with allowErrorBoundary: false
        (
            "class Foo extends React.Component {
               componentDidCatch(error, errorInfo) {
                 logErrorToMyService(error, errorInfo);
               }
               render() {
                 return <div>{this.props.foo}</div>;
               }
             }",
            Some(serde_json::json!([{ "allowErrorBoundary": false }])),
        ),
        (
            "class Foo extends React.PureComponent {
               componentDidCatch(error, errorInfo) {
                 logErrorToMyService(error, errorInfo);
               }
               render() {
                 return <div>{this.props.foo}</div>;
               }
             }",
            Some(serde_json::json!([{ "allowErrorBoundary": false }])),
        ),
        (
            "const Foo = class extends React.Component {
               componentDidCatch(error, errorInfo) {
                 logErrorToMyService(error, errorInfo);
               }
               render() {
                 return <div>{this.props.foo}</div>;
               }
             };",
            Some(serde_json::json!([{ "allowErrorBoundary": false }])),
        ),
        (
            "class Foo extends React.Component {
               constructor(props) {
                 super(props);
                 this.state = { hasError: false };
               }
               static getDerivedStateFromError(error) {
                 return { hasError: true };
               }
               render() {
                 return <div>{this.state.hasError ? 'Error' : this.props.foo}</div>;
               }
             }",
            Some(serde_json::json!([{ "allowErrorBoundary": false }])),
        ),
        (
            "class Foo extends React.PureComponent {
               constructor(props) {
                 super(props);
                 this.state = { hasError: false };
               }
               static getDerivedStateFromError(error) {
                 return { hasError: true };
               }
               render() {
                 return <div>{this.state.hasError ? 'Error' : this.props.foo}</div>;
               }
             }",
            Some(serde_json::json!([{ "allowErrorBoundary": false }])),
        ),
        (
            "const Foo = class extends React.Component {
               constructor(props) {
                 super(props);
                 this.state = { hasError: false };
               }
               static getDerivedStateFromError(error) {
                 return { hasError: true };
               }
               render() {
                 return <div>{this.state.hasError ? 'Error' : this.props.foo}</div>;
               }
             };",
            Some(serde_json::json!([{ "allowErrorBoundary": false }])),
        ),
        // JSX utility class (fails by default)
        (
            "class Foo {
               getBar() {
                 return <Bar />;
               }
             };",
            None,
        ),
        // JSX utility class with allowErrorBoundary: false (still fails)
        (
            "class Foo {
               getBar() {
                 return <Bar />;
               }
             };",
            Some(serde_json::json!([{ "allowErrorBoundary": false }])),
        ),
        // invalidForAllOptions with allowJsxUtilityClass: true
        (
            "import { Component } from 'react';

             class Foo extends Component {
               render() {
                 return <div>{this.props.foo}</div>;
               }
             }",
            Some(serde_json::json!([{ "allowJsxUtilityClass": true }])),
        ),
        (
            "class Foo extends React.Component {
               render() {
                 return <div>{this.props.foo}</div>;
               }
             };",
            Some(serde_json::json!([{ "allowJsxUtilityClass": true }])),
        ),
        (
            "class Foo extends React.PureComponent {
               render() {
                 return <div>{this.props.foo}</div>;
               }
             };",
            Some(serde_json::json!([{ "allowJsxUtilityClass": true }])),
        ),
        (
            "const Foo = class extends React.Component {
               render() {
                 return <div>{this.props.foo}</div>;
               }
             };",
            Some(serde_json::json!([{ "allowJsxUtilityClass": true }])),
        ),
        (
            "export default class extends React.Component {
               render() {
                 return <div>{this.props.foo}</div>;
               }
             };",
            Some(serde_json::json!([{ "allowJsxUtilityClass": true }])),
        ),
        (
            "class Foo extends React.Component {
               render() {
                 return null;
               }
             };",
            Some(serde_json::json!([{ "allowJsxUtilityClass": true }])),
        ),
        (
            "import { Component } from 'react';

             class Foo extends Component {
               render() {
                 return null;
               }
             }",
            Some(serde_json::json!([{ "allowJsxUtilityClass": true }])),
        ),
        (
            "const Foo = class extends React.Component {
               render() {
                 return null;
               }
             };",
            Some(serde_json::json!([{ "allowJsxUtilityClass": true }])),
        ),
        (
            "import { Component } from 'react';

             const Foo = class extends Component {
               render() {
                 return null;
               }
             };",
            Some(serde_json::json!([{ "allowJsxUtilityClass": true }])),
        ),
        (
            "import { Component } from 'react';

             const Foo = class Bar extends Component {
               render() {
                 return null;
               }
             };",
            Some(serde_json::json!([{ "allowJsxUtilityClass": true }])),
        ),
        (
            "import { Component } from 'react';

             export default class extends Component {
               render() {
                 return null;
               }
             };",
            Some(serde_json::json!([{ "allowJsxUtilityClass": true }])),
        ),
        (
            "import { Component } from 'react';

             export default class Foo extends Component {
               render() {
                 return null;
               }
             };",
            Some(serde_json::json!([{ "allowJsxUtilityClass": true }])),
        ),
        // invalidForAllOptions with allowErrorBoundary: false
        (
            "import { Component } from 'react';

             class Foo extends Component {
               render() {
                 return <div>{this.props.foo}</div>;
               }
             }",
            Some(serde_json::json!([{ "allowErrorBoundary": false }])),
        ),
        (
            "class Foo extends React.Component {
               render() {
                 return <div>{this.props.foo}</div>;
               }
             };",
            Some(serde_json::json!([{ "allowErrorBoundary": false }])),
        ),
        (
            "class Foo extends React.PureComponent {
               render() {
                 return <div>{this.props.foo}</div>;
               }
             };",
            Some(serde_json::json!([{ "allowErrorBoundary": false }])),
        ),
        (
            "const Foo = class extends React.Component {
               render() {
                 return <div>{this.props.foo}</div>;
               }
             };",
            Some(serde_json::json!([{ "allowErrorBoundary": false }])),
        ),
        (
            "export default class extends React.Component {
               render() {
                 return <div>{this.props.foo}</div>;
               }
             };",
            Some(serde_json::json!([{ "allowErrorBoundary": false }])),
        ),
        (
            "class Foo extends React.Component {
               render() {
                 return null;
               }
             };",
            Some(serde_json::json!([{ "allowErrorBoundary": false }])),
        ),
        (
            "import { Component } from 'react';

             class Foo extends Component {
               render() {
                 return null;
               }
             }",
            Some(serde_json::json!([{ "allowErrorBoundary": false }])),
        ),
        (
            "const Foo = class extends React.Component {
               render() {
                 return null;
               }
             };",
            Some(serde_json::json!([{ "allowErrorBoundary": false }])),
        ),
        (
            "import { Component } from 'react';

             const Foo = class extends Component {
               render() {
                 return null;
               }
             };",
            Some(serde_json::json!([{ "allowErrorBoundary": false }])),
        ),
        (
            "import { Component } from 'react';

             const Foo = class Bar extends Component {
               render() {
                 return null;
               }
             };",
            Some(serde_json::json!([{ "allowErrorBoundary": false }])),
        ),
        (
            "import { Component } from 'react';

             export default class extends Component {
               render() {
                 return null;
               }
             };",
            Some(serde_json::json!([{ "allowErrorBoundary": false }])),
        ),
        (
            "import { Component } from 'react';

             export default class Foo extends Component {
               render() {
                 return null;
               }
             };",
            Some(serde_json::json!([{ "allowErrorBoundary": false }])),
        ),
        // invalidForAllOptions with both options
        (
            "import { Component } from 'react';

             class Foo extends Component {
               render() {
                 return <div>{this.props.foo}</div>;
               }
             }",
            Some(
                serde_json::json!([{ "allowJsxUtilityClass": true, "allowErrorBoundary": false }]),
            ),
        ),
        (
            "class Foo extends React.Component {
               render() {
                 return <div>{this.props.foo}</div>;
               }
             };",
            Some(
                serde_json::json!([{ "allowJsxUtilityClass": true, "allowErrorBoundary": false }]),
            ),
        ),
        (
            "class Foo extends React.PureComponent {
               render() {
                 return <div>{this.props.foo}</div>;
               }
             };",
            Some(
                serde_json::json!([{ "allowJsxUtilityClass": true, "allowErrorBoundary": false }]),
            ),
        ),
        (
            "const Foo = class extends React.Component {
               render() {
                 return <div>{this.props.foo}</div>;
               }
             };",
            Some(
                serde_json::json!([{ "allowJsxUtilityClass": true, "allowErrorBoundary": false }]),
            ),
        ),
        (
            "export default class extends React.Component {
               render() {
                 return <div>{this.props.foo}</div>;
               }
             };",
            Some(
                serde_json::json!([{ "allowJsxUtilityClass": true, "allowErrorBoundary": false }]),
            ),
        ),
        (
            "class Foo extends React.Component {
               render() {
                 return null;
               }
             };",
            Some(
                serde_json::json!([{ "allowJsxUtilityClass": true, "allowErrorBoundary": false }]),
            ),
        ),
        (
            "import { Component } from 'react';

             class Foo extends Component {
               render() {
                 return null;
               }
             }",
            Some(
                serde_json::json!([{ "allowJsxUtilityClass": true, "allowErrorBoundary": false }]),
            ),
        ),
        (
            "const Foo = class extends React.Component {
               render() {
                 return null;
               }
             };",
            Some(
                serde_json::json!([{ "allowJsxUtilityClass": true, "allowErrorBoundary": false }]),
            ),
        ),
        (
            "import { Component } from 'react';

             const Foo = class extends Component {
               render() {
                 return null;
               }
             };",
            Some(
                serde_json::json!([{ "allowJsxUtilityClass": true, "allowErrorBoundary": false }]),
            ),
        ),
        (
            "import { Component } from 'react';

             const Foo = class Bar extends Component {
               render() {
                 return null;
               }
             };",
            Some(
                serde_json::json!([{ "allowJsxUtilityClass": true, "allowErrorBoundary": false }]),
            ),
        ),
        (
            "import { Component } from 'react';

             export default class extends Component {
               render() {
                 return null;
               }
             };",
            Some(
                serde_json::json!([{ "allowJsxUtilityClass": true, "allowErrorBoundary": false }]),
            ),
        ),
        (
            "import { Component } from 'react';

             export default class Foo extends Component {
               render() {
                 return null;
               }
             };",
            Some(
                serde_json::json!([{ "allowJsxUtilityClass": true, "allowErrorBoundary": false }]),
            ),
        ),
    ];

    Tester::new(PreferFunctionComponent::NAME, PreferFunctionComponent::PLUGIN, pass, fail)
        .test_and_snapshot();
}
