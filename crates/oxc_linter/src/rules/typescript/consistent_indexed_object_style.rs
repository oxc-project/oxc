use oxc_ast::{
    ast::{TSInterfaceDeclaration, TSSignature, TSType, TSTypeName},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

fn consistent_indexed_object_style_diagnostic(a: &str, b: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "typescript-eslint(consistent-indexed-object-style):A {a} is preferred over an {b}."
    ))
    .with_help(format!("A {a} is preferred over an {b}."))
    .with_labels([span.into()])
}

#[derive(Debug, Default, Clone)]
pub struct ConsistentIndexedObjectStyle {
    is_record_mode: bool,
}

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
enum ConsistentIndexedObjectStyleConfig {
    #[default]
    Record,
    IndexSignature,
}

declare_oxc_lint!(
    /// ### What it does
    /// Require or disallow the `Record` type.
    ///
    /// ### Why is this bad?
    ///
    ///
    /// ### Example
    /// ```javascript
    /// ```
    ConsistentIndexedObjectStyle,
    style
);

impl Rule for ConsistentIndexedObjectStyle {
    fn from_configuration(value: serde_json::Value) -> Self {
        let config = value.get(0).and_then(serde_json::Value::as_str).map_or_else(
            ConsistentIndexedObjectStyleConfig::default,
            |value| match value {
                "record" => ConsistentIndexedObjectStyleConfig::Record,
                _ => ConsistentIndexedObjectStyleConfig::IndexSignature,
            },
        );
        Self { is_record_mode: config == ConsistentIndexedObjectStyleConfig::Record }
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if self.is_record_mode {
            match node.kind() {
                AstKind::TSInterfaceDeclaration(inf) => {
                    if inf.body.body.len() > 1 {
                        return;
                    }
                    let member = inf.body.body.first();
                    let Some(member) = member else {
                        return;
                    };

                    let TSSignature::TSIndexSignature(sig) = member else { return };

                    let Some(parameter) = sig.parameters.first() else { return };

                    let _key_type = &parameter.type_annotation;

                    let value_type = &sig.type_annotation.type_annotation;
                    let interface_name = &inf.id.name;

                    match value_type {
                        TSType::TSTypeReference(r) => match &r.type_name {
                            TSTypeName::IdentifierReference(ide) => {
                                if ide.name != interface_name {
                                    ctx.diagnostic(consistent_indexed_object_style_diagnostic(
                                        "record", "index 4", r.span,
                                    ));
                                }
                            }
                            _ => {
                                ctx.diagnostic(consistent_indexed_object_style_diagnostic(
                                    "record", "index 42", r.span,
                                ));
                            }
                        },
                        TSType::TSUnionType(_uni) => {}
                        _ => {
                            ctx.diagnostic(consistent_indexed_object_style_diagnostic(
                                "record", "index 4", sig.span,
                            ));
                        }
                    }
                }
                AstKind::TSTypeLiteral(lit) => {
                    if lit.members.len() > 1 {
                        return;
                    }

                    let Some(TSSignature::TSIndexSignature(sig)) = lit.members.first() else {
                        return;
                    };

                    let Some(parameter) = sig.parameters.first() else { return };

                    let _key_type = &parameter.type_annotation;

                    let value_type = &sig.type_annotation.type_annotation;

                    match value_type {
                        TSType::TSTypeReference(r) => match &r.type_name {
                            TSTypeName::IdentifierReference(ide) => {
                                let Some(parent) = ctx.nodes().parent_kind(node.id()) else {
                                    return;
                                };

                                let parent_name =
                                    if let AstKind::TSTypeAliasDeclaration(dec) = parent {
                                        &dec.id.name
                                    } else {
                                        return;
                                    };

                                if ide.name != parent_name {
                                    ctx.diagnostic(consistent_indexed_object_style_diagnostic(
                                        "record", "index 4", r.span,
                                    ));
                                }
                            }
                            _ => {
                                ctx.diagnostic(consistent_indexed_object_style_diagnostic(
                                    "record", "index 7", r.span,
                                ));
                            }
                        },
                        TSType::TSUnionType(_uni) => {}
                        _ => {
                            ctx.diagnostic(consistent_indexed_object_style_diagnostic(
                                "record", "index 4", sig.span,
                            ));
                        }
                    }
                }
                AstKind::TSTypeReference(tref) => {
                    let TSTypeName::IdentifierReference(ide) = &tref.type_name else { return };

                    if ide.name == "Record" {
                        return;
                    }

                    let interface_name = &ide.name;
                    let is_circular = ide.name == interface_name;

                    if !is_circular {
                        ctx.diagnostic(consistent_indexed_object_style_diagnostic(
                            "record", "index 2", tref.span,
                        ));
                    }
                }
                _ => {}
            }
        } else {
            match node.kind() {
                AstKind::TSTypeReference(tref) => {
                    if let TSTypeName::IdentifierReference(ide) = &tref.type_name {
                        if ide.name != "Record" {
                            return;
                        }

                        let Some(params) = &tref.type_parameters else { return };
                        if params.params.len() != 2 {
                            return;
                        }

                        ctx.diagnostic(consistent_indexed_object_style_diagnostic(
                            "record", "index 1", tref.span,
                        ));
                    }
                }
                _ => {}
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("type Foo = Record<string, any>;", None),
        ("interface Foo {}", None),
        (
            "
        	interface Foo {
        	  bar: string;
        	}
        	    ",
            None,
        ),
        (
            "
        	interface Foo {
        	  bar: string;
        	  [key: string]: any;
        	}
        	    ",
            None,
        ),
        (
            "
        	interface Foo {
        	  [key: string]: any;
        	  bar: string;
        	}
        	    ",
            None,
        ),
        ("type Foo = { [key: string]: string | Foo };", None),
        ("type Foo = { [key: string]: Foo };", None),
        // ("type Foo = { [key: string]: Foo } | Foo;", None),
        (
            "
        	interface Foo {
        	  [key: string]: Foo;
        	}
        	    ",
            None,
        ),
        (
            "
        	interface Foo<T> {
        	  [key: string]: Foo<T>;
        	}
        	    ",
            None,
        ),
        (
            "
        	interface Foo<T> {
        	  [key: string]: Foo<T> | string;
        	}
        	    ",
            None,
        ),
        ("type Foo = {};", None),
        (
            "
        	type Foo = {
        	  bar: string;
        	  [key: string]: any;
        	};
        	    ",
            None,
        ),
        (
            "
        	type Foo = {
        	  bar: string;
        	};
        	    ",
            None,
        ),
        (
            "
        	type Foo = {
        	  [key: string]: any;
        	  bar: string;
        	};
        	    ",
            None,
        ),
        (
            "
        	type Foo = Generic<{
        	  [key: string]: any;
        	  bar: string;
        	}>;
        	    ",
            None,
        ),
        ("function foo(arg: { [key: string]: any; bar: string }) {}", None),
        ("function foo(): { [key: string]: any; bar: string } {}", None),
        ("type Foo = Misc<string, unknown>;", Some(serde_json::json!(["index-signature"]))),
        ("type Foo = Record;", Some(serde_json::json!(["index-signature"]))),
        ("type Foo = { [key: string]: any };", Some(serde_json::json!(["index-signature"]))),
        (
            "type Foo = Generic<{ [key: string]: any }>;",
            Some(serde_json::json!(["index-signature"])),
        ),
        (
            "function foo(arg: { [key: string]: any }) {}",
            Some(serde_json::json!(["index-signature"])),
        ),
        ("function foo(): { [key: string]: any } {}", Some(serde_json::json!(["index-signature"]))),
        ("type T = A.B;", Some(serde_json::json!(["index-signature"]))),
    ];

    let fail = vec![
        (
            "
        	interface Foo {
        	  [key: string]: any;
        	}
        	      ",
            None,
        ),
        (
            "
        	interface Foo {
        	  readonly [key: string]: any;
        	}
        	      ",
            None,
        ),
        (
            "
        	interface Foo<A> {
        	  [key: string]: A;
        	}
        	      ",
            None,
        ),
        (
            "
        	interface Foo<A = any> {
        	  [key: string]: A;
        	}
        	      ",
            None,
        ),
        (
            "
        	interface B extends A {
        	  [index: number]: unknown;
        	}
        	      ",
            None,
        ),
        (
            "
        	interface Foo<A> {
        	  readonly [key: string]: A;
        	}
        	      ",
            None,
        ),
        (
            "
        	interface Foo<A, B> {
        	  [key: A]: B;
        	}
        	      ",
            None,
        ),
        (
            "
        	interface Foo<A, B> {
        	  readonly [key: A]: B;
        	}
        	      ",
            None,
        ),
        ("type Foo = { [key: boolean]: any };", None),
        ("type Foo = { readonly [key: string]: any };", None),
        ("type Foo = Generic<{ [key: boolean]: any }>;", None),
        ("type Foo = Generic<{ readonly [key: string]: any }>;", None),
        ("function foo(arg: { [key: string]: any }) {}", None),
        ("function foo(): { [key: string]: any } {}", None),
        ("function foo(arg: { readonly [key: string]: any }) {}", None),
        ("function foo(): { readonly [key: string]: any } {}", None),
        ("type Foo = Record<string, any>;", Some(serde_json::json!(["index-signature"]))),
        ("type Foo<T> = Record<string, T>;", Some(serde_json::json!(["index-signature"]))),
        ("type Foo = { [k: string]: A.Foo };", None),
        ("type Foo = { [key: string]: AnotherFoo };", None),
        ("type Foo = { [key: string]: { [key: string]: Foo } };", None),
        ("type Foo = { [key: string]: string } | Foo;", None),
        (
            "
        	interface Foo<T> {
        	  [k: string]: T;
        	}
        	      ",
            None,
        ),
        (
            "
        	interface Foo {
        	  [k: string]: A.Foo;
        	}
        	      ",
            None,
        ),
        (
            "
        	interface Foo {
        	  [k: string]: { [key: string]: Foo };
        	}
        	      ",
            None,
        ),
        ("type Foo = Generic<Record<string, any>>;", Some(serde_json::json!(["index-signature"]))),
        ("function foo(arg: Record<string, any>) {}", Some(serde_json::json!(["index-signature"]))),
        ("funcction foo(): Record<string, any> {}", Some(serde_json::json!(["index-signature"]))),
    ];

    Tester::new(ConsistentIndexedObjectStyle::NAME, pass, fail).test();
}
