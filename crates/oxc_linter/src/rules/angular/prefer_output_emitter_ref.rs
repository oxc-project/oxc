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

fn prefer_output_emitter_ref_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Prefer using `output()` function over `@Output()` decorator")
        .with_help(
            "Use the `output()` function instead of `@Output()` with `EventEmitter`. \
            The `output()` function provides better type inference, cleaner syntax, \
            and integrates better with Angular's signal-based architecture. \
            Example: `myEvent = output<string>();`",
        )
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferOutputEmitterRef;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces using the `output()` function instead of the `@Output()` decorator with `EventEmitter`.
    ///
    /// ### Why is this bad?
    ///
    /// The traditional `@Output()` decorator with `EventEmitter` has been the standard way to
    /// define outputs in Angular, but the newer `output()` function (introduced in Angular 17.3)
    /// provides several advantages:
    /// - Better type inference without manual type parameters
    /// - Cleaner, more concise syntax
    /// - Better integration with Angular's signal-based architecture
    /// - Consistent with the `input()` function pattern
    /// - Returns `OutputEmitterRef` which has a simpler API than `EventEmitter`
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```typescript
    /// import { Component, Output, EventEmitter } from '@angular/core';
    ///
    /// @Component({
    ///   selector: 'app-example',
    ///   template: ''
    /// })
    /// export class ExampleComponent {
    ///   @Output() myEvent = new EventEmitter<string>();
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```typescript
    /// import { Component, output } from '@angular/core';
    ///
    /// @Component({
    ///   selector: 'app-example',
    ///   template: ''
    /// })
    /// export class ExampleComponent {
    ///   myEvent = output<string>();
    /// }
    /// ```
    PreferOutputEmitterRef,
    angular,
    pedantic,
    pending
);

impl Rule for PreferOutputEmitterRef {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::Decorator(decorator) = node.kind() else {
            return;
        };

        // Check if this is an @Output decorator
        let Some(decorator_name) = get_decorator_name(decorator) else {
            return;
        };

        if decorator_name != "Output" {
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

        // Check if the class has a @Component or @Directive decorator
        let Some((decorator_type, _)) = get_class_angular_decorator(class, ctx) else {
            return;
        };

        if !matches!(
            decorator_type,
            AngularDecoratorType::Component | AngularDecoratorType::Directive
        ) {
            return;
        }

        ctx.diagnostic(prefer_output_emitter_ref_diagnostic(decorator.span));
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
        // Using output() function
        r"
        import { Component, output } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {
            myEvent = output<string>();
        }
        ",
        // Using output() with alias
        r"
        import { Component, output } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {
            myEvent = output({ alias: 'onMyEvent' });
        }
        ",
        // No outputs
        r"
        import { Component } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {}
        ",
        // Non-Angular class with @Output
        r"
        import { Output, EventEmitter } from 'other-library';
        class TestClass {
            @Output() myEvent = new EventEmitter();
        }
        ",
        // @Output in non-component/directive class
        r"
        import { Injectable, Output, EventEmitter } from '@angular/core';
        @Injectable({ providedIn: 'root' })
        class TestService {
            @Output() myEvent = new EventEmitter();
        }
        ",
    ];

    let fail = vec![
        // @Output with EventEmitter
        r"
        import { Component, Output, EventEmitter } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {
            @Output() myEvent = new EventEmitter<string>();
        }
        ",
        // @Output with alias
        r"
        import { Component, Output, EventEmitter } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {
            @Output('onMyEvent') myEvent = new EventEmitter();
        }
        ",
        // Multiple @Output decorators
        r"
        import { Component, Output, EventEmitter } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {
            @Output() eventA = new EventEmitter();
            @Output() eventB = new EventEmitter();
        }
        ",
        // @Output in Directive
        r"
        import { Directive, Output, EventEmitter } from '@angular/core';
        @Directive({
            selector: '[appTest]'
        })
        class TestDirective {
            @Output() myEvent = new EventEmitter();
        }
        ",
        // @Output with typed EventEmitter
        r"
        import { Component, Output, EventEmitter } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {
            @Output() itemSelected = new EventEmitter<{ id: number; name: string }>();
        }
        ",
    ];

    Tester::new(PreferOutputEmitterRef::NAME, PreferOutputEmitterRef::PLUGIN, pass, fail)
        .test_and_snapshot();
}
