use oxc_ast::{
    AstKind,
    ast::{CallExpression, Expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    AstNode,
    ast_util::{could_be_asi_hazard, outermost_paren_parent},
    context::LintContext,
    fixer::RuleFixer,
    rule::Rule,
};

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
    ///   <div />
    /// ));
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// var React = require('react');
    ///
    /// var Component = React.forwardRef((props, ref) => (
    ///   <div ref={ref} />
    /// ));
    ///
    /// var Component = React.forwardRef((props, ref) => (
    ///   <div />
    /// ));
    ///
    /// function Component(props) {
    ///   return <div />;
    /// };
    /// ```
    ForwardRefUsesRef,
    react,
    correctness,
    suggestion,
    version = "0.16.9",
    short_description = "Require all `forwardRef` components include a `ref` parameter.",
);

impl Rule for ForwardRefUsesRef {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let function_span = match node.kind() {
            AstKind::Function(function) => function.span,
            AstKind::ArrowFunctionExpression(arrow) => arrow.span,
            _ => return,
        };
        let node = ctx.nodes().parent_node(node.id());
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
        if first_arg_as_exp.span() != function_span {
            return;
        }

        check_forward_ref_inner(first_arg_as_exp, call_expr, node, ctx);
    }
}

fn check_forward_ref_inner<'a>(
    exp: &Expression,
    call_expr: &CallExpression,
    node: &AstNode<'a>,
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

    let can_remove_forward_ref = match exp {
        Expression::FunctionExpression(f) if f.id.is_none() => !matches!(
            outermost_paren_parent(node, ctx.semantic()).map(AstNode::kind),
            Some(AstKind::ExpressionStatement(_))
        ),
        _ => true,
    };

    let add_ref_parameter_fix = |fixer: RuleFixer<'_, 'a>| {
        let param = &params.items[0];
        let fix = if ctx.source_range(params.span).starts_with('(') {
            fixer.insert_text_after(param, ", ref")
        } else {
            fixer.replace(param.span, format!("({}, ref)", ctx.source_range(param.span)))
        };
        fix.with_message("add `ref` parameter")
    };

    if can_remove_forward_ref {
        ctx.diagnostics_with_multiple_fixes(
            forward_ref_uses_ref_diagnostic(span),
            (FixKind::Suggestion, |fixer: RuleFixer<'_, 'a>| {
                let text = ctx.source_range(exp.span());
                let semi =
                    if text.starts_with('(') && could_be_asi_hazard(node, ctx) { ";" } else { "" };
                fixer
                    .replace(call_expr.span, format!("{semi}{text}"))
                    .with_message("remove `forwardRef` wrapper")
            }),
            (FixKind::Suggestion, add_ref_parameter_fix),
        );
    } else {
        ctx.diagnostic_with_suggestion(
            forward_ref_uses_ref_diagnostic(span),
            add_ref_parameter_fix,
        );
    }
}

#[test]
fn test() {
    use crate::tester::{ExpectFixTestCase, Tester};

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
        "forwardRef(null, props => null)",
        "forwardRef((props, ref) => null, props => null)",
        "forwardRef((props => null))",
        "forwardRef(...[props => null])",
        "forwardRef((props => null) as Component)",
    ];

    let fail = vec![
        "forwardRef(props => null, unused => null)",
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

    let fix: Vec<ExpectFixTestCase> = vec![
        // Arrow functions - two fixes (remove wrapper or add ref)
        ("forwardRef((a) => {})", ("(a) => {}", "forwardRef((a, ref) => {})")).into(),
        ("forwardRef(a => {})", ("a => {}", "forwardRef((a, ref) => {})")).into(),
        // Named function expressions - two fixes
        (
            "forwardRef(function Component(a) {})",
            ("function Component(a) {}", "forwardRef(function Component(a, ref) {})"),
        )
            .into(),
        // Anonymous functions in expression context - two fixes
        (
            "const x = forwardRef(function(a) {})",
            ("const x = function(a) {}", "const x = forwardRef(function(a, ref) {})"),
        )
            .into(),
        // Anonymous functions in statement context - one fix (add ref only)
        ("forwardRef(function (a) {})", "forwardRef(function (a, ref) {})").into(),
        ("forwardRef(function(a,) {})", "forwardRef(function(a, ref,) {})").into(),
        ("forwardRef(function(a, ) {})", "forwardRef(function(a, ref, ) {})").into(),
        ("React.forwardRef(function(a) {})", "React.forwardRef(function(a, ref) {})").into(),
        ("forwardRef(function(a,/* c */) {})", "forwardRef(function(a, ref,/* c */) {})").into(),
        ("forwardRef(function(a // c\n) {})", "forwardRef(function(a, ref // c\n) {})").into(),
        ("forwardRef(function(a, // c\n) {})", "forwardRef(function(a, ref, // c\n) {})").into(),
        ("foo\nforwardRef((a) => {})", ("foo\n;(a) => {}", "foo\nforwardRef((a, ref) => {})"))
            .into(),
        ("foo\nforwardRef(a => {})", ("foo\na => {}", "foo\nforwardRef((a, ref) => {})")).into(),
    ];

    Tester::new(ForwardRefUsesRef::NAME, ForwardRefUsesRef::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
