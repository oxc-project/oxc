use oxc_macros::declare_oxc_lint;

use crate::{context::ContextHost, rule::Rule};

#[derive(Debug, Default, Clone)]
pub struct JsxUsesVars;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule is a companion to the ESLint `no-unused-vars` rule. It marks
    /// variables used in JSX as used, preventing `no-unused-vars` from
    /// incorrectly flagging them as unused.
    ///
    /// ### Why is this bad?
    ///
    /// In Oxc, the semantic analyzer already tracks JSX element names as
    /// references during the initial AST analysis phase. This means variables
    /// used as JSX component names are automatically considered "used" by
    /// `no-unused-vars`.
    ///
    /// This rule exists solely for ESLint compatibility and performs no
    /// operations in Oxc.
    ///
    /// ### Examples
    ///
    /// Variables used as JSX component names are automatically tracked:
    /// ```jsx
    /// import React from 'react';
    /// import App from './App';
    ///
    /// <App />;
    /// ```
    ///
    /// Member expressions are also tracked:
    /// ```jsx
    /// import React from 'react';
    ///
    /// <React.Fragment />;
    /// ```
    JsxUsesVars,
    react,
    nursery
);

impl Rule for JsxUsesVars {
    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.source_type().is_jsx()
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // Basic component usage
        "var App; <App />",
        "var App; function f() { return <App />; }",
        // Member expressions
        "var App; <App.Item />",
        "var App; <App.Foo.Bar />",
        "var App; <App.Foo.Bar.Baz />",
        // Nested components
        "var App, Bar; <App><Bar /></App>",
        // Components in props
        "var App, Icon; <App icon={Icon} />",
        "var App, a; <App>{a}</App>",
        // Function components
        "var App; function Component() { return <App />; }",
        "var App; const Component = () => <App />;",
        // Class components
        "var App; class Component { render() { return <App />; } }",
        // Multiple components
        "var App, Header, Footer; <div><Header /><App /><Footer /></div>",
        // Import statements
        "import App from './App'; <App />",
        "import { Button } from './components'; <Button />",
        "import * as Components from './components'; <Components.Button />",
        // Destructuring
        "const { App } = require('./components'); <App />",
        // Variable reassignment
        "var App; let Component = App; <Component />",
        // Conditional rendering
        "var App, Loading; const Component = () => isLoading ? <Loading /> : <App />;",
        // Array of components
        "var App; [<App key='1' />, <App key='2' />]",
        // Fragment shorthand
        "var App; <><App /></>",
        // React.Fragment
        "var React, App; <React.Fragment><App /></React.Fragment>",
        // Spread attributes
        "var App, props; <App {...props} />",
        // Ternary in JSX
        "var App, Other; <div>{condition ? <App /> : <Other />}</div>",
        // Logical AND
        "var App; <div>{condition && <App />}</div>",
        // Components in loops
        "var Item; items.map(item => <Item key={item.id} />)",
        // Higher-order components
        "var App; const Enhanced = withHOC(App);",
        // Namespace (should be ignored but valid JSX)
        "<foo:bar />",
        // HTML elements (lowercase) should not count as component usage
        "<div />",
        "<span />",
        "<custom-element />",
        // this.props usage
        "class Component { render() { return <this.props.tag />; } }",
    ];

    let fail = vec![];

    Tester::new(JsxUsesVars::NAME, JsxUsesVars::PLUGIN, pass, fail).test_and_snapshot();
}
