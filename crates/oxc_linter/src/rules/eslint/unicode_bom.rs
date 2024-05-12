use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule};

fn unexpected_unicode_bom_diagnostic(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::warning("eslint(unicode-bom): Unexpected Unicode BOM (Byte Order Mark)")
        .with_help("File must not begin with the Unicode BOM")
        .with_labels([span0.into()])
}

fn expected_unicode_bom_diagnostic(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::warning("eslint(unicode-bom): Expected Unicode BOM (Byte Order Mark)")
        .with_help("File must begin with the Unicode BOM")
        .with_labels([span0.into()])
}

#[derive(Debug, Default, Clone)]
pub struct UnicodeBom {
    bom_option: BomOptionType,
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
    /// ### Example
    /// ```javascript
    /// ﻿var a = 123;"
    /// ```
    UnicodeBom,
    restriction,
);

impl Rule for UnicodeBom {
    fn from_configuration(value: serde_json::Value) -> Self {
        let obj = value.get(0);

        Self {
            bom_option: obj
                .and_then(serde_json::Value::as_str)
                .map(BomOptionType::from)
                .unwrap_or_default(),
        }
    }

    fn run_once(&self, ctx: &LintContext) {
        let source = ctx.source_text();
        let has_bomb = source.starts_with('﻿');

        if has_bomb && matches!(self.bom_option, BomOptionType::Never) {
            ctx.diagnostic(unexpected_unicode_bom_diagnostic(Span::new(0, 0)));
        }

        if !has_bomb && matches!(self.bom_option, BomOptionType::Always) {
            ctx.diagnostic(expected_unicode_bom_diagnostic(Span::new(0, 0)));
        }
    }
}

#[derive(Debug, Default, Clone)]
enum BomOptionType {
    Always,
    #[default]
    Never,
}

impl BomOptionType {
    pub fn from(raw: &str) -> Self {
        match raw {
            "always" => Self::Always,
            _ => Self::Never,
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("﻿ var a = 123;", Some(serde_json::json!(["always"]))),
        ("var a = 123;", Some(serde_json::json!(["never"]))),
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

    Tester::new(UnicodeBom::NAME, pass, fail).test_and_snapshot();
}
