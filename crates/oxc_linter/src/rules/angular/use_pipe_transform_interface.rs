use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    AstNode,
    context::LintContext,
    rule::Rule,
    utils::{
        class_implements_interface, get_decorator_identifier, get_decorator_name,
        is_angular_core_import,
    },
};

fn use_pipe_transform_interface_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Pipes should implement `PipeTransform` interface")
        .with_help(
            "Add `implements PipeTransform` to the class declaration and import \
            `PipeTransform` from '@angular/core'. This provides type safety for the \
            `transform` method.",
        )
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct UsePipeTransformInterface;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Ensures that classes decorated with `@Pipe` implement the `PipeTransform` interface.
    ///
    /// ### Why is this bad?
    ///
    /// Implementing the `PipeTransform` interface:
    /// - Provides type safety for the `transform` method signature
    /// - Improves IDE support with better autocompletion
    /// - Makes the code more self-documenting
    /// - Helps catch errors at compile time rather than runtime
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```typescript
    /// import { Pipe } from '@angular/core';
    ///
    /// @Pipe({
    ///   name: 'myPipe'
    /// })
    /// export class MyPipe {
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
    /// })
    /// export class MyPipe implements PipeTransform {
    ///   transform(value: any): any {
    ///     return value;
    ///   }
    /// }
    /// ```
    UsePipeTransformInterface,
    angular,
    correctness,
    pending
);

impl Rule for UsePipeTransformInterface {
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

        // Find the parent class
        let Some(class) = get_parent_class_from_decorator(node, ctx) else {
            return;
        };

        // Check if the class implements PipeTransform
        if class_implements_interface(class, "PipeTransform") {
            return;
        }

        ctx.diagnostic(use_pipe_transform_interface_diagnostic(decorator.span));
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
        // Implements PipeTransform
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
        // Implements multiple interfaces including PipeTransform
        r"
        import { Pipe, PipeTransform, OnDestroy } from '@angular/core';
        @Pipe({
            name: 'myPipe'
        })
        class MyPipe implements PipeTransform, OnDestroy {
            transform(value: any): any {
                return value;
            }
            ngOnDestroy() {}
        }
        ",
        // Standalone pipe with interface
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
            name: 'myPipe'
        })
        class MyPipe {
            transform(value: any): any {
                return value;
            }
        }
        ",
    ];

    let fail = vec![
        // Missing PipeTransform interface
        r"
        import { Pipe } from '@angular/core';
        @Pipe({
            name: 'myPipe'
        })
        class MyPipe {
            transform(value: any): any {
                return value;
            }
        }
        ",
        // Standalone pipe missing interface
        r"
        import { Pipe } from '@angular/core';
        @Pipe({
            name: 'myPipe',
            standalone: true
        })
        class MyPipe {
            transform(value: any): any {
                return value;
            }
        }
        ",
        // Implements other interface but not PipeTransform
        r"
        import { Pipe, OnDestroy } from '@angular/core';
        @Pipe({
            name: 'myPipe'
        })
        class MyPipe implements OnDestroy {
            transform(value: any): any {
                return value;
            }
            ngOnDestroy() {}
        }
        ",
    ];

    Tester::new(UsePipeTransformInterface::NAME, UsePipeTransformInterface::PLUGIN, pass, fail)
        .test_and_snapshot();
}
