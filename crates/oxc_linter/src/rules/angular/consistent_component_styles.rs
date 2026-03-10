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

fn consistent_component_styles_diagnostic(span: Span, message_id: &str) -> OxcDiagnostic {
    let (message, help) = match message_id {
        "useStylesString" => (
            "Component styles should use string format",
            "Use a single string instead of an array for component styles: `styles: 'css-here'`",
        ),
        "useStylesArray" => (
            "Component styles should use array format",
            "Use an array for component styles: `styles: ['css-here']`",
        ),
        "useStyleUrl" => (
            "Use `styleUrl` instead of `styleUrls` for a single stylesheet",
            "Use `styleUrl: './style.css'` instead of `styleUrls: ['./style.css']`",
        ),
        "useStyleUrls" => (
            "Use `styleUrls` instead of `styleUrl`",
            "Use `styleUrls: ['./style.css']` instead of `styleUrl: './style.css'`",
        ),
        _ => ("Use consistent style format", "Use consistent style format"),
    };
    OxcDiagnostic::warn(message).with_help(help).with_label(span)
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum StyleFormat {
    #[default]
    String,
    Array,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct ConsistentComponentStylesConfig {
    #[serde(default)]
    format: StyleFormat,
}

#[derive(Debug, Clone)]
pub struct ConsistentComponentStyles {
    format: StyleFormat,
}

impl Default for ConsistentComponentStyles {
    fn default() -> Self {
        Self { format: StyleFormat::String }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces consistent usage of `styles` property format in components.
    ///
    /// ### Why is this bad?
    ///
    /// Angular allows both string and array formats for inline styles:
    /// - `styles: 'css-here'` (string format)
    /// - `styles: ['css-here']` (array format)
    ///
    /// Using a consistent format across your codebase improves readability and
    /// makes it easier to understand the component structure at a glance.
    ///
    /// Note: Since Angular 17, the preferred approach is to use `styleUrl` (singular)
    /// for a single file or `styleUrls` for multiple files. This rule focuses on
    /// the inline `styles` property.
    ///
    /// ### Configuration
    ///
    /// ```json
    /// {
    ///   "angular/consistent-component-styles": ["error", { "format": "string" }]
    /// }
    /// ```
    ///
    /// Options:
    /// - `"string"` (default): Enforce single string format
    /// - `"array"`: Enforce array format
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule (with `"string"` config):
    /// ```typescript
    /// @Component({
    ///   selector: 'app-example',
    ///   template: '',
    ///   styles: ['.class { color: red; }'] // Should be a string
    /// })
    /// export class ExampleComponent {}
    /// ```
    ///
    /// Examples of **correct** code for this rule (with `"string"` config):
    /// ```typescript
    /// @Component({
    ///   selector: 'app-example',
    ///   template: '',
    ///   styles: '.class { color: red; }'
    /// })
    /// export class ExampleComponent {}
    /// ```
    ConsistentComponentStyles,
    angular,
    style,
    pending
);

impl Rule for ConsistentComponentStyles {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::Error> {
        if value.is_null() {
            return Ok(Self::default());
        }
        let config_value = value.get(0).unwrap_or(&value);

        // Handle string shorthand: ["string"] or ["array"]
        if let Some(format_str) = config_value.as_str() {
            let format = match format_str {
                "array" => StyleFormat::Array,
                _ => StyleFormat::String,
            };
            return Ok(Self { format });
        }

        let config: ConsistentComponentStylesConfig = serde_json::from_value(config_value.clone())?;
        Ok(Self { format: config.format })
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

        // Find style-related properties
        for prop in &metadata.properties {
            if let oxc_ast::ast::ObjectPropertyKind::ObjectProperty(obj_prop) = prop {
                let prop_name = match &obj_prop.key {
                    oxc_ast::ast::PropertyKey::StaticIdentifier(ident) => Some(ident.name.as_str()),
                    oxc_ast::ast::PropertyKey::StringLiteral(lit) => Some(lit.value.as_str()),
                    _ => None,
                };

                match prop_name {
                    Some("styles") => {
                        let is_array =
                            matches!(&obj_prop.value, oxc_ast::ast::Expression::ArrayExpression(_));

                        match (&self.format, is_array) {
                            (StyleFormat::String, true) => {
                                // Using array but expecting string
                                if let oxc_ast::ast::Expression::ArrayExpression(array) =
                                    &obj_prop.value
                                {
                                    // Only report if it's a single-element array (could be converted to string)
                                    if array.elements.len() == 1 {
                                        ctx.diagnostic(consistent_component_styles_diagnostic(
                                            obj_prop.span,
                                            "useStylesString",
                                        ));
                                    }
                                }
                            }
                            (StyleFormat::Array, false) => {
                                // Using string but expecting array
                                ctx.diagnostic(consistent_component_styles_diagnostic(
                                    obj_prop.span,
                                    "useStylesArray",
                                ));
                            }
                            _ => {}
                        }
                    }
                    Some("styleUrl") => {
                        // styleUrl is singular - in array mode, we expect styleUrls
                        if matches!(self.format, StyleFormat::Array) {
                            ctx.diagnostic(consistent_component_styles_diagnostic(
                                obj_prop.span,
                                "useStyleUrls",
                            ));
                        }
                    }
                    Some("styleUrls") => {
                        // styleUrls is plural - in string mode with single element, we expect styleUrl
                        if matches!(self.format, StyleFormat::String)
                            && let oxc_ast::ast::Expression::ArrayExpression(array) =
                                &obj_prop.value
                                && array.elements.len() == 1 {
                                    ctx.diagnostic(consistent_component_styles_diagnostic(
                                        obj_prop.span,
                                        "useStyleUrl",
                                    ));
                                }
                    }
                    _ => {}
                }
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // String format with string config (default)
        (
            r"
            import { Component } from '@angular/core';
            @Component({
                selector: 'app-test',
                template: '',
                styles: '.class { color: red; }'
            })
            class TestComponent {}
            ",
            None,
        ),
        // No styles property
        (
            r"
            import { Component } from '@angular/core';
            @Component({
                selector: 'app-test',
                template: ''
            })
            class TestComponent {}
            ",
            None,
        ),
        // Array format with array config
        (
            r"
            import { Component } from '@angular/core';
            @Component({
                selector: 'app-test',
                template: '',
                styles: ['.class { color: red; }']
            })
            class TestComponent {}
            ",
            Some(serde_json::json!([{ "format": "array" }])),
        ),
        // Multiple styles in array (always allowed)
        (
            r"
            import { Component } from '@angular/core';
            @Component({
                selector: 'app-test',
                template: '',
                styles: ['.class1 { color: red; }', '.class2 { color: blue; }']
            })
            class TestComponent {}
            ",
            None,
        ),
        // Template literal string
        (
            r"
            import { Component } from '@angular/core';
            @Component({
                selector: 'app-test',
                template: '',
                styles: `.class { color: red; }`
            })
            class TestComponent {}
            ",
            None,
        ),
        // styleUrl with string config (default) - singular is fine
        (
            r"
            import { Component } from '@angular/core';
            @Component({
                selector: 'app-test',
                template: '',
                styleUrl: './test.component.css'
            })
            class TestComponent {}
            ",
            None,
        ),
        // styleUrls with array config
        (
            r"
            import { Component } from '@angular/core';
            @Component({
                selector: 'app-test',
                template: '',
                styleUrls: ['./test.component.css']
            })
            class TestComponent {}
            ",
            Some(serde_json::json!([{ "format": "array" }])),
        ),
        // Multiple styleUrls (always allowed with string config)
        (
            r"
            import { Component } from '@angular/core';
            @Component({
                selector: 'app-test',
                template: '',
                styleUrls: ['./test1.css', './test2.css']
            })
            class TestComponent {}
            ",
            None,
        ),
    ];

    let fail = vec![
        // Single-element array with string config (default)
        (
            r"
            import { Component } from '@angular/core';
            @Component({
                selector: 'app-test',
                template: '',
                styles: ['.class { color: red; }']
            })
            class TestComponent {}
            ",
            None,
        ),
        // String format with array config
        (
            r"
            import { Component } from '@angular/core';
            @Component({
                selector: 'app-test',
                template: '',
                styles: '.class { color: red; }'
            })
            class TestComponent {}
            ",
            Some(serde_json::json!([{ "format": "array" }])),
        ),
        // Template literal in single-element array
        (
            r"
            import { Component } from '@angular/core';
            @Component({
                selector: 'app-test',
                template: '',
                styles: [`.class { color: red; }`]
            })
            class TestComponent {}
            ",
            None,
        ),
        // Single-element styleUrls with string config - should use styleUrl
        (
            r"
            import { Component } from '@angular/core';
            @Component({
                selector: 'app-test',
                template: '',
                styleUrls: ['./test.component.css']
            })
            class TestComponent {}
            ",
            None,
        ),
        // styleUrl with array config - should use styleUrls
        (
            r"
            import { Component } from '@angular/core';
            @Component({
                selector: 'app-test',
                template: '',
                styleUrl: './test.component.css'
            })
            class TestComponent {}
            ",
            Some(serde_json::json!([{ "format": "array" }])),
        ),
    ];

    Tester::new(ConsistentComponentStyles::NAME, ConsistentComponentStyles::PLUGIN, pass, fail)
        .test_and_snapshot();
}
