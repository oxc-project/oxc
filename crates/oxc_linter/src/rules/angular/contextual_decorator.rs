use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    AstNode,
    context::LintContext,
    rule::Rule,
    utils::{
        AngularDecoratorType, get_class_angular_decorator, get_decorator_identifier,
        get_decorator_name, is_angular_core_import,
    },
};

fn contextual_decorator_diagnostic(
    span: Span,
    decorator_name: &str,
    class_decorator: &str,
) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "`@{decorator_name}` cannot be used in `@{class_decorator}` class"
    ))
    .with_help(format!(
        "The `@{decorator_name}` decorator is not valid in a class decorated with `@{class_decorator}`. \
        This decorator is only allowed in @Component or @Directive classes."
    ))
    .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct ContextualDecorator;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Ensures that decorators are only used in the correct class context.
    ///
    /// ### Why is this bad?
    ///
    /// Angular decorators have specific contexts where they are valid:
    /// - `@Input`, `@Output`, `@HostBinding`, `@HostListener`, `@ViewChild`, `@ViewChildren`,
    ///   `@ContentChild`, `@ContentChildren` are only valid in `@Component` and `@Directive` classes.
    /// - Using these decorators in `@Injectable`, `@NgModule`, or `@Pipe` classes will cause
    ///   unexpected behavior or runtime errors.
    ///
    /// Only `@Host`, `@Inject`, `@Optional`, `@Self`, and `@SkipSelf` can be used in any Angular class.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```typescript
    /// import { Injectable, Input } from '@angular/core';
    ///
    /// @Injectable({ providedIn: 'root' })
    /// export class MyService {
    ///   @Input() myInput: string; // Invalid - @Input cannot be used in @Injectable
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```typescript
    /// import { Component, Input } from '@angular/core';
    ///
    /// @Component({
    ///   selector: 'app-example',
    ///   template: ''
    /// })
    /// export class ExampleComponent {
    ///   @Input() myInput: string; // Valid - @Input in @Component
    /// }
    /// ```
    ContextualDecorator,
    angular,
    correctness,
    pending
);

/// Decorators that are only valid in @Component and @Directive
const COMPONENT_DIRECTIVE_ONLY_DECORATORS: [&str; 8] = [
    "Input",
    "Output",
    "HostBinding",
    "HostListener",
    "ViewChild",
    "ViewChildren",
    "ContentChild",
    "ContentChildren",
];

impl Rule for ContextualDecorator {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::Decorator(decorator) = node.kind() else {
            return;
        };

        // Get decorator name
        let Some(decorator_name) = get_decorator_name(decorator) else {
            return;
        };

        // Check if this is a component/directive-only decorator
        if !COMPONENT_DIRECTIVE_ONLY_DECORATORS.contains(&decorator_name) {
            return;
        }

        // Verify it's from @angular/core
        let Some(ident) = get_decorator_identifier(decorator) else {
            return;
        };

        if !is_angular_core_import(ident, ctx) {
            return;
        }

        // Find the parent class
        let Some(class) = get_parent_class(node, ctx) else {
            return;
        };

        // Get the class's Angular decorator
        let Some((class_decorator_type, _)) = get_class_angular_decorator(class, ctx) else {
            return;
        };

        // Check if the decorator is used in an invalid context
        let is_valid_context = matches!(
            class_decorator_type,
            AngularDecoratorType::Component | AngularDecoratorType::Directive
        );

        if !is_valid_context {
            let class_decorator_name = match class_decorator_type {
                AngularDecoratorType::Injectable => "Injectable",
                AngularDecoratorType::NgModule => "NgModule",
                AngularDecoratorType::Pipe => "Pipe",
                _ => return,
            };

            ctx.diagnostic(contextual_decorator_diagnostic(
                decorator.span,
                decorator_name,
                class_decorator_name,
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
        // @Input in @Component
        r"
        import { Component, Input } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {
            @Input() value: string;
        }
        ",
        // @Output in @Directive
        r"
        import { Directive, Output, EventEmitter } from '@angular/core';
        @Directive({
            selector: '[appTest]'
        })
        class TestDirective {
            @Output() myEvent = new EventEmitter();
        }
        ",
        // @HostBinding in @Component
        r"
        import { Component, HostBinding } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {
            @HostBinding('class.active') isActive = false;
        }
        ",
        // @ViewChild in @Component
        r"
        import { Component, ViewChild, ElementRef } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {
            @ViewChild('myElement') element: ElementRef;
        }
        ",
        // @Inject in @Injectable (valid - DI decorator)
        r"
        import { Injectable, Inject } from '@angular/core';
        @Injectable({ providedIn: 'root' })
        class TestService {
            constructor(@Inject('TOKEN') private value: string) {}
        }
        ",
        // Non-Angular decorator
        r"
        import { Input } from 'other-library';
        class TestClass {
            @Input() value: string;
        }
        ",
    ];

    let fail = vec![
        // @Input in @Injectable
        r"
        import { Injectable, Input } from '@angular/core';
        @Injectable({ providedIn: 'root' })
        class TestService {
            @Input() value: string;
        }
        ",
        // @Output in @Pipe
        r"
        import { Pipe, Output, EventEmitter } from '@angular/core';
        @Pipe({ name: 'test' })
        class TestPipe {
            @Output() myEvent = new EventEmitter();
        }
        ",
        // @HostBinding in @NgModule
        r"
        import { NgModule, HostBinding } from '@angular/core';
        @NgModule({})
        class TestModule {
            @HostBinding('class.active') isActive = false;
        }
        ",
        // @ViewChild in @Injectable
        r"
        import { Injectable, ViewChild, ElementRef } from '@angular/core';
        @Injectable({ providedIn: 'root' })
        class TestService {
            @ViewChild('element') element: ElementRef;
        }
        ",
        // @ContentChild in @Pipe
        r"
        import { Pipe, ContentChild, ElementRef } from '@angular/core';
        @Pipe({ name: 'test' })
        class TestPipe {
            @ContentChild('content') content: ElementRef;
        }
        ",
    ];

    Tester::new(ContextualDecorator::NAME, ContextualDecorator::PLUGIN, pass, fail)
        .test_and_snapshot();
}
