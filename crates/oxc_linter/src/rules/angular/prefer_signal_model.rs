use std::collections::BTreeMap;

use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    AstNode,
    context::LintContext,
    rule::Rule,
    utils::{AngularDecoratorType, get_class_angular_decorator},
};

fn prefer_signal_model_diagnostic(span: Span, input_name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "Input `{input_name}` and output `{input_name}Change` can be replaced with `model()`"
    ))
    .with_help(
        "Use the `model()` function instead of separate `input()` and `output()` for two-way binding patterns. \
        Example: `value = model<string>();` replaces both `value = input<string>()` and `valueChange = output<string>()`.",
    )
    .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferSignalModel;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Suggests using `model()` instead of separate `input()` and `output()` for two-way binding patterns.
    ///
    /// ### Why is this bad?
    ///
    /// When you have an input and a corresponding output with the naming pattern `propName` and
    /// `propNameChange`, this is typically used for two-way binding (`[(propName)]`). Angular's
    /// `model()` function (introduced in Angular 17.2) provides a cleaner way to express this:
    ///
    /// - Reduces boilerplate by combining input and output into a single declaration
    /// - Makes the two-way binding intent explicit
    /// - Provides a writable signal that automatically emits on changes
    /// - Better type inference
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```typescript
    /// import { Component, input, output } from '@angular/core';
    ///
    /// @Component({
    ///   selector: 'app-example',
    ///   template: ''
    /// })
    /// export class ExampleComponent {
    ///   value = input<string>();
    ///   valueChange = output<string>();
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```typescript
    /// import { Component, model } from '@angular/core';
    ///
    /// @Component({
    ///   selector: 'app-example',
    ///   template: ''
    /// })
    /// export class ExampleComponent {
    ///   value = model<string>();
    /// }
    /// ```
    PreferSignalModel,
    angular,
    pedantic,
    pending
);

impl Rule for PreferSignalModel {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::Class(class) = node.kind() else {
            return;
        };

        // Check if the class has a @Component or @Directive decorator
        let Some((decorator_type, _)) = get_class_angular_decorator(class, ctx) else {
            return;
        };

        if !matches!(
            decorator_type,
            AngularDecoratorType::Component | AngularDecoratorType::Directive
        ) {
            return;
        }

        // Collect all input() and output() signal calls
        let mut inputs: BTreeMap<String, Span> = BTreeMap::new();
        let mut outputs: BTreeMap<String, Span> = BTreeMap::new();

        for element in &class.body.body {
            if let oxc_ast::ast::ClassElement::PropertyDefinition(prop) = element {
                let Some(prop_name) = get_property_name(&prop.key) else {
                    continue;
                };

                let Some(value) = &prop.value else {
                    continue;
                };

                // Check if it's a call expression
                let oxc_ast::ast::Expression::CallExpression(call) = value else {
                    continue;
                };

                // Get the callee name
                let callee_name = match &call.callee {
                    oxc_ast::ast::Expression::Identifier(ident) => ident.name.as_str(),
                    _ => continue,
                };

                // Check if it's input() or output()
                match callee_name {
                    "input" => {
                        // Verify it's from @angular/core
                        if let Some(ident) = call.callee.get_identifier_reference()
                            && is_angular_core_import_manual(ident, ctx) {
                                inputs.insert(prop_name.to_string(), prop.span);
                            }
                    }
                    "output" => {
                        // Verify it's from @angular/core
                        if let Some(ident) = call.callee.get_identifier_reference()
                            && is_angular_core_import_manual(ident, ctx) {
                                outputs.insert(prop_name.to_string(), prop.span);
                            }
                    }
                    _ => {}
                }
            }
        }

        // Find matching pairs: inputName + inputNameChange
        for (input_name, input_span) in &inputs {
            let expected_output_name = format!("{input_name}Change");
            if outputs.contains_key(&expected_output_name) {
                ctx.diagnostic(prefer_signal_model_diagnostic(*input_span, input_name));
            }
        }
    }
}

fn get_property_name<'a>(key: &'a oxc_ast::ast::PropertyKey<'a>) -> Option<&'a str> {
    match key {
        oxc_ast::ast::PropertyKey::StaticIdentifier(ident) => Some(ident.name.as_str()),
        _ => None,
    }
}

fn is_angular_core_import_manual(
    ident: &oxc_ast::ast::IdentifierReference<'_>,
    ctx: &LintContext<'_>,
) -> bool {
    let reference = ctx.scoping().get_reference(ident.reference_id());
    let Some(symbol_id) = reference.symbol_id() else {
        return false;
    };

    if !ctx.scoping().symbol_flags(symbol_id).is_import() {
        return false;
    }

    let declaration_id = ctx.scoping().symbol_declaration(symbol_id);
    let AstKind::ImportDeclaration(import_decl) = ctx.nodes().parent_kind(declaration_id) else {
        return false;
    };

    import_decl.source.value.as_str() == "@angular/core"
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // Using model()
        r"
        import { Component, model } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {
            value = model<string>();
        }
        ",
        // Only input, no corresponding output
        r"
        import { Component, input } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {
            value = input<string>();
        }
        ",
        // Only output, no corresponding input
        r"
        import { Component, output } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {
            valueChange = output<string>();
        }
        ",
        // Different naming (not a Change suffix)
        r"
        import { Component, input, output } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {
            value = input<string>();
            valueUpdated = output<string>();
        }
        ",
        // Non-Angular class
        r"
        import { input, output } from 'other-library';
        class TestClass {
            value = input<string>();
            valueChange = output<string>();
        }
        ",
    ];

    let fail = vec![
        // Classic two-way binding pattern
        r"
        import { Component, input, output } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {
            value = input<string>();
            valueChange = output<string>();
        }
        ",
        // Multiple two-way binding patterns
        r"
        import { Component, input, output } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {
            count = input<number>();
            countChange = output<number>();
            name = input<string>();
            nameChange = output<string>();
        }
        ",
        // In Directive
        r"
        import { Directive, input, output } from '@angular/core';
        @Directive({
            selector: '[appTest]'
        })
        class TestDirective {
            selected = input<boolean>();
            selectedChange = output<boolean>();
        }
        ",
    ];

    Tester::new(PreferSignalModel::NAME, PreferSignalModel::PLUGIN, pass, fail).test_and_snapshot();
}
