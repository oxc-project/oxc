use fast_glob::glob_match;

use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactStr, Span};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    context::LintContext,
    module_record::ImportImportName,
    rule::{DefaultRuleConfig, Rule},
};

fn no_namespace_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Usage of namespaced aka wildcard \"*\" imports prohibited")
        .with_help("Use named or default imports")
        .with_label(span)
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct NoNamespace(Box<NoNamespaceConfig>);

#[derive(Debug, Default, Clone, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct NoNamespaceConfig {
    /// An array of glob strings for modules that should be ignored by the rule.
    /// For example, `["*.json"]` will ignore all JSON imports.
    ignore: Vec<CompactStr>,
}

impl std::ops::Deref for NoNamespace {
    type Target = NoNamespaceConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce a convention of not using namespaced (a.k.a. "wildcard" *) imports.
    ///
    /// ### Why is this bad?
    ///
    /// Namespaced imports, while sometimes used, are generally considered less ideal in modern JavaScript development for several reasons:
    ///
    /// 1. **Specificity and Namespace Pollution**:
    /// * **Specificity**: Namespaced imports import the entire module, bringing in everything, even if you only need a few specific functions or classes. This can lead to potential naming conflicts if different modules have the same names for different functions.
    /// * **Pollution**: Importing an entire namespace pollutes your current scope with potentially unnecessary functions and variables. It increases the chance of accidental use of an unintended function or variable, leading to harder-to-debug errors.
    ///
    /// 2. **Maintainability**:
    /// * **Clarity**: Namespaced imports can make it harder to understand which specific functions or classes are being used in your code. This is especially true in larger projects with numerous imports.
    /// * **Refactoring**: If a function or class name changes within the imported module, you might need to update several parts of your code if you are using namespaced imports. This becomes even more challenging when dealing with multiple namespaces.
    ///
    /// 3. **Modern Practice**:
    /// * **Explicit Imports**: Modern JavaScript practices encourage explicit imports for specific components. This enhances code readability and maintainability.
    /// * **Tree-Shaking**: Tools like Webpack and Rollup use tree-shaking to remove unused code from your bundles. Namespaced imports can prevent efficient tree-shaking, leading to larger bundle sizes.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// import * as user from 'user-lib';
    ///
    /// import some, * as user from './user';
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// import { getUserName, isUser } from 'user-lib';
    ///
    /// import user from 'user-lib';
    /// import defaultExport, { isUser } from './user';
    /// ```
    NoNamespace,
    import,
    style,
    pending,  // TODO: fixer
    config = NoNamespaceConfig,
);

/// <https://github.com/import-js/eslint-plugin-import/blob/v2.29.1/docs/rules/no-namespace.md>
impl Rule for NoNamespace {
    fn from_configuration(value: serde_json::Value) -> Self {
        serde_json::from_value::<DefaultRuleConfig<NoNamespace>>(value)
            .unwrap_or_default()
            .into_inner()
    }

    fn run_once(&self, ctx: &LintContext<'_>) {
        let module_record = ctx.module_record();

        if !module_record.has_module_syntax {
            return;
        }

        module_record.import_entries.iter().for_each(|entry| match &entry.import_name {
            ImportImportName::NamespaceObject => {
                let source = entry.module_request.name();

                if self.ignore.is_empty()
                    || self.ignore.iter().all(|pattern| {
                        !glob_match(pattern.as_str(), source.trim_start_matches("./"))
                    })
                {
                    ctx.diagnostic(no_namespace_diagnostic(entry.local_name.span));
                }
            }
            ImportImportName::Name(_) | ImportImportName::Default(_) => {}
        });
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (r"import { a, b } from 'foo';", None),
        (r"import { a, b } from './foo';", None),
        (r"import bar from 'bar';", None),
        (r"import bar from './bar';", None),
        (
            r"import * as bar from './ignored-module.ext';",
            Some(serde_json::json!([{ "ignore": ["*.ext"] }])),
        ),
        (
            r"import * as bar from './ignored-module.js';
              import * as baz from './other-module.ts'",
            Some(serde_json::json!([{ "ignore": ["*.js", "*.ts"] }])),
        ),
    ];

    let fail = vec![
        (r"import * as foo from 'foo';", None),
        (r"import defaultExport, * as foo from 'foo';", None),
        (r"import * as foo from './foo';", None),
        (
            r"
            import * as zod from 'zod'
            import * as DrizzleKit from 'drizzle-kit/api'
            ",
            Some(serde_json::json!([{ "ignore": ["zod"] }])),
        ),
    ];

    Tester::new(NoNamespace::NAME, NoNamespace::PLUGIN, pass, fail)
        .change_rule_path("index.js")
        .with_import_plugin(true)
        .test_and_snapshot();
}
