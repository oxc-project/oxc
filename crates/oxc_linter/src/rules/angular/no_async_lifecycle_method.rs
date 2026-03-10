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

fn no_async_lifecycle_method_diagnostic(span: Span, method_name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Lifecycle method `{method_name}` should not be async"))
        .with_help(
            "Angular does not wait for async lifecycle methods to complete. \
        Remove the async keyword and handle async operations differently, \
        such as subscribing to observables or using promises without await.",
        )
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoAsyncLifecycleMethod;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows using async lifecycle methods in Angular classes.
    ///
    /// ### Why is this bad?
    ///
    /// Angular lifecycle methods are designed to be synchronous. When you mark a lifecycle
    /// method as async, Angular will NOT wait for the async operation to complete before
    /// continuing with its lifecycle processing. This can lead to:
    /// - Race conditions with component initialization
    /// - Unexpected behavior in change detection
    /// - Misleading code that suggests async operations are awaited
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```typescript
    /// import { Component } from '@angular/core';
    ///
    /// @Component({
    ///   selector: 'app-example',
    ///   template: ''
    /// })
    /// export class ExampleComponent {
    ///   async ngOnInit() {
    ///     await this.loadData();
    ///   }
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```typescript
    /// import { Component } from '@angular/core';
    ///
    /// @Component({
    ///   selector: 'app-example',
    ///   template: ''
    /// })
    /// export class ExampleComponent {
    ///   ngOnInit() {
    ///     this.loadData().then(data => {
    ///       // handle data
    ///     });
    ///   }
    /// }
    /// ```
    NoAsyncLifecycleMethod,
    angular,
    correctness,
    pending
);

impl Rule for NoAsyncLifecycleMethod {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::MethodDefinition(method) = node.kind() else {
            return;
        };

        // Get method name
        let Some(method_name) = method.key.static_name() else {
            return;
        };

        // Check if this is a lifecycle method
        if !is_lifecycle_method(&method_name) {
            return;
        }

        // Check if the method is async
        if !method.value.r#async {
            return;
        }

        // Find the parent class
        let Some(class) = get_parent_class(node, ctx) else {
            return;
        };

        // Check if the class has an Angular decorator
        if get_class_angular_decorator(class, ctx).is_none() {
            return;
        }

        // Report the diagnostic on the method
        ctx.diagnostic(no_async_lifecycle_method_diagnostic(method.span, &method_name));
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
        // Non-async lifecycle methods
        r"
        import { Component } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {
            ngOnInit() {}
        }
        ",
        // Non-async ngOnDestroy
        r"
        import { Component } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {
            ngOnDestroy() {}
        }
        ",
        // Async method that is not a lifecycle method
        r"
        import { Component } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {
            async loadData() {}
        }
        ",
        // Standalone async function with lifecycle name (not in Angular class)
        r"
        async function ngOnInit() {}
        ",
        // Non-Angular class with async lifecycle method name
        r"
        class TestClass {
            async ngOnInit() {}
        }
        ",
        // Injectable with non-async ngOnDestroy
        r"
        import { Injectable } from '@angular/core';
        @Injectable({ providedIn: 'root' })
        class TestService {
            ngOnDestroy() {}
        }
        ",
    ];

    let fail = vec![
        // Async ngOnInit
        r"
        import { Component } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {
            async ngOnInit() {}
        }
        ",
        // Async ngOnDestroy
        r"
        import { Component } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {
            async ngOnDestroy() {}
        }
        ",
        // Async ngAfterViewInit
        r"
        import { Component } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {
            async ngAfterViewInit() {}
        }
        ",
        // Directive with async lifecycle
        r"
        import { Directive } from '@angular/core';
        @Directive({
            selector: '[appTest]'
        })
        class TestDirective {
            async ngOnInit() {}
        }
        ",
        // Injectable with async ngOnDestroy
        r"
        import { Injectable } from '@angular/core';
        @Injectable({ providedIn: 'root' })
        class TestService {
            async ngOnDestroy() {}
        }
        ",
        // Pipe with async ngOnDestroy
        r"
        import { Pipe } from '@angular/core';
        @Pipe({ name: 'test' })
        class TestPipe {
            async ngOnDestroy() {}
        }
        ",
        // Multiple async lifecycle methods
        r"
        import { Component } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {
            async ngOnInit() {}
            async ngOnDestroy() {}
        }
        ",
    ];

    Tester::new(NoAsyncLifecycleMethod::NAME, NoAsyncLifecycleMethod::PLUGIN, pass, fail)
        .test_and_snapshot();
}
