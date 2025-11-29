use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{SPAN, Span};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};

fn unexpected_unicode_bom_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected Unicode BOM (Byte Order Mark)")
        .with_help("File must not begin with the Unicode BOM")
        .with_label(span)
}

fn expected_unicode_bom_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Expected Unicode BOM (Byte Order Mark)")
        .with_help("File must begin with the Unicode BOM")
        .with_label(span)
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct UnicodeBom(BomOptionType);

#[derive(Debug, Default, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
enum BomOptionType {
    /// Always require a Unicode BOM (Byte Order Mark) at the beginning of the file.
    Always,
    /// Never allow a Unicode BOM (Byte Order Mark) at the beginning of the file.
    /// This is the default option.
    #[default]
    Never,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Require or disallow Unicode byte order mark (BOM)
    ///
    /// ### Why is this bad?
    ///
    /// The Unicode Byte Order Mark (BOM) is used to specify whether code units are big endian or
    /// little endian. That is, whether the most significant or least significant bytes come first.
    /// UTF-8 does not require a BOM because byte ordering does not matter when characters are a
    /// single byte. Since UTF-8 is the dominant encoding of the web, we make "never" the default
    /// option.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// ﻿var a = 123;
    /// ```
    UnicodeBom,
    eslint,
    restriction,
    fix,
    config = BomOptionType,
);

impl Rule for UnicodeBom {
    fn from_configuration(value: serde_json::Value) -> Self {
        serde_json::from_value::<DefaultRuleConfig<UnicodeBom>>(value)
            .unwrap_or_default()
            .into_inner()
    }

    fn run_once(&self, ctx: &LintContext) {
        let source = ctx.source_text();
        let has_bomb = source.starts_with('﻿');

        if has_bomb && matches!(self.0, BomOptionType::Never) {
            ctx.diagnostic_with_fix(unexpected_unicode_bom_diagnostic(SPAN), |fixer| {
                fixer.delete_range(Span::new(0, 3))
            });
        }

        if !has_bomb && matches!(self.0, BomOptionType::Always) {
            ctx.diagnostic_with_fix(expected_unicode_bom_diagnostic(SPAN), |fixer| {
                fixer.replace(SPAN, "﻿")
            });
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("﻿ var a = 123;", Some(serde_json::json!(["always"]))),
        ("var a = 123;", Some(serde_json::json!(["never"]))),
        // Ensure default config works
        ("var a = 123;", None),
        ("var a = 123; ﻿", Some(serde_json::json!(["never"]))),
    ];

    let fail = vec![
        ("var a = 123;", Some(serde_json::json!(["always"]))),
        (
            " // here's a comment
			var a = 123;",
            Some(serde_json::json!(["always"])),
        ),
        ("﻿ var a = 123;", None),
        ("﻿ var a = 123;", Some(serde_json::json!(["never"]))),
    ];

    let fix = vec![
        ("﻿var a = 123;", "var a = 123;", Some(serde_json::json!(["never"]))),
        ("﻿var a = 123;", "var a = 123;", None),
        ("var a = 123;", "﻿var a = 123;", Some(serde_json::json!(["always"]))),
    ];

    Tester::new(UnicodeBom::NAME, UnicodeBom::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
