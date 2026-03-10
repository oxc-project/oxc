use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    AstNode,
    context::LintContext,
    rule::Rule,
    utils::{get_decorator_identifier, get_decorator_name, is_angular_core_import},
};

fn no_attribute_decorator_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "The `@Attribute` decorator is rarely what is required; use `@Input` instead",
    )
    .with_help(
        "`@Attribute` can only obtain a single static value from the DOM attribute and does not \
        support data binding. In most cases, `@Input` is the appropriate choice as it supports \
        both static values and data binding.",
    )
    .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoAttributeDecorator;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows the use of `@Attribute` decorator in Angular classes.
    ///
    /// ### Why is this bad?
    ///
    /// The `@Attribute` decorator is commonly misused. It has significant limitations:
    /// - It can only read a static string value from the DOM attribute
    /// - It cannot support data binding (`[attr]="value"`)
    /// - It reads the value only once during construction
    /// - Changes to the attribute are not reflected
    ///
    /// In most cases, developers intend to use `@Input()` which supports:
    /// - Data binding with `[property]="expression"`
    /// - Static values with `property="value"`
    /// - Change detection and updates
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```typescript
    /// import { Component, Attribute } from '@angular/core';
    ///
    /// @Component({
    ///   selector: 'app-example',
    ///   template: ''
    /// })
    /// export class ExampleComponent {
    ///   constructor(@Attribute('type') private type: string) {}
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
    ///   @Input() type: string;
    /// }
    /// ```
    NoAttributeDecorator,
    angular,
    pedantic,
    pending
);

impl Rule for NoAttributeDecorator {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::Decorator(decorator) = node.kind() else {
            return;
        };

        // Check for @Attribute decorator
        let Some(decorator_name) = get_decorator_name(decorator) else {
            return;
        };

        if decorator_name != "Attribute" {
            return;
        }

        // Verify it's from @angular/core
        let Some(ident) = get_decorator_identifier(decorator) else {
            return;
        };

        if !is_angular_core_import(ident, ctx) {
            return;
        }

        ctx.diagnostic(no_attribute_decorator_diagnostic(decorator.span));
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // Using @Input instead
        r"
        import { Component, Input } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {
            @Input() type: string;
        }
        ",
        // Using input() signal
        r"
        import { Component, input } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {
            type = input<string>();
        }
        ",
        // Non-Angular Attribute decorator
        r"
        import { Attribute } from 'other-lib';
        class TestComponent {
            constructor(@Attribute('type') private type: string) {}
        }
        ",
        // No decorators
        r"
        import { Component } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {
            constructor(private type: string) {}
        }
        ",
    ];

    let fail = vec![
        // @Attribute in constructor
        r"
        import { Component, Attribute } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {
            constructor(@Attribute('type') private type: string) {}
        }
        ",
        // Multiple @Attribute decorators
        r"
        import { Component, Attribute } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {
            constructor(
                @Attribute('type') private type: string,
                @Attribute('size') private size: string
            ) {}
        }
        ",
        // @Attribute in directive
        r"
        import { Directive, Attribute } from '@angular/core';
        @Directive({
            selector: '[appTest]'
        })
        class TestDirective {
            constructor(@Attribute('title') private title: string) {}
        }
        ",
    ];

    Tester::new(NoAttributeDecorator::NAME, NoAttributeDecorator::PLUGIN, pass, fail)
        .test_and_snapshot();
}
