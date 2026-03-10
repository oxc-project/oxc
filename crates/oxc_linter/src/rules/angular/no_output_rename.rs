use oxc_ast::{AstKind, ast::Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    AstNode,
    context::LintContext,
    rule::Rule,
    utils::{
        get_decorator_call, get_decorator_identifier, get_decorator_name, is_angular_core_import,
    },
};

fn no_output_rename_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Output bindings should not be aliased")
        .with_help(
            "Avoid aliasing outputs as it can lead to confusion. Use the original property name \
            or rename the property if the alias is more appropriate.",
        )
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoOutputRename;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows aliasing output bindings (renaming outputs with a different public name).
    ///
    /// ### Why is this bad?
    ///
    /// Two names for the same property (one private, one public) is confusing.
    /// It requires developers to remember both names and understand the mapping.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```typescript
    /// import { Component, Output, EventEmitter } from '@angular/core';
    ///
    /// @Component({
    ///   selector: 'app-example',
    ///   template: ''
    /// })
    /// export class ExampleComponent {
    ///   @Output('valueChanged') change = new EventEmitter<string>();
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```typescript
    /// import { Component, Output, EventEmitter } from '@angular/core';
    ///
    /// @Component({
    ///   selector: 'app-example',
    ///   template: ''
    /// })
    /// export class ExampleComponent {
    ///   @Output() valueChange = new EventEmitter<string>();
    /// }
    /// ```
    NoOutputRename,
    angular,
    correctness,
    pending
);

impl Rule for NoOutputRename {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        // Check for @Output decorator
        if let AstKind::Decorator(decorator) = node.kind() {
            self.check_output_decorator(decorator, node, ctx);
            return;
        }

        // Check for output() signal function
        if let AstKind::PropertyDefinition(prop) = node.kind() {
            self.check_output_signal(prop, ctx);
        }
    }
}

impl NoOutputRename {
    fn check_output_decorator(
        &self,
        decorator: &oxc_ast::ast::Decorator<'_>,
        node: &AstNode<'_>,
        ctx: &LintContext<'_>,
    ) {
        let Some(decorator_name) = get_decorator_name(decorator) else {
            return;
        };

        if decorator_name != "Output" {
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
            oxc_ast::ast::Argument::ObjectExpression(obj) => get_alias_from_object(obj),
            _ => None,
        };

        let Some(alias) = alias else {
            return;
        };

        // Get the property name from parent
        let property_name = self.get_property_name_from_decorator(node, ctx);

        // Allow if alias matches property name
        if property_name.as_deref() == Some(alias) {
            return;
        }

        ctx.diagnostic(no_output_rename_diagnostic(decorator.span));
    }

    fn check_output_signal(
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

        if callee.name.as_str() != "output" {
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

                    ctx.diagnostic(no_output_rename_diagnostic(prop.span));
                    return;
                }
        }
    }

    fn get_property_name_from_decorator(
        &self,
        node: &crate::AstNode<'_>,
        ctx: &LintContext<'_>,
    ) -> Option<String> {
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
        import { Component, Output, EventEmitter } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {
            @Output() change = new EventEmitter<string>();
        }
        ",
        // Alias matches property name
        r"
        import { Component, Output, EventEmitter } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {
            @Output('change') change = new EventEmitter<string>();
        }
        ",
        // output() signal without alias
        r"
        import { Component, output } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {
            change = output<string>();
        }
        ",
        // output() signal with matching alias
        r"
        import { Component, output } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {
            change = output<string>({ alias: 'change' });
        }
        ",
        // Non-Angular Output
        r"
        import { Output, EventEmitter } from 'other-lib';
        class TestComponent {
            @Output('alias') change = new EventEmitter<string>();
        }
        ",
    ];

    let fail = vec![
        // Aliased output with string
        r"
        import { Component, Output, EventEmitter } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {
            @Output('valueChanged') change = new EventEmitter<string>();
        }
        ",
        // Aliased output with object
        r"
        import { Component, Output, EventEmitter } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {
            @Output({ alias: 'valueChanged' }) change = new EventEmitter<string>();
        }
        ",
        // output() signal with alias
        r"
        import { Component, output } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {
            change = output<string>({ alias: 'valueChanged' });
        }
        ",
    ];

    Tester::new(NoOutputRename::NAME, NoOutputRename::PLUGIN, pass, fail).test_and_snapshot();
}
