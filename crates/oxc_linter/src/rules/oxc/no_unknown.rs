use oxc_ast::{
    AstKind,
    ast::{Argument, Expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule};

use super::boundary_utils::{classify_path, read_boundary_elements, resolve_local_specifier};

fn no_unknown_diagnostic(span: Span, specifier: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn("Dependencies to unknown elements are not allowed.")
        .with_help(format!(
            "Classify `{specifier}` with `boundaries/elements` or move the dependency into a known architectural area."
        ))
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoUnknown;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Reports local dependencies that resolve to files outside all configured `boundaries/elements`.
    ///
    /// ### Why is this bad?
    ///
    /// Imports into unknown areas bypass the declared architecture even when the source file itself
    /// belongs to a valid element.
    NoUnknown,
    oxc,
    restriction,
);

impl Rule for NoUnknown {
    fn run_once(&self, ctx: &LintContext<'_>) {
        let Some(elements) = read_boundary_elements(ctx) else {
            return;
        };

        let module_record = ctx.module_record();

        for import_entry in &module_record.import_entries {
            let specifier = import_entry.module_request.name();
            let Some(remote_module) = module_record.get_loaded_module(specifier) else {
                continue;
            };

            if classify_path(&remote_module.resolved_absolute_path, &elements).is_none() {
                ctx.diagnostic(no_unknown_diagnostic(import_entry.module_request.span, specifier));
            }
        }

        for export_entry in &module_record.indirect_export_entries {
            let Some(module_request) = &export_entry.module_request else {
                continue;
            };
            let specifier = module_request.name();
            let Some(remote_module) = module_record.get_loaded_module(specifier) else {
                continue;
            };

            if classify_path(&remote_module.resolved_absolute_path, &elements).is_none() {
                ctx.diagnostic(no_unknown_diagnostic(module_request.span, specifier));
            }
        }

        for export_entry in &module_record.star_export_entries {
            let Some(module_request) = &export_entry.module_request else {
                continue;
            };
            let specifier = module_request.name();
            let Some(remote_module) = module_record.get_loaded_module(specifier) else {
                continue;
            };

            if classify_path(&remote_module.resolved_absolute_path, &elements).is_none() {
                ctx.diagnostic(no_unknown_diagnostic(module_request.span, specifier));
            }
        }

        for node in ctx.nodes() {
            match node.kind() {
                AstKind::ImportExpression(import_expr) => {
                    let Expression::StringLiteral(string_literal) = &import_expr.source else {
                        continue;
                    };
                    let specifier = string_literal.value.as_str();
                    let Some(resolved_path) = resolve_local_specifier(ctx.file_path(), specifier)
                    else {
                        continue;
                    };

                    if classify_path(&resolved_path, &elements).is_none() {
                        ctx.diagnostic(no_unknown_diagnostic(string_literal.span, specifier));
                    }
                }
                AstKind::CallExpression(call_expr) => {
                    let Expression::Identifier(ident) = &call_expr.callee else {
                        continue;
                    };

                    if ident.name != "require" {
                        continue;
                    }

                    let Some(Argument::StringLiteral(string_literal)) = call_expr.arguments.first()
                    else {
                        continue;
                    };
                    let specifier = string_literal.value.as_str();
                    let Some(resolved_path) = resolve_local_specifier(ctx.file_path(), specifier)
                    else {
                        continue;
                    };

                    if classify_path(&resolved_path, &elements).is_none() {
                        ctx.diagnostic(no_unknown_diagnostic(string_literal.span, specifier));
                    }
                }
                _ => {}
            }
        }
    }
}

#[test]
fn test() {
    use serde_json::json;

    use crate::tester::Tester;

    fn eslint_config() -> serde_json::Value {
        json!({
            "settings": {
                "boundaries/elements": [
                    { "type": "admin-routes", "pattern": ["src/routes/admin/*"] },
                    { "type": "user-routes", "pattern": ["src/routes/user/*"] },
                    { "type": "public-routes", "pattern": ["src/routes/public/*"] },
                    { "type": "admin-components", "pattern": ["src/components/admin-*", "src/components/admin/*"] },
                    { "type": "user-components", "pattern": ["src/components/user-*"] },
                    { "type": "api-admin", "pattern": ["src/api/admin/*"] },
                    { "type": "api-user", "pattern": ["src/api/user/*"] },
                    { "type": "api-public", "pattern": ["src/api/public/*"] },
                    { "type": "shared", "pattern": ["src/components/ui/*", "src/library/*", "src/hooks/*", "src/utils/*"] },
                    { "type": "layouts", "pattern": ["src/layouts/*"] },
                    { "type": "locales", "pattern": ["src/locales/**/*"] },
                    { "type": "styles", "pattern": ["src/styles/**/*"] }
                ]
            }
        })
    }

    let pass = vec![(
        r#"
            import { uiButton } from "../../components/ui/button.ts";
            const api = require("../../api/public/info");
            const lazy = import("../../layouts/main");
            export const known = [uiButton, api, lazy];
            "#,
        None,
        Some(eslint_config()),
    )];

    let fail = vec![(
        r#"
            import { scratch } from "../../experimental/scratch.ts";
            const required = require("../../experimental/scratch");
            const lazy = import("../../experimental/scratch");
            export const unknown = [scratch, required, lazy];
            "#,
        None,
        Some(eslint_config()),
    )];

    Tester::new(NoUnknown::NAME, NoUnknown::PLUGIN, pass, fail)
        .change_rule_path("boundaries-app/src/routes/public/no-unknown.ts")
        .with_import_plugin(true)
        .intentionally_allow_no_fix_tests()
        .test_and_snapshot();
}
