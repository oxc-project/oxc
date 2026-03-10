use oxc_ast::{AstKind, ast::Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    AstNode,
    context::LintContext,
    rule::Rule,
    utils::{
        get_component_metadata, get_decorator_identifier, get_decorator_name,
        get_metadata_property, is_angular_core_import,
    },
};

fn no_pipe_impure_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Impure pipes should be avoided")
        .with_help(
            "Impure pipes (`pure: false`) are invoked on each change-detection cycle, which can \
            significantly impact performance. Consider redesigning your pipe to be pure, or use \
            a different approach such as a method in the component.",
        )
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoPipeImpure;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows the use of impure pipes (`pure: false`) in Angular.
    ///
    /// ### Why is this bad?
    ///
    /// Impure pipes are invoked on every change-detection cycle, regardless of whether
    /// their input values have changed. This can cause:
    /// - Significant performance degradation
    /// - Unexpected behavior
    /// - Difficulty in debugging
    ///
    /// Pure pipes (the default) are only invoked when their input values change,
    /// which is much more efficient.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```typescript
    /// import { Pipe, PipeTransform } from '@angular/core';
    ///
    /// @Pipe({
    ///   name: 'myPipe',
    ///   pure: false  // Impure pipe
    /// })
    /// export class MyPipe implements PipeTransform {
    ///   transform(value: any): any {
    ///     return value;
    ///   }
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```typescript
    /// import { Pipe, PipeTransform } from '@angular/core';
    ///
    /// @Pipe({
    ///   name: 'myPipe'
    ///   // pure: true is the default
    /// })
    /// export class MyPipe implements PipeTransform {
    ///   transform(value: any): any {
    ///     return value;
    ///   }
    /// }
    /// ```
    NoPipeImpure,
    angular,
    pedantic,
    pending
);

impl Rule for NoPipeImpure {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::Decorator(decorator) = node.kind() else {
            return;
        };

        // Only check @Pipe decorator
        let Some(decorator_name) = get_decorator_name(decorator) else {
            return;
        };

        if decorator_name != "Pipe" {
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

        // Look for the pure property
        let Some(pure_value) = get_metadata_property(metadata, "pure") else {
            return;
        };

        // Check if pure is set to false
        let is_impure = match pure_value {
            Expression::BooleanLiteral(lit) => !lit.value,
            Expression::UnaryExpression(unary) => {
                // Handle !true case
                if unary.operator == oxc_ast::ast::UnaryOperator::LogicalNot {
                    if let Expression::BooleanLiteral(lit) = &unary.argument {
                        lit.value
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            _ => false,
        };

        if is_impure {
            let span = find_property_span(metadata, "pure").unwrap_or(decorator.span);
            ctx.diagnostic(no_pipe_impure_diagnostic(span));
        }
    }
}

fn find_property_span(obj: &oxc_ast::ast::ObjectExpression<'_>, key: &str) -> Option<Span> {
    use oxc_ast::ast::{ObjectPropertyKind, PropertyKey};
    use oxc_span::GetSpan;

    for property in &obj.properties {
        if let ObjectPropertyKind::ObjectProperty(prop) = property
            && let PropertyKey::StaticIdentifier(ident) = &prop.key
                && ident.name.as_str() == key {
                    return Some(prop.span());
                }
    }
    None
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // Pure pipe (default)
        r"
        import { Pipe, PipeTransform } from '@angular/core';
        @Pipe({
            name: 'myPipe'
        })
        class MyPipe implements PipeTransform {
            transform(value: any): any {
                return value;
            }
        }
        ",
        // Explicit pure: true
        r"
        import { Pipe, PipeTransform } from '@angular/core';
        @Pipe({
            name: 'myPipe',
            pure: true
        })
        class MyPipe implements PipeTransform {
            transform(value: any): any {
                return value;
            }
        }
        ",
        // Standalone pure pipe
        r"
        import { Pipe, PipeTransform } from '@angular/core';
        @Pipe({
            name: 'myPipe',
            standalone: true
        })
        class MyPipe implements PipeTransform {
            transform(value: any): any {
                return value;
            }
        }
        ",
        // Non-Angular Pipe
        r"
        import { Pipe } from 'other-lib';
        @Pipe({
            name: 'myPipe',
            pure: false
        })
        class MyPipe {}
        ",
    ];

    let fail = vec![
        // Impure pipe
        r"
        import { Pipe, PipeTransform } from '@angular/core';
        @Pipe({
            name: 'myPipe',
            pure: false
        })
        class MyPipe implements PipeTransform {
            transform(value: any): any {
                return value;
            }
        }
        ",
        // Impure standalone pipe
        r"
        import { Pipe, PipeTransform } from '@angular/core';
        @Pipe({
            name: 'myPipe',
            standalone: true,
            pure: false
        })
        class MyPipe implements PipeTransform {
            transform(value: any): any {
                return value;
            }
        }
        ",
        // Impure pipe with !true
        r"
        import { Pipe, PipeTransform } from '@angular/core';
        @Pipe({
            name: 'myPipe',
            pure: !true
        })
        class MyPipe implements PipeTransform {
            transform(value: any): any {
                return value;
            }
        }
        ",
    ];

    Tester::new(NoPipeImpure::NAME, NoPipeImpure::PLUGIN, pass, fail).test_and_snapshot();
}
