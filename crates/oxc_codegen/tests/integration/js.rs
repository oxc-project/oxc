use oxc_codegen::{CodegenOptions, IndentChar};

use crate::tester::{
    test, test_minify, test_minify_same, test_options, test_same, test_with_parse_options,
};

#[test]
fn cases() {
    test_same("class C {\n\t@foo static accessor A = @bar class {};\n}\n");
    test_same("function foo(@foo x = @bar class {}) {}\n");
}

#[test]
fn decl() {
    test_minify("const [foo] = bar", "const[foo]=bar;");
    test_minify("const {foo} = bar", "const{foo}=bar;");
    test_minify("const foo = bar", "const foo=bar;");
}

#[test]
fn module_decl() {
    test("export * as foo from 'foo'", "export * as foo from \"foo\";\n");
    test("import x from './foo.js' with {}", "import x from \"./foo.js\" with {};\n");
    test("import {} from './foo.js' with {}", "import {} from \"./foo.js\" with {};\n");
    test("export * from './foo.js' with {}", "export * from \"./foo.js\" with {};\n");
    test_minify("export { '☿' } from 'mod';", "export{\"☿\"}from\"mod\";");
    test_minify("export { '☿' as '☿' } from 'mod';", "export{\"☿\"}from\"mod\";");
    test_minify(
        "import x from './foo.custom' with { 'type': 'json' }",
        "import x from\"./foo.custom\"with{\"type\":\"json\"};",
    );
}

#[test]
fn export_type() {
    test_same("export type {} from \"mod\";\n");
    test_same("export type { Foo } from \"mod\";\n");
    test_same("export { type Foo, type Bar } from \"mod\";\n");
}

#[test]
fn expr() {
    test("new (foo()).bar();", "new (foo()).bar();\n");
    test_minify("x in new Error()", "x in new Error;");
    test(
        "new function() { let a = foo?.bar().baz; return a; }();",
        "new function() {\n\tlet a = foo?.bar().baz;\n\treturn a;\n}();\n",
    );
    test(
        "new class { foo() { let a = foo?.bar().baz; return a; } }();",
        "new class {\n\tfoo() {\n\t\tlet a = foo?.bar().baz;\n\t\treturn a;\n\t}\n}();\n",
    );

    test("1000000000000000128.0.toFixed(0)", "0xde0b6b3a7640080.toFixed(0);\n");
    test_minify("1000000000000000128.0.toFixed(0)", "0xde0b6b3a7640080.toFixed(0);");

    test_minify("throw 'foo'", "throw`foo`;");
    test_minify("return 'foo'", "return`foo`;");
    test_minify("return class {}", "return class{};");
    test_minify("return async function foo() {}", "return async function foo(){};");
    test_minify_same("return super();");
    test_minify_same("return new.target;");
    test_minify_same("throw await 1;");
    test_minify_same("await import(``);");

    test("delete 2e308", "delete (0, Infinity);\n");
    test_minify("delete 2e308", "delete(1/0);");

    test_minify_same(r#"({"http://a\r\" \n<'b:b@c\r\nd/e?f":{}});"#);
    test_minify_same("new(import(``),function(){});");
}

#[test]
fn private_in() {
    test(
        "class Foo { #test; bar() { if (!(#test in Foo)) { } } }",
        "class Foo {\n\t#test;\n\tbar() {\n\t\tif (!(#test in Foo)) {}\n\t}\n}\n",
    );
    test(
        "class Foo { #test; bar() { #field in {} << 0 } }",
        "class Foo {\n\t#test;\n\tbar() {\n\t\t#field in {} << 0;\n\t}\n}\n",
    );
}

#[test]
fn class() {
    test(
        "export default class Foo { @x @y accessor #aDef = 1 }",
        "export default class Foo {\n\t@x @y accessor #aDef = 1;\n}\n",
    );
    test(
        "export class Test2 {\n@decorator\nproperty: ((arg: any) => any) | undefined;\n}",
        "export class Test2 {\n\t@decorator property: ((arg: any) => any) | undefined;\n}\n",
    );
    test_same("export @dec1 @dec2 class C {}\n");
    test_same("export default @dec1 @dec2 class {}\n");
    test_minify("class { static [computed] }", "class{static[computed]}");
}

#[test]
fn for_stmt() {
    test("for (let x = 0; x < 10; x++) {}", "for (let x = 0; x < 10; x++) {}\n");
    test("for (;;) {}", "for (;;) {}\n");
    test("for (let x = 1;;) {}", "for (let x = 1;;) {}\n");
    test("for (;true;) {}", "for (; true;) {}\n");
    test("for (;;i++) {}", "for (;; i++) {}\n");

    test("for (using x = 1;;) {}", "for (using x = 1;;) {}\n");

    //  `in` expression
    test("for (x = (y in z) || y;;);", "for (x = (y in z) || y;;);\n");
    test("for (x = (y in z) || y || y;;);", "for (x = (y in z) || y || y;;);\n");
    test("for (x = y || y || (y in z);;);", "for (x = y || y || (y in z);;);\n");
    test(
        "for (x = (y in z) || y || (y in z) || y || (y in z);;);",
        "for (x = (y in z) || y || (y in z) || y || (y in z);;);\n",
    );
    test(
        "for (var a = 1 || (2 in {}) in { x: 1 }) count++;",
        "for (var a = 1 || (2 in {}) in { x: 1 }) count++;\n",
    );
}

#[test]
fn do_while_stmt() {
    test("do ; while (true)", "do;\nwhile (true);\n");
    test_minify("do ; while (true)", "do;while(true);");
    test_minify("do break; while (true)", "do break;while(true);");
    test_minify("do continue; while (true)", "do continue;while(true);");
    test_minify("do debugger; while (true)", "do debugger;while(true);");
    test_minify("do for(x in y); while (true)", "do for(x in y);while(true);");
    test_minify("do for(x of y); while (true)", "do for(x of y);while(true);");
    test_minify("do for(;;); while (true)", "do for(;;);while(true);");
    test_minify("do if (test) {} while (true)", "do if(test){}while(true);");
    test_minify("do foo:; while (true)", "do foo:;while(true);");
    test_minify("do return; while (true)", "do return;while(true);");
    test_minify("do switch(test){} while (true)", "do switch(test){}while(true);");
    test_minify("do throw x; while (true)", "do throw x;while(true);");
    test_minify("do with(x); while (true)", "do with(x);while(true);");
    test_minify("do try{} catch{} while (true)", "do try{}catch{}while(true);");
    test_minify("do do ; while(true) while (true)", "do do;while(true);while(true);");
}

#[test]
fn if_stmt() {
    test(
        "function f() { if (foo) return foo; else if (bar) return foo; }",
        "function f() {\n\tif (foo) return foo;\n\telse if (bar) return foo;\n}\n",
    );
    test_minify(
        "function f() { if (foo) return foo; else if (bar) return foo; }",
        "function f(){if(foo)return foo;else if(bar)return foo}",
    );
}

#[test]
fn shorthand() {
    test("let _ = { x }", "let _ = { x };\n");
    test("let { x } = y", "let { x } = y;\n");
    test("({ x: (x) })", "({ x });\n");
    test("({ x } = y)", "({x} = y);\n");
    // https://github.com/tc39/test262/blob/05c45a4c430ab6fee3e0c7f0d47d8a30d8876a6d/test/language/expressions/object/__proto__-permitted-dup-shorthand.js
    test("var obj = { __proto__, __proto__, };", "var obj = {\n\t__proto__,\n\t__proto__\n};\n");
    test("var obj = { __proto__: __proto__, };", "var obj = { __proto__: __proto__ };\n");
}

#[test]
fn unicode_escape() {
    test("console.log('你好');", "console.log(\"你好\");\n");
    test("console.log('こんにちは');", "console.log(\"こんにちは\");\n");
    test("console.log('안녕하세요');", "console.log(\"안녕하세요\");\n");
    test("console.log('🧑‍🤝‍🧑');", "console.log(\"🧑‍🤝‍🧑\");\n");
    test("console.log(\"\\uD800\\uD801\")", "console.log(\"\\ud800\\ud801\");\n");
}

#[test]
fn regex() {
    fn test_all(source: &str, expect: &str, minify: &str) {
        test(source, expect);
        test_minify(source, minify);
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
    test("[a, ...rest] = arr;", "[a, ...rest] = arr;\n");
    test("[...rest] = arr;", "[...rest] = arr;\n");
    test("({a, ...rest} = obj);", "({a, ...rest} = obj);\n");
    test("({...rest} = obj);", "({...rest} = obj);\n");
    test_minify("a = b ? c : d", "a=b?c:d;");
    test_minify("[a,b] = (1, 2)", "[a,b]=(1,2);");
    // `{a,b}` is a block, must wrap the whole expression to be an assignment expression
    test_minify("({a,b} = (1, 2))", "({a,b}=(1,2));");
    test_minify("a *= yield b", "a*=yield b;");
    test_minify("a /= () => {}", "a/=()=>{};");
    test_minify("a %= async () => {}", "a%=async()=>{};");
    test_minify("a -= (1, 2)", "a-=(1,2);");
    test_minify("({ x: x = flag1 = true } = {})", "({x=flag1=true}={});");

    test_minify("({ 0: x } = foo);", "({0:x}=foo);");
    test_minify("({ [0]: x } = foo);", "({[0]:x}=foo);");
    test_minify("({ a: x } = foo);", "({a:x}=foo);");
    test_minify("({ [a.b]: x } = foo);", "({[a.b]:x}=foo);");

    test_minify(r#"({"my-key": value} = obj);"#, r#"({"my-key":value}=obj);"#);
    test_minify(
        r#"({["computed"]: a, "literal": b} = obj);"#,
        r#"({["computed"]:a,"literal":b}=obj);"#,
    );
    test_minify(r#"let {"test-key": testKey} = obj;"#, r#"let{"test-key":testKey}=obj;"#);

    test_minify(r#"({ "test-key": key });"#, r#"({"test-key":key});"#);
    test_minify(r#"(class { "test-key" = key });"#, r#"(class{"test-key"=key});"#);
}

#[test]
fn r#yield() {
    test("function * foo() { yield * 1 }", "function* foo() {\n\tyield* 1;\n}\n");
    test_minify("function *foo() { yield }", "function*foo(){yield}");
    test_minify("function *foo() { yield * a ? b : c }", "function*foo(){yield*a?b:c}");
    test_minify("function *foo() { yield * yield * a }", "function*foo(){yield*yield*a}");
    test_minify("function *foo() { yield * () => {} }", "function*foo(){yield*()=>{}}");
    test_minify("function *foo() { yield * async () => {} }", "function*foo(){yield*async()=>{}}");
    test_minify("function *foo() { yield a ? b : c }", "function*foo(){yield a?b:c}");
    test_minify("function *foo() { yield yield a }", "function*foo(){yield yield a}");
    test_minify("function *foo() { yield () => {} }", "function*foo(){yield()=>{}}");
    test_minify("function *foo() { yield async () => {} }", "function*foo(){yield async()=>{}}");
    test_minify(
        "function *foo() { yield { a } = [ b ] = c ? b : d }",
        "function*foo(){yield{a}=[b]=c?b:d}",
    );
    test_minify("function *foo() { yield (a, b) }", "function*foo(){yield(a,b)}");
    test_minify("function *foo() { yield a, b }", "function*foo(){yield a,b}");
}

#[test]
fn arrow() {
    test_minify("x => a, b", "x=>a,b;");
    test_minify("x => (a, b)", "x=>(a,b);");
    test_minify("x => (a => b)", "x=>(a=>b);");
    test_minify("x => y => a, b", "x=>y=>a,b;");
    test_minify("x => y => (a = b)", "x=>y=>a=b;");
    test_minify("x => y => z => a = b, c", "x=>y=>z=>a=b,c;");
    test_minify("x => y => z => a = (b, c)", "x=>y=>z=>a=(b,c);");
    test_minify("x => ({} + 0)", "x=>({})+0;");
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
    test_minify("a ?? (b ?? (c ?? d))", "a??b??c??d;");
    test_minify("(a ?? (b ?? (c ?? d)))", "a??b??c??d;");
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
    test_minify("(a || (b || c)) || d", "a||b||c||d;");
    test_minify("a || (b || (c || d))", "a||b||c||d;");
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
    test_minify("a && ((b && c) && d)", "a&&b&&c&&d;");
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

#[test]
fn import_phase() {
    test_minify("import.defer('foo')", "import.defer(`foo`);");
    test_minify("import.source('foo')", "import.source(`foo`);");
    test("import.defer('foo')", "import.defer(\"foo\");\n");
    test("import.source('foo')", "import.source(\"foo\");\n");
}

// <https://github.com/javascript-compiler-hints/compiler-notations-spec/blob/main/pure-notation-spec.md#semantics>
#[test]
fn pure_comment() {
    test_same("/* @__PURE__ */ pureOperation();\n");
    test_same("/* @__PURE__ */ new PureConsutrctor();\n");
    test("/* @__PURE__ */\npureOperation();\n", "/* @__PURE__ */ pureOperation();\n");
    test(
        "/* @__PURE__ The comment may contain additional text */ pureOperation();\n",
        "/* @__PURE__ */ pureOperation();\n",
    );
    test("const foo /* #__PURE__ */ = pureOperation();", "const foo = pureOperation();\n"); // INVALID: "=" not allowed after annotation

    test("/* #__PURE__ */ function foo() {}\n", "function foo() {}\n");

    test("/* @__PURE__ */ (foo());", "/* @__PURE__ */ foo();\n");
    test("/* @__PURE__ */ (new Foo());\n", "/* @__PURE__ */ new Foo();\n");
    test("/*#__PURE__*/ (foo(), bar());", "foo(), bar();\n"); // INVALID, there is a comma expression in the parentheses

    test_same("/* @__PURE__ */ a.b().c.d();\n");
    test("/* @__PURE__ */ a().b;", "a().b;\n"); // INVALID, it does not end with a call
    test_same("(/* @__PURE__ */ a()).b;\n");

    // More
    test_same("/* @__PURE__ */ a() || b;\n");
    test_same("/* @__PURE__ */ a() && b;\n");
    test_same("/* @__PURE__ */ a() ?? b;\n");
    test_same("/* @__PURE__ */ a() ? b : c;\n");
    test_same("/* @__PURE__ */ a.b();\n");
    test_same("/* @__PURE__ */ a?.b();\n");
    test_same("true && /* @__PURE__ */ noEffect();\n");
    test_same("false || /* @__PURE__ */ noEffect();\n");
}

#[test]
fn pife() {
    test_same("foo((() => 0));\n");
    test_minify_same("foo((()=>0));");
    test_same("(() => 0)();\n");
    test_minify_same("(()=>0)();");
    test_same("foo((function() {\n\treturn 0;\n}));\n");
    test_minify_same("foo((function(){return0}));");
    test_same("(function() {\n\treturn 0;\n})();\n");
    test_minify_same("(function(){return0})();");
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

#[test]
fn in_expr_in_arrow_function_expression() {
    test("() => ('foo' in bar)", "() => \"foo\" in bar;\n");
    test("() => 'foo' in bar", "() => \"foo\" in bar;\n");
    test("() => { ('foo' in bar) }", "() => {\n\t\"foo\" in bar;\n};\n");
}

#[test]
fn big_int() {
    test("9007199254740991n;", "9007199254740991n;\n");
    test("-9007199254740991n;", "-9007199254740991n;\n");
    test("-90_0719_92547_40991n;", "-9007199254740991n;\n");
    test("+9007199254740991n;", "+9007199254740991n;\n");
    test("1000n", "1000n;\n");
    test("-15n", "-15n;\n");

    test("100_000_000n;", "100000000n;\n");
    test("10000000000000000n;", "10000000000000000n;\n");
    test("0n;", "0n;\n");
    test("+0n;", "+0n;\n");
    test("-0n;", "-0n;\n");

    test("0x1_0n;", "16n;\n");
    test("0x10n;", "16n;\n");

    test("0b1_01n;", "5n;\n");
    test("0b101n;", "5n;\n");
    test("0b101_101n;", "45n;\n");
    test("0b10_1n", "5n;\n");

    test("0o13n;", "11n;\n");
    test("0o7n", "7n;\n");

    test("0x2_0n", "32n;\n");
    test("0xfabn", "4011n;\n");
    test("0xaef_en;", "44798n;\n");
    test("0xaefen;", "44798n;\n");

    test("return 1n", "return 1n;\n");
    test_minify("return 1n", "return 1n;");
}

#[test]
#[ignore = "Minify bigint is not implemented."]
fn big_int_minify() {
    test_minify("9007199254740991n", "9007199254740991n;");
    test_minify("-9007199254740991n;", "-9007199254740991n;");
    test_minify("-90_0719_92547_40991n;", "-9007199254740991n;");
    test_minify("+9007199254740991n;", "+9007199254740991n;");
    test_minify("1000n", "1000n;");
    test_minify("-15n", "-15n;");

    test_minify("100_000_000n;", "100000000n;");
    test_minify("10000000000000000n;", "0x2386f26fc10000n;");
    test_minify("0n;", "0n;");
    test_minify("+0n;", "+0n;");
    test_minify("-0n;", "-0n;");

    test_minify("0x1_0n;", "16n;");
    test_minify("0x10n;", "16n;");

    test_minify("0b1_01n;", "5n;");
    test_minify("0b101n;", "5n;");
    test_minify("0b101_101n;", "45n;");
    test_minify("0b10_1n", "5n;");

    test_minify("0o13n;", "11n;");
    test_minify("0o7n", "7n;");

    test_minify("0x2_0n", "32n;");
    test_minify("0xfabn", "4011n;");
    test_minify("0xaef_en;", "44798n;");
    test_minify("0xaefen;", "44798n;");
}

#[test]
fn directive() {
    let single_quote = CodegenOptions { single_quote: true, ..CodegenOptions::default() };
    test_options("\"'\"", "\"'\";\n", single_quote.clone());
    test_options("'\"'", "'\"';\n", single_quote);
    let double_quote = CodegenOptions { single_quote: false, ..CodegenOptions::default() };
    test_options("\"'\"", "\"'\";\n", double_quote.clone());
    test_options("'\"'", "'\"';\n", double_quote.clone());
    test_options(r#""'\"""#, "\"'\\\"\";\n", double_quote);
}

#[test]
fn getter_setter() {
    test_minify("({ get [foo]() {} })", "({get[foo](){}});");
    test_minify("({ set [foo]() {} })", "({set[foo](){}});");
}

#[test]
fn string() {
    // `${` only escaped when quote is backtick
    test("let x = \"${}\";", "let x = \"${}\";\n");
    test_minify("let x = \"${}\";", "let x=\"${}\";");
    test("let x = '\"\"${}';", "let x = \"\\\"\\\"${}\";\n");
    test_minify("let x = '\"\"${}';", "let x='\"\"${}';");
    test("let x = '\"\"\\'\\'${}';", "let x = \"\\\"\\\"''${}\";\n");
    test_minify("let x = '\"\"\\'\\'${}';", "let x=`\"\"''\\${}`;");
    test_minify("let x = '\\'\\'\\'\"\"\"${}';", "let x=`'''\"\"\"\\${}`;");

    // Lossy replacement character
    test("let x = \"�\\u{FFFD}\";", "let x = \"��\";\n");
    test_minify("let x = \"�\\u{FFFD}\";", "let x=`��`;");
    test(
        "let x = \"� ��� \\u{FFFD} \\u{FFFD}\\u{FFFD}\\u{FFFD} �\";",
        "let x = \"� ��� � ��� �\";\n",
    );
    test_minify(
        "let x = \"� ��� \\u{FFFD} \\u{FFFD}\\u{FFFD}\\u{FFFD} �\";",
        "let x=`� ��� � ��� �`;",
    );
    // Lone surrogates
    test(
        "let x = \"\\uD800 \\uDBFF \\uDC00 \\uDFFF\";",
        "let x = \"\\ud800 \\udbff \\udc00 \\udfff\";\n",
    );
    test_minify(
        "let x = \"\\uD800 \\uDBFF \\uDC00 \\uDFFF\";",
        "let x=`\\ud800 \\udbff \\udc00 \\udfff`;",
    );
    test("let x = \"\\uD800\u{41}\";", "let x = \"\\ud800A\";\n");
    test_minify("let x = \"\\uD800\u{41}\";", "let x=`\\ud800A`;");
    // Invalid pairs
    test(
        "let x = \"\\uD800\\uDBFF \\uDC00\\uDFFF\";",
        "let x = \"\\ud800\\udbff \\udc00\\udfff\";\n",
    );
    test_minify(
        "let x = \"\\uD800\\uDBFF \\uDC00\\uDFFF\";",
        "let x=`\\ud800\\udbff \\udc00\\udfff`;",
    );
    // Lone surrogates and lossy replacement characters
    test(
        "let x = \"��\\u{FFFD}\\u{FFFD}\\uD800\\uDBFF��\\u{FFFD}\\u{FFFD}\\uDC00\\uDFFF��\\u{FFFD}\\u{FFFD}\";",
        "let x = \"����\\ud800\\udbff����\\udc00\\udfff����\";\n",
    );
    test_minify(
        "let x = \"��\\u{FFFD}\\u{FFFD}\\uD800\\uDBFF��\\u{FFFD}\\u{FFFD}\\uDC00\\uDFFF��\\u{FFFD}\\u{FFFD}\";",
        "let x=`����\\ud800\\udbff����\\udc00\\udfff����`;",
    );

    test_minify(
        r#";'eval("\'\\vstr\\ving\\v\'") === "\\vstr\\ving\\v"'"#,
        r#";`eval("'\\vstr\\ving\\v'") === "\\vstr\\ving\\v"`;"#,
    );
    test_minify(r#"foo("\n")"#, "foo(`\n`);");
}

#[test]
fn v8_intrinsics() {
    let parse_opts = oxc_parser::ParseOptions {
        allow_v8_intrinsics: true,
        ..oxc_parser::ParseOptions::default()
    };

    test_with_parse_options(
        "const p = %DebugPrint('hi')",
        "const p = %DebugPrint(\"hi\");\n",
        parse_opts,
    );
}

#[test]
fn indentation() {
    // Test default - tabs with width 1
    test_options(
        "if (true) {\nif (nested) {\nconsole.log('test');\n}\n}",
        "if (true) {\n\tif (nested) {\n\t\tconsole.log(\"test\");\n\t}\n}\n",
        CodegenOptions::default(),
    );

    // Test tabs with width 2
    test_options(
        "if (true) {\nif (nested) {\nconsole.log('test');\n}\n}",
        "if (true) {\n\t\tif (nested) {\n\t\t\t\tconsole.log(\"test\");\n\t\t}\n}\n",
        CodegenOptions {
            indent_char: IndentChar::Tab,
            indent_width: 2,
            ..CodegenOptions::default()
        },
    );

    // Test spaces with width 2
    test_options(
        "if (true) {\nif (nested) {\nconsole.log('test');\n}\n}",
        "if (true) {\n  if (nested) {\n    console.log(\"test\");\n  }\n}\n",
        CodegenOptions {
            indent_char: IndentChar::Space,
            indent_width: 2,
            ..CodegenOptions::default()
        },
    );

    // Test spaces with width 4
    test_options(
        "if (true) {\nif (nested) {\nconsole.log('test');\n}\n}",
        "if (true) {\n    if (nested) {\n        console.log(\"test\");\n    }\n}\n",
        CodegenOptions {
            indent_char: IndentChar::Space,
            indent_width: 4,
            ..CodegenOptions::default()
        },
    );

    // Test initial indent with 1
    test_options(
        "let foo = 1;",
        "\tlet foo = 1;\n",
        CodegenOptions { initial_indent: 1, ..CodegenOptions::default() },
    );
}
