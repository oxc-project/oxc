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
        is_native_event_name,
    },
};

fn no_output_native_diagnostic(span: Span, event_name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "Output bindings, including aliases, should not be named as native DOM events: `{event_name}`"
    ))
    .with_help(
        "Using native event names like 'click', 'change', 'blur' can cause confusion and \
        unexpected behavior. Rename your output to something more descriptive like 'itemClick' \
        or 'valueChange'.",
    )
    .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoOutputNative;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows naming outputs with the same names as native DOM events.
    ///
    /// ### Why is this bad?
    ///
    /// Naming an output the same as a native DOM event (e.g., `click`, `change`, `blur`)
    /// can lead to confusion about whether the event is a native DOM event or a custom
    /// Angular output. It can also cause unexpected behavior when the event bubbles or
    /// is captured at a higher level in the DOM.
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
    ///   @Output() click = new EventEmitter<void>(); // Same as native DOM event
    ///   @Output() change = new EventEmitter<string>(); // Same as native DOM event
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
    ///   @Output() itemClick = new EventEmitter<void>();
    ///   @Output() valueChange = new EventEmitter<string>();
    /// }
    /// ```
    NoOutputNative,
    angular,
    correctness,
    pending
);

impl Rule for NoOutputNative {
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

        // Check if the output name is a native event name
        if is_native_event_name(&output_name) {
            ctx.diagnostic(no_output_native_diagnostic(span, &output_name));
        }
    }
}

impl NoOutputNative {
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
        // Custom output name
        r"
        import { Component, Output, EventEmitter } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {
            @Output() itemClick = new EventEmitter<void>();
        }
        ",
        // Custom output name with Change suffix
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
        // output() signal with custom name
        r"
        import { Component, output } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {
            itemSelected = output<string>();
        }
        ",
        // Non-Angular Output
        r"
        import { Output, EventEmitter } from 'other-lib';
        class TestComponent {
            @Output() click = new EventEmitter<void>();
        }
        ",
        // Not an output (Input decorator)
        r"
        import { Component, Input } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {
            @Input() click: boolean;
        }
        ",
    ];

    let fail = vec![
        // Native event name: click
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
        // Native event name: change
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
        // Native event name: blur
        r"
        import { Component, Output, EventEmitter } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {
            @Output() blur = new EventEmitter<void>();
        }
        ",
        // Native event name: focus
        r"
        import { Component, Output, EventEmitter } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {
            @Output() focus = new EventEmitter<void>();
        }
        ",
        // Aliased to native event name
        r"
        import { Component, Output, EventEmitter } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {
            @Output('click') buttonClick = new EventEmitter<void>();
        }
        ",
        // output() signal with native event name
        r"
        import { Component, output } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {
            scroll = output<void>();
        }
        ",
        // output() signal aliased to native event name
        r"
        import { Component, output } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {
            buttonPress = output<void>({ alias: 'keydown' });
        }
        ",
    ];

    Tester::new(NoOutputNative::NAME, NoOutputNative::PLUGIN, pass, fail).test_and_snapshot();
}
