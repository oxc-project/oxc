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

fn no_queries_metadata_property_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Avoid using `queries` metadata property")
        .with_help(
            "Use `@ViewChild`, `@ViewChildren`, `@ContentChild`, or `@ContentChildren` decorators \
            instead of the `queries` metadata property. Decorator-based queries provide better \
            type inference and are easier to understand.",
        )
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoQueriesMetadataProperty;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows using the `queries` metadata property in @Component and @Directive decorators.
    ///
    /// ### Why is this bad?
    ///
    /// While Angular supports defining queries in the decorator metadata using the `queries` property,
    /// the preferred approach is to use the query decorators directly on class properties:
    /// - `@ViewChild` and `@ViewChildren` for view queries
    /// - `@ContentChild` and `@ContentChildren` for content queries
    ///
    /// Benefits of using decorators instead of metadata:
    /// - Better type inference and IDE support
    /// - More readable and maintainable code
    /// - Consistent with other Angular patterns
    /// - Easier to see which properties are queries
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```typescript
    /// import { Component, ElementRef } from '@angular/core';
    ///
    /// @Component({
    ///   selector: 'app-example',
    ///   template: '<div #myDiv></div>',
    ///   queries: {
    ///     myDiv: new ViewChild('myDiv')
    ///   }
    /// })
    /// export class ExampleComponent {
    ///   myDiv: ElementRef;
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```typescript
    /// import { Component, ViewChild, ElementRef } from '@angular/core';
    ///
    /// @Component({
    ///   selector: 'app-example',
    ///   template: '<div #myDiv></div>'
    /// })
    /// export class ExampleComponent {
    ///   @ViewChild('myDiv') myDiv: ElementRef;
    /// }
    /// ```
    NoQueriesMetadataProperty,
    angular,
    style,
    pending
);

impl Rule for NoQueriesMetadataProperty {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::Decorator(decorator) = node.kind() else {
            return;
        };

        // Only check @Component and @Directive decorators
        let Some(decorator_name) = get_decorator_name(decorator) else {
            return;
        };

        if !matches!(decorator_name, "Component" | "Directive") {
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

        // Check for 'queries' property
        if get_metadata_property(metadata, "queries").is_some() {
            // Find the span of the 'queries' property
            for prop in &metadata.properties {
                if let oxc_ast::ast::ObjectPropertyKind::ObjectProperty(obj_prop) = prop
                    && let oxc_ast::ast::PropertyKey::StaticIdentifier(ident) = &obj_prop.key
                        && ident.name == "queries" {
                            ctx.diagnostic(no_queries_metadata_property_diagnostic(obj_prop.span));
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
        // Using @ViewChild decorator
        r"
        import { Component, ViewChild, ElementRef } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: '<div #myDiv></div>'
        })
        class TestComponent {
            @ViewChild('myDiv') myDiv: ElementRef;
        }
        ",
        // Using @ContentChild decorator
        r"
        import { Directive, ContentChild, ElementRef } from '@angular/core';
        @Directive({
            selector: '[appTest]'
        })
        class TestDirective {
            @ContentChild('content') content: ElementRef;
        }
        ",
        // No queries property
        r"
        import { Component } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {}
        ",
        // queries property in non-Angular decorator
        r"
        import { Component } from 'other-library';
        @Component({
            selector: 'app-test',
            template: '',
            queries: { myDiv: new ViewChild('myDiv') }
        })
        class TestComponent {}
        ",
    ];

    let fail = vec![
        // queries in @Component
        r"
        import { Component, ViewChild } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: '<div #myDiv></div>',
            queries: {
                myDiv: new ViewChild('myDiv')
            }
        })
        class TestComponent {}
        ",
        // queries in @Directive
        r"
        import { Directive, ContentChild } from '@angular/core';
        @Directive({
            selector: '[appTest]',
            queries: {
                content: new ContentChild('content')
            }
        })
        class TestDirective {}
        ",
        // Multiple queries
        r"
        import { Component, ViewChild, ViewChildren, QueryList } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: '',
            queries: {
                item: new ViewChild('item'),
                items: new ViewChildren('item')
            }
        })
        class TestComponent {}
        ",
    ];

    Tester::new(NoQueriesMetadataProperty::NAME, NoQueriesMetadataProperty::PLUGIN, pass, fail)
        .test_and_snapshot();
}
