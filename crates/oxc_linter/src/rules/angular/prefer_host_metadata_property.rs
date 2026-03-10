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

fn prefer_host_metadata_property_diagnostic(span: Span, decorator_name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "Prefer using the `host` metadata property over `@{decorator_name}`"
    ))
    .with_help(
        "Use the `host` property in the @Component or @Directive decorator instead of \
            `@HostBinding` and `@HostListener` decorators. This centralizes host-related \
            configuration and can improve readability.",
    )
    .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferHostMetadataProperty;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces using the `host` metadata property instead of `@HostBinding` and `@HostListener` decorators.
    ///
    /// ### Why is this bad?
    ///
    /// While `@HostBinding` and `@HostListener` decorators work correctly, using the `host` metadata
    /// property in the component/directive decorator has several advantages:
    /// - Centralizes all host-related bindings in one place
    /// - Makes it easier to see all host interactions at a glance
    /// - Reduces decorator clutter on class members
    /// - Aligns with the recommended pattern in Angular style guides
    ///
    /// Note: This rule is the opposite of `no-host-metadata-property`. Use one or the other
    /// based on your team's preference.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```typescript
    /// import { Component, HostBinding, HostListener } from '@angular/core';
    ///
    /// @Component({
    ///   selector: 'app-example',
    ///   template: ''
    /// })
    /// export class ExampleComponent {
    ///   @HostBinding('class.active') isActive = false;
    ///   @HostListener('click') onClick() {}
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```typescript
    /// import { Component } from '@angular/core';
    ///
    /// @Component({
    ///   selector: 'app-example',
    ///   template: '',
    ///   host: {
    ///     '[class.active]': 'isActive',
    ///     '(click)': 'onClick()'
    ///   }
    /// })
    /// export class ExampleComponent {
    ///   isActive = false;
    ///   onClick() {}
    /// }
    /// ```
    PreferHostMetadataProperty,
    angular,
    style,
    pending
);

impl Rule for PreferHostMetadataProperty {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::Decorator(decorator) = node.kind() else {
            return;
        };

        // Check if this is a @HostBinding or @HostListener decorator
        let Some(decorator_name) = get_decorator_name(decorator) else {
            return;
        };

        if !matches!(decorator_name, "HostBinding" | "HostListener") {
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

        ctx.diagnostic(prefer_host_metadata_property_diagnostic(decorator.span, decorator_name));
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
        // Using host metadata property
        r"
        import { Component } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: '',
            host: {
                '[class.active]': 'isActive',
                '(click)': 'onClick()'
            }
        })
        class TestComponent {
            isActive = false;
            onClick() {}
        }
        ",
        // No host bindings at all
        r"
        import { Component } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {}
        ",
        // Non-Angular class
        r"
        import { HostBinding } from 'other-library';
        class TestClass {
            @HostBinding('class.active') isActive = false;
        }
        ",
        // HostBinding in non-component/directive class
        r"
        import { Injectable, HostBinding } from '@angular/core';
        @Injectable({ providedIn: 'root' })
        class TestService {
            @HostBinding('class.active') isActive = false;
        }
        ",
    ];

    let fail = vec![
        // Using @HostBinding
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
        // Using @HostListener
        r"
        import { Component, HostListener } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {
            @HostListener('click') onClick() {}
        }
        ",
        // Both @HostBinding and @HostListener
        r"
        import { Component, HostBinding, HostListener } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {
            @HostBinding('class.active') isActive = false;
            @HostListener('click') onClick() {}
        }
        ",
        // In Directive
        r"
        import { Directive, HostBinding } from '@angular/core';
        @Directive({
            selector: '[appTest]'
        })
        class TestDirective {
            @HostBinding('attr.role') role = 'button';
        }
        ",
        // HostListener with arguments
        r"
        import { Component, HostListener } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {
            @HostListener('keydown', ['$event']) onKeyDown(event: KeyboardEvent) {}
        }
        ",
    ];

    Tester::new(PreferHostMetadataProperty::NAME, PreferHostMetadataProperty::PLUGIN, pass, fail)
        .test_and_snapshot();
}
