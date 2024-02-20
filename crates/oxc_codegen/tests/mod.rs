use oxc_allocator::Allocator;
use oxc_codegen::{Codegen, CodegenOptions};
use oxc_parser::Parser;
use oxc_span::SourceType;

fn test(source_text: &str, expected: &str) {
    let allocator = Allocator::default();
    let source_type = SourceType::default().with_module(true);
    let program = Parser::new(&allocator, source_text, source_type).parse().program;
    let program = allocator.alloc(program);
    let result = Codegen::<false>::new(source_text.len(), CodegenOptions::default()).build(program);
    assert_eq!(expected, result, "for source {source_text}, expect {expected}, got {result}");
}

fn test_ts_same_out(source_text: &str) {
    test_ts(source_text, &format!("{source_text}\n"), false);
}

fn test_ts(source_text: &str, expected: &str, is_typescript_definition: bool) {
    let allocator = Allocator::default();
    let source_type = SourceType::default()
        .with_typescript(true)
        .with_typescript_definition(is_typescript_definition)
        .with_module(true);
    let program = Parser::new(&allocator, source_text, source_type).parse().program;
    let program = allocator.alloc(program);
    let result =
        Codegen::<false>::new(source_text.len(), CodegenOptions { enable_typescript: true })
            .build(program);
    assert_eq!(expected, result, "for source {source_text}, expect {expected}, got {result}");
}

#[test]
fn string() {
    test("let x = ''", "let x = '';\n");
    test(r"let x = '\b'", "let x = '\\b';\n");
    test(r"let x = '\f'", "let x = '\\f';\n");
    test("let x = '\t'", "let x = '\\t';\n");
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
    // test( "let x = '\\01'", "let x = '\x01';\n");
    // test( "let x = '\x10'", "let x = '\x10';\n");
    // test( "let x = '\\x10'", "let x = '\x10';\n");
    test("let x = '\x1B'", "let x = '\\x1B';\n");
    test("let x = '\\x1B'", "let x = '\\x1B';\n");
    // test( r#"let x = '\uABCD'"#, r#"let x = "\uABCD";"#);
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
    test("import {} from './foo.js' with {}", "import './foo.js' with {\n};\n");
    test("export * from './foo.js' with {}", "export * from './foo.js' with {\n};\n");
}

#[test]
fn new_expr() {
    test("new (foo()).bar();", "new (foo()).bar();\n");
}

#[test]
fn typescript() {
    test_ts_same_out("let x: string = `\\x01`;");

    test_ts_same_out("function foo<T extends string>(x: T, y: string, ...restOfParams: Omit<T, 'x'>): T {\n\treturn x;\n}");

    test_ts_same_out("let x: string[] = ['abc', 'def', 'ghi'];");
    test_ts_same_out("let x: Array<string> = ['abc', 'def', 'ghi',];");
    test_ts_same_out("let x: [string, number] = ['abc', 123];");
    test_ts_same_out("let x: string | number = 'abc';");
    test_ts_same_out("let x: string & number = 'abc';");
    test_ts_same_out("let x: typeof String = 'string';");
    test_ts_same_out("let x: keyof string = 'length';");
    test_ts_same_out("let x: keyof typeof String = 'length';");
    test_ts_same_out("let x: string['length'] = 123;");

    test_ts_same_out("function isString(value: unknown): asserts value is string {\n\tif (typeof value !== 'string') {\n\t\tthrow new Error('Not a string');\n\t}\n}");
}
