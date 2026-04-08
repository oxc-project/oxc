use oxc_ast::{
    AstKind,
    ast::{Argument, Expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};

use super::boundary_utils::{classify_path, read_boundary_elements, resolve_local_specifier};

fn boundaries_dependencies_diagnostic(
    span: Span,
    from_type: &str,
    to_type: &str,
    specifier: &str,
) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "Dependency from `{from_type}` to `{to_type}` is not allowed."
    ))
    .with_help(format!(
        "Remove the import of `{specifier}` or move the dependency to an allowed architectural layer."
    ))
    .with_label(span)
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct BoundariesDependencies(Box<BoundariesDependenciesConfig>);

#[derive(Debug, Default, Clone, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct BoundariesDependenciesConfig {
    default: BoundaryPolicy,
    rules: Vec<BoundaryDependencyRule>,
}

#[derive(Debug, Clone, Copy, Default, JsonSchema, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum BoundaryPolicy {
    #[default]
    Allow,
    Disallow,
}

#[derive(Debug, Default, Clone, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
struct BoundaryDependencyRule {
    from: BoundaryTypeSelector,
    allow: Option<BoundaryDependencyTargetSelector>,
    disallow: Option<BoundaryDependencyTargetSelector>,
}

#[derive(Debug, Clone, Default, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
struct BoundaryDependencyTargetSelector {
    to: BoundaryTypeSelector,
}

#[derive(Debug, Clone, Default, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
struct BoundaryTypeSelector {
    #[serde(rename = "type", default)]
    element_types: BoundaryStringOrList,
}

#[derive(Debug, Clone, Default, JsonSchema, Deserialize)]
#[serde(untagged)]
enum BoundaryStringOrList {
    Single(String),
    Many(Vec<String>),
    #[default]
    Empty,
}

impl BoundaryTypeSelector {
    fn matches(&self, element_type: &str) -> bool {
        match &self.element_types {
            BoundaryStringOrList::Single(single) => single == "*" || single == element_type,
            BoundaryStringOrList::Many(many) => {
                many.iter().any(|candidate| candidate == "*" || candidate == element_type)
            }
            BoundaryStringOrList::Empty => false,
        }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces architectural dependency boundaries between configured path-based elements.
    ///
    /// ### Why is this bad?
    ///
    /// Large frontends often rely on route, component, API, and shared-layer boundaries
    /// to avoid cross-layer coupling. Invalid imports make the architecture harder to
    /// evolve and easier to accidentally break.
    ///
    /// ### Examples
    ///
    /// Given `public-routes` configured to disallow `admin-routes`:
    ///
    /// ```ts
    /// import { adminDashboard } from "../../admin/dashboard";
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// import { uiButton } from "../../components/ui/button";
    /// ```
    BoundariesDependencies,
    oxc,
    restriction,
    config = BoundariesDependenciesConfig,
);

impl Rule for BoundariesDependencies {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run_once(&self, ctx: &LintContext<'_>) {
        let Some(elements) = read_boundary_elements(ctx) else {
            return;
        };

        let Some(from_type) = classify_path(ctx.file_path(), &elements) else {
            return;
        };

        let module_record = ctx.module_record();

        for import_entry in &module_record.import_entries {
            let specifier = import_entry.module_request.name();
            let Some(remote_module) = module_record.get_loaded_module(specifier) else {
                continue;
            };
            let Some(to_type) = classify_path(&remote_module.resolved_absolute_path, &elements)
            else {
                continue;
            };

            if !is_allowed_dependency(&self.0, &from_type, &to_type) {
                ctx.diagnostic(boundaries_dependencies_diagnostic(
                    import_entry.module_request.span,
                    &from_type,
                    &to_type,
                    specifier,
                ));
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
            let Some(to_type) = classify_path(&remote_module.resolved_absolute_path, &elements)
            else {
                continue;
            };

            if !is_allowed_dependency(&self.0, &from_type, &to_type) {
                ctx.diagnostic(boundaries_dependencies_diagnostic(
                    module_request.span,
                    &from_type,
                    &to_type,
                    specifier,
                ));
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
            let Some(to_type) = classify_path(&remote_module.resolved_absolute_path, &elements)
            else {
                continue;
            };

            if !is_allowed_dependency(&self.0, &from_type, &to_type) {
                ctx.diagnostic(boundaries_dependencies_diagnostic(
                    module_request.span,
                    &from_type,
                    &to_type,
                    specifier,
                ));
            }
        }

        for node in ctx.nodes() {
            match node.kind() {
                AstKind::ImportExpression(import_expr) => {
                    let Expression::StringLiteral(string_literal) = &import_expr.source else {
                        continue;
                    };

                    check_local_dependency(
                        &self.0,
                        ctx,
                        &elements,
                        &from_type,
                        string_literal.value.as_str(),
                        string_literal.span,
                    );
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

                    check_local_dependency(
                        &self.0,
                        ctx,
                        &elements,
                        &from_type,
                        string_literal.value.as_str(),
                        string_literal.span,
                    );
                }
                _ => {}
            }
        }
    }
}

fn check_local_dependency(
    config: &BoundariesDependenciesConfig,
    ctx: &LintContext<'_>,
    elements: &[super::boundary_utils::BoundaryElementSetting],
    from_type: &str,
    specifier: &str,
    span: Span,
) {
    let Some(resolved_path) = resolve_local_specifier(ctx.file_path(), specifier) else {
        return;
    };
    let Some(to_type) = classify_path(&resolved_path, elements) else {
        return;
    };

    if !is_allowed_dependency(config, from_type, &to_type) {
        ctx.diagnostic(boundaries_dependencies_diagnostic(span, from_type, &to_type, specifier));
    }
}

fn is_allowed_dependency(
    config: &BoundariesDependenciesConfig,
    from_type: &str,
    to_type: &str,
) -> bool {
    let mut decision = match config.default {
        BoundaryPolicy::Allow => Some(true),
        BoundaryPolicy::Disallow => Some(false),
    };

    for rule in &config.rules {
        if !rule.from.matches(from_type) {
            continue;
        }

        if let Some(disallow) = &rule.disallow
            && disallow.to.matches(to_type)
        {
            decision = Some(false);
            continue;
        }

        if let Some(allow) = &rule.allow
            && allow.to.matches(to_type)
        {
            decision = Some(true);
        }
    }

    decision.unwrap_or(true)
}

#[test]
fn test_public_routes_boundaries_dependencies() {
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

    fn rule_config() -> serde_json::Value {
        json!([{
            "default": "allow",
            "rules": [
                {
                    "from": { "type": ["user-routes", "user-components"] },
                    "disallow": { "to": { "type": ["admin-routes", "admin-components", "api-admin"] } }
                },
                {
                    "from": { "type": ["public-routes"] },
                    "disallow": { "to": { "type": ["admin-routes", "admin-components", "api-admin", "user-routes", "user-components", "api-user"] } }
                },
                {
                    "from": { "type": ["api-admin", "api-user", "api-public"] },
                    "disallow": { "to": { "type": ["admin-routes", "admin-components", "user-routes", "user-components", "public-routes"] } }
                }
            ]
        }])
    }

    let pass = vec![(
        r#"
            import { uiButton } from "../../components/ui/button.ts";
            import { publicInfoApi } from "../../api/public/info.ts";
            import { mainLayout } from "../../layouts/main.ts";

            export const landingPage = [uiButton, publicInfoApi, mainLayout];
            "#,
        Some(rule_config()),
        Some(eslint_config()),
    )];

    let fail = vec![(
        r#"
            import { adminDashboard } from "../admin/dashboard.ts";
            import { userProfile } from "../user/profile.ts";
            import { adminCard } from "../../components/admin-card.ts";
            import { userCard } from "../../components/user-card.ts";
            import { userProfileApi } from "../../api/user/profile.ts";
            import { adminUsersApi } from "../../api/admin/users.ts";

            export const landingPage = [
              adminDashboard,
              userProfile,
              adminCard,
              userCard,
              userProfileApi,
              adminUsersApi,
            ];
            "#,
        Some(rule_config()),
        Some(eslint_config()),
    )];

    Tester::new(BoundariesDependencies::NAME, BoundariesDependencies::PLUGIN, pass, fail)
        .change_rule_path("boundaries-app/src/routes/public/landing.ts")
        .with_import_plugin(true)
        .intentionally_allow_no_fix_tests()
        .test_and_snapshot();
}

#[test]
fn test_dynamic_import_and_require_boundaries_dependencies() {
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

    fn rule_config() -> serde_json::Value {
        json!([{
            "default": "allow",
            "rules": [
                {
                    "from": { "type": ["public-routes"] },
                    "disallow": { "to": { "type": ["admin-routes", "user-routes", "admin-components", "user-components", "api-admin", "api-user"] } }
                }
            ]
        }])
    }

    let pass = vec![(
        r#"
        async function loadPage() {
          const ui = await import("../../components/ui/button");
          const api = require("../../api/public/info");
          return [ui, api];
        }
        "#,
        Some(rule_config()),
        Some(eslint_config()),
    )];

    let fail = vec![(
        r#"
        async function loadPage() {
          const adminRoute = await import("../admin/dashboard");
          const userRoute = await import("../user/profile");
          const adminApi = require("../../api/admin/users");
          return [adminRoute, userRoute, adminApi];
        }
        "#,
        Some(rule_config()),
        Some(eslint_config()),
    )];

    Tester::new(BoundariesDependencies::NAME, BoundariesDependencies::PLUGIN, pass, fail)
        .change_rule_path("boundaries-app/src/routes/public/lazy.ts")
        .with_import_plugin(true)
        .with_snapshot_suffix("dynamic")
        .intentionally_allow_no_fix_tests()
        .test_and_snapshot();
}

#[test]
fn test_default_disallow_with_allow_override_boundaries_dependencies() {
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

    fn rule_config() -> serde_json::Value {
        json!([{
            "default": "disallow",
            "rules": [
                {
                    "from": { "type": ["public-routes"] },
                    "allow": { "to": { "type": ["shared", "layouts", "api-public"] } }
                }
            ]
        }])
    }

    let pass = vec![(
        r#"
        import { uiButton } from "../../components/ui/button";
        import { publicInfoApi } from "../../api/public/info";
        import { mainLayout } from "../../layouts/main";

        export const allowedPage = [uiButton, publicInfoApi, mainLayout];
        "#,
        Some(rule_config()),
        Some(eslint_config()),
    )];

    let fail = vec![(
        r#"
        import { uiButton } from "../../components/ui/button";
        import { publicInfoApi } from "../../api/public/info";
        import { adminDashboard } from "../admin/dashboard";

        export const blockedPage = [uiButton, publicInfoApi, adminDashboard];
        "#,
        Some(rule_config()),
        Some(eslint_config()),
    )];

    Tester::new(BoundariesDependencies::NAME, BoundariesDependencies::PLUGIN, pass, fail)
        .change_rule_path("boundaries-app/src/routes/public/explicit.ts")
        .with_import_plugin(true)
        .with_snapshot_suffix("default_disallow")
        .intentionally_allow_no_fix_tests()
        .test_and_snapshot();
}

#[test]
fn test_user_components_boundaries_dependencies() {
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

    fn rule_config() -> serde_json::Value {
        json!([{
            "default": "allow",
            "rules": [
                {
                    "from": { "type": ["user-routes", "user-components"] },
                    "disallow": { "to": { "type": ["admin-routes", "admin-components", "api-admin"] } }
                },
                {
                    "from": { "type": ["public-routes"] },
                    "disallow": { "to": { "type": ["admin-routes", "admin-components", "api-admin", "user-routes", "user-components", "api-user"] } }
                },
                {
                    "from": { "type": ["api-admin", "api-user", "api-public"] },
                    "disallow": { "to": { "type": ["admin-routes", "admin-components", "user-routes", "user-components", "public-routes"] } }
                }
            ]
        }])
    }

    let pass = vec![(
        r#"
        import { uiButton } from "./ui/button.ts";
        import { userProfileApi } from "../api/user/profile.ts";

        export const userComponent = [uiButton, userProfileApi];
        "#,
        Some(rule_config()),
        Some(eslint_config()),
    )];

    let fail = vec![(
        r#"
        import { adminDashboard } from "../routes/admin/dashboard.ts";
        import { adminCard } from "./admin-card.ts";
        import { adminUsersApi } from "../api/admin/users.ts";

        export const userComponent = [adminDashboard, adminCard, adminUsersApi];
        "#,
        Some(rule_config()),
        Some(eslint_config()),
    )];

    Tester::new(BoundariesDependencies::NAME, BoundariesDependencies::PLUGIN, pass, fail)
        .change_rule_path("boundaries-app/src/components/user-card.ts")
        .with_import_plugin(true)
        .with_snapshot_suffix("user_components")
        .intentionally_allow_no_fix_tests()
        .test_and_snapshot();
}

#[test]
fn test_api_public_boundaries_dependencies() {
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

    fn rule_config() -> serde_json::Value {
        json!([{
            "default": "allow",
            "rules": [
                {
                    "from": { "type": ["user-routes", "user-components"] },
                    "disallow": { "to": { "type": ["admin-routes", "admin-components", "api-admin"] } }
                },
                {
                    "from": { "type": ["public-routes"] },
                    "disallow": { "to": { "type": ["admin-routes", "admin-components", "api-admin", "user-routes", "user-components", "api-user"] } }
                },
                {
                    "from": { "type": ["api-admin", "api-user", "api-public"] },
                    "disallow": { "to": { "type": ["admin-routes", "admin-components", "user-routes", "user-components", "public-routes"] } }
                }
            ]
        }])
    }

    let pass = vec![(
        r#"
        import { uiButton } from "../../components/ui/button.ts";
        import { adminUsersApi } from "../admin/users.ts";

        export const apiResult = [uiButton, adminUsersApi];
        "#,
        Some(rule_config()),
        Some(eslint_config()),
    )];

    let fail = vec![(
        r#"
        import { adminDashboard } from "../../routes/admin/dashboard.ts";
        import { userProfile } from "../../routes/user/profile.ts";
        import { mainLayout } from "../../layouts/main.ts";
        import { userCard } from "../../components/user-card.ts";

        export const apiResult = [adminDashboard, userProfile, mainLayout, userCard];
        "#,
        Some(rule_config()),
        Some(eslint_config()),
    )];

    Tester::new(BoundariesDependencies::NAME, BoundariesDependencies::PLUGIN, pass, fail)
        .change_rule_path("boundaries-app/src/api/public/info.ts")
        .with_import_plugin(true)
        .with_snapshot_suffix("api_public")
        .intentionally_allow_no_fix_tests()
        .test_and_snapshot();
}
