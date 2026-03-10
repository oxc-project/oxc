use oxc_ast::{AstKind, ast::Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use serde::Deserialize;

use crate::{
    AstNode,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
    utils::{
        get_component_metadata, get_decorator_identifier, get_decorator_name,
        get_metadata_property, is_angular_core_import,
    },
};

fn standalone_false_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Component should use standalone architecture")
        .with_help(
            "Remove `standalone: false` to use standalone components. \
            In Angular 20, components are standalone by default. \
            See https://angular.dev/guide/components/importing for migration guidance.",
        )
        .with_label(span)
}

fn standalone_redundant_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Redundant `standalone: true` property")
        .with_help(
            "In Angular 20, components are standalone by default. \
            You can remove the explicit `standalone: true` declaration.",
        )
        .with_label(span)
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct PreferStandalone {
    /// Whether to warn on redundant `standalone: true` (default: false)
    warn_on_redundant: bool,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces standalone component architecture by flagging components that use
    /// `standalone: false` and optionally warning about redundant `standalone: true`.
    ///
    /// ### Why is this bad?
    ///
    /// In Angular 20, standalone components are the default and recommended approach.
    /// Using `standalone: false` requires NgModules which adds complexity and is considered
    /// a legacy pattern.
    ///
    /// The `standalone: true` declaration is redundant in Angular 20 since it's the default,
    /// though this warning is optional and disabled by default.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```typescript
    /// import { Component } from '@angular/core';
    ///
    /// // Error: standalone: false is discouraged
    /// @Component({
    ///   selector: 'app-legacy',
    ///   template: '',
    ///   standalone: false
    /// })
    /// export class LegacyComponent {}
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```typescript
    /// import { Component } from '@angular/core';
    ///
    /// // Good: No standalone property (defaults to true in Angular 20)
    /// @Component({
    ///   selector: 'app-modern',
    ///   template: ''
    /// })
    /// export class ModernComponent {}
    /// ```
    ///
    /// ### Options
    ///
    /// ```json
    /// {
    ///   "angular/prefer-standalone": ["error", { "warnOnRedundant": true }]
    /// }
    /// ```
    ///
    /// - `warnOnRedundant`: When `true`, warns about explicit `standalone: true` declarations
    ///   which are redundant in Angular 20. Default is `false`.
    PreferStandalone,
    angular,
    correctness,
    pending // not yet ready for production
);

impl Rule for PreferStandalone {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

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

        // Look for the standalone property
        let Some(standalone_value) = get_metadata_property(metadata, "standalone") else {
            // No standalone property - this is fine (defaults to true in Angular 20)
            return;
        };

        // Check the value
        if let Expression::BooleanLiteral(bool_lit) = standalone_value {
            if !bool_lit.value {
                // standalone: false - this is an error
                ctx.diagnostic(standalone_false_diagnostic(bool_lit.span));
            } else if self.warn_on_redundant {
                // standalone: true - this is redundant (optional warning)
                ctx.diagnostic(standalone_redundant_diagnostic(bool_lit.span));
            }
        } else {
            // Non-boolean value (e.g., a variable) - we can't statically analyze this
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // No standalone property (defaults to true in Angular 20)
        r"
        import { Component } from '@angular/core';
        @Component({ selector: 'app-test', template: '' })
        class TestComponent {}
        ",
        // Directive without standalone property
        r"
        import { Directive } from '@angular/core';
        @Directive({ selector: '[appTest]' })
        class TestDirective {}
        ",
        // standalone: true without warnOnRedundant option (default)
        r"
        import { Component } from '@angular/core';
        @Component({ selector: 'app-test', template: '', standalone: true })
        class TestComponent {}
        ",
        // Directive with standalone: true (default option, no warning)
        r"
        import { Directive } from '@angular/core';
        @Directive({ selector: '[appTest]', standalone: true })
        class TestDirective {}
        ",
        // Non-Angular decorator (should not trigger)
        r"
        import { Component } from 'some-other-lib';
        @Component({ selector: 'app-test', standalone: false })
        class TestComponent {}
        ",
        // Directive from non-Angular library (should not trigger)
        r"
        import { Directive } from 'some-other-lib';
        @Directive({ selector: '[appTest]', standalone: false })
        class TestDirective {}
        ",
        // Injectable (not Component/Directive)
        r"
        import { Injectable } from '@angular/core';
        @Injectable({ providedIn: 'root' })
        class TestService {}
        ",
        // Pipe (not Component/Directive)
        r"
        import { Pipe } from '@angular/core';
        @Pipe({ name: 'test', standalone: true })
        class TestPipe {}
        ",
        // Component with templateUrl and no standalone
        r"
        import { Component } from '@angular/core';
        @Component({ selector: 'app-test', templateUrl: './test.html' })
        class TestComponent {}
        ",
        // Component with imports array (implicit standalone)
        r"
        import { Component } from '@angular/core';
        import { CommonModule } from '@angular/common';
        @Component({ selector: 'app-test', template: '', imports: [CommonModule] })
        class TestComponent {}
        ",
    ];

    let fail = vec![
        // standalone: false on Component
        r"
        import { Component } from '@angular/core';
        @Component({ selector: 'app-test', template: '', standalone: false })
        class TestComponent {}
        ",
        // standalone: false on Directive
        r"
        import { Directive } from '@angular/core';
        @Directive({ selector: '[appTest]', standalone: false })
        class TestDirective {}
        ",
        // standalone: false with other metadata
        r"
        import { Component } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: '<div>Test</div>',
            styleUrls: ['./test.css'],
            standalone: false
        })
        class TestComponent {}
        ",
        // standalone: false on Component with providers
        r"
        import { Component } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: '',
            providers: [],
            standalone: false
        })
        class TestComponent {}
        ",
        // standalone: false on attribute directive
        r"
        import { Directive } from '@angular/core';
        @Directive({
            selector: '[appHighlight]',
            standalone: false
        })
        class HighlightDirective {}
        ",
    ];

    Tester::new(PreferStandalone::NAME, PreferStandalone::PLUGIN, pass, fail).test_and_snapshot();
}

#[test]
fn test_warn_on_redundant() {
    use crate::tester::Tester;
    use serde_json::json;

    // When warnOnRedundant is true, standalone: true should trigger a warning
    let pass: Vec<(&str, Option<serde_json::Value>)> = vec![];

    let fail = vec![(
        r"
        import { Component } from '@angular/core';
        @Component({ selector: 'app-test', template: '', standalone: true })
        class TestComponent {}
        ",
        Some(json!([{ "warnOnRedundant": true }])),
    )];

    Tester::new(PreferStandalone::NAME, PreferStandalone::PLUGIN, pass, fail)
        .with_snapshot_suffix("warn_on_redundant")
        .test_and_snapshot();
}
