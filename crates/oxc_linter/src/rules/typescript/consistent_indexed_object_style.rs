use oxc_ast::{
    ast::{Statement, TSSignature, TSType, TSTypeName},
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
#[error("typescript-eslint(consistent-indexed-object-style):A {0:?} is preferred over an {1:?}.")]
#[diagnostic(severity(warning), help("A {0} is preferred over an {1}."))]
struct ConsistentIndexedObjectStyleDiagnostic(
    &'static str,
    &'static str,
    #[label("A {0} is preferred over an {1}.")] pub Span,
);

#[derive(Debug, Default, Clone)]
pub struct ConsistentIndexedObjectStyle {
    is_index_signature: bool,
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
        Self { is_index_signature: config == ConsistentIndexedObjectStyleConfig::IndexSignature }
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::TSInterfaceDeclaration(decl) => {
                if self.is_index_signature {
                    return;
                }

                if decl.body.body.len() != 1 {
                    return;
                }

                let TSSignature::TSIndexSignature(idx) = &decl.body.body[0] else { return };

                match &idx.type_annotation.type_annotation {
                    TSType::TSTypeLiteral(lit) => {
                        for member in &lit.members {
                            if let TSSignature::TSIndexSignature(_) = member {
                                ctx.diagnostic(ConsistentIndexedObjectStyleDiagnostic(
                                    "record",
                                    "index signature",
                                    idx.span,
                                ));
                            }
                        }
                    }
                    TSType::TSTypeReference(tref) => match &tref.type_name {
                        TSTypeName::IdentifierReference(iden) => {
                            if iden.name != decl.id.name {
                                ctx.diagnostic(ConsistentIndexedObjectStyleDiagnostic(
                                    "record",
                                    "index signature",
                                    idx.span,
                                ));
                            }
                        }
                        TSTypeName::QualifiedName(_) => {
                            ctx.diagnostic(ConsistentIndexedObjectStyleDiagnostic(
                                "record",
                                "index signature",
                                idx.span,
                            ))
                        }
                    },
                    TSType::TSUnknownKeyword(_) | TSType::TSAnyKeyword(_) => {
                        ctx.diagnostic(ConsistentIndexedObjectStyleDiagnostic(
                            "record",
                            "index signature",
                            idx.span,
                        ));
                    }
                    _ => {}
                }
            }
            AstKind::TSTypeAliasDeclaration(al) => match &al.type_annotation {
                TSType::TSUnionType(uni) => {
                    for t in &uni.types {
                        if let TSType::TSTypeLiteral(lit) = t {
                            if self.is_index_signature {
                                return;
                            }

                            if lit.members.len() != 1 {
                                return;
                            }

                            let TSSignature::TSIndexSignature(idx) = &lit.members[0] else {
                                return;
                            };

                            if let TSType::TSTypeReference(tref) =
                                &idx.type_annotation.type_annotation
                            {
                                if let TSTypeName::IdentifierReference(i) = &tref.type_name {
                                    if i.name != al.id.name {
                                        ctx.diagnostic(ConsistentIndexedObjectStyleDiagnostic(
                                            "record",
                                            "index signature",
                                            idx.span,
                                        ));
                                    }
                                }
                            } else {
                                ctx.diagnostic(ConsistentIndexedObjectStyleDiagnostic(
                                    "record",
                                    "index signature",
                                    idx.span,
                                ));
                            }
                        }
                    }
                }
                TSType::TSTypeLiteral(lit) => {
                    if lit.members.len() == 1 {
                        if let TSSignature::TSIndexSignature(sig) = &lit.members[0] {
                            match &sig.type_annotation.type_annotation {
                                TSType::TSUnionType(uni) => {
                                    for t in &uni.types {
                                        if let TSType::TSTypeReference(re) = t {
                                            if let TSTypeName::IdentifierReference(i) =
                                                &re.type_name
                                            {
                                                if i.name == al.id.name && self.is_index_signature {
                                                    ctx.diagnostic(
                                                        ConsistentIndexedObjectStyleDiagnostic(
                                                            "index signature",
                                                            "record",
                                                            re.span,
                                                        ),
                                                    );
                                                }
                                            }
                                        }
                                    }
                                }
                                TSType::TSTypeReference(tref) => match &tref.type_name {
                                    TSTypeName::IdentifierReference(i) => {
                                        if i.name != al.id.name {
                                            ctx.diagnostic(ConsistentIndexedObjectStyleDiagnostic(
                                                "record",
                                                "index signature",
                                                tref.span,
                                            ));
                                        }
                                    }
                                    TSTypeName::QualifiedName(_) => {
                                        ctx.diagnostic(ConsistentIndexedObjectStyleDiagnostic(
                                            "record",
                                            "index signature",
                                            sig.span,
                                        ));
                                    }
                                },
                                _ => {
                                    if !self.is_index_signature {
                                        ctx.diagnostic(ConsistentIndexedObjectStyleDiagnostic(
                                            "record",
                                            "index signature",
                                            sig.span,
                                        ));
                                    }
                                }
                            }
                        }
                    }
                }
                TSType::TSTypeReference(decl) => {
                    if let TSTypeName::IdentifierReference(iden) = &decl.type_name {
                        if iden.name == "Record"
                            && decl.type_parameters.is_some()
                            && self.is_index_signature
                        {
                            ctx.diagnostic(ConsistentIndexedObjectStyleDiagnostic(
                                "index signature",
                                "record",
                                decl.span,
                            ));
                        }
                    }

                    for param in &decl.type_parameters {
                        for p in &param.params {
                            if !self.is_index_signature {
                                if let TSType::TSTypeLiteral(lit) = p {
                                    if lit.members.len() == 1 {
                                        if let TSSignature::TSIndexSignature(idx) = &lit.members[0]
                                        {
                                            ctx.diagnostic(ConsistentIndexedObjectStyleDiagnostic(
                                                "record",
                                                "index signature",
                                                idx.span,
                                            ));
                                        }
                                    }
                                }
                            }

                            if self.is_index_signature {
                                if let TSType::TSTypeReference(r) = p {
                                    if let TSTypeName::IdentifierReference(iden) = &r.type_name {
                                        if iden.name == "Record" {
                                            ctx.diagnostic(ConsistentIndexedObjectStyleDiagnostic(
                                                "index signature",
                                                "record",
                                                r.span,
                                            ));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                _ => (),
            },
            AstKind::Program(prog) => {
                for body in &prog.body {
                    if let Statement::FunctionDeclaration(func) = body {
                        if let Some(return_type) = &func.return_type {
                            if !self.is_index_signature {
                                if let TSType::TSTypeLiteral(lit) = &return_type.type_annotation {
                                    if lit.members.len() == 1 {
                                        if let TSSignature::TSIndexSignature(sig) = &lit.members[0]
                                        {
                                            ctx.diagnostic(ConsistentIndexedObjectStyleDiagnostic(
                                                "record",
                                                "index signature",
                                                sig.span,
                                            ));
                                        }
                                    }
                                }
                            }

                            if self.is_index_signature {
                                if let TSType::TSTypeReference(tref) = &return_type.type_annotation
                                {
                                    if let TSTypeName::IdentifierReference(r) = &tref.type_name {
                                        if r.name == "Record" {
                                            ctx.diagnostic(ConsistentIndexedObjectStyleDiagnostic(
                                                "index signature",
                                                "record",
                                                tref.span,
                                            ));
                                        }
                                    }
                                }
                            }
                        }

                        for param in &func.params.items {
                            if let Some(ts_type_annotation) = &param.pattern.type_annotation {
                                if !self.is_index_signature {
                                    if let TSType::TSTypeLiteral(lit) =
                                        &ts_type_annotation.type_annotation
                                    {
                                        if lit.members.len() == 1 {
                                            if let TSSignature::TSIndexSignature(sig) =
                                                &lit.members[0]
                                            {
                                                ctx.diagnostic(
                                                    ConsistentIndexedObjectStyleDiagnostic(
                                                        "record",
                                                        "index signature",
                                                        sig.span,
                                                    ),
                                                );
                                            }
                                        }
                                    }
                                }

                                if self.is_index_signature {
                                    if let TSType::TSTypeReference(tref) =
                                        &ts_type_annotation.type_annotation
                                    {
                                        if let TSTypeName::IdentifierReference(r) = &tref.type_name
                                        {
                                            if r.name == "Record" {
                                                ctx.diagnostic(
                                                    ConsistentIndexedObjectStyleDiagnostic(
                                                        "index signature",
                                                        "record",
                                                        tref.span,
                                                    ),
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
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

    Tester::new(ConsistentIndexedObjectStyle::NAME, pass, fail).test_and_snapshot();
}
