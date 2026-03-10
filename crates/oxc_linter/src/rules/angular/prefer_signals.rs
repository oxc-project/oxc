use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    AstNode,
    context::LintContext,
    rule::Rule,
    utils::{
        get_decorator_identifier, get_decorator_name, get_signal_replacement,
        is_angular_core_import, is_legacy_angular_decorator,
    },
};

fn prefer_signals_diagnostic(span: Span, decorator_name: &str, replacement: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "Prefer signal-based `{replacement}()` over legacy `@{decorator_name}()` decorator"
    ))
    .with_help(format!(
        "Replace `@{decorator_name}()` with `{replacement}()` for better performance and reactivity. \
        See https://angular.dev/guide/signals for migration guidance."
    ))
    .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferSignals;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces the use of Angular signal-based APIs (`input()`, `output()`, `viewChild()`, etc.)
    /// over legacy decorators (`@Input()`, `@Output()`, `@ViewChild()`, etc.).
    ///
    /// ### Why is this bad?
    ///
    /// Legacy decorators rely on Angular's zone-based change detection, which checks the entire
    /// component tree for changes. Signal-based APIs provide fine-grained reactivity where only
    /// affected parts of the UI are updated, resulting in better performance.
    ///
    /// Additionally, signals are the future of Angular reactivity and provide better TypeScript
    /// type inference, making code more maintainable.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```typescript
    /// import { Component, Input, Output, EventEmitter, ViewChild } from '@angular/core';
    ///
    /// @Component({ selector: 'app-example', template: '' })
    /// export class ExampleComponent {
    ///   @Input() name: string;
    ///   @Input({ required: true }) id: number;
    ///   @Output() nameChange = new EventEmitter<string>();
    ///   @ViewChild('container') container: ElementRef;
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```typescript
    /// import { Component, input, output, viewChild, ElementRef } from '@angular/core';
    ///
    /// @Component({ selector: 'app-example', template: '' })
    /// export class ExampleComponent {
    ///   name = input<string>();
    ///   id = input.required<number>();
    ///   nameChange = output<string>();
    ///   container = viewChild<ElementRef>('container');
    /// }
    /// ```
    PreferSignals,
    angular,
    pedantic,
    pending // not yet ready for production
);

impl Rule for PreferSignals {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        // We're looking for decorators on class properties
        let AstKind::Decorator(decorator) = node.kind() else {
            return;
        };

        // Get the decorator name (e.g., "Input", "Output", "ViewChild")
        let Some(decorator_name) = get_decorator_name(decorator) else {
            return;
        };

        // Check if it's a legacy Angular decorator we care about
        if !is_legacy_angular_decorator(decorator_name) {
            return;
        }

        // Get the identifier to check import source
        let Some(ident) = get_decorator_identifier(decorator) else {
            return;
        };

        // Verify it's imported from @angular/core
        if !is_angular_core_import(ident, ctx) {
            return;
        }

        // Get the signal-based replacement
        let Some(replacement) = get_signal_replacement(decorator_name) else {
            return;
        };

        ctx.diagnostic(prefer_signals_diagnostic(decorator.span, decorator_name, replacement));
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // Signal-based input
        r"
        import { Component, input } from '@angular/core';
        @Component({ selector: 'app-test', template: '' })
        class TestComponent {
            name = input<string>();
        }
        ",
        // Signal-based required input
        r"
        import { Component, input } from '@angular/core';
        @Component({ selector: 'app-test', template: '' })
        class TestComponent {
            id = input.required<number>();
        }
        ",
        // Signal-based output
        r"
        import { Component, output } from '@angular/core';
        @Component({ selector: 'app-test', template: '' })
        class TestComponent {
            clicked = output<void>();
        }
        ",
        // Signal-based viewChild
        r"
        import { Component, viewChild, ElementRef } from '@angular/core';
        @Component({ selector: 'app-test', template: '' })
        class TestComponent {
            container = viewChild<ElementRef>('container');
        }
        ",
        // Signal-based viewChildren
        r"
        import { Component, viewChildren, ElementRef } from '@angular/core';
        @Component({ selector: 'app-test', template: '' })
        class TestComponent {
            items = viewChildren<ElementRef>('item');
        }
        ",
        // Signal-based contentChild
        r"
        import { Component, contentChild, ElementRef } from '@angular/core';
        @Component({ selector: 'app-test', template: '' })
        class TestComponent {
            content = contentChild<ElementRef>('content');
        }
        ",
        // Signal-based contentChildren
        r"
        import { Component, contentChildren, ElementRef } from '@angular/core';
        @Component({ selector: 'app-test', template: '' })
        class TestComponent {
            items = contentChildren<ElementRef>('item');
        }
        ",
        // Input from a different library (should not trigger)
        r"
        import { Input } from 'some-other-lib';
        class TestComponent {
            @Input() name: string;
        }
        ",
        // Output from a different library (should not trigger)
        r"
        import { Output } from 'some-other-lib';
        class TestComponent {
            @Output() clicked = {};
        }
        ",
        // Component decorator is fine
        r"
        import { Component } from '@angular/core';
        @Component({ selector: 'app-test', template: '' })
        class TestComponent {}
        ",
        // Injectable decorator is fine
        r"
        import { Injectable } from '@angular/core';
        @Injectable({ providedIn: 'root' })
        class TestService {}
        ",
        // Pipe decorator is fine
        r"
        import { Pipe } from '@angular/core';
        @Pipe({ name: 'test' })
        class TestPipe {}
        ",
    ];

    let fail = vec![
        // Legacy @Input decorator
        r"
        import { Component, Input } from '@angular/core';
        @Component({ selector: 'app-test', template: '' })
        class TestComponent {
            @Input() name: string;
        }
        ",
        // Legacy @Input with options
        r"
        import { Component, Input } from '@angular/core';
        @Component({ selector: 'app-test', template: '' })
        class TestComponent {
            @Input({ required: true }) id: number;
        }
        ",
        // Legacy @Output decorator
        r"
        import { Component, Output, EventEmitter } from '@angular/core';
        @Component({ selector: 'app-test', template: '' })
        class TestComponent {
            @Output() clicked = new EventEmitter<void>();
        }
        ",
        // Legacy @ViewChild decorator
        r"
        import { Component, ViewChild, ElementRef } from '@angular/core';
        @Component({ selector: 'app-test', template: '' })
        class TestComponent {
            @ViewChild('container') container: ElementRef;
        }
        ",
        // Legacy @ViewChildren decorator
        r"
        import { Component, ViewChildren, QueryList, ElementRef } from '@angular/core';
        @Component({ selector: 'app-test', template: '' })
        class TestComponent {
            @ViewChildren('item') items: QueryList<ElementRef>;
        }
        ",
        // Legacy @ContentChild decorator
        r"
        import { Component, ContentChild, ElementRef } from '@angular/core';
        @Component({ selector: 'app-test', template: '' })
        class TestComponent {
            @ContentChild('content') content: ElementRef;
        }
        ",
        // Legacy @ContentChildren decorator
        r"
        import { Component, ContentChildren, QueryList, ElementRef } from '@angular/core';
        @Component({ selector: 'app-test', template: '' })
        class TestComponent {
            @ContentChildren('item') items: QueryList<ElementRef>;
        }
        ",
    ];

    Tester::new(PreferSignals::NAME, PreferSignals::PLUGIN, pass, fail).test_and_snapshot();
}
