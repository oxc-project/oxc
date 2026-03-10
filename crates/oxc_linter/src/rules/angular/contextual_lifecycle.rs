use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    AstNode,
    context::LintContext,
    rule::Rule,
    utils::{
        AngularDecoratorType, get_class_angular_decorator, is_lifecycle_method,
        is_lifecycle_valid_for_decorator,
    },
};

fn contextual_lifecycle_diagnostic(
    span: Span,
    method_name: &str,
    decorator_name: &str,
) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "Angular will not invoke the `{method_name}` lifecycle method within `@{decorator_name}()` classes"
    ))
    .with_help(format!(
        "Remove `{method_name}` or move the class to an appropriate Angular class type that supports this lifecycle method"
    ))
    .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct ContextualLifecycle;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Ensures that lifecycle methods are used in appropriate Angular class types.
    ///
    /// ### Why is this bad?
    ///
    /// Certain lifecycle methods are only invoked by Angular in specific class types:
    /// - `@Component` and `@Directive`: All lifecycle methods except `ngDoBootstrap`
    /// - `@Injectable` and `@Pipe`: Only `ngOnDestroy`
    /// - `@NgModule`: Only `ngDoBootstrap`
    ///
    /// Using a lifecycle method in an inappropriate context will result in dead code
    /// that is never executed, which can lead to confusion and bugs.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```typescript
    /// import { Injectable, OnInit } from '@angular/core';
    ///
    /// @Injectable({ providedIn: 'root' })
    /// export class MyService implements OnInit {
    ///   ngOnInit() {
    ///     // This will never be called!
    ///   }
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```typescript
    /// import { Injectable, OnDestroy } from '@angular/core';
    ///
    /// @Injectable({ providedIn: 'root' })
    /// export class MyService implements OnDestroy {
    ///   ngOnDestroy() {
    ///     // This will be called when the service is destroyed
    ///   }
    /// }
    /// ```
    ContextualLifecycle,
    angular,
    correctness,
    pending
);

impl Rule for ContextualLifecycle {
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

        // Find the parent class
        let Some(class) = get_parent_class(node, ctx) else {
            return;
        };

        // Check if the class has an Angular decorator and get its type
        let Some((decorator_type, _)) = get_class_angular_decorator(class, ctx) else {
            return;
        };

        // Check if the lifecycle method is valid for this decorator type
        if !is_lifecycle_valid_for_decorator(&method_name, decorator_type) {
            let decorator_name = match decorator_type {
                AngularDecoratorType::Component => "Component",
                AngularDecoratorType::Directive => "Directive",
                AngularDecoratorType::Injectable => "Injectable",
                AngularDecoratorType::Pipe => "Pipe",
                AngularDecoratorType::NgModule => "NgModule",
            };
            ctx.diagnostic(contextual_lifecycle_diagnostic(
                method.span,
                &method_name,
                decorator_name,
            ));
        }
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
        // Component with valid lifecycle methods
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
        // Directive with valid lifecycle methods
        r"
        import { Directive, OnInit, AfterViewInit } from '@angular/core';
        @Directive({
            selector: '[appTest]'
        })
        class TestDirective implements OnInit, AfterViewInit {
            ngOnInit() {}
            ngAfterViewInit() {}
        }
        ",
        // Injectable with ngOnDestroy
        r"
        import { Injectable, OnDestroy } from '@angular/core';
        @Injectable({ providedIn: 'root' })
        class TestService implements OnDestroy {
            ngOnDestroy() {}
        }
        ",
        // Pipe with ngOnDestroy
        r"
        import { Pipe, OnDestroy } from '@angular/core';
        @Pipe({ name: 'myPipe' })
        class MyPipe implements OnDestroy {
            ngOnDestroy() {}
        }
        ",
        // NgModule with ngDoBootstrap
        r"
        import { NgModule, DoBootstrap } from '@angular/core';
        @NgModule({})
        class AppModule implements DoBootstrap {
            ngDoBootstrap() {}
        }
        ",
        // Non-Angular class (ignored)
        r"
        class TestClass {
            ngOnInit() {}
        }
        ",
        // Component with all valid hooks
        r"
        import { Component, OnInit, OnChanges, DoCheck, AfterContentInit, AfterContentChecked, AfterViewInit, AfterViewChecked, OnDestroy } from '@angular/core';
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
    ];

    let fail = vec![
        // Injectable with ngOnInit (invalid)
        r"
        import { Injectable, OnInit } from '@angular/core';
        @Injectable({ providedIn: 'root' })
        class TestService implements OnInit {
            ngOnInit() {}
        }
        ",
        // Injectable with ngAfterViewInit (invalid)
        r"
        import { Injectable, AfterViewInit } from '@angular/core';
        @Injectable({ providedIn: 'root' })
        class TestService implements AfterViewInit {
            ngAfterViewInit() {}
        }
        ",
        // Pipe with ngOnInit (invalid)
        r"
        import { Pipe, OnInit } from '@angular/core';
        @Pipe({ name: 'myPipe' })
        class MyPipe implements OnInit {
            ngOnInit() {}
        }
        ",
        // NgModule with ngOnInit (invalid)
        r"
        import { NgModule, OnInit } from '@angular/core';
        @NgModule({})
        class AppModule implements OnInit {
            ngOnInit() {}
        }
        ",
        // NgModule with ngOnDestroy (invalid)
        r"
        import { NgModule, OnDestroy } from '@angular/core';
        @NgModule({})
        class AppModule implements OnDestroy {
            ngOnDestroy() {}
        }
        ",
        // Component with ngDoBootstrap (invalid - only for NgModule)
        r"
        import { Component, DoBootstrap } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent implements DoBootstrap {
            ngDoBootstrap() {}
        }
        ",
    ];

    Tester::new(ContextualLifecycle::NAME, ContextualLifecycle::PLUGIN, pass, fail)
        .test_and_snapshot();
}
