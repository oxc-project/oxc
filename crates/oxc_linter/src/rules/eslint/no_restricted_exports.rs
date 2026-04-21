use std::borrow::Borrow;

use lazy_regex::Regex;
use rustc_hash::FxHashSet;
use schemars::JsonSchema;
use serde::Deserialize;

use oxc_ast::{
    AstKind,
    ast::{Declaration, ExportAllDeclaration, ExportDefaultDeclaration, ExportNamedDeclaration},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    AstNode,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
    utils::deserialize_regex_option,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DefaultExportType {
    Direct,
    Named,
    DefaultFrom,
    NamedFrom,
    NamespaceFrom,
}

fn no_restricted_default_exports_diagnostic(
    span: Span,
    export_type: DefaultExportType,
) -> OxcDiagnostic {
    let warn = match export_type {
        DefaultExportType::DefaultFrom => "Reexporting 'default' export is restricted.",
        DefaultExportType::Direct => "Exporting 'default' is restricted.",
        DefaultExportType::Named => "Exporting named value as default is restricted.",
        DefaultExportType::NamedFrom => "Reexporting named export as default is restricted.",
        DefaultExportType::NamespaceFrom => "Reexporting namespace as default is restricted.",
    };

    OxcDiagnostic::warn(warn).with_help("Use named export instead.").with_label(span)
}

fn no_restricted_named_exports_diagnostic(span: Span, name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("'{name}' is restricted from being used as an exported name."))
        .with_help("Rename this export.")
        .with_label(span)
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct NoRestrictedExports(Box<NoRestrictedExportsConfig>);

#[derive(Debug, Default, Clone, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct NoRestrictedExportsConfig {
    /// An array of strings, where each string is a name to be restricted.
    ///
    /// Example of **incorrect** code for `"restrictedNamedExports": ["foo"]`:
    ///
    /// ```ts
    /// export const foo = 1;
    /// ```
    ///
    /// Example of **correct** code for `"restrictedNamedExports": ["foo"]`:
    ///
    /// ```ts
    /// export const bar = 1;
    /// ```
    ///
    /// By design, this option doesn't disallow export default declarations. If
    /// you configure `default` as a restricted name, that restriction will apply
    /// only to named export declarations.
    ///
    /// Example of **incorrect** code for `"restrictedNamedExports": ["default"]`:
    ///
    /// ```ts
    /// function foo() {}
    /// export { foo as default };
    ///
    /// export { default } from "some_module";
    /// ```
    restricted_named_exports: FxHashSet<String>,
    /// A string representing a regular expression pattern. Named exports
    /// matching this pattern will be restricted. This option does not apply to
    /// default named exports.
    ///
    /// Example of **incorrect** code for `"restrictedNamedExportsPattern": "bar$":
    ///
    /// ```ts
    /// export const foobar = 1;
    /// ```
    ///
    /// Example of **correct** code for `"restrictedNamedExportsPattern": "bar$":
    ///
    /// ```ts
    /// export const foo = 1;
    /// ```
    #[serde(deserialize_with = "deserialize_regex_option")]
    restricted_named_exports_pattern: Option<Regex>,
    /// An object with boolean properties to restrict certain default export
    /// declarations. This option works only if the `restrictedNamedExports`
    /// option does not contain the `"default"` value.
    restrict_default_exports: RestrictDefaultExports,
    #[serde(skip_serializing)]
    has_default_restricted_named_export: bool,
}

#[derive(Debug, Default, Clone, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
struct RestrictDefaultExports {
    /// Whether to restrict `export { default } from` declarations.
    ///
    /// Example of **incorrect** code for `"restrictDefaultExports": { "defaultFrom": true }`:
    ///
    /// ```js
    /// export { default } from 'foo';
    /// ```
    default_from: bool,
    /// Whether to restrict `export default` declarations.
    ///
    /// Example of **incorrect** code for `"restrictDefaultExports": { "direct": true }`:
    ///
    /// ```js
    /// const foo = 123;
    /// export default foo;
    /// ```
    direct: bool,
    /// Whether to restrict `export { foo as default }` declarations.
    ///
    /// Example of **incorrect** code for `"restrictDefaultExports": { "named": true }`:
    ///
    /// ```js
    /// const foo = 123;
    /// export { foo as default };
    /// ```
    named: bool,
    /// Whether to restrict `export { foo as default } from` declarations.
    ///
    /// Example of **incorrect** code for `"restrictDefaultExports": { "namedFrom": true }`:
    ///
    /// ```js
    /// export { foo as default } from 'foo';
    /// ```
    named_from: bool,
    /// Whether to restrict `export * as default from` declarations.
    ///
    /// Example of **incorrect** code for `"restrictDefaultExports": { "namespaceFrom": true }`:
    ///
    /// ```js
    /// export * as default from 'foo';
    /// ```
    namespace_from: bool,
}

impl std::ops::Deref for NoRestrictedExports {
    type Target = NoRestrictedExportsConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for NoRestrictedExports {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule disallows specified names from being used as exported names.
    ///
    /// By default, this rule doesn’t disallow any names. Only the names you specify in the configuration will be disallowed.
    ///
    /// ### Why is this bad?
    ///
    /// In a project, certain names may be disallowed from being used as exported names for various reasons.
    NoRestrictedExports,
    eslint,
    nursery, // TODO: change category to `restriction`
    config = NoRestrictedExportsConfig,
    version = "1.59.0",
);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LocalFromSpecifier {
    Named,     // export { foo as default } from 'foo';
    Default,   // export { default } from 'mod';
    Namespace, // export * as default from 'mod';
}

impl Rule for NoRestrictedExports {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value)
            .map(DefaultRuleConfig::into_inner)
            .map(|mut c| {
                // Cache if "default" is in restricted_named_exports
                c.has_default_restricted_named_export =
                    c.restricted_named_exports.contains("default");
                c
            })
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::ExportAllDeclaration(export) => self.check_export_all(export, ctx),
            AstKind::ExportDefaultDeclaration(export) => self.check_export_default(export, ctx),
            AstKind::ExportNamedDeclaration(export) => self.check_export_named(export, ctx),
            _ => {}
        }
    }
}

impl NoRestrictedExports {
    fn check_export_all(&self, export: &ExportAllDeclaration, ctx: &LintContext) {
        if let Some(exported) = export.exported.as_ref() {
            // Always check for restricted named exports
            self.check_no_restricted_named_exports(
                ctx,
                export.span,
                std::iter::once(exported.name().into_string()),
            );

            // If exported name is default, also check for restricted named default export
            if exported.name() == "default" {
                self.check_no_restricted_named_default_exports(
                    ctx,
                    export.span,
                    true,
                    std::iter::once(LocalFromSpecifier::Namespace),
                );
            }
        }
    }

    fn check_export_default(&self, export: &ExportDefaultDeclaration, ctx: &LintContext) {
        self.check_no_restricted_direct_default_exports(ctx, export.span);
    }

    fn check_export_named(&self, export: &ExportNamedDeclaration, ctx: &LintContext) {
        let (named_exports, has_default_exports, local_specifiers) = export.specifiers.iter().fold(
            (Vec::new(), false, Vec::new()),
            |(mut names, mut has_default, mut specifiers), spec| {
                names.push(spec.exported.name().into_string());

                if spec.exported.name() == "default" {
                    has_default = true;
                    let local_spec = match spec.local.name().as_str() {
                        "default" => LocalFromSpecifier::Default,
                        _ => LocalFromSpecifier::Named,
                    };
                    specifiers.push(local_spec);
                }

                (names, has_default, specifiers)
            },
        );

        self.check_no_restricted_named_exports(ctx, export.span, named_exports);

        if has_default_exports {
            self.check_no_restricted_named_default_exports(
                ctx,
                export.span,
                export.source.is_some(),
                local_specifiers,
            );
        }

        if let Some(declaration) = export.declaration.as_ref() {
            match declaration {
                Declaration::FunctionDeclaration(_) | Declaration::ClassDeclaration(_) => {
                    if let Some(id) = declaration.id() {
                        self.check_no_restricted_named_exports(
                            ctx,
                            export.span,
                            std::iter::once(id.name.into_string()),
                        );
                    }
                }
                Declaration::VariableDeclaration(variable) => {
                    self.check_no_restricted_named_exports(
                        ctx,
                        export.span,
                        variable.declarations.iter().flat_map(|d| {
                            d.id.get_binding_identifiers()
                                .into_iter()
                                .map(|id| id.name.into_string())
                        }),
                    );
                }
                _ => {}
            }
        }
    }

    fn check_no_restricted_direct_default_exports(&self, ctx: &LintContext<'_>, span: Span) {
        // restrict default exports option only works if the restrictedNamedExports option does not contain the "default" value.
        if self.has_default_restricted_named_export {
            return;
        }

        if self.restrict_default_exports.direct {
            ctx.diagnostic(no_restricted_default_exports_diagnostic(
                span,
                DefaultExportType::Direct,
            ));
        }
    }

    fn check_no_restricted_named_default_exports<S>(
        &self,
        ctx: &LintContext<'_>,
        span: Span,
        has_source: bool,
        specifiers: S,
    ) where
        S: IntoIterator,
        S::Item: Borrow<LocalFromSpecifier>,
    {
        // restrict default exports option only works if the restrictedNamedExports option does not contain the "default" value.
        if self.has_default_restricted_named_export {
            return;
        }

        if let Some(type_export) = match (has_source, &self.restrict_default_exports) {
            // Without source: check .named
            (false, opts) => opts.named.then_some(DefaultExportType::Named),
            // With source: check specific types
            (true, opts) => specifiers.into_iter().find_map(|spec| match spec.borrow() {
                LocalFromSpecifier::Default => {
                    opts.default_from.then_some(DefaultExportType::DefaultFrom)
                }
                LocalFromSpecifier::Named => {
                    opts.named_from.then_some(DefaultExportType::NamedFrom)
                }
                LocalFromSpecifier::Namespace => {
                    opts.namespace_from.then_some(DefaultExportType::NamespaceFrom)
                }
            }),
        } {
            ctx.diagnostic(no_restricted_default_exports_diagnostic(span, type_export));
        }
    }

    fn check_no_restricted_named_exports<S>(&self, ctx: &LintContext<'_>, span: Span, exports: S)
    where
        S: IntoIterator,
        S::Item: Borrow<String>,
    {
        if self.restricted_named_exports.is_empty()
            && self.restricted_named_exports_pattern.is_none()
        {
            return;
        }

        for export in exports {
            let export = export.borrow();
            if self.restricted_named_exports.contains(export)
                || (export != "default"
                    && self
                        .restricted_named_exports_pattern
                        .as_ref()
                        .is_some_and(|r| r.is_match(export)))
            {
                ctx.diagnostic(no_restricted_named_exports_diagnostic(span, export));
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("export var a;", None),
        ("export function a() {}", None),
        ("export class A {}", None),
        ("var a; export { a };", None),
        ("var b; export { b as a };", None),
        ("export { a } from 'foo';", None),
        ("export { b as a } from 'foo';", None),
        ("export var a;", Some(serde_json::json!([{}]))),
        ("export function a() {}", Some(serde_json::json!([{}]))),
        ("export class A {}", Some(serde_json::json!([{}]))),
        ("var a; export { a };", Some(serde_json::json!([{}]))),
        ("var b; export { b as a };", Some(serde_json::json!([{}]))),
        ("export { a } from 'foo';", Some(serde_json::json!([{}]))),
        ("export { b as a } from 'foo';", Some(serde_json::json!([{}]))),
        ("export var a;", Some(serde_json::json!([{ "restrictedNamedExports": [] }]))),
        ("export function a() {}", Some(serde_json::json!([{ "restrictedNamedExports": [] }]))),
        ("export class A {}", Some(serde_json::json!([{ "restrictedNamedExports": [] }]))),
        ("var a; export { a };", Some(serde_json::json!([{ "restrictedNamedExports": [] }]))),
        ("var b; export { b as a };", Some(serde_json::json!([{ "restrictedNamedExports": [] }]))),
        ("export { a } from 'foo';", Some(serde_json::json!([{ "restrictedNamedExports": [] }]))),
        (
            "export { b as a } from 'foo';",
            Some(serde_json::json!([{ "restrictedNamedExports": [] }])),
        ),
        ("export var a;", Some(serde_json::json!([{ "restrictedNamedExports": ["x"] }]))),
        ("export let a;", Some(serde_json::json!([{ "restrictedNamedExports": ["x"] }]))),
        ("export const a = 1;", Some(serde_json::json!([{ "restrictedNamedExports": ["x"] }]))),
        ("export function a() {}", Some(serde_json::json!([{ "restrictedNamedExports": ["x"] }]))),
        ("export function *a() {}", Some(serde_json::json!([{ "restrictedNamedExports": ["x"] }]))),
        (
            "export async function a() {}",
            Some(serde_json::json!([{ "restrictedNamedExports": ["x"] }])),
        ),
        (
            "export async function *a() {}",
            Some(serde_json::json!([{ "restrictedNamedExports": ["x"] }])),
        ),
        ("export class A {}", Some(serde_json::json!([{ "restrictedNamedExports": ["x"] }]))),
        ("var a; export { a };", Some(serde_json::json!([{ "restrictedNamedExports": ["x"] }]))),
        (
            "var b; export { b as a };",
            Some(serde_json::json!([{ "restrictedNamedExports": ["x"] }])),
        ),
        (
            "export { a } from 'foo';",
            Some(serde_json::json!([{ "restrictedNamedExports": ["x"] }])),
        ),
        (
            "export { b as a } from 'foo';",
            Some(serde_json::json!([{ "restrictedNamedExports": ["x"] }])),
        ),
        (
            "export { '' } from 'foo';",
            Some(serde_json::json!([{ "restrictedNamedExports": ["undefined"] }])),
        ),
        (
            "export { '' } from 'foo';",
            Some(serde_json::json!([{ "restrictedNamedExports": [" "] }])),
        ),
        (
            "export { ' ' } from 'foo';",
            Some(serde_json::json!([{ "restrictedNamedExports": [""] }])),
        ),
        (
            "export { ' a', 'a ' } from 'foo';",
            Some(serde_json::json!([{ "restrictedNamedExports": ["a"] }])),
        ),
        ("export var b = a;", Some(serde_json::json!([{ "restrictedNamedExports": ["a"] }]))),
        (
            "export let [b = a] = [];",
            Some(serde_json::json!([{ "restrictedNamedExports": ["a"] }])),
        ),
        ("export const [b] = [a];", Some(serde_json::json!([{ "restrictedNamedExports": ["a"] }]))),
        (
            "export var { a: b } = {};",
            Some(serde_json::json!([{ "restrictedNamedExports": ["a"] }])),
        ),
        (
            "export let { b = a } = {};",
            Some(serde_json::json!([{ "restrictedNamedExports": ["a"] }])),
        ),
        (
            "export const { c: b = a } = {};",
            Some(serde_json::json!([{ "restrictedNamedExports": ["a"] }])),
        ),
        ("export function b(a) {}", Some(serde_json::json!([{ "restrictedNamedExports": ["a"] }]))),
        (
            "export class A { a(){} }",
            Some(serde_json::json!([{ "restrictedNamedExports": ["a"] }])),
        ),
        (
            "export class A extends B {}",
            Some(serde_json::json!([{ "restrictedNamedExports": ["B"] }])),
        ),
        (
            "var a; export { a as b };",
            Some(serde_json::json!([{ "restrictedNamedExports": ["a"] }])),
        ),
        (
            "var a; export { a as 'a ' };",
            Some(serde_json::json!([{ "restrictedNamedExports": ["a"] }])),
        ),
        (
            "export { a as b } from 'foo';",
            Some(serde_json::json!([{ "restrictedNamedExports": ["a"] }])),
        ),
        (
            "export { a as 'a ' } from 'foo';",
            Some(serde_json::json!([{ "restrictedNamedExports": ["a"] }])),
        ),
        (
            "export { 'a' as 'a ' } from 'foo';",
            Some(serde_json::json!([{ "restrictedNamedExports": ["a"] }])),
        ),
        ("export { b } from 'a';", Some(serde_json::json!([{ "restrictedNamedExports": ["a"] }]))),
        ("export * as b from 'a';", Some(serde_json::json!([{ "restrictedNamedExports": ["a"] }]))),
        ("var a;", Some(serde_json::json!([{ "restrictedNamedExports": ["a"] }]))),
        ("let a;", Some(serde_json::json!([{ "restrictedNamedExports": ["a"] }]))),
        ("const a = 1;", Some(serde_json::json!([{ "restrictedNamedExports": ["a"] }]))),
        ("function a() {}", Some(serde_json::json!([{ "restrictedNamedExports": ["a"] }]))),
        ("class A {}", Some(serde_json::json!([{ "restrictedNamedExports": ["A"] }]))),
        ("import a from 'foo';", Some(serde_json::json!([{ "restrictedNamedExports": ["a"] }]))),
        (
            "import { a } from 'foo';",
            Some(serde_json::json!([{ "restrictedNamedExports": ["a"] }])),
        ),
        (
            "import { b as a } from 'foo';",
            Some(serde_json::json!([{ "restrictedNamedExports": ["a"] }])),
        ),
        (
            "var setSomething; export { setSomething };",
            Some(serde_json::json!([{ "restrictedNamedExportsPattern": "^get" }])),
        ),
        // (
        //     "var foo, bar; export { foo, bar };",
        //     Some(serde_json::json!([{ "restrictedNamedExportsPattern": "^(?!foo)(?!bar).+$" }])),
        // ),
        (
            "var foobar; export default foobar;",
            Some(serde_json::json!([{ "restrictedNamedExportsPattern": "bar$" }])),
        ),
        (
            "var foobar; export default foobar;",
            Some(serde_json::json!([{ "restrictedNamedExportsPattern": "default" }])),
        ),
        (
            "export default 'default';",
            Some(serde_json::json!([{ "restrictedNamedExportsPattern": "default" }])),
        ),
        (
            "var foobar; export { foobar as default };",
            Some(serde_json::json!([{ "restrictedNamedExportsPattern": "default" }])),
        ),
        (
            "var foobar; export { foobar as 'default' };",
            Some(serde_json::json!([{ "restrictedNamedExportsPattern": "default" }])),
        ),
        (
            "export { default } from 'mod';",
            Some(serde_json::json!([{ "restrictedNamedExportsPattern": "default" }])),
        ),
        (
            "export { default as default } from 'mod';",
            Some(serde_json::json!([{ "restrictedNamedExportsPattern": "default" }])),
        ),
        (
            "export { foobar as default } from 'mod';",
            Some(serde_json::json!([{ "restrictedNamedExportsPattern": "default" }])),
        ),
        (
            "export * as default from 'mod';",
            Some(serde_json::json!([{ "restrictedNamedExportsPattern": "default" }])),
        ),
        ("export * from 'foo';", Some(serde_json::json!([{ "restrictedNamedExports": ["a"] }]))),
        ("export * from 'a';", Some(serde_json::json!([{ "restrictedNamedExports": ["a"] }]))),
        ("export default a;", Some(serde_json::json!([{ "restrictedNamedExports": ["a"] }]))),
        (
            "export default function a() {}",
            Some(serde_json::json!([{ "restrictedNamedExports": ["a"] }])),
        ),
        (
            "export default class A {}",
            Some(serde_json::json!([{ "restrictedNamedExports": ["A"] }])),
        ),
        (
            "export default (function a() {});",
            Some(serde_json::json!([{ "restrictedNamedExports": ["a"] }])),
        ),
        (
            "export default (class A {});",
            Some(serde_json::json!([{ "restrictedNamedExports": ["A"] }])),
        ),
        ("export default 1;", Some(serde_json::json!([{ "restrictedNamedExports": ["default"] }]))),
        (
            "export { default as a } from 'foo';",
            Some(serde_json::json!([{ "restrictedNamedExports": ["default"] }])),
        ),
        (
            "export default foo;",
            Some(serde_json::json!([{ "restrictDefaultExports": { "direct": false } }])),
        ),
        (
            "export default 42;",
            Some(serde_json::json!([{ "restrictDefaultExports": { "direct": false } }])),
        ),
        (
            "export default function foo() {}",
            Some(serde_json::json!([{ "restrictDefaultExports": { "direct": false } }])),
        ),
        (
            "const foo = 123;
            export { foo as default };",
            Some(serde_json::json!([{ "restrictDefaultExports": { "named": false } }])),
        ),
        (
            "export { default } from 'mod';",
            Some(serde_json::json!([{ "restrictDefaultExports": { "defaultFrom": false } }])),
        ),
        (
            "export { default as default } from 'mod';",
            Some(serde_json::json!([{ "restrictDefaultExports": { "defaultFrom": false } }])),
        ),
        (
            "export { foo as default } from 'mod';",
            Some(serde_json::json!([{ "restrictDefaultExports": { "defaultFrom": true } }])),
        ),
        (
            "export { default } from 'mod';",
            Some(
                serde_json::json!([ { "restrictDefaultExports": { "named": true, "defaultFrom": false } }, ]),
            ),
        ),
        (
            "export { 'default' } from 'mod'; ",
            Some(serde_json::json!([{ "restrictDefaultExports": { "defaultFrom": false } }])),
        ),
        (
            "export { foo as default } from 'mod';",
            Some(serde_json::json!([{ "restrictDefaultExports": { "namedFrom": false } }])),
        ),
        (
            "export { default as default } from 'mod';",
            Some(serde_json::json!([{ "restrictDefaultExports": { "namedFrom": true } }])),
        ),
        (
            "export { default as default } from 'mod';",
            Some(serde_json::json!([{ "restrictDefaultExports": { "namedFrom": false } }])),
        ),
        (
            "export { 'default' } from 'mod'; ",
            Some(
                serde_json::json!([ { "restrictDefaultExports": { "defaultFrom": false, "namedFrom": true, }, }, ]),
            ),
        ),
        (
            "export * as default from 'mod';",
            Some(serde_json::json!([{ "restrictDefaultExports": { "namespaceFrom": false } }])),
        ),
    ];

    let fail = vec![
        (
            "export function someFunction() {}",
            Some(serde_json::json!([{ "restrictedNamedExports": ["someFunction"] }])),
        ),
        ("export var a;", Some(serde_json::json!([{ "restrictedNamedExports": ["a"] }]))),
        ("export var a = 1;", Some(serde_json::json!([{ "restrictedNamedExports": ["a"] }]))),
        ("export let a;", Some(serde_json::json!([{ "restrictedNamedExports": ["a"] }]))),
        ("export let a = 1;", Some(serde_json::json!([{ "restrictedNamedExports": ["a"] }]))),
        ("export const a = 1;", Some(serde_json::json!([{ "restrictedNamedExports": ["a"] }]))),
        ("export function a() {}", Some(serde_json::json!([{ "restrictedNamedExports": ["a"] }]))),
        ("export function *a() {}", Some(serde_json::json!([{ "restrictedNamedExports": ["a"] }]))),
        (
            "export async function a() {}",
            Some(serde_json::json!([{ "restrictedNamedExports": ["a"] }])),
        ),
        (
            "export async function *a() {}",
            Some(serde_json::json!([{ "restrictedNamedExports": ["a"] }])),
        ),
        ("export class A {}", Some(serde_json::json!([{ "restrictedNamedExports": ["A"] }]))),
        ("let a; export { a };", Some(serde_json::json!([{ "restrictedNamedExports": ["a"] }]))),
        ("export { a }; var a;", Some(serde_json::json!([{ "restrictedNamedExports": ["a"] }]))),
        (
            "let b; export { b as a };",
            Some(serde_json::json!([{ "restrictedNamedExports": ["a"] }])),
        ),
        (
            "export { a } from 'foo';",
            Some(serde_json::json!([{ "restrictedNamedExports": ["a"] }])),
        ),
        (
            "export { b as a } from 'foo';",
            Some(serde_json::json!([{ "restrictedNamedExports": ["a"] }])),
        ),
        (
            "let a; export { a as 'a' };",
            Some(serde_json::json!([{ "restrictedNamedExports": ["a"] }])),
        ),
        (
            "let a; export { a as 'b' };",
            Some(serde_json::json!([{ "restrictedNamedExports": ["b"] }])),
        ),
        (
            "let a; export { a as ' b ' };",
            Some(serde_json::json!([{ "restrictedNamedExports": [" b "] }])),
        ),
        (
            "let a; export { a as '👍' };",
            Some(serde_json::json!([{ "restrictedNamedExports": ["👍"] }])),
        ),
        (
            "export { 'a' } from 'foo';",
            Some(serde_json::json!([{ "restrictedNamedExports": ["a"] }])),
        ),
        (
            "export { '' } from 'foo';",
            Some(serde_json::json!([{ "restrictedNamedExports": [""] }])),
        ),
        (
            "export { ' ' } from 'foo';",
            Some(serde_json::json!([{ "restrictedNamedExports": [" "] }])),
        ),
        (
            "export { b as 'a' } from 'foo';",
            Some(serde_json::json!([{ "restrictedNamedExports": ["a"] }])),
        ),
        (
            r"export { b as '\u0061' } from 'foo';",
            Some(serde_json::json!([{ "restrictedNamedExports": ["a"] }])),
        ),
        (
            "export * as 'a' from 'foo';",
            Some(serde_json::json!([{ "restrictedNamedExports": ["a"] }])),
        ),
        ("export var [a] = [];", Some(serde_json::json!([{ "restrictedNamedExports": ["a"] }]))),
        ("export let { a } = {};", Some(serde_json::json!([{ "restrictedNamedExports": ["a"] }]))),
        (
            "export const { b: a } = {};",
            Some(serde_json::json!([{ "restrictedNamedExports": ["a"] }])),
        ),
        (
            "export var [{ a }] = [];",
            Some(serde_json::json!([{ "restrictedNamedExports": ["a"] }])),
        ),
        (
            "export let { b: { c: a = d } = e } = {};",
            Some(serde_json::json!([{ "restrictedNamedExports": ["a"] }])),
        ),
        ("var a; export var a;", Some(serde_json::json!([{ "restrictedNamedExports": ["a"] }]))),
        ("export var a; var a;", Some(serde_json::json!([{ "restrictedNamedExports": ["a"] }]))),
        ("export var a = a;", Some(serde_json::json!([{ "restrictedNamedExports": ["a"] }]))),
        ("export let b = a, a;", Some(serde_json::json!([{ "restrictedNamedExports": ["a"] }]))),
        (
            "export const a = 1, b = a;",
            Some(serde_json::json!([{ "restrictedNamedExports": ["a"] }])),
        ),
        ("export var [a] = a;", Some(serde_json::json!([{ "restrictedNamedExports": ["a"] }]))),
        (
            "export let { a: a } = {};",
            Some(serde_json::json!([{ "restrictedNamedExports": ["a"] }])),
        ),
        (
            "export const { a: b, b: a } = {};",
            Some(serde_json::json!([{ "restrictedNamedExports": ["a"] }])),
        ),
        (
            "export var { b: a, a: b } = {};",
            Some(serde_json::json!([{ "restrictedNamedExports": ["a"] }])),
        ),
        (
            "export let a, { a: b } = {};",
            Some(serde_json::json!([{ "restrictedNamedExports": ["a"] }])),
        ),
        (
            "export const { a: b } = {}, a = 1;",
            Some(serde_json::json!([{ "restrictedNamedExports": ["a"] }])),
        ),
        (
            "export var [a = a] = [];",
            Some(serde_json::json!([{ "restrictedNamedExports": ["a"] }])),
        ),
        (
            "export var { a: a = a } = {};",
            Some(serde_json::json!([{ "restrictedNamedExports": ["a"] }])),
        ),
        (
            "export let { a } = { a };",
            Some(serde_json::json!([{ "restrictedNamedExports": ["a"] }])),
        ),
        (
            "export function a(a) {};",
            Some(serde_json::json!([{ "restrictedNamedExports": ["a"] }])),
        ),
        (
            "export class A { A(){} };",
            Some(serde_json::json!([{ "restrictedNamedExports": ["A"] }])),
        ),
        (
            "var a; export { a as a };",
            Some(serde_json::json!([{ "restrictedNamedExports": ["a"] }])),
        ),
        (
            "let a, b; export { a as b, b as a };",
            Some(serde_json::json!([{ "restrictedNamedExports": ["a"] }])),
        ),
        (
            "const a = 1, b = 2; export { b as a, a as b };",
            Some(serde_json::json!([{ "restrictedNamedExports": ["a"] }])),
        ),
        (
            "var a; export { a as b, a };",
            Some(serde_json::json!([{ "restrictedNamedExports": ["a"] }])),
        ),
        (
            "export { a as a } from 'a';",
            Some(serde_json::json!([{ "restrictedNamedExports": ["a"] }])),
        ),
        (
            "export { a as b, b as a } from 'foo';",
            Some(serde_json::json!([{ "restrictedNamedExports": ["a"] }])),
        ),
        (
            "export { b as a, a as b } from 'foo';",
            Some(serde_json::json!([{ "restrictedNamedExports": ["a"] }])),
        ),
        ("export * as a from 'a';", Some(serde_json::json!([{ "restrictedNamedExports": ["a"] }]))),
        ("export var a, b;", Some(serde_json::json!([{ "restrictedNamedExports": ["a"] }]))),
        ("export let b, a;", Some(serde_json::json!([{ "restrictedNamedExports": ["a"] }]))),
        (
            "export const b = 1, a = 2;",
            Some(serde_json::json!([{ "restrictedNamedExports": ["a", "b"] }])),
        ),
        (
            "export var a, b, c;",
            Some(serde_json::json!([{ "restrictedNamedExports": ["a", "c"] }])),
        ),
        (
            "export let { a, b, c } = {};",
            Some(serde_json::json!([{ "restrictedNamedExports": ["b", "c"] }])),
        ),
        (
            "export const [a, b, c, d] = {};",
            Some(serde_json::json!([{ "restrictedNamedExports": ["b", "c"] }])),
        ),
        (
            "export var { a, x: b, c, d, e: y } = {}, e, f = {};",
            Some(
                serde_json::json!([ { "restrictedNamedExports": [ "foo", "a", "b", "bar", "d", "e", "baz", ], }, ]),
            ),
        ),
        (
            "var a, b; export { a, b };",
            Some(serde_json::json!([{ "restrictedNamedExports": ["a"] }])),
        ),
        (
            "let a, b; export { b, a };",
            Some(serde_json::json!([{ "restrictedNamedExports": ["a"] }])),
        ),
        (
            "const a = 1, b = 1; export { a, b };",
            Some(serde_json::json!([{ "restrictedNamedExports": ["a", "b"] }])),
        ),
        (
            "export { a, b, c }; var a, b, c;",
            Some(serde_json::json!([{ "restrictedNamedExports": ["a", "c"] }])),
        ),
        (
            "export { b as a, b } from 'foo';",
            Some(serde_json::json!([{ "restrictedNamedExports": ["a"] }])),
        ),
        (
            "export { b as a, b } from 'foo';",
            Some(serde_json::json!([{ "restrictedNamedExports": ["b"] }])),
        ),
        (
            "export { b as a, b } from 'foo';",
            Some(serde_json::json!([{ "restrictedNamedExports": ["a", "b"] }])),
        ),
        (
            "export { a, b, c, d, x as e, f, g } from 'foo';",
            Some(
                serde_json::json!([ { "restrictedNamedExports": [ "foo", "b", "bar", "d", "e", "f", "baz", ], }, ]),
            ),
        ),
        (
            "var getSomething; export { getSomething };",
            Some(serde_json::json!([{ "restrictedNamedExportsPattern": "get*" }])),
        ),
        (
            "var getSomethingFromUser; export { getSomethingFromUser };",
            Some(serde_json::json!([{ "restrictedNamedExportsPattern": "User$" }])),
        ),
        (
            "var foo, ab, xy; export { foo, ab, xy };",
            Some(serde_json::json!([{ "restrictedNamedExportsPattern": "(b|y)$" }])),
        ),
        (
            "var foo; export { foo as ab };",
            Some(serde_json::json!([{ "restrictedNamedExportsPattern": "(b|y)$" }])),
        ),
        (
            "var privateUserEmail; export { privateUserEmail };",
            Some(serde_json::json!([{ "restrictedNamedExportsPattern": "^privateUser" }])),
        ),
        // (
        //     "export const a = 1;",
        //     Some(serde_json::json!([{ "restrictedNamedExportsPattern": "^(?!foo)(?!bar).+$" }])),
        // ),
        (
            "var a; export { a as default };",
            Some(serde_json::json!([{ "restrictedNamedExports": ["default"] }])),
        ),
        (
            "export { default } from 'foo';",
            Some(serde_json::json!([{ "restrictedNamedExports": ["default"] }])),
        ),
        (
            "export default foo;",
            Some(serde_json::json!([{ "restrictDefaultExports": { "direct": true } }])),
        ),
        (
            "export default 42;",
            Some(serde_json::json!([{ "restrictDefaultExports": { "direct": true } }])),
        ),
        (
            "export default function foo() {}",
            Some(serde_json::json!([{ "restrictDefaultExports": { "direct": true } }])),
        ),
        (
            "export default foo;",
            Some(
                serde_json::json!([ { "restrictedNamedExports": ["bar"], "restrictDefaultExports": { "direct": true }, }, ]),
            ),
        ),
        (
            "const foo = 123;
            export { foo as default };",
            Some(serde_json::json!([{ "restrictDefaultExports": { "named": true } }])),
        ),
        (
            "export { default } from 'mod';",
            Some(serde_json::json!([{ "restrictDefaultExports": { "defaultFrom": true } }])),
        ),
        (
            "export { default as default } from 'mod';",
            Some(serde_json::json!([{ "restrictDefaultExports": { "defaultFrom": true } }])),
        ),
        (
            "export { 'default' } from 'mod';",
            Some(serde_json::json!([{ "restrictDefaultExports": { "defaultFrom": true } }])),
        ),
        (
            "export { foo as default } from 'mod';",
            Some(serde_json::json!([{ "restrictDefaultExports": { "namedFrom": true } }])),
        ),
        (
            "export * as default from 'mod';",
            Some(serde_json::json!([{ "restrictDefaultExports": { "namespaceFrom": true } }])),
        ),
    ];

    Tester::new(NoRestrictedExports::NAME, NoRestrictedExports::PLUGIN, pass, fail)
        .test_and_snapshot();
}
