use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    AstNode,
    context::LintContext,
    rule::Rule,
    utils::{get_class_angular_decorator, get_decorator_name},
};

fn prefer_inject_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Prefer using the `inject()` function over constructor parameter injection")
        .with_help(
            "Use the `inject()` function to inject dependencies. It provides better type inference, \
            works with signals, and allows for more flexible dependency injection patterns. \
            Example: `private readonly service = inject(MyService);`",
        )
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferInject;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces using the `inject()` function instead of constructor parameter injection
    /// in Angular components, directives, pipes, and services.
    ///
    /// ### Why is this bad?
    ///
    /// Constructor injection is the traditional way to inject dependencies in Angular,
    /// but the `inject()` function provides several advantages:
    /// - Better type inference
    /// - Works seamlessly with signals
    /// - No need for parameter decorators like `@Inject()`, `@Optional()`, etc.
    /// - More flexible - can be used in any initialization context
    /// - Cleaner class structure without constructor bloat
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```typescript
    /// import { Component } from '@angular/core';
    /// import { MyService } from './my.service';
    ///
    /// @Component({
    ///   selector: 'app-example',
    ///   template: ''
    /// })
    /// export class ExampleComponent {
    ///   constructor(private myService: MyService) {}
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```typescript
    /// import { Component, inject } from '@angular/core';
    /// import { MyService } from './my.service';
    ///
    /// @Component({
    ///   selector: 'app-example',
    ///   template: ''
    /// })
    /// export class ExampleComponent {
    ///   private readonly myService = inject(MyService);
    /// }
    /// ```
    PreferInject,
    angular,
    pedantic,
    pending
);

/// Primitive types that should be skipped (not DI candidates)
const PRIMITIVE_TYPES: [&str; 10] = [
    "string",
    "number",
    "boolean",
    "bigint",
    "symbol",
    "undefined",
    "null",
    "any",
    "unknown",
    "never",
];

impl Rule for PreferInject {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::MethodDefinition(method) = node.kind() else {
            return;
        };

        // Check if this is a constructor
        if !method.kind.is_constructor() {
            return;
        }

        // Find the parent class
        let Some(class) = get_parent_class(node, ctx) else {
            return;
        };

        // Check if the class has a relevant Angular decorator
        let Some((decorator_type, _)) = get_class_angular_decorator(class, ctx) else {
            return;
        };

        // Only check Component, Directive, Injectable, and Pipe classes
        use crate::utils::AngularDecoratorType;
        if !matches!(
            decorator_type,
            AngularDecoratorType::Component
                | AngularDecoratorType::Directive
                | AngularDecoratorType::Injectable
                | AngularDecoratorType::Pipe
        ) {
            return;
        }

        // Check constructor parameters for DI candidates
        for param in &method.value.params.items {
            // Check if the parameter has DI decorators like @Inject, @Optional, etc.
            // This check happens first because even primitives with @Inject should be flagged
            let has_di_decorator = param.decorators.iter().any(|dec| {
                let Some(name) = get_decorator_name(dec) else {
                    return false;
                };
                matches!(name, "Inject" | "Optional" | "Self" | "SkipSelf" | "Host" | "Attribute")
            });

            // If it has DI decorators, it's definitely DI - report it
            if has_di_decorator {
                ctx.diagnostic(prefer_inject_diagnostic(param.span));
                continue;
            }

            // Skip if it has no type annotation (can't determine if it's DI)
            let Some(type_annotation) = &param.type_annotation else {
                continue;
            };

            // Check if the type is a primitive (skip primitives without DI decorators)
            if is_primitive_type(type_annotation) {
                continue;
            }

            // Check if the type looks like a service/injectable class (starts with uppercase)
            if looks_like_injectable_type(type_annotation) {
                ctx.diagnostic(prefer_inject_diagnostic(param.span));
            }
        }
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

fn is_primitive_type(type_annotation: &oxc_ast::ast::TSTypeAnnotation<'_>) -> bool {
    use oxc_ast::ast::TSType;

    match &type_annotation.type_annotation {
        TSType::TSStringKeyword(_)
        | TSType::TSNumberKeyword(_)
        | TSType::TSBooleanKeyword(_)
        | TSType::TSBigIntKeyword(_)
        | TSType::TSSymbolKeyword(_)
        | TSType::TSUndefinedKeyword(_)
        | TSType::TSNullKeyword(_)
        | TSType::TSAnyKeyword(_)
        | TSType::TSUnknownKeyword(_)
        | TSType::TSNeverKeyword(_)
        | TSType::TSVoidKeyword(_) => true,
        TSType::TSTypeReference(type_ref) => {
            if let oxc_ast::ast::TSTypeName::IdentifierReference(ident) = &type_ref.type_name {
                PRIMITIVE_TYPES.contains(&ident.name.as_str())
            } else {
                false
            }
        }
        _ => false,
    }
}

fn looks_like_injectable_type(type_annotation: &oxc_ast::ast::TSTypeAnnotation<'_>) -> bool {
    use oxc_ast::ast::TSType;

    match &type_annotation.type_annotation {
        TSType::TSTypeReference(type_ref) => {
            if let oxc_ast::ast::TSTypeName::IdentifierReference(ident) = &type_ref.type_name {
                // Check if the type name starts with uppercase (likely a class/service)
                ident.name.chars().next().is_some_and(|c| c.is_ascii_uppercase())
            } else {
                false
            }
        }
        _ => false,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // Using inject() function
        r"
        import { Component, inject } from '@angular/core';
        import { MyService } from './my.service';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {
            private readonly myService = inject(MyService);
        }
        ",
        // No constructor injection
        r"
        import { Component } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {
            constructor() {}
        }
        ",
        // Constructor with primitive types only
        r"
        import { Component } from '@angular/core';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {
            constructor(private name: string, private count: number) {}
        }
        ",
        // Non-Angular class
        r"
        class TestClass {
            constructor(private service: MyService) {}
        }
        ",
        // NgModule (not flagged - different pattern)
        r"
        import { NgModule } from '@angular/core';
        @NgModule({})
        class AppModule {
            constructor(private service: MyService) {}
        }
        ",
    ];

    let fail = vec![
        // Component with constructor injection
        r"
        import { Component } from '@angular/core';
        import { MyService } from './my.service';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {
            constructor(private myService: MyService) {}
        }
        ",
        // Directive with constructor injection
        r"
        import { Directive } from '@angular/core';
        import { MyService } from './my.service';
        @Directive({
            selector: '[appTest]'
        })
        class TestDirective {
            constructor(private myService: MyService) {}
        }
        ",
        // Injectable with constructor injection
        r"
        import { Injectable, Inject } from '@angular/core';
        @Injectable({ providedIn: 'root' })
        class TestService {
            constructor(@Inject('TOKEN') private value: string) {}
        }
        ",
        // Pipe with constructor injection
        r"
        import { Pipe } from '@angular/core';
        import { MyService } from './my.service';
        @Pipe({ name: 'test' })
        class TestPipe {
            constructor(private myService: MyService) {}
        }
        ",
        // Multiple constructor parameters
        r"
        import { Component } from '@angular/core';
        import { ServiceA, ServiceB } from './services';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {
            constructor(
                private serviceA: ServiceA,
                private serviceB: ServiceB
            ) {}
        }
        ",
        // With @Optional decorator
        r"
        import { Component, Optional } from '@angular/core';
        import { MyService } from './my.service';
        @Component({
            selector: 'app-test',
            template: ''
        })
        class TestComponent {
            constructor(@Optional() private myService: MyService) {}
        }
        ",
    ];

    Tester::new(PreferInject::NAME, PreferInject::PLUGIN, pass, fail).test_and_snapshot();
}
