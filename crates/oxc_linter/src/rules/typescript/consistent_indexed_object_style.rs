use oxc_ast::{
    AstKind,
    ast::{TSSignature, TSType, TSTypeName},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    AstNode,
    context::{ContextHost, LintContext},
    rule::Rule,
};

fn consistent_indexed_object_style_diagnostic(a: &str, b: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("A {a} is preferred over an {b}."))
        .with_help(format!("A {a} is preferred over an {b}."))
        .with_label(span)
}

#[derive(Debug, Clone)]
pub struct ConsistentIndexedObjectStyle {
    is_record_mode: bool,
}

impl Default for ConsistentIndexedObjectStyle {
    fn default() -> Self {
        Self { is_record_mode: true }
    }
}

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
enum ConsistentIndexedObjectStyleConfig {
    #[default]
    Record,
    IndexSignature,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Choose between requiring either `Record` type or indexed signature types.
    ///
    /// ### Why is this bad?
    ///
    /// Inconsistent style for indexed object types can harm readability in a project.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule with the default "record":
    /// ```ts
    /// /*eslint consistent-indexed-object-style: ["error", "record"]*/
    ///
    /// interface Foo {
    ///  [key: string]: unknown;
    /// }
    /// type Foo = {
    ///  [key: string]: unknown;
    /// };
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// /*eslint consistent-indexed-object-style: ["error", "record"]*/
    ///
    /// type Foo = Record<string, unknown>;
    /// ```
    ///
    /// Examples of **incorrect** code for this rule with "index-signature":
    /// ```ts
    /// /*eslint consistent-indexed-object-style: ["error", "index-signature"]*/
    ///
    /// type Foo = Record<string, unknown>;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// /*eslint consistent-indexed-object-style: ["error", "index-signature"]*/
    ///
    /// interface Foo {
    ///  [key: string]: unknown;
    /// }
    /// type Foo = {
    ///  [key: string]: unknown;
    /// };
    /// ```
    ConsistentIndexedObjectStyle,
    typescript,
    style,
    conditional_fix
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

                    match &sig.type_annotation.type_annotation {
                        TSType::TSTypeReference(r) => match &r.type_name {
                            TSTypeName::IdentifierReference(ide) => {
                                if ide.name != inf.id.name {
                                    ctx.diagnostic(consistent_indexed_object_style_diagnostic(
                                        "record",
                                        "index signature",
                                        sig.span,
                                    ));
                                }
                            }
                            TSTypeName::QualifiedName(_) | TSTypeName::ThisExpression(_) => {
                                ctx.diagnostic(consistent_indexed_object_style_diagnostic(
                                    "record",
                                    "index signature",
                                    sig.span,
                                ));
                            }
                        },
                        TSType::TSUnionType(uni) => {
                            for t in &uni.types {
                                if let TSType::TSTypeReference(tref) = t {
                                    if let TSTypeName::IdentifierReference(ide) = &tref.type_name {
                                        let AstKind::TSTypeAliasDeclaration(dec) =
                                            ctx.nodes().parent_kind(node.id())
                                        else {
                                            return;
                                        };

                                        if dec.id.name != ide.name {
                                            ctx.diagnostic(
                                                consistent_indexed_object_style_diagnostic(
                                                    "record",
                                                    "index signature",
                                                    sig.span,
                                                ),
                                            );
                                        }
                                    }
                                }
                            }
                        }
                        _ => {
                            ctx.diagnostic(consistent_indexed_object_style_diagnostic(
                                "record",
                                "index signature",
                                sig.span,
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

                    match &sig.type_annotation.type_annotation {
                        TSType::TSTypeReference(r) => match &r.type_name {
                            TSTypeName::IdentifierReference(ide) => {
                                let AstKind::TSTypeAliasDeclaration(dec) =
                                    ctx.nodes().parent_kind(node.id())
                                else {
                                    return;
                                };

                                if ide.name != dec.id.name {
                                    ctx.diagnostic(consistent_indexed_object_style_diagnostic(
                                        "record",
                                        "index signature",
                                        sig.span,
                                    ));
                                }
                            }
                            TSTypeName::QualifiedName(_) | TSTypeName::ThisExpression(_) => {
                                ctx.diagnostic(consistent_indexed_object_style_diagnostic(
                                    "record",
                                    "index signature",
                                    sig.span,
                                ));
                            }
                        },
                        TSType::TSUnionType(uni) => {
                            for t in &uni.types {
                                if let TSType::TSTypeReference(tref) = t {
                                    if let TSTypeName::IdentifierReference(ide) = &tref.type_name {
                                        let AstKind::TSTypeAliasDeclaration(dec) =
                                            ctx.nodes().parent_kind(node.id())
                                        else {
                                            return;
                                        };

                                        if dec.id.name != ide.name {
                                            ctx.diagnostic(
                                                consistent_indexed_object_style_diagnostic(
                                                    "record",
                                                    "index signature",
                                                    sig.span,
                                                ),
                                            );
                                        }
                                    }
                                }
                            }
                        }
                        _ => {
                            ctx.diagnostic(consistent_indexed_object_style_diagnostic(
                                "record",
                                "index signature",
                                sig.span,
                            ));
                        }
                    }
                }
                _ => {}
            }
        } else if let AstKind::TSTypeReference(tref) = node.kind() {
            if let TSTypeName::IdentifierReference(ide) = &tref.type_name {
                if ide.name != "Record" {
                    return;
                }

                let Some(params) = &tref.type_arguments else { return };
                if params.params.len() != 2 {
                    return;
                }

                if let Some(TSType::TSStringKeyword(first)) =
                    &tref.type_arguments.as_ref().and_then(|params| params.params.first())
                {
                    ctx.diagnostic_with_fix(
                        consistent_indexed_object_style_diagnostic(
                            "index signature",
                            "record",
                            tref.span,
                        ),
                        |fixer| {
                            let key = fixer.source_range(first.span);
                            let params_span = Span::new(first.span.end + 1, tref.span.end - 1);
                            let params = fixer.source_range(params_span).trim();
                            let content = format!("{{ [key: {key}]: {params} }}");
                            fixer.replace(tref.span, content)
                        },
                    );
                } else {
                    ctx.diagnostic(consistent_indexed_object_style_diagnostic(
                        "index signature",
                        "record",
                        tref.span,
                    ));
                }
            }
        }
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.source_type().is_typescript()
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let fix = vec![
        (
            "type Foo = Record<string, any>;",
            "type Foo = { [key: string]: any };",
            Some(serde_json::json!(["index-signature"])),
        ),
        (
            "type Foo<T> = Record<string, T>;",
            "type Foo<T> = { [key: string]: T };",
            Some(serde_json::json!(["index-signature"])),
        ),
        (
            "export function getCookies (headers: Headers): Record<string,Østring>",
            "export function getCookies (headers: Headers): { [key: string]: Østring }",
            Some(serde_json::json!(["index-signature"])),
        ),
    ];

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
        ("type Foo = { [key: string]: Foo } | Foo;", None),
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
        ("type Foo = { [key: string]: string | Bar };", None),
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
        ("function foo(): Record<string, any> {}", Some(serde_json::json!(["index-signature"]))),
    ];

    Tester::new(
        ConsistentIndexedObjectStyle::NAME,
        ConsistentIndexedObjectStyle::PLUGIN,
        pass,
        fail,
    )
    .expect_fix(fix)
    .test_and_snapshot();
}
