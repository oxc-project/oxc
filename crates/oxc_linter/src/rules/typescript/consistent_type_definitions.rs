use oxc_ast::{
    ast::{ExportDefaultDeclarationKind, TSType},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    context::{ContextHost, LintContext},
    rule::Rule,
    AstNode,
};

fn consistent_type_definitions_diagnostic(
    preferred_type_kind: &str,
    bad_type_kind: &str,
    span: Span,
) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Use an `{preferred_type_kind}` instead of a `{bad_type_kind}`"))
        .with_help(format!("Use an `{preferred_type_kind}` instead of a `{bad_type_kind}`"))
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct ConsistentTypeDefinitions {
    config: ConsistentTypeDefinitionsConfig,
}

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
enum ConsistentTypeDefinitionsConfig {
    #[default]
    Interface,
    Type,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce type definitions to consistently use either interface or type.
    ///
    /// ### Why is this bad?
    ///
    /// TypeScript provides two common ways to define an object type: interface and type.
    /// The two are generally very similar, and can often be used interchangeably.
    /// Using the same type declaration style consistently helps with code readability.
    ///
    /// ### Example
    /// ```ts
    /// // incorrect, when set to "interface"
    /// type T = { x: number };
    ///
    /// // incorrect when set to "type"
    /// interface T {
    /// x: number;
    /// }
    /// ```
    ConsistentTypeDefinitions,
    typescript,
    style,
    fix
);

impl Rule for ConsistentTypeDefinitions {
    fn from_configuration(value: serde_json::Value) -> Self {
        let config = value.get(0).and_then(serde_json::Value::as_str).map_or_else(
            ConsistentTypeDefinitionsConfig::default,
            |value| match value {
                "type" => ConsistentTypeDefinitionsConfig::Type,
                _ => ConsistentTypeDefinitionsConfig::Interface,
            },
        );
        Self { config }
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::TSTypeAliasDeclaration(decl) => match &decl.type_annotation {
                TSType::TSTypeLiteral(_)
                    if self.config == ConsistentTypeDefinitionsConfig::Interface =>
                {
                    let start = if decl.declare {
                        let base_start = decl.span.start + 7;
                        ctx.source_range(Span::new(base_start, decl.span.end))
                            .find("type")
                            .map_or(base_start + 1, |v| u32::try_from(v).unwrap_or(0) + base_start)
                    } else {
                        decl.span.start
                    };

                    let name_span_start = &decl.id.span.start;
                    let mut name_span_end = &decl.id.span.end;

                    if let Some(params) = &decl.type_parameters {
                        name_span_end = &params.span.end;
                    };

                    let name =
                        &ctx.source_text()[*name_span_start as usize..*name_span_end as usize];

                    if let TSType::TSTypeLiteral(type_ann) = &decl.type_annotation {
                        let body_span = type_ann.span;
                        let body =
                            &ctx.source_text()[body_span.start as usize..body_span.end as usize];

                        ctx.diagnostic_with_fix(
                            consistent_type_definitions_diagnostic(
                                "interface",
                                "type",
                                Span::new(start, start + 4),
                            ),
                            |fixer| {
                                fixer.replace(
                                    Span::new(start, decl.span.end),
                                    format!("interface {name} {body}"),
                                )
                            },
                        );
                    }
                }
                _ => {}
            },

            AstKind::ExportDefaultDeclaration(exp) => match &exp.declaration {
                ExportDefaultDeclarationKind::TSInterfaceDeclaration(decl)
                    if self.config == ConsistentTypeDefinitionsConfig::Type =>
                {
                    let name_span_start = &decl.id.span.start;
                    let mut name_span_end = &decl.id.span.end;

                    if let Some(params) = &decl.type_parameters {
                        name_span_end = &params.span.end;
                    };

                    let name =
                        &ctx.source_text()[*name_span_start as usize..*name_span_end as usize];

                    let body_span = &decl.body.span;
                    let body = &ctx.source_text()[body_span.start as usize..body_span.end as usize];

                    let extends_vec = {
                        let mut concatenated_names: Vec<&str> = Vec::new();

                        if let Some(extends) = decl.extends.as_deref() {
                            for exp in extends {
                                concatenated_names.push(
                                    &ctx.source_text()
                                        [exp.span.start as usize..exp.span.end as usize],
                                );
                            }
                        }

                        concatenated_names
                    };

                    let extends = if extends_vec.is_empty() {
                        String::new()
                    } else {
                        let joined_extends = extends_vec.join(" & ");
                        format!(" & {joined_extends}")
                    };

                    ctx.diagnostic_with_fix(
                        consistent_type_definitions_diagnostic(
                            "type",
                            "interface",
                            Span::new(decl.span.start, decl.span.start + 9),
                        ),
                        |fixer| {
                            fixer.replace(
                                exp.span,
                                format!("type {name} = {body}{extends}\nexport default {name}"),
                            )
                        },
                    );
                }
                _ => {}
            },

            AstKind::TSInterfaceDeclaration(decl)
                if self.config == ConsistentTypeDefinitionsConfig::Type =>
            {
                let start = if decl.declare {
                    let base_start = decl.span.start + 7;
                    ctx.source_range(Span::new(base_start, decl.span.end))
                        .find("interface")
                        .map_or(base_start + 1, |v| u32::try_from(v).unwrap_or(0) + base_start)
                } else {
                    decl.span.start
                };

                let name_span_start = &decl.id.span.start;
                let mut name_span_end = &decl.id.span.end;

                if let Some(params) = &decl.type_parameters {
                    name_span_end = &params.span.end;
                };

                let name = &ctx.source_text()[*name_span_start as usize..*name_span_end as usize];

                let body_span = &decl.body.span;
                let body = &ctx.source_text()[body_span.start as usize..body_span.end as usize];

                let extends_vec = {
                    let mut concatenated_names: Vec<&str> = Vec::new();

                    if let Some(extends) = decl.extends.as_deref() {
                        for exp in extends {
                            concatenated_names.push(
                                &ctx.source_text()[exp.span.start as usize..exp.span.end as usize],
                            );
                        }
                    }

                    concatenated_names
                };
                let extends = if extends_vec.is_empty() {
                    String::new()
                } else {
                    let joined_extends = extends_vec.join(" & ");
                    format!(" & {joined_extends}")
                };

                ctx.diagnostic_with_fix(
                    consistent_type_definitions_diagnostic(
                        "type",
                        "interface",
                        Span::new(start, start + 9),
                    ),
                    |fixer| {
                        fixer.replace(
                            Span::new(start, decl.span.end),
                            format!("type {name} = {body}{extends}"),
                        )
                    },
                );
            }
            _ => {}
        }
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.source_type().is_typescript()
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("var foo = {};", Some(serde_json::json!(["interface"]))),
        ("interface A {}", Some(serde_json::json!(["interface"]))),
        (
            "
			interface A extends B {
			  x: number;
			}
			      ",
            Some(serde_json::json!(["interface"])),
        ),
        ("type U = string;", Some(serde_json::json!(["interface"]))),
        ("type V = { x: number } | { y: string };", Some(serde_json::json!(["interface"]))),
        (
            "
			type Record<T, U> = {
			  [K in T]: U;
			};
			      ",
            Some(serde_json::json!(["interface"])),
        ),
        ("type T = { x: number };", Some(serde_json::json!(["type"]))),
        ("type A = { x: number } & B & C;", Some(serde_json::json!(["type"]))),
        ("type A = { x: number } & B<T1> & C<T2>;", Some(serde_json::json!(["type"]))),
        (
            "
			export type W<T> = {
			  x: T;
			};
			      ",
            Some(serde_json::json!(["type"])),
        ),
    ];

    let fail = vec![
        ("type T = { x: number; };", Some(serde_json::json!(["interface"]))),
        ("type T={ x: number; };", Some(serde_json::json!(["interface"]))),
        ("type T=                         { x: number; };", Some(serde_json::json!(["interface"]))),
        (
            "
			export type W<T> = {
			  x: T;
			};
			      ",
            Some(serde_json::json!(["interface"])),
        ),
        ("interface T { x: number; }", Some(serde_json::json!(["type"]))),
        ("interface T{ x: number; }", Some(serde_json::json!(["type"]))),
        ("interface T                          { x: number; }", Some(serde_json::json!(["type"]))),
        ("interface A extends B, C { x: number; };", Some(serde_json::json!(["type"]))),
        ("interface A extends B<T1>, C<T2> { x: number; };", Some(serde_json::json!(["type"]))),
        (
            "
			export interface W<T> {
			  x: T;
			}
			      ",
            Some(serde_json::json!(["type"])),
        ),
        (
            "
			namespace JSX {
			  interface Array<T> {
			    foo(x: (x: number) => T): T[];
			  }
			}
			      ",
            Some(serde_json::json!(["type"])),
        ),
        (
            "
			global {
			  interface Array<T> {
			    foo(x: (x: number) => T): T[];
			  }
			}
			      ",
            Some(serde_json::json!(["type"])),
        ),
        (
            "
			declare global {
			  interface Array<T> {
			    foo(x: (x: number) => T): T[];
			  }
			}
			      ",
            Some(serde_json::json!(["type"])),
        ),
        (
            "
			declare global {
			  namespace Foo {
			    interface Bar {}
			  }
			}
			      ",
            Some(serde_json::json!(["type"])),
        ),
        (
            "
			export default interface Test {
			  bar(): string;
			  foo(): number;
			}
			      ",
            Some(serde_json::json!(["type"])),
        ),
        (
            "
			export declare type Test = {
			  foo: string;
			  bar: string;
			};
			      ",
            Some(serde_json::json!(["interface"])),
        ),
        (
            "
			export declare interface Test {
			  foo: string;
			  bar: string;
			}
			      ",
            Some(serde_json::json!(["type"])),
        ),
        // Issue: <https://github.com/oxc-project/oxc/issues/7552>
        ("declaretype S={}", Some(serde_json::json!(["interface"]))),
        ("declareinterface S {}", Some(serde_json::json!(["type"]))),
        ("export declaretype S={}", Some(serde_json::json!(["interface"]))),
        ("export declareinterface S {}", Some(serde_json::json!(["type"]))),
    ];

    let fix = vec![
        (
            "type T = { x: number; };",
            "interface T { x: number; }",
            Some(serde_json::json!(["interface"])),
        ),
        (
            "type T={ x: number; };",
            "interface T { x: number; }",
            Some(serde_json::json!(["interface"])),
        ),
        (
            "type T=                         { x: number; };",
            "interface T { x: number; }",
            Some(serde_json::json!(["interface"])),
        ),
        (
            "export type W<T> = {
            x: T;
          };",
            "export interface W<T> {
            x: T;
          }",
            Some(serde_json::json!(["interface"])),
        ),
        (
            "interface T { x: number; }",
            "type T = { x: number; }",
            Some(serde_json::json!(["type"])),
        ),
        ("interface T{ x: number; }", "type T = { x: number; }", Some(serde_json::json!(["type"]))),
        (
            "interface A extends B, C { x: number; };",
            "type A = { x: number; } & B & C;",
            Some(serde_json::json!(["type"])),
        ),
        (
            "interface A extends B<T1>, C<T2> { x: number; };",
            "type A = { x: number; } & B<T1> & C<T2>;",
            Some(serde_json::json!(["type"])),
        ),
        (
            "export interface W<T> {
                x: T;
              }",
            "export type W<T> = {
                x: T;
              }",
            Some(serde_json::json!(["type"])),
        ),
        (
            "namespace JSX {
                interface Array<T> {
                  foo(x: (x: number) => T): T[];
                }
              }",
            "namespace JSX {
                type Array<T> = {
                  foo(x: (x: number) => T): T[];
                }
              }",
            Some(serde_json::json!(["type"])),
        ),
        (
            "global {
                interface Array<T> {
                  foo(x: (x: number) => T): T[];
                }
              }",
            "global {
                type Array<T> = {
                  foo(x: (x: number) => T): T[];
                }
              }",
            Some(serde_json::json!(["type"])),
        ),
        (
            "
export default interface Test {
    baz(): string;
    foo(): number;
}
            ",
            "
type Test = {
    baz(): string;
    foo(): number;
}
export default Test
            ",
            Some(serde_json::json!(["type"])),
        ),
        (
            "
export default interface Custom extends T {
    baz(): string;
    foo(): number;
}
            ",
            "
type Custom = {
    baz(): string;
    foo(): number;
} & T
export default Custom
            ",
            Some(serde_json::json!(["type"])),
        ),
        (
            "
export declare type Test = {
    foo: string;
    bar: string;
};
            ",
            "
export declare interface Test {
    foo: string;
    bar: string;
}
            ",
            Some(serde_json::json!(["interface"])),
        ),
        (
            "
export declare interface Test {
    foo: string;
    bar: string;
}
            ",
            "
export declare type Test = {
    foo: string;
    bar: string;
}
            ",
            Some(serde_json::json!(["type"])),
        ),
        // Issue: <https://github.com/oxc-project/oxc/issues/7552>
        ("declaretype S={}", "declareinterface S {}", Some(serde_json::json!(["interface"]))),
        ("declareinterface S {}", "declaretype S = {}", Some(serde_json::json!(["type"]))),
        (
            "export declaretype S={}",
            "export declareinterface S {}",
            Some(serde_json::json!(["interface"])),
        ),
        (
            "export declareinterface S {}",
            "export declaretype S = {}",
            Some(serde_json::json!(["type"])),
        ),
    ];

    Tester::new(ConsistentTypeDefinitions::NAME, ConsistentTypeDefinitions::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
