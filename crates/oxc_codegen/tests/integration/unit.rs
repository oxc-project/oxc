use crate::tester::test;

#[test]
fn module_decl() {
    test("export * as foo from 'foo'", "export * as foo from \"foo\";\n");
    test("import x from './foo.js' with {}", "import x from \"./foo.js\" with {\n};\n");
    test("import {} from './foo.js' with {}", "import {} from \"./foo.js\" with {\n};\n");
    test("export * from './foo.js' with {}", "export * from \"./foo.js\" with {\n};\n");
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
    test("let _ = { x }", "let _ = { x };\n");
    test("let { x } = y", "let { x } = y;\n");
    test("({ x: (x) })", "({ x });\n");
    test("({ x } = y)", "({x} = y);\n");
}

#[test]
fn unicode_escape() {
    test("console.log('你好');", "console.log(\"你好\");\n");
    test("console.log('こんにちは');", "console.log(\"こんにちは\");\n");
    test("console.log('안녕하세요');", "console.log(\"안녕하세요\");\n");
    test("console.log('🧑‍🤝‍🧑');", "console.log(\"🧑‍🤝‍🧑\");\n");
}
