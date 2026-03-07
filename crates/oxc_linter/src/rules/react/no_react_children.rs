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
    /// It is an uncommon pattern and can lead to fragile code.
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

    let pass = vec![
        "import React from 'react';",
        "const children = []; children.map(x => x)",
        "const Children = { map: () => {} }; Children.map()",
        "import React from 'react'; React.createElement('div')",
        // Violations in comments do not count.
        "const foo = []; /* React.Children */",
        "const foo = []; /* import { Children } from 'react'; */",
    ];

    let fail = vec![
        // Named import { Children } + each method
        "import { Children } from 'react'; Children.toArray(children)",
        "import { Children } from 'react'; Children.map(children, child => <div>{child}</div>)",
        "import { Children } from 'react'; Children.only(children)",
        "import { Children } from 'react'; Children.count(children)",
        "import { Children } from 'react'; Children.forEach(children, (child, index) => {})",
        // Default import React + React.Children.*
        "import React from 'react'; React.Children.toArray(children)",
        "import React from 'react'; React.Children.map(children, child => <div>{child}</div>)",
        "import React from 'react'; React.Children.only(children)",
        "import React from 'react'; React.Children.count(children)",
        "import React from 'react'; React.Children.forEach(children, (child, index) => {})",
        // Wildcard import * as React
        "import * as React from 'react'; React.Children.map(children, child => <div>{child}</div>)",
        // Combined import React, { Children }
        "import React, { Children } from 'react'; Children.toArray(children)",
        // Various complex examples
        "import { Children } from 'react';
         function RowList({ children }) {
           return (
             <><h1>Total rows: {Children.count(children)}</h1></>
           );
         }",
        "import React from 'react';
         export const Table = ({ children }) => {
           const mappedChildren = React.Children.map(children, (child) =>
             <tr>{child}</tr>
           );
           return <table>{mappedChildren}</table>;
         }",
        "import { Children } from 'react';
         function SeparatorList({ children }) {
           const result = [];
           Children.forEach(children, (child, index) => {
             result.push(child);
             result.push(<hr key={index} />);
           });
         }",
        r#"import { Children } from 'react';
           function RowList({ children }) {
             return (
               <div className="RowList">
                 {Children.map(children, child =>
                   <div className="Row">
                     {child}
                   </div>
                 )}
               </div>
             );
           }"#,
        "function Box({ children }) { const element = Children.only(children); }",
        "import * as React from 'react';
         function SeparatorList({ children }) {
           const result = [];
           React.Children.forEach(children, (child, index) => {
             result.push(child);
             result.push(<hr key={index} />);
           });
           // ...
         }",
    ];

    Tester::new(NoReactChildren::NAME, NoReactChildren::PLUGIN, pass, fail).test_and_snapshot();
}
