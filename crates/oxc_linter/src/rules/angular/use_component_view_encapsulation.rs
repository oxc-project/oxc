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

fn use_component_view_encapsulation_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Avoid using `ViewEncapsulation.None`")
        .with_help(
            "Using `ViewEncapsulation.None` makes component styles global, which can lead to \
            unintended style conflicts. Use `ViewEncapsulation.Emulated` (default) or \
            `ViewEncapsulation.ShadowDom` instead.",
        )
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct UseComponentViewEncapsulation;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows the use of `ViewEncapsulation.None` in Angular components.
    ///
    /// ### Why is this bad?
    ///
    /// Using `ViewEncapsulation.None` removes Angular's style encapsulation, causing
    /// all component styles to become global. This can lead to:
    /// - Unintended style conflicts with other components
    /// - Difficulty maintaining and debugging styles
    /// - CSS specificity issues
    /// - Styles leaking into or out of components
    ///
    /// Use `ViewEncapsulation.Emulated` (the default) or `ViewEncapsulation.ShadowDom`
    /// to keep styles scoped to the component.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```typescript
    /// import { Component, ViewEncapsulation } from '@angular/core';
    ///
    /// @Component({
    ///   selector: 'app-example',
    ///   template: '',
    ///   encapsulation: ViewEncapsulation.None
    /// })
    /// export class ExampleComponent {}
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```typescript
    /// import { Component, ViewEncapsulation } from '@angular/core';
    ///
    /// @Component({
    ///   selector: 'app-example',
    ///   template: ''
    ///   // Using default Emulated encapsulation
    /// })
    /// export class ExampleComponent {}
    ///
    /// @Component({
    ///   selector: 'app-shadow',
    ///   template: '',
    ///   encapsulation: ViewEncapsulation.ShadowDom
    /// })
    /// export class ShadowComponent {}
    /// ```
    UseComponentViewEncapsulation,
    angular,
    pedantic,
    pending
);

impl Rule for UseComponentViewEncapsulation {
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

        // Check if encapsulation is set to ViewEncapsulation.None
        let Some(encapsulation) = get_metadata_property(metadata, "encapsulation") else {
            return;
        };

        if is_view_encapsulation_none(encapsulation) {
            let span = find_property_span(metadata, "encapsulation").unwrap_or(decorator.span);
            ctx.diagnostic(use_component_view_encapsulation_diagnostic(span));
        }
    }
}

fn is_view_encapsulation_none(expr: &Expression<'_>) -> bool {
    match expr {
        // ViewEncapsulation.None
        Expression::StaticMemberExpression(member) => {
            if let Expression::Identifier(obj) = &member.object {
                obj.name.as_str() == "ViewEncapsulation" && member.property.name.as_str() == "None"
            } else {
                false
            }
        }
        // Numeric literal 2 (ViewEncapsulation.None = 2)
        Expression::NumericLiteral(lit) => {
            #[expect(clippy::float_cmp)]
            {
                lit.value == 2.0
            }
        }
        _ => false,
    }
}

fn find_property_span(obj: &oxc_ast::ast::ObjectExpression<'_>, key: &str) -> Option<Span> {
    use oxc_ast::ast::{ObjectPropertyKind, PropertyKey};
    use oxc_span::GetSpan;

    for property in &obj.properties {
        if let ObjectPropertyKind::ObjectProperty(prop) = property
            && let PropertyKey::StaticIdentifier(ident) = &prop.key
                && ident.name.as_str() == key {
                    return Some(prop.span());
                }
    }
    None
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // Default encapsulation (Emulated)
        r"
        import { Component } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {}
        ",
        // Explicit Emulated encapsulation
        r"
        import { Component, ViewEncapsulation } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: '',
            encapsulation: ViewEncapsulation.Emulated
        })
        class TestComponent {}
        ",
        // ShadowDom encapsulation
        r"
        import { Component, ViewEncapsulation } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: '',
            encapsulation: ViewEncapsulation.ShadowDom
        })
        class TestComponent {}
        ",
        // Non-Angular Component
        r"
        import { Component, ViewEncapsulation } from 'other-lib';
        @Component({
            selector: 'app-test',
            template: '',
            encapsulation: ViewEncapsulation.None
        })
        class TestComponent {}
        ",
    ];

    let fail = vec![
        // ViewEncapsulation.None
        r"
        import { Component, ViewEncapsulation } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: '',
            encapsulation: ViewEncapsulation.None
        })
        class TestComponent {}
        ",
        // Numeric value 2 (None)
        r"
        import { Component } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: '',
            encapsulation: 2
        })
        class TestComponent {}
        ",
        // Standalone component with None
        r"
        import { Component, ViewEncapsulation } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: '',
            standalone: true,
            encapsulation: ViewEncapsulation.None
        })
        class TestComponent {}
        ",
    ];

    Tester::new(
        UseComponentViewEncapsulation::NAME,
        UseComponentViewEncapsulation::PLUGIN,
        pass,
        fail,
    )
    .test_and_snapshot();
}
