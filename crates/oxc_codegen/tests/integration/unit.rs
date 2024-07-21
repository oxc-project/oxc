use crate::tester::test;

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
fn shorthand() {
    test("let _ = { x }", "let _ = {x};\n");
    test("let { x } = y", "let { x } = y;\n");
    test("({ x: (x) })", "({x});\n");
    test("({ x } = y)", "({x} = y);\n");
}

#[test]
fn unicode_escape() {
    test("console.log('你好');", "console.log('你好');\n");
    test("console.log('こんにちは');", "console.log('こんにちは');\n");
    test("console.log('안녕하세요');", "console.log('안녕하세요');\n");
    test("console.log('🧑‍🤝‍🧑');", "console.log('🧑‍🤝‍🧑');\n");
}
