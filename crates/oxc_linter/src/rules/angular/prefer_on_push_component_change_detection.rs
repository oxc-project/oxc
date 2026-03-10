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

fn prefer_on_push_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "Component should use `ChangeDetectionStrategy.OnPush` for better performance",
    )
    .with_help(
        "Add `changeDetection: ChangeDetectionStrategy.OnPush` to the component decorator. \
        OnPush change detection only checks the component when its inputs change or when \
        events are triggered within it.",
    )
    .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferOnPushComponentChangeDetection;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces the use of `ChangeDetectionStrategy.OnPush` in Angular components.
    ///
    /// ### Why is this bad?
    ///
    /// Using `ChangeDetectionStrategy.Default` (the default) can lead to performance issues
    /// because Angular will check the component and its children on every change detection cycle.
    ///
    /// `ChangeDetectionStrategy.OnPush` provides better performance by:
    /// - Only checking when input references change
    /// - Only checking when events are triggered within the component
    /// - Only checking when explicitly triggered via `markForCheck()` or `detectChanges()`
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```typescript
    /// import { Component } from '@angular/core';
    ///
    /// @Component({
    ///   selector: 'app-example',
    ///   template: ''
    /// })
    /// export class ExampleComponent {}
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```typescript
    /// import { Component, ChangeDetectionStrategy } from '@angular/core';
    ///
    /// @Component({
    ///   selector: 'app-example',
    ///   template: '',
    ///   changeDetection: ChangeDetectionStrategy.OnPush
    /// })
    /// export class ExampleComponent {}
    /// ```
    PreferOnPushComponentChangeDetection,
    angular,
    pedantic,
    pending
);

impl Rule for PreferOnPushComponentChangeDetection {
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

        // Check if changeDetection is set to OnPush
        match get_metadata_property(metadata, "changeDetection") {
            None => {
                // No changeDetection property - using default
                ctx.diagnostic(prefer_on_push_diagnostic(decorator.span));
            }
            Some(expr) => {
                if !is_on_push(expr) {
                    ctx.diagnostic(prefer_on_push_diagnostic(decorator.span));
                }
            }
        }
    }
}

fn is_on_push(expr: &Expression<'_>) -> bool {
    match expr {
        // ChangeDetectionStrategy.OnPush
        Expression::StaticMemberExpression(member) => {
            if let Expression::Identifier(obj) = &member.object {
                obj.name.as_str() == "ChangeDetectionStrategy"
                    && member.property.name.as_str() == "OnPush"
            } else {
                false
            }
        }
        // Numeric literal 0 (ChangeDetectionStrategy.OnPush = 0)
        Expression::NumericLiteral(lit) => lit.value == 0.0,
        _ => false,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // OnPush change detection
        r"
        import { Component, ChangeDetectionStrategy } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: '',
            changeDetection: ChangeDetectionStrategy.OnPush
        })
        class TestComponent {}
        ",
        // OnPush with numeric value 0
        r"
        import { Component } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: '',
            changeDetection: 0
        })
        class TestComponent {}
        ",
        // Non-Angular Component
        r"
        import { Component } from 'other-lib';
        @Component({
            selector: 'app-test',
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
        // OnPush with string literal key
        r"
        import { Component, ChangeDetectionStrategy } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: '',
            'changeDetection': ChangeDetectionStrategy.OnPush
        })
        class TestComponent {}
        ",
        // OnPush with computed string literal key
        r"
        import { Component, ChangeDetectionStrategy } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: '',
            ['changeDetection']: ChangeDetectionStrategy.OnPush
        })
        class TestComponent {}
        ",
    ];

    let fail = vec![
        // No changeDetection property
        r"
        import { Component } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {}
        ",
        // Default change detection
        r"
        import { Component, ChangeDetectionStrategy } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: '',
            changeDetection: ChangeDetectionStrategy.Default
        })
        class TestComponent {}
        ",
        // Numeric value 1 (Default)
        r"
        import { Component } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: '',
            changeDetection: 1
        })
        class TestComponent {}
        ",
        // Standalone component without OnPush
        r"
        import { Component } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: '',
            standalone: true
        })
        class TestComponent {}
        ",
        // String literal key with Default
        r"
        import { Component, ChangeDetectionStrategy } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: '',
            'changeDetection': ChangeDetectionStrategy.Default
        })
        class TestComponent {}
        ",
        // Computed key with Default
        r"
        import { Component, ChangeDetectionStrategy } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: '',
            ['changeDetection']: ChangeDetectionStrategy.Default
        })
        class TestComponent {}
        ",
    ];

    Tester::new(
        PreferOnPushComponentChangeDetection::NAME,
        PreferOnPushComponentChangeDetection::PLUGIN,
        pass,
        fail,
    )
    .test_and_snapshot();
}
