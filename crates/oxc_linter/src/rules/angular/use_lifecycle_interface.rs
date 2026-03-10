use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    AstNode,
    context::LintContext,
    rule::Rule,
    utils::{
        class_implements_interface, get_class_angular_decorator,
        get_lifecycle_interface_for_method, is_lifecycle_method,
    },
};

fn use_lifecycle_interface_diagnostic(
    span: Span,
    method_name: &str,
    interface_name: &str,
) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "Lifecycle interface `{interface_name}` should be implemented for method `{method_name}`"
    ))
    .with_help(format!(
        "Add `implements {interface_name}` to the class declaration and import `{interface_name}` from '@angular/core'"
    ))
    .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct UseLifecycleInterface;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Ensures that classes implementing lifecycle methods also implement the corresponding
    /// lifecycle interfaces from Angular.
    ///
    /// ### Why is this bad?
    ///
    /// Implementing lifecycle interfaces explicitly:
    /// - Provides better type checking and IDE support
    /// - Makes the code more self-documenting
    /// - Helps catch typos in lifecycle method names
    /// - Follows Angular style guide recommendations
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
    ///   ngOnInit() {
    ///     console.log('initialized');
    ///   }
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
    ///     console.log('initialized');
    ///   }
    /// }
    /// ```
    UseLifecycleInterface,
    angular,
    pedantic,
    pending
);

impl Rule for UseLifecycleInterface {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::MethodDefinition(method) = node.kind() else {
            return;
        };

        // Skip methods with override keyword
        if method.r#override {
            return;
        }

        // Get the method name
        let Some(method_name) = method.key.static_name() else {
            return;
        };

        // Check if it's a lifecycle method
        if !is_lifecycle_method(&method_name) {
            return;
        }

        // Get the corresponding interface name
        let Some(interface_name) = get_lifecycle_interface_for_method(&method_name) else {
            return;
        };

        // Find the parent class
        let Some(class) = get_parent_class(node, ctx) else {
            return;
        };

        // Check if the class has an Angular decorator
        if get_class_angular_decorator(class, ctx).is_none() {
            return;
        }

        // Check if the class implements the interface
        if class_implements_interface(class, interface_name) {
            return;
        }

        ctx.diagnostic(use_lifecycle_interface_diagnostic(
            method.span,
            &method_name,
            interface_name,
        ));
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
        // Implements OnInit
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
        // Implements multiple interfaces
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
        // Implements all interfaces
        r"
        import { Component, OnInit, OnChanges, DoCheck, AfterContentInit, AfterContentChecked, AfterViewInit, AfterViewChecked, OnDestroy } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent implements OnInit, OnChanges, DoCheck, AfterContentInit, AfterContentChecked, AfterViewInit, AfterViewChecked, OnDestroy {
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
        // Directive with interface
        r"
        import { Directive, OnInit } from '@angular/core';
        @Directive({
            selector: '[appTest]'
        })
        class TestDirective implements OnInit {
            ngOnInit() {}
        }
        ",
        // Injectable with OnDestroy interface
        r"
        import { Injectable, OnDestroy } from '@angular/core';
        @Injectable({ providedIn: 'root' })
        class TestService implements OnDestroy {
            ngOnDestroy() {}
        }
        ",
        // Override method (should be skipped)
        r"
        import { Component } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent extends BaseComponent {
            override ngOnInit() {}
        }
        ",
        // Non-Angular class (should be ignored)
        r"
        class TestClass {
            ngOnInit() {}
        }
        ",
        // Method with other name (not a lifecycle)
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
    ];

    let fail = vec![
        // Missing OnInit interface
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
        // Missing OnDestroy interface
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
        // Missing AfterViewInit interface
        r"
        import { Component } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {
            ngAfterViewInit() {}
        }
        ",
        // Directive missing interface
        r"
        import { Directive } from '@angular/core';
        @Directive({
            selector: '[appTest]'
        })
        class TestDirective {
            ngOnInit() {}
        }
        ",
        // Injectable missing interface
        r"
        import { Injectable } from '@angular/core';
        @Injectable({ providedIn: 'root' })
        class TestService {
            ngOnDestroy() {}
        }
        ",
        // Missing multiple interfaces (reports each)
        r"
        import { Component } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {
            ngOnInit() {}
            ngOnDestroy() {}
        }
        ",
        // Partial implementation - has one but not the other
        r"
        import { Component, OnInit } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent implements OnInit {
            ngOnInit() {}
            ngOnDestroy() {}
        }
        ",
    ];

    Tester::new(UseLifecycleInterface::NAME, UseLifecycleInterface::PLUGIN, pass, fail)
        .test_and_snapshot();
}
