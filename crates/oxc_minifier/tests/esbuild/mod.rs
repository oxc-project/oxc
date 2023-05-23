//! <https://github.com/evanw/esbuild/blob/main/internal/js_printer/js_printer_test.go#L164>

use crate::{expect, expect_same};

#[test]
#[ignore]
fn number() {
    expect("x = 1e-100", "x=1e-100");
    expect("x = 1e-5", "x=1e-5");
    expect("x = 1e-4", "x=1e-4");
    expect("x = 1e-3", "x=.001");
    expect("x = 1e-2", "x=.01");
    expect("x = 1e-1", "x=.1");
    expect("x = 1e0", "x=1");
    expect("x = 1e1", "x=10");
    expect("x = 1e2", "x=100");
    expect("x = 1e3", "x=1e3");
    expect("x = 1e4", "x=1e4");
    expect("x = 1e100", "x=1e100");

    expect("x = 12e-100", "x=12e-100;");
    expect("x = 12e-6", "x=12e-6;");
    expect("x = 12e-5", "x=12e-5;");
    expect("x = 12e-4", "x=.0012;");
    expect("x = 12e-3", "x=.012;");
    expect("x = 12e-2", "x=.12;");
    expect("x = 12e-1", "x=1.2;");
    expect("x = 12e0", "x=12;");
    expect("x = 12e1", "x=120;");
    expect("x = 12e2", "x=1200;");
    expect("x = 12e3", "x=12e3;");
    expect("x = 12e4", "x=12e4;");
    expect("x = 12e100", "x=12e100;");

    expect("x = 999999999999", "x=999999999999;");
    expect("x = 1000000000001", "x=0xe8d4a51001;");
    expect("x = 0x0FFF_FFFF_FFFF_FF80", "x=0xfffffffffffff80;");
    expect("x = 0x1000_0000_0000_0000", "x=1152921504606847e3;");
    expect("x = 0xFFFF_FFFF_FFFF_F000", "x=0xfffffffffffff000;");
    expect("x = 0xFFFF_FFFF_FFFF_F800", "x=1844674407370955e4;");
    expect("x = 0xFFFF_FFFF_FFFF_FFFF", "x=18446744073709552e3;");

    expect("x = 0.0001 .y", "x=1e-4.y;");
    expect("x = 0.001 .y", "x=.001.y;");
    expect("x = 0.01 .y", "x=.01.y;");
    expect("x = 0.1 .y", "x=.1.y;");
    expect("x = 0 .y", "x=0 .y;");
    expect("x = 10 .y", "x=10 .y;");
    expect("x = 100 .y", "x=100 .y;");
    expect("x = 1000 .y", "x=1e3.y;");
    expect("x = 12345 .y", "x=12345 .y;");
    expect("x = 0xFFFF_0000_FFFF_0000 .y", "x=0xffff0000ffff0000.y;");
}

#[test]
fn array() {
    expect("[]", "[]");
    expect("[,]", "[,]");
    expect("[,,]", "[,,]");
}

#[test]
fn splat() {
    expect("[...(a, b)]", "[...(a,b)]");
    expect("x(...(a, b))", "x(...(a,b))");
    expect("({...(a, b)})", "({...(a,b)})");
}

#[test]
#[ignore]
fn new() {
    expect("new x", "new x()");
    expect("new x()", "new x()");
    expect("new (x)", "new x()");
    expect("new (x())", "new (x())()");
    expect("new (new x())", "new new x()()");
    expect("new (x + x)", "new (x + x)()");
    expect("(new x)()", "new x()()");

    expect("new foo().bar", "new foo().bar");
    expect("new (foo().bar)", "new (foo()).bar()");
    expect("new (foo()).bar", "new (foo()).bar()");
    expect("new foo()[bar]", "new foo()[bar]");
    expect("new (foo()[bar])", "new (foo())[bar]()");
    expect("new (foo())[bar]", "new (foo())[bar]()");

    expect("new (import('foo').bar)", "new (import(\"foo\")).bar()");
    expect("new (import('foo')).bar", "new (import(\"foo\")).bar()");
    expect("new (import('foo')[bar])", "new (import(\"foo\"))[bar]()");
    expect("new (import('foo'))[bar]", "new (import(\"foo\"))[bar]()");

    expect("new x", "new x;");
    expect("new x.y", "new x.y;");
    expect("(new x).y", "new x().y;");
    expect("new x().y", "new x().y;");
    expect("new x() + y", "new x+y;");
    expect("new x() ** 2", "new x**2;");

    // Test preservation of Webpack-specific comments
    // expect( "new Worker(// webpackFoo: 1\n // webpackBar: 2\n 'path');", "new Worker(\n  // webpackFoo: 1\n  // webpackBar: 2\n  \"path\"\n);\n")
    // expect( "new Worker(/* webpackFoo: 1 */ /* webpackBar: 2 */ 'path');", "new Worker(\n  /* webpackFoo: 1 */\n  /* webpackBar: 2 */\n  \"path\"\n);\n")
    // expect( "new Worker(\n    /* multi\n     * line\n     * webpackBar: */ 'path');", "new Worker(\n  /* multi\n   * line\n   * webpackBar: */\n  \"path\"\n);\n")
    // expect( "new Worker(/* webpackFoo: 1 */ 'path' /* webpackBar:2 */);", "new Worker(\n  /* webpackFoo: 1 */\n  \"path\"\n  /* webpackBar:2 */\n);\n")
    // expect( "new Worker(/* webpackFoo: 1 */ 'path' /* webpackBar:2 */ ,);", "new Worker(\n  /* webpackFoo: 1 */\n  \"path\"\n);\n") // Not currently handled
    // expect( "new Worker(/* webpackFoo: 1 */ 'path', /* webpackBar:2 */ );", "new Worker(\n  /* webpackFoo: 1 */\n  \"path\"\n  /* webpackBar:2 */\n);\n")
    // expect( "new Worker(new URL('path', /* webpackFoo: these can go anywhere */ import.meta.url))",
    // 	"new Worker(new URL(\n  \"path\",\n  /* webpackFoo: these can go anywhere */\n  import.meta.url\n));\n")
}

#[test]
fn call() {
    expect("x()()()", "x()()()");
    expect("x().y()[z]()", "x().y()[z]()");
    expect("(--x)();", "(--x)()");
    expect("(x--)();", "(x--)()");

    expect("eval(x)", "eval(x)");
    expect("eval?.(x)", "eval?.(x)");
    // expect("(eval)(x)", "eval(x)");
    // expect("(eval)?.(x)", "eval?.(x)");

    expect("eval(x, y)", "eval(x,y)");
    expect("eval?.(x, y)", "eval?.(x,y)");
    expect("(1, eval)(x)", "(1,eval)(x)");
    expect("(1, eval)?.(x)", "(1,eval)?.(x)");
    // expect("(1 ? eval : 2)(x)", "(0,eval)(x)");
    // expect("(1 ? eval : 2)?.(x)", "eval?.(x)");

    expect("eval?.(x)", "eval?.(x)");
    expect("eval(x,y)", "eval(x,y)");
    expect("eval?.(x,y)", "eval?.(x,y)");
    expect("(1, eval)(x)", "(1,eval)(x)");
    expect("(1, eval)?.(x)", "(1,eval)?.(x)");
    // expect("(1 ? eval : 2)(x)", "(0,eval)(x)");
    // expect("(1 ? eval : 2)?.(x)", "eval?.(x)");
}

#[test]
#[ignore]
fn member() {
    expect("x.y[z]", "x.y[z]");
    expect("((x+1).y+1)[z]", "((x + 1).y + 1)[z]");
}

#[test]
fn comma() {
    expect("1, 2, 3", "1,2,3");
    // expect("(1, 2), 3", "1,2,3");
    // expect("1, (2, 3)", "1,2,3");
    expect("a ? (b, c) : (d, e)", "a?(b,c):(d,e)");
    expect("let x = (a, b)", "let x=(a,b)");
    // expect("(x = a), b", "x=a,b");
    expect("x = (a, b)", "x=(a,b)");
    expect("x((1, 2))", "x((1,2))");
}

#[test]
#[ignore]
fn unary() {
    expect("+(x--)", "+x--");
    expect("-(x++)", "-x++");
}

#[test]
#[ignore]
fn nullish() {
    // "??" can't directly contain "||" or "&&"
    expect("(a && b) ?? c", "(a && b) ?? c");
    expect("(a || b) ?? c", "(a || b) ?? c");
    expect("a ?? (b && c)", "a ?? (b && c)");
    expect("a ?? (b || c)", "a ?? (b || c)");

    // "||" and "&&" can't directly contain "??"
    expect("a && (b ?? c)", "a && (b ?? c)");
    expect("a || (b ?? c)", "a || (b ?? c)");
    expect("(a ?? b) && c", "(a ?? b) && c");
    expect("(a ?? b) || c", "(a ?? b) || c");
}

#[test]
#[ignore]
fn string() {
    expect("let x = ''", "let x = \"\"");
    expect("let x = '\\b'", "let x = \"\\b\"");
    expect("let x = '\\f'", "let x = \"\\f\"");
    expect("let x = '\t'", "let x = \"\t\"");
    expect("let x = '\\v'", "let x = \"\\v\"");
    expect("let x = '\\n'", "let x = \"\\n\"");
    expect("let x = '\\''", "let x = \"'\"");
    expect("let x = '\\\"'", "let x = '\"'");
    expect("let x = '\\'\"'", "let x = `'\"`");
    expect("let x = '\\\\'", "let x = \"\\\\\"");
    expect("let x = '\x00'", "let x = \"\\0\"");
    expect("let x = '\x00!'", "let x = \"\\0!\"");
    expect("let x = '\x001'", "let x = \"\\x001\"");
    expect("let x = '\\0'", "let x = \"\\0\"");
    expect("let x = '\\0!'", "let x = \"\\0!\"");
    expect("let x = '\x07'", "let x = \"\\x07\"");
    expect("let x = '\x07!'", "let x = \"\\x07!\"");
    expect("let x = '\x071'", "let x = \"\\x071\"");
    expect("let x = '\\7'", "let x = \"\\x07\"");
    expect("let x = '\\7!'", "let x = \"\\x07!\"");
    expect("let x = '\\01'", "let x = \"\x01\"");
    expect("let x = '\x10'", "let x = \"\x10\"");
    expect("let x = '\\x10'", "let x = \"\x10\"");
    expect("let x = '\x1B'", "let x = \"\\x1B\"");
    expect("let x = '\\x1B'", "let x = \"\\x1B\"");
    expect("let x = '\\uABCD'", "let x = \"\\uABCD\"");
    expect("let x = '\\uABCD'", "let x = \"\\uABCD\"");
    expect("let x = '\\U000123AB'", "let x = \"\\U000123AB\"");
    expect("let x = '\\u{123AB}'", "let x = \"\\U000123AB\"");
    expect("let x = '\\uD808\\uDFAB'", "let x = \"\\U000123AB\"");
    expect("let x = '\\uD808'", "let x = \"\\uD808\"");
    expect("let x = '\\uD808X'", "let x = \"\\uD808X\"");
    expect("let x = '\\uDFAB'", "let x = \"\\uDFAB\"");
    expect("let x = '\\uDFABX'", "let x = \"\\uDFABX\"");

    expect("let x = '\\x80'", "let x = \"\\U00000080\"");
    expect("let x = '\\xFF'", "let x = \"\\U000000FF\"");
    expect(
        "let x = '\\xF0\\x9F\\x8D\\x95'",
        "let x = \"\\U000000F0\\U0000009F\\U0000008D\\U00000095\"",
    );
    expect("let x = '\\uD801\\uDC02\\uDC03\\uD804'", "let x = \"\\U00010402\\uDC03\\uD804\"");
}

#[test]
#[ignore]
fn template() {
    expect("let x = `\\0`", "let x = `\\0`");
    expect("let x = `\\x01`", "let x = `\x01`");
    expect("let x = `\\0${0}`", "let x = `\\0${0}`");
    expect("let x = `\\x01${0}`", "let x = `\x01${0}`");
    expect("let x = `${0}\\0`", "let x = `${0}\\0`");
    expect("let x = `${0}\\x01`", "let x = `${0}\x01`");
    expect("let x = `${0}\\0${1}`", "let x = `${0}\\0${1}`");
    expect("let x = `${0}\\x01${1}`", "let x = `${0}\x01${1}`");

    expect("let x = String.raw`\\1`", "let x = String.raw`\\1`");
    expect("let x = String.raw`\\x01`", "let x = String.raw`\\x01`");
    expect("let x = String.raw`\\1${0}`", "let x = String.raw`\\1${0}`");
    expect("let x = String.raw`\\x01${0}`", "let x = String.raw`\\x01${0}`");
    expect("let x = String.raw`${0}\\1`", "let x = String.raw`${0}\\1`");
    expect("let x = String.raw`${0}\\x01`", "let x = String.raw`${0}\\x01`");
    expect("let x = String.raw`${0}\\1${1}`", "let x = String.raw`${0}\\1${1}`");
    expect("let x = String.raw`${0}\\x01${1}`", "let x = String.raw`${0}\\x01${1}`");

    expect("let x = `${y}`", "let x = `${y}`");
    expect("let x = `$(y)`", "let x = `$(y)`");
    expect("let x = `{y}$`", "let x = `{y}$`");
    expect("let x = `$}y{`", "let x = `$}y{`");
    expect("let x = `\\${y}`", "let x = `\\${y}`");
    expect("let x = `$\\{y}`", "let x = `\\${y}`");

    expect("await tag`x`", "await tag`x`");
    expect("await (tag`x`)", "await tag`x`");
    expect("(await tag)`x`", "(await tag)`x`");

    expect("await tag`${x}`", "await tag`${x}`");
    expect("await (tag`${x}`)", "await tag`${x}`");
    expect("(await tag)`${x}`", "(await tag)`${x}`");

    expect("new tag`x`", "new tag`x`()");
    expect("new (tag`x`)", "new tag`x`()");
    expect("new tag()`x`", "new tag()`x`");
    expect("(new tag)`x`", "new tag()`x`");
    expect("new tag`x`", "new tag`x`;");
    expect("new (tag`x`)", "new tag`x`;");
    expect("new tag()`x`", "new tag()`x`;");
    expect("(new tag)`x`", "new tag()`x`;");

    expect("new tag`${x}`", "new tag`${x}`()");
    expect("new (tag`${x}`)", "new tag`${x}`()");
    expect("new tag()`${x}`", "new tag()`${x}`");
    expect("(new tag)`${x}`", "new tag()`${x}`");
    expect("new tag`${x}`", "new tag`${x}`;");
    expect("new (tag`${x}`)", "new tag`${x}`;");
    expect("new tag()`${x}`", "new tag()`${x}`;");
    expect("(new tag)`${x}`", "new tag()`${x}`;");
}

#[test]
fn object() {
    expect("let x = {'(':')'}", "let x={'(':')'}");
    expect("({})", "({})");
    // expect("({}.x)", "({}).x");
    expect("({} = {})", "({}={})");
    // expect("(x, {} = {})", "x,{}={}");
    expect("let x = () => ({})", "let x=()=>({})");
    // expect("let x = () => ({}.x)", "let x=()=>({}).x");
    expect("let x = () => ({} = {})", "let x=()=>({}={})");
    expect("let x = () => (x, {} = {})", "let x=()=>(x,{}={})");
}

#[test]
#[ignore]
fn r#for() {
    // Make sure "in" expressions are forbidden in the right places
    expect("for ((a in b);;);", "for ((a in b); ; )");
    expect("for (a ? b : (c in d);;);", "for (a ? b : (c in d); ; )");
    expect("for ((a ? b : c in d).foo;;);", "for ((a ? b : c in d).foo; ; )");
    expect("for (var x = (a in b);;);", "for (var x = (a in b); ; )");
    expect("for (x = (a in b);;);", "for (x = (a in b); ; )");
    expect("for (x == (a in b);;);", "for (x == (a in b); ; )");
    expect("for (1 * (x == a in b);;);", "for (1 * (x == a in b); ; )");
    expect("for (a ? b : x = (c in d);;);", "for (a ? b : x = (c in d); ; )");
    expect("for (var x = y = (a in b);;);", "for (var x = y = (a in b); ; )");
    expect("for ([a in b];;);", "for ([a in b]; ; )");
    expect("for (x(a in b);;);", "for (x(a in b); ; )");
    expect("for (x[a in b];;);", "for (x[a in b]; ; )");
    expect("for (x?.[a in b];;);", "for (x?.[a in b]; ; )");
    expect("for ((x => a in b);;);", "for ((x) => (a in b); ; )");

    // Make sure for-of loops with commas are wrapped in parentheses
    expect("for (let a in b, c);", "for (let a in b, c)");
    expect("for (let a of (b, c));", "for (let a of (b, c))");
}

#[test]
fn function() {
    expect("function foo(a = (b, c), ...d) {}", "function foo(a=(b,c),...d){}");
    expect(
        "function foo({[1 + 2]: a = 3} = {[1 + 2]: 3}) {}",
        "function foo({[1+2]:a=3}={[1+2]:3}){}",
    );
    expect(
        "function foo([a = (1, 2), ...[b, ...c]] = [1, [2, 3]]) {}",
        "function foo([a=(1,2),...[b,...c]]=[1,[2,3]]){}",
    );
    expect("function foo([] = []) {}", "function foo([]=[]){}");
    expect("function foo([,] = [,]) {}", "function foo([,]=[,]){}");
    expect("function foo([,,] = [,,]) {}", "function foo([,,]=[,,]){}");
}

#[test]
#[ignore]
fn comments_and_parentheses() {
    expect("(/* foo */ { x() { foo() } }.x());", "/* foo */\n({ x() {\n  foo();\n} }).x();\n");
    expect(
        "(/* foo */ function f() { foo(f) }());",
        "/* foo */\n(function f() {\n  foo(f);\n})();\n",
    );
    expect(
        "(/* foo */ class x { static y() { foo(x) } }.y());",
        "/* foo */\n(class x {\n  static y() {\n    foo(x);\n  }\n}).y();\n",
    );
    expect("(/* @__PURE__ */ (() => foo())());", "/* @__PURE__ */ (() => foo())();\n");
    expect(
        "export default (/* foo */ function f() {});",
        "export default (\n  /* foo */\n  function f() {\n  }\n);\n",
    );
    expect(
        "export default (/* foo */ class x {});",
        "export default (\n  /* foo */\n  class x {\n  }\n);\n",
    );
    expect("x = () => (/* foo */ {});", "x = () => (\n  /* foo */\n  {}\n);\n");
    expect("for ((/* foo */ let).x of y) ;", "for (\n  /* foo */\n  (let).x of y\n)\n  ;\n");
    expect("for (/* foo */ (let).x of y) ;", "for (\n  /* foo */\n  (let).x of y\n)\n  ;\n");
    expect(
        "function *x() { yield (/* foo */ y) }",
        "function* x() {\n  yield (\n    /* foo */\n    y\n  );\n}\n",
    );
}

#[test]
#[ignore]
fn pure_comment() {
    expect("(function() {})", "(function() {\n});\n");
    expect("(function() {})()", "(function() {\n})();\n");
    expect("/*@__PURE__*/(function() {})()", "/* @__PURE__ */ (function() {\n})();\n");

    expect("new (function() {})", "new function() {\n}();\n");
    expect("new (function() {})()", "new function() {\n}();\n");
    expect("/*@__PURE__*/new (function() {})()", "/* @__PURE__ */ new function() {\n}();\n");

    expect("export default (function() {})", "export default (function() {\n});\n");
    expect("export default (function() {})()", "export default (function() {\n})();\n");
    expect(
        "export default /*@__PURE__*/(function() {})()",
        "export default /* @__PURE__ */ (function() {\n})();\n",
    );
}

#[test]
fn generator() {
    expect("function* foo() {}", "function*foo(){}");
    expect("(function* () {})", "(function*(){})");
    expect("(function* foo() {})", "(function*foo(){})");

    expect("class Foo { *foo() {} }", "class Foo{*foo(){}}");
    expect("class Foo { static *foo() {} }", "class Foo{static *foo(){}}");
    expect("class Foo { *[foo]() {} }", "class Foo{*[foo](){}}");
    expect("class Foo { static *[foo]() {} }", "class Foo{static *[foo](){}}");

    expect("(class { *foo() {} })", "(class{*foo(){}})");
    expect("(class { static *foo() {} })", "(class{static *foo(){}})");
    expect("(class { *[foo]() {} })", "(class{*[foo](){}})");
    expect("(class { static *[foo]() {} })", "(class{static *[foo](){}})");
}

#[test]
fn arrow() {
    expect("() => {}", "()=>{}");
    expect("x => (x, 0)", "(x)=>(x,0)");
    expect("x => {y}", "(x)=>{y}");
    expect("(a = (b, c), ...d) => {}", "(a=(b,c),...d)=>{}");
    expect("({[1 + 2]: a = 3} = {[1 + 2]: 3}) => {}", "({[1+2]:a=3}={[1+2]:3})=>{}");
    expect(
        "([a = (1, 2), ...[b, ...c]] = [1, [2, 3]]) => {}",
        "([a=(1,2),...[b,...c]]=[1,[2,3]])=>{}",
    );
    expect("([] = []) => {}", "([]=[])=>{}");
    expect("([,] = [,]) => {}", "([,]=[,])=>{}");
    expect("([,,] = [,,]) => {}", "([,,]=[,,])=>{}");
    expect("a = () => {}", "a=()=>{}");
    expect("a || (() => {})", "a||(()=>{})");
    // expect("({a = b, c = d}) => {}", "({a=b,c=d})=>{}");
    // expect("([{a = b, c = d} = {}] = []) => {}", "([{a=b,c=d}={}]=[])=>{}");
    expect("({a: [b = c] = []} = {}) => {}", "({a:[b=c]=[]}={})=>{}");

    // These are not arrow functions but initially look like one
    // expect("(a = b, c)", "a=b,c");
    // expect("([...a = b])", "[...a=b]");
    // expect("([...a, ...b])", "[...a,...b]");
    expect("({a: b, c() {}})", "({a:b,c(){}})");
    expect("({a: b, get c() {}})", "({a:b,get c(){}})");
    expect("({a: b, set c(x) {}})", "({a:b,set c(x){}})");
}

#[test]
fn class() {
    expect("class Foo extends (a, b) {}", "class Foo extends (a,b){}");
    expect("class Foo { get foo() {} }", "class Foo{get foo(){}}");
    expect("class Foo { set foo(x) {} }", "class Foo{set foo(x){}}");
    expect("class Foo { static foo() {} }", "class Foo{static foo(){}}");
    expect("class Foo { static get foo() {} }", "class Foo{static get foo(){}}");
    expect("class Foo { static set foo(x) {} }", "class Foo{static set foo(x){}}");
}

#[test]
#[ignore]
fn private_identifiers() {
    expect(
        "class Foo { #foo; foo() { return #foo in this } }",
        "class Foo {\n  #foo;\n  foo() {\n    return #foo in this;\n  }\n}\n",
    );
    expect(
        "class Foo { #foo; foo() { return #foo in this } }",
        "class Foo{#foo;foo(){return#foo in this}}",
    );
}

#[test]
#[ignore]
fn import() {
    expect("import('path');", "import(\"path\");\n"); // The semicolon must not be a separate statement

    // Test preservation of Webpack-specific comments
    expect(
        "import(\n  // webpackFoo: 1\n  // webpackBar: 2\n  \"path\"\n);\n",
        "import(// webpackFoo: 1\n // webpackBar: 2\n 'path');",
    );
    expect(
        "import(// webpackFoo: 1\n // webpackBar: 2\n 'path', {type: 'module'});",
        "import(\n  // webpackFoo: 1\n  // webpackBar: 2\n  \"path\",\n  { type: \"module\" }\n);\n",
    );
    expect(
        "import(/* webpackFoo: 1 */ /* webpackBar: 2 */ 'path');",
        "import(\n  /* webpackFoo: 1 */\n  /* webpackBar: 2 */\n  \"path\"\n);\n",
    );
    expect(
        "import(/* webpackFoo: 1 */ /* webpackBar: 2 */ 'path', {type: 'module'});",
        "import(\n  /* webpackFoo: 1 */\n  /* webpackBar: 2 */\n  \"path\",\n  { type: \"module\" }\n);\n",
    );
    expect(
        "import(\n    /* multi\n     * line\n     * webpackBar: */ 'path');",
        "import(\n  /* multi\n   * line\n   * webpackBar: */\n  \"path\"\n);\n",
    );
    expect(
        "import(/* webpackFoo: 1 */ 'path' /* webpackBar:2 */);",
        "import(\n  /* webpackFoo: 1 */\n  \"path\"\n  /* webpackBar:2 */\n);\n",
    );
    expect(
        "import(/* webpackFoo: 1 */ 'path' /* webpackBar:2 */ ,);",
        "import(\n  /* webpackFoo: 1 */\n  \"path\"\n);\n",
    ); // Not currently handled
    expect(
        "import(/* webpackFoo: 1 */ 'path', /* webpackBar:2 */ );",
        "import(\n  /* webpackFoo: 1 */\n  \"path\"\n  /* webpackBar:2 */\n);\n",
    );
    expect(
        "import(/* webpackFoo: 1 */ 'path', { type: 'module' } /* webpackBar:2 */ );",
        "import(\n  /* webpackFoo: 1 */\n  \"path\",\n  { type: \"module\" }\n  /* webpackBar:2 */\n);\n",
    );
    expect(
        "import(new URL('path', /* webpackFoo: these can go anywhere */ import.meta.url))",
        "import(new URL(\n  \"path\",\n  /* webpackFoo: these can go anywhere */\n  import.meta.url\n));\n",
    );
}

#[test]
#[ignore]
fn export_default() {
    expect("export default function() {}", "export default function() {\n}\n");
    expect("export default function foo() {}", "export default function foo() {\n}\n");
    expect("export default async function() {}", "export default async function() {\n}\n");
    expect("export default async function foo() {}", "export default async function foo() {\n}\n");
    expect("export default class {}", "export default class {\n}\n");
    expect("export default class foo {}", "export default class foo {\n}\n");

    expect("export default (function() {})", "export default (function() {\n});\n");
    expect("export default (function foo() {})", "export default (function foo() {\n});\n");
    expect("export default (async function() {})", "export default (async function() {\n});\n");
    expect(
        "export default (async function foo() {})",
        "export default (async function foo() {\n});\n",
    );
    expect("export default (class {})", "export default (class {\n});\n");
    expect("export default (class foo {})", "export default (class foo {\n});\n");

    expect(
        "export default (function() {}.toString())",
        "export default (function() {\n}).toString();\n",
    );
    expect(
        "export default (function foo() {}.toString())",
        "export default (function foo() {\n}).toString();\n",
    );
    expect(
        "export default (async function() {}.toString())",
        "export default (async function() {\n}).toString();\n",
    );
    expect(
        "export default (async function foo() {}.toString())",
        "export default (async function foo() {\n}).toString();\n",
    );
    expect("export default (class {}.toString())", "export default (class {\n}).toString();\n");
    expect(
        "export default (class foo {}.toString())",
        "export default (class foo {\n}).toString();\n",
    );

    expect("export default function() {}", "export default function(){}");
    expect("export default function foo() {}", "export default function foo(){}");
    expect("export default async function() {}", "export default async function(){}");
    expect("export default async function foo() {}", "export default async function foo(){}");
    expect("export default class {}", "export default class{}");
    expect("export default class foo {}", "export default class foo{}");
}

#[test]
fn whitespace() {
    expect("- -x", "- -x");
    expect("+ -x", "+-x");
    expect("- +x", "-+x");
    expect("+ +x", "+ +x");
    expect("- --x", "- --x");
    expect("+ --x", "+--x");
    expect("- ++x", "-++x");
    expect("+ ++x", "+ ++x");

    expect("- -x", "- -x");
    expect("+ -x", "+-x");
    expect("- +x", "-+x");
    expect("+ +x", "+ +x");
    expect("- --x", "- --x");
    expect("+ --x", "+--x");
    expect("- ++x", "-++x");
    expect("+ ++x", "+ ++x");

    expect("x - --y", "x- --y");
    expect("x + --y", "x+--y");
    expect("x - ++y", "x-++y");
    expect("x + ++y", "x+ ++y");

    expect("x-- > y", "x-- >y");
    expect("x < !--y", "x<! --y");
    // expect("x > !--y", "x>!--y");
    expect("!--y", "!--y");

    expect("1 + -0", "1+-0");
    expect("1 - -0", "1- -0");
    // expect("1 + -Infinity", "1+-Infinity");
    // expect("1 - -Infinity", "1- -Infinity");

    // expect("/x/ / /y/", "/x// /y/");
    expect("/x/ + Foo", "/x/+Foo");
    expect("/x/ instanceof Foo", "/x/ instanceof Foo");
    // expect("[x] instanceof Foo", "[x]instanceof Foo");

    expect("throw x", "throw x");
    expect("throw typeof x", "throw typeof x");
    expect("throw delete x", "throw delete x");
    expect("throw function(){}", "throw function(){}");

    expect("x in function(){}", "x in function(){}");
    expect("x instanceof function(){}", "x instanceof function(){}");
    expect("π in function(){}", "π in function(){}");
    expect("π instanceof function(){}", "π instanceof function(){}");

    expect("()=>({})", "()=>({})");
    // expect("()=>({}[1])", "()=>({})[1]");
    expect("()=>({}+0)", "()=>({}+0)");
    expect("()=>function(){}", "()=>function(){}");

    expect("(function(){})", "(function(){})");
    expect("(class{})", "(class{})");
    expect("({})", "({})");
}

#[test]
#[ignore]
fn mangle() {
    expect("let x = '\\n'", "let x = `\n`;\n");
    expect("let x = `\n`", "let x = `\n`;\n");
    expect("let x = '\\n${}'", "let x = \"\\n${}\";\n");
    expect("let x = `\n\\${}`", "let x = \"\\n${}\";\n");
    expect("let x = `\n\\${}${y}\\${}`", "let x = `\n\\${}${y}\\${}`;\n");
}

#[test]
#[ignore]
fn minify() {
    expect("0.1", ".1;");
    expect("1.2", "1.2;");

    expect("() => {}", "()=>{};");
    expect("(a) => {}", "a=>{};");
    expect("(...a) => {}", "(...a)=>{};");
    expect("(a = 0) => {}", "(a=0)=>{};");
    expect("(a, b) => {}", "(a,b)=>{};");

    expect("true ** 2", "true ** 2;\n");
    expect("false ** 2", "false ** 2;\n");
    expect("true ** 2", "true**2;");
    expect("false ** 2", "false**2;");
    expect("true ** 2", "(!0) ** 2;\n");
    expect("false ** 2", "(!1) ** 2;\n");

    expect("import a from 'path'", "import a from\"path\";");
    expect("import * as ns from 'path'", "import*as ns from\"path\";");
    expect("import {a, b as c} from 'path'", "import{a,b as c}from\"path\";");
    expect("import {a, ' ' as c} from 'path'", "import{a,\" \"as c}from\"path\";");

    expect("export * as ns from 'path'", "export*as ns from\"path\";");
    expect("export * as ' ' from 'path'", "export*as\" \"from\"path\";");
    expect("export {a, b as c} from 'path'", "export{a,b as c}from\"path\";");
    expect("export {' ', '-' as ';'} from 'path'", "export{\" \",\"-\"as\";\"}from\"path\";");
    expect("let a, b; export {a, b as c}", "let a,b;export{a,b as c};");
    expect("let a, b; export {a, b as ' '}", "let a,b;export{a,b as\" \"};");

    // Print some strings using template literals when minifying
    expect("x = '\\n'", "x = \"\\n\";\n");
    expect("x = '\\n'", "x = `\n`;\n");
    expect("x = {'\\n': 0}", "x = { \"\\n\": 0 };\n");
    expect("(class{'\\n' = 0})", "(class {\n  \"\\n\" = 0;\n});\n");
    expect("class Foo{'\\n' = 0}", "class Foo {\n  \"\\n\" = 0;\n}\n");

    // Special identifiers must not be minified
    expect("exports", "exports;");
    expect("require", "require;");
    expect("module", "module;");

    // Comment statements must not affect their surroundings when minified
    expect("//!single\nthrow 1 + 2", "//!single\nthrow 1+2;");
    expect("/*!multi-\nline*/\nthrow 1 + 2", "/*!multi-\nline*/throw 1+2;");
}

#[test]
#[ignore]
fn infinity() {
    expect("x = Infinity", "x=1/0");
    expect("x = -Infinity", "x=-1/0");
    expect("x = (Infinity).toString", "x=(1/0).toString");
    expect("x = (-Infinity).toString", "x=(-1/0).toString");
    expect("x = Infinity ** 2", "x=(1/0) ** 2");
    expect("x = (-Infinity) ** 2", "x=(-1/0)**2");
    expect("x = Infinity * y", "x=1/0*y");
    expect("x = Infinity / y", "x=1/0/y");
    expect("x = y * Infinity", "x=y*(1/0)");
    expect("x = y / Infinity", "x=y/(1/0)");
    expect("throw Infinity", "throw 1/0");

    expect("x = Infinity", "x=1/0");
    expect("x = -Infinity", "x=-1/0");
    expect("x = (Infinity).toString", "x=(1/0).toString");
    expect("x = (-Infinity).toString", "x=(-1/0).toString");
    expect("x = Infinity ** 2", "x=(1/0)**2");
    expect("x = (-Infinity) ** 2", "x=(-1/0)**2");
    expect("x = Infinity * y", "x=1/0*y");
    expect("x = Infinity / y", "x=1/0/y");
    expect("x = y * Infinity", "x=y*(1/0)");
    expect("x = y / Infinity", "x=y/(1/0)");
    expect("throw Infinity", "throw 1/0");
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
#[ignore]
fn avoid_slash_script() {}

#[test]
fn binary_operator_visitor() {
    // Make sure the inner "/*b*/" comment doesn't disappear due to weird binary visitor stuff
    // expect(
    // "x = (0, /*a*/ (0, /*b*/ (0, /*c*/ 1 == 2) + 3) * 4)",
    // "x = /*a*/\n/*b*/\n(/*c*/\n!1 + 3) * 4;\n",
    // );

    // Make sure deeply-nested ASTs don't cause a stack overflow
    let x = format!("x=f(){}", "||f()".repeat(1000)); // TODO: change this to 10_000
    expect_same(&x);
}
