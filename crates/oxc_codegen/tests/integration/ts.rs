use oxc_codegen::CodegenOptions;

use crate::{
    snapshot, snapshot_options,
    tester::{test_same, test_tsx},
};

#[test]
fn cases() {
    test_same("({ foo(): string {} });\n");
    test_same("interface I<in out T,> {}\n");
    test_same("function F<const in out T,>() {}\n");
    test_same("class C {\n\tp = await(0);\n}\n");
    test_same(
        "class Foo {\n\t#name: string;\n\tf() {\n\t\t#name in other && this.#name === other.#name;\n\t}\n}\n",
    );
    test_same("class B {\n\tconstructor(override readonly a: number) {}\n}\n");
    test_same("export { type as as };\n");
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
        "abstract class A {private abstract static m() {}}",
        "abstract class A {private abstract static readonly prop: string}",
        "interface A { a: string, 'b': number, 'c'(): void }",
        "enum A { a, 'b' }",
        "module 'a'",
        "declare module 'a'",
        "a = x!;",
        "b = (x as y);",
        "c = foo<string>;",
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
    ];

    snapshot("ts", &cases);
    snapshot_options("minify", &cases, &CodegenOptions::minify());
}
