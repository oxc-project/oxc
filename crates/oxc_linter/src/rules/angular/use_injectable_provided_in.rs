use oxc_ast::{AstKind, ast::Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use serde::Deserialize;

use crate::{
    AstNode,
    context::LintContext,
    rule::Rule,
    utils::{
        get_component_metadata, get_decorator_identifier, get_decorator_name,
        get_metadata_property, is_angular_core_import,
    },
};

fn use_injectable_provided_in_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Injectable should use `providedIn` for tree-shaking support")
        .with_help(
            "Add `providedIn: 'root'` (or 'any', 'platform') to the @Injectable decorator. \
            This enables tree-shaking, which removes unused services from your bundle.",
        )
        .with_label(span)
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct UseInjectableProvidedInConfig {
    /// Suffix pattern to ignore certain class names (e.g., "Interceptor")
    #[serde(default)]
    ignore_class_name_suffix: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct UseInjectableProvidedIn {
    ignore_suffix: Option<String>,
}

impl From<UseInjectableProvidedInConfig> for UseInjectableProvidedIn {
    fn from(config: UseInjectableProvidedInConfig) -> Self {
        Self { ignore_suffix: config.ignore_class_name_suffix }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Ensures that `@Injectable` classes use the `providedIn` property for tree-shaking.
    ///
    /// ### Why is this bad?
    ///
    /// Without `providedIn`, services must be added to a module's `providers` array,
    /// which prevents tree-shaking. This means:
    /// - Unused services are included in the bundle
    /// - Larger bundle sizes
    /// - Reduced application performance
    ///
    /// Using `providedIn: 'root'` (or another value) enables tree-shaking, allowing
    /// the compiler to remove unused services from the final bundle.
    ///
    /// Note: Classes implementing `HttpInterceptor` are excluded from this rule as
    /// they require a different registration pattern.
    ///
    /// ### Configuration
    ///
    /// ```json
    /// {
    ///   "angular/use-injectable-provided-in": ["error", {
    ///     "ignoreClassNamePattern": ".*Interceptor$"
    ///   }]
    /// }
    /// ```
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```typescript
    /// import { Injectable } from '@angular/core';
    ///
    /// @Injectable()
    /// export class MyService {}
    ///
    /// @Injectable({})
    /// export class AnotherService {}
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```typescript
    /// import { Injectable } from '@angular/core';
    ///
    /// @Injectable({ providedIn: 'root' })
    /// export class MyService {}
    ///
    /// @Injectable({ providedIn: 'any' })
    /// export class ScopedService {}
    /// ```
    UseInjectableProvidedIn,
    angular,
    pedantic,
    pending
);

impl Rule for UseInjectableProvidedIn {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::Error> {
        if value.is_null() {
            return Ok(Self::default());
        }
        let config_value = value.get(0).unwrap_or(&value);
        serde_json::from_value::<UseInjectableProvidedInConfig>(config_value.clone())
            .map(Into::into)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::Decorator(decorator) = node.kind() else {
            return;
        };

        // Only check @Injectable decorator
        let Some(decorator_name) = get_decorator_name(decorator) else {
            return;
        };

        if decorator_name != "Injectable" {
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

        // Check if class name matches ignore pattern
        if let Some(class_name) = class.id.as_ref().map(|id| id.name.as_str()) {
            if let Some(suffix) = &self.ignore_suffix
                && class_name.ends_with(suffix) {
                    return;
                }

            // Skip HttpInterceptor implementations
            if class_name.ends_with("Interceptor") || implements_http_interceptor(class) {
                return;
            }
        }

        // Get the metadata object
        let Some(metadata) = get_component_metadata(decorator) else {
            // No metadata object - @Injectable() with no args
            ctx.diagnostic(use_injectable_provided_in_diagnostic(decorator.span));
            return;
        };

        // Check if providedIn is present and valid
        match get_metadata_property(metadata, "providedIn") {
            None => {
                ctx.diagnostic(use_injectable_provided_in_diagnostic(decorator.span));
            }
            Some(Expression::NullLiteral(_)) => {
                ctx.diagnostic(use_injectable_provided_in_diagnostic(decorator.span));
            }
            Some(Expression::Identifier(ident)) if ident.name.as_str() == "undefined" => {
                ctx.diagnostic(use_injectable_provided_in_diagnostic(decorator.span));
            }
            _ => {}
        }
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

fn implements_http_interceptor(class: &oxc_ast::ast::Class<'_>) -> bool {
    if class.implements.is_empty() {
        return false;
    }
    class.implements.iter().any(|ts_impl| {
        if let oxc_ast::ast::TSTypeName::IdentifierReference(ident) = &ts_impl.expression {
            return ident.name.as_str() == "HttpInterceptor"
                || ident.name.as_str() == "HttpInterceptorFn";
        }
        false
    })
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // providedIn: 'root'
        (
            r"
            import { Injectable } from '@angular/core';
            @Injectable({ providedIn: 'root' })
            class MyService {}
            ",
            None,
        ),
        // providedIn: 'any'
        (
            r"
            import { Injectable } from '@angular/core';
            @Injectable({ providedIn: 'any' })
            class MyService {}
            ",
            None,
        ),
        // providedIn: 'platform'
        (
            r"
            import { Injectable } from '@angular/core';
            @Injectable({ providedIn: 'platform' })
            class MyService {}
            ",
            None,
        ),
        // HttpInterceptor (skipped)
        (
            r"
            import { Injectable } from '@angular/core';
            @Injectable()
            class MyInterceptor implements HttpInterceptor {}
            ",
            None,
        ),
        // Class name ending with Interceptor (skipped)
        (
            r"
            import { Injectable } from '@angular/core';
            @Injectable()
            class AuthInterceptor {}
            ",
            None,
        ),
        // Non-Angular Injectable
        (
            r"
            import { Injectable } from 'other-lib';
            @Injectable()
            class MyService {}
            ",
            None,
        ),
        // Custom ignore suffix
        (
            r"
            import { Injectable } from '@angular/core';
            @Injectable()
            class MyTestService {}
            ",
            Some(serde_json::json!([{ "ignoreClassNameSuffix": "Service" }])),
        ),
    ];

    let fail = vec![
        // No providedIn
        (
            r"
            import { Injectable } from '@angular/core';
            @Injectable()
            class MyService {}
            ",
            None,
        ),
        // Empty metadata
        (
            r"
            import { Injectable } from '@angular/core';
            @Injectable({})
            class MyService {}
            ",
            None,
        ),
        // providedIn: null
        (
            r"
            import { Injectable } from '@angular/core';
            @Injectable({ providedIn: null })
            class MyService {}
            ",
            None,
        ),
        // providedIn: undefined
        (
            r"
            import { Injectable } from '@angular/core';
            @Injectable({ providedIn: undefined })
            class MyService {}
            ",
            None,
        ),
    ];

    Tester::new(UseInjectableProvidedIn::NAME, UseInjectableProvidedIn::PLUGIN, pass, fail)
        .test_and_snapshot();
}
