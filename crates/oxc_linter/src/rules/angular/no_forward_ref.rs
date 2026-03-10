use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule, utils::is_angular_core_import};

fn no_forward_ref_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Avoid using `forwardRef`")
        .with_help(
            "The use of `forwardRef` can often be avoided by restructuring your code. \
            Consider moving the referenced class above the usage, using injection tokens, \
            or refactoring to avoid circular dependencies.",
        )
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoForwardRef;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows usage of `forwardRef` in Angular applications.
    ///
    /// ### Why is this bad?
    ///
    /// `forwardRef` is typically used to work around circular dependencies or to reference
    /// classes that are defined later in the file. While it works, it's often a sign of
    /// code that could be better structured:
    /// - It makes the code harder to understand
    /// - It can hide architectural issues like circular dependencies
    /// - It may indicate that classes should be reordered or split into separate files
    ///
    /// In most cases, `forwardRef` can be avoided by:
    /// - Moving the referenced class above its usage
    /// - Using injection tokens
    /// - Restructuring to avoid circular dependencies
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```typescript
    /// import { Component, forwardRef, Inject } from '@angular/core';
    ///
    /// @Component({
    ///   selector: 'app-example',
    ///   template: ''
    /// })
    /// export class ExampleComponent {
    ///   constructor(@Inject(forwardRef(() => SomeService)) private service: SomeService) {}
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```typescript
    /// import { Component, Inject } from '@angular/core';
    /// import { SomeService } from './some.service';
    ///
    /// @Component({
    ///   selector: 'app-example',
    ///   template: ''
    /// })
    /// export class ExampleComponent {
    ///   constructor(private service: SomeService) {}
    /// }
    /// ```
    NoForwardRef,
    angular,
    pedantic,
    pending
);

impl Rule for NoForwardRef {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        // Check if this is a call to forwardRef
        let callee_name = match &call_expr.callee {
            oxc_ast::ast::Expression::Identifier(ident) => ident.name.as_str(),
            _ => return,
        };

        if callee_name != "forwardRef" {
            return;
        }

        // Verify it's imported from @angular/core
        let Some(ident) = call_expr.callee.get_identifier_reference() else {
            return;
        };

        if !is_angular_core_import(ident, ctx) {
            return;
        }

        ctx.diagnostic(no_forward_ref_diagnostic(call_expr.span));
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // Not using forwardRef
        r"
        import { Component } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {
            constructor(private service: MyService) {}
        }
        ",
        // forwardRef from non-Angular source
        r"
        import { forwardRef } from 'other-library';
        const ref = forwardRef(() => SomeClass);
        ",
        // Regular function call named forwardRef (not imported)
        r"
        function forwardRef(fn) { return fn(); }
        forwardRef(() => 'test');
        ",
    ];

    let fail = vec![
        // Basic forwardRef usage
        r"
        import { forwardRef } from '@angular/core';
        const ref = forwardRef(() => SomeClass);
        ",
        // forwardRef in Inject
        r"
        import { Component, forwardRef, Inject } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {
            constructor(@Inject(forwardRef(() => SomeService)) private service: any) {}
        }
        ",
        // forwardRef in providers
        r"
        import { NgModule, forwardRef } from '@angular/core';
        @NgModule({
            providers: [
                { provide: 'TOKEN', useClass: forwardRef(() => MyService) }
            ]
        })
        class AppModule {}
        ",
    ];

    Tester::new(NoForwardRef::NAME, NoForwardRef::PLUGIN, pass, fail).test_and_snapshot();
}
