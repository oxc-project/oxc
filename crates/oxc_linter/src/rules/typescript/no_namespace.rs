use oxc_ast::{
    ast::{ModifierKind, TSModuleDeclarationName},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("")]
#[diagnostic(severity(warning), help(""))]
struct NoNamespaceDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoNamespace {
    allow_declarations: bool,
    allow_definition_files: bool,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    ///
    /// ### Why is this bad?
    ///
    ///
    /// ### Example
    /// ```javascript
    /// ```
    NoNamespace,
    correctness
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
                .unwrap_or(false),
        }
    }

    // TODO: Change to run on symbol
    // TODO: manually copy the test cases
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        println!("@@ SDF {:?}", node);

        let AstKind::TSModuleDeclaration(declaration) = node.kind() else { return };
        let TSModuleDeclarationName::Identifier(ident) = &declaration.id else { return };

        if ident.name == "global" {
            return;
        }

        if let Some(parent) = ctx.nodes().parent_node(node.id()) {
            if let AstKind::TSModuleDeclaration(_) = parent.kind() {
                return;
            }
        }

        if self.allow_declarations && declaration.modifiers.contains(ModifierKind::Declare) {
            return;
        }

        if self.allow_definition_files && ctx.source_type().is_typescript_definition() {
            return;
        }

        ctx.diagnostic(NoNamespaceDiagnostic(declaration.span));
    }
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
            "namespace foo {}",
            Some(serde_json::json!([{ "allowDefinitionFiles": true, "extension": "d.ts" }])),
        ),
        (
            "module foo {}",
            Some(serde_json::json!([{ "allowDefinitionFiles": true, "extension": "d.ts" }])),
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
        ("namespace foo {}", Some(serde_json::json!([{ "allowDefinitionFiles": false,  }]))),
        ("module foo {}", Some(serde_json::json!([{ "allowDefinitionFiles": false,  }]))),
        ("declare module foo {}", Some(serde_json::json!([{ "allowDefinitionFiles": false,  }]))),
        (
            "declare namespace foo {}",
            Some(serde_json::json!([{ "allowDefinitionFiles": false,  }])),
        ),
        ("namespace Foo.Bar {}", Some(serde_json::json!([{ "allowDeclarations": false }]))),
    ];

    Tester::new(NoNamespace::NAME, pass, fail).test_and_snapshot();

    // Tester::new(
    //     NoNamespace::NAME,
    //     vec![],
    //     vec![("namespace foo {}", Some(serde_json::json!([{ "allowDeclarations": true }])))],
    // )
    // .test_and_snapshot();
}
