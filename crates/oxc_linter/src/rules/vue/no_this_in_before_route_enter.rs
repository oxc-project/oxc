use oxc_ast::{AstKind, ast::{Argument, Expression, Function, ThisExpression}};
use oxc_ast_visit::Visit;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::ScopeFlags;
use oxc_span::Span;

use crate::{
    AstNode,
    context::LintContext,
    rule::Rule,
};

fn no_this_in_before_route_enter_diagnostic(span: Span) -> OxcDiagnostic {
    // See <https://oxc.rs/docs/contribute/linter/adding-rules.html#diagnostics> for details
    OxcDiagnostic::warn("Should be an imperative statement about what is wrong")
        .with_help("Should be a command-like statement that tells the user how to fix the issue")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoThisInBeforeRouteEnter;

// See <https://github.com/oxc-project/oxc/issues/6050> for documentation details.
declare_oxc_lint!(
    /// ### What it does
    ///
    /// Briefly describe the rule's purpose.
    ///
    /// ### Why is this bad?
    ///
    /// Explain why violating this rule is problematic.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// FIXME: Tests will fail if examples are missing or syntactically incorrect.
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// FIXME: Tests will fail if examples are missing or syntactically incorrect.
    /// ```
    NoThisInBeforeRouteEnter,
    vue,
    correctness,
);

impl Rule for NoThisInBeforeRouteEnter {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        let Some(ident) = call_expr.callee.get_identifier_reference() else {
            return;
        };

        if ident.name != "beforeRouteEnter" {
            return;
        }


    }

    fn should_run(&self, ctx: &crate::context::ContextHost) -> bool {
        ctx.file_path().extension().is_some_and(|ext| ext == "vue")
    }
}

struct ThisFinder {
    found: bool,
}

impl<'a> Visit<'a> for ThisFinder {
    fn visit_this_expression(&mut self, _expr: &ThisExpression) {
        self.found = true;
    }

    fn visit_function(&mut self, _func: &Function<'a>, _flags: ScopeFlags) {}
}

#[test]
fn test() {
    use crate::tester::Tester;
    use std::path::PathBuf;

    let pass = vec![(
        r#"
			<template>
			  <p>{{ greeting }} World!</p>
			</template>
			
			<script>
			export default {
			  data () {
			    return {
			      greeting: "Hello"
			    };
			  },
			};"#,
        None,
        None,
        Some(PathBuf::from("test.vue")),
    )];

    let fail = vec![];

    Tester::new(NoThisInBeforeRouteEnter::NAME, NoThisInBeforeRouteEnter::PLUGIN, pass, fail)
        .test_and_snapshot();
}
