use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_react_children_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("`React.Children` should not be used.")
        .with_help("`React.Children` is uncommon and leads to fragile React components.")
        .with_label(span)
        .with_note("https://react.dev/reference/react/Children#alternatives")
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
    /// It is recommended to use alternative approaches for handling children. See the
    /// [React documentation](https://react.dev/reference/react/Children#alternatives) for
    /// more information.
    ///
    /// ::: tip
    /// Don't confuse `React.Children` with using the `children` prop (lowercase `c`), which is
    /// good and encouraged.
    /// :::
    ///
    /// Note that this rule is based on a combination of multiple rules from `@eslint-react/eslint-plugin`,
    /// including [`@eslint-react/no-children-count`](https://www.eslint-react.xyz/docs/rules/no-children-count)
    /// and [`@eslint-react/no-children-for-each`](https://www.eslint-react.xyz/docs/rules/no-children-for-each).
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// import { Children } from 'react';
    ///
    /// Children.toArray(children);
    /// Children.map(children, child => <div>{child}</div>);
    /// Children.only(children);
    /// Children.count(children);
    /// Children.forEach(children, (child, index) => {});
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
    none,
    version = "1.53.0",
);

impl Rule for NoReactChildren {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        let Some(member_expr) = call_expr.callee.get_member_expr() else {
            return;
        };

        let object = member_expr.object().get_inner_expression();

        // Pattern 1: Children.method() where Children is imported from 'react'
        if let Some(ident) = object.get_identifier_reference()
            && ident.name == "Children"
            && is_imported_from_react(ident.name.as_str(), ctx)
        {
            ctx.diagnostic(no_react_children_diagnostic(member_expr.span()));
            return;
        }

        // Pattern 2: React.Children.method() where React is imported from 'react'
        if let Some(inner_member) = object.as_member_expression()
            && inner_member.static_property_name() == Some("Children")
            && let Some(ident) =
                inner_member.object().get_inner_expression().get_identifier_reference()
            && is_imported_from_react(ident.name.as_str(), ctx)
        {
            ctx.diagnostic(no_react_children_diagnostic(member_expr.span()));
        }
    }
}

fn is_imported_from_react(local_name: &str, ctx: &LintContext) -> bool {
    ctx.module_record().import_entries.iter().any(|entry| {
        entry.module_request.name() == "react" && entry.local_name.name() == local_name
    })
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
        "<div>Children</div>",
        "<MyComponent>Children</MyComponent>",
        "import React from 'react'; return <MyComponent>React.Children</MyComponent>",
        r#""React.Children""#,
        "import { Children } from 'something-else'; Children.toArray(children)",
        // Unresolved Children reference (no import) - not flagged
        "function Box({ children }) { const element = Children.only(children); }",
        // Local React object, not an import
        "const React = { Children: { map: () => {} } }; React.Children.map()",
        "<Foo>{children}</Foo>",
        // Usage of `children` in the props for a React component is fine.
        r#"import React from 'react';
           function Card({ children }) {
             return (
               <div className="card">
                 {children}
               </div>
             );
           }"#,
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
        "import * as React from 'react';
         function SeparatorList({ children }) {
           const result = [];
           React.Children.forEach(children, (child, index) => {
             result.push(child);
             result.push(<hr key={index} />);
           });
           // ...
         }",
        // Optional chaining
        "import React from 'react'; React.Children?.map(children, child => child)",
        // Parenthesized expressions
        "import { Children } from 'react'; (Children).map(children, child => child)",
        "import React from 'react'; (React.Children).map(children, child => child)",
        "import React from 'react'; (React.Children as any).map(children, child => child)",
    ];

    Tester::new(NoReactChildren::NAME, NoReactChildren::PLUGIN, pass, fail).test_and_snapshot();
}
