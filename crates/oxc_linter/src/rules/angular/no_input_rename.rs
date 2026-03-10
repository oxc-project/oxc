use oxc_ast::{AstKind, ast::Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use serde::Deserialize;

use crate::{
    AstNode,
    context::LintContext,
    rule::Rule,
    utils::{
        get_decorator_call, get_decorator_identifier, get_decorator_name, is_angular_core_import,
    },
};

fn no_input_rename_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Input bindings should not be aliased")
        .with_help(
            "Avoid aliasing inputs as it can lead to confusion. Use the original property name \
            or rename the property if the alias is more appropriate.",
        )
        .with_label(span)
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct NoInputRenameConfig {
    #[serde(default)]
    allowed_names: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct NoInputRename {
    allowed_names: Vec<String>,
}

impl From<NoInputRenameConfig> for NoInputRename {
    fn from(config: NoInputRenameConfig) -> Self {
        Self { allowed_names: config.allowed_names }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows aliasing input bindings (renaming inputs with a different public name).
    ///
    /// ### Why is this bad?
    ///
    /// Two names for the same property (one private, one public) is confusing.
    /// It requires developers to remember both names and understand the mapping.
    ///
    /// Exceptions are made for:
    /// - Cases where the alias matches the property name (redundant but harmless)
    /// - Names specified in the `allowedNames` configuration
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```typescript
    /// import { Component, Input } from '@angular/core';
    ///
    /// @Component({
    ///   selector: 'app-example',
    ///   template: ''
    /// })
    /// export class ExampleComponent {
    ///   @Input('label') name: string; // Aliased input
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```typescript
    /// import { Component, Input } from '@angular/core';
    ///
    /// @Component({
    ///   selector: 'app-example',
    ///   template: ''
    /// })
    /// export class ExampleComponent {
    ///   @Input() name: string; // No alias
    /// }
    /// ```
    ///
    /// ### Configuration
    ///
    /// ```json
    /// {
    ///   "angular/no-input-rename": ["error", { "allowedNames": ["appCustom"] }]
    /// }
    /// ```
    NoInputRename,
    angular,
    correctness,
    pending
);

impl Rule for NoInputRename {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::Error> {
        if value.is_null() {
            return Ok(Self::default());
        }
        let config_value = value.get(0).unwrap_or(&value);
        serde_json::from_value::<NoInputRenameConfig>(config_value.clone()).map(Into::into)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        // Check for @Input decorator
        if let AstKind::Decorator(decorator) = node.kind() {
            self.check_input_decorator(decorator, node, ctx);
            return;
        }

        // Check for input() signal function
        if let AstKind::PropertyDefinition(prop) = node.kind() {
            self.check_input_signal(prop, ctx);
        }
    }
}

impl NoInputRename {
    fn check_input_decorator(
        &self,
        decorator: &oxc_ast::ast::Decorator<'_>,
        _node: &AstNode<'_>,
        ctx: &LintContext<'_>,
    ) {
        let Some(decorator_name) = get_decorator_name(decorator) else {
            return;
        };

        if decorator_name != "Input" {
            return;
        }

        // Verify it's from @angular/core
        let Some(ident) = get_decorator_identifier(decorator) else {
            return;
        };

        if !is_angular_core_import(ident, ctx) {
            return;
        }

        // Get the decorator call to check for alias
        let Some(call) = get_decorator_call(decorator) else {
            return;
        };

        // Get the first argument (the alias)
        let Some(first_arg) = call.arguments.first() else {
            return;
        };

        let alias = match first_arg {
            oxc_ast::ast::Argument::StringLiteral(lit) => Some(lit.value.as_str()),
            oxc_ast::ast::Argument::ObjectExpression(obj) => {
                // Check for { alias: 'name' } format
                get_alias_from_object(obj)
            }
            _ => None,
        };

        let Some(alias) = alias else {
            return;
        };

        // Get the property name from parent
        let property_name = self.get_property_name_from_decorator(_node, ctx);

        // Allow if alias matches property name
        if property_name.as_deref() == Some(alias) {
            return;
        }

        // Allow if alias is in allowed names
        if self.allowed_names.iter().any(|name| name == alias) {
            return;
        }

        ctx.diagnostic(no_input_rename_diagnostic(decorator.span));
    }

    fn check_input_signal(
        &self,
        prop: &oxc_ast::ast::PropertyDefinition<'_>,
        ctx: &LintContext<'_>,
    ) {
        let Some(value) = &prop.value else {
            return;
        };

        let Expression::CallExpression(call) = value else {
            return;
        };

        let Expression::Identifier(callee) = &call.callee else {
            return;
        };

        if callee.name.as_str() != "input" {
            return;
        }

        // Verify it's from @angular/core
        if !is_angular_core_import(callee.as_ref(), ctx) {
            return;
        }

        // Check for alias in options object
        for arg in &call.arguments {
            if let oxc_ast::ast::Argument::ObjectExpression(obj) = arg
                && let Some(alias) = get_alias_from_object(obj) {
                    let property_name = prop.key.static_name();

                    // Allow if alias matches property name
                    if property_name.as_deref() == Some(alias) {
                        continue;
                    }

                    // Allow if alias is in allowed names
                    if self.allowed_names.iter().any(|name| name == alias) {
                        continue;
                    }

                    ctx.diagnostic(no_input_rename_diagnostic(prop.span));
                    return;
                }
        }
    }

    fn get_property_name_from_decorator(
        &self,
        node: &crate::AstNode<'_>,
        ctx: &LintContext<'_>,
    ) -> Option<String> {
        // Find the parent property definition
        for ancestor in ctx.nodes().ancestors(node.id()) {
            if let AstKind::PropertyDefinition(prop) = ancestor.kind() {
                return prop.key.static_name().map(|s| s.to_string());
            }
        }
        None
    }
}

fn get_alias_from_object<'a>(obj: &'a oxc_ast::ast::ObjectExpression<'a>) -> Option<&'a str> {
    use oxc_ast::ast::{ObjectPropertyKind, PropertyKey};

    for property in &obj.properties {
        if let ObjectPropertyKind::ObjectProperty(prop) = property
            && let PropertyKey::StaticIdentifier(ident) = &prop.key
                && ident.name.as_str() == "alias"
                    && let Expression::StringLiteral(lit) = &prop.value {
                        return Some(lit.value.as_str());
                    }
    }
    None
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // No alias
        r"
        import { Component, Input } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {
            @Input() name: string;
        }
        ",
        // Alias matches property name (redundant but allowed)
        r"
        import { Component, Input } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {
            @Input('name') name: string;
        }
        ",
        // Multiple inputs without alias
        r"
        import { Component, Input } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {
            @Input() firstName: string;
            @Input() lastName: string;
        }
        ",
        // input() signal without alias
        r"
        import { Component, input } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {
            name = input<string>();
        }
        ",
        // input() signal with matching alias
        r"
        import { Component, input } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {
            name = input<string>({ alias: 'name' });
        }
        ",
        // Non-Angular Input
        r"
        import { Input } from 'other-lib';
        class TestComponent {
            @Input('alias') name: string;
        }
        ",
    ];

    let fail = vec![
        // Aliased input with string
        r"
        import { Component, Input } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {
            @Input('label') name: string;
        }
        ",
        // Aliased input with object
        r"
        import { Component, Input } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {
            @Input({ alias: 'label' }) name: string;
        }
        ",
        // input() signal with alias
        r"
        import { Component, input } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {
            name = input<string>({ alias: 'label' });
        }
        ",
        // Multiple aliased inputs
        r"
        import { Component, Input } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {
            @Input('firstName') name: string;
            @Input('userAge') age: number;
        }
        ",
    ];

    Tester::new(NoInputRename::NAME, NoInputRename::PLUGIN, pass, fail).test_and_snapshot();
}
