use oxc_ast::{AstKind, ast::TSModuleDeclarationName};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use schemars::JsonSchema;

use crate::{
    AstNode,
    context::{ContextHost, LintContext},
    rule::Rule,
};

fn no_namespace_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("ES2015 module syntax is preferred over namespaces.")
        .with_help("Replace the namespace with an ES2015 module or use `declare module`")
        .with_label(span)
}

#[derive(Debug, Clone, JsonSchema)]
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
    fn from_configuration(value: serde_json::Value) -> Self {
        Self {
            allow_declarations: value
                .get(0)
                .and_then(|x| x.get("allowDeclarations"))
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false),
            allow_definition_files: value
                .get(0)
                .and_then(|x| x.get("allowDefinitionFiles"))
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(true),
        }
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

    let pass = vec![
        ("declare global {}", None),
        ("declare module 'foo' {}", None),
        ("declare module foo {}", Some(serde_json::json!([{ "allowDeclarations": true }]))),
        ("declare namespace foo {}", Some(serde_json::json!([{ "allowDeclarations": true }]))),
        (
            "declare global {
    		   namespace foo {}
    		 }",
            Some(serde_json::json!([{ "allowDeclarations": true }])),
        ),
        (
            "declare module foo {
    		   namespace bar {}
    		 }",
            Some(serde_json::json!([{ "allowDeclarations": true }])),
        ),
        (
            "declare global {
    		   namespace foo {
    		     namespace bar {}
    		   }
    		 }",
            Some(serde_json::json!([{ "allowDeclarations": true }])),
        ),
        (
            "declare namespace foo {
    		   namespace bar {
    		     namespace baz {}
    		   }
    		 }",
            Some(serde_json::json!([{ "allowDeclarations": true }])),
        ),
        (
            "export declare namespace foo {
    		   export namespace bar {
    		     namespace baz {}
    		   }
    		 }",
            Some(serde_json::json!([{ "allowDeclarations": true }])),
        ),
    ];

    let fail = vec![
        ("module foo {}", None),
        ("namespace foo {}", None),
        ("module foo {}", Some(serde_json::json!([{ "allowDeclarations": false }]))),
        ("namespace foo {}", Some(serde_json::json!([{ "allowDeclarations": false }]))),
        ("module foo {}", Some(serde_json::json!([{ "allowDeclarations": true }]))),
        ("namespace foo {}", Some(serde_json::json!([{ "allowDeclarations": true }]))),
        ("declare module foo {}", None),
        ("declare namespace foo {}", None),
        ("declare module foo {}", Some(serde_json::json!([{ "allowDeclarations": false }]))),
        ("declare namespace foo {}", Some(serde_json::json!([{ "allowDeclarations": false }]))),
        ("namespace Foo.Bar {}", Some(serde_json::json!([{ "allowDeclarations": false }]))),
        (
            "namespace Foo.Bar {
    		   namespace Baz.Bas {
    		     interface X {}
    		   }
    		 }",
            None,
        ),
        (
            "namespace A {
    		   namespace B {
    		     declare namespace C {}
    		   }
    		 }",
            Some(serde_json::json!([{ "allowDeclarations": true }])),
        ),
        (
            "namespace A {
    		   namespace B {
    		     export declare namespace C {}
    		   }
    		 }",
            Some(serde_json::json!([{ "allowDeclarations": true }])),
        ),
        (
            "namespace A {
    		   declare namespace B {
    		     namespace C {}
    		   }
    		 }",
            Some(serde_json::json!([{ "allowDeclarations": true }])),
        ),
        (
            "namespace A {
    		   export declare namespace B {
    		     namespace C {}
    		   }
    		 }",
            Some(serde_json::json!([{ "allowDeclarations": true }])),
        ),
        (
            "namespace A {
    		   export declare namespace B {
    		     declare namespace C {}
    		   }
    		 }",
            Some(serde_json::json!([{ "allowDeclarations": true }])),
        ),
        (
            "namespace A {
    		   export declare namespace B {
    		     export declare namespace C {}
    		   }
    		 }",
            Some(serde_json::json!([{ "allowDeclarations": true }])),
        ),
        (
            "namespace A {
    		   declare namespace B {
    		     export declare namespace C {}
    		   }
    		 }",
            Some(serde_json::json!([{ "allowDeclarations": true }])),
        ),
        (
            "namespace A {
    		   export namespace B {
    		     export declare namespace C {}
    		   }
    	 	 }",
            Some(serde_json::json!([{ "allowDeclarations": true }])),
        ),
        (
            "export namespace A {
    		   namespace B {
    		     declare namespace C {}
    		   }
    		 }",
            Some(serde_json::json!([{ "allowDeclarations": true }])),
        ),
        (
            "export namespace A {
    		   namespace B {
    		     export declare namespace C {}
    		   }
    		 }",
            Some(serde_json::json!([{ "allowDeclarations": true }])),
        ),
        (
            "export namespace A {
    		   declare namespace B {
    		     namespace C {}
    		   }
    	 	 }",
            Some(serde_json::json!([{ "allowDeclarations": true }])),
        ),
        (
            "export namespace A {
    		   export declare namespace B {
    		     namespace C {}
    		   }
    		 }",
            Some(serde_json::json!([{ "allowDeclarations": true }])),
        ),
        (
            "export namespace A {
    		   export declare namespace B {
    		     declare namespace C {}
    		   }
    		 }",
            Some(serde_json::json!([{ "allowDeclarations": true }])),
        ),
        (
            "export namespace A {
    		   export declare namespace B {
    		     export declare namespace C {}
    		   }
    		 }",
            Some(serde_json::json!([{ "allowDeclarations": true }])),
        ),
        (
            "export namespace A {
    		   declare namespace B {
    		     export declare namespace C {}
    		   }
    	 	}",
            Some(serde_json::json!([{ "allowDeclarations": true }])),
        ),
        (
            "export namespace A {
    		   export namespace B {
    		     export declare namespace C {}
    		   }
    		 }",
            Some(serde_json::json!([{ "allowDeclarations": true }])),
        ),
        ("declare /* module */ module foo {}", None),
        ("declare /* namespace */ namespace foo {}", None),
    ];

    Tester::new(NoNamespace::NAME, NoNamespace::PLUGIN, pass, fail).test_and_snapshot();
}
