use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    AstNode,
    context::LintContext,
    rule::Rule,
    utils::{get_class_angular_decorator, get_lifecycle_method_order, is_lifecycle_method},
};

fn sort_lifecycle_methods_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Lifecycle methods are not declared in order of execution")
        .with_help(
            "Declare lifecycle methods in the order they are invoked by Angular: \
            ngOnChanges → ngOnInit → ngDoCheck → ngAfterContentInit → ngAfterContentChecked → \
            ngAfterViewInit → ngAfterViewChecked → ngOnDestroy",
        )
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct SortLifecycleMethods;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Ensures that lifecycle methods in Angular classes are declared in the order
    /// they are invoked by Angular.
    ///
    /// ### Why is this bad?
    ///
    /// Declaring lifecycle methods in execution order:
    /// - Makes the code more readable and predictable
    /// - Helps developers understand the component's lifecycle at a glance
    /// - Follows a consistent pattern across the codebase
    /// - Makes it easier to reason about initialization and cleanup logic
    ///
    /// ### Execution Order
    ///
    /// 1. `ngOnChanges` - Called when input properties change
    /// 2. `ngOnInit` - Called once after the first ngOnChanges
    /// 3. `ngDoCheck` - Called during every change detection run
    /// 4. `ngAfterContentInit` - Called after content (ng-content) has been projected
    /// 5. `ngAfterContentChecked` - Called after every check of projected content
    /// 6. `ngAfterViewInit` - Called after the component's view has been initialized
    /// 7. `ngAfterViewChecked` - Called after every check of the component's view
    /// 8. `ngOnDestroy` - Called before the component is destroyed
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```typescript
    /// import { Component, OnInit, OnDestroy } from '@angular/core';
    ///
    /// @Component({
    ///   selector: 'app-example',
    ///   template: ''
    /// })
    /// export class ExampleComponent implements OnInit, OnDestroy {
    ///   ngOnDestroy() {}  // Wrong order
    ///   ngOnInit() {}
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```typescript
    /// import { Component, OnInit, OnDestroy } from '@angular/core';
    ///
    /// @Component({
    ///   selector: 'app-example',
    ///   template: ''
    /// })
    /// export class ExampleComponent implements OnInit, OnDestroy {
    ///   ngOnInit() {}
    ///   ngOnDestroy() {}
    /// }
    /// ```
    SortLifecycleMethods,
    angular,
    pedantic,
    pending
);

impl Rule for SortLifecycleMethods {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::Class(class) = node.kind() else {
            return;
        };

        // Check if the class has an Angular decorator
        if get_class_angular_decorator(class, ctx).is_none() {
            return;
        }

        // Collect lifecycle methods with their positions
        let mut lifecycle_methods: Vec<(usize, String, Span)> = Vec::new();

        for member in &class.body.body {
            if let oxc_ast::ast::ClassElement::MethodDefinition(method) = member
                && let Some(name) = method.key.static_name()
                    && is_lifecycle_method(&name)
                        && let Some(order) = get_lifecycle_method_order(&name) {
                            lifecycle_methods.push((order, name.to_string(), method.span));
                        }
        }

        // Check if methods are in order
        if lifecycle_methods.len() < 2 {
            return;
        }

        let mut prev_order = lifecycle_methods[0].0;
        for (order, _name, span) in lifecycle_methods.iter().skip(1) {
            if *order < prev_order {
                ctx.diagnostic(sort_lifecycle_methods_diagnostic(*span));
            }
            prev_order = *order;
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // Correct order
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
        // Full lifecycle in order
        r"
        import { Component, OnChanges, OnInit, DoCheck, AfterContentInit, AfterContentChecked, AfterViewInit, AfterViewChecked, OnDestroy } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {
            ngOnChanges() {}
            ngOnInit() {}
            ngDoCheck() {}
            ngAfterContentInit() {}
            ngAfterContentChecked() {}
            ngAfterViewInit() {}
            ngAfterViewChecked() {}
            ngOnDestroy() {}
        }
        ",
        // Single lifecycle method
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
        // Non-Angular class
        r"
        class TestClass {
            ngOnDestroy() {}
            ngOnInit() {}
        }
        ",
        // Directive with correct order
        r"
        import { Directive, OnInit, OnDestroy } from '@angular/core';
        @Directive({
            selector: '[appTest]'
        })
        class TestDirective implements OnInit, OnDestroy {
            ngOnInit() {}
            ngOnDestroy() {}
        }
        ",
    ];

    let fail = vec![
        // Wrong order: OnDestroy before OnInit
        r"
        import { Component, OnInit, OnDestroy } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent implements OnInit, OnDestroy {
            ngOnDestroy() {}
            ngOnInit() {}
        }
        ",
        // Wrong order: AfterViewInit before OnInit
        r"
        import { Component, OnInit, AfterViewInit } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent implements OnInit, AfterViewInit {
            ngAfterViewInit() {}
            ngOnInit() {}
        }
        ",
        // Multiple out of order
        r"
        import { Component, OnInit, OnDestroy, AfterViewInit } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {
            ngOnDestroy() {}
            ngAfterViewInit() {}
            ngOnInit() {}
        }
        ",
        // OnChanges after OnInit
        r"
        import { Component, OnInit, OnChanges } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent implements OnInit, OnChanges {
            ngOnInit() {}
            ngOnChanges() {}
        }
        ",
    ];

    Tester::new(SortLifecycleMethods::NAME, SortLifecycleMethods::PLUGIN, pass, fail)
        .test_and_snapshot();
}
