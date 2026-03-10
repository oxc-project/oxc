use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule, utils::is_angular_core_import};

fn computed_must_return_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Computed signal callback must return a value")
        .with_help(
            "The function passed to `computed()` must return a value. \
            Computed signals derive their value from the return value of this function. \
            Add a return statement or use an arrow function with an expression body.",
        )
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct ComputedMustReturn;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Ensures that the callback function passed to `computed()` returns a value.
    ///
    /// ### Why is this bad?
    ///
    /// The `computed()` function creates a signal that derives its value from other signals.
    /// The callback function must return a value - this is the computed signal's value.
    /// If the callback doesn't return anything, the computed signal will always be `undefined`,
    /// which is almost certainly a bug.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```typescript
    /// import { Component, computed, signal } from '@angular/core';
    ///
    /// @Component({ selector: 'app-example', template: '' })
    /// export class ExampleComponent {
    ///   count = signal(0);
    ///   double = computed(() => {
    ///     this.count() * 2; // Missing return!
    ///   });
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```typescript
    /// import { Component, computed, signal } from '@angular/core';
    ///
    /// @Component({ selector: 'app-example', template: '' })
    /// export class ExampleComponent {
    ///   count = signal(0);
    ///   // Arrow function with expression body (implicit return)
    ///   double = computed(() => this.count() * 2);
    ///
    ///   // Or with explicit return
    ///   triple = computed(() => {
    ///     return this.count() * 3;
    ///   });
    /// }
    /// ```
    ComputedMustReturn,
    angular,
    correctness,
    pending
);

impl Rule for ComputedMustReturn {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        // Check if this is a call to computed
        let callee_name = match &call_expr.callee {
            oxc_ast::ast::Expression::Identifier(ident) => ident.name.as_str(),
            _ => return,
        };

        if callee_name != "computed" {
            return;
        }

        // Verify it's imported from @angular/core
        let Some(ident) = call_expr.callee.get_identifier_reference() else {
            return;
        };

        if !is_angular_core_import(ident, ctx) {
            return;
        }

        // Get the first argument (the callback function)
        let Some(first_arg) = call_expr.arguments.first() else {
            return;
        };

        // Check if the callback returns a value
        match first_arg {
            oxc_ast::ast::Argument::ArrowFunctionExpression(arrow) => {
                // Arrow functions with expression body always return
                if arrow.expression {
                    return;
                }

                // Check if the function body has a return statement
                if !has_return_statement(&arrow.body) {
                    ctx.diagnostic(computed_must_return_diagnostic(arrow.span));
                }
            }
            oxc_ast::ast::Argument::FunctionExpression(func) => {
                // Check if the function body has a return statement
                if let Some(body) = &func.body
                    && !has_return_statement_in_body(body) {
                        ctx.diagnostic(computed_must_return_diagnostic(func.span));
                    }
            }
            _ => {}
        }
    }
}

fn has_return_statement(body: &oxc_ast::ast::FunctionBody<'_>) -> bool {
    has_return_statement_in_body(body)
}

fn has_return_statement_in_body(body: &oxc_ast::ast::FunctionBody<'_>) -> bool {
    for stmt in &body.statements {
        if has_return_in_statement(stmt) {
            return true;
        }
    }
    false
}

fn has_return_in_statement(stmt: &oxc_ast::ast::Statement<'_>) -> bool {
    match stmt {
        oxc_ast::ast::Statement::ReturnStatement(ret) => {
            // Must return something, not just `return;`
            ret.argument.is_some()
        }
        oxc_ast::ast::Statement::BlockStatement(block) => {
            block.body.iter().any(has_return_in_statement)
        }
        oxc_ast::ast::Statement::IfStatement(if_stmt) => {
            // Both branches should ideally return, but we check if any returns
            let consequent_returns = has_return_in_statement(&if_stmt.consequent);
            let alternate_returns =
                if_stmt.alternate.as_ref().is_some_and(|alt| has_return_in_statement(alt));
            consequent_returns || alternate_returns
        }
        oxc_ast::ast::Statement::SwitchStatement(switch) => {
            switch.cases.iter().any(|case| case.consequent.iter().any(has_return_in_statement))
        }
        oxc_ast::ast::Statement::TryStatement(try_stmt) => {
            let block_returns = try_stmt.block.body.iter().any(has_return_in_statement);
            let handler_returns = try_stmt
                .handler
                .as_ref()
                .is_some_and(|h| h.body.body.iter().any(has_return_in_statement));
            let finalizer_returns = try_stmt
                .finalizer
                .as_ref()
                .is_some_and(|f| f.body.iter().any(has_return_in_statement));
            block_returns || handler_returns || finalizer_returns
        }
        _ => false,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // Arrow function with expression body (implicit return)
        r"
        import { computed, signal } from '@angular/core';
        const count = signal(0);
        const double = computed(() => count() * 2);
        ",
        // Arrow function with explicit return
        r"
        import { computed, signal } from '@angular/core';
        const count = signal(0);
        const double = computed(() => {
            return count() * 2;
        });
        ",
        // Function expression with return
        r"
        import { computed, signal } from '@angular/core';
        const count = signal(0);
        const double = computed(function() {
            return count() * 2;
        });
        ",
        // Conditional return
        r"
        import { computed, signal } from '@angular/core';
        const count = signal(0);
        const value = computed(() => {
            if (count() > 0) {
                return 'positive';
            }
            return 'non-positive';
        });
        ",
        // Non-Angular computed
        r"
        import { computed } from 'other-library';
        const value = computed(() => {
            console.log('no return');
        });
        ",
    ];

    let fail = vec![
        // Arrow function without return
        r"
        import { computed, signal } from '@angular/core';
        const count = signal(0);
        const double = computed(() => {
            count() * 2;
        });
        ",
        // Function expression without return
        r"
        import { computed, signal } from '@angular/core';
        const count = signal(0);
        const double = computed(function() {
            count() * 2;
        });
        ",
        // Only has side effects
        r"
        import { computed, signal } from '@angular/core';
        const count = signal(0);
        const value = computed(() => {
            console.log(count());
        });
        ",
        // Return without value
        r"
        import { computed, signal } from '@angular/core';
        const count = signal(0);
        const value = computed(() => {
            if (count() > 0) {
                return;
            }
        });
        ",
    ];

    Tester::new(ComputedMustReturn::NAME, ComputedMustReturn::PLUGIN, pass, fail)
        .test_and_snapshot();
}
