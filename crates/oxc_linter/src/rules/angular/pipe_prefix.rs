use oxc_ast::AstKind;
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
        get_metadata_string_value, is_angular_core_import,
    },
};

fn pipe_prefix_diagnostic(span: Span, prefixes: &[String]) -> OxcDiagnostic {
    let prefix_list = prefixes.join(", ");
    OxcDiagnostic::warn(format!("Pipe names should be prefixed with: {prefix_list}"))
        .with_help(
            "Use a consistent prefix for pipe names to avoid naming collisions and to make it \
            clear which pipes belong to your application.",
        )
        .with_label(span)
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct PipePrefixConfig {
    #[serde(default)]
    prefixes: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct PipePrefix {
    prefixes: Vec<String>,
}

impl From<PipePrefixConfig> for PipePrefix {
    fn from(config: PipePrefixConfig) -> Self {
        Self { prefixes: config.prefixes }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces a consistent prefix for pipe names.
    ///
    /// ### Why is this bad?
    ///
    /// Using a prefix for pipe names helps:
    /// - Avoid naming collisions with Angular built-in pipes or third-party pipes
    /// - Easily identify which pipes belong to your application
    /// - Maintain consistency across the codebase
    ///
    /// ### Configuration
    ///
    /// This rule requires configuration to specify the allowed prefixes:
    ///
    /// ```json
    /// {
    ///   "angular/pipe-prefix": ["error", { "prefixes": ["app", "my"] }]
    /// }
    /// ```
    ///
    /// ### Examples
    ///
    /// With configuration `{ "prefixes": ["app"] }`:
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```typescript
    /// import { Pipe, PipeTransform } from '@angular/core';
    ///
    /// @Pipe({
    ///   name: 'formatDate'  // Missing prefix
    /// })
    /// export class FormatDatePipe implements PipeTransform {
    ///   transform(value: Date): string {
    ///     return value.toISOString();
    ///   }
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```typescript
    /// import { Pipe, PipeTransform } from '@angular/core';
    ///
    /// @Pipe({
    ///   name: 'appFormatDate'  // Has 'app' prefix
    /// })
    /// export class FormatDatePipe implements PipeTransform {
    ///   transform(value: Date): string {
    ///     return value.toISOString();
    ///   }
    /// }
    /// ```
    PipePrefix,
    angular,
    pedantic,
    pending
);

impl Rule for PipePrefix {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::Error> {
        if value.is_null() {
            return Ok(Self::default());
        }
        let config_value = value.get(0).unwrap_or(&value);
        serde_json::from_value::<PipePrefixConfig>(config_value.clone()).map(Into::into)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        // If no prefixes configured, skip the rule
        if self.prefixes.is_empty() {
            return;
        }

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

        // Get the pipe name
        let Some(pipe_name) = get_metadata_string_value(metadata, "name") else {
            return;
        };

        // Check if the pipe name starts with any of the configured prefixes
        let has_valid_prefix = self.prefixes.iter().any(|prefix| {
            if pipe_name.starts_with(prefix) {
                // Ensure the prefix is followed by uppercase (camelCase convention)
                let rest = &pipe_name[prefix.len()..];
                rest.is_empty() || rest.chars().next().is_some_and(|c| c.is_ascii_uppercase())
            } else {
                false
            }
        });

        if !has_valid_prefix {
            ctx.diagnostic(pipe_prefix_diagnostic(decorator.span, &self.prefixes));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // Correct prefix
        (
            r"
            import { Pipe, PipeTransform } from '@angular/core';
            @Pipe({
                name: 'appFormatDate'
            })
            class FormatDatePipe implements PipeTransform {
                transform(value: Date): string {
                    return value.toISOString();
                }
            }
            ",
            Some(serde_json::json!([{ "prefixes": ["app"] }])),
        ),
        // Multiple allowed prefixes
        (
            r"
            import { Pipe, PipeTransform } from '@angular/core';
            @Pipe({
                name: 'myFormatDate'
            })
            class FormatDatePipe implements PipeTransform {
                transform(value: Date): string {
                    return value.toISOString();
                }
            }
            ",
            Some(serde_json::json!([{ "prefixes": ["app", "my"] }])),
        ),
        // No prefixes configured (rule disabled)
        (
            r"
            import { Pipe, PipeTransform } from '@angular/core';
            @Pipe({
                name: 'formatDate'
            })
            class FormatDatePipe implements PipeTransform {
                transform(value: Date): string {
                    return value.toISOString();
                }
            }
            ",
            None,
        ),
        // Non-Angular Pipe
        (
            r"
            import { Pipe } from 'other-lib';
            @Pipe({
                name: 'formatDate'
            })
            class FormatDatePipe {}
            ",
            Some(serde_json::json!([{ "prefixes": ["app"] }])),
        ),
        // Template literal pipe name with correct prefix
        (
            r"
            import { Pipe, PipeTransform } from '@angular/core';
            @Pipe({
                name: `appFormatDate`
            })
            class FormatDatePipe implements PipeTransform {
                transform(value: Date): string {
                    return value.toISOString();
                }
            }
            ",
            Some(serde_json::json!([{ "prefixes": ["app"] }])),
        ),
    ];

    let fail = vec![
        // Missing prefix
        (
            r"
            import { Pipe, PipeTransform } from '@angular/core';
            @Pipe({
                name: 'formatDate'
            })
            class FormatDatePipe implements PipeTransform {
                transform(value: Date): string {
                    return value.toISOString();
                }
            }
            ",
            Some(serde_json::json!([{ "prefixes": ["app"] }])),
        ),
        // Wrong prefix
        (
            r"
            import { Pipe, PipeTransform } from '@angular/core';
            @Pipe({
                name: 'myFormatDate'
            })
            class FormatDatePipe implements PipeTransform {
                transform(value: Date): string {
                    return value.toISOString();
                }
            }
            ",
            Some(serde_json::json!([{ "prefixes": ["app"] }])),
        ),
        // Prefix without proper casing
        (
            r"
            import { Pipe, PipeTransform } from '@angular/core';
            @Pipe({
                name: 'appformatdate'
            })
            class FormatDatePipe implements PipeTransform {
                transform(value: Date): string {
                    return value.toISOString();
                }
            }
            ",
            Some(serde_json::json!([{ "prefixes": ["app"] }])),
        ),
        // Template literal missing prefix
        (
            r"
            import { Pipe, PipeTransform } from '@angular/core';
            @Pipe({
                name: `formatDate`
            })
            class FormatDatePipe implements PipeTransform {
                transform(value: Date): string {
                    return value.toISOString();
                }
            }
            ",
            Some(serde_json::json!([{ "prefixes": ["app"] }])),
        ),
    ];

    Tester::new(PipePrefix::NAME, PipePrefix::PLUGIN, pass, fail).test_and_snapshot();
}
