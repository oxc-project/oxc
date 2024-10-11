use oxc_ast::{
    ast::{ExportDefaultDeclarationKind, Expression, TSInterfaceDeclaration, TSSignature, TSType},
    AstKind, CommentKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    context::{ContextHost, LintContext},
    fixer::Fix,
    rule::Rule,
    AstNode,
};

fn prefer_function_type_diagnostic(suggestion: &str, span: Span) -> OxcDiagnostic {
    // FIXME: use imperative message phrasing
    OxcDiagnostic::warn("Enforce using function types instead of interfaces with call signatures.")
        .with_help(format!("The function type form `{suggestion}` is generally preferred when possible for being more succinct."))
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferFunctionType;

declare_oxc_lint!(
    /// ### What it does
    /// Enforce using function types instead of interfaces with call signatures.
    /// TypeScript allows for two common ways to declare a type for a function:
    ///
    /// - Function type: `() => string`
    /// - Object type with a signature: `{ (): string }`
    ///
    /// The function type form is generally preferred when possible for being more succinct.
    ///
    /// This rule suggests using a function type instead of an interface or object type literal with a single call signature.
    ///
    /// ### Example
    /// ```ts
    /// // error
    /// interface Example {
    ///   (): string;
    /// }
    ///
    /// function foo(example: { (): number }): number {
    ///   return example();
    /// }
    ///
    /// interface ReturnsSelf {
    ///   (arg: string): this;
    /// }
    ///
    /// // success
    /// type Example = () => string;
    ///
    /// function foo(example: () => number): number {
    ///   return bar();
    /// }
    ///
    /// // returns the function itself, not the `this` argument.
    /// type ReturnsSelf = (arg: string) => ReturnsSelf;
    ///
    /// function foo(bar: { (): string; baz: number }): string {
    ///   return bar();
    /// }
    ///
    /// interface Foo {
    ///   bar: string;
    /// }
    /// interface Bar extends Foo {
    ///   (): void;
    /// }
    ///
    /// // multiple call signatures (overloads) is allowed:
    /// interface Overloaded {
    ///   (data: string): number;
    ///   (id: number): string;
    /// }
    /// // this is equivalent to Overloaded interface.
    /// type Intersection = ((data: string) => number) & ((id: number) => string);
    /// ```
    PreferFunctionType,
    style,
    conditional_fix
);

fn has_one_super_type(decl: &TSInterfaceDeclaration) -> bool {
    if decl.extends.is_none() {
        return false;
    }

    let decl_extends_vec = decl.extends.as_deref().unwrap();

    if decl_extends_vec.is_empty() {
        return false;
    }

    if decl_extends_vec.len() != 1 {
        return true;
    }

    let expr = &decl_extends_vec[0].expression;

    if let Expression::Identifier(identifier) = expr {
        return !matches!(identifier.name.as_str(), "Function");
    }

    true
}

fn check_member(member: &TSSignature, node: &AstNode<'_>, ctx: &LintContext<'_>) {
    match member {
        TSSignature::TSCallSignatureDeclaration(decl) => {
            let start = decl.span.start;
            let end: u32 = decl.span.end;
            if let Some(type_annotation) = &decl.return_type {
                let colon_pos = type_annotation.span.start - start;
                let source_code = &ctx.source_text();
                let text: &str = &source_code[start as usize..end as usize];
                let mut suggestion = format!(
                    "{} =>{}",
                    &text[0..colon_pos as usize],
                    &text[(colon_pos + 1) as usize..text.len()]
                );

                if suggestion.ends_with(';') {
                    suggestion.pop();
                }

                match node.kind() {
                    AstKind::TSInterfaceDeclaration(interface_decl) => {
                        if let Some(type_parameters) = &interface_decl.type_parameters {
                            ctx.diagnostic_with_fix(
                                prefer_function_type_diagnostic(&suggestion, decl.span),
                                |fixer| {
                                    let mut span = interface_decl.id.span;
                                    span.end = type_parameters.span.end;
                                    let type_name = fixer.source_range(span);
                                    fixer.replace(
                                        interface_decl.span,
                                        format!("type {type_name} = {suggestion};"),
                                    )
                                },
                            );
                        } else {
                            ctx.diagnostic_with_fix(
                                prefer_function_type_diagnostic(&suggestion, decl.span),
                                |_| {
                                    let mut is_parent_exported = false;
                                    let mut node_start = interface_decl.span.start;
                                    let mut node_end = interface_decl.span.end;
                                    if let Some(parent_node) = ctx.nodes().parent_node(node.id()) {
                                        if let AstKind::ExportNamedDeclaration(export_name_decl) =
                                            parent_node.kind()
                                        {
                                            is_parent_exported = true;
                                            node_start = export_name_decl.span.start;
                                            node_end = export_name_decl.span.end;
                                        }
                                    }

                                    let has_comments =
                                        ctx.semantic().has_comments_between(interface_decl.span);

                                    if has_comments {
                                        let comments = ctx
                                            .semantic()
                                            .comments_range(node_start..node_end)
                                            .map(|comment| (*comment, comment.span));

                                        let comments_text = {
                                            let mut comments_vec: Vec<String> = vec![];
                                            comments.for_each(|(comment_interface, span)| {
                                                let comment = &source_code
                                                    [span.start as usize..span.end as usize];

                                                match comment_interface.kind {
                                                    CommentKind::Line => {
                                                        let single_line_comment: String =
                                                            format!("//{comment}\n");
                                                        comments_vec.push(single_line_comment);
                                                    }
                                                    CommentKind::Block => {
                                                        let multi_line_comment: String =
                                                            format!("/*{comment}*/\n");
                                                        comments_vec.push(multi_line_comment);
                                                    }
                                                }
                                            });

                                            comments_vec.join("")
                                        };

                                        return Fix::new(
                                            format!(
                                                "{}{}{} = {};",
                                                comments_text,
                                                if is_parent_exported {
                                                    "export type "
                                                } else {
                                                    "type "
                                                },
                                                &interface_decl.id.name,
                                                &suggestion
                                            ),
                                            Span::new(node_start, node_end),
                                        );
                                    }

                                    Fix::new(
                                        format!(
                                            "{} {} = {};",
                                            if is_parent_exported { "export type" } else { "type" },
                                            &interface_decl.id.name,
                                            &suggestion
                                        ),
                                        Span::new(node_start, node_end),
                                    )
                                },
                            );
                        }
                    }

                    AstKind::TSTypeAnnotation(ts_type_annotation) => match &ts_type_annotation
                        .type_annotation
                    {
                        TSType::TSUnionType(union_type) => {
                            union_type.types.iter().for_each(|ts_type| {
                                if let TSType::TSTypeLiteral(literal) = ts_type {
                                    ctx.diagnostic_with_fix(
                                        prefer_function_type_diagnostic(&suggestion, decl.span),
                                        |fixer| {
                                            fixer.replace(literal.span, format!("({suggestion})"))
                                        },
                                    );
                                }
                            });
                        }

                        TSType::TSTypeLiteral(literal) => ctx.diagnostic_with_fix(
                            prefer_function_type_diagnostic(&suggestion, decl.span),
                            |fixer| fixer.replace(literal.span, suggestion),
                        ),

                        _ => {
                            ctx.diagnostic(prefer_function_type_diagnostic(&suggestion, decl.span));
                        }
                    },

                    AstKind::TSTypeAliasDeclaration(ts_type_alias_decl) => {
                        match &ts_type_alias_decl.type_annotation {
                            TSType::TSUnionType(union_type) => {
                                union_type.types.iter().for_each(|ts_type| {
                                    if let TSType::TSTypeLiteral(literal) = ts_type {
                                        let body = &literal.members;
                                        if body.len() == 1 {
                                            ctx.diagnostic_with_fix(
                                                prefer_function_type_diagnostic(
                                                    &suggestion,
                                                    decl.span,
                                                ),
                                                |fixer| {
                                                    fixer.replace(
                                                        literal.span,
                                                        format!("({suggestion})"),
                                                    )
                                                },
                                            );
                                        }
                                    }
                                });
                            }

                            TSType::TSIntersectionType(intersection_type) => {
                                intersection_type.types.iter().for_each(|ts_type| {
                                    if let TSType::TSTypeLiteral(literal) = ts_type {
                                        let body = &literal.members;
                                        if body.len() == 1 {
                                            ctx.diagnostic_with_fix(
                                                prefer_function_type_diagnostic(
                                                    &suggestion,
                                                    decl.span,
                                                ),
                                                |fixer| {
                                                    fixer.replace(
                                                        literal.span,
                                                        format!("({suggestion})"),
                                                    )
                                                },
                                            );
                                        }
                                    }
                                });
                            }

                            TSType::TSTypeLiteral(literal) => ctx.diagnostic_with_fix(
                                prefer_function_type_diagnostic(&suggestion, decl.span),
                                |fixer| fixer.replace(literal.span, suggestion),
                            ),

                            _ => {}
                        }
                    }

                    _ => ctx.diagnostic(prefer_function_type_diagnostic(&suggestion, decl.span)),
                }
            }
        }

        TSSignature::TSConstructSignatureDeclaration(_)
        | TSSignature::TSIndexSignature(_)
        | TSSignature::TSPropertySignature(_)
        | TSSignature::TSMethodSignature(_) => {}
    }
}

impl Rule for PreferFunctionType {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::TSInterfaceDeclaration(decl) => {
                let body: &oxc_allocator::Vec<'_, TSSignature<'_>> = &decl.body.body;

                if !has_one_super_type(decl) && body.len() == 1 {
                    check_member(&body[0], node, ctx);
                }
            }

            AstKind::ExportDefaultDeclaration(decl) => {
                if let ExportDefaultDeclarationKind::TSInterfaceDeclaration(interface_decl) =
                    &decl.declaration
                {
                    let body = &interface_decl.body.body;
                    if !has_one_super_type(interface_decl) && body.len() == 1 {
                        check_member(&body[0], node, ctx);
                    }
                }
            }

            AstKind::TSTypeAnnotation(ts_type_annotation) => {
                match &ts_type_annotation.type_annotation {
                    TSType::TSUnionType(union_type) => {
                        union_type.types.iter().for_each(|ts_type| {
                            if let TSType::TSTypeLiteral(literal) = ts_type {
                                let body = &literal.members;
                                if body.len() == 1 {
                                    check_member(&body[0], node, ctx);
                                }
                            }
                        });
                    }

                    TSType::TSTypeLiteral(literal) => {
                        let body = &literal.members;
                        if body.len() == 1 {
                            check_member(&body[0], node, ctx);
                        }
                    }

                    _ => {}
                }
            }

            AstKind::TSTypeAliasDeclaration(ts_type_alias_decl) => {
                match &ts_type_alias_decl.type_annotation {
                    TSType::TSUnionType(union_type) => {
                        union_type.types.iter().for_each(|ts_type| {
                            if let TSType::TSTypeLiteral(literal) = ts_type {
                                let body = &literal.members;
                                if body.len() == 1 {
                                    check_member(&body[0], node, ctx);
                                }
                            }
                        });
                    }

                    TSType::TSIntersectionType(intersection_type) => {
                        intersection_type.types.iter().for_each(|ts_type| {
                            if let TSType::TSTypeLiteral(literal) = ts_type {
                                let body = &literal.members;
                                if body.len() == 1 {
                                    check_member(&body[0], node, ctx);
                                }
                            }
                        });
                    }

                    TSType::TSTypeLiteral(literal) => {
                        let body = &literal.members;

                        if body.len() == 1 {
                            check_member(&body[0], node, ctx);
                        }
                    }
                    _ => {}
                }
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
    let pass: Vec<&str> = vec![
        "interface Foo { (): void; bar: number; }",
        "type Foo = { (): void; bar: number; };",
        "function foo(bar: { (): string; baz: number }): string { return bar(); }",
        r"
        interface Foo {
          bar: string;
        }
        interface Bar extends Foo {
          (): void;
        }
            ",
        r"
        interface Foo {
          bar: string;
        }
        interface Bar extends Function, Foo {
          (): void;
        }
            ",
        "let foo: number | {};",
    ];

    let fail = vec![
        "interface Foo { (): string; }",
        "export default interface Foo { /** comment */ (): string; }",
        r"
        interface Foo {
          // comment
          (): string;
        }
              ",
        "export interface Foo { /** comment */ (): string; }",
        r"
        export interface Foo {
          // comment
          (): string;
        }
              ",
        r"
        function foo(bar: { /* comment */ (s: string): number } | undefined): number {
          return bar('hello');
        }
              ",
        r"
        type Foo = {
          (): string;
        };
              ",
        r"
        function foo(bar: { (s: string): number }): number {
          return bar('hello');
        }
              ",
        r"
        function foo(bar: { (s: string): number } | undefined): number {
          return bar('hello');
        }
              ",
        r"
        interface Foo extends Function {
          (): void;
        }
              ",
        r"
        interface Foo<T> {
          (bar: T): string;
        }
              ",
        r"
        interface Foo<T> {
          (this: T): void;
        }
              ",
        r"
        type Foo<T> = { (this: string): T };
              ",
        r"
        interface Foo {
          (arg: this): void;
        }
              ",
        r"
        interface Foo {
          (arg: number): this | undefined;
        }
              ",
        r"
        // isn't actually valid ts but want to not give message saying it refers to Foo.
        interface Foo {
          (): {
            a: {
              nested: this;
            };
            between: this;
            b: {
              nested: string;
            };
          };
        }
              ",
        "type X = {} | { (): void; }",
        "type X = {} & { (): void; };",
    ];

    let fix = vec![
        ("interface Foo { (): string; }", "type Foo = () => string;", None),
        (
            r"
interface Foo {
  // comment
  (): string;
}
                        ",
            r"
// comment
type Foo = () => string;
                        ",
            None,
        ),
        (
            r"
interface Foo {
/* comment */
(): string;
}
                      ",
            r"
/* comment */
type Foo = () => string;
                      ",
            None,
        ),
        (
            r"
export interface Foo {
  /** comment */
  (): string;
}",
            r"
/** comment */
export type Foo = () => string;",
            None,
        ),
        (
            r"
export interface Foo {
  // comment
  (): string;
}
",
            r"
// comment
export type Foo = () => string;
",
            None,
        ),
        (
            r"
function foo(bar: { (s: string): number } | undefined): number {
  return bar('hello');
}
",
            r"
function foo(bar: ((s: string) => number) | undefined): number {
  return bar('hello');
}
",
            None,
        ),
        (
            r"
interface Foo extends Function {
  (): void;
}
                        ",
            r"
type Foo = () => void;
                        ",
            None,
        ),
        (
            r"
interface Foo<T> {
  (bar: T): string;
}
                        ",
            r"
type Foo<T> = (bar: T) => string;
                        ",
            None,
        ),
        (
            r"
type Foo = {
  (): string;
};
                      ",
            r"
type Foo = () => string;
                      ",
            None,
        ),
        (
            r"
function foo(bar: { (s: string): number }): number {
  return bar('hello');
}
                      ",
            r"
function foo(bar: (s: string) => number): number {
  return bar('hello');
}
                      ",
            None,
        ),
        (
            r"
function foo(bar: { (s: string): number } | undefined): number {
  return bar('hello');
}
                      ",
            r"
function foo(bar: ((s: string) => number) | undefined): number {
  return bar('hello');
}
                      ",
            None,
        ),
        (
            r"
interface Foo extends Function {
  (): void;
}
                      ",
            r"
type Foo = () => void;
                      ",
            None,
        ),
        (
            r"
interface Foo<T> {
  (bar: T): string;
}
                      ",
            r"
type Foo<T> = (bar: T) => string;
                      ",
            None,
        ),
        (
            r"
interface Foo<T> {
  (this: T): void;
}
                      ",
            r"
type Foo<T> = (this: T) => void;
                      ",
            None,
        ),
        (
            r"
type Foo<T> = { (this: string): T };
                      ",
            r"
type Foo<T> = (this: string) => T;
                      ",
            None,
        ),
        (
            r"
interface Foo {
  (): {
    a: {
      nested: this;
    };
    between: this;
    b: {
      nested: string;
    };
  };
}
                      ",
            r"
type Foo = () => {
    a: {
      nested: this;
    };
    between: this;
    b: {
      nested: string;
    };
  };
                      ",
            None,
        ),
        (
            r"
type X = {} | { (): void; }
                      ",
            r"
type X = {} | (() => void)
                      ",
            None,
        ),
        (
            r"
type X = {} & { (): void; };
                      ",
            r"
type X = {} & (() => void);
                      ",
            None,
        ),
        (
            "export interface AnyFn { (...args: any[]): any }",
            "export type AnyFn = (...args: any[]) => any;",
            None,
        ),
    ];

    Tester::new(PreferFunctionType::NAME, pass, fail).expect_fix(fix).test_and_snapshot();
}
