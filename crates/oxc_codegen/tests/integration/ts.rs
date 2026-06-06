use oxc_codegen::CodegenOptions;
use oxc_parser::ParseOptions;
use oxc_span::SourceType;

use crate::{
    snapshot, snapshot_options,
    tester::{
        default_options, test_idempotency, test_options_with_source_type, test_same, test_tsx,
        test_with_parse_options,
    },
};

#[test]
fn cases() {
    test_same("({ foo(): string {} });\n");
    test_same("({ method(this: Foo): void {} });\n");
    test_same("({ methodWithParam(this: Foo, bar: string): void {} });\n");
    test_same("type T = (A | B)[];\n");
    test_same("type T = (A & B)[];\n");
    test_same("type T = (keyof A)[];\n");
    test_same("type T = (() => A)[];\n");
    test_same("type T = (new () => A)[];\n");
    test_same("type T = (A extends B ? C : D)[];\n");
    test_same("type T = (A | B)[K];\n");
    test_same("type T = (A & B)[K];\n");
    test_same("type T = (keyof A)[K];\n");
    test_same("type T = (A extends B ? C : D)[K];\n");
    test_same("type T = A extends (B extends C ? D : E) ? F : G;\n");
    test_same("type T = { [K in U]: V };\n");
    test_same("type T = { [K in U]?: V };\n");
    test_same("type T = { -readonly [K in U]-?: V };\n");
    test_same("type T = (A extends B ? C : D) extends E ? F : G;\n");
    test_same("type T = A & (B extends C ? D : E);\n");
    test_same("type T = (A | B) & C;\n");
    test_same("declare interface A {}\n");
    test_same("interface I<in out T,> {}\n");
    test_same("function F<const in out T,>() {}\n");
    test_same("class C {\n\tp = await(0);\n}\n");
    test_same(
        "class Foo {\n\t#name: string;\n\tf() {\n\t\t#name in other && this.#name === other.#name;\n\t}\n}\n",
    );
    test_same("class B {\n\tconstructor(override readonly a: number) {}\n}\n");
    test_same("class C extends B {\n\toverride show(): void;\n\toverride hide(): void;\n}\n");
    test_same("class D extends B {\n\toverride readonly x: number;\n}\n");
    test_same(
        "declare namespace ns {\n\tclass Foo {}\n\tenum Bar {}\n\ttype Baz = undefined;\n}\n",
    );
    test_same("class E {\n\tsubscribe!: string;\n}\n");
    test_same("class F {\n\taccessor value!: string;\n}\n");
    test_same("class E {\n\tstatic [key: string]: string;\n}\n");
    test_same("export { type as as };\n");
    test_same("import type from = require(\"./a\");\n");
    test_same("try {} catch (e: unknown) {} finally {}\n");
    test_same("const Bar = class<T,> {};\n");
}

#[test]
fn decorators() {
    test_same("@a abstract class C {}\n");
    test_tsx("@a @b export default abstract class {}", "export default @a @b abstract class {}\n");
}

#[test]
fn tsx() {
    test_tsx("<T,>() => {}", "<T,>() => {};\n");
    test_tsx("<T, B>() => {}", "<\n\tT,\n\tB\n>() => {};\n");
    test_tsx("<Foo<T> />", "<Foo<T> />;\n");
}

#[test]
fn ts() {
    let cases = [
        "let x: string = `\\x01`;",
        "function foo<T extends string>(x: T, y: string, ...restOfParams: Omit<T, 'x'>): T {\n\treturn x;\n}",
        "let x: string[] = ['abc', 'def', 'ghi'];",
        "let x: Array<string> = ['abc', 'def', 'ghi',];",
        "let x: [string, number] = ['abc', 123];",
        "let x: string | number = 'abc';",
        "let x: string & number = 'abc';",
        "let x: typeof String = 'string';",
        "let x: keyof string = 'length';",
        "let x: keyof typeof String = 'length';",
        "let x: string['length'] = 123;",
        "function isString(value: unknown): asserts value is string {
            if (typeof value !== 'string') {
                throw new Error('Not a string');
            }
        }",
        "import type { Foo } from 'foo';",
        "import { Foo, type Bar } from 'foo';",
        "export { Foo, type Bar } from 'foo';",
        "type A<T> = { [K in keyof T as K extends string ? B<K> : K ]: T[K] }",
        "class A {readonly type = 'frame'}",
        "let foo: { <T>(t: T): void }",
        "let foo: { new <T>(t: T): void }",
        "function <const T>(){}",
        "class A {m?(): void}",
        "class A {constructor(public readonly a: number) {}}",
        "abstract class A {protected abstract m()}",
        "class A {private static readonly prop: string}",
        "interface A { a: string, 'b': number, 'c'(): void }",
        "enum A { a, 'b' }",
        "module 'a'",
        "declare module 'a'",
        "a = x!;",
        "b = (x as y);",
        "c = foo<string>;",
        "new Map<string, number>();",
        "d = x satisfies y;",
        "export @x declare abstract class C {}",
        "div<T>``",
        "export type Component<Props = any> = Foo;",
        "
export type Component<
  Props = any,
  RawBindings = any,
  D = any,
  C extends ComputedOptions = ComputedOptions,
  M extends MethodOptions = MethodOptions,
  E extends EmitsOptions | Record<string, any[]> = {},
  S extends Record<string, any> = any,
> =
  | ConcreteComponent<Props, RawBindings, D, C, M, E, S>
  | ComponentPublicInstanceConstructor<Props>
",
        "(a || b) as any",
        "(a ** b) as any",
        "(function g() {}) as any",
        r#"
import defaultExport from "module-name";
import * as name from "module-name";
import { export1 } from "module-name";
import { export1 as alias1 } from "module-name";
import { default as alias } from "module-name";
import { export1, export2 } from "module-name";
import { export1, export2 as alias2, /* … */ } from "module-name";
import { "string name" as alias } from "module-name";
import defaultExport, { export1, /* … */ } from "module-name";
import defaultExport, * as name from "module-name";
import "module-name";
import {} from 'mod';

export let name1, name2/*, … */; // also var
export const name3 = 1, name4 = 2/*, … */; // also var, let
export function functionName() { /* … */ }
export class ClassName { /* … */ }
export function* generatorFunctionName() { /* … */ }
export const { name5, name2: bar } = o;
export const [ name6, name7 ] = array;

export { name8, /* …, */ name81 };
export { variable1 as name9, variable2 as name10, /* …, */ name82 };
export { variable1 as "string name" };
export { name1 as default1 /*, … */ };

export * from "module-name";
export * as name11 from "module-name";
export { name12, /* …, */ nameN } from "module-name";
export { import1 as name13, import2 as name14, /* …, */ name15 } from "module-name";
export { default, /* …, */ } from "module-name";
export { default as name16 } from "module-name";
"#,
        r#"
import a = require("a");
export import b = require("b");
"#,
        "class C {
  static
  static
  static
  bar() {}
}",
        // TSImportType - ensure backticks are not used in minify mode
        "type T = typeof import('react');",
        "type U = typeof import(\"vue\");",
        "type V = typeof import('some-module').SomeType;",
        "type W = typeof import('pkg').default<string>;",
    ];

    snapshot("ts", &cases);
    snapshot_options("minify", &cases, &CodegenOptions::minify());
}

#[test]
fn minify_export_default() {
    let min = |src: &str, expected: &str| {
        test_options_with_source_type(src, expected, SourceType::ts(), CodegenOptions::minify());
    };
    // A leading `interface`/`abstract`/keyword needs the space; `{`/`<` does not.
    min("export default interface I { x: number }", "export default interface I{x:number;}");
    min("export default abstract class {}", "export default abstract class{}");
    min("export default <const>x;", "export default<const>x;");
}

#[test]
fn minify_return_type_colon() {
    let min = |src: &str, expected: &str| {
        test_options_with_source_type(src, expected, SourceType::ts(), CodegenOptions::minify());
    };
    // No space after `:` in a function return type / `this` param annotation,
    // matching method/arrow return types.
    min("function f(): Promise<void> {}", "function f():Promise<void>{}");
    min(
        "function g(a: string): boolean { return true; }",
        "function g(a:string):boolean{return true}",
    );
    min("function h(this: Foo): void {}", "function h(this:Foo):void{}");
}

#[test]
fn minify_ts_type_space() {
    let min = |src: &str, expected: &str| {
        test_options_with_source_type(src, expected, SourceType::ts(), CodegenOptions::minify());
    };
    // Conditional type: `?`/`:` are tight like a JS conditional expression.
    min("type T = A extends B ? C : D;", "type T=A extends B?C:D;");
    min("type T = A extends {} ? B : C;", "type T=A extends {}?B:C;");
    // Constructor type: no space after `new` before `(`/`<`.
    min("type N = new () => Foo;", "type N=new()=>Foo;");
    min("type N = abstract new (x: number) => Foo;", "type N=abstract new(x:number)=>Foo;");
    // A JSDoc-nullable branch must not merge into `??`.
    min("type T = A extends B ? ?C : D;", "type T=A extends B? ?C:D;");
    min("type T = A extends C? ? D : E;", "type T=A extends C? ?D:E;");
}

#[test]
fn ts_as_expression_in_binary_expr() {
    test_idempotency("key in (that as object)");
    test_idempotency("'foo' in (x as Record<string, unknown>)");
    test_idempotency("(x as object) instanceof Map");
    test_idempotency("'foo' in ((x as object) as Record<string, unknown>)");
    test_idempotency(
        "!(typeof that === 'object' && 'keys' in that && typeof (that as object & { keys: unknown }).keys === 'function')",
    );
}

#[test]
fn ts_type_assertion() {
    // `<T>x` (TS angle-bracket assertion) is only valid in non-tsx source.
    let test_ts =
        |src: &str| test_options_with_source_type(src, src, SourceType::ts(), default_options());
    // `<T>x` is a unary expression; it should not be over-parenthesized.
    test_ts("y = <T>x;\n");
    test_ts("z = <T>x + 1;\n");
    test_ts("foo(<T>x);\n");
    test_ts("c = -<T>x;\n");
    // Parentheses are required where a unary expression would re-associate.
    test_ts("m = (<T>x).foo;\n");
    test_ts("o = (<T>x)();\n");
    // The base of `**` must be an UpdateExpression, so a type assertion is wrapped.
    test_ts("n = (<T>x) ** 2;\n");
    // Minified `a < <T>x` must keep a space so `<` + `<` isn't tokenized as `<<`.
    test_options_with_source_type(
        "a < <T>x;",
        "a< <T>x;",
        SourceType::ts(),
        CodegenOptions::minify(),
    );
    // The assertion operand is a UnaryExpression and must not be over-parenthesized.
    test_ts("a = <T>-x;\n");
    test_ts("b = <T>typeof x;\n");
    test_ts("c = <T><U>x;\n");
    test_ts("d = <T>x();\n");
    // Looser operands still need parentheses.
    test_ts("e = <T>(b + c);\n");
    test_ts("f = <T>(d ** e);\n");
    test_ts("g = <T>(h ? i : j);\n");
}

#[test]
fn ts_instantiation_expression() {
    test_same("v = (a ?? b)<T>;\n");
    test_same("w = (a + b)<T>;\n");
    test_same("x = (a, b)<T>;\n");
    test_same("q = (a as B)<T>;\n");
    test_same("r = (-a)<T>;\n");
    test_same("y = a.b<T>;\n");
    test_same("z = f<T>;\n");
    test_same("p = a()<T>;\n");
}

#[test]
fn ts_satisfies_expression() {
    test_idempotency("d = x satisfies y");
    test_idempotency("const Foo = (() => {})() satisfies X");
    test_idempotency("const Bar = (x as Y) satisfies Z");
    test_idempotency("(x satisfies Y).foo");
    test_idempotency("(x satisfies Y)[0]");
    test_idempotency("(x satisfies Y)()");
    test_idempotency("x satisfies Y || z");
    test_idempotency("x satisfies Y && z");
    test_idempotency("x satisfies Y === z");
}

#[test]
fn type_codegen_with_preserve_parens_off() {
    let parse_options = ParseOptions { preserve_parens: false, ..Default::default() };

    test_with_parse_options(
        "type T = keyof (EventMap & Extra);",
        "type T = keyof (EventMap & Extra);\n",
        parse_options,
    );
    test_with_parse_options(
        "type T = [(Anno | undefined)?];",
        "type T = [(Anno | undefined)?];\n",
        parse_options,
    );
    test_with_parse_options("const foo = (a ?? b)!;", "const foo = (a ?? b)!;\n", parse_options);
    test_with_parse_options(
        "type T = (Out & (Step extends A ? B : C)) & (Step extends D ? E : F);",
        "type T = (Out & (Step extends A ? B : C)) & (Step extends D ? E : F);\n",
        parse_options,
    );
    test_with_parse_options(
        "type T = ({ [K in keyof Obj]: Obj[K] } & { a: 1 }) & { b: 2 };",
        "type T = ({ [K in keyof Obj]: Obj[K] } & {\n\ta: 1;\n}) & {\n\tb: 2;\n};\n",
        parse_options,
    );
}
