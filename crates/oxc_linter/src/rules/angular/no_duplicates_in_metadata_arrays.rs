use oxc_ast::AstKind;
use rustc_hash::FxHashSet;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    AstNode,
    context::LintContext,
    rule::Rule,
    utils::{
        get_component_metadata, get_decorator_identifier, get_decorator_name,
        is_angular_core_import,
    },
};

fn no_duplicates_in_metadata_arrays_diagnostic(
    span: Span,
    name: &str,
    property: &str,
) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Duplicate `{name}` in `{property}` metadata array"))
        .with_help("Remove the duplicate entry from the metadata array.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoDuplicatesInMetadataArrays;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows duplicate entries in Angular decorator metadata arrays.
    ///
    /// ### Why is this bad?
    ///
    /// Duplicate entries in metadata arrays like `imports`, `declarations`, `exports`,
    /// `providers`, `schemas`, and `bootstrap` are unnecessary and can indicate:
    /// - Copy-paste errors
    /// - Misunderstanding of how Angular module configuration works
    /// - Potential maintenance issues
    ///
    /// Duplicates don't cause runtime errors in most cases, but they add clutter
    /// and can make the code harder to maintain.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```typescript
    /// import { NgModule } from '@angular/core';
    /// import { CommonModule } from '@angular/common';
    /// import { MyComponent } from './my.component';
    ///
    /// @NgModule({
    ///   imports: [CommonModule, CommonModule], // Duplicate
    ///   declarations: [MyComponent, MyComponent] // Duplicate
    /// })
    /// export class MyModule {}
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```typescript
    /// import { NgModule } from '@angular/core';
    /// import { CommonModule } from '@angular/common';
    /// import { MyComponent } from './my.component';
    ///
    /// @NgModule({
    ///   imports: [CommonModule],
    ///   declarations: [MyComponent]
    /// })
    /// export class MyModule {}
    /// ```
    NoDuplicatesInMetadataArrays,
    angular,
    style,
    pending
);

/// Metadata properties that are arrays and should be checked for duplicates
const ARRAY_PROPERTIES: [&str; 7] =
    ["imports", "declarations", "exports", "providers", "schemas", "bootstrap", "hostDirectives"];

impl Rule for NoDuplicatesInMetadataArrays {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::Decorator(decorator) = node.kind() else {
            return;
        };

        // Only check @NgModule, @Component, and @Directive decorators
        let Some(decorator_name) = get_decorator_name(decorator) else {
            return;
        };

        if !matches!(decorator_name, "NgModule" | "Component" | "Directive") {
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

        // Check each array property for duplicates
        for prop in &metadata.properties {
            if let oxc_ast::ast::ObjectPropertyKind::ObjectProperty(obj_prop) = prop {
                let Some(prop_name) = get_property_key_name(&obj_prop.key) else {
                    continue;
                };

                if !ARRAY_PROPERTIES.contains(&prop_name) {
                    continue;
                }

                // Check if the value is an array
                if let oxc_ast::ast::Expression::ArrayExpression(array) = &obj_prop.value {
                    check_array_for_duplicates(array, prop_name, ctx);
                }
            }
        }
    }
}

fn get_property_key_name<'a>(key: &'a oxc_ast::ast::PropertyKey<'a>) -> Option<&'a str> {
    match key {
        oxc_ast::ast::PropertyKey::StaticIdentifier(ident) => Some(ident.name.as_str()),
        oxc_ast::ast::PropertyKey::StringLiteral(lit) => Some(lit.value.as_str()),
        _ => None,
    }
}

fn check_array_for_duplicates(
    array: &oxc_ast::ast::ArrayExpression<'_>,
    property_name: &str,
    ctx: &LintContext<'_>,
) {
    let mut seen: FxHashSet<String> = FxHashSet::default();

    for element in &array.elements {
        let Some(expr) = element.as_expression() else {
            continue;
        };

        // Get the identifier name from the expression
        let Some(name) = get_expression_name(expr) else {
            continue;
        };

        if seen.contains(&name) {
            // Find the span of this duplicate
            let span = get_expression_span(expr);
            ctx.diagnostic(no_duplicates_in_metadata_arrays_diagnostic(span, &name, property_name));
        } else {
            seen.insert(name);
        }
    }
}

fn get_expression_name(expr: &oxc_ast::ast::Expression<'_>) -> Option<String> {
    match expr {
        oxc_ast::ast::Expression::Identifier(ident) => Some(ident.name.to_string()),
        oxc_ast::ast::Expression::CallExpression(call) => {
            // For cases like forwardRef(() => Component)
            if let oxc_ast::ast::Expression::Identifier(ident) = &call.callee
                && ident.name == "forwardRef"
                    && let Some(arg) = call.arguments.first()
                        && let oxc_ast::ast::Argument::ArrowFunctionExpression(arrow) = arg {
                            // Check if arrow function has expression body with single statement
                            if arrow.expression
                                && let Some(stmt) = arrow.body.statements.first()
                                    && let oxc_ast::ast::Statement::ExpressionStatement(expr_stmt) =
                                        stmt
                                        && let oxc_ast::ast::Expression::Identifier(ret_ident) =
                                            &expr_stmt.expression
                                        {
                                            return Some(ret_ident.name.to_string());
                                        }
                        }
            None
        }
        _ => None,
    }
}

fn get_expression_span(expr: &oxc_ast::ast::Expression<'_>) -> Span {
    match expr {
        oxc_ast::ast::Expression::Identifier(ident) => ident.span,
        oxc_ast::ast::Expression::CallExpression(call) => call.span,
        _ => expr.span(),
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // No duplicates in imports
        r"
        import { NgModule } from '@angular/core';
        import { CommonModule } from '@angular/common';
        import { FormsModule } from '@angular/forms';
        @NgModule({
            imports: [CommonModule, FormsModule]
        })
        class TestModule {}
        ",
        // No duplicates in declarations
        r"
        import { NgModule } from '@angular/core';
        @NgModule({
            declarations: [ComponentA, ComponentB]
        })
        class TestModule {}
        ",
        // Empty arrays
        r"
        import { NgModule } from '@angular/core';
        @NgModule({
            imports: [],
            declarations: []
        })
        class TestModule {}
        ",
        // Single items
        r"
        import { NgModule } from '@angular/core';
        @NgModule({
            imports: [CommonModule],
            declarations: [MyComponent]
        })
        class TestModule {}
        ",
        // Non-Angular decorator
        r"
        import { NgModule } from 'other-library';
        @NgModule({
            imports: [CommonModule, CommonModule]
        })
        class TestModule {}
        ",
    ];

    let fail = vec![
        // Duplicate in imports
        r"
        import { NgModule } from '@angular/core';
        import { CommonModule } from '@angular/common';
        @NgModule({
            imports: [CommonModule, CommonModule]
        })
        class TestModule {}
        ",
        // Duplicate in declarations
        r"
        import { NgModule } from '@angular/core';
        @NgModule({
            declarations: [MyComponent, MyComponent]
        })
        class TestModule {}
        ",
        // Duplicate in providers
        r"
        import { NgModule } from '@angular/core';
        @NgModule({
            providers: [MyService, MyService]
        })
        class TestModule {}
        ",
        // Duplicate in exports
        r"
        import { NgModule } from '@angular/core';
        @NgModule({
            exports: [MyComponent, MyComponent]
        })
        class TestModule {}
        ",
        // Multiple duplicates in same array
        r"
        import { NgModule } from '@angular/core';
        @NgModule({
            imports: [ModuleA, ModuleA, ModuleA]
        })
        class TestModule {}
        ",
        // Duplicates in Component imports
        r"
        import { Component } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: '',
            imports: [CommonModule, CommonModule]
        })
        class TestComponent {}
        ",
    ];

    Tester::new(
        NoDuplicatesInMetadataArrays::NAME,
        NoDuplicatesInMetadataArrays::PLUGIN,
        pass,
        fail,
    )
    .test_and_snapshot();
}
