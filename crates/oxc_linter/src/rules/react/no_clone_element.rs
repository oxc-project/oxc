use oxc_ast::{AstKind, ast::Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{AstNode, context::LintContext, rule::Rule, utils::is_import_from_module};

fn no_clone_element_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("`React.cloneElement` should not be used.")
        .with_help("`React.cloneElement` is uncommon and leads to fragile React components.")
        .with_label(span)
        .with_note("https://react.dev/reference/react/cloneElement#alternatives")
}

#[derive(Debug, Default, Clone)]
pub struct NoCloneElement;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prevents the usage of `React.cloneElement`, which is considered an anti-pattern in React.
    ///
    /// ### Why is this bad?
    ///
    /// It is recommended not to use `React.cloneElement` because it can lead to code that is
    /// harder to follow and understand. It is generally uncommon and fragile, and there are various
    /// alternatives recommended by
    /// [the React documentation](https://react.dev/reference/react/cloneElement#alternatives).
    ///
    /// Note that this rule is based on [`@eslint-react/no-clone-element`](https://www.eslint-react.xyz/docs/rules/no-clone-element)
    /// from `@eslint-react`, not a rule from `eslint-plugin-react`.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// import { cloneElement } from "react";
    ///
    /// function List({ children }) {
    ///   const [selectedIndex, setSelectedIndex] = useState(0);
    ///   return (
    ///     <div className="List">
    ///       {Children.map(children, (child, index) =>
    ///         cloneElement(child, {
    ///           isHighlighted: index === selectedIndex
    ///         })
    ///       )}
    ///     </div>
    ///   );
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// // Using a map with a `renderItem` function prop instead.
    /// function List({ items, renderItem }) {
    ///   const [selectedIndex, setSelectedIndex] = useState(0);
    ///   return (
    ///     <div className="List">
    ///       {items.map((item, index) => {
    ///         const isHighlighted = index === selectedIndex;
    ///         return renderItem(item, isHighlighted);
    ///       })}
    ///     </div>
    ///   );
    /// }
    /// ```
    NoCloneElement,
    react,
    restriction,
    none,
    version = "1.53.0",
);

impl Rule for NoCloneElement {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        // import { cloneElement } from 'react';
        // cloneElement(...) / (cloneElement)(...)
        if let Expression::Identifier(ident) = call_expr.callee.get_inner_expression()
            && ident.name == "cloneElement"
            && is_import_from_module(ident, "react", ctx)
        {
            ctx.diagnostic(no_clone_element_diagnostic(ident.span));
            return;
        }

        // import React from 'react';
        // React.cloneElement(...) / React?.cloneElement(...) / React["cloneElement"](...)
        if let Some(member_expr) = call_expr.callee.get_inner_expression().get_member_expr()
            && let Some(name) = member_expr.static_property_name()
            && name == "cloneElement"
            && let Expression::Identifier(ident) = member_expr.object()
            && is_import_from_module(ident, "react", ctx)
        {
            ctx.diagnostic(no_clone_element_diagnostic(call_expr.callee.span()));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "/* import { cloneElement } from 'react'; */",
        "// import { cloneElement } from 'react';",
        "import { cloneElement } from 'react'; // cloneElement() in a comment.",
        "import { cloneElement } from 'react'; /* cloneElement() in a comment. */",
        "import { cloneElement } from 'something-else'; const clonedElement = cloneElement(<div />);",
        "import React from 'something-else'; const clonedElement = React.cloneElement(<div />);",
        "const cloneElement = () => {}; const clonedElement = cloneElement(<div />);",
    ];

    let fail = vec![
        r#"import { cloneElement } from "react";
           const clonedElement = cloneElement(
             <Row title="Cabbage">Hello</Row>,
             { isHighlighted: true },
             "Goodbye",
           );"#,
        r#"import { cloneElement } from "react";
           function Component() {
             const element = <div />;
             return cloneElement(element, { isHighlighted: true });
           }"#,
        "import React, { cloneElement } from 'react';
         const element = <div />;
         const clonedElement = cloneElement(element);",
        "import React from 'react';
         const element = <div />;
         const clonedElement = React.cloneElement(element);",
        "import React from 'react'; const clonedElement = React.cloneElement(<div />);",
        "import Aliased from 'react'; const clonedElement = Aliased.cloneElement(<div />);",
        "import { cloneElement } from 'react'; const clonedElement = (cloneElement)(<div />);",
        r#"import React from 'react'; const clonedElement = React["cloneElement"](<div />);"#,
        "import React from 'react'; const clonedElement = React?.cloneElement(<div />);",
    ];

    Tester::new(NoCloneElement::NAME, NoCloneElement::PLUGIN, pass, fail).test_and_snapshot();
}
