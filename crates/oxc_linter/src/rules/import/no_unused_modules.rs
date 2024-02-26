use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactString, Span};

use crate::{context::LintContext, rule::Rule};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-import(namespace): ")]
#[diagnostic(severity(warning), help(""))]
struct NoUnusedModulesDiagnostic(CompactString, #[label] pub Span);

/// <https://github.com/import-js/eslint-plugin-import/blob/main/docs/rules/namespace.md>
#[derive(Debug, Default, Clone)]
pub struct NoUnusedModules;

declare_oxc_lint!(
    /// ### What it does
    /// TODO
    NoUnusedModules,
    nursery
);

impl Rule for NoUnusedModules {
    fn run_once(&self, _ctx: &LintContext<'_>) {}
}

#[test]
fn test() {
    // use crate::tester::Tester;

    // let pass = vec![];

    // let fail = vec![];

    // Tester::new(NoUnusedModules::NAME, pass, fail)
    // .change_rule_path("index.js")
    // .with_import_plugin(true)
    // .test_and_snapshot();
}
