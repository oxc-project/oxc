use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule};

use super::boundary_utils::{classify_path, read_boundary_elements};

fn no_unknown_files_diagnostic() -> OxcDiagnostic {
    OxcDiagnostic::warn("File does not match any configured element pattern.")
        .with_help("Move the file into a configured architectural element or extend `boundaries/elements`.")
        .with_label(Span::default())
}

#[derive(Debug, Default, Clone)]
pub struct NoUnknownFiles;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Reports files whose paths do not match any configured `boundaries/elements` pattern.
    ///
    /// ### Why is this bad?
    ///
    /// Unclassified files usually mean architecture drift: code exists outside the intended
    /// route, component, API, or shared boundaries.
    NoUnknownFiles,
    oxc,
    restriction,
);

impl Rule for NoUnknownFiles {
    fn run_once(&self, ctx: &LintContext<'_>) {
        let Some(elements) = read_boundary_elements(ctx) else {
            return;
        };

        if classify_path(ctx.file_path(), &elements).is_none() {
            ctx.diagnostic(no_unknown_files_diagnostic());
        }
    }

    fn should_run(&self, ctx: &crate::rules::ContextHost) -> bool {
        ctx.is_first_sub_host()
    }
}

#[test]
fn test() {
    use std::path::PathBuf;

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

    fn test_case(
        path: &'static str,
    ) -> (&'static str, Option<serde_json::Value>, Option<serde_json::Value>, Option<PathBuf>) {
        ("export const value = 1;", None, Some(eslint_config()), Some(PathBuf::from(path)))
    }

    let pass = vec![
        test_case("boundaries-app/src/routes/public/known.ts"),
        test_case("boundaries-app/src/components/ui/button.ts"),
    ];

    let fail = vec![
        test_case("boundaries-app/src/experimental/scratch.ts"),
        test_case("boundaries-app/src/misc/orphan.ts"),
    ];

    Tester::new(NoUnknownFiles::NAME, NoUnknownFiles::PLUGIN, pass, fail)
        .change_rule_path("boundaries-app/src/routes/public/known.ts")
        .intentionally_allow_no_fix_tests()
        .test_and_snapshot();
}
