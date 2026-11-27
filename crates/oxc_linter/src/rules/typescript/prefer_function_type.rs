use oxc_ast::{
    AstKind,
    ast::{TSInterfaceDeclaration, TSSignature, TSThisType, TSTypeLiteral, TSTypeName},
};
use oxc_ast_visit::Visit;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_syntax::line_terminator::is_line_terminator;

use crate::{
    AstNode,
    context::{ContextHost, LintContext},
    rule::Rule,
};

fn prefer_function_type_diagnostic(suggestion: &str, span: Span) -> OxcDiagnostic {
    // FIXME: use imperative message phrasing
    OxcDiagnostic::warn("Enforce using function types instead of interfaces with call signatures.")
        .with_help(format!("The function type form `{suggestion}` is generally preferred when possible for being more succinct."))
        .with_label(span)
}

fn unexpected_this_diagnostic(interface_name: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "`this` refers to the function type `{interface_name}`, did you intend to use a generic `this` parameter like `<Self>(this: Self, ...) => Self` instead?"
    ))
    .with_label(span)
}

const CONVERT_TO_FUNCTION_TYPE: &str = "Convert to function type";

#[derive(Debug, Default, Clone)]
pub struct PreferFunctionType;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce using function types instead of interfaces with call signatures.
    ///
    /// ### Why is this bad?
    ///
    /// TypeScript allows for two common ways to declare a type for a function:
    /// - Function type: `() => string`
    /// - Object type with a signature: `{ (): string }`
    ///
    /// The function type form is generally preferred when possible for being
    /// more succinct and readable. Interfaces with only call signatures add
    /// unnecessary verbosity without providing additional functionality.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```typescript
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
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```typescript
    /// type Example = () => string;
    ///
    /// function foo(example: () => number): number {
    ///   return example();
    /// }
    ///
    /// // Returns the function itself, not the `this` argument
    /// type ReturnsSelf = (arg: string) => ReturnsSelf;
    ///
    /// // Multiple properties are allowed
    /// function foo(bar: { (): string; baz: number }): string {
    ///   return bar();
    /// }
    ///
    /// // Multiple call signatures (overloads) are allowed
    /// interface Overloaded {
    ///   (data: string): number;
    ///   (id: number): string;
    /// }
    ///
    /// // this is equivalent to Overloaded interface.
    /// type Intersection = ((data: string) => number) & ((id: number) => string);
    /// ```
    PreferFunctionType,
    typescript,
    style,
    conditional_fix,
    version = "0.2.11",
    short_description = "Enforce using function types instead of interfaces with call signatures.",
);

fn has_one_super_type(decl: &TSInterfaceDeclaration) -> bool {
    match decl.extends.len() {
        0 => return false,
        1 => {}
        _ => return true,
    }

    let expr = &decl.extends[0].type_name;
    if let TSTypeName::IdentifierReference(identifier) = expr {
        return &identifier.name != "Function";
    }

    true
}

fn function_type_text(member: &TSSignature, ctx: &LintContext<'_>) -> Option<(Span, String)> {
    let (span, return_type) = match member {
        TSSignature::TSConstructSignatureDeclaration(decl) => (decl.span, &decl.return_type),
        TSSignature::TSCallSignatureDeclaration(decl) => (decl.span, &decl.return_type),
        _ => return None,
    };
    let type_annotation = return_type.as_ref()?;
    let text = ctx.source_range(span);
    let colon_pos = (type_annotation.span.start - span.start) as usize;
    let mut suggestion = format!("{} =>{}", &text[..colon_pos], &text[colon_pos + 1..]);
    if suggestion.ends_with([';', ',']) {
        suggestion.pop();
    }
    Some((span, suggestion))
}

fn check_interface<'a>(
    decl: &TSInterfaceDeclaration<'a>,
    node: &AstNode<'a>,
    ctx: &LintContext<'a>,
) {
    let Some((span, suggestion)) = function_type_text(&decl.body.body[0], ctx) else {
        return;
    };

    let mut this_finder = ThisTypeFinder::default();
    this_finder.visit_ts_interface_declaration(decl);
    if let Some(this_span) = this_finder.span {
        ctx.diagnostic(unexpected_this_diagnostic(&decl.id.name, this_span));
        return;
    }

    let diagnostic = prefer_function_type_diagnostic(&suggestion, span);
    let (node_span, keyword) = match ctx.nodes().parent_kind(node.id()) {
        AstKind::ExportDefaultDeclaration(_) => {
            ctx.diagnostic(diagnostic);
            return;
        }
        AstKind::ExportDeclaration(export_decl) => (export_decl.span, "export type"),
        _ => (decl.span, "type"),
    };

    ctx.diagnostic_with_fix(diagnostic, |fixer| {
        let type_name = match &decl.type_parameters {
            Some(type_parameters) => {
                ctx.source_range(Span::new(decl.id.span.start, type_parameters.span.end))
            }
            None => decl.id.name.as_str(),
        };
        let mut replacement = String::new();
        for comment in ctx
            .comments_range(node_span.start..node_span.end)
            .filter(|comment| !span.contains_inclusive(comment.span))
        {
            replacement.push_str(ctx.source_range(comment.span));
            replacement.push('\n');
        }
        replacement.push_str(keyword);
        replacement.push(' ');
        replacement.push_str(type_name);
        replacement.push_str(" = ");
        replacement.push_str(&suggestion);
        replacement.push(';');
        fixer.replace(node_span, replacement).with_message(CONVERT_TO_FUNCTION_TYPE)
    });
}

fn check_type_literal<'a>(literal: &TSTypeLiteral<'a>, node: &AstNode<'a>, ctx: &LintContext<'a>) {
    let member = &literal.members[0];
    let Some((span, suggestion)) = function_type_text(member, ctx) else {
        return;
    };

    ctx.diagnostic_with_fix(prefer_function_type_diagnostic(&suggestion, span), |fixer| {
        let needs_parens = matches!(
            ctx.nodes().parent_kind(node.id()),
            AstKind::TSUnionType(_)
                | AstKind::TSIntersectionType(_)
                | AstKind::TSArrayType(_)
                | AstKind::TSTypeOperator(_)
                | AstKind::TSIndexedAccessType(_)
                | AstKind::TSConditionalType(_)
                | AstKind::TSOptionalType(_)
                | AstKind::TSRestType(_)
        );
        let mut replacement = String::new();
        for comment in ctx
            .comments_range(literal.span.start..span.start)
            .chain(ctx.comments_range(span.end..literal.span.end))
        {
            let between = if comment.span.start < span.start {
                Span::new(comment.span.start, span.start)
            } else {
                Span::new(span.start, comment.span.start)
            };
            replacement.push_str(ctx.source_range(comment.span));
            replacement.push(if ctx.source_range(between).contains(is_line_terminator) {
                '\n'
            } else {
                ' '
            });
        }
        if needs_parens {
            replacement.push('(');
            replacement.push_str(&suggestion);
            replacement.push(')');
        } else {
            replacement.push_str(&suggestion);
        }
        fixer.replace(literal.span, replacement).with_message(CONVERT_TO_FUNCTION_TYPE)
    });
}

#[derive(Default)]
struct ThisTypeFinder {
    span: Option<Span>,
}

impl<'a> Visit<'a> for ThisTypeFinder {
    fn visit_ts_this_type(&mut self, it: &TSThisType) {
        self.span.get_or_insert(it.span);
    }

    fn visit_ts_type_literal(&mut self, _it: &TSTypeLiteral<'a>) {}
}

impl Rule for PreferFunctionType {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::TSInterfaceDeclaration(decl)
                if !has_one_super_type(decl) && decl.body.body.len() == 1 =>
            {
                check_interface(decl, node, ctx);
            }
            AstKind::TSTypeLiteral(literal) if literal.members.len() == 1 => {
                check_type_literal(literal, node, ctx);
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
        "type K = { new(): T };",
        "export default interface Foo<T> { (): T }",
        "interface Factory { (): this; }",
        "type Result = { (): number; } | { error: string };",
        "type Callbacks = { (): void }[];",
        "type Keys = keyof { (): void };",
        "interface Callback<T> { /* Documents the callback */ (): T; }",
        "type Callback = { /* Documents the callback */ (): void };",
        "interface Callback { (/* value */ value: string): void }",
    ];

    let fix = vec![
        ("interface Foo { (): string; }", "type Foo = () => string;"),
        ("interface Foo { (): string, }", "type Foo = () => string;"),
        ("type Foo = { (): void, }", "type Foo = () => void"),
        ("let x: { (): void, };", "let x: () => void;"),
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
        ),
        (
            r"
type Foo<T> = { (this: string): T };
                      ",
            r"
type Foo<T> = (this: string) => T;
                      ",
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
        ),
        (
            r"
type X = {} | { (): void; }
                      ",
            r"
type X = {} | (() => void)
                      ",
        ),
        (
            r"
type X = {} & { (): void; };
                      ",
            r"
type X = {} & (() => void);
                      ",
        ),
        (
            "export interface AnyFn { (...args: any[]): any }",
            "export type AnyFn = (...args: any[]) => any;",
        ),
        ("type K = { new(): T };", "type K = new() => T;"),
        (
            "export default interface Foo { /** comment */ (): string; }",
            "export default interface Foo { /** comment */ (): string; }",
        ),
        ("export default interface Foo<T> { (): T }", "export default interface Foo<T> { (): T }"),
        ("interface Factory { (): this; }", "interface Factory { (): this; }"),
        (
            "type Result = { (): number; } | { error: string };",
            "type Result = (() => number) | { error: string };",
        ),
        ("type Callbacks = { (): void }[];", "type Callbacks = (() => void)[];"),
        ("type Keys = keyof { (): void };", "type Keys = keyof (() => void);"),
        (
            "interface Callback<T> { /* Documents the callback */ (): T; }",
            "/* Documents the callback */\ntype Callback<T> = () => T;",
        ),
        (
            "type Callback = { /* Documents the callback */ (): void };",
            "type Callback = /* Documents the callback */ () => void;",
        ),
        (
            "interface Callback { (/* value */ value: string): void }",
            "type Callback = (/* value */ value: string) => void;",
        ),
    ];

    Tester::new(PreferFunctionType::NAME, PreferFunctionType::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
