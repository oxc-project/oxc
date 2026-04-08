use std::cmp::Ordering;

use cow_utils::CowUtils;
use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    AstNode,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};

fn sort_named_imports_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Named import specifiers should be sorted alphabetically.").with_label(span)
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
pub struct SortNamedImportsConfig {
    ignore_case: bool,
    order: SortOrder,
}

impl Default for SortNamedImportsConfig {
    fn default() -> Self {
        Self { ignore_case: true, order: SortOrder::Asc }
    }
}

#[derive(Debug, Default, Clone)]
pub struct SortNamedImports(Box<SortNamedImportsConfig>);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces alphabetically sorted named import specifiers within import statements.
    ///
    /// ### Why is this bad?
    ///
    /// Unsorted named imports are harder to scan and lead to unnecessary merge
    /// conflicts when multiple developers add imports.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// import { z, a, m } from "module";
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// import { a, m, z } from "module";
    /// ```
    SortNamedImports,
    oxc,
    style,
    conditional_fix,
    config = SortNamedImportsConfig
);

impl Rule for SortNamedImports {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<SortNamedImportsConfig>>(value)
            .map(DefaultRuleConfig::into_inner)
            .map(|config| Self(Box::new(config)))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::ImportDeclaration(import) = node.kind() else {
            return;
        };

        let Some(specifiers) = &import.specifiers else {
            return;
        };

        let named: Vec<(usize, &str)> = specifiers
            .iter()
            .enumerate()
            .filter_map(|(i, spec)| {
                if let oxc_ast::ast::ImportDeclarationSpecifier::ImportSpecifier(s) = spec {
                    Some((i, s.local.name.as_str()))
                } else {
                    None
                }
            })
            .collect();

        if named.len() < 2 {
            return;
        }

        for window in named.windows(2) {
            let a_key = if self.0.ignore_case {
                window[0].1.cow_to_ascii_lowercase().into_owned()
            } else {
                window[0].1.to_string()
            };
            let b_key = if self.0.ignore_case {
                window[1].1.cow_to_ascii_lowercase().into_owned()
            } else {
                window[1].1.to_string()
            };

            let ord = a_key.cmp(&b_key);
            let ord = match self.0.order {
                SortOrder::Asc => ord,
                SortOrder::Desc => ord.reverse(),
            };

            if ord == Ordering::Greater {
                let span = specifiers[window[1].0].span();
                // Try to autofix by sorting all named specifiers
                if let Some(fix_text) = self.build_fix(import, &named, ctx) {
                    ctx.diagnostic_with_fix(sort_named_imports_diagnostic(span), |fixer| {
                        fixer.replace(import.span(), fix_text)
                    });
                } else {
                    ctx.diagnostic(sort_named_imports_diagnostic(span));
                }
                return;
            }
        }
    }
}

impl SortNamedImports {
    fn build_fix(
        &self,
        import: &oxc_ast::ast::ImportDeclaration<'_>,
        named: &[(usize, &str)],
        ctx: &LintContext<'_>,
    ) -> Option<String> {
        if ctx.has_comments_between(import.span()) {
            return None;
        }

        let specifiers = import.specifiers.as_ref()?;

        // Only fix if all specifiers are named (no default or namespace)
        if named.len() != specifiers.len() {
            return None;
        }

        let mut sorted: Vec<&str> = named.iter().map(|(_, name)| *name).collect();
        sorted.sort_by(|a, b| {
            let ak = if self.0.ignore_case {
                a.cow_to_ascii_lowercase().into_owned()
            } else {
                a.to_string()
            };
            let bk = if self.0.ignore_case {
                b.cow_to_ascii_lowercase().into_owned()
            } else {
                b.to_string()
            };
            let ord = ak.cmp(&bk);
            match self.0.order {
                SortOrder::Asc => ord,
                SortOrder::Desc => ord.reverse(),
            }
        });

        let specifier_text = sorted.join(", ");
        let source = ctx.source_range(import.source.span());
        Some(format!("import {{ {specifier_text} }} from {source}"))
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r#"import { a, b, c } from "mod";"#,
        r#"import { a } from "mod";"#,
        r#"import def from "mod";"#,
        r#"import { A, b } from "mod";"#,
    ];

    let fail = vec![r#"import { z, a, m } from "mod";"#, r#"import { b, a } from "mod";"#];

    let fix = vec![
        (r#"import { z, a, m } from "mod";"#, r#"import { a, m, z } from "mod""#, None),
        (r#"import { b, a } from "mod";"#, r#"import { a, b } from "mod""#, None),
    ];

    Tester::new(SortNamedImports::NAME, SortNamedImports::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
