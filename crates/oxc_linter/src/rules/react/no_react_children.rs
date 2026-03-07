use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_react_children_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("`React.Children` should not be used.")
        .with_help("Children passed to this React component should be handled in a different way, see the docs for alternatives.")
        .with_label(span)
        .with_note("https://react.dev/reference/react/Children")
}

#[derive(Debug, Default, Clone)]
pub struct NoReactChildren;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows the usage of `React.Children`, as it is considered a bad practice.
    ///
    /// ### Why is this bad?
    ///
    /// Using `React.Children` is
    /// [discouraged by the React documentation](https://react.dev/reference/react/Children).
    ///
    /// It is recommended to use alternative approaches for handling children.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// import { Children } from 'react';
    ///
    /// Children.toArray(children)
    /// Children.map(children, child => <div>{child}</div>)
    /// Children.only(children)
    /// Children.count(children)
    /// Children.forEach(children, (child, index) => {})
    /// ```
    ///
    /// ```jsx
    /// import React from 'react';
    ///
    /// function Table({ children }) {
    ///   const mappedChildren = React.Children.map(children, (child) =>
    ///     <tr>{child}</tr>
    ///   );
    ///
    ///   return <table>{mappedChildren}</table>;
    /// }
    /// ```
    ///
    /// ```jsx
    /// import { Children } from 'react';
    ///
    /// function RowList({ children }) {
    ///   return (
    ///     <>
    ///       <h1>Total rows: {Children.count(children)}</h1>
    ///     </>
    ///   );
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// function Card({ children }) {
    ///   return (
    ///     <div className="card">
    ///       {children}
    ///     </div>
    ///   );
    /// }
    /// ```
    NoReactChildren,
    react,
    restriction,
    none
);

impl Rule for NoReactChildren {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {}
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![];

    let fail = vec![];

    Tester::new(NoReactChildren::NAME, NoReactChildren::PLUGIN, pass, fail).test_and_snapshot();
}
