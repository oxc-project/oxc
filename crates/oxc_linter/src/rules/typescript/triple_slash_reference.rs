use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use oxc_ast::{AstKind, ast::TSModuleReference};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    context::{ContextHost, LintContext},
    rule::{DefaultRuleConfig, Rule},
    utils::AlwaysNever,
};

fn triple_slash_reference_diagnostic(ref_kind: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Do not use a triple slash reference for {ref_kind}, use `import` style instead."))
        .with_help("Use of triple-slash reference type directives is generally discouraged in favor of ECMAScript Module imports.")
        .with_label(span)
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct TripleSlashReference(Box<TripleSlashReferenceConfig>);

#[derive(Debug, Clone, Default, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct TripleSlashReferenceConfig {
    /// What to enforce for `/// <reference lib="..." />` references.
    lib: AlwaysNever,
    /// What to enforce for `/// <reference path="..." />` references.
    path: PathOption,
    /// What to enforce for `/// <reference types="..." />` references.
    types: TypesOption,
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
enum PathOption {
    /// Allow triple-slash `path` references.
    Always,
    #[default]
    /// Disallow triple-slash `path` references.
    Never,
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
enum TypesOption {
    /// Allow triple-slash `types` references.
    Always,
    /// Disallow triple-slash `types` references.
    Never,
    #[default]
    /// Prefer ES module import declarations over triple-slash `types` references.
    /// This option only reports when there is an existing `import` declaration for the same module.
    ///
    /// For example, this would be reported as a lint violation with `prefer-import`:
    /// ```ts
    /// /// <reference types="foo" />
    /// import { bar } from 'foo';
    /// ```
    PreferImport,
}

impl std::ops::Deref for TripleSlashReference {
    type Target = TripleSlashReferenceConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow certain triple slash directives in favor of ES module import declarations.
    ///
    /// ### Why is this bad?
    ///
    /// Use of triple-slash reference type directives is generally discouraged in favor of ECMAScript Module imports.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// /// <reference lib="code" />
    /// globalThis.value;
    /// ```
    TripleSlashReference,
    typescript,
    correctness,
    recommended,
    config = TripleSlashReferenceConfig,
    version = "0.2.0",
    short_description =
        "Disallow certain triple slash directives in favor of ES module import declarations.",
);

impl Rule for TripleSlashReference {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        DefaultRuleConfig::<Self>::from_value(value).map(DefaultRuleConfig::into_inner)
    }

    fn run_once(&self, ctx: &LintContext) {
        let program = ctx.nodes().program();

        // We don't need to iterate over all comments since Triple-slash directives are only valid at the top of their containing file.
        // We are trying to get the first directive or statement start position, falling back to the program end if neither exists
        let comments_range_end = program
            .directives
            .first()
            .map(|directive| directive.span.start)
            .or_else(|| program.body.first().map(|stmt| stmt.span().start))
            .unwrap_or(program.span.end);
        let mut refs_for_import = vec![];

        for comment in ctx.comments_range(0..comments_range_end) {
            if !comment.is_line() {
                continue;
            }
            let raw = ctx.source_range(comment.content_span());
            if let Some((group1, group2)) = get_attr_key_and_value(raw) {
                if (group1 == "types" && self.types == TypesOption::Never)
                    || (group1 == "path" && self.path == PathOption::Never)
                    || (group1 == "lib" && self.lib == AlwaysNever::Never)
                {
                    ctx.diagnostic(triple_slash_reference_diagnostic(group2, comment.span));
                }

                if group1 == "types" && self.types == TypesOption::PreferImport {
                    refs_for_import.push((group2, comment.span));
                }
            }
        }

        if refs_for_import.is_empty() {
            return;
        }

        for node in ctx.nodes().iter() {
            let source = match node.kind() {
                AstKind::ImportDeclaration(decl) => &decl.source,
                AstKind::TSImportEqualsDeclaration(decl) => match &decl.module_reference {
                    TSModuleReference::ExternalModuleReference(mod_ref) => &mod_ref.expression,
                    TSModuleReference::IdentifierReference(_)
                    | TSModuleReference::QualifiedName(_) => continue,
                },
                _ => continue,
            };
            let Some(value) = source.value.as_str() else { continue };
            for &(name, span) in &refs_for_import {
                if name == value {
                    ctx.diagnostic(triple_slash_reference_diagnostic(value, span));
                }
            }
        }
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.source_type().is_typescript()
    }
}

/// Parses the attribute of a `/// <reference types|path|lib="..." />` directive from the
/// content of a line comment.
fn get_attr_key_and_value(raw: &str) -> Option<(&str, &str)> {
    let rest = raw.strip_prefix('/')?.trim_start().strip_prefix("<reference")?.trim_start();
    let key = ["types", "path", "lib"].into_iter().find(|key| rest.starts_with(key))?;
    let rest = rest[key.len()..].trim_start().strip_prefix('=')?.trim_start();
    let quote = rest.chars().next().filter(|ch| matches!(ch, '"' | '\''))?;
    let value = &rest[1..];
    let end = value.find(quote)?;
    Some((key, &value[..end]))
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("\"use strict\";\n/// <reference path=\"./types.d.ts\" />\nexport {};", None),
        ("//// <reference path=\"./types.d.ts\" />\nexport {};", None),
        ("/*/ <reference path=\"./types.d.ts\" /> */\nexport {};", None),
        ("/// <reference types=\"lodash\" />\nexport import lodash = Foo.Bar;", None),
        (
            r#"
                    // <reference path="foo" />
                    // <reference types="bar" />
                    // <reference lib="baz" />
                    import * as foo from 'foo';
                    import * as bar from 'bar';
                    import * as baz from 'baz';
                  "#,
            Some(serde_json::json!([{ "lib": "never", "path": "never", "types": "never" }])),
        ),
        (
            r#"
                    // <reference path="foo" />
                    // <reference types="bar" />
                    // <reference lib="baz" />
                    import foo = require('foo');
                    import bar = require('bar');
                    import baz = require('baz');
                  "#,
            Some(serde_json::json!([{ "lib": "never", "path": "never", "types": "never" }])),
        ),
        (
            r#"
                    /// <reference path="foo" />
                    /// <reference types="bar" />
                    /// <reference lib="baz" />
                    import * as foo from 'foo';
                    import * as bar from 'bar';
                    import * as baz from 'baz';
                  "#,
            Some(serde_json::json!([{ "lib": "always", "path": "always", "types": "always" }])),
        ),
        (
            r#"
                    /// <reference path="foo" />
                    /// <reference types="bar" />
                    /// <reference lib="baz" />
                    import foo = require('foo');
                    import bar = require('bar');
                    import baz = require('baz');
                  "#,
            Some(serde_json::json!([{ "lib": "always", "path": "always", "types": "always" }])),
        ),
        (
            r#"
                    /// <reference path="foo" />
                    /// <reference types="bar" />
                    /// <reference lib="baz" />
                    import foo = foo;
                    import bar = bar;
                    import baz = baz;
                  "#,
            Some(serde_json::json!([{ "lib": "always", "path": "always", "types": "always" }])),
        ),
        (
            r#"
                    /// <reference path="foo" />
                    /// <reference types="bar" />
                    /// <reference lib="baz" />
                    import foo = foo.foo;
                    import bar = bar.bar.bar.bar;
                    import baz = baz.baz;
                  "#,
            Some(serde_json::json!([{ "lib": "always", "path": "always", "types": "always" }])),
        ),
        ("import * as foo from 'foo';", Some(serde_json::json!([{ "path": "never" }]))),
        ("import foo = require('foo');", Some(serde_json::json!([{ "path": "never" }]))),
        ("import * as foo from 'foo';", Some(serde_json::json!([{ "types": "never" }]))),
        ("import foo = require('foo');", Some(serde_json::json!([{ "types": "never" }]))),
        ("import * as foo from 'foo';", Some(serde_json::json!([{ "lib": "never" }]))),
        ("import foo = require('foo');", Some(serde_json::json!([{ "lib": "never" }]))),
        ("import * as foo from 'foo';", Some(serde_json::json!([{ "types": "prefer-import" }]))),
        ("import foo = require('foo');", Some(serde_json::json!([{ "types": "prefer-import" }]))),
        (
            r#"
                    /// <reference types="foo" />
                    import * as bar from 'bar';
                  "#,
            Some(serde_json::json!([{ "types": "prefer-import" }])),
        ),
        (
            r#"
                    /*
                    /// <reference types="foo" />
                    */
                    import * as foo from 'foo';
                  "#,
            Some(serde_json::json!([{ "lib": "never", "path": "never", "types": "never" }])),
        ),
    ];

    let fail = vec![
        ("/// <reference path=\"./types.d.ts\" />\n\"use strict\";\nexport {};", None),
        ("/// <reference types=\"lodash\" />\nexport import lodash = require(\"lodash\");", None),
        ("/// <reference types='lodash' />\nimport 'lodash';", None),
        ("/// <reference types=\"foo\" />\ndeclare module \"x\" { import \"foo\"; }", None),
        ("/// <reference types = \"lodash\" />\nimport 'lodash';", None),
        (
            "/// <reference types=\"lodash\" />\n/// <reference types=\"lodash\" />\nimport \"lodash\";",
            None,
        ),
        ("//// <reference path=\"./a.d.ts\" />\n/// <reference path=\"./b.d.ts\" />", None),
        (
            r#"
            /// <reference types="foo" />
            import * as foo from 'foo';
                  "#,
            Some(serde_json::json!([{ "types": "prefer-import" }])),
        ),
        (
            r#"
            /// <reference types="foo" />
            import foo = require('foo');
                  "#,
            Some(serde_json::json!([{ "types": "prefer-import" }])),
        ),
        (r#"/// <reference path="foo" />"#, Some(serde_json::json!([{ "path": "never" }]))),
        (r#"/// <reference types="foo" />"#, Some(serde_json::json!([{ "types": "never" }]))),
        (r#"/// <reference lib="foo" />"#, Some(serde_json::json!([{ "lib": "never" }]))),
    ];

    Tester::new(TripleSlashReference::NAME, TripleSlashReference::PLUGIN, pass, fail)
        .test_and_snapshot();
}

#[test]
fn test_first_reference_attribute() {
    assert_eq!(
        get_attr_key_and_value(r#"/ <reference types="first" path="second" />"#),
        Some(("types", "first")),
    );
    assert_eq!(
        get_attr_key_and_value(r#"/ <reference types="a=b" path="valid" />"#),
        Some(("types", "a=b")),
    );
    assert_eq!(get_attr_key_and_value(r"/<reference types = 'pkg' />"), Some(("types", "pkg")));
    assert_eq!(get_attr_key_and_value(r#"/ <reference no-default-lib="true" />"#), None);
    assert_eq!(get_attr_key_and_value(r#"/ <reference unrelated="value" />"#), None);
    assert_eq!(get_attr_key_and_value(r#"/ <reference path="pkg"/>"#), Some(("path", "pkg")),);
}
