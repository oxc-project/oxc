use oxc_allocator::Allocator;
use oxc_codegen::{CodeGenerator, CommentOptions};
use oxc_parser::Parser;
use oxc_span::SourceType;

fn test(source_text: &str, expected: &str) {
    let allocator = Allocator::default();
    let source_type = SourceType::default().with_module(true);
    let ret = Parser::new(&allocator, source_text, source_type).parse();
    let result = CodeGenerator::new().build(&ret.program).source_text;
    assert_eq!(expected, result, "for source {source_text}, expect {expected}, got {result}");
}

fn test_ts(source_text: &str, expected: &str, is_typescript_definition: bool) {
    let allocator = Allocator::default();
    let source_type = SourceType::default()
        .with_typescript(true)
        .with_typescript_definition(is_typescript_definition)
        .with_module(true);
    let ret = Parser::new(&allocator, source_text, source_type).parse();
    let result = CodeGenerator::new().build(&ret.program).source_text;
    assert_eq!(expected, result, "for source {source_text}, expect {expected}, got {result}");
}

#[test]
fn string() {
    test("let x = ''", "let x = '';\n");
    test(r"let x = '\b'", "let x = '\\b';\n");
    test(r"let x = '\f'", "let x = '\\f';\n");
    test("let x = '\t'", "let x = '\t';\n");
    test(r"let x = '\v'", "let x = '\\v';\n");
    test("let x = '\\n'", "let x = '\\n';\n");
    test("let x = '\\''", "let x = \"'\";\n");
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

    test("await tag`x`", "await tag`x`;\n");
    test("await (tag`x`)", "await tag`x`;\n");
    test("(await tag)`x`", "(await tag)`x`;\n");

    test("await tag`${x}`", "await tag`${x}`;\n");
    test("await (tag`${x}`)", "await tag`${x}`;\n");
    test("(await tag)`${x}`", "(await tag)`${x}`;\n");

    test("new tag`x`", "new tag`x`();\n");
    test("new (tag`x`)", "new tag`x`();\n");
    test("new tag()`x`", "new tag()`x`;\n");
    test("(new tag)`x`", "new tag()`x`;\n");

    test("new tag`${x}`", "new tag`${x}`();\n");
    test("new (tag`${x}`)", "new tag`${x}`();\n");
    test("new tag()`${x}`", "new tag()`${x}`;\n");
    test("(new tag)`${x}`", "new tag()`${x}`;\n");
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
fn for_stmt() {
    test("for (let x = 0; x < 10; x++) {}", "for (let x = 0; x < 10; x++) {}\n");
    test("for (;;) {}", "for (;;) {}\n");
    test("for (let x = 1;;) {}", "for (let x = 1;;) {}\n");
    test("for (;true;) {}", "for (; true;) {}\n");
    test("for (;;i++) {}", "for (;; i++) {}\n");

    test("for (using x = 1;;) {}", "for (using x = 1;;) {}\n");
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
        "let x: (string)[] = ['abc', 'def', 'ghi'];\n",
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
    test_ts("let x: string | number = 'abc';", "let x: ((string) | (number)) = 'abc';\n", false);
    test_ts("let x: string & number = 'abc';", "let x: ((string) & (number)) = 'abc';\n", false);
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
}

fn test_comment_helper(source_text: &str, expected: &str) {
    let allocator = Allocator::default();
    let source_type = SourceType::default().with_module(true);
    let ret = Parser::new(&allocator, source_text, source_type).parse();
    let result = CodeGenerator::new()
        .enable_comment(
            source_text,
            ret.trivias,
            CommentOptions { preserve_annotate_comments: true },
        )
        .build(&ret.program)
        .source_text;
    assert_eq!(expected, result, "for source {source_text}, expect {expected}, got {result}");
}

#[test]
fn annotate_comment() {
    test_comment_helper(
        r"
    				x([
    					/* #__NO_SIDE_EFFECTS__ */ function() {},
    					/* #__NO_SIDE_EFFECTS__ */ function y() {},
    					/* #__NO_SIDE_EFFECTS__ */ function*() {},
    					/* #__NO_SIDE_EFFECTS__ */ function* y() {},
    					/* #__NO_SIDE_EFFECTS__ */ async function() {},
    					/* #__NO_SIDE_EFFECTS__ */ async function y() {},
    					/* #__NO_SIDE_EFFECTS__ */ async function*() {},
    					/* #__NO_SIDE_EFFECTS__ */ async function* y() {},
    				])",
        r"x([/* #__NO_SIDE_EFFECTS__ */ function() {}, /* #__NO_SIDE_EFFECTS__ */ function y() {}, /* #__NO_SIDE_EFFECTS__ */ function* () {}, /* #__NO_SIDE_EFFECTS__ */ function* y() {}, /* #__NO_SIDE_EFFECTS__ */ async function() {}, /* #__NO_SIDE_EFFECTS__ */ async function y() {}, /* #__NO_SIDE_EFFECTS__ */ async function* () {}, /* #__NO_SIDE_EFFECTS__ */ async function* y() {},]);
",
    );

    test_comment_helper(
        r"
            x([
    					/* #__NO_SIDE_EFFECTS__ */ y => y,
    					/* #__NO_SIDE_EFFECTS__ */ () => {},
    					/* #__NO_SIDE_EFFECTS__ */ (y) => (y),
    					/* #__NO_SIDE_EFFECTS__ */ async y => y,
    					/* #__NO_SIDE_EFFECTS__ */ async () => {},
    					/* #__NO_SIDE_EFFECTS__ */ async (y) => (y),
    				])",
        r"x([/* #__NO_SIDE_EFFECTS__ */ (y) => y, /* #__NO_SIDE_EFFECTS__ */ () => {}, /* #__NO_SIDE_EFFECTS__ */ (y) => y, /* #__NO_SIDE_EFFECTS__ */ async (y) => y, /* #__NO_SIDE_EFFECTS__ */ async () => {}, /* #__NO_SIDE_EFFECTS__ */ async (y) => y,]);
",
    );
    test_comment_helper(
        r"
            x([
    					/* #__NO_SIDE_EFFECTS__ */ y => y,
    					/* #__NO_SIDE_EFFECTS__ */ () => {},
    					/* #__NO_SIDE_EFFECTS__ */ (y) => (y),
    					/* #__NO_SIDE_EFFECTS__ */ async y => y,
    					/* #__NO_SIDE_EFFECTS__ */ async () => {},
    					/* #__NO_SIDE_EFFECTS__ */ async (y) => (y),
    				])",
        r"x([/* #__NO_SIDE_EFFECTS__ */ (y) => y, /* #__NO_SIDE_EFFECTS__ */ () => {}, /* #__NO_SIDE_EFFECTS__ */ (y) => y, /* #__NO_SIDE_EFFECTS__ */ async (y) => y, /* #__NO_SIDE_EFFECTS__ */ async () => {}, /* #__NO_SIDE_EFFECTS__ */ async (y) => y,]);
",
    );
    //
    test_comment_helper(
        r"
    // #__NO_SIDE_EFFECTS__
    function a() {}
    // #__NO_SIDE_EFFECTS__
    function* b() {}
    // #__NO_SIDE_EFFECTS__
    async function c() {}
    // #__NO_SIDE_EFFECTS__
    async function* d() {}
            ",
        r"// #__NO_SIDE_EFFECTS__
function a() {}
// #__NO_SIDE_EFFECTS__
function* b() {}
// #__NO_SIDE_EFFECTS__
async function c() {}
// #__NO_SIDE_EFFECTS__
async function* d() {}
",
    );

    test_comment_helper(
        r"
    // #__NO_SIDE_EFFECTS__
    function a() {}
    // #__NO_SIDE_EFFECTS__
    function* b() {}
    // #__NO_SIDE_EFFECTS__
    async function c() {}
    // #__NO_SIDE_EFFECTS__
    async function* d() {}
            ",
        r"// #__NO_SIDE_EFFECTS__
function a() {}
// #__NO_SIDE_EFFECTS__
function* b() {}
// #__NO_SIDE_EFFECTS__
async function c() {}
// #__NO_SIDE_EFFECTS__
async function* d() {}
",
    );

    test_comment_helper(
        r"
    /* @__NO_SIDE_EFFECTS__ */ export function a() {}
    /* @__NO_SIDE_EFFECTS__ */ export function* b() {}
    /* @__NO_SIDE_EFFECTS__ */ export async function c() {}
    /* @__NO_SIDE_EFFECTS__ */ export async function* d() {}",
        r"/* @__NO_SIDE_EFFECTS__ */ export function a() {}
/* @__NO_SIDE_EFFECTS__ */ export function* b() {}
/* @__NO_SIDE_EFFECTS__ */ export async function c() {}
/* @__NO_SIDE_EFFECTS__ */ export async function* d() {}
",
    );
    // Only "c0" and "c2" should have "no side effects" (Rollup only respects "const" and only for the first one)
    test_comment_helper(
        r"
        					/* #__NO_SIDE_EFFECTS__ */ export var v0 = function() {}, v1 = function() {}
        					/* #__NO_SIDE_EFFECTS__ */ export let l0 = function() {}, l1 = function() {}
        					/* #__NO_SIDE_EFFECTS__ */ export const c0 = function() {}, c1 = function() {}
        					/* #__NO_SIDE_EFFECTS__ */ export var v2 = () => {}, v3 = () => {}
        					/* #__NO_SIDE_EFFECTS__ */ export let l2 = () => {}, l3 = () => {}
        					/* #__NO_SIDE_EFFECTS__ */ export const c2 = () => {}, c3 = () => {}
        ",
        r"export var v0 = function() {}, v1 = function() {};
export let l0 = function() {}, l1 = function() {};
export const c0 = /* #__NO_SIDE_EFFECTS__ */ function() {}, c1 = function() {};
export var v2 = () => {}, v3 = () => {};
export let l2 = () => {}, l3 = () => {};
export const c2 = /* #__NO_SIDE_EFFECTS__ */ () => {}, c3 = () => {};
",
    );
    // Only "c0" and "c2" should have "no side effects" (Rollup only respects "const" and only for the first one)
    test_comment_helper(
        r"
    /* #__NO_SIDE_EFFECTS__ */ var v0 = function() {}, v1 = function() {}
    /* #__NO_SIDE_EFFECTS__ */ let l0 = function() {}, l1 = function() {}
    /* #__NO_SIDE_EFFECTS__ */ const c0 = function() {}, c1 = function() {}
    /* #__NO_SIDE_EFFECTS__ */ var v2 = () => {}, v3 = () => {}
    /* #__NO_SIDE_EFFECTS__ */ let l2 = () => {}, l3 = () => {}
    /* #__NO_SIDE_EFFECTS__ */ const c2 = () => {}, c3 = () => {}
        ",
        r"var v0 = function() {}, v1 = function() {};
let l0 = function() {}, l1 = function() {};
const c0 = /* #__NO_SIDE_EFFECTS__ */ function() {}, c1 = function() {};
var v2 = () => {}, v3 = () => {};
let l2 = () => {}, l3 = () => {};
const c2 = /* #__NO_SIDE_EFFECTS__ */ () => {}, c3 = () => {};
",
    );
}

#[test]
fn unicode_escape() {
    test("console.log('你好');", "console.log('你好');\n");
    test("console.log('こんにちは');", "console.log('こんにちは');\n");
    test("console.log('안녕하세요');", "console.log('안녕하세요');\n");
    test("console.log('🧑‍🤝‍🧑');", "console.log('🧑‍🤝‍🧑');\n");
}

#[test]
fn ts_misc() {
    test_ts(
        "type A<T> = { [K in keyof T as K extends string ? B<K> : K ]: T[K] }",
        "type A<T> = { [K in keyof T as K extends (string) ? B<K> : K] : T[K]};\n",
        false,
    );
}
