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

/// Legacy structural directives that should be replaced with built-in control flow.
const LEGACY_DIRECTIVES: &[(&str, &str)] = &[
    ("*ngIf", "@if"),
    ("*ngFor", "@for"),
    ("*ngSwitch", "@switch"),
    ("[ngIf]", "@if"),
    ("[ngFor]", "@for"),
    ("[ngSwitch]", "@switch"),
];

fn prefer_control_flow_diagnostic(span: Span, directive: &str, replacement: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Prefer built-in control flow over `{directive}`"))
        .with_help(format!(
            "Replace `{directive}` with `{replacement}` syntax. \
            Built-in control flow was introduced in Angular 17 and is the recommended approach. \
            See https://angular.dev/guide/templates/control-flow for migration guidance."
        ))
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferControlFlow;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces usage of Angular's built-in control flow syntax (`@if`, `@for`, `@switch`)
    /// over legacy structural directives (`*ngIf`, `*ngFor`, `*ngSwitch`).
    ///
    /// ### Why is this bad?
    ///
    /// Angular 17+ introduced built-in control flow blocks that provide better performance,
    /// improved type checking, and cleaner syntax. The legacy structural directives
    /// (`*ngIf`, `*ngFor`, `*ngSwitch`) are still supported but are considered legacy.
    ///
    /// ### Limitations
    ///
    /// This rule only analyzes inline templates (the `template` property in decorators).
    /// It does not analyze external template files (`.html` files referenced via `templateUrl`).
    /// The detection uses pattern matching which may have false positives in edge cases
    /// (e.g., if the pattern appears in a string literal within the template).
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```typescript
    /// import { Component } from '@angular/core';
    ///
    /// @Component({
    ///   selector: 'app-example',
    ///   template: `
    ///     <div *ngIf="isVisible">Content</div>
    ///     <div *ngFor="let item of items">{{ item }}</div>
    ///   `
    /// })
    /// export class ExampleComponent {
    ///   isVisible = true;
    ///   items = ['a', 'b', 'c'];
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```typescript
    /// import { Component } from '@angular/core';
    ///
    /// @Component({
    ///   selector: 'app-example',
    ///   template: `
    ///     @if (isVisible) {
    ///       <div>Content</div>
    ///     }
    ///     @for (item of items; track item) {
    ///       <div>{{ item }}</div>
    ///     }
    ///   `
    /// })
    /// export class ExampleComponent {
    ///   isVisible = true;
    ///   items = ['a', 'b', 'c'];
    /// }
    /// ```
    PreferControlFlow,
    angular,
    pedantic,
    pending // not yet ready for production
);

impl Rule for PreferControlFlow {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::Decorator(decorator) = node.kind() else {
            return;
        };

        // Only check @Component decorators
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

        // Look for the template property
        let Some(template_value) = get_metadata_property(metadata, "template") else {
            // No inline template - skip (could be using templateUrl)
            return;
        };

        // Extract the template string content and check for legacy directives
        match template_value {
            Expression::StringLiteral(str_lit) => {
                check_template_content(str_lit.value.as_str(), str_lit.span, ctx);
            }
            Expression::TemplateLiteral(tpl_lit) => {
                // For template literals, we check each quasi (string part)
                for quasi in &tpl_lit.quasis {
                    check_template_content(quasi.value.raw.as_str(), quasi.span, ctx);
                }
            }
            _ => {
                // Template is not a static string (e.g., variable reference) - can't analyze
            }
        }
    }
}

/// Check template content for legacy directive patterns and report diagnostics.
fn check_template_content(content: &str, base_span: Span, ctx: &LintContext<'_>) {
    for (legacy_directive, replacement) in LEGACY_DIRECTIVES {
        // Look for the directive pattern followed by '=' (attribute binding)
        let pattern = format!("{legacy_directive}=");

        if let Some(offset) = content.find(&pattern) {
            // Calculate the span for the directive occurrence
            // The base_span.start includes the opening quote, so we add 1
            let directive_start = base_span.start + 1 + offset as u32;
            let directive_end = directive_start + legacy_directive.len() as u32;
            let span = Span::new(directive_start, directive_end);

            ctx.diagnostic(prefer_control_flow_diagnostic(span, legacy_directive, replacement));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // Using built-in control flow @if
        r"
        import { Component } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: `
                @if (isVisible) {
                    <div>Content</div>
                }
            `
        })
        class TestComponent {}
        ",
        // Using built-in control flow @for
        r"
        import { Component } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: `
                @for (item of items; track item) {
                    <div>{{ item }}</div>
                }
            `
        })
        class TestComponent {}
        ",
        // Using built-in control flow @switch
        r"
        import { Component } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: `
                @switch (status) {
                    @case ('active') { <span>Active</span> }
                    @default { <span>Unknown</span> }
                }
            `
        })
        class TestComponent {}
        ",
        // Using templateUrl (external template - not analyzed)
        r"
        import { Component } from '@angular/core';
        @Component({
            selector: 'app-test',
            templateUrl: './test.component.html'
        })
        class TestComponent {}
        ",
        // Non-Angular Component decorator (should not trigger)
        r#"
        import { Component } from 'some-other-lib';
        @Component({
            template: '<div *ngIf="show">Content</div>'
        })
        class TestComponent {}
        "#,
        // No template property
        r"
        import { Component } from '@angular/core';
        @Component({
            selector: 'app-test'
        })
        class TestComponent {}
        ",
        // Empty template
        r"
        import { Component } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {}
        ",
        // Template with regular HTML (no directives)
        r#"
        import { Component } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: '<div class="container"><span>Hello</span></div>'
        })
        class TestComponent {}
        "#,
        // Template with Angular event binding (not control flow)
        r#"
        import { Component } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: '<button (click)="onClick()">Click me</button>'
        })
        class TestComponent {}
        "#,
        // Directive with no template property
        r"
        import { Directive } from '@angular/core';
        @Directive({
            selector: '[appTest]'
        })
        class TestDirective {}
        ",
    ];

    let fail = vec![
        // Using *ngIf
        r#"
        import { Component } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: '<div *ngIf="isVisible">Content</div>'
        })
        class TestComponent {}
        "#,
        // Using *ngFor
        r#"
        import { Component } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: '<div *ngFor="let item of items">{{ item }}</div>'
        })
        class TestComponent {}
        "#,
        // Using *ngSwitch
        r#"
        import { Component } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: '<div *ngSwitch="status"><span *ngSwitchCase="active">Active</span></div>'
        })
        class TestComponent {}
        "#,
        // Using [ngIf] binding syntax
        r#"
        import { Component } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: '<ng-template [ngIf]="isVisible"><div>Content</div></ng-template>'
        })
        class TestComponent {}
        "#,
        // Template literal with *ngIf
        r#"
        import { Component } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: `
                <div *ngIf="isVisible">
                    Content
                </div>
            `
        })
        class TestComponent {}
        "#,
    ];

    Tester::new(PreferControlFlow::NAME, PreferControlFlow::PLUGIN, pass, fail).test_and_snapshot();
}
