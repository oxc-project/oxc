use oxc_ast::{AstKind, ast::TSModuleDeclarationName};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    AstNode,
    context::{ContextHost, LintContext},
    rule::{DefaultRuleConfig, Rule},
};

fn no_namespace_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("ES2015 module syntax is preferred over namespaces.")
        .with_help("Replace the namespace with an ES2015 module or use `declare module`")
        .with_label(span)
}

#[derive(Debug, Clone, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct NoNamespace {
    /// Whether to allow declare with custom TypeScript namespaces.
    ///
    /// Examples of **incorrect** code for this rule when `{ "allowDeclarations": true }`
    /// ```typescript
    /// module foo {}
    /// namespace foo {}
    /// ```
    ///
    /// Examples of **correct** code for this rule when `{ "allowDeclarations": true }`
    /// ```typescript
    /// declare module 'foo' {}
    /// declare module foo {}
    /// declare namespace foo {}
    ///
    /// declare global {
    ///   namespace foo {}
    /// }
    ///
    /// declare module foo {
    ///   namespace foo {}
    /// }
    /// ```
    ///
    /// Examples of **incorrect** code for this rule when `{ "allowDeclarations": false }`
    /// ```typescript
    /// module foo {}
    /// namespace foo {}
    /// declare module foo {}
    /// declare namespace foo {}
    /// ```
    ///
    /// Examples of **correct** code for this rule when `{ "allowDeclarations": false }`
    /// ```typescript
    /// declare module 'foo' {}
    /// ```
    allow_declarations: bool,
    /// Examples of **incorrect** code for this rule when `{ "allowDefinitionFiles": true }`
    /// ```typescript
    /// // if outside a d.ts file
    /// module foo {}
    /// namespace foo {}
    ///
    /// // if outside a d.ts file
    /// module foo {}
    /// namespace foo {}
    /// declare module foo {}
    /// declare namespace foo {}
    /// ```
    ///
    /// Examples of **correct** code for this rule when `{ "allowDefinitionFiles": true }`
    /// ```typescript
    /// declare module 'foo' {}
    /// // anything inside a d.ts file
    /// ```
    allow_definition_files: bool,
}

impl Default for NoNamespace {
    fn default() -> Self {
        Self { allow_declarations: false, allow_definition_files: true }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow TypeScript namespaces.
    ///
    /// ### Why is this bad?
    ///
    /// TypeScript historically allowed a form of code organization called "custom modules" (module Example {}),
    /// later renamed to "namespaces" (namespace Example). Namespaces are an outdated way to organize TypeScript code.
    /// ES2015 module syntax is now preferred (import/export).
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```typescript
    /// module foo {}
    /// namespace foo {}
    /// declare module foo {}
    /// declare namespace foo {}
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```typescript
    /// declare module 'foo' {}
    /// // anything inside a d.ts file
    /// ```
    NoNamespace,
    typescript,
    restriction,
    config = NoNamespace,
);

impl Rule for NoNamespace {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        Ok(serde_json::from_value::<DefaultRuleConfig<Self>>(value)
            .unwrap_or_default()
            .into_inner())
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::TSModuleDeclaration(declaration) = node.kind() else {
            return;
        };
        if !matches!(&declaration.id, TSModuleDeclarationName::Identifier(_)) {
            return;
        }

        // Ignore nested `TSModuleDeclaration`s
        // e.g. the 2 inner `TSModuleDeclaration`s in `module A.B.C {}`
        if let AstKind::TSModuleDeclaration(_) = ctx.nodes().parent_kind(node.id()) {
            return;
        }

        if self.allow_declarations
            && (declaration.declare || is_any_ancestor_declaration(node, ctx))
        {
            return;
        }

        let keyword = declaration.kind.as_str();
        let mut span_start = declaration.span.start;
        span_start += ctx.find_next_token_from(span_start, keyword).unwrap();
        #[expect(clippy::cast_possible_truncation)]
        let span = Span::sized(span_start, keyword.len() as u32);
        ctx.diagnostic(no_namespace_diagnostic(span));
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        if self.allow_definition_files && ctx.source_type().is_typescript_definition() {
            return false;
        }
        ctx.source_type().is_typescript()
    }
}

fn is_any_ancestor_declaration(node: &AstNode, ctx: &LintContext) -> bool {
    ctx.nodes().ancestors(node.id()).any(|node| match node.kind() {
        AstKind::TSModuleDeclaration(decl) => decl.declare,
        // No need to check `declare` field, as `global` is only valid in ambient context
        AstKind::TSGlobalDeclaration(_) => true,
        _ => false,
    })
}

#[test]
fn test() {
    use crate::tester::Tester;
    use std::path::PathBuf;

    let pass = vec![
        ("declare global {}", None, None, None),
        ("declare module 'foo' {}", None, None, None),
        (
            "declare module foo {}",
            Some(serde_json::json!([{ "allowDeclarations": true }])),
            None,
            None,
        ),
        (
            "declare namespace foo {}",
            Some(serde_json::json!([{ "allowDeclarations": true }])),
            None,
            None,
        ),
        (
            "declare global {
               namespace foo {}
             }",
            Some(serde_json::json!([{ "allowDeclarations": true }])),
            None,
            None,
        ),
        (
            "declare module foo {
               namespace bar {}
             }",
            Some(serde_json::json!([{ "allowDeclarations": true }])),
            None,
            None,
        ),
        (
            "declare global {
               namespace foo {
                 namespace bar {}
               }
             }",
            Some(serde_json::json!([{ "allowDeclarations": true }])),
            None,
            None,
        ),
        (
            "declare namespace foo {
               namespace bar {
                 namespace baz {}
               }
             }",
            Some(serde_json::json!([{ "allowDeclarations": true }])),
            None,
            None,
        ),
        (
            "export declare namespace foo {
               export namespace bar {
                 namespace baz {}
               }
             }",
            Some(serde_json::json!([{ "allowDeclarations": true }])),
            None,
            None,
        ),
        (
            "namespace foo {}",
            Some(serde_json::json!([{ "allowDefinitionFiles": true }])),
            None,
            Some(PathBuf::from("test.d.ts")),
        ),
        (
            "module foo {}",
            Some(serde_json::json!([{ "allowDefinitionFiles": true }])),
            None,
            Some(PathBuf::from("test.d.ts")),
        ),
    ];

    let fail = vec![
        ("module foo {}", None, None, None),
        ("namespace foo {}", None, None, None),
        ("module foo {}", Some(serde_json::json!([{ "allowDeclarations": false }])), None, None),
        ("namespace foo {}", Some(serde_json::json!([{ "allowDeclarations": false }])), None, None),
        ("module foo {}", Some(serde_json::json!([{ "allowDeclarations": true }])), None, None),
        ("namespace foo {}", Some(serde_json::json!([{ "allowDeclarations": true }])), None, None),
        ("declare module foo {}", None, None, None),
        ("declare namespace foo {}", None, None, None),
        (
            "declare module foo {}",
            Some(serde_json::json!([{ "allowDeclarations": false }])),
            None,
            None,
        ),
        (
            "declare namespace foo {}",
            Some(serde_json::json!([{ "allowDeclarations": false }])),
            None,
            None,
        ),
        (
            "namespace foo {}",
            Some(serde_json::json!([{ "allowDefinitionFiles": false }])),
            None,
            Some(PathBuf::from("test.d.ts")),
        ),
        (
            "module foo {}",
            Some(serde_json::json!([{ "allowDefinitionFiles": false }])),
            None,
            Some(PathBuf::from("test.d.ts")),
        ),
        (
            "declare module foo {}",
            Some(serde_json::json!([{ "allowDefinitionFiles": false }])),
            None,
            Some(PathBuf::from("test.d.ts")),
        ),
        (
            "declare namespace foo {}",
            Some(serde_json::json!([{ "allowDefinitionFiles": false }])),
            None,
            Some(PathBuf::from("test.d.ts")),
        ),
        (
            "namespace Foo.Bar {}",
            Some(serde_json::json!([{ "allowDeclarations": false }])),
            None,
            None,
        ),
        (
            "namespace Foo.Bar {
               namespace Baz.Bas {
                 interface X {}
               }
             }",
            None,
            None,
            None,
        ),
        (
            "namespace A {
               namespace B {
                 declare namespace C {}
               }
             }",
            Some(serde_json::json!([{ "allowDeclarations": true }])),
            None,
            None,
        ),
        (
            "namespace A {
               namespace B {
                 export declare namespace C {}
               }
             }",
            Some(serde_json::json!([{ "allowDeclarations": true }])),
            None,
            None,
        ),
        (
            "namespace A {
               declare namespace B {
                 namespace C {}
               }
             }",
            Some(serde_json::json!([{ "allowDeclarations": true }])),
            None,
            None,
        ),
        (
            "namespace A {
               export declare namespace B {
                 namespace C {}
               }
             }",
            Some(serde_json::json!([{ "allowDeclarations": true }])),
            None,
            None,
        ),
        (
            "namespace A {
               export declare namespace B {
                 declare namespace C {}
               }
             }",
            Some(serde_json::json!([{ "allowDeclarations": true }])),
            None,
            None,
        ),
        (
            "namespace A {
               export declare namespace B {
                 export declare namespace C {}
               }
             }",
            Some(serde_json::json!([{ "allowDeclarations": true }])),
            None,
            None,
        ),
        (
            "namespace A {
               declare namespace B {
                 export declare namespace C {}
               }
             }",
            Some(serde_json::json!([{ "allowDeclarations": true }])),
            None,
            None,
        ),
        (
            "namespace A {
               export namespace B {
                 export declare namespace C {}
               }
              }",
            Some(serde_json::json!([{ "allowDeclarations": true }])),
            None,
            None,
        ),
        (
            "export namespace A {
               namespace B {
                 declare namespace C {}
               }
             }",
            Some(serde_json::json!([{ "allowDeclarations": true }])),
            None,
            None,
        ),
        (
            "export namespace A {
               namespace B {
                 export declare namespace C {}
               }
             }",
            Some(serde_json::json!([{ "allowDeclarations": true }])),
            None,
            None,
        ),
        (
            "export namespace A {
               declare namespace B {
                 namespace C {}
               }
              }",
            Some(serde_json::json!([{ "allowDeclarations": true }])),
            None,
            None,
        ),
        (
            "export namespace A {
               export declare namespace B {
                 namespace C {}
               }
             }",
            Some(serde_json::json!([{ "allowDeclarations": true }])),
            None,
            None,
        ),
        (
            "export namespace A {
               export declare namespace B {
                 declare namespace C {}
               }
             }",
            Some(serde_json::json!([{ "allowDeclarations": true }])),
            None,
            None,
        ),
        (
            "export namespace A {
               export declare namespace B {
                 export declare namespace C {}
               }
             }",
            Some(serde_json::json!([{ "allowDeclarations": true }])),
            None,
            None,
        ),
        (
            "export namespace A {
               declare namespace B {
                 export declare namespace C {}
               }
             }",
            Some(serde_json::json!([{ "allowDeclarations": true }])),
            None,
            None,
        ),
        (
            "export namespace A {
               export namespace B {
                 export declare namespace C {}
               }
             }",
            Some(serde_json::json!([{ "allowDeclarations": true }])),
            None,
            None,
        ),
        ("declare /* module */ module foo {}", None, None, None),
        ("declare /* namespace */ namespace foo {}", None, None, None),
    ];

    Tester::new(NoNamespace::NAME, NoNamespace::PLUGIN, pass, fail).test_and_snapshot();
}
