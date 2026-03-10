use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    AstNode,
    context::LintContext,
    rule::Rule,
    utils::{
        find_property_key_span, get_component_metadata, get_decorator_identifier,
        get_decorator_name, is_angular_core_import,
    },
};

fn no_inputs_metadata_property_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Use `@Input` decorator rather than the `inputs` metadata property")
        .with_help(
            "The `inputs` metadata property is a legacy API. Use the `@Input()` decorator \
            or `input()` signal function directly on properties instead.",
        )
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoInputsMetadataProperty;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows using the `inputs` array in `@Component` and `@Directive` metadata.
    ///
    /// ### Why is this bad?
    ///
    /// Using the `inputs` metadata property is a legacy approach. The `@Input()` decorator
    /// or `input()` signal function provides:
    /// - Better type safety
    /// - Better IDE support
    /// - Co-location of metadata with the property
    /// - Easier to understand and maintain
    ///
    /// Note: This rule does not flag `inputs` when used in `hostDirectives` configuration,
    /// as that is a valid use case.
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
    ///   inputs: ['name', 'value: inputValue']
    /// })
    /// export class ExampleComponent {
    ///   name: string;
    ///   value: string;
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
    ///   @Input() name: string;
    ///   @Input('inputValue') value: string;
    /// }
    /// ```
    NoInputsMetadataProperty,
    angular,
    correctness,
    pending
);

impl Rule for NoInputsMetadataProperty {
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

        // Look for the inputs property at the top level (not inside hostDirectives)
        for prop in &metadata.properties {
            if let oxc_ast::ast::ObjectPropertyKind::ObjectProperty(obj_prop) = prop {
                let prop_name = match &obj_prop.key {
                    oxc_ast::ast::PropertyKey::StaticIdentifier(ident) => Some(ident.name.as_str()),
                    oxc_ast::ast::PropertyKey::StringLiteral(lit) => Some(lit.value.as_str()),
                    _ => None,
                };

                if prop_name == Some("inputs") {
                    // This is a top-level inputs property, not inside hostDirectives
                    let span = find_property_key_span(metadata, "inputs").unwrap_or(metadata.span);
                    ctx.diagnostic(no_inputs_metadata_property_diagnostic(span));
                    return;
                }
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // Using @Input decorator
        r"
        import { Component, Input } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {
            @Input() name: string;
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
            name = input<string>();
        }
        ",
        // No inputs at all
        r"
        import { Component } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {}
        ",
        // Directive with @Input
        r"
        import { Directive, Input } from '@angular/core';
        @Directive({
            selector: '[appTest]'
        })
        class TestDirective {
            @Input() value: string;
        }
        ",
        // Non-Angular Component
        r"
        import { Component } from 'other-lib';
        @Component({
            selector: 'app-test',
            template: '',
            inputs: ['name']
        })
        class TestComponent {}
        ",
        // hostDirectives with inputs (allowed)
        r"
        import { Component } from '@angular/core';
        import { SomeDirective } from './some.directive';
        @Component({
            selector: 'app-test',
            template: '',
            hostDirectives: [{
                directive: SomeDirective,
                inputs: ['value']
            }]
        })
        class TestComponent {}
        ",
    ];

    let fail = vec![
        // Component with inputs metadata
        r"
        import { Component } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: '',
            inputs: ['name']
        })
        class TestComponent {
            name: string;
        }
        ",
        // Component with multiple inputs
        r"
        import { Component } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: '',
            inputs: ['name', 'value', 'count']
        })
        class TestComponent {}
        ",
        // Component with aliased inputs
        r"
        import { Component } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: '',
            inputs: ['value: inputValue']
        })
        class TestComponent {}
        ",
        // Directive with inputs metadata
        r"
        import { Directive } from '@angular/core';
        @Directive({
            selector: '[appTest]',
            inputs: ['highlight']
        })
        class TestDirective {}
        ",
        // Empty inputs array (still flagged)
        r"
        import { Component } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: '',
            inputs: []
        })
        class TestComponent {}
        ",
    ];

    Tester::new(NoInputsMetadataProperty::NAME, NoInputsMetadataProperty::PLUGIN, pass, fail)
        .test_and_snapshot();
}
