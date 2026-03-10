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
        get_metadata_property, is_angular_core_import,
    },
};

fn directive_class_suffix_diagnostic(span: Span, suffixes: &[String]) -> OxcDiagnostic {
    let suffix_list = suffixes.join(", ");
    OxcDiagnostic::warn(format!(
        "Directive class names should end with one of these suffixes: {suffix_list}"
    ))
    .with_help("Rename the class to include a valid suffix (e.g., 'HighlightDirective').")
    .with_label(span)
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct DirectiveClassSuffixConfig {
    #[serde(default = "default_suffixes")]
    suffixes: Vec<String>,
}

fn default_suffixes() -> Vec<String> {
    vec!["Directive".to_string()]
}

impl Default for DirectiveClassSuffixConfig {
    fn default() -> Self {
        Self { suffixes: default_suffixes() }
    }
}

#[derive(Debug, Clone)]
pub struct DirectiveClassSuffix {
    suffixes: Vec<String>,
}

impl Default for DirectiveClassSuffix {
    fn default() -> Self {
        Self { suffixes: default_suffixes() }
    }
}

impl From<DirectiveClassSuffixConfig> for DirectiveClassSuffix {
    fn from(config: DirectiveClassSuffixConfig) -> Self {
        Self { suffixes: config.suffixes }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Ensures that directive class names end with a specific suffix (default: "Directive").
    ///
    /// ### Why is this bad?
    ///
    /// Using a consistent suffix for directive classes:
    /// - Makes it easy to identify directives in the codebase
    /// - Follows Angular naming conventions
    /// - Improves code readability and maintainability
    ///
    /// Note: Classes implementing the `Validator` interface are also allowed to use
    /// the "Validator" suffix.
    ///
    /// ### Configuration
    ///
    /// ```json
    /// {
    ///   "angular/directive-class-suffix": ["error", { "suffixes": ["Directive", "Validator"] }]
    /// }
    /// ```
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```typescript
    /// import { Directive } from '@angular/core';
    ///
    /// @Directive({
    ///   selector: '[appHighlight]'
    /// })
    /// export class Highlight {}  // Missing "Directive" suffix
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```typescript
    /// import { Directive } from '@angular/core';
    ///
    /// @Directive({
    ///   selector: '[appHighlight]'
    /// })
    /// export class HighlightDirective {}
    /// ```
    DirectiveClassSuffix,
    angular,
    pedantic,
    pending
);

impl Rule for DirectiveClassSuffix {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::Error> {
        if value.is_null() {
            return Ok(Self::default());
        }
        let config_value = value.get(0).unwrap_or(&value);
        serde_json::from_value::<DirectiveClassSuffixConfig>(config_value.clone()).map(Into::into)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::Decorator(decorator) = node.kind() else {
            return;
        };

        // Only check @Directive decorator
        let Some(decorator_name) = get_decorator_name(decorator) else {
            return;
        };

        if decorator_name != "Directive" {
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

        // Only check directives with a selector (abstract directives without selector are skipped)
        if get_metadata_property(metadata, "selector").is_none() {
            return;
        }

        // Find the parent class
        let Some(class) = get_parent_class_from_decorator(node, ctx) else {
            return;
        };

        // Get the class name
        let Some(class_name) = class.id.as_ref().map(|id| id.name.as_str()) else {
            return;
        };

        // Build list of valid suffixes (include Validator if class implements Validator)
        let mut valid_suffixes = self.suffixes.clone();
        if implements_validator(class) {
            valid_suffixes.push("Validator".to_string());
        }

        // Check if the class name ends with any of the configured suffixes
        let has_valid_suffix = valid_suffixes.iter().any(|suffix| class_name.ends_with(suffix));

        if !has_valid_suffix {
            let span = class.id.as_ref().map_or(decorator.span, |id| id.span);
            ctx.diagnostic(directive_class_suffix_diagnostic(span, &valid_suffixes));
        }
    }
}

fn get_parent_class_from_decorator<'a, 'b>(
    node: &'b crate::AstNode<'a>,
    ctx: &'b LintContext<'a>,
) -> Option<&'b oxc_ast::ast::Class<'a>> {
    for ancestor in ctx.nodes().ancestors(node.id()) {
        if let AstKind::Class(class) = ancestor.kind() {
            return Some(class);
        }
    }
    None
}

fn implements_validator(class: &oxc_ast::ast::Class<'_>) -> bool {
    if class.implements.is_empty() {
        return false;
    }
    class.implements.iter().any(|ts_impl| {
        if let oxc_ast::ast::TSTypeName::IdentifierReference(ident) = &ts_impl.expression {
            return ident.name.as_str() == "Validator" || ident.name.as_str() == "AsyncValidator";
        }
        false
    })
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // Default suffix
        (
            r"
            import { Directive } from '@angular/core';
            @Directive({
                selector: '[appHighlight]'
            })
            class HighlightDirective {}
            ",
            None,
        ),
        // Custom suffix
        (
            r"
            import { Directive } from '@angular/core';
            @Directive({
                selector: '[appHighlight]'
            })
            class HighlightDir {}
            ",
            Some(serde_json::json!([{ "suffixes": ["Dir", "Directive"] }])),
        ),
        // Validator interface allows Validator suffix
        (
            r"
            import { Directive } from '@angular/core';
            @Directive({
                selector: '[appRequired]'
            })
            class RequiredValidator implements Validator {
                validate() { return null; }
            }
            ",
            None,
        ),
        // No selector (abstract directive - skipped)
        (
            r"
            import { Directive } from '@angular/core';
            @Directive({})
            class BaseDirective {}
            ",
            None,
        ),
        // Non-Angular Directive
        (
            r"
            import { Directive } from 'other-lib';
            @Directive({
                selector: '[appTest]'
            })
            class Test {}
            ",
            None,
        ),
    ];

    let fail = vec![
        // Missing Directive suffix
        (
            r"
            import { Directive } from '@angular/core';
            @Directive({
                selector: '[appHighlight]'
            })
            class Highlight {}
            ",
            None,
        ),
        // Wrong suffix
        (
            r"
            import { Directive } from '@angular/core';
            @Directive({
                selector: '[appHighlight]'
            })
            class HighlightComponent {}
            ",
            None,
        ),
    ];

    Tester::new(DirectiveClassSuffix::NAME, DirectiveClassSuffix::PLUGIN, pass, fail)
        .test_and_snapshot();
}
