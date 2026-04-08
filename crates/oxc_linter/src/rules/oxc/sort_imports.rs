use std::cmp::Ordering;

use cow_utils::CowUtils;
use oxc_ast::ast::{ImportDeclaration, Statement};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};

fn sort_imports_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Import statements should be sorted alphabetically by source.")
        .with_label(span)
}

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum SortOrder {
    Desc,
    #[default]
    Asc,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct SortImportsConfig {
    ignore_case: bool,
    order: SortOrder,
}

impl Default for SortImportsConfig {
    fn default() -> Self {
        Self { ignore_case: true, order: SortOrder::Asc }
    }
}

#[derive(Debug, Default, Clone)]
pub struct SortImports(Box<SortImportsConfig>);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces alphabetically sorted import statements by their source path.
    ///
    /// ### Why is this bad?
    ///
    /// Unsorted imports make it harder to find specific imports and lead to
    /// unnecessary merge conflicts. Consistent ordering improves readability.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// import z from "z";
    /// import a from "a";
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// import a from "a";
    /// import z from "z";
    /// ```
    SortImports,
    oxc,
    style,
    none,
    config = SortImportsConfig
);

impl Rule for SortImports {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<SortImportsConfig>>(value)
            .map(DefaultRuleConfig::into_inner)
            .map(|config| Self(Box::new(config)))
    }

    fn run_once(&self, ctx: &LintContext<'_>) {
        let program = ctx.semantic().nodes().program();

        // Collect contiguous groups of import declarations
        let mut imports: Vec<&ImportDeclaration<'_>> = Vec::new();

        for stmt in &program.body {
            if let Statement::ImportDeclaration(import) = stmt {
                imports.push(import);
            } else {
                self.check_group(&imports, ctx);
                imports.clear();
            }
        }

        self.check_group(&imports, ctx);
    }
}

impl SortImports {
    fn check_group(&self, imports: &[&ImportDeclaration<'_>], ctx: &LintContext<'_>) {
        if imports.len() < 2 {
            return;
        }

        for window in imports.windows(2) {
            let a_key = self.import_key(window[0]);
            let b_key = self.import_key(window[1]);

            let ord = a_key.cmp(&b_key);
            let ord = match self.0.order {
                SortOrder::Asc => ord,
                SortOrder::Desc => ord.reverse(),
            };

            if ord == Ordering::Greater {
                ctx.diagnostic(sort_imports_diagnostic(window[1].span()));
                return;
            }
        }
    }

    fn import_key(&self, import: &ImportDeclaration<'_>) -> String {
        let source = import.source.value.as_str();
        if self.0.ignore_case {
            source.cow_to_ascii_lowercase().into_owned()
        } else {
            source.to_string()
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (r#"import a from "a"; import b from "b";"#, None),
        (r#"import a from "a";"#, None),
        (r#"import A from "A"; import b from "b";"#, None),
        // Non-import between groups resets
        (r#"import b from "b"; const x = 1; import a from "a";"#, None),
    ];

    let fail = vec![
        (r#"import z from "z"; import a from "a";"#, None),
        (r#"import b from "b"; import a from "a"; import c from "c";"#, None),
    ];

    Tester::new(SortImports::NAME, SortImports::PLUGIN, pass, fail).test_and_snapshot();
}
