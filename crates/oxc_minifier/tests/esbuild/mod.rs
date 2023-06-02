//! <https://github.com/evanw/esbuild/blob/main/internal/js_printer/js_printer_test.go#L164>

use crate::{test, test_same};

#[test]
fn number() {
    test("x = 1e-100", "x=1e-100");
    test("x = 1e-5", "x=1e-5");
    test("x = 1e-4", "x=1e-4");
    test("x = 1e-3", "x=.001");
    test("x = 1e-2", "x=.01");
    test("x = 1e-1", "x=.1");
    test("x = 1e1", "x=10");
    test("x = 1e2", "x=100");
    test("x = 1e3", "x=1e3");
    test("x = 1e4", "x=1e4");
    test("x = 1e100", "x=1e100");
    test("x = 1e0", "x=1");

    test("x = 12e-100", "x=12e-100");
    test("x = 12e-6", "x=12e-6");
    test("x = 12e-5", "x=12e-5");
    test("x = 12e-4", "x=.0012");
    test("x = 12e-3", "x=.012");
    test("x = 12e-2", "x=.12");
    test("x = 12e-1", "x=1.2");
    test("x = 12e0", "x=12");
    test("x = 12e1", "x=120");
    test("x = 12e2", "x=1200");
    test("x = 12e3", "x=12e3");
    test("x = 12e4", "x=12e4");
    test("x = 12e100", "x=12e100");

    test("x = 999999999999", "x=999999999999");
    test("x = 1000000000001", "x=0xe8d4a51001");
    test("x = 0x0FFF_FFFF_FFFF_FF80", "x=0xfffffffffffff80");
    test("x = 0x1000_0000_0000_0000", "x=1152921504606847e3");
    test("x = 0xFFFF_FFFF_FFFF_F000", "x=0xfffffffffffff000");
    test("x = 0xFFFF_FFFF_FFFF_F800", "x=1844674407370955e4");
    test("x = 0xFFFF_FFFF_FFFF_FFFF", "x=18446744073709552e3");

    test("x = 0.0001 .y", "x=1e-4.y");
    test("x = 0.001 .y", "x=.001.y");
    test("x = 0.01 .y", "x=.01.y");
    test("x = 0.1 .y", "x=.1.y");
    test("x = 0 .y", "x=0 .y");
    test("x = 10 .y", "x=10 .y");
    test("x = 100 .y", "x=100 .y");
    test("x = 1000 .y", "x=1e3.y");
    test("x = 12345 .y", "x=12345 .y");
    test("x = 0xFFFF_0000_FFFF_0000 .y", "x=0xffff0000ffff0000.y");
}

#[test]
fn array() {
    test("[]", "[]");
    test("[,]", "[,]");
    test("[,,]", "[,,]");
}

#[test]
fn splat() {
    test("[...(a, b)]", "[...(a,b)]");
    test("x(...(a, b))", "x(...(a,b))");
    test("({...(a, b)})", "({...(a,b)})");
}

#[test]
#[ignore]
fn new() {
    test("new (x)", "new x");
    test("new (x())", "new (x())");
    test("new (new x())", "new new x");
    test("new (x + x)", "new (x+x)");
    test("(new x)()", "(new x)()");

    test("new foo().bar", "new foo().bar");
    test("new (foo().bar)", "new (foo()).bar()");
    test("new (foo()).bar", "new (foo()).bar()");
    test("new foo()[bar]", "new foo()[bar]");
    test("new (foo()[bar])", "new (foo())[bar]()");
    test("new (foo())[bar]", "new (foo())[bar]()");

    test("new (import('foo').bar)", "new (import(\"foo\")).bar()");
    test("new (import('foo')).bar", "new (import(\"foo\")).bar()");
    test("new (import('foo')[bar])", "new (import(\"foo\"))[bar]()");
    test("new (import('foo'))[bar]", "new (import(\"foo\"))[bar]()");

    test("new x", "new x");
    test("new x.y", "new x.y");
    test("(new x).y", "new x().y");
    test("new x().y", "new x().y");
    test("new x() + y", "new x+y");
    test("new x() ** 2", "new x**2");

    // Test preservation of Webpack-specific comments
    // test( "new Worker(// webpackFoo: 1\n // webpackBar: 2\n 'path');", "new Worker(\n  // webpackFoo: 1\n  // webpackBar: 2\n  \"path\"\n);\n")
    // test( "new Worker(/* webpackFoo: 1 */ /* webpackBar: 2 */ 'path');", "new Worker(\n  /* webpackFoo: 1 */\n  /* webpackBar: 2 */\n  \"path\"\n);\n")
    // test( "new Worker(\n    /* multi\n     * line\n     * webpackBar: */ 'path');", "new Worker(\n  /* multi\n   * line\n   * webpackBar: */\n  \"path\"\n);\n")
    // test( "new Worker(/* webpackFoo: 1 */ 'path' /* webpackBar:2 */);", "new Worker(\n  /* webpackFoo: 1 */\n  \"path\"\n  /* webpackBar:2 */\n);\n")
    // test( "new Worker(/* webpackFoo: 1 */ 'path' /* webpackBar:2 */ ,);", "new Worker(\n  /* webpackFoo: 1 */\n  \"path\"\n);\n") // Not currently handled
    // test( "new Worker(/* webpackFoo: 1 */ 'path', /* webpackBar:2 */ );", "new Worker(\n  /* webpackFoo: 1 */\n  \"path\"\n  /* webpackBar:2 */\n);\n")
    // test( "new Worker(new URL('path', /* webpackFoo: these can go anywhere */ import.meta.url))",
    // 	"new Worker(new URL(\n  \"path\",\n  /* webpackFoo: these can go anywhere */\n  import.meta.url\n));\n")
}

#[test]
fn call() {
    test("x()()()", "x()()()");
    test("x().y()[z]()", "x().y()[z]()");
    test("(--x)();", "(--x)()");
    test("(x--)();", "(x--)()");

    test("eval(x)", "eval(x)");
    test("eval?.(x)", "eval?.(x)");
    test("(eval)(x)", "eval(x)");
    test("(eval)?.(x)", "eval?.(x)");

    test("eval(x, y)", "eval(x,y)");
    test("eval?.(x, y)", "eval?.(x,y)");
    test("(1, eval)(x)", "(1,eval)(x)");
    test("(1, eval)?.(x)", "(1,eval)?.(x)");
    // test("(1 ? eval : 2)(x)", "(0,eval)(x)");
    // test("(1 ? eval : 2)?.(x)", "eval?.(x)");

    test("eval?.(x)", "eval?.(x)");
    test("eval(x,y)", "eval(x,y)");
    test("eval?.(x,y)", "eval?.(x,y)");
    test("(1, eval)(x)", "(1,eval)(x)");
    test("(1, eval)?.(x)", "(1,eval)?.(x)");
    // test("(1 ? eval : 2)(x)", "(0,eval)(x)");
    // test("(1 ? eval : 2)?.(x)", "eval?.(x)");
}

#[test]
fn member() {
    test("x.y[z]", "x.y[z]");
    test("((x+1).y+1)[z]", "((x+1).y+1)[z]");
}

#[test]
fn comma() {
    test("1, 2, 3", "1,2,3");
    // test("(1, 2), 3", "1,2,3");
    // test("1, (2, 3)", "1,2,3");
    test("a ? (b, c) : (d, e)", "a?(b,c):(d,e)");
    test("let x = (a, b)", "let x=(a,b)");
    test("(x = a), b", "x=a,b");
    test("x = (a, b)", "x=(a,b)");
    test("x((1, 2))", "x((1,2))");
}

#[test]
fn unary() {
    test("+(x--)", "+x--");
    test("-(x++)", "-x++");
}

#[test]
fn nullish() {
    // "??" can't directly contain "||" or "&&"
    test("(a && b) ?? c", "(a&&b)??c");
    test("(a || b) ?? c", "(a||b)??c");
    test("a ?? (b && c)", "a??(b&&c)");
    test("a ?? (b || c)", "a??(b||c)");

    // "||" and "&&" can't directly contain "??"
    test("a && (b ?? c)", "a&&(b??c)");
    test("a || (b ?? c)", "a||(b??c)");
    test("(a ?? b) && c", "(a??b)&&c");
    test("(a ?? b) || c", "(a??b)||c");
}

#[test]
#[ignore]
fn string() {
    test("let x = ''", "let x = \"\"");
    test("let x = '\\b'", "let x = \"\\b\"");
    test("let x = '\\f'", "let x = \"\\f\"");
    test("let x = '\t'", "let x = \"\t\"");
    test("let x = '\\v'", "let x = \"\\v\"");
    test("let x = '\\n'", "let x = \"\\n\"");
    test("let x = '\\''", "let x = \"'\"");
    test("let x = '\\\"'", "let x = '\"'");
    test("let x = '\\'\"'", "let x = `'\"`");
    test("let x = '\\\\'", "let x = \"\\\\\"");
    test("let x = '\x00'", "let x = \"\\0\"");
    test("let x = '\x00!'", "let x = \"\\0!\"");
    test("let x = '\x001'", "let x = \"\\x001\"");
    test("let x = '\\0'", "let x = \"\\0\"");
    test("let x = '\\0!'", "let x = \"\\0!\"");
    test("let x = '\x07'", "let x = \"\\x07\"");
    test("let x = '\x07!'", "let x = \"\\x07!\"");
    test("let x = '\x071'", "let x = \"\\x071\"");
    test("let x = '\\7'", "let x = \"\\x07\"");
    test("let x = '\\7!'", "let x = \"\\x07!\"");
    test("let x = '\\01'", "let x = \"\x01\"");
    test("let x = '\x10'", "let x = \"\x10\"");
    test("let x = '\\x10'", "let x = \"\x10\"");
    test("let x = '\x1B'", "let x = \"\\x1B\"");
    test("let x = '\\x1B'", "let x = \"\\x1B\"");
    test("let x = '\\uABCD'", "let x = \"\\uABCD\"");
    test("let x = '\\uABCD'", "let x = \"\\uABCD\"");
    test("let x = '\\U000123AB'", "let x = \"\\U000123AB\"");
    test("let x = '\\u{123AB}'", "let x = \"\\U000123AB\"");
    test("let x = '\\uD808\\uDFAB'", "let x = \"\\U000123AB\"");
    test("let x = '\\uD808'", "let x = \"\\uD808\"");
    test("let x = '\\uD808X'", "let x = \"\\uD808X\"");
    test("let x = '\\uDFAB'", "let x = \"\\uDFAB\"");
    test("let x = '\\uDFABX'", "let x = \"\\uDFABX\"");

    test("let x = '\\x80'", "let x = \"\\U00000080\"");
    test("let x = '\\xFF'", "let x = \"\\U000000FF\"");
    test(
        "let x = '\\xF0\\x9F\\x8D\\x95'",
        "let x = \"\\U000000F0\\U0000009F\\U0000008D\\U00000095\"",
    );
    test("let x = '\\uD801\\uDC02\\uDC03\\uD804'", "let x = \"\\U00010402\\uDC03\\uD804\"");
}

#[test]
#[ignore]
fn template() {
    test("let x = `\\0`", "let x = `\\0`");
    test("let x = `\\x01`", "let x = `\x01`");
    test("let x = `\\0${0}`", "let x = `\\0${0}`");
    test("let x = `\\x01${0}`", "let x = `\x01${0}`");
    test("let x = `${0}\\0`", "let x = `${0}\\0`");
    test("let x = `${0}\\x01`", "let x = `${0}\x01`");
    test("let x = `${0}\\0${1}`", "let x = `${0}\\0${1}`");
    test("let x = `${0}\\x01${1}`", "let x = `${0}\x01${1}`");

    test("let x = String.raw`\\1`", "let x = String.raw`\\1`");
    test("let x = String.raw`\\x01`", "let x = String.raw`\\x01`");
    test("let x = String.raw`\\1${0}`", "let x = String.raw`\\1${0}`");
    test("let x = String.raw`\\x01${0}`", "let x = String.raw`\\x01${0}`");
    test("let x = String.raw`${0}\\1`", "let x = String.raw`${0}\\1`");
    test("let x = String.raw`${0}\\x01`", "let x = String.raw`${0}\\x01`");
    test("let x = String.raw`${0}\\1${1}`", "let x = String.raw`${0}\\1${1}`");
    test("let x = String.raw`${0}\\x01${1}`", "let x = String.raw`${0}\\x01${1}`");

    test("let x = `${y}`", "let x = `${y}`");
    test("let x = `$(y)`", "let x = `$(y)`");
    test("let x = `{y}$`", "let x = `{y}$`");
    test("let x = `$}y{`", "let x = `$}y{`");
    test("let x = `\\${y}`", "let x = `\\${y}`");
    test("let x = `$\\{y}`", "let x = `\\${y}`");

    test("await tag`x`", "await tag`x`");
    test("await (tag`x`)", "await tag`x`");
    test("(await tag)`x`", "(await tag)`x`");

    test("await tag`${x}`", "await tag`${x}`");
    test("await (tag`${x}`)", "await tag`${x}`");
    test("(await tag)`${x}`", "(await tag)`${x}`");

    test("new tag`x`", "new tag`x`()");
    test("new (tag`x`)", "new tag`x`()");
    test("new tag()`x`", "new tag()`x`");
    test("(new tag)`x`", "new tag()`x`");
    test("new tag`x`", "new tag`x`;");
    test("new (tag`x`)", "new tag`x`;");
    test("new tag()`x`", "new tag()`x`;");
    test("(new tag)`x`", "new tag()`x`;");

    test("new tag`${x}`", "new tag`${x}`()");
    test("new (tag`${x}`)", "new tag`${x}`()");
    test("new tag()`${x}`", "new tag()`${x}`");
    test("(new tag)`${x}`", "new tag()`${x}`");
    test("new tag`${x}`", "new tag`${x}`;");
    test("new (tag`${x}`)", "new tag`${x}`;");
    test("new tag()`${x}`", "new tag()`${x}`;");
    test("(new tag)`${x}`", "new tag()`${x}`;");
}

#[test]
fn object() {
    test("let x = {'(':')'}", "let x={'(':')'}");
    test("({})", "({})");
    test("({}.x)", "({}).x");
    test("({} = {})", "({}={})");
    test("(x, {} = {})", "x,{}={}");
    test("let x = () => ({})", "let x=()=>({})");
    test("let x = () => ({}.x)", "let x=()=>({}).x");
    test("let x = () => ({} = {})", "let x=()=>({}={})");
    test("let x = () => (x, {} = {})", "let x=()=>(x,{}={})");
}

#[test]
#[ignore]
fn r#for() {
    // Make sure "in" expressions are forbidden in the right places
    test("for ((a in b);;);", "for ((a in b); ; )");
    test("for (a ? b : (c in d);;);", "for (a ? b : (c in d); ; )");
    test("for ((a ? b : c in d).foo;;);", "for ((a ? b : c in d).foo; ; )");
    test("for (var x = (a in b);;);", "for (var x = (a in b); ; )");
    test("for (x = (a in b);;);", "for (x = (a in b); ; )");
    test("for (x == (a in b);;);", "for (x == (a in b); ; )");
    test("for (1 * (x == a in b);;);", "for (1 * (x == a in b); ; )");
    test("for (a ? b : x = (c in d);;);", "for (a ? b : x = (c in d); ; )");
    test("for (var x = y = (a in b);;);", "for (var x = y = (a in b); ; )");
    test("for ([a in b];;);", "for ([a in b]; ; )");
    test("for (x(a in b);;);", "for (x(a in b); ; )");
    test("for (x[a in b];;);", "for (x[a in b]; ; )");
    test("for (x?.[a in b];;);", "for (x?.[a in b]; ; )");
    test("for ((x => a in b);;);", "for ((x) => (a in b); ; )");

    // Make sure for-of loops with commas are wrapped in parentheses
    test("for (let a in b, c);", "for (let a in b, c)");
    test("for (let a of (b, c));", "for (let a of (b, c))");
}

#[test]
fn function() {
    test("function foo(a = (b, c), ...d) {}", "function foo(a=(b,c),...d){}");
    test(
        "function foo({[1 + 2]: a = 3} = {[1 + 2]: 3}) {}",
        "function foo({[1+2]:a=3}={[1+2]:3}){}",
    );
    test(
        "function foo([a = (1, 2), ...[b, ...c]] = [1, [2, 3]]) {}",
        "function foo([a=(1,2),...[b,...c]]=[1,[2,3]]){}",
    );
    test("function foo([] = []) {}", "function foo([]=[]){}");
    test("function foo([,] = [,]) {}", "function foo([,]=[,]){}");
    test("function foo([,,] = [,,]) {}", "function foo([,,]=[,,]){}");
}

#[test]
#[ignore]
fn comments_and_parentheses() {
    test("(/* foo */ { x() { foo() } }.x());", "/* foo */\n({ x() {\n  foo();\n} }).x();\n");
    test(
        "(/* foo */ function f() { foo(f) }());",
        "/* foo */\n(function f() {\n  foo(f);\n})();\n",
    );
    test(
        "(/* foo */ class x { static y() { foo(x) } }.y());",
        "/* foo */\n(class x {\n  static y() {\n    foo(x);\n  }\n}).y();\n",
    );
    test("(/* @__PURE__ */ (() => foo())());", "/* @__PURE__ */ (() => foo())();\n");
    test(
        "export default (/* foo */ function f() {});",
        "export default (\n  /* foo */\n  function f() {\n  }\n);\n",
    );
    test(
        "export default (/* foo */ class x {});",
        "export default (\n  /* foo */\n  class x {\n  }\n);\n",
    );
    test("x = () => (/* foo */ {});", "x = () => (\n  /* foo */\n  {}\n);\n");
    test("for ((/* foo */ let).x of y) ;", "for (\n  /* foo */\n  (let).x of y\n)\n  ;\n");
    test("for (/* foo */ (let).x of y) ;", "for (\n  /* foo */\n  (let).x of y\n)\n  ;\n");
    test(
        "function *x() { yield (/* foo */ y) }",
        "function* x() {\n  yield (\n    /* foo */\n    y\n  );\n}\n",
    );
}

#[test]
#[ignore]
fn pure_comment() {
    test("(function() {})", "(function() {\n});\n");
    test("(function() {})()", "(function() {\n})();\n");
    test("/*@__PURE__*/(function() {})()", "/* @__PURE__ */ (function() {\n})();\n");

    test("new (function() {})", "new function() {\n}();\n");
    test("new (function() {})()", "new function() {\n}();\n");
    test("/*@__PURE__*/new (function() {})()", "/* @__PURE__ */ new function() {\n}();\n");

    test("export default (function() {})", "export default (function() {\n});\n");
    test("export default (function() {})()", "export default (function() {\n})();\n");
    test(
        "export default /*@__PURE__*/(function() {})()",
        "export default /* @__PURE__ */ (function() {\n})();\n",
    );
}

#[test]
fn generator() {
    test("function* foo() {}", "function*foo(){}");
    test("(function* () {})", "(function*(){})");
    test("(function* foo() {})", "(function*foo(){})");

    test("class Foo { *foo() {} }", "class Foo{*foo(){}}");
    test("class Foo { static *foo() {} }", "class Foo{static *foo(){}}");
    test("class Foo { *[foo]() {} }", "class Foo{*[foo](){}}");
    test("class Foo { static *[foo]() {} }", "class Foo{static *[foo](){}}");

    test("(class { *foo() {} })", "(class{*foo(){}})");
    test("(class { static *foo() {} })", "(class{static *foo(){}})");
    test("(class { *[foo]() {} })", "(class{*[foo](){}})");
    test("(class { static *[foo]() {} })", "(class{static *[foo](){}})");
}

#[test]
fn arrow() {
    test("() => {}", "()=>{}");
    test("x => (x, 0)", "x=>(x,0)");
    test("x => {y}", "x=>{y}");
    test("(a = (b, c), ...d) => {}", "(a=(b,c),...d)=>{}");
    test("({[1 + 2]: a = 3} = {[1 + 2]: 3}) => {}", "({[1+2]:a=3}={[1+2]:3})=>{}");
    test(
        "([a = (1, 2), ...[b, ...c]] = [1, [2, 3]]) => {}",
        "([a=(1,2),...[b,...c]]=[1,[2,3]])=>{}",
    );
    test("([] = []) => {}", "([]=[])=>{}");
    test("([,] = [,]) => {}", "([,]=[,])=>{}");
    test("([,,] = [,,]) => {}", "([,,]=[,,])=>{}");
    test("a = () => {}", "a=()=>{}");
    test("a || (() => {})", "a||(()=>{})");
    // test("({a = b, c = d}) => {}", "({a=b,c=d})=>{}");
    // test("([{a = b, c = d} = {}] = []) => {}", "([{a=b,c=d}={}]=[])=>{}");
    test("({a: [b = c] = []} = {}) => {}", "({a:[b=c]=[]}={})=>{}");

    // These are not arrow functions but initially look like one
    test("(a = b, c)", "a=b,c");
    test("([...a = b])", "[...a=b]");
    test("([...a, ...b])", "[...a,...b]");
    test("({a: b, c() {}})", "({a:b,c(){}})");
    test("({a: b, get c() {}})", "({a:b,get c(){}})");
    test("({a: b, set c(x) {}})", "({a:b,set c(x){}})");
}

#[test]
fn class() {
    test("class Foo extends (a, b) {}", "class Foo extends (a,b){}");
    test("class Foo { get foo() {} }", "class Foo{get foo(){}}");
    test("class Foo { set foo(x) {} }", "class Foo{set foo(x){}}");
    test("class Foo { static foo() {} }", "class Foo{static foo(){}}");
    test("class Foo { static get foo() {} }", "class Foo{static get foo(){}}");
    test("class Foo { static set foo(x) {} }", "class Foo{static set foo(x){}}");
}

#[test]
#[ignore]
fn private_identifiers() {
    test(
        "class Foo { #foo; foo() { return #foo in this } }",
        "class Foo{#foo;foo(){return#foo in this}}",
    );
}

#[test]
#[ignore]
fn import() {
    test("import('path');", "import(\"path\");\n"); // The semicolon must not be a separate statement

    // Test preservation of Webpack-specific comments
    test(
        "import(\n  // webpackFoo: 1\n  // webpackBar: 2\n  \"path\"\n);\n",
        "import(// webpackFoo: 1\n // webpackBar: 2\n 'path');",
    );
    test(
        "import(// webpackFoo: 1\n // webpackBar: 2\n 'path', {type: 'module'});",
        "import(\n  // webpackFoo: 1\n  // webpackBar: 2\n  \"path\",\n  { type: \"module\" }\n);\n",
    );
    test(
        "import(/* webpackFoo: 1 */ /* webpackBar: 2 */ 'path');",
        "import(\n  /* webpackFoo: 1 */\n  /* webpackBar: 2 */\n  \"path\"\n);\n",
    );
    test(
        "import(/* webpackFoo: 1 */ /* webpackBar: 2 */ 'path', {type: 'module'});",
        "import(\n  /* webpackFoo: 1 */\n  /* webpackBar: 2 */\n  \"path\",\n  { type: \"module\" }\n);\n",
    );
    test(
        "import(\n    /* multi\n     * line\n     * webpackBar: */ 'path');",
        "import(\n  /* multi\n   * line\n   * webpackBar: */\n  \"path\"\n);\n",
    );
    test(
        "import(/* webpackFoo: 1 */ 'path' /* webpackBar:2 */);",
        "import(\n  /* webpackFoo: 1 */\n  \"path\"\n  /* webpackBar:2 */\n);\n",
    );
    test(
        "import(/* webpackFoo: 1 */ 'path' /* webpackBar:2 */ ,);",
        "import(\n  /* webpackFoo: 1 */\n  \"path\"\n);\n",
    ); // Not currently handled
    test(
        "import(/* webpackFoo: 1 */ 'path', /* webpackBar:2 */ );",
        "import(\n  /* webpackFoo: 1 */\n  \"path\"\n  /* webpackBar:2 */\n);\n",
    );
    test(
        "import(/* webpackFoo: 1 */ 'path', { type: 'module' } /* webpackBar:2 */ );",
        "import(\n  /* webpackFoo: 1 */\n  \"path\",\n  { type: \"module\" }\n  /* webpackBar:2 */\n);\n",
    );
    test(
        "import(new URL('path', /* webpackFoo: these can go anywhere */ import.meta.url))",
        "import(new URL(\n  \"path\",\n  /* webpackFoo: these can go anywhere */\n  import.meta.url\n));\n",
    );
}

#[test]
fn export_default() {
    test("export default function() {}", "export default function(){}");
    test("export default function foo() {}", "export default function foo(){}");
    test("export default async function() {}", "export default async function(){}");
    test("export default async function foo() {}", "export default async function foo(){}");
    test("export default class {}", "export default class{}");
    test("export default class foo {}", "export default class foo{}");

    test("export default (function() {})", "export default (function(){})");
    test("export default (function foo() {})", "export default (function foo(){})");
    test("export default (async function() {})", "export default (async function(){})");
    test("export default (async function foo() {})", "export default (async function foo(){})");
    test("export default (class {})", "export default (class{})");
    test("export default (class foo {})", "export default (class foo{})");

    // test(
    // "export default (function() {}.toString())",
    // "export default (function() {}).toString()",
    // );
    // test(
    // "export default (function foo() {}.toString())",
    // "export default (function foo() {}).toString()",
    // );
    // test(
    // "export default (async function() {}.toString())",
    // "export default (async function() {}).toString()",
    // );
    // test(
    // "export default (async function foo() {}.toString())",
    // "export default (async function foo() {}).toString()",
    // );
    // test("export default (class {}.toString())", "export default (class {}).toString()");
    // test("export default (class foo {}.toString())", "export default (class foo {}).toString()");

    test("export default function() {}", "export default function(){}");
    test("export default function foo() {}", "export default function foo(){}");
    test("export default async function() {}", "export default async function(){}");
    test("export default async function foo() {}", "export default async function foo(){}");
    test("export default class {}", "export default class{}");
    test("export default class foo {}", "export default class foo{}");
}

#[test]
fn whitespace() {
    test("- -x", "- -x");
    test("+ -x", "+-x");
    test("- +x", "-+x");
    test("+ +x", "+ +x");
    test("- --x", "- --x");
    test("+ --x", "+--x");
    test("- ++x", "-++x");
    test("+ ++x", "+ ++x");

    test("- -x", "- -x");
    test("+ -x", "+-x");
    test("- +x", "-+x");
    test("+ +x", "+ +x");
    test("- --x", "- --x");
    test("+ --x", "+--x");
    test("- ++x", "-++x");
    test("+ ++x", "+ ++x");

    test("x - --y", "x- --y");
    test("x + --y", "x+--y");
    test("x - ++y", "x-++y");
    test("x + ++y", "x+ ++y");

    test("x-- > y", "x-- >y");
    test("x < !--y", "x<! --y");
    test("x > !--y", "x>!--y");
    test("!--y", "!--y");

    test("1 + -0", "1+-0");
    test("1 - -0", "1- -0");
    // test("1 + -Infinity", "1+-1/0");
    // test("1 - -Infinity", "1- -1/0");

    test("/x/ / /y/", "/x// /y/");
    test("/x/ + Foo", "/x/+Foo");
    test("/x/ instanceof Foo", "/x/ instanceof Foo");
    test("[x] instanceof Foo", "[x]instanceof Foo");

    test("throw x", "throw x");
    test("throw typeof x", "throw typeof x");
    test("throw delete x", "throw delete x");
    test("throw function(){}", "throw function(){}");

    // test("x in function(){}", "x in function(){}");
    // test("x instanceof function(){}", "x instanceof function(){}");
    // test("π in function(){}", "π in function(){}");
    // test("π instanceof function(){}", "π instanceof function(){}");

    test("()=>({})", "()=>({})");
    test("()=>({}[1])", "()=>({})[1]");
    test("()=>({}+0)", "()=>({})+0");
    test("()=>function(){}", "()=>function(){}");

    test("(function(){})", "(function(){})");
    test("(class{})", "(class{})");
    test("({})", "({})");
}

#[test]
#[ignore]
fn mangle() {
    test("let x = '\\n'", "let x = `\n`;\n");
    test("let x = `\n`", "let x = `\n`;\n");
    test("let x = '\\n${}'", "let x = \"\\n${}\";\n");
    test("let x = `\n\\${}`", "let x = \"\\n${}\";\n");
    test("let x = `\n\\${}${y}\\${}`", "let x = `\n\\${}${y}\\${}`;\n");
}

#[test]
fn minify() {
    // test("0.1", ".1");
    test("1.2", "1.2");

    test("() => {}", "()=>{}");
    test("(a) => {}", "a=>{}");
    test("(...a) => {}", "(...a)=>{}");
    test("(a = 0) => {}", "(a=0)=>{}");
    test("(a, b) => {}", "(a,b)=>{}");

    test("true ** 2", "(!0)**2");
    test("false ** 2", "(!1)**2");

    // test("import a from 'path'", "import a from'path'");
    // test("import * as ns from 'path'", "import*as ns from'path'");
    // test("import {a, b as c} from 'path'", "import{a,b as c}from'path'");
    // test("import {a, ' ' as c} from 'path'", "import{a,' 'as c}from'path'");

    // test("export * as ns from 'path'", "export*as ns from'path'");
    // test("export * as ' ' from 'path'", "export*as' 'from'path'");
    // test("export {a, b as c} from 'path'", "export{a,b as c}from'path'");
    // test("export {' ', '-' as ';'} from 'path'", "export{' ','-'as';'}from'path'");
    // test("let a, b; export {a, b as c}", "let a,b;export{a,b as c}");
    // test("let a, b; export {a, b as ' '}", "let a,b;export{a,b as' '}");

    // Print some strings using template literals when minifying
    // test("x = '\\n'", "x = \"\\n\";\n");
    // test("x = '\\n'", "x = `\n`;\n");
    // test("x = {'\\n': 0}", "x = { \"\\n\": 0 };\n");
    // test("(class{'\\n' = 0})", "(class {\n  \"\\n\" = 0;\n});\n");
    // test("class Foo{'\\n' = 0}", "class Foo {\n  \"\\n\" = 0;\n}\n");

    // Special identifiers must not be minified
    test("exports", "exports");
    test("require", "require");
    test("module", "module");

    // Comment statements must not affect their surroundings when minified
    // test("//!single\nthrow 1 + 2", "//!single\nthrow 1+2;");
    // test("/*!multi-\nline*/\nthrow 1 + 2", "/*!multi-\nline*/throw 1+2;");
}

#[test]
#[ignore]
fn infinity() {
    test("x = Infinity", "x=1/0");
    // test("x = -Infinity", "x=-1/0");
    test("x = (Infinity).toString", "x=(1/0).toString");
    // test("x = (-Infinity).toString", "x=(-1/0).toString");
    test("x = Infinity ** 2", "x=(1/0)**2");
    // test("x = (-Infinity) ** 2", "x=(-1/0)**2");
    test("x = Infinity * y", "x=1/0*y");
    test("x = Infinity / y", "x=1/0/y");
    test("x = y * Infinity", "x=y*1/0");
    test("x = y / Infinity", "x=y/1/0");
    test("throw Infinity", "throw 1/0");
}

#[test]
#[ignore]
fn es5() {}

#[test]
#[ignore]
fn ascii_only() {}

#[test]
#[ignore]
fn jsx() {}

#[test]
#[ignore]
fn jsx_single_line() {}

#[test]
fn avoid_slash_script() {
    // Positive cases
    test("x = 1 < /script/.exec(y).length", "x=1< /script/.exec(y).length");
    test("x = 1 < /SCRIPT/.exec(y).length", "x=1< /SCRIPT/.exec(y).length");
    test("x = 1 < /ScRiPt/.exec(y).length", "x=1< /ScRiPt/.exec(y).length");
    test("x = 1 << /script/.exec(y).length", "x=1<< /script/.exec(y).length");

    // Negative cases
    test("x = 1 < / script/.exec(y).length", "x=1</ script/.exec(y).length");
    test("x = 1 << / script/.exec(y).length", "x=1<</ script/.exec(y).length");
}

#[test]
fn binary_operator_visitor() {
    // Make sure the inner "/*b*/" comment doesn't disappear due to weird binary visitor stuff
    // test(
    // "x = (0, /*a*/ (0, /*b*/ (0, /*c*/ 1 == 2) + 3) * 4)",
    // "x = /*a*/\n/*b*/\n(/*c*/\n!1 + 3) * 4;\n",
    // );

    // Make sure deeply-nested ASTs don't cause a stack overflow
    let x = format!("x=f(){}", "||f()".repeat(1000)); // TODO: change this to 10_000
    test_same(&x);
}
