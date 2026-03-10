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

fn no_outputs_metadata_property_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Use `@Output` decorator rather than the `outputs` metadata property")
        .with_help(
            "The `outputs` metadata property is a legacy API. Use the `@Output()` decorator \
            or `output()` signal function directly on properties instead.",
        )
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoOutputsMetadataProperty;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows using the `outputs` array in `@Component` and `@Directive` metadata.
    ///
    /// ### Why is this bad?
    ///
    /// Using the `outputs` metadata property is a legacy approach. The `@Output()` decorator
    /// or `output()` signal function provides:
    /// - Better type safety
    /// - Better IDE support
    /// - Co-location of metadata with the property
    /// - Easier to understand and maintain
    ///
    /// Note: This rule does not flag `outputs` when used in `hostDirectives` configuration,
    /// as that is a valid use case.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```typescript
    /// import { Component, EventEmitter } from '@angular/core';
    ///
    /// @Component({
    ///   selector: 'app-example',
    ///   template: '',
    ///   outputs: ['change', 'submit: formSubmit']
    /// })
    /// export class ExampleComponent {
    ///   change = new EventEmitter<string>();
    ///   submit = new EventEmitter<void>();
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```typescript
    /// import { Component, Output, EventEmitter } from '@angular/core';
    ///
    /// @Component({
    ///   selector: 'app-example',
    ///   template: ''
    /// })
    /// export class ExampleComponent {
    ///   @Output() change = new EventEmitter<string>();
    ///   @Output('formSubmit') submit = new EventEmitter<void>();
    /// }
    /// ```
    NoOutputsMetadataProperty,
    angular,
    correctness,
    pending
);

impl Rule for NoOutputsMetadataProperty {
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

        // Look for the outputs property at the top level (not inside hostDirectives)
        for prop in &metadata.properties {
            if let oxc_ast::ast::ObjectPropertyKind::ObjectProperty(obj_prop) = prop {
                let prop_name = match &obj_prop.key {
                    oxc_ast::ast::PropertyKey::StaticIdentifier(ident) => Some(ident.name.as_str()),
                    oxc_ast::ast::PropertyKey::StringLiteral(lit) => Some(lit.value.as_str()),
                    _ => None,
                };

                if prop_name == Some("outputs") {
                    // This is a top-level outputs property, not inside hostDirectives
                    let span = find_property_key_span(metadata, "outputs").unwrap_or(metadata.span);
                    ctx.diagnostic(no_outputs_metadata_property_diagnostic(span));
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
        // Using @Output decorator
        r"
        import { Component, Output, EventEmitter } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {
            @Output() change = new EventEmitter<string>();
        }
        ",
        // Using output() signal
        r"
        import { Component, output } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {
            change = output<string>();
        }
        ",
        // No outputs at all
        r"
        import { Component } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {}
        ",
        // Directive with @Output
        r"
        import { Directive, Output, EventEmitter } from '@angular/core';
        @Directive({
            selector: '[appTest]'
        })
        class TestDirective {
            @Output() triggered = new EventEmitter<void>();
        }
        ",
        // Non-Angular Component
        r"
        import { Component } from 'other-lib';
        @Component({
            selector: 'app-test',
            template: '',
            outputs: ['change']
        })
        class TestComponent {}
        ",
        // hostDirectives with outputs (allowed)
        r"
        import { Component } from '@angular/core';
        import { SomeDirective } from './some.directive';
        @Component({
            selector: 'app-test',
            template: '',
            hostDirectives: [{
                directive: SomeDirective,
                outputs: ['valueChange']
            }]
        })
        class TestComponent {}
        ",
    ];

    let fail = vec![
        // Component with outputs metadata
        r"
        import { Component, EventEmitter } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: '',
            outputs: ['change']
        })
        class TestComponent {
            change = new EventEmitter<string>();
        }
        ",
        // Component with multiple outputs
        r"
        import { Component, EventEmitter } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: '',
            outputs: ['change', 'submit', 'cancel']
        })
        class TestComponent {}
        ",
        // Component with aliased outputs
        r"
        import { Component, EventEmitter } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: '',
            outputs: ['submit: formSubmit']
        })
        class TestComponent {}
        ",
        // Directive with outputs metadata
        r"
        import { Directive, EventEmitter } from '@angular/core';
        @Directive({
            selector: '[appTest]',
            outputs: ['activated']
        })
        class TestDirective {}
        ",
        // Empty outputs array (still flagged)
        r"
        import { Component } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: '',
            outputs: []
        })
        class TestComponent {}
        ",
    ];

    Tester::new(NoOutputsMetadataProperty::NAME, NoOutputsMetadataProperty::PLUGIN, pass, fail)
        .test_and_snapshot();
}
