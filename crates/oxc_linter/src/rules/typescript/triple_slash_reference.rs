use oxc_ast::{
    ast::{Statement, TSModuleReference},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use rustc_hash::FxHashMap;

use crate::{
    context::{ContextHost, LintContext},
    rule::Rule,
};

fn triple_slash_reference_diagnostic(ref_kind: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Do not use a triple slash reference for {ref_kind}, use `import` style instead."))
        .with_help("Use of triple-slash reference type directives is generally discouraged in favor of ECMAScript Module imports.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct TripleSlashReference(Box<TripleSlashReferenceConfig>);

#[derive(Debug, Clone, Default)]
pub struct TripleSlashReferenceConfig {
    lib: LibOption,
    path: PathOption,
    types: TypesOption,
}
#[derive(Debug, Default, Clone, PartialEq)]
enum LibOption {
    #[default]
    Always,
    Never,
}
#[derive(Debug, Default, Clone, PartialEq)]
enum PathOption {
    Always,
    #[default]
    Never,
}
#[derive(Debug, Default, Clone, PartialEq)]
enum TypesOption {
    Always,
    Never,
    #[default]
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
    /// Disallow certain triple slash directives in favor of ES6-style import declarations.
    ///
    /// ### Why is this bad?
    /// Use of triple-slash reference type directives is generally discouraged in favor of ECMAScript Module imports.
    ///
    /// ### Example
    /// ```ts
    /// /// <reference lib="code" />
    /// globalThis.value;
    /// ```
    TripleSlashReference,
    correctness
);

impl Rule for TripleSlashReference {
    fn from_configuration(value: serde_json::Value) -> Self {
        let options: Option<&serde_json::Value> = value.get(0);
        Self(Box::new(TripleSlashReferenceConfig {
            lib: options
                .and_then(|x| x.get("lib"))
                .and_then(serde_json::Value::as_str)
                .map_or_else(LibOption::default, |value| match value {
                    "always" => LibOption::Always,
                    "never" => LibOption::Never,
                    _ => LibOption::default(),
                }),
            path: options
                .and_then(|x| x.get("path"))
                .and_then(serde_json::Value::as_str)
                .map_or_else(PathOption::default, |value| match value {
                    "always" => PathOption::Always,
                    "never" => PathOption::Never,
                    _ => PathOption::default(),
                }),
            types: options
                .and_then(|x| x.get("types"))
                .and_then(serde_json::Value::as_str)
                .map_or_else(TypesOption::default, |value| match value {
                    "always" => TypesOption::Always,
                    "never" => TypesOption::Never,
                    "prefer-import" => TypesOption::PreferImport,
                    _ => TypesOption::default(),
                }),
        }))
    }

    fn run_once(&self, ctx: &LintContext) {
        let Some(root) = ctx.nodes().root_node() else {
            return;
        };
        let AstKind::Program(program) = root.kind() else { unreachable!() };

        // We don't need to iterate over all comments since Triple-slash directives are only valid at the top of their containing file.
        // We are trying to get the first statement start potioin, falling back to the program end if statement does not exist
        let comments_range_end = program.body.first().map_or(program.span.end, |v| v.span().start);
        let mut refs_for_import = FxHashMap::default();

        for comment in ctx.semantic().comments_range(0..comments_range_end) {
            let raw = &ctx.semantic().source_text()
                [comment.span.start as usize..comment.span.end as usize];
            if let Some((group1, group2)) = get_attr_key_and_value(raw) {
                if (group1 == "types" && self.types == TypesOption::Never)
                    || (group1 == "path" && self.path == PathOption::Never)
                    || (group1 == "lib" && self.lib == LibOption::Never)
                {
                    ctx.diagnostic(triple_slash_reference_diagnostic(
                        &group2,
                        Span::new(comment.span.start - 2, comment.span.end),
                    ));
                }

                if group1 == "types" && self.types == TypesOption::PreferImport {
                    refs_for_import
                        .insert(group2, Span::new(comment.span.start - 2, comment.span.end));
                }
            }
        }

        if !refs_for_import.is_empty() {
            for stmt in &program.body {
                match stmt {
                    Statement::TSImportEqualsDeclaration(decl) => match decl.module_reference {
                        TSModuleReference::ExternalModuleReference(ref mod_ref) => {
                            if let Some(v) = refs_for_import.get(mod_ref.expression.value.as_str())
                            {
                                ctx.diagnostic(triple_slash_reference_diagnostic(
                                    &mod_ref.expression.value,
                                    *v,
                                ));
                            }
                        }
                        TSModuleReference::IdentifierReference(_)
                        | TSModuleReference::QualifiedName(_) => {}
                    },
                    Statement::ImportDeclaration(decl) => {
                        if let Some(v) = refs_for_import.get(decl.source.value.as_str()) {
                            ctx.diagnostic(triple_slash_reference_diagnostic(
                                &decl.source.value,
                                *v,
                            ));
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.source_type().is_typescript()
    }
}

fn get_attr_key_and_value(raw: &str) -> Option<(String, String)> {
    if !raw.starts_with('/') {
        return None;
    }

    let reference_start = "<reference ";
    let reference_end = "/>";

    if let Some(start_idx) = raw.find(reference_start) {
        // Check if the string contains '/>' after the start index
        if let Some(end_idx) = raw[start_idx..].find(reference_end) {
            let reference_str = &raw[start_idx + reference_start.len()..start_idx + end_idx];

            // Split the string by whitespaces
            let parts = reference_str.split_whitespace();

            // Filter parts that start with attribute key pattern
            let filtered_parts: Vec<&str> = parts
                .into_iter()
                .filter(|part| {
                    part.starts_with("types=")
                        || part.starts_with("path=")
                        || part.starts_with("lib=")
                })
                .collect();

            if let Some(attr) = filtered_parts.first() {
                // Split the attribute by '=' to get key and value
                let attr_parts: Vec<&str> = attr.split('=').collect();
                if attr_parts.len() == 2 {
                    let key = attr_parts[0].trim().trim_matches('"').to_string();
                    let value = attr_parts[1].trim_matches('"').trim_end_matches('/').to_string();
                    return Some((key, value));
                }
            }
        }
    }
    None
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (
            r#"
        	        // <reference path="foo" />
        	        // <reference types="bar" />
        	        // <reference lib="baz" />
        	        import * as foo from 'foo';
        	        import * as bar from 'bar';
        	        import * as baz from 'baz';
        	      "#,
            Some(serde_json::json!([{ "path": "never", "types": "never", "lib": "never" }])),
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
            Some(serde_json::json!([{ "path": "never", "types": "never", "lib": "never" }])),
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
            Some(serde_json::json!([{ "path": "always", "types": "always", "lib": "always" }])),
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
            Some(serde_json::json!([{ "path": "always", "types": "always", "lib": "always" }])),
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
            Some(serde_json::json!([{ "path": "always", "types": "always", "lib": "always" }])),
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
            Some(serde_json::json!([{ "path": "always", "types": "always", "lib": "always" }])),
        ),
        (r"import * as foo from 'foo';", Some(serde_json::json!([{ "path": "never" }]))),
        (r"import foo = require('foo');", Some(serde_json::json!([{ "path": "never" }]))),
        (r"import * as foo from 'foo';", Some(serde_json::json!([{ "types": "never" }]))),
        (r"import foo = require('foo');", Some(serde_json::json!([{ "types": "never" }]))),
        (r"import * as foo from 'foo';", Some(serde_json::json!([{ "lib": "never" }]))),
        (r"import foo = require('foo');", Some(serde_json::json!([{ "lib": "never" }]))),
        (r"import * as foo from 'foo';", Some(serde_json::json!([{ "types": "prefer-import" }]))),
        (r"import foo = require('foo');", Some(serde_json::json!([{ "types": "prefer-import" }]))),
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
            Some(serde_json::json!([{ "path": "never", "types": "never", "lib": "never" }])),
        ),
    ];

    let fail = vec![
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

    Tester::new(TripleSlashReference::NAME, pass, fail).test_and_snapshot();
}
