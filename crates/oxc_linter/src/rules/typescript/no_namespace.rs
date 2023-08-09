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
#[error("ES2015 module syntax is preferred over namespaces.")]
#[diagnostic(severity(warning), help("Replace the namespace with an ES2015 module"))]
struct NoNamespaceDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoNamespace {
    allow_declarations: bool,
    allow_definition_files: bool,
}

declare_oxc_lint!(
    /// ### What it does
    /// Disallow TypeScript namespaces.
    ///
    /// ### Why is this bad?
    /// TypeScript historically allowed a form of code organization called "custom modules" (module Example {}),
    /// later renamed to "namespaces" (namespace Example). Namespaces are an outdated way to organize TypeScript code.
    /// ES2015 module syntax is now preferred (import/export).
    ///
    /// ### Example
    /// ```typescript
    /// module foo {}
    /// namespace foo {}
    /// declare module foo {}
    /// declare namespace foo {}
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

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
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

        if self.allow_declarations && is_declaration(node, ctx) {
            return;
        }

        if self.allow_definition_files && ctx.source_type().is_typescript_definition() {
            return;
        }

        ctx.diagnostic(NoNamespaceDiagnostic(declaration.span));
    }
}

fn is_declaration(node: &AstNode, ctx: &LintContext) -> bool {
    ctx.nodes().iter_parents(node.id()).any(|node| {
        let AstKind::TSModuleDeclaration(declaration) = node.kind() else { return false };
        declaration.modifiers.contains(ModifierKind::Declare)
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
            "
    		declare global {
    		  namespace foo {}
    		}
    		      ",
            Some(serde_json::json!([{ "allowDeclarations": true }])),
        ),
        (
            "
    		declare module foo {
    		  namespace bar {}
    		}
    		      ",
            Some(serde_json::json!([{ "allowDeclarations": true }])),
        ),
        (
            "
    		declare global {
    		  namespace foo {
    		    namespace bar {}
    		  }
    		}
    		      ",
            Some(serde_json::json!([{ "allowDeclarations": true }])),
        ),
        (
            "
    		declare namespace foo {
    		  namespace bar {
    		    namespace baz {}
    		  }
    		}
    		      ",
            Some(serde_json::json!([{ "allowDeclarations": true }])),
        ),
        (
            "
    		export declare namespace foo {
    		  export namespace bar {
    		    namespace baz {}
    		  }
    		}
    		      ",
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
            "
    		namespace Foo.Bar {
    		  namespace Baz.Bas {
    		    interface X {}
    		  }
    		}
    		      ",
            None,
        ),
        (
            "
    		namespace A {
    		  namespace B {
    		    declare namespace C {}
    		  }
    		}
    		      ",
            Some(serde_json::json!([{ "allowDeclarations": true }])),
        ),
        (
            "
    		namespace A {
    		  namespace B {
    		    export declare namespace C {}
    		  }
    		}
    		      ",
            Some(serde_json::json!([{ "allowDeclarations": true }])),
        ),
        (
            "
    		namespace A {
    		  declare namespace B {
    		    namespace C {}
    		  }
    		}
    		      ",
            Some(serde_json::json!([{ "allowDeclarations": true }])),
        ),
        (
            "
    		namespace A {
    		  export declare namespace B {
    		    namespace C {}
    		  }
    		}
    		      ",
            Some(serde_json::json!([{ "allowDeclarations": true }])),
        ),
        (
            "
    		namespace A {
    		  export declare namespace B {
    		    declare namespace C {}
    		  }
    		}
    		      ",
            Some(serde_json::json!([{ "allowDeclarations": true }])),
        ),
        (
            "
    		namespace A {
    		  export declare namespace B {
    		    export declare namespace C {}
    		  }
    		}
    		      ",
            Some(serde_json::json!([{ "allowDeclarations": true }])),
        ),
        (
            "
    		namespace A {
    		  declare namespace B {
    		    export declare namespace C {}
    		  }
    		}
    		      ",
            Some(serde_json::json!([{ "allowDeclarations": true }])),
        ),
        (
            "
    		namespace A {
    		  export namespace B {
    		    export declare namespace C {}
    		  }
    		}
    		      ",
            Some(serde_json::json!([{ "allowDeclarations": true }])),
        ),
        (
            "
    		export namespace A {
    		  namespace B {
    		    declare namespace C {}
    		  }
    		}
    		      ",
            Some(serde_json::json!([{ "allowDeclarations": true }])),
        ),
        (
            "
    		export namespace A {
    		  namespace B {
    		    export declare namespace C {}
    		  }
    		}
    		      ",
            Some(serde_json::json!([{ "allowDeclarations": true }])),
        ),
        (
            "
    		export namespace A {
    		  declare namespace B {
    		    namespace C {}
    		  }
    		}
    		      ",
            Some(serde_json::json!([{ "allowDeclarations": true }])),
        ),
        (
            "
    		export namespace A {
    		  export declare namespace B {
    		    namespace C {}
    		  }
    		}
    		      ",
            Some(serde_json::json!([{ "allowDeclarations": true }])),
        ),
        (
            "
    		export namespace A {
    		  export declare namespace B {
    		    declare namespace C {}
    		  }
    		}
    		      ",
            Some(serde_json::json!([{ "allowDeclarations": true }])),
        ),
        (
            "
    		export namespace A {
    		  export declare namespace B {
    		    export declare namespace C {}
    		  }
    		}
    		      ",
            Some(serde_json::json!([{ "allowDeclarations": true }])),
        ),
        (
            "
    		export namespace A {
    		  declare namespace B {
    		    export declare namespace C {}
    		  }
    		}
    		      ",
            Some(serde_json::json!([{ "allowDeclarations": true }])),
        ),
        (
            "
    		export namespace A {
    		  export namespace B {
    		    export declare namespace C {}
    		  }
    		}
    		      ",
            Some(serde_json::json!([{ "allowDeclarations": true }])),
        ),
    ];

    Tester::new(NoNamespace::NAME, pass, fail).test_and_snapshot();
}

#[test]
fn test_declarations() {
    use crate::tester::Tester;

    let pass = vec![
        ("namespace foo {}", Some(serde_json::json!([{ "allowDefinitionFiles": true }]))),
        ("module foo {}", Some(serde_json::json!([{ "allowDefinitionFiles": true }]))),
    ];

    let fail = vec![
        ("namespace foo {}", Some(serde_json::json!([{ "allowDefinitionFiles": false }]))),
        ("module foo {}", Some(serde_json::json!([{ "allowDefinitionFiles": false }]))),
        ("declare module foo {}", Some(serde_json::json!([{ "allowDefinitionFiles": false }]))),
        ("declare namespace foo {}", Some(serde_json::json!([{ "allowDefinitionFiles": false }]))),
    ];

    Tester::new(NoNamespace::NAME, pass, fail)
        .with_extension("d.ts".to_string())
        .test_and_snapshot();
}
