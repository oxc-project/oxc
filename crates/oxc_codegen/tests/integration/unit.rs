use crate::tester::{test, test_minify};

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

#[test]
fn comma() {
    test_minify("1, 2, 3", "1,2,3;");
    test_minify("1, a = b, 3", "1,a=b,3;");
    test_minify("1, (2, 3), 4", "1,2,3,4;");
}

#[test]
fn assignment() {
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
