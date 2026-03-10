use oxc_ast::{AstKind, ast::Expression};
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

fn use_component_selector_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Components should have a selector")
        .with_help(
            "Add a `selector` property to the @Component decorator. Without a selector, \
            the component can only be used programmatically and not in templates.",
        )
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct UseComponentSelector;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Ensures that all `@Component` decorators have a `selector` property defined.
    ///
    /// ### Why is this bad?
    ///
    /// A component without a selector cannot be used in templates and can only be
    /// created programmatically. While this might be intentional for some components
    /// (like those used with route configurations), most components should have a
    /// selector defined.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```typescript
    /// import { Component } from '@angular/core';
    ///
    /// @Component({
    ///   template: '<div>Hello</div>'
    /// })
    /// export class ExampleComponent {}
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```typescript
    /// import { Component } from '@angular/core';
    ///
    /// @Component({
    ///   selector: 'app-example',
    ///   template: '<div>Hello</div>'
    /// })
    /// export class ExampleComponent {}
    /// ```
    UseComponentSelector,
    angular,
    pedantic,
    pending
);

impl Rule for UseComponentSelector {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::Decorator(decorator) = node.kind() else {
            return;
        };

        // Only check @Component decorator
        let Some(decorator_name) = get_decorator_name(decorator) else {
            return;
        };

        if decorator_name != "Component" {
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

        // Check if selector is present and non-empty
        match get_metadata_property(metadata, "selector") {
            None => {
                ctx.diagnostic(use_component_selector_diagnostic(decorator.span));
            }
            Some(Expression::StringLiteral(lit)) if lit.value.is_empty() => {
                ctx.diagnostic(use_component_selector_diagnostic(decorator.span));
            }
            _ => {}
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // Component with selector
        r"
        import { Component } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {}
        ",
        // Component with attribute selector
        r"
        import { Component } from '@angular/core';
        @Component({
            selector: '[appTest]',
            template: ''
        })
        class TestComponent {}
        ",
        // Standalone component with selector
        r"
        import { Component } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: '',
            standalone: true
        })
        class TestComponent {}
        ",
        // Non-Angular Component
        r"
        import { Component } from 'other-lib';
        @Component({
            template: ''
        })
        class TestComponent {}
        ",
        // Directive (not a component)
        r"
        import { Directive } from '@angular/core';
        @Directive({
            selector: '[appTest]'
        })
        class TestDirective {}
        ",
    ];

    let fail = vec![
        // Component without selector
        r"
        import { Component } from '@angular/core';
        @Component({
            template: ''
        })
        class TestComponent {}
        ",
        // Component with empty selector
        r"
        import { Component } from '@angular/core';
        @Component({
            selector: '',
            template: ''
        })
        class TestComponent {}
        ",
        // Standalone component without selector
        r"
        import { Component } from '@angular/core';
        @Component({
            template: '',
            standalone: true
        })
        class TestComponent {}
        ",
        // Component with templateUrl but no selector
        r"
        import { Component } from '@angular/core';
        @Component({
            templateUrl: './test.component.html'
        })
        class TestComponent {}
        ",
    ];

    Tester::new(UseComponentSelector::NAME, UseComponentSelector::PLUGIN, pass, fail)
        .test_and_snapshot();
}
