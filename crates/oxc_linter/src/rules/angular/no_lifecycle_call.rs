use oxc_ast::{AstKind, ast::Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    AstNode,
    context::LintContext,
    rule::Rule,
    utils::{get_class_angular_decorator, is_lifecycle_method},
};

fn no_lifecycle_call_diagnostic(span: Span, method_name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Avoid explicit calls to lifecycle method `{method_name}`"))
        .with_help(
            "Lifecycle methods are called by Angular. Calling them explicitly can lead to \
            unexpected behavior. Move the logic to a separate method if you need to call it.",
        )
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoLifecycleCall;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows explicit calls to lifecycle methods like `ngOnInit()`, `ngOnDestroy()`, etc.
    ///
    /// ### Why is this bad?
    ///
    /// Lifecycle methods are meant to be called by Angular, not by the developer.
    /// Calling them explicitly can lead to:
    /// - Unexpected side effects
    /// - Logic being executed at wrong times
    /// - Difficulty understanding when code is actually running
    ///
    /// If you need to reuse logic from a lifecycle method, extract it to a separate method
    /// and call that method instead.
    ///
    /// Note: Calling `super.ngOnInit()` (or other lifecycle) from within the same lifecycle
    /// method is allowed, as this is a valid pattern for class inheritance.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```typescript
    /// import { Component, OnInit } from '@angular/core';
    ///
    /// @Component({
    ///   selector: 'app-example',
    ///   template: ''
    /// })
    /// export class ExampleComponent implements OnInit {
    ///   ngOnInit() {}
    ///
    ///   reset() {
    ///     this.ngOnInit(); // Bad: explicit call to lifecycle method
    ///   }
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```typescript
    /// import { Component, OnInit } from '@angular/core';
    ///
    /// @Component({
    ///   selector: 'app-example',
    ///   template: ''
    /// })
    /// export class ExampleComponent implements OnInit {
    ///   ngOnInit() {
    ///     this.initialize();
    ///   }
    ///
    ///   reset() {
    ///     this.initialize(); // Good: call the extracted method
    ///   }
    ///
    ///   private initialize() {
    ///     // initialization logic
    ///   }
    /// }
    /// ```
    NoLifecycleCall,
    angular,
    pedantic,
    pending
);

impl Rule for NoLifecycleCall {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call) = node.kind() else {
            return;
        };

        // Check if it's a member expression call like `this.ngOnInit()` or `instance.ngOnInit()`
        let Expression::StaticMemberExpression(member) = &call.callee else {
            return;
        };

        let method_name = member.property.name.as_str();

        // Check if it's a lifecycle method
        if !is_lifecycle_method(method_name) {
            return;
        }

        // Allow super.ngXxx() calls within the same lifecycle method
        if is_super_call_in_same_lifecycle(&member.object, method_name, node, ctx) {
            return;
        }

        // Only report if we're inside an Angular class
        let Some(class) = get_parent_class(node, ctx) else {
            return;
        };

        if get_class_angular_decorator(class, ctx).is_none() {
            return;
        }

        ctx.diagnostic(no_lifecycle_call_diagnostic(call.span, method_name));
    }
}

/// Check if this is a super.ngXxx() call within the same ngXxx method
fn is_super_call_in_same_lifecycle(
    object: &Expression<'_>,
    method_name: &str,
    node: &AstNode<'_>,
    ctx: &LintContext<'_>,
) -> bool {
    // Check if the object is `super`
    if !matches!(object, Expression::Super(_)) {
        return false;
    }

    // Check if we're inside the same lifecycle method
    for ancestor in ctx.nodes().ancestors(node.id()) {
        if let AstKind::MethodDefinition(method) = ancestor.kind()
            && let Some(name) = method.key.static_name() {
                return name == method_name;
            }
    }

    false
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
        // Normal method calls
        r"
        import { Component } from '@angular/core';
        @Component({ selector: 'app-test', template: '' })
        class TestComponent {
            onClick() {
                this.doSomething();
            }
            doSomething() {}
        }
        ",
        // Super call within same lifecycle method (allowed)
        r"
        import { Component } from '@angular/core';
        @Component({ selector: 'app-test', template: '' })
        class TestComponent extends BaseComponent {
            ngOnInit() {
                super.ngOnInit();
            }
        }
        ",
        r"
        import { Component } from '@angular/core';
        @Component({ selector: 'app-test', template: '' })
        class TestComponent extends BaseComponent {
            ngOnDestroy() {
                super.ngOnDestroy();
            }
        }
        ",
        r"
        import { Component } from '@angular/core';
        @Component({ selector: 'app-test', template: '' })
        class TestComponent extends BaseComponent {
            ngAfterViewInit() {
                super.ngAfterViewInit();
            }
        }
        ",
        // Calling non-lifecycle methods
        r"
        import { Component } from '@angular/core';
        @Component({ selector: 'app-test', template: '' })
        class TestComponent {
            ngOnInit() {
                this.initialize();
            }
            initialize() {}
        }
        ",
        // Calling methods with similar names but not lifecycle
        r"
        import { Component } from '@angular/core';
        @Component({ selector: 'app-test', template: '' })
        class TestComponent {
            ngOnInitCustom() {}
            test() {
                this.ngOnInitCustom();
            }
        }
        ",
        // Non-Angular class - lifecycle calls should be allowed
        r"
        class TestClass {
            ngOnInit() {}
            reset() {
                this.ngOnInit();
            }
        }
        ",
    ];

    let fail = vec![
        // Explicit this.ngOnInit() call
        r"
        import { Component } from '@angular/core';
        @Component({ selector: 'app-test', template: '' })
        class TestComponent {
            ngOnInit() {}
            reset() {
                this.ngOnInit();
            }
        }
        ",
        // Explicit this.ngOnDestroy() call
        r"
        import { Component } from '@angular/core';
        @Component({ selector: 'app-test', template: '' })
        class TestComponent {
            ngOnDestroy() {}
            cleanup() {
                this.ngOnDestroy();
            }
        }
        ",
        // Explicit this.ngAfterViewInit() call
        r"
        import { Component } from '@angular/core';
        @Component({ selector: 'app-test', template: '' })
        class TestComponent {
            ngAfterViewInit() {}
            refresh() {
                this.ngAfterViewInit();
            }
        }
        ",
        // Call on another instance
        r"
        import { Component } from '@angular/core';
        @Component({ selector: 'app-test', template: '' })
        class TestComponent {
            otherComponent: OtherComponent;
            test() {
                this.otherComponent.ngOnInit();
            }
        }
        ",
        // Multiple lifecycle calls
        r"
        import { Component } from '@angular/core';
        @Component({ selector: 'app-test', template: '' })
        class TestComponent {
            ngOnInit() {}
            ngOnDestroy() {}
            reset() {
                this.ngOnInit();
                this.ngOnDestroy();
            }
        }
        ",
        // Super call in different lifecycle method (not allowed)
        r"
        import { Component } from '@angular/core';
        @Component({ selector: 'app-test', template: '' })
        class TestComponent extends BaseComponent {
            ngOnInit() {
                super.ngOnDestroy();
            }
        }
        ",
        // Call in nested function
        r"
        import { Component } from '@angular/core';
        @Component({ selector: 'app-test', template: '' })
        class TestComponent {
            ngOnInit() {}
            setup() {
                const fn = () => {
                    this.ngOnInit();
                };
            }
        }
        ",
        // Directive with lifecycle call
        r"
        import { Directive } from '@angular/core';
        @Directive({ selector: '[appTest]' })
        class TestDirective {
            ngOnInit() {}
            reset() {
                this.ngOnInit();
            }
        }
        ",
    ];

    Tester::new(NoLifecycleCall::NAME, NoLifecycleCall::PLUGIN, pass, fail).test_and_snapshot();
}
