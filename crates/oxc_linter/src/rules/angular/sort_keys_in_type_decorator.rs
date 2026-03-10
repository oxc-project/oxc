use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
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

fn sort_keys_diagnostic(span: Span, decorator: &str, expected_order: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "Keys in @{decorator} decorator should be ordered: {expected_order}"
    ))
    .with_help(
        "Maintaining a consistent order for properties in Angular decorators makes code more \
        predictable and easier to scan.",
    )
    .with_label(span)
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "PascalCase", default, deny_unknown_fields)]
pub struct SortKeysInTypeDecoratorConfig {
    #[serde(default = "default_component_order")]
    component: Vec<String>,
    #[serde(default = "default_directive_order")]
    directive: Vec<String>,
    #[serde(default = "default_ng_module_order")]
    ng_module: Vec<String>,
    #[serde(default = "default_pipe_order")]
    pipe: Vec<String>,
}

fn default_component_order() -> Vec<String> {
    vec![
        "selector",
        "imports",
        "standalone",
        "templateUrl",
        "template",
        "styleUrl",
        "styleUrls",
        "styles",
        "providers",
        "changeDetection",
        "encapsulation",
        "viewProviders",
        "host",
        "hostDirectives",
        "inputs",
        "outputs",
        "animations",
        "schemas",
        "exportAs",
        "queries",
        "preserveWhitespaces",
        "jit",
        "moduleId",
        "interpolation",
    ]
    .into_iter()
    .map(String::from)
    .collect()
}

fn default_directive_order() -> Vec<String> {
    vec![
        "selector",
        "standalone",
        "providers",
        "host",
        "hostDirectives",
        "inputs",
        "outputs",
        "exportAs",
        "queries",
        "jit",
    ]
    .into_iter()
    .map(String::from)
    .collect()
}

fn default_ng_module_order() -> Vec<String> {
    vec!["id", "imports", "declarations", "providers", "exports", "bootstrap", "schemas", "jit"]
        .into_iter()
        .map(String::from)
        .collect()
}

fn default_pipe_order() -> Vec<String> {
    vec!["name", "standalone", "pure"].into_iter().map(String::from).collect()
}

#[derive(Debug, Clone, Default)]
#[expect(clippy::struct_field_names)]
pub struct SortKeysInTypeDecorator {
    component_order: Vec<String>,
    directive_order: Vec<String>,
    ng_module_order: Vec<String>,
    pipe_order: Vec<String>,
}

impl From<SortKeysInTypeDecoratorConfig> for SortKeysInTypeDecorator {
    fn from(config: SortKeysInTypeDecoratorConfig) -> Self {
        Self {
            component_order: if config.component.is_empty() {
                default_component_order()
            } else {
                config.component
            },
            directive_order: if config.directive.is_empty() {
                default_directive_order()
            } else {
                config.directive
            },
            ng_module_order: if config.ng_module.is_empty() {
                default_ng_module_order()
            } else {
                config.ng_module
            },
            pipe_order: if config.pipe.is_empty() { default_pipe_order() } else { config.pipe },
        }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Ensures that keys in type decorators (`@Component`, `@Directive`, `@NgModule`, `@Pipe`)
    /// are sorted in a consistent order.
    ///
    /// ### Why is this bad?
    ///
    /// Maintaining a consistent order for properties in Angular decorators makes code more
    /// predictable and easier to scan. When all components in a codebase follow the same
    /// property order, developers can quickly locate specific metadata without searching.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```typescript
    /// @Component({
    ///   template: '',
    ///   selector: 'app-example', // selector should come first
    /// })
    /// export class ExampleComponent {}
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```typescript
    /// @Component({
    ///   selector: 'app-example',
    ///   template: '',
    /// })
    /// export class ExampleComponent {}
    /// ```
    ///
    /// ### Configuration
    ///
    /// ```json
    /// {
    ///   "angular/sort-keys-in-type-decorator": ["error", {
    ///     "Component": ["selector", "imports", "standalone", "templateUrl", "template"],
    ///     "Directive": ["selector", "standalone", "providers"],
    ///     "NgModule": ["imports", "declarations", "providers", "exports"],
    ///     "Pipe": ["name", "standalone", "pure"]
    ///   }]
    /// }
    /// ```
    SortKeysInTypeDecorator,
    angular,
    style,
    pending
);

impl Rule for SortKeysInTypeDecorator {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::Error> {
        if value.is_null() {
            return Ok(Self::from(SortKeysInTypeDecoratorConfig::default()));
        }
        let config_value = value.get(0).unwrap_or(&value);
        serde_json::from_value::<SortKeysInTypeDecoratorConfig>(config_value.clone())
            .map(Into::into)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::Decorator(decorator) = node.kind() else {
            return;
        };

        let Some(decorator_name) = get_decorator_name(decorator) else {
            return;
        };

        // Get the expected order based on decorator type
        let expected_order = match decorator_name {
            "Component" => &self.component_order,
            "Directive" => &self.directive_order,
            "NgModule" => &self.ng_module_order,
            "Pipe" => &self.pipe_order,
            _ => return,
        };

        if expected_order.is_empty() {
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

        // Get property names in order
        let property_names: Vec<&str> = metadata
            .properties
            .iter()
            .filter_map(|prop| {
                if let oxc_ast::ast::ObjectPropertyKind::ObjectProperty(obj_prop) = prop
                    && let oxc_ast::ast::PropertyKey::StaticIdentifier(ident) = &obj_prop.key {
                        return Some(ident.name.as_str());
                    }
                None
            })
            .collect();

        if property_names.len() <= 1 {
            return;
        }

        // Check if the order is correct
        let configured_props: Vec<&str> = property_names
            .iter()
            .filter(|name| expected_order.iter().any(|e| e == *name))
            .copied()
            .collect();

        let expected_configured_props: Vec<&str> = expected_order
            .iter()
            .filter(|name| configured_props.contains(&name.as_str()))
            .map(String::as_str)
            .collect();

        // Find the first property that's out of order
        let mut out_of_order_span: Option<Span> = None;

        for (i, name) in configured_props.iter().enumerate() {
            if i < expected_configured_props.len() && *name != expected_configured_props[i] {
                // Find the span of the out-of-order property
                for prop in &metadata.properties {
                    if let oxc_ast::ast::ObjectPropertyKind::ObjectProperty(obj_prop) = prop
                        && let oxc_ast::ast::PropertyKey::StaticIdentifier(ident) = &obj_prop.key
                            && ident.name.as_str() == *name {
                                out_of_order_span = Some(obj_prop.key.span());
                                break;
                            }
                }
                break;
            }
        }

        // Also check for unconfigured props that come before configured props
        if out_of_order_span.is_none() {
            let first_configured_idx =
                property_names.iter().position(|name| expected_order.iter().any(|e| e == *name));
            let last_unconfigured_idx =
                property_names.iter().rposition(|name| !expected_order.iter().any(|e| e == *name));

            if let (Some(first_cfg), Some(last_uncfg)) =
                (first_configured_idx, last_unconfigured_idx)
                && last_uncfg < first_cfg {
                    // Unconfigured property comes before configured - find its span
                    let uncfg_name = property_names[last_uncfg];
                    for prop in &metadata.properties {
                        if let oxc_ast::ast::ObjectPropertyKind::ObjectProperty(obj_prop) = prop
                            && let oxc_ast::ast::PropertyKey::StaticIdentifier(ident) =
                                &obj_prop.key
                                && ident.name.as_str() == uncfg_name {
                                    out_of_order_span = Some(obj_prop.key.span());
                                    break;
                                }
                    }
                }
        }

        if let Some(span) = out_of_order_span {
            let relevant_order: Vec<&str> = expected_order
                .iter()
                .filter(|name| property_names.contains(&name.as_str()))
                .map(String::as_str)
                .collect();

            ctx.diagnostic(sort_keys_diagnostic(span, decorator_name, &relevant_order.join(", ")));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // Correct order for Component
        r"
        import { Component } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: '',
            styles: []
        })
        class TestComponent {}
        ",
        // Correct order for Directive
        r"
        import { Directive } from '@angular/core';
        @Directive({
            selector: '[appTest]',
            standalone: true,
            providers: []
        })
        class TestDirective {}
        ",
        // Correct order for NgModule
        r"
        import { NgModule } from '@angular/core';
        @NgModule({
            imports: [],
            declarations: [],
            providers: [],
            exports: []
        })
        class TestModule {}
        ",
        // Correct order for Pipe
        r"
        import { Pipe } from '@angular/core';
        @Pipe({
            name: 'testPipe',
            standalone: true,
            pure: true
        })
        class TestPipe {}
        ",
        // Single property (no order needed)
        r"
        import { Component } from '@angular/core';
        @Component({
            selector: 'app-test'
        })
        class TestComponent {}
        ",
        // Non-Angular decorator
        r"
        import { Component } from 'other-lib';
        @Component({
            template: '',
            selector: 'app-test'
        })
        class TestComponent {}
        ",
    ];

    let fail = vec![
        // Wrong order for Component (template before selector)
        r"
        import { Component } from '@angular/core';
        @Component({
            template: '',
            selector: 'app-test'
        })
        class TestComponent {}
        ",
        // Wrong order for Component (styles before template)
        r"
        import { Component } from '@angular/core';
        @Component({
            selector: 'app-test',
            styles: [],
            template: ''
        })
        class TestComponent {}
        ",
        // Wrong order for Directive
        r"
        import { Directive } from '@angular/core';
        @Directive({
            providers: [],
            selector: '[appTest]'
        })
        class TestDirective {}
        ",
        // Wrong order for NgModule
        r"
        import { NgModule } from '@angular/core';
        @NgModule({
            exports: [],
            imports: []
        })
        class TestModule {}
        ",
        // Wrong order for Pipe
        r"
        import { Pipe } from '@angular/core';
        @Pipe({
            pure: true,
            name: 'testPipe'
        })
        class TestPipe {}
        ",
    ];

    Tester::new(SortKeysInTypeDecorator::NAME, SortKeysInTypeDecorator::PLUGIN, pass, fail)
        .test_and_snapshot();
}
