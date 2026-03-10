use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_implicit_take_until_destroyed_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "`takeUntilDestroyed()` called without `DestroyRef` argument outside injection context",
    )
    .with_help(
        "When using `takeUntilDestroyed()` outside of an injection context (like inside `ngOnInit`, \
        event handlers, or subscriptions), you must pass a `DestroyRef` explicitly. \
        Example: `takeUntilDestroyed(this.destroyRef)` where `destroyRef = inject(DestroyRef)`.",
    )
    .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoImplicitTakeUntilDestroyed;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Requires that `takeUntilDestroyed()` is called with an explicit `DestroyRef` when used
    /// outside of an injection context.
    ///
    /// ### Why is this bad?
    ///
    /// The `takeUntilDestroyed()` operator from `@angular/core/rxjs-interop` can be called
    /// without arguments only within an injection context (like constructors or field initializers).
    /// When called outside an injection context (in lifecycle methods, callbacks, or subscriptions),
    /// it will throw a runtime error.
    ///
    /// Common problematic patterns:
    /// - Calling `takeUntilDestroyed()` in `ngOnInit`
    /// - Calling it in event handlers or callbacks
    /// - Calling it inside `setTimeout`, `setInterval`, or promise callbacks
    ///
    /// The solution is to inject `DestroyRef` and pass it explicitly:
    /// ```typescript
    /// private destroyRef = inject(DestroyRef);
    ///
    /// ngOnInit() {
    ///   this.data$.pipe(takeUntilDestroyed(this.destroyRef)).subscribe();
    /// }
    /// ```
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```typescript
    /// import { Component, inject } from '@angular/core';
    /// import { takeUntilDestroyed } from '@angular/core/rxjs-interop';
    ///
    /// @Component({ selector: 'app-example', template: '' })
    /// export class ExampleComponent {
    ///   ngOnInit() {
    ///     this.data$.pipe(takeUntilDestroyed()).subscribe(); // Error!
    ///   }
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```typescript
    /// import { Component, inject, DestroyRef } from '@angular/core';
    /// import { takeUntilDestroyed } from '@angular/core/rxjs-interop';
    ///
    /// @Component({ selector: 'app-example', template: '' })
    /// export class ExampleComponent {
    ///   private destroyRef = inject(DestroyRef);
    ///
    ///   ngOnInit() {
    ///     this.data$.pipe(takeUntilDestroyed(this.destroyRef)).subscribe();
    ///   }
    /// }
    /// ```
    NoImplicitTakeUntilDestroyed,
    angular,
    correctness,
    pending
);

impl Rule for NoImplicitTakeUntilDestroyed {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        // Check if this is a call to takeUntilDestroyed
        let callee_name = match &call_expr.callee {
            oxc_ast::ast::Expression::Identifier(ident) => ident.name.as_str(),
            _ => return,
        };

        if callee_name != "takeUntilDestroyed" {
            return;
        }

        // Verify it's imported from @angular/core/rxjs-interop
        let Some(ident) = call_expr.callee.get_identifier_reference() else {
            return;
        };

        if !is_rxjs_interop_import(ident, ctx) {
            return;
        }

        // If it has arguments, it's explicit - no problem
        if !call_expr.arguments.is_empty() {
            return;
        }

        // Check if we're in an injection context
        if !is_in_injection_context(node, ctx) {
            ctx.diagnostic(no_implicit_take_until_destroyed_diagnostic(call_expr.span));
        }
    }
}

fn is_rxjs_interop_import(
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

    let source = import_decl.source.value.as_str();
    source == "@angular/core/rxjs-interop" || source == "@angular/core"
}

fn is_in_injection_context(node: &AstNode<'_>, ctx: &LintContext<'_>) -> bool {
    // Walk up the tree to find the containing context
    for ancestor in ctx.nodes().ancestors(node.id()) {
        match ancestor.kind() {
            // Constructors are injection contexts
            AstKind::MethodDefinition(method) => {
                if method.kind.is_constructor() {
                    return true;
                }
                // Regular methods are NOT injection contexts
                return false;
            }
            // Field initializers are injection contexts (class property definitions at class level)
            AstKind::PropertyDefinition(_) => {
                // Check if this is a direct child of a class body
                return true;
            }
            // Static blocks are NOT injection contexts
            AstKind::StaticBlock(_) => {
                return false;
            }
            // Arrow functions or regular functions might not be injection contexts
            // depending on where they are defined
            AstKind::ArrowFunctionExpression(_) | AstKind::Function(_) => {
                // Keep walking up - the function might be in a valid context
                continue;
            }
            AstKind::Class(_) => {
                // If we reach the class without hitting a method, we're in a field initializer
                return true;
            }
            _ => {}
        }
    }

    false
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // With DestroyRef argument
        r"
        import { takeUntilDestroyed } from '@angular/core/rxjs-interop';
        import { DestroyRef, inject } from '@angular/core';
        class TestComponent {
            destroyRef = inject(DestroyRef);
            ngOnInit() {
                this.data$.pipe(takeUntilDestroyed(this.destroyRef)).subscribe();
            }
        }
        ",
        // In constructor (injection context)
        r"
        import { takeUntilDestroyed } from '@angular/core/rxjs-interop';
        class TestComponent {
            constructor() {
                this.data$.pipe(takeUntilDestroyed()).subscribe();
            }
        }
        ",
        // In field initializer (injection context)
        r"
        import { takeUntilDestroyed } from '@angular/core/rxjs-interop';
        class TestComponent {
            data = this.source$.pipe(takeUntilDestroyed());
        }
        ",
        // Non-Angular takeUntilDestroyed
        r"
        import { takeUntilDestroyed } from 'other-library';
        class TestComponent {
            ngOnInit() {
                this.data$.pipe(takeUntilDestroyed()).subscribe();
            }
        }
        ",
    ];

    let fail = vec![
        // In ngOnInit without argument
        r"
        import { takeUntilDestroyed } from '@angular/core/rxjs-interop';
        class TestComponent {
            ngOnInit() {
                this.data$.pipe(takeUntilDestroyed()).subscribe();
            }
        }
        ",
        // In ngAfterViewInit without argument
        r"
        import { takeUntilDestroyed } from '@angular/core/rxjs-interop';
        class TestComponent {
            ngAfterViewInit() {
                this.data$.pipe(takeUntilDestroyed()).subscribe();
            }
        }
        ",
        // In regular method
        r"
        import { takeUntilDestroyed } from '@angular/core/rxjs-interop';
        class TestComponent {
            loadData() {
                this.data$.pipe(takeUntilDestroyed()).subscribe();
            }
        }
        ",
        // In event handler callback
        r"
        import { takeUntilDestroyed } from '@angular/core/rxjs-interop';
        class TestComponent {
            onClick() {
                this.service.getData().pipe(takeUntilDestroyed()).subscribe();
            }
        }
        ",
    ];

    Tester::new(
        NoImplicitTakeUntilDestroyed::NAME,
        NoImplicitTakeUntilDestroyed::PLUGIN,
        pass,
        fail,
    )
    .test_and_snapshot();
}
