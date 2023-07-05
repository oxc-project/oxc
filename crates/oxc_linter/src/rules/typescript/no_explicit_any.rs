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
            // 
        } else {
            if let AstKind::TSAnyKeyword(decl) = node.kind() {
                ctx.diagnostic(NoExplicitAnyDiagnostic(decl.span));
            }
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
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "const number: number = 1;",
        "function greet(): string {}",
        "function greet(): Array<string> {}",
        "function greet(): string[] {}",
        "function greet(): Array<Array<string>> {}",
        "function greet(): Array<string[]> {}",
        "function greet(param: Array<string>): Array<string> {}",
        "class Greeter {message: string;}",
        "class Greeter {message: Array<string>;}",
        "class Greeter {message: string[];}",
        "class Greeter {message: Array<Array<string>>;}",
        "class Greeter {message: Array<string[]>;}",
        "interface Greeter {message: string;}",
        "interface Greeter {message: Array<string>;}",
        "interface Greeter {message: string[];}",
        "interface Greeter {message: Array<Array<string>>;}",
        "interface Greeter {message: Array<string[]>;}",
        "type obj = {message: string;};",
        "type obj = {message: Array<string>;};",
        "type obj = {message: string[];};",
        "type obj = {message: Array<Array<string>>;};",
        "type obj = {message: Array<string[]>;};",
        "type obj = {message: string | number;};",
        "type obj = {message: string | Array<string>;};",
        "type obj = {message: string | string[];};",
        "type obj = {message: string | Array<Array<string>>;};",
        "type obj = {message: string & number;};",
        "type obj = {message: string & Array<string>;};",
        "type obj = {message: string & string[];};",
        "type obj = {message: string & Array<Array<string>>;};",
        // // TODO: the following are dependent on the option `ignoreRestArgs` - how to reference this option?
        // "function foo(a: number, ...rest: any[]): void {return;}",
        // "function foo1(...args: any[]) {}",
        // "const bar1 = function (...args: any[]) {};",
        // "const baz1 = (...args: any[]) => {};",
        // "function foo2(...args: readonly any[]) {}",
        // "const bar2 = function (...args: readonly any[]) {};",
        // "const baz2 = (...args: readonly any[]) => {};",
        // "function foo3(...args: Array<any>) {}",
        // "const bar3 = function (...args: Array<any>) {};",
        // "const baz3 = (...args: Array<any>) => {};",
        // "function foo4(...args: ReadonlyArray<any>) {}",
        // "const bar4 = function (...args: ReadonlyArray<any>) {};",
        // "const baz4 = (...args: ReadonlyArray<any>) => {};",
        // "interface Qux1 {(...args: any[]): void;}",
        // "interface Qux2 {(...args: readonly any[]): void;}",
        // "interface Qux3 {(...args: Array<any>): void;}",
        // "interface Qux4 {(...args: ReadonlyArray<any>): void;}",
        // "function quux1(fn: (...args: any[]) => void): void {}",
        // "function quux2(fn: (...args: readonly any[]) => void): void {}",
        // "function quux3(fn: (...args: Array<any>) => void): void {}",
        // "function quux4(fn: (...args: ReadonlyArray<any>) => void): void {}",
        // "function quuz1(): (...args: any[]) => void {}",
        // "function quuz2(): (...args: readonly any[]) => void {}",
        // "function quuz3(): (...args: Array<any>) => void {}",
        // "function quuz4(): (...args: ReadonlyArray<any>) => void {}",
        // "type Fred1 = (...args: any[]) => void;",
        // "type Fred2 = (...args: readonly any[]) => void;",
        // "type Fred3 = (...args: Array<any>) => void;",
        // "type Fred4 = (...args: ReadonlyArray<any>) => void;",
        // "type Corge1 = new (...args: any[]) => void;",
        // "type Corge2 = new (...args: readonly any[]) => void;",
        // "type Corge3 = new (...args: Array<any>) => void;",
        // "type Corge4 = new (...args: ReadonlyArray<any>) => void;",
        // "interface Grault1 {new (...args: any[]): void;}",
        // "interface Grault2 {new (...args: readonly any[]): void;}",
        // "interface Grault3 {new (...args: Array<any>): void;}",
        // "interface Grault4 {new (...args: ReadonlyArray<any>): void;}",
        // "interface Garply1 {f(...args: any[]): void;}",
        // "interface Garply2 {f(...args: readonly any[]): void;}",
        // "interface Garply3 {f(...args: Array<any>): void;}",
        // "interface Garply4 {f(...args: ReadonlyArray<any>): void;}",
        // "declare function waldo1(...args: any[]): void;",
        // "declare function waldo2(...args: readonly any[]): void;",
        // "declare function waldo3(...args: Array<any>): void;",
        // "declare function waldo4(...args: ReadonlyArray<any>): void;",
    ];

    let fail = vec![
        "const number: any = 1",
        "function generic(): any {}",
        "function generic(): Array<any> {}",
        "function generic(): any[] {}",
        "function generic(param: Array<any>): number {}",
        "function generic(param: any[]): number {}",
        "function generic(param: Array<any>): Array<any> {}",
        "function generic(): Array<Array<any>> {}",
        "function generic(): Array<any[]> {}",
        "class Greeter { constructor(param: Array<any>) {} }",
        "class Greeter { message: any; }",
        "class Greeter { message: Array<any>; }",
        "class Greeter { message: any[]; }",
        "class Greeter { message: Array<Array<any>>; }",
        "class Greeter { message: Array<any[]>; }",
        "interface Greeter { message: any; }",
        "interface Greeter { message: Array<any>; }",
        "interface Greeter { message: any[]; }",
        "interface Greeter { message: Array<Array<any>>; }",
        "interface Greeter { message: Array<any[]>; }",
        "type obj = { message: any; }",
        "type obj = { message: Array<any>; }",
        "type obj = { message: any[]; }",
        "type obj = { message: Array<Array<any>>; }",
        "type obj = { message: Array<any[]>; }",
        "type obj = { message: string | any; }",
        "type obj = { message: string | Array<any>; }",
        "type obj = { message: string | any[]; }",
        "type obj = { message: string | Array<Array<any>>; }",
        "type obj = { message: string | Array<any[]>; }",
        "type obj = { message: string & any; }",
        "type obj = { message: string & Array<any>; }",
        "type obj = { message: string & any[]; }",
        "type obj = { message: string & Array<Array<any>>; }",
        "type obj = { message: string & Array<any[]>; }",
        "class Foo<t = any> extends Bar<any> {}",
        "abstract class Foo<t = any> extends Bar<any> {}",
        "abstract class Foo<t = any> implements Bar<any>, Baz<any> {}",
        "new Foo<any>()",
        "Foo<any>()",
        "function foo(a: number, ...rest: any[]): void { return; }",
        "type Any = any;",
        // TODO: those are dependent on the option `ignoreRestArgs` - how to reference this option?
        "function foo5(...args: any) {}",
        "const bar5 = function (...args: any) {}",
        "const baz5 = (...args: any) => {}",
        "interface Qux5 { (...args: any): void; }",
        "function quux5(fn: (...args: any) => void): void {}",
        "function quuz5(): ((...args: any) => void) {}",
        "type Fred5 = (...args: any) => void;",
        "type Corge5 = new (...args: any) => void;",
        "interface Grault5 { new (...args: any): void; }",
        "interface Garply5 { f(...args: any): void; }",
        "declare function waldo5(...args: any): void;",
        // TODO: this gives a parsing error right now
        // "function test<T extends Partial<any>>() {} const test = <T extends Partial<any>>() => {};",
    ];

    Tester::new_without_config(NoExplicitAny::NAME, pass, fail).test_and_snapshot();
}
