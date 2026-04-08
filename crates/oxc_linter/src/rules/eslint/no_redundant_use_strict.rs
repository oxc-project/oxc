use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_redundant_use_strict_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Redundant 'use strict' directive.")
        .with_help("Remove this 'use strict' directive because the code is already in strict mode (ESM modules are always strict, or an outer scope already has 'use strict').")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoRedundantUseStrict;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows redundant `"use strict"` directives.
    ///
    /// ### Why is this bad?
    ///
    /// A `"use strict"` directive is unnecessary when the code is already in
    /// strict mode. ES modules are always in strict mode, and nested `"use strict"`
    /// directives inside an already-strict function body are also redundant.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// // In an ES module file:
    /// "use strict";
    /// export const foo = 1;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// // In a CommonJS file:
    /// "use strict";
    /// module.exports = {};
    /// ```
    NoRedundantUseStrict,
    eslint,
    suspicious,
    pending
);

impl Rule for NoRedundantUseStrict {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        // Check directives in Program
        if let AstKind::Program(program) = node.kind() {
            if !ctx.source_type().is_module() {
                return;
            }
            // In ESM, any "use strict" directive is redundant
            for directive in &program.directives {
                if directive.directive.as_str() == "use strict" {
                    ctx.diagnostic(no_redundant_use_strict_diagnostic(directive.span));
                }
            }
            return;
        }

        // Check directives in function bodies
        if let AstKind::FunctionBody(body) = node.kind() {
            for directive in &body.directives {
                if directive.directive.as_str() == "use strict" {
                    // Check if parent scopes already have "use strict"
                    if ctx.source_type().is_module() || is_already_strict(node, ctx) {
                        ctx.diagnostic(no_redundant_use_strict_diagnostic(directive.span));
                    }
                }
            }
        }
    }
}

fn is_already_strict(node: &AstNode, ctx: &LintContext) -> bool {
    for ancestor in ctx.nodes().ancestors(node.id()) {
        match ancestor.kind() {
            AstKind::FunctionBody(body) => {
                for directive in &body.directives {
                    if directive.directive.as_str() == "use strict" {
                        return true;
                    }
                }
            }
            AstKind::Program(program) => {
                for directive in &program.directives {
                    if directive.directive.as_str() == "use strict" {
                        return true;
                    }
                }
            }
            _ => {}
        }
    }
    false
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // CJS with use strict is fine
        "\"use strict\"; var foo = 1;",
        // Function body use strict is fine when not in module
        "function foo() { \"use strict\"; return 1; }",
    ];

    let fail = vec![
        // Nested use strict in function that already has outer strict
        "\"use strict\"; function foo() { \"use strict\"; return 1; }",
    ];

    Tester::new(NoRedundantUseStrict::NAME, NoRedundantUseStrict::PLUGIN, pass, fail)
        .test_and_snapshot();
}
