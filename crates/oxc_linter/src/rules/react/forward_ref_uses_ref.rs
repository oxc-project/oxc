use crate::{ContextHost, FrameworkFlags};
use oxc_ast::{
    AstKind,
    ast::{CallExpression, Expression},
};
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
    fix
);

fn check_forward_ref_inner(exp: &Expression, call_expr: &CallExpression, ctx: &LintContext) {
    let (params, span) = match exp {
        Expression::ArrowFunctionExpression(f) => (&f.params, f.span),
        Expression::FunctionExpression(f) => (&f.params, f.span),
        _ => return,
    };
    if params.parameters_count() != 1 || params.rest.is_some() {
        return;
    }
    ctx.diagnostic_with_fix(forward_ref_uses_ref_diagnostic(span), |fixer| {
        fixer.replace_with(call_expr, exp)
    });
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

        check_forward_ref_inner(first_arg_as_exp, call_expr, ctx);
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
        "forwardRef(() => {})",
        "forwardRef(function () {})",
        "forwardRef(function (a, b, c) {})",
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

    let fix = vec![
        ("forwardRef((a) => {})", "(a) => {}"),
        ("forwardRef(a => {})", "a => {}"),
        ("forwardRef(function (a) {})", "function (a) {}"),
        ("forwardRef(function(a,) {})", "function(a,) {}"),
        ("React.forwardRef(function(a) {})", "function(a) {}"),
    ];

    Tester::new(ForwardRefUsesRef::NAME, ForwardRefUsesRef::PLUGIN, pass, fail)
        .with_lint_options(LintOptions {
            framework_hints: FrameworkFlags::React,
            ..LintOptions::default()
        })
        .expect_fix(fix)
        .test_and_snapshot();
}
