use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    AstNode,
    context::LintContext,
    rule::Rule,
    utils::{
        get_component_metadata, get_decorator_identifier, get_decorator_name,
        get_metadata_property, is_angular_core_import,
    },
};

fn no_host_metadata_property_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Avoid using the `host` metadata property")
        .with_help(
            "Use `@HostBinding()` and `@HostListener()` decorators instead of the `host` metadata property. \
            Decorator-based host bindings provide better type safety and are easier to understand. \
            Alternatively, consider using the `host` property with signal-based syntax in Angular 18+.",
        )
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoHostMetadataProperty;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows using the `host` metadata property in `@Component` and `@Directive` decorators.
    ///
    /// ### Why is this bad?
    ///
    /// Using the `host` metadata property makes it harder to understand what host bindings
    /// and listeners are being used. The `@HostBinding()` and `@HostListener()` decorators
    /// provide better readability, type safety, and are co-located with the class properties
    /// and methods they affect.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
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
    ///
    /// Examples of **correct** code for this rule:
    /// ```typescript
    /// import { Component, HostBinding, HostListener } from '@angular/core';
    ///
    /// @Component({
    ///   selector: 'app-example',
    ///   template: ''
    /// })
    /// export class ExampleComponent {
    ///   @HostBinding('class.active') isActive = false;
    ///
    ///   @HostListener('click')
    ///   onClick() {}
    /// }
    /// ```
    NoHostMetadataProperty,
    angular,
    pedantic,
    pending // not yet ready for production
);

impl Rule for NoHostMetadataProperty {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::Decorator(decorator) = node.kind() else {
            return;
        };

        // Only check @Component and @Directive decorators
        let Some(decorator_name) = get_decorator_name(decorator) else {
            return;
        };

        if decorator_name != "Component" && decorator_name != "Directive" {
            return;
        }

        // Verify it's from @angular/core
        let Some(ident) = get_decorator_identifier(decorator) else {
            return;
        };

        if !is_angular_core_import(ident, ctx) {
            return;
        }

        // Get the metadata object
        let Some(metadata) = get_component_metadata(decorator) else {
            return;
        };

        // Look for the host property
        if get_metadata_property(metadata, "host").is_some() {
            // Find the span of the host property key
            let span = find_property_key_span(metadata, "host");
            ctx.diagnostic(no_host_metadata_property_diagnostic(span));
        }
    }
}

/// Find the span of a property key in an object expression.
fn find_property_key_span(obj: &oxc_ast::ast::ObjectExpression<'_>, key: &str) -> Span {
    use oxc_ast::ast::{ObjectPropertyKind, PropertyKey};
    use oxc_span::GetSpan;

    for property in &obj.properties {
        if let ObjectPropertyKind::ObjectProperty(prop) = property
            && let PropertyKey::StaticIdentifier(ident) = &prop.key
                && ident.name.as_str() == key {
                    return ident.span();
                }
    }

    // Fallback to the object span if we can't find the property
    obj.span
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // No host property - using decorators
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
        // Directive without host property
        r"
        import { Directive, HostBinding } from '@angular/core';
        @Directive({
            selector: '[appTest]'
        })
        class TestDirective {
            @HostBinding('class.highlight') highlight = true;
        }
        ",
        // Component with only basic metadata
        r"
        import { Component } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: '<div>Content</div>'
        })
        class TestComponent {}
        ",
        // Component with multiple HostBinding decorators
        r"
        import { Component, HostBinding } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {
            @HostBinding('class.active') isActive = false;
            @HostBinding('attr.role') role = 'button';
            @HostBinding('style.color') color = 'red';
        }
        ",
        // Directive with HostListener
        r"
        import { Directive, HostListener } from '@angular/core';
        @Directive({
            selector: '[appClickOutside]'
        })
        class ClickOutsideDirective {
            @HostListener('document:click', ['$event'])
            onDocumentClick(event: Event) {}
        }
        ",
        // Non-Angular decorator (should not trigger)
        r"
        import { Component } from 'some-other-lib';
        @Component({
            selector: 'app-test',
            host: { '[class.active]': 'isActive' }
        })
        class TestComponent {}
        ",
        // Directive from non-Angular library
        r"
        import { Directive } from 'some-other-lib';
        @Directive({
            selector: '[appTest]',
            host: { '[class.active]': 'isActive' }
        })
        class TestDirective {}
        ",
        // Injectable (not Component/Directive)
        r"
        import { Injectable } from '@angular/core';
        @Injectable({
            providedIn: 'root'
        })
        class TestService {}
        ",
        // Pipe (not Component/Directive)
        r"
        import { Pipe } from '@angular/core';
        @Pipe({
            name: 'myPipe',
            standalone: true
        })
        class MyPipe {}
        ",
        // Component with inputs/outputs but no host
        r"
        import { Component, Input, Output, EventEmitter } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {
            @Input() value: string;
            @Output() valueChange = new EventEmitter<string>();
        }
        ",
    ];

    let fail = vec![
        // Component with host property
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
        class TestComponent {}
        ",
        // Directive with host property
        r"
        import { Directive } from '@angular/core';
        @Directive({
            selector: '[appTest]',
            host: {
                '[attr.role]': 'role'
            }
        })
        class TestDirective {}
        ",
        // Component with empty host object
        r"
        import { Component } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: '',
            host: {}
        })
        class TestComponent {}
        ",
        // Component with style bindings in host
        r"
        import { Component } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: '',
            host: {
                '[style.width.px]': 'width',
                '[style.height.px]': 'height'
            }
        })
        class TestComponent {}
        ",
        // Directive with event listener in host
        r"
        import { Directive } from '@angular/core';
        @Directive({
            selector: '[appTest]',
            host: {
                '(mouseenter)': 'onMouseEnter()',
                '(mouseleave)': 'onMouseLeave()'
            }
        })
        class TestDirective {}
        ",
    ];

    Tester::new(NoHostMetadataProperty::NAME, NoHostMetadataProperty::PLUGIN, pass, fail)
        .test_and_snapshot();
}
