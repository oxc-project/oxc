use oxc_ast::{
    AstKind,
    ast::{CallExpression, Expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, fixer::RuleFixer, rule::Rule};

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
    suggestion
    suggestion
);

fn check_forward_ref_inner<'a>(
    exp: &Expression,
    call_expr: &CallExpression,
    ctx: &LintContext<'a>,
) {
    let (params, span) = match exp {
        Expression::ArrowFunctionExpression(f) => (&f.params, f.span),
        Expression::FunctionExpression(f) => (&f.params, f.span),
        _ => return,
    };
    if params.parameters_count() != 1 || params.rest.is_some() {
        return;
    }

    ctx.diagnostics_with_multiple_fixes(
        forward_ref_uses_ref_diagnostic(span),
        (FixKind::Suggestion, |fixer: RuleFixer<'_, 'a>| {
            fixer.replace_with(call_expr, exp).with_message("remove `forwardRef` wrapper")
        }),
        (FixKind::Suggestion, |fixer: RuleFixer<'_, 'a>| {
            let fixed = ctx.source_range(params.span);
            // remove the trailing `)`, `` and `,` if they exist
            let fixed = fixed.strip_suffix(')').unwrap_or(fixed).trim_end();
            let mut fixed = fixed.strip_suffix(',').unwrap_or(fixed).to_string();

            if !fixed.starts_with('(') {
                fixed.insert(0, '(');
            }
            fixed.push_str(", ref)");

            fixer.replace(params.span, fixed).with_message("add `ref` parameter")
        }),
    );
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
}

#[test]
fn test() {
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
        ("forwardRef((a) => {})", ("(a) => {}", "forwardRef((a, ref) => {})")),
        ("forwardRef(a => {})", ("a => {}", "forwardRef((a, ref) => {})")),
        ("forwardRef(function (a) {})", ("function (a) {}", "forwardRef(function (a, ref) {})")),
        ("forwardRef(function(a,) {})", ("function(a,) {}", "forwardRef(function(a, ref) {})")),
        ("forwardRef(function(a, ) {})", ("function(a, ) {}", "forwardRef(function(a, ref) {})")),
        (
            "React.forwardRef(function(a) {})",
            ("function(a) {}", "React.forwardRef(function(a, ref) {})"),
        ),
    ];

    Tester::new(ForwardRefUsesRef::NAME, ForwardRefUsesRef::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
