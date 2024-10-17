use crate::tester::{test, test_minify, test_without_source};

#[test]
fn module_decl() {
    test("export * as foo from 'foo'", "export * as foo from \"foo\";\n");
    test("import x from './foo.js' with {}", "import x from \"./foo.js\" with {\n};\n");
    test("import {} from './foo.js' with {}", "import {} from \"./foo.js\" with {\n};\n");
    test("export * from './foo.js' with {}", "export * from \"./foo.js\" with {\n};\n");
}

#[test]
fn expr() {
    test("new (foo()).bar();", "new (foo()).bar();\n");
    test(
        "class Foo { #test
          bar() { if (!(#test in Foo)) { } }
        }",
        "class Foo {\n\t#test;\n\tbar() {\n\t\tif (!(#test in Foo)) {}\n\t}\n}\n",
    );
    test_minify("x in new Error()", "x in new Error();");

    test("1000000000000000128.0.toFixed(0)", "0xde0b6b3a7640080.toFixed(0);\n");
    test_minify("1000000000000000128.0.toFixed(0)", "0xde0b6b3a7640080.toFixed(0);");
}

#[test]
fn access_property() {
    test(
        "export default class Foo { @x @y accessor #aDef = 1 }",
        "export default class Foo {\n\t@x @y accessor #aDef = 1;\n}\n",
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

#[test]
fn regex() {
    fn test_all(source: &str, expect: &str, minify: &str) {
        test(source, expect);
        test_minify(source, minify);
        test_without_source(source, expect);
    }
    test_all("/regex/giv", "/regex/giv;\n", "/regex/giv;");
    test_all(
        r"/(.)(.)(.)(.)(.)(.)(.)(.)\8\8/",
        "/(.)(.)(.)(.)(.)(.)(.)(.)\\8\\8/;\n",
        "/(.)(.)(.)(.)(.)(.)(.)(.)\\8\\8/;",
    );

    test_all(
        r"/\n\cM\0\x41\u{1f600}\./u",
        "/\\n\\cM\\0\\x41\\u{1f600}\\./u;\n",
        "/\\n\\cM\\0\\x41\\u{1f600}\\./u;",
    );
    test_all(r"/\n\cM\0\x41\./u", "/\\n\\cM\\0\\x41\\./u;\n", "/\\n\\cM\\0\\x41\\./u;");
    test_all(
        r"/\n\cM\0\x41\u1234\./",
        "/\\n\\cM\\0\\x41\\u1234\\./;\n",
        "/\\n\\cM\\0\\x41\\u1234\\./;",
    );
}

#[test]
fn comma() {
    test("[1, 2, 3]", "[\n\t1,\n\t2,\n\t3\n];\n");
    test("[1, 2, 3,]", "[\n\t1,\n\t2,\n\t3\n];\n");
    test("[,]", "[,];\n");
    test("[,,]", "[, ,];\n");
    test("[,1]", "[, 1];\n");

    test_minify("1, 2, 3", "1,2,3;");
    test_minify("1, a = b, 3", "1,a=b,3;");
    test_minify("1, (2, 3), 4", "1,2,3,4;");
}

#[test]
fn assignment() {
    test("(let[0] = 100);", "(let)[0] = 100;\n");
    test_minify("a = b ? c : d", "a=b?c:d;");
    test_minify("[a,b] = (1, 2)", "[a,b]=(1,2);");
    // `{a,b}` is a block, must wrap the whole expression to be an assignment expression
    test_minify("({a,b} = (1, 2))", "({a,b}=(1,2));");
    test_minify("a *= yield b", "a*=yield b;");
    test_minify("a /= () => {}", "a/=()=>{};");
    test_minify("a %= async () => {}", "a%=async ()=>{};");
    test_minify("a -= (1, 2)", "a-=(1,2);");
    test_minify("a >>= b >>= c", "a>>=b>>=c;");
}

#[test]
fn r#yield() {
    test_minify("function *foo() { yield }", "function*foo(){yield}");
    test_minify("function *foo() { yield * a ? b : c }", "function*foo(){yield*a?b:c}");
    test_minify("function *foo() { yield * yield * a }", "function*foo(){yield*yield*a}");
    test_minify("function *foo() { yield * () => {} }", "function*foo(){yield*()=>{}}");
    test_minify("function *foo() { yield * async () => {} }", "function*foo(){yield*async ()=>{}}");
    test_minify("function *foo() { yield a ? b : c }", "function*foo(){yield a?b:c}");
    test_minify("function *foo() { yield yield a }", "function*foo(){yield yield a}");
    test_minify("function *foo() { yield () => {} }", "function*foo(){yield ()=>{}}");
    test_minify("function *foo() { yield async () => {} }", "function*foo(){yield async ()=>{}}");
    test_minify(
        "function *foo() { yield { a } = [ b ] = c ? b : d }",
        "function*foo(){yield {a}=[b]=c?b:d}",
    );
    // TODO: remove the extra space in `yield (a,b)`
    test_minify("function *foo() { yield (a, b) }", "function*foo(){yield (a,b)}");
    test_minify("function *foo() { yield a, b }", "function*foo(){yield a,b}");
}

#[test]
fn arrow() {
    test_minify("x => a, b", "(x)=>a,b;");
    test_minify("x => (a, b)", "(x)=>(a,b);");
    test_minify("x => (a => b)", "(x)=>(a)=>b;");
    test_minify("x => y => a, b", "(x)=>(y)=>a,b;");
    test_minify("x => y => (a = b)", "(x)=>(y)=>a=b;");
    test_minify("x => y => z => a = b, c", "(x)=>(y)=>(z)=>a=b,c;");
    test_minify("x => y => z => a = (b, c)", "(x)=>(y)=>(z)=>a=(b,c);");
    test_minify("x => ({} + 0)", "(x)=>({})+0;");
}

#[test]
fn conditional() {
    test_minify("a ? b : c", "a?b:c;");
    test_minify("a ? (b, c) : (d, e)", "a?(b,c):(d,e);");
    test_minify("a ? b : c ? b : c", "a?b:c?b:c;");
    test_minify("(a ? b : c) ? b : c", "(a?b:c)?b:c;");
    test_minify("a, b ? c : d", "a,b?c:d;");
    test_minify("(a, b) ? c : d", "(a,b)?c:d;");
    test_minify("a = b ? c : d", "a=b?c:d;");
    test_minify("(a = b) ? c : d", "(a=b)?c:d;");
}

#[test]
fn coalesce() {
    test_minify("a ?? b", "a??b;");
    test_minify("a ?? b ?? c ?? d", "a??b??c??d;");
    test_minify("a ?? (b ?? (c ?? d))", "a??(b??(c??d));");
    test_minify("(a ?? (b ?? (c ?? d)))", "a??(b??(c??d));");
    test_minify("a, b ?? c", "a,b??c;");
    test_minify("(a, b) ?? c", "(a,b)??c;");
    test_minify("a, b ?? c, d", "a,b??c,d;");
    test_minify("a, b ?? (c, d)", "a,b??(c,d);");
    test_minify("a = b ?? c", "a=b??c;");
    test_minify("a ?? (b = c)", "a??(b=c);");
    test_minify("(a | b) ?? (c | d)", "a|b??c|d;");
}

#[test]
fn logical_or() {
    test_minify("a || b || c", "a||b||c;");
    test_minify("(a || (b || c)) || d", "a||(b||c)||d;");
    test_minify("a || (b || (c || d))", "a||(b||(c||d));");
    test_minify("a || b && c", "a||b&&c;");
    test_minify("(a || b) && c", "(a||b)&&c;");
    test_minify("a, b || c, d", "a,b||c,d;");
    test_minify("(a, b) || (c, d)", "(a,b)||(c,d);");
    test_minify("(a && b) || (c && d)", "a&&b||c&&d;");
    test_minify("a && b || c && d", "a&&b||c&&d;");
}

#[test]
fn logical_and() {
    test_minify("a && b && c", "a&&b&&c;");
    test_minify("a && ((b && c) && d)", "a&&(b&&c&&d);");
    test_minify("((a && b) && c) && d", "a&&b&&c&&d;");
    test_minify("(a || b) && (c || d)", "(a||b)&&(c||d);");
    test_minify("a, b && c, d", "a,b&&c,d;");
    test_minify("(a, b) && (c, d)", "(a,b)&&(c,d);");
    test_minify("a || b && c || d", "a||b&&c||d;");
}

#[test]
fn bitwise_or() {
    test_minify("a | b | c", "a|b|c;");
    test_minify("(a | b) | c", "a|b|c;");
    test_minify("a | (b | c)", "a|(b|c);");
    test_minify("a | b ^ c", "a|b^c;");
    test_minify("a | (b ^ c)", "a|b^c;");
    test_minify("a | (b && c)", "a|(b&&c);");
    test_minify("a | b && c", "a|b&&c;");
    test_minify("(a ^ b) | (c ^ d)", "a^b|c^d;");
    test_minify("(a, b) | (c, d)", "(a,b)|(c,d);");
    test_minify("a, b | c, d", "a,b|c,d;");
}

#[test]
fn bitwise_xor() {
    test_minify("a ^ b ^ c", "a^b^c;");
    test_minify("(a ^ b) ^ c", "a^b^c;");
    test_minify("a ^ (b ^ c)", "a^(b^c);");
    test_minify("a | b & c", "a|b&c;");
    test_minify("a | (b & c)", "a|b&c;");
    test_minify("a | (b || c)", "a|(b||c);");
    test_minify("a | b || c", "a|b||c;");
    test_minify("(a, b) ^ (c, d)", "(a,b)^(c,d);");
    test_minify("(a | b) ^ (c | d)", "(a|b)^(c|d);");
    test_minify("a, b ^ c, d", "a,b^c,d;");
}

#[test]
fn bitwise_and() {
    test_minify("a & b & c", "a&b&c;");
    test_minify("((a & b) & c) & d", "a&b&c&d;");
    test_minify("a & (b & (c & d))", "a&(b&(c&d));");
    test_minify("a & b == c", "a&b==c;");
    test_minify("a & (b == c)", "a&b==c;");
    test_minify("a == b & c", "a==b&c;");
    test_minify("(a == b) & c", "a==b&c;");
    test_minify("a ^ b & c", "a^b&c;");
    test_minify("(a ^ b) & c", "(a^b)&c;");
    test_minify("(a, b) & (c, d)", "(a,b)&(c,d);");
    test_minify("a, b & c, d", "a,b&c,d;");
}

#[test]
fn equality() {
    test_minify("a == b != c === d !== e", "a==b!=c===d!==e;");
    test_minify("a == (b != (c === (d !== e)))", "a==(b!=(c===(d!==e)));");
    test_minify("(((a == b) != c) === d) !== e", "a==b!=c===d!==e;");
    test_minify("a > b == c < d", "a>b==c<d;");
    test_minify("(a > b) == (c < d)", "a>b==c<d;");
    test_minify("a | b == c & d", "a|b==c&d;");
    test_minify("(a | b) == (c & d)", "(a|b)==(c&d);");
    test_minify("a, b == c , d", "a,b==c,d;");
    test_minify("(a, b) == (c , d)", "(a,b)==(c,d);");
}

#[test]
fn vite_special_comments() {
    test(
        "new URL(/* @vite-ignore */ 'non-existent', import.meta.url)",
        "new URL(\n\t/* @vite-ignore */\n\t\"non-existent\",\n\timport.meta.url\n);\n",
    );
    test(
        "const importPromise = import(\n/* @vite-ignore */\nbase + '.js'\n);",
        "const importPromise = import(\n\t/* @vite-ignore */\n\tbase + \".js\"\n);\n",
    );
    test(
        "import(/* @vite-ignore */ module1Url).then((module1) => {\nself.postMessage(module.default + module1.msg1 + import.meta.env.BASE_URL)})",
        "import(\n\t/* @vite-ignore */\n\tmodule1Url\n).then((module1) => {\n\tself.postMessage(module.default + module1.msg1 + import.meta.env.BASE_URL);\n});\n",
    );
}

// followup from https://github.com/oxc-project/oxc/pull/6422
#[test]
fn in_expr_in_sequence_in_for_loop_init() {
    test(
        "for (l = ('foo' in bar), i; i < 10; i += 1) {}",
        "for (l = (\"foo\" in bar), i; i < 10; i += 1) {}\n",
    );

    test(
        "for (('hidden' in a) && (m = a.hidden), r = 0; s > r; r++) {}",
        "for ((\"hidden\" in a) && (m = a.hidden), r = 0; s > r; r++) {}\n",
    );
}
