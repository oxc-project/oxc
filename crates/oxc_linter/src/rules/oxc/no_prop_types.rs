use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_prop_types_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("PropTypes is deprecated")
        .with_help("Use TypeScript for type checking instead of PropTypes. PropTypes is deprecated since React 15.5.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoPropTypes;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Detects usage of `Component.propTypes` or imports of `prop-types`.
    ///
    /// ### Why is this bad?
    ///
    /// PropTypes has been deprecated since React 15.5 and provides only
    /// runtime type checking. Use TypeScript for static type checking instead.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// import PropTypes from "prop-types";
    /// Component.propTypes = { name: PropTypes.string };
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```tsx
    /// interface ComponentProps { name: string; }
    /// function Component({ name }: ComponentProps) { return <div />; }
    /// ```
    NoPropTypes,
    oxc,
    restriction,
    none
);

impl Rule for NoPropTypes {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::AssignmentExpression(assign) => {
                let oxc_ast::ast::AssignmentTarget::StaticMemberExpression(member) = &assign.left
                else {
                    return;
                };
                if member.property.name == "propTypes" {
                    ctx.diagnostic(no_prop_types_diagnostic(member.property.span));
                }
            }
            AstKind::ImportDeclaration(import) => {
                if import.source.value.as_str() == "prop-types" {
                    ctx.diagnostic(no_prop_types_diagnostic(import.source.span));
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
        "Component.displayName = 'Component';",
        "Component.defaultProps = {};",
        "import React from 'react';",
    ];

    let fail = vec![
        "Component.propTypes = { name: PropTypes.string };",
        "import PropTypes from 'prop-types';",
    ];

    Tester::new(NoPropTypes::NAME, NoPropTypes::PLUGIN, pass, fail)
        .change_rule_path_extension("tsx")
        .test_and_snapshot();
}
