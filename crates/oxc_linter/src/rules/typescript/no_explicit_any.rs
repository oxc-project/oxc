use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use serde_json::Value;

use crate::{
    context::{ContextHost, LintContext},
    rule::Rule,
    AstNode,
};

fn no_explicit_any_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected any. Specify a different type.")
        .with_help("Use `unknown` instead, this will force you to explicitly, and safely, assert the type is correct.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoExplicitAny {
    /// Whether to enable auto-fixing in which the `any` type is converted to the `unknown` type.
    ///
    /// `false` by default.
    fix_to_unknown: bool,
    /// Whether to ignore rest parameter arrays.
    ///
    /// `false` by default.
    ignore_rest_args: bool,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows explicit use of the `any` type.
    ///
    /// ### Why is this bad?
    ///
    /// The `any` type in TypeScript is a dangerous "escape hatch" from the type system. Using
    /// `any` disables many type checking rules and is generally best used only as a last resort or
    /// when prototyping code. This rule reports on explicit uses of the `any` keyword as a type
    /// annotation.
    ///
    /// > TypeScript's `--noImplicitAny` compiler option prevents an implied `any`, but doesn't
    /// > prevent `any` from being explicitly used the way this rule does.
    ///
    /// ### Example
    ///
    /// Examples of **incorrect** code for this rule:
    ///
    /// ```typescript
    /// const age: any = 'seventeen';
    /// const ages: any[] = ['seventeen']
    /// const ages: Array<any> = ['seventeen'];
    /// function greet(): any {}
    /// function greet(): any[] {}
    /// function greet(): Array<any> {}
    /// function greet(): Array<Array<any>> {}
    /// function greet(param: Array<any>): string {}
    /// function greet(param: Array<any>): Array<any> {}
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    ///
    /// ```typescript
    /// const age: number = 17;
    /// const ages: number[] = [17];
    /// const ages: Array<number> = [17];
    /// function greet(): string {}
    /// function greet(): string[] {}
    /// function greet(): Array<string> {}
    /// function greet(): Array<Array<string>> {}
    /// function greet(param: Array<string>): string {}
    /// function greet(param: Array<string>): Array<string> {}
    /// ```
    ///
    /// ## Options
    ///
    /// This rule accepts the following options:
    ///
    /// ### `ignoreRestArgs`
    /// A boolean to specify if arrays from the rest operator are considered ok. `false` by
    /// default.
    ///
    /// ### `fixToUnknown`
    ///
    /// Whether to enable auto-fixing in which the `any` type is converted to the `unknown` type.
    /// `false` by default.
    NoExplicitAny,
    restriction,
    conditional_fix
);

impl Rule for NoExplicitAny {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::TSAnyKeyword(any) = node.kind() else {
            return;
        };
        if self.ignore_rest_args && Self::is_in_rest(node, ctx) {
            return;
        }

        if self.fix_to_unknown {
            ctx.diagnostic_with_fix(no_explicit_any_diagnostic(any.span), |fixer| {
                fixer.replace(any.span, "unknown")
            });
        } else {
            ctx.diagnostic(no_explicit_any_diagnostic(any.span));
        }
    }

    fn from_configuration(value: Value) -> Self {
        let Some(cfg) = value.get(0) else {
            return Self::default();
        };
        let fix_to_unknown = cfg.get("fixToUnknown").and_then(Value::as_bool).unwrap_or(false);
        let ignore_rest_args = cfg.get("ignoreRestArgs").and_then(Value::as_bool).unwrap_or(false);

        Self { fix_to_unknown, ignore_rest_args }
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.source_type().is_typescript()
    }
}

impl NoExplicitAny {
    fn is_in_rest<'a>(node: &AstNode<'a>, ctx: &LintContext<'a>) -> bool {
        debug_assert!(matches!(node.kind(), AstKind::TSAnyKeyword(_)));
        ctx.nodes()
            .iter_parents(node.id())
            .any(|parent| matches!(parent.kind(), AstKind::BindingRestElement(_)))
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::tester::Tester;

    #[test]
    fn test_simple() {
        let pass = vec!["let x: number = 1"];
        let fail = vec!["let x: any = 1"];
        let fix = vec![
            ("let x: any = 1", "let x: unknown = 1", Some(json!([{ "fixToUnknown": true }]))),
            ("let x: any = 1", "let x: any = 1", None),
        ];
        Tester::new(NoExplicitAny::NAME, pass, fail).expect_fix(fix).test();
    }

    #[test]
    fn test() {
        let pass = vec![
            ("const number: number = 1;", None),
            ("function greet(): string {}", None),
            ("function greet(): Array<string> {}", None),
            ("function greet(): string[] {}", None),
            ("function greet(): Array<Array<string>> {}", None),
            ("function greet(): Array<string[]> {}", None),
            ("function greet(param: Array<string>): Array<string> {}", None),
            (
                "
                class Greeter {
                  message: string;
                }
                    ",
                None,
            ),
            (
                "
                class Greeter {
                  message: Array<string>;
                }
                    ",
                None,
            ),
            (
                "
                class Greeter {
                  message: string[];
                }
                    ",
                None,
            ),
            (
                "
                class Greeter {
                  message: Array<Array<string>>;
                }
                    ",
                None,
            ),
            (
                "
                class Greeter {
                  message: Array<string[]>;
                }
                    ",
                None,
            ),
            (
                "
                interface Greeter {
                  message: string;
                }
                    ",
                None,
            ),
            (
                "
                interface Greeter {
                  message: Array<string>;
                }
                    ",
                None,
            ),
            (
                "
                interface Greeter {
                  message: string[];
                }
                    ",
                None,
            ),
            (
                "
                interface Greeter {
                  message: Array<Array<string>>;
                }
                    ",
                None,
            ),
            (
                "
                interface Greeter {
                  message: Array<string[]>;
                }
                    ",
                None,
            ),
            (
                "
                type obj = {
                  message: string;
                };
                    ",
                None,
            ),
            (
                "
                type obj = {
                  message: Array<string>;
                };
                    ",
                None,
            ),
            (
                "
                type obj = {
                  message: string[];
                };
                    ",
                None,
            ),
            (
                "
                type obj = {
                  message: Array<Array<string>>;
                };
                    ",
                None,
            ),
            (
                "
                type obj = {
                  message: Array<string[]>;
                };
                    ",
                None,
            ),
            (
                "
                type obj = {
                  message: string | number;
                };
                    ",
                None,
            ),
            (
                "
                type obj = {
                  message: string | Array<string>;
                };
                    ",
                None,
            ),
            (
                "
                type obj = {
                  message: string | string[];
                };
                    ",
                None,
            ),
            (
                "
                type obj = {
                  message: string | Array<Array<string>>;
                };
                    ",
                None,
            ),
            (
                "
                type obj = {
                  message: string & number;
                };
                    ",
                None,
            ),
            (
                "
                type obj = {
                  message: string & Array<string>;
                };
                    ",
                None,
            ),
            (
                "
                type obj = {
                  message: string & string[];
                };
                    ",
                None,
            ),
            (
                "
                type obj = {
                  message: string & Array<Array<string>>;
                };
                    ",
                None,
            ),
            (
                "
                        function foo(a: number, ...rest: any[]): void {
                          return;
                        }
                      ",
                Some(serde_json::json!([{ "ignoreRestArgs": true }])),
            ),
            (
                "function foo1(...args: any[]) {}",
                Some(serde_json::json!([{ "ignoreRestArgs": true }])),
            ),
            (
                "const bar1 = function (...args: any[]) {};",
                Some(serde_json::json!([{ "ignoreRestArgs": true }])),
            ),
            (
                "const baz1 = (...args: any[]) => {};",
                Some(serde_json::json!([{ "ignoreRestArgs": true }])),
            ),
            (
                "function foo2(...args: readonly any[]) {}",
                Some(serde_json::json!([{ "ignoreRestArgs": true }])),
            ),
            (
                "const bar2 = function (...args: readonly any[]) {};",
                Some(serde_json::json!([{ "ignoreRestArgs": true }])),
            ),
            (
                "const baz2 = (...args: readonly any[]) => {};",
                Some(serde_json::json!([{ "ignoreRestArgs": true }])),
            ),
            (
                "function foo3(...args: Array<any>) {}",
                Some(serde_json::json!([{ "ignoreRestArgs": true }])),
            ),
            (
                "const bar3 = function (...args: Array<any>) {};",
                Some(serde_json::json!([{ "ignoreRestArgs": true }])),
            ),
            (
                "const baz3 = (...args: Array<any>) => {};",
                Some(serde_json::json!([{ "ignoreRestArgs": true }])),
            ),
            (
                "function foo4(...args: ReadonlyArray<any>) {}",
                Some(serde_json::json!([{ "ignoreRestArgs": true }])),
            ),
            (
                "const bar4 = function (...args: ReadonlyArray<any>) {};",
                Some(serde_json::json!([{ "ignoreRestArgs": true }])),
            ),
            (
                "const baz4 = (...args: ReadonlyArray<any>) => {};",
                Some(serde_json::json!([{ "ignoreRestArgs": true }])),
            ),
            (
                "
                interface Qux1 {
                  (...args: any[]): void;
                }
                      ",
                Some(serde_json::json!([{ "ignoreRestArgs": true }])),
            ),
            (
                "
                interface Qux2 {
                  (...args: readonly any[]): void;
                }
                      ",
                Some(serde_json::json!([{ "ignoreRestArgs": true }])),
            ),
            (
                "
                interface Qux3 {
                  (...args: Array<any>): void;
                }
                      ",
                Some(serde_json::json!([{ "ignoreRestArgs": true }])),
            ),
            (
                "
                interface Qux4 {
                  (...args: ReadonlyArray<any>): void;
                }
                      ",
                Some(serde_json::json!([{ "ignoreRestArgs": true }])),
            ),
            (
                "function quux1(fn: (...args: any[]) => void): void {}",
                Some(serde_json::json!([{ "ignoreRestArgs": true }])),
            ),
            (
                "function quux2(fn: (...args: readonly any[]) => void): void {}",
                Some(serde_json::json!([{ "ignoreRestArgs": true }])),
            ),
            (
                "function quux3(fn: (...args: Array<any>) => void): void {}",
                Some(serde_json::json!([{ "ignoreRestArgs": true }])),
            ),
            (
                "function quux4(fn: (...args: ReadonlyArray<any>) => void): void {}",
                Some(serde_json::json!([{ "ignoreRestArgs": true }])),
            ),
            (
                "function quuz1(): (...args: any[]) => void {}",
                Some(serde_json::json!([{ "ignoreRestArgs": true }])),
            ),
            (
                "function quuz2(): (...args: readonly any[]) => void {}",
                Some(serde_json::json!([{ "ignoreRestArgs": true }])),
            ),
            (
                "function quuz3(): (...args: Array<any>) => void {}",
                Some(serde_json::json!([{ "ignoreRestArgs": true }])),
            ),
            (
                "function quuz4(): (...args: ReadonlyArray<any>) => void {}",
                Some(serde_json::json!([{ "ignoreRestArgs": true }])),
            ),
            (
                "type Fred1 = (...args: any[]) => void;",
                Some(serde_json::json!([{ "ignoreRestArgs": true }])),
            ),
            (
                "type Fred2 = (...args: readonly any[]) => void;",
                Some(serde_json::json!([{ "ignoreRestArgs": true }])),
            ),
            (
                "type Fred3 = (...args: Array<any>) => void;",
                Some(serde_json::json!([{ "ignoreRestArgs": true }])),
            ),
            (
                "type Fred4 = (...args: ReadonlyArray<any>) => void;",
                Some(serde_json::json!([{ "ignoreRestArgs": true }])),
            ),
            (
                "type Corge1 = new (...args: any[]) => void;",
                Some(serde_json::json!([{ "ignoreRestArgs": true }])),
            ),
            (
                "type Corge2 = new (...args: readonly any[]) => void;",
                Some(serde_json::json!([{ "ignoreRestArgs": true }])),
            ),
            (
                "type Corge3 = new (...args: Array<any>) => void;",
                Some(serde_json::json!([{ "ignoreRestArgs": true }])),
            ),
            (
                "type Corge4 = new (...args: ReadonlyArray<any>) => void;",
                Some(serde_json::json!([{ "ignoreRestArgs": true }])),
            ),
            (
                "
                interface Grault1 {
                  new (...args: any[]): void;
                }
                      ",
                Some(serde_json::json!([{ "ignoreRestArgs": true }])),
            ),
            (
                "
                interface Grault2 {
                  new (...args: readonly any[]): void;
                }
                      ",
                Some(serde_json::json!([{ "ignoreRestArgs": true }])),
            ),
            (
                "
                interface Grault3 {
                  new (...args: Array<any>): void;
                }
                      ",
                Some(serde_json::json!([{ "ignoreRestArgs": true }])),
            ),
            (
                "
                interface Grault4 {
                  new (...args: ReadonlyArray<any>): void;
                }
                      ",
                Some(serde_json::json!([{ "ignoreRestArgs": true }])),
            ),
            (
                "
                interface Garply1 {
                  f(...args: any[]): void;
                }
                      ",
                Some(serde_json::json!([{ "ignoreRestArgs": true }])),
            ),
            (
                "
                interface Garply2 {
                  f(...args: readonly any[]): void;
                }
                      ",
                Some(serde_json::json!([{ "ignoreRestArgs": true }])),
            ),
            (
                "
                interface Garply3 {
                  f(...args: Array<any>): void;
                }
                      ",
                Some(serde_json::json!([{ "ignoreRestArgs": true }])),
            ),
            (
                "
                interface Garply4 {
                  f(...args: ReadonlyArray<any>): void;
                }
                      ",
                Some(serde_json::json!([{ "ignoreRestArgs": true }])),
            ),
            (
                "declare function waldo1(...args: any[]): void;",
                Some(serde_json::json!([{ "ignoreRestArgs": true }])),
            ),
            (
                "declare function waldo2(...args: readonly any[]): void;",
                Some(serde_json::json!([{ "ignoreRestArgs": true }])),
            ),
            (
                "declare function waldo3(...args: Array<any>): void;",
                Some(serde_json::json!([{ "ignoreRestArgs": true }])),
            ),
            (
                "declare function waldo4(...args: ReadonlyArray<any>): void;",
                Some(serde_json::json!([{ "ignoreRestArgs": true }])),
            ),
        ];

        let fail = vec![
            ("const number: any = 1", None),
            ("function generic(): any {}", None),
            ("function generic(): Array<any>", None),
            ("function generic(): any[] {}", None),
            ("function generic(param: Array<any>): number { return 1 }", None),
            ("function generic(param: any[]): number { return 1 }", None),
            ("function generic(param: Array<any>): Array<any>", None),
            ("function generic(): Array<Array<any>>", None),
            ("function generic(param: Array<any[]>): Array<any>", None),
            ("class Greeter { constructor(param: Array<any>) {} }", None),
            ("class Greeter { message: any }", None),
            ("class Greeter { message: Array<any> }", None),
            ("class Greeter { message: any[] }", None),
            ("class Greeter { message: Array<Array<any>> }", None),
            ("interface Greeter { constructor(param: Array<any>) {} }", None),
            ("interface Greeter { message: any }", None),
            ("interface Greeter { message: Array<any> }", None),
            ("interface Greeter { message: any[] }", None),
            ("interface Greeter { message: Array<Array<any>> }", None),
            ("type obj = { constructor(param: Array<any>) {} }", None),
            ("type obj = { message: any }", None),
            ("type obj = { message: Array<any> }", None),
            ("type obj = { message: any[] }", None),
            ("type obj = { message: Array<Array<any>> }", None),
            ("type obj = { message: string | any }", None),
            ("type obj = { message: string | Array<any> }", None),
            ("type obj = { message: string | Array<any[]> }", None),
            ("type obj = { message: string | Array<Array<any>> }", None),
            ("type obj = { message: string & any }", None),
            ("type obj = { message: string & any[] }", None),
            ("type obj = { message: string & Array<any> }", None),
            ("type obj = { message: string & Array<Array<any>> }", None),
            ("type obj = { message: string & Array<any[]> }", None),
            ("class Foo<T = any> extends Bar<any> {}", None),
            ("abstract class Foo<T = any> extends Bar<any> {}", None),
            ("function test<T extends Partial<any>>() {}", None),
            ("const test = <T extends Partial<any>>() => {};", None),
            ("function foo(a: number, ...rest: any[]): void { return; }", None),
            ("type Any = any;", None),
            // todo
            // (
            //     "function foo(...args: any) {}",
            //     Some(serde_json::json!([{ "ignoreRestArgs": true }])),
            // ),
        ];

        let fix_options = Some(json!([{ "fixToUnknown": true }]));
        let fixes = vec![
            ("let x: any = 1", "let x: unknown = 1", fix_options.clone()),
            ("function foo(): any", "function foo(): unknown", fix_options.clone()),
            (
                "function foo(args: any): void {}",
                "function foo(args: unknown): void {}",
                fix_options.clone(),
            ),
            (
                "function foo(args: any[]): void {}",
                "function foo(args: unknown[]): void {}",
                fix_options.clone(),
            ),
            (
                "function foo(...args: any[]): void {}",
                "function foo(...args: unknown[]): void {}",
                fix_options.clone(),
            ),
            (
                "function foo(args: Array<any>): void {}",
                "function foo(args: Array<unknown>): void {}",
                fix_options,
            ),
            // NOTE: no current way to check that fixes don't occur when `ignoreRestArgs` is
            // `true`, since no fix technically occurs and `expect_fix()` panics without a fix.
        ];
        Tester::new(NoExplicitAny::NAME, pass, fail).expect_fix(fixes).test_and_snapshot();
    }
}
