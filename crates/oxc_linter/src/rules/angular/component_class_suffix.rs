use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use serde::Deserialize;

use crate::{
    AstNode,
    context::LintContext,
    rule::Rule,
    utils::{get_decorator_identifier, get_decorator_name, is_angular_core_import},
};

fn component_class_suffix_diagnostic(span: Span, suffixes: &[String]) -> OxcDiagnostic {
    let suffix_list = suffixes.join(", ");
    OxcDiagnostic::warn(format!(
        "Component class names should end with one of these suffixes: {suffix_list}"
    ))
    .with_help(
        "Rename the class to include a valid suffix (e.g., 'ExampleComponent'). \
        Note: As of Angular v20, this naming convention is no longer officially recommended.",
    )
    .with_label(span)
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct ComponentClassSuffixConfig {
    #[serde(default = "default_suffixes")]
    suffixes: Vec<String>,
}

fn default_suffixes() -> Vec<String> {
    vec!["Component".to_string()]
}

impl Default for ComponentClassSuffixConfig {
    fn default() -> Self {
        Self { suffixes: default_suffixes() }
    }
}

#[derive(Debug, Clone)]
pub struct ComponentClassSuffix {
    suffixes: Vec<String>,
}

impl Default for ComponentClassSuffix {
    fn default() -> Self {
        Self { suffixes: default_suffixes() }
    }
}

impl From<ComponentClassSuffixConfig> for ComponentClassSuffix {
    fn from(config: ComponentClassSuffixConfig) -> Self {
        Self { suffixes: config.suffixes }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Ensures that component class names end with a specific suffix (default: "Component").
    ///
    /// ### Why is this bad?
    ///
    /// Using a consistent suffix for component classes:
    /// - Makes it easy to identify components in the codebase
    /// - Follows Angular naming conventions
    /// - Improves code readability and maintainability
    ///
    /// **Note:** As of Angular v20, this naming convention is no longer officially
    /// recommended by the Angular team, but many teams still find it useful for
    /// consistency.
    ///
    /// ### Configuration
    ///
    /// ```json
    /// {
    ///   "angular/component-class-suffix": ["error", { "suffixes": ["Component", "View"] }]
    /// }
    /// ```
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
    /// export class Example {}  // Missing "Component" suffix
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```typescript
    /// import { Component } from '@angular/core';
    ///
    /// @Component({
    ///   selector: 'app-example',
    ///   template: ''
    /// })
    /// export class ExampleComponent {}
    /// ```
    ComponentClassSuffix,
    angular,
    pedantic,
    pending
);

impl Rule for ComponentClassSuffix {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::Error> {
        if value.is_null() {
            return Ok(Self::default());
        }
        let config_value = value.get(0).unwrap_or(&value);
        serde_json::from_value::<ComponentClassSuffixConfig>(config_value.clone()).map(Into::into)
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

        // Find the parent class
        let Some(class) = get_parent_class_from_decorator(node, ctx) else {
            return;
        };

        // Get the class name
        let Some(class_name) = class.id.as_ref().map(|id| id.name.as_str()) else {
            return;
        };

        // Check if the class name ends with any of the configured suffixes
        let has_valid_suffix = self.suffixes.iter().any(|suffix| class_name.ends_with(suffix));

        if !has_valid_suffix {
            let span = class.id.as_ref().map_or(decorator.span, |id| id.span);
            ctx.diagnostic(component_class_suffix_diagnostic(span, &self.suffixes));
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

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // Default suffix
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
        // Custom suffix
        (
            r"
            import { Component } from '@angular/core';
            @Component({
                selector: 'app-test',
                template: ''
            })
            class TestView {}
            ",
            Some(serde_json::json!([{ "suffixes": ["View", "Component"] }])),
        ),
        // Non-Angular Component
        (
            r"
            import { Component } from 'other-lib';
            @Component({
                selector: 'app-test',
                template: ''
            })
            class Test {}
            ",
            None,
        ),
        // Not a component (Directive)
        (
            r"
            import { Directive } from '@angular/core';
            @Directive({
                selector: '[appTest]'
            })
            class TestDirective {}
            ",
            None,
        ),
    ];

    let fail = vec![
        // Missing Component suffix
        (
            r"
            import { Component } from '@angular/core';
            @Component({
                selector: 'app-test',
                template: ''
            })
            class Test {}
            ",
            None,
        ),
        // Wrong suffix
        (
            r"
            import { Component } from '@angular/core';
            @Component({
                selector: 'app-test',
                template: ''
            })
            class TestComp {}
            ",
            None,
        ),
        // Custom suffixes - not matching
        (
            r"
            import { Component } from '@angular/core';
            @Component({
                selector: 'app-test',
                template: ''
            })
            class TestPage {}
            ",
            Some(serde_json::json!([{ "suffixes": ["Component", "View"] }])),
        ),
    ];

    Tester::new(ComponentClassSuffix::NAME, ComponentClassSuffix::PLUGIN, pass, fail)
        .test_and_snapshot();
}
