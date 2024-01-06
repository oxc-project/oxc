use std::collections::HashSet;

use once_cell::sync::Lazy;
use oxc_ast::{
    ast::{Declaration, ModuleDeclaration, Statement, TSModuleReference},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::{self, Error},
};
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use regex::Regex;

use crate::{context::LintContext, rule::Rule};

static REFERENCE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^\/\s*<reference\s*(types|path|lib)\s*=\s*["|'](.*)["|']"#).unwrap()
});

#[derive(Debug, Error, Diagnostic)]
#[error("typescript-eslint(triple-slash-reference): Do not use a triple slash reference for {0}, use `import` style instead.")]
#[diagnostic(severity(warning), help("Use of triple-slash reference type directives is generally discouraged in favor of ECMAScript Module imports."))]
struct TripleSlashReferenceDiagnostic(String, #[label] pub Span);

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
    /// ```javascript
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
        let Some(root) = ctx.nodes().iter().next() else { return };
        let AstKind::Program(program) = root.kind() else { return };
        let mut import_source_set = HashSet::new();

        for stmt in &program.body {
            match stmt {
                Statement::Declaration(Declaration::TSImportEqualsDeclaration(decl)) => {
                    match *decl.module_reference {
                        TSModuleReference::ExternalModuleReference(ref mod_ref) => {
                            import_source_set.insert(mod_ref.expression.value.as_str());
                        }
                        TSModuleReference::TypeName(_) => {}
                    }
                }
                Statement::ModuleDeclaration(st) => {
                    if let ModuleDeclaration::ImportDeclaration(ref decl) = **st {
                        import_source_set.insert(decl.source.value.as_str());
                    }
                }
                _ => {}
            }
        }

        // We don't need to iterate over all comments since Triple-slash directives are only valid at the top of their containing file.
        // We are trying to get the first statement start potioin, falling back to the program end if statement does not exist
        let comments_range_end = program.body.first().map_or(program.span.end, |v| v.span().start);

        let comments = ctx.semantic().trivias().comments();
        for (start, comment) in comments.range(0..comments_range_end) {
            let raw = &ctx.semantic().source_text()[*start as usize..comment.end() as usize];
            if let Some(captures) = REFERENCE_REGEX.captures(raw) {
                let group1 = captures.get(1).map_or("", |m| m.as_str());
                let group2 = captures.get(2).map_or("", |m| m.as_str());

                if (group1 == "types" && self.types == TypesOption::Never)
                    || (group1 == "path" && self.path == PathOption::Never)
                    || (group1 == "lib" && self.lib == LibOption::Never)
                {
                    ctx.diagnostic(TripleSlashReferenceDiagnostic(
                        group2.to_string(),
                        Span { start: *start - 2, end: comment.end() },
                    ));
                }

                if group1 == "types"
                    && self.types == TypesOption::PreferImport
                    && import_source_set.contains(group2)
                {
                    ctx.diagnostic(TripleSlashReferenceDiagnostic(
                        group2.to_string(),
                        Span { start: *start - 2, end: comment.end() },
                    ));
                }
            }
        }
    }
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
