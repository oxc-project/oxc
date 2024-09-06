use std::fmt::Write;

use oxc_allocator::Allocator;
use oxc_codegen::{CodeGenerator, CodegenOptions};
use oxc_parser::Parser;
use oxc_span::SourceType;

fn codegen(source_text: &str) -> String {
    let allocator = Allocator::default();
    let source_type = SourceType::default().with_typescript(true).with_module(true);
    let ret = Parser::new(&allocator, source_text, source_type).parse();
    CodeGenerator::new()
        .with_options(CodegenOptions { single_quote: true, ..CodegenOptions::default() })
        .build(&ret.program)
        .source_text
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
        "a = x!;",
        "b = (x as y);",
        "c = foo<string>;",
        "d = x satisfies y;",
        "export @x declare abstract class C {}",
        "div<T>``",
    ];

    let snapshot = cases.into_iter().fold(String::new(), |mut w, case| {
        write!(w, "{case}\n{}\n", codegen(case)).unwrap();
        w
    });

    insta::with_settings!({ prepend_module_to_snapshot => false, omit_expression => true }, {
        insta::assert_snapshot!("ts", snapshot);
    });
}
