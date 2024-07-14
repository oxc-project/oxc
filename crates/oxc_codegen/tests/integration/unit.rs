use oxc_allocator::Allocator;
use oxc_codegen::{CodeGenerator, CodegenOptions};
use oxc_parser::Parser;
use oxc_span::SourceType;

fn check(source_text: &str, expected: &str, source_type: SourceType) {
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source_text, source_type).parse();
    let result = CodeGenerator::new()
        .with_options(CodegenOptions { single_quote: true })
        .build(&ret.program)
        .source_text;
    assert_eq!(expected, result, "for source {source_text}, expect {expected}, got {result}");
}

fn test(source_text: &str, expected: &str) {
    let source_type = SourceType::default().with_module(true);
    check(source_text, expected, source_type);
}

fn test_ts(source_text: &str, expected: &str, is_typescript_definition: bool) {
    let source_type = SourceType::default()
        .with_typescript(true)
        .with_typescript_definition(is_typescript_definition)
        .with_module(true);
    check(source_text, expected, source_type);
}

#[test]
fn string() {
    test("let x = ''", "let x = '';\n");
    test(r"let x = '\b'", "let x = '\\b';\n");
    test(r"let x = '\f'", "let x = '\\f';\n");
    test("let x = '\t'", "let x = '\t';\n");
    test(r"let x = '\v'", "let x = '\\v';\n");
    test("let x = '\\n'", "let x = '\\n';\n");
    test("let x = '\\''", "let x = '\\'';\n");
    test("let x = '\\\"'", "let x = '\"';\n");
    // test( "let x = '\\'''", "let x = `''`;\n");
    test("let x = '\\\\'", "let x = '\\\\';\n");
    test("let x = '\x00'", "let x = '\\0';\n");
    test("let x = '\x00!'", "let x = '\\0!';\n");
    test("let x = '\x001'", "let x = '\\x001';\n");
    test("let x = '\\0'", "let x = '\\0';\n");
    test("let x = '\\0!'", "let x = '\\0!';\n");
    test("let x = '\x07'", "let x = '\\x07';\n");
    test("let x = '\x07!'", "let x = '\\x07!';\n");
    test("let x = '\x071'", "let x = '\\x071';\n");
    test("let x = '\\7'", "let x = '\\x07';\n");
    test("let x = '\\7!'", "let x = '\\x07!';\n");
    test("let x = '\\01'", "let x = '\x01';\n");
    test("let x = '\x10'", "let x = '\x10';\n");
    test("let x = '\\x10'", "let x = '\x10';\n");
    test("let x = '\x1B'", "let x = '\\x1B';\n");
    test("let x = '\\x1B'", "let x = '\\x1B';\n");
    test("let x = '\\uABCD'", "let x = 'ꯍ';\n");
    // test( "let x = '\\uABCD'", r#"let x = '\uABCD';\n"#);
    // test( r#"let x = '\U000123AB'"#, r#"let x = '\U000123AB';\n"#);
    // test( "let x = '\\u{123AB}'", r#"let x = '\U000123AB';\n"#);
    // test( "let x = '\\uD808\\uDFAB'", r#"let x = '\U000123AB';\n"#);
    test("let x = '\\uD808'", "let x = '\\\\ud808';\n");
    test("let x = '\\uD808X'", "let x = '\\\\ud808X';\n");
    test("let x = '\\uDFAB'", "let x = '\\\\udfab';\n");
    test("let x = '\\uDFABX'", "let x = '\\\\udfabX';\n");

    // test( "let x = '\\x80'", r#"let x = '\U00000080';\n"#);
    // test( "let x = '\\xFF'", r#"let x = '\U000000FF';\n"#);
    // test( "let x = '\\xF0\\x9F\\x8D\\x95'", r#"let x = '\U000000F0\U0000009F\U0000008D\U00000095';\n"#);
    // test("let x = '\\uD801\\uDC02\\uDC03\\uD804'", r#"let x = '\U00010402\\uDC03\\uD804';\n"#)
}

#[test]
fn template() {
    test("let x = `\\0`", "let x = `\\0`;\n");
    test("let x = `\\x01`", "let x = `\\x01`;\n");
    test("let x = `\\0${0}`", "let x = `\\0${0}`;\n");
    // test("let x = `\\x01${0}`", "let x = `\x01${0}`;\n");
    test("let x = `${0}\\0`", "let x = `${0}\\0`;\n");
    // test("let x = `${0}\\x01`", "let x = `${0}\x01`;\n");
    test("let x = `${0}\\0${1}`", "let x = `${0}\\0${1}`;\n");
    // test("let x = `${0}\\x01${1}`", "let x = `${0}\x01${1}`;\n");

    test("let x = String.raw`\\1`", "let x = String.raw`\\1`;\n");
    test("let x = String.raw`\\x01`", "let x = String.raw`\\x01`;\n");
    test("let x = String.raw`\\1${0}`", "let x = String.raw`\\1${0}`;\n");
    test("let x = String.raw`\\x01${0}`", "let x = String.raw`\\x01${0}`;\n");
    test("let x = String.raw`${0}\\1`", "let x = String.raw`${0}\\1`;\n");
    test("let x = String.raw`${0}\\x01`", "let x = String.raw`${0}\\x01`;\n");
    test("let x = String.raw`${0}\\1${1}`", "let x = String.raw`${0}\\1${1}`;\n");
    test("let x = String.raw`${0}\\x01${1}`", "let x = String.raw`${0}\\x01${1}`;\n");

    test("let x = `${y}`", "let x = `${y}`;\n");
    test("let x = `$(y)`", "let x = `$(y)`;\n");
    test("let x = `{y}$`", "let x = `{y}$`;\n");
    test("let x = `$}y{`", "let x = `$}y{`;\n");
    test("let x = `\\${y}`", "let x = `\\${y}`;\n");
    // test("let x = `$\\{y}`", "let x = `\\${y}`;\n");

    // test("await tag`x`", "await tag`x`;\n");
    // test("await (tag`x`)", "await tag`x`;\n");
    // test("(await tag)`x`", "(await tag)`x`;\n");

    // test("await tag`${x}`", "await tag`${x}`;\n");
    // test("await (tag`${x}`)", "await tag`${x}`;\n");
    // test("(await tag)`${x}`", "(await tag)`${x}`;\n");

    // test("new tag`x`", "new tag`x`();\n");
    // test("new (tag`x`)", "new tag`x`();\n");
    // test("new tag()`x`", "new tag()`x`;\n");
    // test("(new tag)`x`", "new tag()`x`;\n");

    // test("new tag`${x}`", "new tag`${x}`();\n");
    // test("new (tag`${x}`)", "new tag`${x}`();\n");
    // test("new tag()`${x}`", "new tag()`${x}`;\n");
    // test("(new tag)`${x}`", "new tag()`${x}`;\n");
}

#[test]
fn module_decl() {
    test("export * as foo from 'foo'", "export * as foo from 'foo';\n");
    test("import x from './foo.js' with {}", "import x from './foo.js' with {\n};\n");
    test("import {} from './foo.js' with {}", "import {} from './foo.js' with {\n};\n");
    test("export * from './foo.js' with {}", "export * from './foo.js' with {\n};\n");
}

#[test]
fn new_expr() {
    test("new (foo()).bar();", "new (foo()).bar();\n");
}

#[test]
fn access_property() {
    test(
        "export default class Foo { @x @y accessor #aDef = 1 }",
        "export default class Foo {\n\taccessor #aDef=1;\n}\n",
    );
}

#[test]
fn for_stmt() {
    test("for (let x = 0; x < 10; x++) {}", "for (let x = 0; x < 10; x++) {}\n");
    test("for (;;) {}", "for (;;) {}\n");
    test("for (let x = 1;;) {}", "for (let x = 1;;) {}\n");
    test("for (;true;) {}", "for (; true;) {}\n");
    test("for (;;i++) {}", "for (;; i++) {}\n");

    test("for (using x = 1;;) {}", "for (using x = 1;;) {}\n");
    // TODO
    // test(
    // "for (var a = 1 || (2 in {}) in { x: 1 }) count++;",
    // "for (var a = 1 || (2 in {}) in {x: 1}) count++;\n",
    // );
}

#[test]
fn typescript() {
    test_ts("let x: string = `\\x01`;", "let x: string = `\\x01`;\n", false);

    test_ts(
        "function foo<T extends string>(x: T, y: string, ...restOfParams: Omit<T, 'x'>): T {\n\treturn x;\n}",
        "function foo<T extends string>(x: T, y: string, ...restOfParams: Omit<T, 'x'>): T {\n\treturn x;\n}\n",
        false,
    );

    test_ts(
        "let x: string[] = ['abc', 'def', 'ghi'];",
        "let x: string[] = ['abc', 'def', 'ghi'];\n",
        false,
    );
    test_ts(
        "let x: Array<string> = ['abc', 'def', 'ghi',];",
        "let x: Array<string> = ['abc', 'def', 'ghi',];\n",
        false,
    );
    test_ts(
        "let x: [string, number] = ['abc', 123];",
        "let x: [string, number] = ['abc', 123];\n",
        false,
    );
    test_ts("let x: string | number = 'abc';", "let x: string | number = 'abc';\n", false);
    test_ts("let x: string & number = 'abc';", "let x: string & number = 'abc';\n", false);
    test_ts("let x: typeof String = 'string';", "let x: typeof String = 'string';\n", false);
    test_ts("let x: keyof string = 'length';", "let x: keyof string = 'length';\n", false);
    test_ts(
        "let x: keyof typeof String = 'length';",
        "let x: keyof typeof String = 'length';\n",
        false,
    );
    test_ts("let x: string['length'] = 123;", "let x: string['length'] = 123;\n", false);

    test_ts(
        "function isString(value: unknown): asserts value is string {\n\tif (typeof value !== 'string') {\n\t\tthrow new Error('Not a string');\n\t}\n}",
        "function isString(value: unknown): asserts value is string {\n\tif (typeof value !== 'string') {\n\t\tthrow new Error('Not a string');\n\t}\n}\n",
        false,
    );

    // type-only imports/exports
    test_ts("import type { Foo } from 'foo';", "import type { Foo } from 'foo';\n", false);
    test_ts(
        "import { Foo, type Bar } from 'foo';",
        "import { Foo, type Bar } from 'foo';\n",
        false,
    );
    test_ts(
        "export { Foo, type Bar } from 'foo';",
        "export { Foo, type Bar } from 'foo';\n",
        false,
    );
    test_ts(
        "type A<T> = { [K in keyof T as K extends string ? B<K> : K ]: T[K] }",
        "type A<T> = { [K in keyof T as K extends string ? B<K> : K] : T[K]};\n",
        false,
    );
    test_ts(
        "class A {readonly type = 'frame'}",
        "class A {\n\treadonly type = 'frame';\n}\n",
        false,
    );
    test_ts("let foo: { <T>(t: T): void }", "let foo: {<T>(t: T): void};\n", false);
    test_ts("let foo: { new <T>(t: T): void }", "let foo: {new <T>(t: T): void};\n", false);
    test_ts("function <const T>(){}", "function<const T>() {}\n", false);
    test_ts("class A {m?(): void}", "class A {\n\tm?(): void;\n}\n", false);
    test_ts(
        "class A {constructor(public readonly a: number) {}}",
        "class A {\n\tconstructor(public readonly a: number) {}\n}\n",
        false,
    );
    test_ts(
        "abstract class A {private abstract static m() {}}",
        "abstract class A {\n\tprivate abstract static m() {}\n}\n",
        false,
    );
    test_ts(
        "abstract class A {private abstract static readonly prop: string}",
        "abstract class A {\n\tprivate abstract static readonly prop: string;\n}\n",
        false,
    );
}

#[test]
fn unicode_escape() {
    test("console.log('你好');", "console.log('你好');\n");
    test("console.log('こんにちは');", "console.log('こんにちは');\n");
    test("console.log('안녕하세요');", "console.log('안녕하세요');\n");
    test("console.log('🧑‍🤝‍🧑');", "console.log('🧑‍🤝‍🧑');\n");
}
