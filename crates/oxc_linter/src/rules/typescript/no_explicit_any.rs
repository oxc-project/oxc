use oxc_ast::AstKind;
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error(
"typescript-eslint(no-explicit-any): Using any disables many type checking rules and is generally best used only as a last resort or when prototyping code"
)]
#[diagnostic(severity(warning))]
struct NoExplicitAnyDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoExplicitAny {
    pub ignore_rest_args: bool,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow the any type.
    ///
    /// ### Why is this bad?
    ///
    /// The any type in TypeScript is a dangerous "escape hatch" from the type system.
    /// Using any disables many type checking rules and is generally best used only as a last
    /// resort or when prototyping code. This rule reports on explicit uses of the any keyword
    /// as a type annotation.
    ///
    /// ### Example
    /// ```typescript
    /// const age: any = 'seventeen';
    /// const ages: any[] = ['seventeen'];
    /// const ages: Array<any> = ['seventeen'];
    /// ```
    NoExplicitAny,
    correctness
);

impl Rule for NoExplicitAny {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if self.ignore_rest_args {
// todo!("implement linting for the ignoreRestArgs option");
        } else if let AstKind::TSAnyKeyword(decl) = node.kind() {
            ctx.diagnostic(NoExplicitAnyDiagnostic(decl.span));
        }
    }

    fn from_configuration(value: serde_json::Value) -> Self {
        let ignore_rest_args = value
            .get(0)
            .and_then(|config| config.get("ignoreRestArgs"))
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);

        Self { ignore_rest_args }
    }
}

#[test]
#[allow(clippy::too_many_lines)]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("const number: number = 1;", None),
        ("function greet(): string {}", None),
        ("function greet(): Array<string> {}", None),
        ("function greet(): string[] {}", None),
        ("function greet(): Array<Array<string>> {}", None),
        ("function greet(): Array<string[]> {}", None),
        ("function greet(param: Array<string>): Array<string> {}", None),
        ("class Greeter {message: string;}  ", None),
        ("class Greeter {message: Array<string>;}  ", None),
        ("class Greeter {message: string[];}  ", None),
        ("class Greeter {message: Array<Array<string>>;}  ", None),
        ("class Greeter {message: Array<string[]>;}  ", None),
        ("interface Greeter {message: string;}  ", None),
        ("interface Greeter {message: Array<string>;}  ", None),
        ("interface Greeter {message: string[];}  ", None),
        ("interface Greeter {message: Array<Array<string>>;}  ", None),
        ("interface Greeter {message: Array<string[]>;}  ", None),
        ("type obj = {message: string;};  ", None),
        ("type obj = {message: Array<string>;};  ", None),
        ("type obj = {message: string[];};  ", None),
        ("type obj = {message: Array<Array<string>>;};  ", None),
        ("type obj = {message: Array<string[]>;};  ", None),
        ("type obj = {message: string | number;};  ", None),
        ("type obj = {message: string | Array<string>;};  ", None),
        ("type obj = {message: string | string[];};  ", None),
        ("type obj = {message: string | Array<Array<string>>;};  ", None),
        ("type obj = {message: string & number;};  ", None),
        ("type obj = {message: string & Array<string>;};  ", None),
        ("type obj = {message: string & string[];};  ", None),
        ("type obj = {message: string & Array<Array<string>>;};  ", None),
        // (
        //     "function foo(a: number, ...rest: any[]): void { return; }",
        //     Some(serde_json::json!([{ "ignoreRestArgs": true }])),
        // ),
        // ("function foo1(...args: any[]) {}", Some(serde_json::json!([{ "ignoreRestArgs": true }]))),
        // (
        //     "const bar1 = function (...args: any[]) {};",
        //     Some(serde_json::json!([{ "ignoreRestArgs": true }])),
        // ),
        // (
        //     "const baz1 = (...args: any[]) => {};",
        //     Some(serde_json::json!([{ "ignoreRestArgs": true }])),
        // ),
        // (
        //     "function foo2(...args: readonly any[]) {}",
        //     Some(serde_json::json!([{ "ignoreRestArgs": true }])),
        // ),
        // (
        //     "const bar2 = function (...args: readonly any[]) {};",
        //     Some(serde_json::json!([{ "ignoreRestArgs": true }])),
        // ),
        // (
        //     "const baz2 = (...args: readonly any[]) => {};",
        //     Some(serde_json::json!([{ "ignoreRestArgs": true }])),
        // ),
        // (
        //     "function foo3(...args: Array<any>) {}",
        //     Some(serde_json::json!([{ "ignoreRestArgs": true }])),
        // ),
        // (
        //     "const bar3 = function (...args: Array<any>) {};",
        //     Some(serde_json::json!([{ "ignoreRestArgs": true }])),
        // ),
        // (
        //     "const baz3 = (...args: Array<any>) => {};",
        //     Some(serde_json::json!([{ "ignoreRestArgs": true }])),
        // ),
        // (
        //     "function foo4(...args: ReadonlyArray<any>) {}",
        //     Some(serde_json::json!([{ "ignoreRestArgs": true }])),
        // ),
        // (
        //     "const bar4 = function (...args: ReadonlyArray<any>) {};",
        //     Some(serde_json::json!([{ "ignoreRestArgs": true }])),
        // ),
        // (
        //     "const baz4 = (...args: ReadonlyArray<any>) => {};",
        //     Some(serde_json::json!([{ "ignoreRestArgs": true }])),
        // ),
        // (
        //     "interface Qux1 {(...args: any[]): void;}",
        //     Some(serde_json::json!([{ "ignoreRestArgs": true }])),
        // ),
        // (
        //     "interface Qux2 {(...args: readonly any[]): void;}",
        //     Some(serde_json::json!([{ "ignoreRestArgs": true }])),
        // ),
        // (
        //     "interface Qux3 {(...args: Array<any>): void;}",
        //     Some(serde_json::json!([{ "ignoreRestArgs": true }])),
        // ),
        // (
        //     "interface Qux4 {(...args: ReadonlyArray<any>): void;}",
        //     Some(serde_json::json!([{ "ignoreRestArgs": true }])),
        // ),
        // (
        //     "function quux1(fn: (...args: any[]) => void): void {}",
        //     Some(serde_json::json!([{ "ignoreRestArgs": true }])),
        // ),
        // (
        //     "function quux2(fn: (...args: readonly any[]) => void): void {}",
        //     Some(serde_json::json!([{ "ignoreRestArgs": true }])),
        // ),
        // (
        //     "function quux3(fn: (...args: Array<any>) => void): void {}",
        //     Some(serde_json::json!([{ "ignoreRestArgs": true }])),
        // ),
        // (
        //     "function quux4(fn: (...args: ReadonlyArray<any>) => void): void {}",
        //     Some(serde_json::json!([{ "ignoreRestArgs": true }])),
        // ),
        // (
        //     "function quuz1(): (...args: any[]) => void {}",
        //     Some(serde_json::json!([{ "ignoreRestArgs": true }])),
        // ),
        // (
        //     "function quuz2(): (...args: readonly any[]) => void {}",
        //     Some(serde_json::json!([{ "ignoreRestArgs": true }])),
        // ),
        // (
        //     "function quuz3(): (...args: Array<any>) => void {}",
        //     Some(serde_json::json!([{ "ignoreRestArgs": true }])),
        // ),
        // (
        //     "function quuz4(): (...args: ReadonlyArray<any>) => void {}",
        //     Some(serde_json::json!([{ "ignoreRestArgs": true }])),
        // ),
        // (
        //     "type Fred1 = (...args: any[]) => void;",
        //     Some(serde_json::json!([{ "ignoreRestArgs": true }])),
        // ),
        // (
        //     "type Fred2 = (...args: readonly any[]) => void;",
        //     Some(serde_json::json!([{ "ignoreRestArgs": true }])),
        // ),
        // (
        //     "type Fred3 = (...args: Array<any>) => void;",
        //     Some(serde_json::json!([{ "ignoreRestArgs": true }])),
        // ),
        // (
        //     "type Fred4 = (...args: ReadonlyArray<any>) => void;",
        //     Some(serde_json::json!([{ "ignoreRestArgs": true }])),
        // ),
        // (
        //     "type Corge1 = new (...args: any[]) => void;",
        //     Some(serde_json::json!([{ "ignoreRestArgs": true }])),
        // ),
        // (
        //     "type Corge2 = new (...args: readonly any[]) => void;",
        //     Some(serde_json::json!([{ "ignoreRestArgs": true }])),
        // ),
        // (
        //     "type Corge3 = new (...args: Array<any>) => void;",
        //     Some(serde_json::json!([{ "ignoreRestArgs": true }])),
        // ),
        // (
        //     "type Corge4 = new (...args: ReadonlyArray<any>) => void;",
        //     Some(serde_json::json!([{ "ignoreRestArgs": true }])),
        // ),
        // (
        //     "interface Grault1 {new (...args: any[]): void;}",
        //     Some(serde_json::json!([{ "ignoreRestArgs": true }])),
        // ),
        // (
        //     "interface Grault2 {new (...args: readonly any[]): void;}",
        //     Some(serde_json::json!([{ "ignoreRestArgs": true }])),
        // ),
        // (
        //     "interface Grault3 {new (...args: Array<any>): void;}",
        //     Some(serde_json::json!([{ "ignoreRestArgs": true }])),
        // ),
        // (
        //     "interface Grault4 {new (...args: ReadonlyArray<any>): void;}",
        //     Some(serde_json::json!([{ "ignoreRestArgs": true }])),
        // ),
        // (
        //     "interface Garply1 {f(...args: any[]): void;}",
        //     Some(serde_json::json!([{ "ignoreRestArgs": true }])),
        // ),
        // (
        //     "interface Garply2 {f(...args: readonly any[]): void;}",
        //     Some(serde_json::json!([{ "ignoreRestArgs": true }])),
        // ),
        // (
        //     "interface Garply3 {f(...args: Array<any>): void;}",
        //     Some(serde_json::json!([{ "ignoreRestArgs": true }])),
        // ),
        // (
        //     "interface Garply4 {f(...args: ReadonlyArray<any>): void;}",
        //     Some(serde_json::json!([{ "ignoreRestArgs": true }])),
        // ),
        // (
        //     "declare function waldo1(...args: any[]): void;",
        //     Some(serde_json::json!([{ "ignoreRestArgs": true }])),
        // ),
        // (
        //     "declare function waldo2(...args: readonly any[]): void;",
        //     Some(serde_json::json!([{ "ignoreRestArgs": true }])),
        // ),
        // (
        //     "declare function waldo3(...args: Array<any>): void;",
        //     Some(serde_json::json!([{ "ignoreRestArgs": true }])),
        // ),
        // (
        //     "declare function waldo4(...args: ReadonlyArray<any>): void;",
        //     Some(serde_json::json!([{ "ignoreRestArgs": true }])),
        // ),
    ];

    let fail = vec![
        ("const number: any = 1", None),
        ("function generic(): any {}", None),
        ("function generic(): Array<any> {}", None),
        ("function generic(): any[] {}", None),
        ("function generic(param: Array<any>): number {}", None),
        ("function generic(param: any[]): number {}", None),
        ("function generic(param: Array<any>): Array<any> {}", None),
        ("function generic(): Array<Array<any>> {}", None),
        ("function generic(): Array<any[]> {}", None),
        ("class Greeter { constructor(param: Array<any>) {}}", None),
        ("class Greeter { message: any; }", None),
        ("class Greeter { message: Array<any>; }", None),
        ("class Greeter { message: any[]; }", None),
        ("class Greeter { message: Array<Array<any>>; }", None),
        ("class Greeter { message: Array<any[]>; }", None),
        ("interface Greeter { message: any; }", None),
        ("interface Greeter { message: Array<any>; }", None),
        ("interface Greeter { message: any[]; }", None),
        ("interface Greeter { message: Array<Array<any>>; }", None),
        ("interface Greeter { message: Array<any[]>; }", None),
        ("type obj = { message: any; }", None),
        ("type obj = { message: Array<any>; }", None),
        ("type obj = { message: any[]; }", None),
        ("type obj = { message: Array<Array<any>>; }", None),
        ("type obj = { message: Array<any[]>; }", None),
        ("type obj = { message: string | any; }", None),
        ("type obj = { message: string | Array<any>; }", None),
        ("type obj = { message: string | any[]; }", None),
        ("type obj = { message: string | Array<Array<any>>; }", None),
        ("type obj = { message: string | Array<any[]>; }", None),
        ("type obj = { message: string & any; }", None),
        ("type obj = { message: string & Array<any>; }", None),
        ("type obj = { message: string & any[]; }", None),
        ("type obj = { message: string & Array<Array<any>>; }", None),
        ("type obj = { message: string & Array<any[]>; }", None),
        ("class Foo<t = any> extends Bar<any> {}", None),
        ("abstract class Foo<t = any> extends Bar<any> {}", None),
        ("abstract class Foo<t = any> implements Bar<any>, Baz<any> {}", None),
        ("new Foo<any>()", None),
        ("Foo<any>()", None),
        // TODO: this is a parsing error - possible bug in oxc parser?
        // (
        //     "function test<T extends Partial<any>>() {} const test = <T extends Partial<any>>() => {};",
        //     None,
        // ),
        ("function foo(a: number, ...rest: any[]): void { return; }", None),
        
        
        // ("type Any = any;", Some(serde_json::json!([{ "ignoreRestArgs": true }]))),
        // ("function foo5(...args: any) {}", Some(serde_json::json!([{ "ignoreRestArgs": true }]))),
        // (
        //     "const bar5 = function (...args: any) {}",
        //     Some(serde_json::json!([{ "ignoreRestArgs": true }])),
        // ),
        // (
        //     "const baz5 = (...args: any) => {}",
        //     Some(serde_json::json!([{ "ignoreRestArgs": true }])),
        // ),
        // (
        //     "interface Qux5 { (...args: any): void; }",
        //     Some(serde_json::json!([{ "ignoreRestArgs": true }])),
        // ),
        // (
        //     "function quux5(fn: (...args: any) => void): void {}",
        //     Some(serde_json::json!([{ "ignoreRestArgs": true }])),
        // ),
        // (
        //     "function quuz5(): ((...args: any) => void) {}",
        //     Some(serde_json::json!([{ "ignoreRestArgs": true }])),
        // ),
        // (
        //     "type Fred5 = (...args: any) => void;",
        //     Some(serde_json::json!([{ "ignoreRestArgs": true }])),
        // ),
        // (
        //     "type Corge5 = new (...args: any) => void;",
        //     Some(serde_json::json!([{ "ignoreRestArgs": true }])),
        // ),
        // (
        //     "interface Grault5 { new (...args: any): void; }",
        //     Some(serde_json::json!([{ "ignoreRestArgs": true }])),
        // ),
        // (
        //     "interface Garply5 { f(...args: any): void; }",
        //     Some(serde_json::json!([{ "ignoreRestArgs": true }])),
        // ),
        // (
        //     "declare function waldo5(...args: any): void;",
        //     Some(serde_json::json!([{ "ignoreRestArgs": true }])),
        // ),
    ];

    Tester::new(NoExplicitAny::NAME, pass, fail).test_and_snapshot();
}
