use oxc_ast::{
    AstKind,
    ast::{TSMappedType, TSSignature, TSType, TSTypeName},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use rustc_hash::FxHashSet;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    AstNode,
    context::{ContextHost, LintContext},
    rule::{DefaultRuleConfig, Rule},
};

fn consistent_indexed_object_style_diagnostic(
    preferred: ConsistentIndexedObjectStyleConfig,
    span: Span,
) -> OxcDiagnostic {
    let (warning_message, help_message) = match preferred {
        ConsistentIndexedObjectStyleConfig::Record => (
            "A record is preferred over an index signature.",
            "Use a record type such as `Record<string, unknown>` instead of an index signature.",
        ),
        ConsistentIndexedObjectStyleConfig::IndexSignature => (
            "An index signature is preferred over a record.",
            "Use an index signature such as `{ [key: string]: unknown }` instead of a record type.",
        ),
    };

    OxcDiagnostic::warn(warning_message).with_help(help_message).with_label(span)
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ConsistentIndexedObjectStyle(ConsistentIndexedObjectStyleConfig);

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
enum ConsistentIndexedObjectStyleConfig {
    /// When set to `record`, enforces the use of a `Record` for indexed object types, e.g. `Record<string, unknown>`.
    #[default]
    Record,
    /// When set to `index-signature`, enforces the use of indexed signature types, e.g. `{ [key: string]: unknown }`.
    IndexSignature,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Choose between requiring either `Record` type or indexed signature types.
    ///
    /// These two types are equivalent, this rule enforces consistency in picking one style over the other:
    ///
    /// ```ts
    /// type Foo = Record<string, unknown>;
    ///
    /// type Foo = {
    ///   [key: string]: unknown;
    /// }
    /// ```
    ///
    /// ### Why is this bad?
    ///
    /// Inconsistent style for indexed object types can harm readability in a project.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule with
    /// `consistent-indexed-object-style: ["error", "record"]` (default):
    ///
    /// ```ts
    /// interface Foo {
    ///   [key: string]: unknown;
    /// }
    /// type Foo = {
    ///   [key: string]: unknown;
    /// };
    /// ```
    ///
    /// Examples of **correct** code for this rule with
    /// `consistent-indexed-object-style: ["error", "record"]` (default):
    /// ```ts
    /// type Foo = Record<string, unknown>;
    /// ```
    ///
    /// Examples of **incorrect** code for this rule with
    /// `consistent-indexed-object-style: ["error", "index-signature"]`:
    /// ```ts
    /// type Foo = Record<string, unknown>;
    /// ```
    ///
    /// Examples of **correct** code for this rule with
    /// `consistent-indexed-object-style: ["error", "index-signature"]`:
    /// ```ts
    /// interface Foo {
    ///   [key: string]: unknown;
    /// }
    /// type Foo = {
    ///   [key: string]: unknown;
    /// };
    /// ```
    ConsistentIndexedObjectStyle,
    typescript,
    style,
    conditional_fix,
    config = ConsistentIndexedObjectStyleConfig,
);

impl Rule for ConsistentIndexedObjectStyle {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let preferred_style = self.0;

        if self.0 == ConsistentIndexedObjectStyleConfig::Record {
            match node.kind() {
                AstKind::TSInterfaceDeclaration(inf) => {
                    if inf.body.body.len() != 1 {
                        return;
                    }
                    let Some(TSSignature::TSIndexSignature(sig)) = inf.body.body.first() else {
                        return;
                    };

                    let parent_name = inf.id.name.as_str();

                    if contains_convertible_index_signature(&sig.type_annotation.type_annotation) {
                        return;
                    }

                    let should_report = match &sig.type_annotation.type_annotation {
                        TSType::TSTypeReference(r) => match &r.type_name {
                            TSTypeName::IdentifierReference(ide) => {
                                ide.name != parent_name
                                    && !is_circular_reference(
                                        &sig.type_annotation.type_annotation,
                                        parent_name,
                                        ctx,
                                    )
                            }
                            TSTypeName::QualifiedName(_) | TSTypeName::ThisExpression(_) => true,
                        },
                        TSType::TSUnionType(uni) => !uni.types.iter().any(|t| {
                            if let TSType::TSTypeReference(tref) = t
                                && let TSTypeName::IdentifierReference(ide) = &tref.type_name
                            {
                                ide.name == parent_name
                                    || is_circular_reference(t, parent_name, ctx)
                            } else {
                                false
                            }
                        }),
                        _ => !is_circular_reference(
                            &sig.type_annotation.type_annotation,
                            parent_name,
                            ctx,
                        ),
                    };

                    if should_report {
                        ctx.diagnostic_with_fix(
                            consistent_indexed_object_style_diagnostic(preferred_style, sig.span),
                            |fixer| {
                                if matches!(
                                    ctx.nodes().parent_kind(node.id()),
                                    AstKind::ExportDefaultDeclaration(_)
                                ) {
                                    return fixer.noop();
                                }

                                let key_type = sig.parameters.first().map_or("string", |p| {
                                    fixer.source_range(p.type_annotation.type_annotation.span())
                                });
                                let value_type =
                                    fixer.source_range(sig.type_annotation.type_annotation.span());
                                let type_params = inf
                                    .type_parameters
                                    .as_ref()
                                    .map_or("", |tp| fixer.source_range(tp.span));

                                let record = if sig.readonly {
                                    format!("Readonly<Record<{key_type}, {value_type}>>")
                                } else {
                                    format!("Record<{key_type}, {value_type}>")
                                };

                                let replacement =
                                    format!("type {}{type_params} = {record};", inf.id.name);
                                fixer.replace(inf.span, replacement)
                            },
                        );
                    }
                }
                AstKind::TSTypeLiteral(lit) => {
                    if lit.members.len() != 1 {
                        return;
                    }

                    let Some(TSSignature::TSIndexSignature(sig)) = lit.members.first() else {
                        return;
                    };

                    if contains_convertible_index_signature(&sig.type_annotation.type_annotation) {
                        return;
                    }

                    let is_nested_in_type_literal = ctx
                        .nodes()
                        .ancestors(node.id())
                        .any(|ancestor| matches!(ancestor.kind(), AstKind::TSTypeLiteral(_)));

                    let parent_name = if is_nested_in_type_literal {
                        None
                    } else {
                        ctx.nodes().ancestors(node.id()).find_map(|ancestor| {
                            if let AstKind::TSTypeAliasDeclaration(dec) = ancestor.kind() {
                                Some(dec.id.name.as_str())
                            } else {
                                None
                            }
                        })
                    };

                    let should_report = match &sig.type_annotation.type_annotation {
                        TSType::TSTypeReference(r) => match &r.type_name {
                            TSTypeName::IdentifierReference(ide) => {
                                if let Some(parent_name) = parent_name {
                                    ide.name != parent_name
                                        && !is_circular_reference(
                                            &sig.type_annotation.type_annotation,
                                            parent_name,
                                            ctx,
                                        )
                                } else {
                                    true
                                }
                            }
                            TSTypeName::QualifiedName(_) | TSTypeName::ThisExpression(_) => true,
                        },
                        TSType::TSUnionType(uni) => {
                            if let Some(parent_name) = parent_name {
                                !uni.types.iter().any(|t| {
                                    if let TSType::TSTypeReference(tref) = t
                                        && let TSTypeName::IdentifierReference(ide) =
                                            &tref.type_name
                                    {
                                        ide.name == parent_name
                                            || is_circular_reference(t, parent_name, ctx)
                                    } else {
                                        false
                                    }
                                })
                            } else {
                                true
                            }
                        }
                        _ => {
                            if let Some(parent_name) = parent_name {
                                !is_circular_reference(
                                    &sig.type_annotation.type_annotation,
                                    parent_name,
                                    ctx,
                                )
                            } else {
                                true
                            }
                        }
                    };

                    if should_report {
                        ctx.diagnostic_with_fix(
                            consistent_indexed_object_style_diagnostic(preferred_style, sig.span),
                            |fixer| {
                                let key_type = sig.parameters.first().map_or("string", |p| {
                                    fixer.source_range(p.type_annotation.type_annotation.span())
                                });
                                let value_type =
                                    fixer.source_range(sig.type_annotation.type_annotation.span());

                                let record = if sig.readonly {
                                    format!("Readonly<Record<{key_type}, {value_type}>>")
                                } else {
                                    format!("Record<{key_type}, {value_type}>")
                                };

                                fixer.replace(lit.span, record)
                            },
                        );
                    }
                }
                AstKind::TSMappedType(mapped) => {
                    let constraint = &mapped.constraint;

                    // Bare `keyof` mapped types preserve structure and can't be converted
                    if is_bare_keyof(constraint) {
                        return;
                    }

                    // Key remapping (`as`) cannot be represented with `Record`.
                    if mapped.name_type.is_some() {
                        return;
                    }

                    // Can't convert if value type references the key parameter
                    if let Some(type_annotation) = &mapped.type_annotation
                        && mapped_type_value_references_key(mapped, type_annotation, ctx)
                    {
                        return;
                    }

                    if let AstKind::TSTypeAliasDeclaration(dec) = ctx.nodes().parent_kind(node.id())
                    {
                        if let Some(type_annotation) = &mapped.type_annotation
                            && is_circular_reference(type_annotation, dec.id.name.as_str(), ctx)
                        {
                            return;
                        }
                        if is_circular_reference(constraint, dec.id.name.as_str(), ctx) {
                            return;
                        }
                    }

                    ctx.diagnostic_with_fix(
                        consistent_indexed_object_style_diagnostic(preferred_style, mapped.span),
                        |fixer| {
                            let unwrapped_constraint = {
                                let mut current = constraint;
                                while let TSType::TSParenthesizedType(p) = current {
                                    current = &p.type_annotation;
                                }
                                current
                            };
                            let key_type = fixer.source_range(unwrapped_constraint.span());
                            let value_type = mapped
                                .type_annotation
                                .as_ref()
                                .map_or("any", |t| fixer.source_range(t.span()));

                            let is_readonly = mapped.readonly.is_some();
                            let is_optional = mapped.optional.is_some_and(|o| {
                                matches!(
                                    o,
                                    oxc_ast::ast::TSMappedTypeModifierOperator::True
                                        | oxc_ast::ast::TSMappedTypeModifierOperator::Plus
                                )
                            });
                            let is_required = mapped.optional.is_some_and(|o| {
                                matches!(o, oxc_ast::ast::TSMappedTypeModifierOperator::Minus)
                            });

                            let mut record = format!("Record<{key_type}, {value_type}>");
                            if is_required {
                                record = format!("Required<{record}>");
                            }
                            if is_optional {
                                record = format!("Partial<{record}>");
                            }
                            if is_readonly {
                                record = format!("Readonly<{record}>");
                            }

                            fixer.replace(mapped.span, record)
                        },
                    );
                }
                _ => {}
            }
        } else if let AstKind::TSTypeReference(tref) = node.kind()
            && let TSTypeName::IdentifierReference(ide) = &tref.type_name
        {
            if ide.name != "Record" {
                return;
            }

            let Some(params) = &tref.type_arguments else { return };
            if params.params.len() != 2 {
                return;
            }

            let first_param = params.params.first();
            let key_span = match first_param {
                Some(TSType::TSStringKeyword(k)) => Some(k.span),
                Some(TSType::TSNumberKeyword(k)) => Some(k.span),
                Some(TSType::TSSymbolKeyword(k)) => Some(k.span),
                _ => None,
            };

            if let Some(key_span) = key_span {
                ctx.diagnostic_with_fix(
                    consistent_indexed_object_style_diagnostic(preferred_style, tref.span),
                    |fixer| {
                        let key = fixer.source_range(key_span);
                        let params_span = Span::new(key_span.end + 1, tref.span.end - 1);
                        let params = fixer.source_range(params_span).trim();
                        let content = format!("{{ [key: {key}]: {params} }}");
                        fixer.replace(tref.span, content)
                    },
                );
            } else {
                ctx.diagnostic(consistent_indexed_object_style_diagnostic(
                    preferred_style,
                    tref.span,
                ));
            }
        }
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.source_type().is_typescript()
    }
}

fn contains_convertible_index_signature(r#type: &TSType) -> bool {
    matches!(r#type, TSType::TSTypeLiteral(lit) if lit.members.len() == 1
        && matches!(lit.members.first(), Some(TSSignature::TSIndexSignature(_))))
}

fn is_bare_keyof(r#type: &TSType) -> bool {
    matches!(r#type, TSType::TSTypeOperatorType(op)
        if matches!(op.operator, oxc_ast::ast::TSTypeOperatorOperator::Keyof))
}

fn mapped_type_value_references_key(
    mapped: &TSMappedType,
    type_annotation: &TSType,
    ctx: &LintContext,
) -> bool {
    let type_annotation_span = type_annotation.span();
    ctx.symbol_references(mapped.key.symbol_id()).any(|reference| {
        let reference_span = ctx.nodes().get_node(reference.node_id()).kind().span();
        type_annotation_span.contains_inclusive(reference_span)
    })
}

fn is_circular_reference(type_: &TSType, parent_name: &str, ctx: &LintContext) -> bool {
    is_circular_reference_impl(type_, parent_name, ctx, &mut FxHashSet::default())
}

fn is_circular_reference_impl(
    type_: &TSType,
    parent_name: &str,
    ctx: &LintContext,
    visited: &mut FxHashSet<String>,
) -> bool {
    match type_ {
        TSType::TSTypeReference(r) => {
            if let TSTypeName::IdentifierReference(ide) = &r.type_name {
                if ide.name.as_str() == parent_name {
                    return true;
                }
                if let Some(type_args) = &r.type_arguments
                    && type_args
                        .params
                        .iter()
                        .any(|t| is_circular_reference_impl(t, parent_name, ctx, visited))
                {
                    return true;
                }
                let name_str = ide.name.to_string();
                if visited.contains(&name_str) {
                    return false;
                }
                visited.insert(name_str);

                for node in ctx.nodes().iter() {
                    if let AstKind::TSTypeAliasDeclaration(dec) = node.kind()
                        && dec.id.name == ide.name
                    {
                        if let TSType::TSTypeLiteral(lit) = &dec.type_annotation
                            && lit.members.len() == 1
                            && let Some(TSSignature::TSIndexSignature(sig)) = lit.members.first()
                        {
                            return is_circular_reference_impl(
                                &sig.type_annotation.type_annotation,
                                parent_name,
                                ctx,
                                visited,
                            );
                        }
                        return is_circular_reference_impl(
                            &dec.type_annotation,
                            parent_name,
                            ctx,
                            visited,
                        );
                    }
                    if let AstKind::TSInterfaceDeclaration(dec) = node.kind()
                        && dec.id.name == ide.name
                        && dec.body.body.len() == 1
                        && let Some(TSSignature::TSIndexSignature(sig)) = dec.body.body.first()
                    {
                        return is_circular_reference_impl(
                            &sig.type_annotation.type_annotation,
                            parent_name,
                            ctx,
                            visited,
                        );
                    }
                }
            }
            false
        }
        TSType::TSUnionType(u) => {
            u.types.iter().any(|t| is_circular_reference_impl(t, parent_name, ctx, visited))
        }
        TSType::TSIntersectionType(i) => {
            i.types.iter().any(|t| is_circular_reference_impl(t, parent_name, ctx, visited))
        }
        TSType::TSConditionalType(c) => {
            is_circular_reference_impl(&c.check_type, parent_name, ctx, visited)
                || is_circular_reference_impl(&c.extends_type, parent_name, ctx, visited)
                || is_circular_reference_impl(&c.true_type, parent_name, ctx, visited)
                || is_circular_reference_impl(&c.false_type, parent_name, ctx, visited)
        }
        TSType::TSIndexedAccessType(i) => {
            is_circular_reference_impl(&i.object_type, parent_name, ctx, visited)
                || is_circular_reference_impl(&i.index_type, parent_name, ctx, visited)
        }
        _ => false,
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
        ("type Foo = { [key in string]: Foo };", None),
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
        (
            "
			interface Foo {
			  [s: string]: Foo & {};
			}
			    ",
            None,
        ),
        (
            "
			interface Foo {
			  [s: string]: Foo | string;
			}
			    ",
            None,
        ),
        (
            "
			interface Foo<T> {
			  [s: string]: Foo extends T ? string : number;
			}
			    ",
            None,
        ),
        (
            "
			interface Foo<T> {
			  [s: string]: T extends Foo ? string : number;
			}
			    ",
            None,
        ),
        (
            "
			interface Foo<T> {
			  [s: string]: T extends true ? Foo : number;
			}
			    ",
            None,
        ),
        (
            "
			interface Foo<T> {
			  [s: string]: T extends true ? string : Foo;
			}
			    ",
            None,
        ),
        (
            "
			interface Foo {
			  [s: string]: Foo[number];
			}
			    ",
            None,
        ),
        (
            "
			interface Foo {
			  [s: string]: {}[Foo];
			}
			    ",
            None,
        ),
        (
            "
			interface Foo1 {
			  [key: string]: Foo2;
			}

			interface Foo2 {
			  [key: string]: Foo1;
			}
			    ",
            None,
        ),
        (
            "
			interface Foo1 {
			  [key: string]: Foo2;
			}

			interface Foo2 {
			  [key: string]: Foo3;
			}

			interface Foo3 {
			  [key: string]: Foo1;
			}
			    ",
            None,
        ),
        (
            "
			interface Foo1 {
			  [key: string]: Foo2;
			}

			interface Foo2 {
			  [key: string]: Foo3;
			}

			interface Foo3 {
			  [key: string]: Record<string, Foo1>;
			}
			    ",
            None,
        ),
        (
            "
			type Foo1 = {
			  [key: string]: Foo2;
			};

			type Foo2 = {
			  [key: string]: Foo3;
			};

			type Foo3 = {
			  [key: string]: Foo1;
			};
			    ",
            None,
        ),
        (
            "
			interface Foo1 {
			  [key: string]: Foo2;
			}

			type Foo2 = {
			  [key: string]: Foo3;
			};

			interface Foo3 {
			  [key: string]: Foo1;
			}
			    ",
            None,
        ),
        (
            "
			type Foo1 = {
			  [key: string]: Foo2;
			};

			interface Foo2 {
			  [key: string]: Foo3;
			}

			interface Foo3 {
			  [key: string]: Foo1;
			}
			    ",
            None,
        ),
        (
            "
			type ExampleUnion = boolean | number;

			type ExampleRoot = ExampleUnion | ExampleObject;

			interface ExampleObject {
			  [key: string]: ExampleRoot;
			}
			    ",
            None,
        ),
        (
            "
			type Bar<K extends string = never> = {
			  [k in K]: Bar;
			};
			    ",
            None,
        ),
        (
            "
			type Bar<K extends string = never> = {
			  [k in K]: Foo;
			};

			type Foo = Bar;
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
        // ("type Foo = { [key: string] };", None),
        // ("type Foo = { [] };", None),
        // ("interface Foo { [key: string]; }", None),
        // ("interface Foo { []; }", None),
        ("type Foo = Misc<string, unknown>;", Some(serde_json::json!(["index-signature"]))),
        ("type Foo = Record;", Some(serde_json::json!(["index-signature"]))),
        ("type Foo = Record<string>;", Some(serde_json::json!(["index-signature"]))),
        (
            "type Foo = Record<string, number, unknown>;",
            Some(serde_json::json!(["index-signature"])),
        ),
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
        ("type T = { [key in Foo]: key | number };", None),
        (
            "
			function foo(e: { readonly [key in PropertyKey]-?: key }) {}
			      ",
            None,
        ),
        (
            "
			function f(): {
			  // intentionally not using a Record to preserve optionals
			  [k in keyof ParseResult]: unknown;
			} {
			  return {};
			}
			      ",
            None,
        ),
        (
            "interface Foo { } interface Bar { } type Baz<T extends string> = T;
             export type Error = Foo & { [P in Baz<keyof Bar>]: [P?]; };
            ",
            None,
        ),
        ("type Keys = 'A' | 'B'; type Foo = { [K in Keys]: { x: K } };", None),
        ("type Keys = 'A' | 'B'; type Foo = { [K in Keys]?: { x: K } };", None),
        (
            "type Keys = 'A' | 'B'; interface Gen<T> { a: T } type Foo = { [K in Keys]: { x: Gen<K> } };",
            None,
        ),
        (
            "type Keys = 'A' | 'B'; interface Gen<T> { a: T } type Foo = Partial<{ [K in Keys]: { x: Gen<K> } }>;",
            None,
        ),
        ("type Foo<K extends string> = { [P in K as `x_${P}`]: number };", None),
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
        ("type Foo = { [key: string]: any };", None),
        ("type Foo = { readonly [key: string]: any };", None),
        ("type Foo = Generic<{ [key: string]: any }>;", None),
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
        (
            "
			interface Foo {
			  [key: string]: { foo: Foo };
			}
			      ",
            None,
        ),
        (
            "
			interface Foo {
			  [key: string]: Foo[];
			}
			      ",
            None,
        ),
        (
            "
			interface Foo {
			  [key: string]: () => Foo;
			}
			      ",
            None,
        ),
        (
            "
			interface Foo {
			  [s: string]: [Foo];
			}
			      ",
            None,
        ),
        (
            "
			interface Foo1 {
			  [key: string]: Foo2;
			}

			interface Foo2 {
			  [key: string]: Foo3;
			}

			interface Foo3 {
			  [key: string]: Foo2;
			}
			      ",
            None,
        ),
        (
            "
			interface Foo1 {
			  [key: string]: Record<string, Foo2>;
			}

			interface Foo2 {
			  [key: string]: Foo3;
			}

			interface Foo3 {
			  [key: string]: Foo2;
			}
			      ",
            None,
        ),
        (
            "
			type Foo1 = {
			  [key: string]: { foo2: Foo2 };
			};

			type Foo2 = {
			  [key: string]: Foo3;
			};

			type Foo3 = {
			  [key: string]: Record<string, Foo1>;
			};
			      ",
            None,
        ),
        (
            "
			type Foos<K extends string = never> = {
			  [k in K]: { foo: Foo };
			};

			type Foo = Foos;
			      ",
            None,
        ),
        (
            "
			type Foos<K extends string = never> = {
			  [k in K]: Foo[];
			};

			type Foo = Foos;
			      ",
            None,
        ),
        ("type Foo = Generic<Record<string, any>>;", Some(serde_json::json!(["index-signature"]))),
        ("type Foo = Record<string | number, any>;", Some(serde_json::json!(["index-signature"]))),
        (
            "type Foo = Record<Exclude<'a' | 'b' | 'c', 'a'>, any>;",
            Some(serde_json::json!(["index-signature"])),
        ),
        ("type Foo = Record<number, any>;", Some(serde_json::json!(["index-signature"]))),
        ("type Foo = Record<symbol, any>;", Some(serde_json::json!(["index-signature"]))),
        ("function foo(arg: Record<string, any>) {}", Some(serde_json::json!(["index-signature"]))),
        ("function foo(): Record<string, any> {}", Some(serde_json::json!(["index-signature"]))),
        ("type T = { readonly [key in string]: number };", None),
        ("type T = { +readonly [key in string]: number };", None),
        ("type T = { -readonly [key in string]: number };", None),
        ("type T = { [key in string]: number };", None),
        (
            "
			function foo(e: { [key in PropertyKey]?: string }) {}
			      ",
            None,
        ),
        (
            "
			function foo(e: { [key in PropertyKey]+?: string }) {}
			      ",
            None,
        ),
        (
            "
			function foo(e: { [key in PropertyKey]-?: string }) {}
			      ",
            None,
        ),
        (
            "
			function foo(e: { readonly [key in PropertyKey]-?: string }) {}
			      ",
            None,
        ),
        (
            "
			type Options = [
			  { [Type in (typeof optionTesters)[number]['option']]?: boolean } & {
			    allow?: TypeOrValueSpecifier[];
			  },
			];
			      ",
            None,
        ),
        (
            "
			export type MakeRequired<Base, Key extends keyof Base> = {
			  [K in Key]-?: NonNullable<Base[Key]>;
			} & Omit<Base, Key>;
			      ",
            None,
        ),
        (
            "
			function f(): {
			  [k in (keyof ParseResult)]: unknown;
			} {
			  return {};
			}
			      ",
            None,
        ),
        // (
        //     "
        // 			interface Foo {
        // 			  [key: string]: Bar;
        // 			}
        //
        // 			interface Bar {
        // 			  [key: string];
        // 			}
        // 			      ",
        //     None,
        // ),
        (
            "
			type Foo = {
			  [k in string];
			};
			      ",
            None,
        ),
        // export default interface cannot be converted to export default type
        // because TypeScript doesn't allow "export default type"
        (
            "
			export default interface SchedulerService {
			  [key: string]: unknown;
			}
			      ",
            None,
        ),
    ];

    let fix = vec![
        ("
			interface Foo {
			  [key: string]: any;
			}
			      ", "
			type Foo = Record<string, any>;
			      ", None),
("
			interface Foo {
			  readonly [key: string]: any;
			}
			      ", "
			type Foo = Readonly<Record<string, any>>;
			      ", None),
("
			interface Foo<A> {
			  [key: string]: A;
			}
			      ", "
			type Foo<A> = Record<string, A>;
			      ", None),
("
			interface Foo<A = any> {
			  [key: string]: A;
			}
			      ", "
			type Foo<A = any> = Record<string, A>;
			      ", None),
("
			export interface Bar {
			  [key: string]: any;
			}
			      ", "
			export type Bar = Record<string, any>;
			      ", None),
("
			interface Foo<A> {
			  readonly [key: string]: A;
			}
			      ", "
			type Foo<A> = Readonly<Record<string, A>>;
			      ", None),
("
			interface Foo<A, B> {
			  [key: A]: B;
			}
			      ", "
			type Foo<A, B> = Record<A, B>;
			      ", None),
("
			interface Foo<A, B> {
			  readonly [key: A]: B;
			}
			      ", "
			type Foo<A, B> = Readonly<Record<A, B>>;
			      ", None),
("type Foo = { [key: string]: any };", "type Foo = Record<string, any>;", None),
("type Foo = { readonly [key: string]: any };", "type Foo = Readonly<Record<string, any>>;", None),
("type Foo = Generic<{ [key: string]: any }>;", "type Foo = Generic<Record<string, any>>;", None),
("type Foo = Generic<{ readonly [key: string]: any }>;", "type Foo = Generic<Readonly<Record<string, any>>>;", None),
("function foo(arg: { [key: string]: any }) {}", "function foo(arg: Record<string, any>) {}", None),
("function foo(): { [key: string]: any } {}", "function foo(): Record<string, any> {}", None),
("function foo(arg: { readonly [key: string]: any }) {}", "function foo(arg: Readonly<Record<string, any>>) {}", None),
("function foo(): { readonly [key: string]: any } {}", "function foo(): Readonly<Record<string, any>> {}", None),
("type Foo = Record<string, any>;", "type Foo = { [key: string]: any };", Some(serde_json::json!(["index-signature"]))),
("type Foo<T> = Record<string, T>;", "type Foo<T> = { [key: string]: T };", Some(serde_json::json!(["index-signature"]))),
("type Foo = { [k: string]: A.Foo };", "type Foo = Record<string, A.Foo>;", None),
("type Foo = { [key: string]: AnotherFoo };", "type Foo = Record<string, AnotherFoo>;", None),
("type Foo = { [key: string]: { [key: string]: Foo } };", "type Foo = { [key: string]: Record<string, Foo> };", None),
("type Foo = { [key: string]: string } | Foo;", "type Foo = Record<string, string> | Foo;", None),
("
			interface Foo<T> {
			  [k: string]: T;
			}
			      ", "
			type Foo<T> = Record<string, T>;
			      ", None),
("
			interface Foo {
			  [k: string]: A.Foo;
			}
			      ", "
			type Foo = Record<string, A.Foo>;
			      ", None),
("
			interface Foo {
			  [k: string]: { [key: string]: Foo };
			}
			      ", "
			interface Foo {
			  [k: string]: Record<string, Foo>;
			}
			      ", None),
("
			interface Foo {
			  [key: string]: { foo: Foo };
			}
			      ", "
			type Foo = Record<string, { foo: Foo }>;
			      ", None),
("
			interface Foo {
			  [key: string]: Foo[];
			}
			      ", "
			type Foo = Record<string, Foo[]>;
			      ", None),
("
			interface Foo {
			  [key: string]: () => Foo;
			}
			      ", "
			type Foo = Record<string, () => Foo>;
			      ", None),
("
			interface Foo {
			  [s: string]: [Foo];
			}
			      ", "
			type Foo = Record<string, [Foo]>;
			      ", None),
("
			interface Foo1 {
			  [key: string]: Foo2;
			}

			interface Foo2 {
			  [key: string]: Foo3;
			}

			interface Foo3 {
			  [key: string]: Foo2;
			}
			      ", "
			type Foo1 = Record<string, Foo2>;

			interface Foo2 {
			  [key: string]: Foo3;
			}

			interface Foo3 {
			  [key: string]: Foo2;
			}
			      ", None),
("
			interface Foo1 {
			  [key: string]: Record<string, Foo2>;
			}

			interface Foo2 {
			  [key: string]: Foo3;
			}

			interface Foo3 {
			  [key: string]: Foo2;
			}
			      ", "
			type Foo1 = Record<string, Record<string, Foo2>>;

			interface Foo2 {
			  [key: string]: Foo3;
			}

			interface Foo3 {
			  [key: string]: Foo2;
			}
			      ", None),
("
			type Foo1 = {
			  [key: string]: { foo2: Foo2 };
			};

			type Foo2 = {
			  [key: string]: Foo3;
			};

			type Foo3 = {
			  [key: string]: Record<string, Foo1>;
			};
			      ", "
			type Foo1 = Record<string, { foo2: Foo2 }>;

			type Foo2 = Record<string, Foo3>;

			type Foo3 = Record<string, Record<string, Foo1>>;
			      ", None),
("
			type Foos<K extends string = never> = {
			  [k in K]: { foo: Foo };
			};

			type Foo = Foos;
			      ", "
			type Foos<K extends string = never> = Record<K, { foo: Foo }>;

			type Foo = Foos;
			      ", None),
("
			type Foos<K extends string = never> = {
			  [k in K]: Foo[];
			};

			type Foo = Foos;
			      ", "
			type Foos<K extends string = never> = Record<K, Foo[]>;

			type Foo = Foos;
			      ", None),
("type Foo = Generic<Record<string, any>>;", "type Foo = Generic<{ [key: string]: any }>;", Some(serde_json::json!(["index-signature"]))),
("type Foo = Record<number, any>;", "type Foo = { [key: number]: any };", Some(serde_json::json!(["index-signature"]))),
("type Foo = Record<symbol, any>;", "type Foo = { [key: symbol]: any };", Some(serde_json::json!(["index-signature"]))),
("function foo(arg: Record<string, any>) {}", "function foo(arg: { [key: string]: any }) {}", Some(serde_json::json!(["index-signature"]))),
("function foo(): Record<string, any> {}", "function foo(): { [key: string]: any } {}", Some(serde_json::json!(["index-signature"]))),
("type T = { readonly [key in string]: number };", "type T = Readonly<Record<string, number>>;", None),
("type T = { +readonly [key in string]: number };", "type T = Readonly<Record<string, number>>;", None),
("type T = { [key in string]: number };", "type T = Record<string, number>;", None),
("
			function foo(e: { [key in PropertyKey]?: string }) {}
			      ", "
			function foo(e: Partial<Record<PropertyKey, string>>) {}
			      ", None),
("
			function foo(e: { [key in PropertyKey]+?: string }) {}
			      ", "
			function foo(e: Partial<Record<PropertyKey, string>>) {}
			      ", None),
("
			function foo(e: { [key in PropertyKey]-?: string }) {}
			      ", "
			function foo(e: Required<Record<PropertyKey, string>>) {}
			      ", None),
("
			function foo(e: { readonly [key in PropertyKey]-?: string }) {}
			      ", "
			function foo(e: Readonly<Required<Record<PropertyKey, string>>>) {}
			      ", None),
("
			type Options = [
			  { [Type in (typeof optionTesters)[number]['option']]?: boolean } & {
			    allow?: TypeOrValueSpecifier[];
			  },
			];
			      ", "
			type Options = [
			  Partial<Record<(typeof optionTesters)[number]['option'], boolean>> & {
			    allow?: TypeOrValueSpecifier[];
			  },
			];
			      ", None),
("
			export type MakeRequired<Base, Key extends keyof Base> = {
			  [K in Key]-?: NonNullable<Base[Key]>;
			} & Omit<Base, Key>;
			      ", "
			export type MakeRequired<Base, Key extends keyof Base> = Required<Record<Key, NonNullable<Base[Key]>>> & Omit<Base, Key>;
			      ", None),
("
			function f(): {
			  [k in (keyof ParseResult)]: unknown;
			} {
			  return {};
			}
			      ", "
			function f(): Record<keyof ParseResult, unknown> {
			  return {};
			}
			      ", None),
// ("
// 			interface Foo {
// 			  [key: string]: Bar;
// 			}
//
// 			interface Bar {
// 			  [key: string];
// 			}
// 			      ", "
// 			type Foo = Record<string, Bar>;
//
// 			interface Bar {
// 			  [key: string];
// 			}
// 			      ", None),
("
			type Foo = {
			  [k in string];
			};
			      ", "
			type Foo = Record<string, any>;
			      ", None)
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
