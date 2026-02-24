use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn prefer_function_component_diagnostic(span: Span) -> OxcDiagnostic {
    // See <https://oxc.rs/docs/contribute/linter/adding-rules.html#diagnostics> for details
    OxcDiagnostic::warn("Should be an imperative statement about what is wrong.")
        .with_help("Should be a command-like statement that tells the user how to fix the issue.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferFunctionComponent;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// FIXME: Briefly describe the rule's purpose.
    ///
    /// ### Why is this bad?
    ///
    /// FIXME: Explain why violating this rule is problematic.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// FIXME: Add at least one example of code that violates the rule.
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// FIXME: Add at least one example of code that is allowed with the rule.
    /// ```
    PreferFunctionComponent,
    react,
    restriction,
    none,
);

impl Rule for PreferFunctionComponent {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {}
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
    ];

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
        // Extending from preact
        (
            "import { Component } from 'preact';

             class Foo extends Component {
               render() {
                 return <div>{this.props.foo}</div>;
               }
             };",
            None,
        ),
        // Extending from inferno
        (
            "import { Component } from 'inferno';

             class Foo extends Component {
               render() {
                 return <div>{this.props.foo}</div>;
               }
             };",
            None,
        ),
        // Extending from another class (not Component)
        (
            "import Document from 'next/document';

             class Foo extends Document {
               render() {
                 return <div>{this.props.foo}</div>;
               }
             };",
            None,
        ),
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
        // Does not contain JSX and extends Component in an default export expression context.
        (
            "import { Component } from 'react';

             export default class extends Component {
               render() {
                 return null;
               }
             };",
            None,
        ),
        // Does not contain JSX and extends Component in an default export expression context.
        (
            "import { Component } from 'react';

             export default class Foo extends Component {
               render() {
                 return null;
               }
             };",
            None,
        ),
    ];

    Tester::new(PreferFunctionComponent::NAME, PreferFunctionComponent::PLUGIN, pass, fail)
        .test_and_snapshot();
}
