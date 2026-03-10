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

fn relative_url_prefix_diagnostic(span: Span, property: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "`{property}` should use relative paths starting with `./` or `../`"
    ))
    .with_help(
        "Use relative paths (starting with `./` or `../`) for `templateUrl` and `styleUrls` \
        to ensure proper resolution in all build scenarios.",
    )
    .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct RelativeUrlPrefix;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Ensures that `templateUrl` and `styleUrls` in `@Component` decorators use relative
    /// paths starting with `./` or `../`.
    ///
    /// ### Why is this bad?
    ///
    /// Using non-relative paths for external templates and styles can cause:
    /// - Build failures in certain configurations
    /// - Inconsistent behavior across different bundlers
    /// - Issues with Angular CLI and component compilation
    ///
    /// The standard syntax for relative URLs requires paths to begin with `./` or `../`.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```typescript
    /// import { Component } from '@angular/core';
    ///
    /// @Component({
    ///   selector: 'app-example',
    ///   templateUrl: 'example.component.html',
    ///   styleUrls: ['example.component.css']
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
    ///   templateUrl: './example.component.html',
    ///   styleUrls: ['./example.component.css']
    /// })
    /// export class ExampleComponent {}
    /// ```
    RelativeUrlPrefix,
    angular,
    pedantic,
    pending
);

impl Rule for RelativeUrlPrefix {
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

        // Check templateUrl
        if let Some(template_url) = get_metadata_property(metadata, "templateUrl")
            && let Expression::StringLiteral(lit) = template_url
                && !is_relative_path(lit.value.as_str()) {
                    ctx.diagnostic(relative_url_prefix_diagnostic(lit.span, "templateUrl"));
                }

        // Check styleUrls (array of strings)
        if let Some(style_urls) = get_metadata_property(metadata, "styleUrls")
            && let Expression::ArrayExpression(arr) = style_urls {
                for element in &arr.elements {
                    // Check if the element is a string literal (either directly or through expression)
                    let string_lit = match element {
                        oxc_ast::ast::ArrayExpressionElement::StringLiteral(lit) => Some(lit),
                        _ => element.as_expression().and_then(|e| {
                            if let Expression::StringLiteral(lit) = e { Some(lit) } else { None }
                        }),
                    };

                    if let Some(lit) = string_lit
                        && !is_relative_path(lit.value.as_str()) {
                            ctx.diagnostic(relative_url_prefix_diagnostic(lit.span, "styleUrls"));
                        }
                }
            }

        // Check styleUrl (single string, Angular 17+)
        if let Some(style_url) = get_metadata_property(metadata, "styleUrl")
            && let Expression::StringLiteral(lit) = style_url
                && !is_relative_path(lit.value.as_str()) {
                    ctx.diagnostic(relative_url_prefix_diagnostic(lit.span, "styleUrl"));
                }
    }
}

fn is_relative_path(path: &str) -> bool {
    path.starts_with("./") || path.starts_with("../")
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // Relative templateUrl
        r"
        import { Component } from '@angular/core';
        @Component({
            selector: 'app-test',
            templateUrl: './test.component.html'
        })
        class TestComponent {}
        ",
        // Relative styleUrls
        r"
        import { Component } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: '',
            styleUrls: ['./test.component.css']
        })
        class TestComponent {}
        ",
        // Multiple relative styleUrls
        r"
        import { Component } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: '',
            styleUrls: ['./test.component.css', '../shared/styles.css']
        })
        class TestComponent {}
        ",
        // Parent directory relative path
        r"
        import { Component } from '@angular/core';
        @Component({
            selector: 'app-test',
            templateUrl: '../templates/test.component.html'
        })
        class TestComponent {}
        ",
        // Inline template (not affected)
        r"
        import { Component } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: '<div>Hello</div>'
        })
        class TestComponent {}
        ",
        // Inline styles (not affected)
        r"
        import { Component } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: '',
            styles: [':host { display: block; }']
        })
        class TestComponent {}
        ",
        // Non-Angular Component
        r"
        import { Component } from 'other-lib';
        @Component({
            selector: 'app-test',
            templateUrl: 'test.component.html'
        })
        class TestComponent {}
        ",
    ];

    let fail = vec![
        // Non-relative templateUrl
        r"
        import { Component } from '@angular/core';
        @Component({
            selector: 'app-test',
            templateUrl: 'test.component.html'
        })
        class TestComponent {}
        ",
        // Non-relative styleUrls
        r"
        import { Component } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: '',
            styleUrls: ['test.component.css']
        })
        class TestComponent {}
        ",
        // Absolute path templateUrl
        r"
        import { Component } from '@angular/core';
        @Component({
            selector: 'app-test',
            templateUrl: '/app/test.component.html'
        })
        class TestComponent {}
        ",
        // Mixed - one relative, one not
        r"
        import { Component } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: '',
            styleUrls: ['./valid.css', 'invalid.css']
        })
        class TestComponent {}
        ",
    ];

    Tester::new(RelativeUrlPrefix::NAME, RelativeUrlPrefix::PLUGIN, pass, fail).test_and_snapshot();
}
