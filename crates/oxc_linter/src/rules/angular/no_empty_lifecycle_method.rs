use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    AstNode,
    context::LintContext,
    rule::Rule,
    utils::{get_class_angular_decorator, is_lifecycle_method},
};

fn no_empty_lifecycle_method_diagnostic(span: Span, method_name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Lifecycle method `{method_name}` should not be empty"))
        .with_help("Either implement the lifecycle method or remove it from the class")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoEmptyLifecycleMethod;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows empty lifecycle method implementations in Angular components, directives,
    /// pipes, and services.
    ///
    /// ### Why is this bad?
    ///
    /// Empty lifecycle methods are unnecessary code that adds noise and confusion.
    /// They serve no purpose and should be removed to keep the codebase clean.
    /// If a lifecycle method is intentionally empty (e.g., for interface compliance),
    /// consider adding a comment explaining why.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```typescript
    /// import { Component, OnInit } from '@angular/core';
    ///
    /// @Component({
    ///   selector: 'app-example',
    ///   template: ''
    /// })
    /// export class ExampleComponent implements OnInit {
    ///   ngOnInit() {}
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```typescript
    /// import { Component, OnInit } from '@angular/core';
    ///
    /// @Component({
    ///   selector: 'app-example',
    ///   template: ''
    /// })
    /// export class ExampleComponent implements OnInit {
    ///   ngOnInit() {
    ///     this.loadData();
    ///   }
    /// }
    /// ```
    NoEmptyLifecycleMethod,
    angular,
    correctness,
    pending
);

impl Rule for NoEmptyLifecycleMethod {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::MethodDefinition(method) = node.kind() else {
            return;
        };

        // Get the method name
        let Some(method_name) = method.key.static_name() else {
            return;
        };

        // Check if it's a lifecycle method
        if !is_lifecycle_method(&method_name) {
            return;
        }

        // Check if the method body is empty
        let Some(body) = &method.value.body else {
            return;
        };

        if !body.statements.is_empty() {
            return;
        }

        // Find the parent class and check if it's an Angular class
        let Some(class) = get_parent_class(node, ctx) else {
            return;
        };

        // Check if the class has an Angular decorator
        if get_class_angular_decorator(class, ctx).is_none() {
            return;
        }

        ctx.diagnostic(no_empty_lifecycle_method_diagnostic(method.span, &method_name));
    }
}

fn get_parent_class<'a, 'b>(
    node: &'b AstNode<'a>,
    ctx: &'b LintContext<'a>,
) -> Option<&'b oxc_ast::ast::Class<'a>> {
    for ancestor in ctx.nodes().ancestors(node.id()) {
        if let AstKind::Class(class) = ancestor.kind() {
            return Some(class);
        }
    }
    None
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // Lifecycle method with implementation
        r"
        import { Component, OnInit } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent implements OnInit {
            ngOnInit() {
                console.log('initialized');
            }
        }
        ",
        // Multiple lifecycle methods with implementations
        r"
        import { Component, OnInit, OnDestroy } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent implements OnInit, OnDestroy {
            ngOnInit() {
                this.setup();
            }
            ngOnDestroy() {
                this.cleanup();
            }
        }
        ",
        // No lifecycle methods
        r"
        import { Component } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {
            onClick() {}
        }
        ",
        // Non-Angular class with empty lifecycle-like method
        r"
        class TestClass {
            ngOnInit() {}
        }
        ",
        // Directive with implementation
        r"
        import { Directive, OnInit } from '@angular/core';
        @Directive({
            selector: '[appTest]'
        })
        class TestDirective implements OnInit {
            ngOnInit() {
                this.init();
            }
        }
        ",
        // Injectable with ngOnDestroy implementation
        r"
        import { Injectable, OnDestroy } from '@angular/core';
        @Injectable({ providedIn: 'root' })
        class TestService implements OnDestroy {
            ngOnDestroy() {
                this.subscription.unsubscribe();
            }
        }
        ",
    ];

    let fail = vec![
        // Empty ngOnInit
        r"
        import { Component, OnInit } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent implements OnInit {
            ngOnInit() {}
        }
        ",
        // Empty ngOnDestroy
        r"
        import { Component, OnDestroy } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent implements OnDestroy {
            ngOnDestroy() {}
        }
        ",
        // Empty ngAfterViewInit
        r"
        import { Component, AfterViewInit } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent implements AfterViewInit {
            ngAfterViewInit() {}
        }
        ",
        // Directive with empty lifecycle
        r"
        import { Directive, OnInit } from '@angular/core';
        @Directive({
            selector: '[appTest]'
        })
        class TestDirective implements OnInit {
            ngOnInit() {}
        }
        ",
        // Multiple empty lifecycle methods (reports both)
        r"
        import { Component, OnInit, OnDestroy } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent implements OnInit, OnDestroy {
            ngOnInit() {}
            ngOnDestroy() {}
        }
        ",
    ];

    Tester::new(NoEmptyLifecycleMethod::NAME, NoEmptyLifecycleMethod::PLUGIN, pass, fail)
        .test_and_snapshot();
}
