use oxc_ast::{AstKind, ast::Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    AstNode,
    context::LintContext,
    rule::Rule,
    utils::{
        get_decorator_call, get_decorator_identifier, get_decorator_name, has_on_prefix,
        is_angular_core_import,
    },
};

fn no_output_on_prefix_diagnostic(span: Span, name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "Output bindings, including aliases, should not be named \"on\" or prefixed with it: `{name}`"
    ))
    .with_help(
        "Remove the 'on' prefix from the output name. In Angular templates, outputs are already \
        bound with (output) syntax which implies an event. Adding 'on' is redundant: \
        (onClick)=\"handler()\" becomes confusing.",
    )
    .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoOutputOnPrefix;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows naming outputs with the "on" prefix.
    ///
    /// ### Why is this bad?
    ///
    /// In Angular templates, outputs are bound using the `(output)` syntax which already
    /// implies an event handler. Adding an "on" prefix creates redundancy:
    ///
    /// ```html
    /// <!-- Confusing: (onClick) suggests handling a click twice -->
    /// <app-button (onClick)="handleClick()"></app-button>
    ///
    /// <!-- Clear: (click) or (buttonClick) is more intuitive -->
    /// <app-button (click)="handleClick()"></app-button>
    /// ```
    ///
    /// The "on" prefix is a convention from DOM event handlers (`onclick`, `onblur`) but
    /// is not appropriate for Angular outputs.
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
    ///   @Output() onClick = new EventEmitter<void>();
    ///   @Output() onValueChange = new EventEmitter<string>();
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
    ///   @Output() click = new EventEmitter<void>();
    ///   @Output() valueChange = new EventEmitter<string>();
    /// }
    /// ```
    NoOutputOnPrefix,
    angular,
    correctness,
    pending
);

impl Rule for NoOutputOnPrefix {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::PropertyDefinition(prop) = node.kind() else {
            return;
        };

        // Check if this is an output property
        let (output_name, span) = if let Some(output_info) = self.get_output_info(prop, ctx) {
            output_info
        } else {
            return;
        };

        // Check if the output name has "on" prefix
        if has_on_prefix(&output_name) {
            ctx.diagnostic(no_output_on_prefix_diagnostic(span, &output_name));
        }
    }
}

impl NoOutputOnPrefix {
    /// Returns the effective output name and span if this is an output property
    fn get_output_info(
        &self,
        prop: &oxc_ast::ast::PropertyDefinition<'_>,
        ctx: &LintContext<'_>,
    ) -> Option<(String, Span)> {
        // Check for @Output decorator
        for decorator in &prop.decorators {
            let Some(decorator_name) = get_decorator_name(decorator) else {
                continue;
            };

            if decorator_name != "Output" {
                continue;
            }

            let Some(ident) = get_decorator_identifier(decorator) else {
                continue;
            };

            if !is_angular_core_import(ident, ctx) {
                continue;
            }

            // Get the alias or property name
            if let Some(call) = get_decorator_call(decorator)
                && let Some(first_arg) = call.arguments.first() {
                    match first_arg {
                        oxc_ast::ast::Argument::StringLiteral(lit) => {
                            return Some((lit.value.to_string(), prop.span));
                        }
                        oxc_ast::ast::Argument::ObjectExpression(obj) => {
                            if let Some(alias) = get_alias_from_object(obj) {
                                return Some((alias.to_string(), prop.span));
                            }
                        }
                        _ => {}
                    }
                }

            // Use property name if no alias
            return prop.key.static_name().map(|name| (name.to_string(), prop.span));
        }

        // Check for output() signal function
        let value = prop.value.as_ref()?;
        let Expression::CallExpression(call) = value else {
            return None;
        };

        let Expression::Identifier(callee) = &call.callee else {
            return None;
        };

        if callee.name.as_str() != "output" {
            return None;
        }

        if !is_angular_core_import(callee.as_ref(), ctx) {
            return None;
        }

        // Check for alias in options object
        for arg in &call.arguments {
            if let oxc_ast::ast::Argument::ObjectExpression(obj) = arg
                && let Some(alias) = get_alias_from_object(obj) {
                    return Some((alias.to_string(), prop.span));
                }
        }

        // Use property name
        prop.key.static_name().map(|name| (name.to_string(), prop.span))
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
        // Normal output name without on prefix
        r"
        import { Component, Output, EventEmitter } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {
            @Output() click = new EventEmitter<void>();
        }
        ",
        // Output with 'Change' suffix
        r"
        import { Component, Output, EventEmitter } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {
            @Output() valueChange = new EventEmitter<string>();
        }
        ",
        // Words that start with 'on' but are not prefixed
        r"
        import { Component, Output, EventEmitter } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {
            @Output() online = new EventEmitter<boolean>();
            @Output() one = new EventEmitter<number>();
            @Output() ongoing = new EventEmitter<void>();
        }
        ",
        // output() signal without on prefix
        r"
        import { Component, output } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {
            selected = output<string>();
        }
        ",
        // Non-Angular Output
        r"
        import { Output, EventEmitter } from 'other-lib';
        class TestComponent {
            @Output() onClick = new EventEmitter<void>();
        }
        ",
    ];

    let fail = vec![
        // Output with 'on' prefix
        r"
        import { Component, Output, EventEmitter } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {
            @Output() onClick = new EventEmitter<void>();
        }
        ",
        // Output with 'on' prefix - onChange
        r"
        import { Component, Output, EventEmitter } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {
            @Output() onChange = new EventEmitter<string>();
        }
        ",
        // Output named just 'on'
        r"
        import { Component, Output, EventEmitter } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {
            @Output() on = new EventEmitter<void>();
        }
        ",
        // Aliased to on prefix
        r"
        import { Component, Output, EventEmitter } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {
            @Output('onClick') clickHandler = new EventEmitter<void>();
        }
        ",
        // output() signal with on prefix
        r"
        import { Component, output } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {
            onSelect = output<string>();
        }
        ",
        // output() signal aliased to on prefix
        r"
        import { Component, output } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {
            selectHandler = output<string>({ alias: 'onSelect' });
        }
        ",
        // Multiple outputs with on prefix
        r"
        import { Component, Output, EventEmitter } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {
            @Output() onBlur = new EventEmitter<void>();
            @Output() onFocus = new EventEmitter<void>();
        }
        ",
    ];

    Tester::new(NoOutputOnPrefix::NAME, NoOutputOnPrefix::PLUGIN, pass, fail).test_and_snapshot();
}
