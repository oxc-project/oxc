use crate::{ContextHost, FrameworkFlags};
use oxc_ast::{AstKind, ast::Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn forward_ref_uses_ref_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Components wrapped with `forwardRef` must have a `ref` parameter")
        .with_help("Add a `ref` parameter, or remove `forwardRef`")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct ForwardRefUsesRef;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Requires that components wrapped with `forwardRef` must have a `ref` parameter.
    /// Omitting the `ref` argument is usually a bug,
    /// and components not using `ref` don't need to be wrapped by `forwardRef`.
    ///
    /// ### Why is this bad?
    ///
    /// Omitting the `ref` argument makes the `forwardRef` wrapper unnecessary,
    /// and can lead to confusion.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// var React = require('react');
    ///
    /// var Component = React.forwardRef((props) => (
    ///     <div />
    /// ));
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// var React = require('react');
    ///
    /// var Component = React.forwardRef((props, ref) => (
    ///    <div ref={ref} />
    /// ));
    ///
    /// var Component = React.forwardRef((props, ref) => (
    ///    <div />
    /// ));
    ///
    /// function Component(props) {
    ///    return <div />;
    /// };
    /// ```
    ForwardRefUsesRef,
    react,
    correctness,
    pending
    // TODO: two ways to fix it: add `ref` param or remove `forwardRef` call
);

fn check_forward_ref_inner(exp: &Expression, ctx: &LintContext) {
    match exp {
        Expression::ArrowFunctionExpression(f) => {
            if f.params.parameters_count() >= 2 || f.params.rest.is_some() {
                return;
            }
            ctx.diagnostic(forward_ref_uses_ref_diagnostic(f.span));
        }
        Expression::FunctionExpression(f) => {
            if f.params.parameters_count() >= 2 || f.params.rest.is_some() {
                return;
            }
            ctx.diagnostic(forward_ref_uses_ref_diagnostic(f.span));
        }
        // NOTE: Not sure whether to warn in `forwardRef(((props, ref) => null))` (with parentheses)
        // Expression::ParenthesizedExpression(p) => check_forward_ref_inner(&p.expression),
        _ => {}
    }
}

impl Rule for ForwardRefUsesRef {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        let Some("forwardRef") = call_expr.callee_name() else {
            return;
        };
        let Some(first_arg) = call_expr.arguments.first() else {
            return;
        };
        let Some(first_arg_as_exp) = first_arg.as_expression() else {
            return; // SpreadElement like forwardRef(...x)
        };

        check_forward_ref_inner(first_arg_as_exp, ctx);
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.frameworks().contains(FrameworkFlags::React)
    }
}

#[test]
fn test() {
    use crate::LintOptions;
    use crate::tester::Tester;

    let pass = vec![
        "
			        import { forwardRef } from 'react'
			        forwardRef((props, ref) => {
			          return null;
			        });
			      ",
        "
			        import { forwardRef } from 'react'
			        forwardRef((props, ref) => null);
			      ",
        "
			        import { forwardRef } from 'react'
			        forwardRef(function (props, ref) {
			          return null;
			        });
			      ",
        "
			        import { forwardRef } from 'react'
			        forwardRef(function Component(props, ref) {
			          return null;
			        });
			      ",
        "
			        import * as React from 'react'
			        React.forwardRef((props, ref) => {
			          return null;
			        });
			      ",
        "
			        import * as React from 'react'
			        React.forwardRef((props, ref) => null);
			      ",
        "
			        import * as React from 'react'
			        React.forwardRef(function (props, ref) {
			          return null;
			        });
			      ",
        "
			        import * as React from 'react'
			        React.forwardRef(function Component(props, ref) {
			          return null;
			        });
			      ",
        "
			        import * as React from 'react'
			        function Component(props) {
			          return null;
			        };
			      ",
        "
			        import * as React from 'react'
			        (props) => null;
			      ",
    ];

    let fail = vec![
        "
			        import { forwardRef } from 'react'
			        forwardRef((props) => {
			          return null;
			        });
			      ",
        "
			        import { forwardRef } from 'react'
			        forwardRef(props => {
			          return null;
			        });
			      ",
        "
			        import * as React from 'react'
			        React.forwardRef((props) => null);
			      ",
        "
			        import { forwardRef } from 'react'
			        const Component = forwardRef(function (props) {
			          return null;
			        });
			      ",
        "
			        import * as React from 'react'
			        React.forwardRef(function Component(props) {
			          return null;
			        });
			      ",
    ];

    Tester::new(ForwardRefUsesRef::NAME, ForwardRefUsesRef::PLUGIN, pass, fail)
        .with_lint_options(LintOptions {
            framework_hints: FrameworkFlags::React,
            ..LintOptions::default()
        })
        .test_and_snapshot();
}
