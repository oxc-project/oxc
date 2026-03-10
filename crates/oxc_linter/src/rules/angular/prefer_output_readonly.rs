use oxc_ast::{AstKind, ast::Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    AstNode,
    context::LintContext,
    rule::Rule,
    utils::{get_decorator_identifier, get_decorator_name, is_angular_core_import},
};

fn prefer_output_readonly_diagnostic(span: Span, is_signal: bool) -> OxcDiagnostic {
    let output_type = if is_signal { "output()" } else { "@Output()" };
    OxcDiagnostic::warn(format!("Prefer `readonly` modifier on {output_type} properties"))
        .with_help(
            "Add the `readonly` modifier to prevent accidental reassignment of the output. \
            Outputs should not be reassigned after initialization.",
        )
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferOutputReadonly;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces that `@Output()` properties and `output()` signals are marked as `readonly`
    /// to prevent accidental reassignment.
    ///
    /// ### Why is this bad?
    ///
    /// Reassigning an output property can lead to unexpected behavior where event listeners
    /// are no longer notified of events. The output should remain the same instance throughout
    /// the component's lifecycle.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```typescript
    /// import { Component, Output, EventEmitter, output } from '@angular/core';
    ///
    /// @Component({ selector: 'app-example', template: '' })
    /// export class ExampleComponent {
    ///   // Missing readonly modifier
    ///   @Output() clicked = new EventEmitter<void>();
    ///
    ///   // Signal-based output also needs readonly
    ///   valueChange = output<string>();
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```typescript
    /// import { Component, Output, EventEmitter, output } from '@angular/core';
    ///
    /// @Component({ selector: 'app-example', template: '' })
    /// export class ExampleComponent {
    ///   @Output() readonly clicked = new EventEmitter<void>();
    ///
    ///   readonly valueChange = output<string>();
    /// }
    /// ```
    PreferOutputReadonly,
    angular,
    pedantic,
    pending // not yet ready for production
);

impl Rule for PreferOutputReadonly {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::PropertyDefinition(prop_def) = node.kind() else {
            return;
        };

        // Skip if already readonly
        if prop_def.readonly {
            return;
        }

        // Check for @Output() decorator
        let has_output_decorator = prop_def.decorators.iter().any(|decorator| {
            let Some(name) = get_decorator_name(decorator) else {
                return false;
            };

            if name != "Output" {
                return false;
            }

            // Verify it's from @angular/core
            let Some(ident) = get_decorator_identifier(decorator) else {
                return false;
            };

            is_angular_core_import(ident, ctx)
        });

        if has_output_decorator {
            // Get the property name span for the diagnostic
            let span = prop_def.key.span();
            ctx.diagnostic(prefer_output_readonly_diagnostic(span, false));
            return;
        }

        // Check for output() signal function
        if let Some(init) = &prop_def.value
            && is_output_signal_call(init, ctx) {
                let span = prop_def.key.span();
                ctx.diagnostic(prefer_output_readonly_diagnostic(span, true));
                return;
            }

        // Check for OutputEmitterRef or OutputRef type annotations
        if let Some(type_ann) = &prop_def.type_annotation
            && is_output_type_annotation(&type_ann.type_annotation) {
                let span = prop_def.key.span();
                ctx.diagnostic(prefer_output_readonly_diagnostic(span, true));
            }
    }
}

/// Check if a type annotation is OutputEmitterRef or OutputRef
fn is_output_type_annotation(type_ann: &oxc_ast::ast::TSType<'_>) -> bool {
    match type_ann {
        oxc_ast::ast::TSType::TSTypeReference(type_ref) => {
            if let oxc_ast::ast::TSTypeName::IdentifierReference(ident) = &type_ref.type_name {
                let name = ident.name.as_str();
                return name == "OutputEmitterRef" || name == "OutputRef";
            }
            false
        }
        _ => false,
    }
}

/// Check if an expression is a call to `output()` or `outputFromObservable()` from `@angular/core`
fn is_output_signal_call(expr: &Expression<'_>, ctx: &LintContext<'_>) -> bool {
    let Expression::CallExpression(call) = expr else {
        return false;
    };

    let Expression::Identifier(ident) = &call.callee else {
        return false;
    };

    let name = ident.name.as_str();
    if name != "output" && name != "outputFromObservable" {
        return false;
    }

    is_angular_core_import(ident.as_ref(), ctx)
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // Readonly @Output
        r"
        import { Component, Output, EventEmitter } from '@angular/core';
        @Component({ selector: 'app-test', template: '' })
        class TestComponent {
            @Output() readonly clicked = new EventEmitter<void>();
        }
        ",
        // Readonly @Output with alias
        r"
        import { Component, Output, EventEmitter } from '@angular/core';
        @Component({ selector: 'app-test', template: '' })
        class TestComponent {
            @Output('click') readonly clicked = new EventEmitter<void>();
        }
        ",
        // Readonly output() signal
        r"
        import { Component, output } from '@angular/core';
        @Component({ selector: 'app-test', template: '' })
        class TestComponent {
            readonly valueChange = output<string>();
        }
        ",
        // Readonly output() signal with options
        r"
        import { Component, output } from '@angular/core';
        @Component({ selector: 'app-test', template: '' })
        class TestComponent {
            readonly valueChange = output<string>({ alias: 'change' });
        }
        ",
        // @Output from different library (should not trigger)
        r"
        import { Output } from 'some-other-lib';
        class TestComponent {
            @Output() clicked = {};
        }
        ",
        // output from different library (should not trigger)
        r"
        import { output } from 'some-other-lib';
        class TestComponent {
            valueChange = output();
        }
        ",
        // Regular property without @Output (should not trigger)
        r"
        import { Component } from '@angular/core';
        @Component({ selector: 'app-test', template: '' })
        class TestComponent {
            name = 'test';
        }
        ",
        // Input property (should not trigger - different decorator)
        r"
        import { Component, Input } from '@angular/core';
        @Component({ selector: 'app-test', template: '' })
        class TestComponent {
            @Input() name: string;
        }
        ",
        // Multiple readonly outputs
        r"
        import { Component, Output, EventEmitter, output } from '@angular/core';
        @Component({ selector: 'app-test', template: '' })
        class TestComponent {
            @Output() readonly clicked = new EventEmitter<void>();
            @Output() readonly hovered = new EventEmitter<MouseEvent>();
            readonly valueChange = output<string>();
        }
        ",
        // Directive with readonly output
        r"
        import { Directive, Output, EventEmitter } from '@angular/core';
        @Directive({ selector: '[appTest]' })
        class TestDirective {
            @Output() readonly activated = new EventEmitter<void>();
        }
        ",
        // Readonly outputFromObservable
        r"
        import { Component, outputFromObservable } from '@angular/core';
        @Component({ selector: 'app-test', template: '' })
        class TestComponent {
            readonly valueChange = outputFromObservable(this.value$);
        }
        ",
        // Readonly OutputEmitterRef type annotation
        r"
        import { Component, OutputEmitterRef } from '@angular/core';
        @Component({ selector: 'app-test', template: '' })
        class TestComponent {
            readonly clicked: OutputEmitterRef<void>;
        }
        ",
        // Readonly OutputRef type annotation
        r"
        import { Component, OutputRef } from '@angular/core';
        @Component({ selector: 'app-test', template: '' })
        class TestComponent {
            readonly valueChange: OutputRef<string>;
        }
        ",
    ];

    let fail = vec![
        // @Output without readonly
        r"
        import { Component, Output, EventEmitter } from '@angular/core';
        @Component({ selector: 'app-test', template: '' })
        class TestComponent {
            @Output() clicked = new EventEmitter<void>();
        }
        ",
        // output() without readonly
        r"
        import { Component, output } from '@angular/core';
        @Component({ selector: 'app-test', template: '' })
        class TestComponent {
            valueChange = output<string>();
        }
        ",
        // Multiple outputs without readonly
        r"
        import { Component, Output, EventEmitter, output } from '@angular/core';
        @Component({ selector: 'app-test', template: '' })
        class TestComponent {
            @Output() clicked = new EventEmitter<void>();
            @Output() hovered = new EventEmitter<MouseEvent>();
            valueChange = output<string>();
        }
        ",
        // @Output with alias but no readonly
        r"
        import { Component, Output, EventEmitter } from '@angular/core';
        @Component({ selector: 'app-test', template: '' })
        class TestComponent {
            @Output('click') clicked = new EventEmitter<void>();
        }
        ",
        // Directive with non-readonly output
        r"
        import { Directive, Output, EventEmitter } from '@angular/core';
        @Directive({ selector: '[appTest]' })
        class TestDirective {
            @Output() activated = new EventEmitter<void>();
        }
        ",
        // outputFromObservable without readonly
        r"
        import { Component, outputFromObservable } from '@angular/core';
        @Component({ selector: 'app-test', template: '' })
        class TestComponent {
            valueChange = outputFromObservable(this.value$);
        }
        ",
        // OutputEmitterRef type annotation without readonly
        r"
        import { Component, OutputEmitterRef } from '@angular/core';
        @Component({ selector: 'app-test', template: '' })
        class TestComponent {
            clicked: OutputEmitterRef<void>;
        }
        ",
        // OutputRef type annotation without readonly
        r"
        import { Component, OutputRef } from '@angular/core';
        @Component({ selector: 'app-test', template: '' })
        class TestComponent {
            valueChange: OutputRef<string>;
        }
        ",
    ];

    Tester::new(PreferOutputReadonly::NAME, PreferOutputReadonly::PLUGIN, pass, fail)
        .test_and_snapshot();
}
