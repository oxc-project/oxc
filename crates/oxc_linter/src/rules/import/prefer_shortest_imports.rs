use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactStr, Span};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};

fn prefer_shortest_imports_diagnostic(span: Span, current: &str, preferred: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn("Use the shortest import specifier.")
        .with_help(format!("Replace `{current}` with `{preferred}`."))
        .with_label(span)
}

#[derive(Debug, Clone, JsonSchema, Deserialize, Default)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct PreferShortestImports {
    ban_relative: bool,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prefers the shortest valid import specifier and can optionally ban relative specifiers.
    ///
    /// ### Why is this bad?
    ///
    /// Short, stable specifiers are easier to move and refactor across a large codebase.
    ///
    /// ### Example
    /// ```ts
    /// // bad
    /// import { normalizeString } from '../../lib/utils/string';
    ///
    /// // good
    /// import { normalizeString } from '@/lib/utils/string';
    /// ```
    PreferShortestImports,
    import,
    style,
    conditional_fix,
    config = PreferShortestImports,
);

impl Rule for PreferShortestImports {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run_once(&self, ctx: &LintContext<'_>) {
        let module_record = ctx.module_record();
        for (source, requested_modules) in &module_record.requested_modules {
            let preferred = self.preferred_specifier(module_record, source);
            let Some(preferred) = preferred else {
                continue;
            };
            if source.as_str() == preferred.as_str() {
                continue;
            }
            for requested_module in requested_modules {
                let source_text = ctx.source_range(requested_module.span);
                let replacement = quote_like(source_text, preferred.as_str());
                ctx.diagnostic_with_fix(
                    prefer_shortest_imports_diagnostic(
                        requested_module.span,
                        source.as_str(),
                        preferred.as_str(),
                    ),
                    |fixer| fixer.replace(requested_module.span, replacement.clone()),
                );
            }
        }
    }
}

impl PreferShortestImports {
    fn preferred_specifier(
        &self,
        module_record: &crate::ModuleRecord,
        source: &CompactStr,
    ) -> Option<CompactStr> {
        if self.ban_relative {
            module_record.preferred_non_relative_specifier(source.as_str())
        } else {
            module_record.preferred_specifier(source.as_str())
        }
    }
}

fn quote_like(source_text: &str, replacement: &str) -> String {
    let quote = source_text.chars().next().filter(|ch| *ch == '\'' || *ch == '"').unwrap_or('"');
    format!("{quote}{replacement}{quote}")
}

#[test]
fn test() {
    use crate::tester::Tester;
    use serde_json::json;

    let pass = vec![
        ("import { helper } from '@/utils/helper';", None),
        ("import { helper } from \"@/utils/helper\";", None),
        ("import { helper } from '@utils';", None),
        ("export { helper } from '@/utils/helper';", None),
        ("import '@/utils/helper';", None),
        ("import type { Helper } from '@/utils/helper';", None),
        ("import { helper } from '@/utils/helper';", Some(json!([{ "banRelative": false }]))),
        ("import { helper } from '@/utils/helper';", Some(json!([{ "banRelative": true }]))),
    ];
    let fail = vec![
        ("import { helper } from '../../../utils/helper';", None),
        ("import { helper } from '../../../../src/utils/helper';", None),
        ("import { helper } from \"../../../utils/helper\";", None),
        ("import { helper } from '../../../utils/helper.ts';", None),
        ("export { helper } from '../../../utils/helper';", None),
        ("export * from '../../../utils/helper';", None),
        ("import { helper } from '../../../utils/index';", None),
        ("import '../../../utils/helper';", None),
        ("import type { Helper } from '../../../utils/helper';", None),
        (
            "import { helper } from '../../../utils/helper';",
            Some(json!([{ "banRelative": false }])),
        ),
        ("import { helper } from '../../../utils/helper';", Some(json!([{ "banRelative": true }]))),
        (
            "import { helper } from '../../../utils/helper';\nimport { helper as helper2 } from '../../../utils/helper';",
            None,
        ),
    ];
    let fix = vec![
        (
            "import { helper } from '../../../utils/helper';",
            "import { helper } from '@/utils/helper';",
            None,
        ),
        (
            "import { helper } from '../../../utils/helper';",
            "import { helper } from '@/utils/helper';",
            Some(json!([{ "banRelative": true }])),
        ),
        (
            "import { helper } from '../../../../src/utils/helper';",
            "import { helper } from '@/utils/helper';",
            None,
        ),
        (
            "import { helper } from \"../../../utils/helper\";",
            "import { helper } from \"@/utils/helper\";",
            None,
        ),
        (
            "import { helper } from '../../../utils/helper.ts';",
            "import { helper } from '@/utils/helper';",
            None,
        ),
        (
            "export { helper } from '../../../utils/helper';",
            "export { helper } from '@/utils/helper';",
            None,
        ),
        ("export * from '../../../utils/helper';", "export * from '@/utils/helper';", None),
        (
            "import { helper } from '../../../utils/index';",
            "import { helper } from '@utils';",
            None,
        ),
        ("import '../../../utils/helper';", "import '@/utils/helper';", None),
        (
            "import type { Helper } from '../../../utils/helper';",
            "import type { Helper } from '@/utils/helper';",
            None,
        ),
        (
            "import { helper } from '../../../utils/helper';",
            "import { helper } from '@/utils/helper';",
            Some(json!([{ "banRelative": false }])),
        ),
    ];

    Tester::new(PreferShortestImports::NAME, PreferShortestImports::PLUGIN, pass, fail)
        .change_rule_path("prefer-shortest-imports/src/features/nested/deep/feature.ts")
        .with_import_plugin(true)
        .with_tsconfig("prefer-shortest-imports/tsconfig.json")
        .expect_fix(fix)
        .test_and_snapshot();
}
