use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    AstNode,
    context::LintContext,
    rule::Rule,
    utils::{get_class_angular_decorator, is_lifecycle_method},
};

fn require_lifecycle_on_prototype_diagnostic(span: Span, method_name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "Lifecycle method `{method_name}` should be defined on the class prototype, not as a property"
    ))
    .with_help(
        "Define lifecycle methods as regular methods on the class, not as property assignments. \
        Angular's change detection and lifecycle hooks work with prototype methods, not instance properties.",
    )
    .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct RequireLifecycleOnPrototype;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Ensures lifecycle methods are declared on the class prototype rather than as instance properties.
    ///
    /// ### Why is this bad?
    ///
    /// Lifecycle methods defined as arrow functions or property assignments create a new function
    /// for each component instance. This has several drawbacks:
    /// - Higher memory usage (each instance has its own copy)
    /// - Cannot be overridden in subclasses
    /// - May not work correctly with Angular's change detection in some scenarios
    /// - Inconsistent with Angular's expected method signature pattern
    ///
    /// Regular prototype methods are shared across all instances and work correctly with
    /// Angular's lifecycle system.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```typescript
    /// import { Component } from '@angular/core';
    ///
    /// @Component({
    ///   selector: 'app-example',
    ///   template: ''
    /// })
    /// export class ExampleComponent {
    ///   ngOnInit = () => {
    ///     console.log('initialized');
    ///   };
    ///
    ///   ngOnDestroy = function() {
    ///     console.log('destroyed');
    ///   };
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```typescript
    /// import { Component } from '@angular/core';
    ///
    /// @Component({
    ///   selector: 'app-example',
    ///   template: ''
    /// })
    /// export class ExampleComponent {
    ///   ngOnInit() {
    ///     console.log('initialized');
    ///   }
    ///
    ///   ngOnDestroy() {
    ///     console.log('destroyed');
    ///   }
    /// }
    /// ```
    RequireLifecycleOnPrototype,
    angular,
    correctness,
    pending
);

impl Rule for RequireLifecycleOnPrototype {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::PropertyDefinition(prop) = node.kind() else {
            return;
        };

        // Get property name
        let prop_name = match &prop.key {
            oxc_ast::ast::PropertyKey::StaticIdentifier(ident) => ident.name.as_str(),
            _ => return,
        };

        // Check if this is a lifecycle method name
        if !is_lifecycle_method(prop_name) {
            return;
        }

        // Check if the value is a function expression or arrow function
        let is_function_value = prop.value.as_ref().is_some_and(|value| {
            matches!(
                value,
                oxc_ast::ast::Expression::ArrowFunctionExpression(_)
                    | oxc_ast::ast::Expression::FunctionExpression(_)
            )
        });

        if !is_function_value {
            return;
        }

        // Find the parent class
        let Some(class) = get_parent_class(node, ctx) else {
            return;
        };

        // Check if the class has an Angular decorator
        if get_class_angular_decorator(class, ctx).is_none() {
            return;
        }

        ctx.diagnostic(require_lifecycle_on_prototype_diagnostic(prop.key.span(), prop_name));
    }
}

fn get_parent_class<'a, 'b>(
    node: &'b AstNode<'a>,
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
        // Regular method
        r"
        import { Component } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {
            ngOnInit() {}
        }
        ",
        // Multiple lifecycle methods as regular methods
        r"
        import { Component } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {
            ngOnInit() {}
            ngOnDestroy() {}
            ngAfterViewInit() {}
        }
        ",
        // Arrow function property that's not a lifecycle method
        r"
        import { Component } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {
            handleClick = () => {};
        }
        ",
        // Non-Angular class with lifecycle property
        r"
        class TestClass {
            ngOnInit = () => {};
        }
        ",
    ];

    let fail = vec![
        // Arrow function lifecycle
        r"
        import { Component } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {
            ngOnInit = () => {};
        }
        ",
        // Function expression lifecycle
        r"
        import { Component } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {
            ngOnDestroy = function() {};
        }
        ",
        // Multiple arrow function lifecycles
        r"
        import { Component } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {
            ngOnInit = () => {};
            ngOnChanges = () => {};
        }
        ",
        // Directive with arrow lifecycle
        r"
        import { Directive } from '@angular/core';
        @Directive({
            selector: '[appTest]'
        })
        class TestDirective {
            ngAfterViewInit = () => {};
        }
        ",
        // Injectable with arrow lifecycle
        r"
        import { Injectable } from '@angular/core';
        @Injectable({ providedIn: 'root' })
        class TestService {
            ngOnDestroy = () => {};
        }
        ",
        // NOTE: Assignment expression detection (this.ngOnInit = () => {}) is a potential
        // enhancement tracked for future implementation. Angular-eslint checks this pattern
        // but it requires more complex AST traversal in oxlint.
    ];

    Tester::new(RequireLifecycleOnPrototype::NAME, RequireLifecycleOnPrototype::PLUGIN, pass, fail)
        .test_and_snapshot();
}
