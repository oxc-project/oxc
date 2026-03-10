use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use serde::Deserialize;

use crate::{
    AstNode,
    context::LintContext,
    rule::Rule,
    utils::{
        get_component_metadata, get_decorator_identifier, get_decorator_name,
        is_angular_core_import,
    },
};

fn component_max_inline_declarations_diagnostic(
    span: Span,
    property: &str,
    actual: usize,
    max: usize,
) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "Inline `{property}` has too many lines ({actual}). Maximum allowed is {max}."
    ))
    .with_help(format!(
        "Extract the inline {property} to a separate file. For templates use `templateUrl`, \
        for styles use `styleUrl` or `styleUrls`."
    ))
    .with_label(span)
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct ComponentMaxInlineDeclarationsConfig {
    /// Maximum lines for inline template (default: 3)
    #[serde(default = "default_template")]
    template: usize,
    /// Maximum lines for inline styles (default: 3)
    #[serde(default = "default_styles")]
    styles: usize,
    /// Maximum lines for inline animations (default: 15)
    #[serde(default = "default_animations")]
    animations: usize,
}

fn default_template() -> usize {
    3
}

fn default_styles() -> usize {
    3
}

fn default_animations() -> usize {
    15
}

impl Default for ComponentMaxInlineDeclarationsConfig {
    fn default() -> Self {
        Self {
            template: default_template(),
            styles: default_styles(),
            animations: default_animations(),
        }
    }
}

#[derive(Debug, Clone)]
#[expect(clippy::struct_field_names)]
pub struct ComponentMaxInlineDeclarations {
    template_max: usize,
    styles_max: usize,
    animations_max: usize,
}

impl Default for ComponentMaxInlineDeclarations {
    fn default() -> Self {
        Self {
            template_max: default_template(),
            styles_max: default_styles(),
            animations_max: default_animations(),
        }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces a maximum number of lines for inline templates, styles, and animations in components.
    ///
    /// ### Why is this bad?
    ///
    /// Large inline templates, styles, or animations make components harder to read and maintain.
    /// When these become too long, they should be extracted to separate files:
    /// - Templates: Use `templateUrl` instead of `template`
    /// - Styles: Use `styleUrl` or `styleUrls` instead of `styles`
    /// - Animations: Consider extracting to a separate animations file
    ///
    /// ### Configuration
    ///
    /// ```json
    /// {
    ///   "angular/component-max-inline-declarations": ["error", {
    ///     "template": 3,
    ///     "styles": 3,
    ///     "animations": 15
    ///   }]
    /// }
    /// ```
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule (with default config):
    /// ```typescript
    /// @Component({
    ///   selector: 'app-example',
    ///   template: `
    ///     <div>Line 1</div>
    ///     <div>Line 2</div>
    ///     <div>Line 3</div>
    ///     <div>Line 4</div>
    ///   `
    /// })
    /// export class ExampleComponent {}
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```typescript
    /// @Component({
    ///   selector: 'app-example',
    ///   templateUrl: './example.component.html'
    /// })
    /// export class ExampleComponent {}
    /// ```
    ComponentMaxInlineDeclarations,
    angular,
    style,
    pending
);

impl Rule for ComponentMaxInlineDeclarations {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::Error> {
        if value.is_null() {
            return Ok(Self::default());
        }
        let config_value = value.get(0).unwrap_or(&value);
        let config: ComponentMaxInlineDeclarationsConfig =
            serde_json::from_value(config_value.clone())?;
        Ok(Self {
            template_max: config.template,
            styles_max: config.styles,
            animations_max: config.animations,
        })
    }

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

        // Check template
        if let Some((span, line_count)) = get_property_line_count(metadata, "template")
            && line_count > self.template_max {
                ctx.diagnostic(component_max_inline_declarations_diagnostic(
                    span,
                    "template",
                    line_count,
                    self.template_max,
                ));
            }

        // Check styles array
        if let Some((span, line_count)) = get_styles_line_count(metadata)
            && line_count > self.styles_max {
                ctx.diagnostic(component_max_inline_declarations_diagnostic(
                    span,
                    "styles",
                    line_count,
                    self.styles_max,
                ));
            }

        // Check animations
        if let Some((span, line_count)) = get_property_line_count(metadata, "animations")
            && line_count > self.animations_max {
                ctx.diagnostic(component_max_inline_declarations_diagnostic(
                    span,
                    "animations",
                    line_count,
                    self.animations_max,
                ));
            }
    }
}

fn get_property_line_count(
    metadata: &oxc_ast::ast::ObjectExpression<'_>,
    property_name: &str,
) -> Option<(Span, usize)> {
    for prop in &metadata.properties {
        if let oxc_ast::ast::ObjectPropertyKind::ObjectProperty(obj_prop) = prop {
            let prop_name = match &obj_prop.key {
                oxc_ast::ast::PropertyKey::StaticIdentifier(ident) => Some(ident.name.as_str()),
                oxc_ast::ast::PropertyKey::StringLiteral(lit) => Some(lit.value.as_str()),
                _ => None,
            };

            if prop_name == Some(property_name) {
                let line_count = count_lines_in_expression(&obj_prop.value);
                return Some((obj_prop.span, line_count));
            }
        }
    }
    None
}

fn get_styles_line_count(metadata: &oxc_ast::ast::ObjectExpression<'_>) -> Option<(Span, usize)> {
    for prop in &metadata.properties {
        if let oxc_ast::ast::ObjectPropertyKind::ObjectProperty(obj_prop) = prop {
            let prop_name = match &obj_prop.key {
                oxc_ast::ast::PropertyKey::StaticIdentifier(ident) => Some(ident.name.as_str()),
                oxc_ast::ast::PropertyKey::StringLiteral(lit) => Some(lit.value.as_str()),
                _ => None,
            };

            if prop_name == Some("styles") {
                // styles can be an array or a single string
                // For arrays, SUM the lines of all elements (not max)
                let line_count = match &obj_prop.value {
                    oxc_ast::ast::Expression::ArrayExpression(array) => array
                        .elements
                        .iter()
                        .filter_map(|el| el.as_expression().map(count_lines_in_expression))
                        .sum(),
                    expr => count_lines_in_expression(expr),
                };
                return Some((obj_prop.span, line_count));
            }
        }
    }
    None
}

fn count_lines_in_expression(expr: &oxc_ast::ast::Expression<'_>) -> usize {
    match expr {
        oxc_ast::ast::Expression::StringLiteral(lit) => count_lines(&lit.value),
        oxc_ast::ast::Expression::TemplateLiteral(lit) => {
            // Count lines in the template literal
            lit.quasis.iter().map(|quasi| count_lines(&quasi.value.raw)).sum()
        }
        oxc_ast::ast::Expression::ArrayExpression(array) => {
            // For arrays (like animations), count total lines
            array
                .elements
                .iter()
                .filter_map(|el| el.as_expression().map(count_lines_in_expression))
                .sum()
        }
        _ => 0,
    }
}

fn count_lines(s: &str) -> usize {
    // Count newlines + 1 for the content
    let newline_count = s.chars().filter(|&c| c == '\n').count();
    if s.is_empty() { 0 } else { newline_count + 1 }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // Using templateUrl
        (
            r"
            import { Component } from '@angular/core';
            @Component({
                selector: 'app-test',
                templateUrl: './test.component.html'
            })
            class TestComponent {}
            ",
            None,
        ),
        // Short inline template
        (
            r"
            import { Component } from '@angular/core';
            @Component({
                selector: 'app-test',
                template: '<div>Short</div>'
            })
            class TestComponent {}
            ",
            None,
        ),
        // Template within limit (3 lines)
        (
            r"
            import { Component } from '@angular/core';
            @Component({
                selector: 'app-test',
                template: `<div>Line 1</div>
<div>Line 2</div>
<div>Line 3</div>`
            })
            class TestComponent {}
            ",
            None,
        ),
        // Custom higher limit
        (
            r"
            import { Component } from '@angular/core';
            @Component({
                selector: 'app-test',
                template: `
                    <div>Line 1</div>
                    <div>Line 2</div>
                    <div>Line 3</div>
                    <div>Line 4</div>
                    <div>Line 5</div>
                `
            })
            class TestComponent {}
            ",
            Some(serde_json::json!([{ "template": 10 }])),
        ),
    ];

    let fail = vec![
        // Template exceeds default limit
        (
            r"
            import { Component } from '@angular/core';
            @Component({
                selector: 'app-test',
                template: `
                    <div>Line 1</div>
                    <div>Line 2</div>
                    <div>Line 3</div>
                    <div>Line 4</div>
                    <div>Line 5</div>
                `
            })
            class TestComponent {}
            ",
            None,
        ),
        // Styles exceed limit
        (
            r"
            import { Component } from '@angular/core';
            @Component({
                selector: 'app-test',
                template: '',
                styles: [`
                    .class1 { color: red; }
                    .class2 { color: blue; }
                    .class3 { color: green; }
                    .class4 { color: yellow; }
                `]
            })
            class TestComponent {}
            ",
            None,
        ),
        // Custom lower limit exceeded
        (
            r"
            import { Component } from '@angular/core';
            @Component({
                selector: 'app-test',
                template: `<div>
                    Content
                </div>`
            })
            class TestComponent {}
            ",
            Some(serde_json::json!([{ "template": 1 }])),
        ),
    ];

    Tester::new(
        ComponentMaxInlineDeclarations::NAME,
        ComponentMaxInlineDeclarations::PLUGIN,
        pass,
        fail,
    )
    .test_and_snapshot();
}
