use super::test;

/// Esbuild minfication tests
///
/// In esbuild `internal/js_parser/js_parser_test.go`
///
/// ```go
/// var messagesFile *os.File
///
/// func messages() *os.File {
///  if messagesFile == nil {
///    messagesFile, _ = os.Create("tests.out")
///  }
///  return messagesFile
/// }
///
/// func expectPrintedMangle(t *testing.T, contents string, expected string) {
///  contents = strings.ReplaceAll(contents, "\n", "")
///  contents = strings.ReplaceAll(contents, "\"", "'")
///  expected = strings.ReplaceAll(expected, "\n", "")
///  expected = strings.ReplaceAll(expected, "\"", "'")
///  messages().WriteString("test(\"" + contents + "\", \"" + expected + "\");\n");
///  t.Helper()
///  expectPrintedCommon(t, contents, expected, config.Options{
///   MinifySyntax: true,
///  })
/// }
/// ```
#[test]
fn js_parser_test() {
    test("x = {['_proto_']: x}", "x = { _proto_: x };");
    test("x = {['__proto__']: x}", "x = { ['__proto__']: x };");
    test("x = { '0': y }", "x = { 0: y };");
    test("x = { '123': y }", "x = { 123: y };");
    test("x = { '-123': y }", "x = { '-123': y };");
    test("x = { '-0': y }", "x = { '-0': y };");
    test("x = { '01': y }", "x = { '01': y };");
    test("x = { '-01': y }", "x = { '-01': y };");
    test("x = { '0x1': y }", "x = { '0x1': y };");
    test("x = { '-0x1': y }", "x = { '-0x1': y };");
    test("x = { '2147483647': y }", "x = { 2147483647: y };");
    test("x = { '2147483648': y }", "x = { '2147483648': y };");
    test("x = { '-2147483648': y }", "x = { '-2147483648': y };");
    test("x = { '-2147483649': y }", "x = { '-2147483649': y };");
    test("x.x; y['y']", "x.x, y.y;");
    // test("({y: y, 'z': z} = x)", "({ y, z } = x);");
    // test("var {y: y, 'z': z} = x", "var { y, z } = x;");
    test("x = {y: 1, 'z': 2}", "x = { y: 1, z: 2 };");
    test("x = {y() {}, 'z'() {}}", "x = { y() {}, z() {} };");
    test("x = {get y() {}, set 'z'(z) {}}", "x = { get y() {}, set z(z) {} };");
    test("x = class {y = 1; 'z' = 2}", "x = class { y = 1; z = 2;};");
    test("x = class {y() {}; 'z'() {}}", "x = class { y() { } z() { }};");
    test("x = class {get y() {}; set 'z'(z) {}}", "x = class { get y() { } set z(z) { }};");
    test("function foo() { return undefined }", "function foo() {}");
    test("function* foo() { return undefined }", "function* foo() {}");
    test("async function foo() { return undefined }", "async function foo() {}");
    test("async function* foo() { return undefined }", "async function* foo() { return void 0;}");
    // test("function f() { x() } function f() { y() }", "function f() { y();}");
    // test("function f() { x() } function *f() { y() }", "function* f() { y();}");
    // test("function *f() { x() } function f() { y() }", "function f() { y();}");
    // test("function *f() { x() } function *f() { y() }", "function* f() { y();}");
    // test("function f() { x() } async function f() { y() }", "async function f() { y();}");
    // test("async function f() { x() } function f() { y() }", "function f() { y();}");
    // test("async function f() { x() } async function f() { y() }", "async function f() { y();}");
    test("var f; function f() {}", "var f;function f() {}");
    test("function f() {} var f", "function f() {}var f;");
    // test("var f; function f() { x() } function f() { y() }", "var f;function f() { y();}");
    // test("function f() { x() } function f() { y() } var f", "function f() { y();}var f;");
    test(
        "function f() { x() } var f; function f() { y() }",
        "function f() { x();}var f;function f() { y();}",
    );
    // test("class Foo { ['constructor'] = 0 }", "class Foo { ['constructor'] = 0;}");
    // test("class Foo { ['constructor']() {} }", "class Foo { ['constructor']() { }}");
    // test("class Foo { *['constructor']() {} }", "class Foo { *['constructor']() { }}");
    // test("class Foo { get ['constructor']() {} }", "class Foo { get ['constructor']() { }}");
    // test("class Foo { set ['constructor'](x) {} }", "class Foo { set ['constructor'](x) { }}");
    // test("class Foo { async ['constructor']() {} }", "class Foo { async ['constructor']() { }}");
    // test("class Foo { static ['constructor'] = 0 }", "class Foo { static ['constructor'] = 0;}");
    // test("class Foo { static ['constructor']() {} }", "class Foo { static constructor() { }}");
    // test("class Foo { static *['constructor']() {} }", "class Foo { static *constructor() { }}");
    // test(
    // "class Foo { static get ['constructor']() {} }",
    // "class Foo { static get constructor() { }}",
    // );
    // test(
    // "class Foo { static set ['constructor'](x) {} }",
    // "class Foo { static set constructor(x) { }}",
    // );
    // test(
    // "class Foo { static async ['constructor']() {} }",
    // "class Foo { static async constructor() { }}",
    // );
    // test("class Foo { ['prototype'] = 0 }", "class Foo { prototype = 0;}");
    // test("class Foo { ['prototype']() {} }", "class Foo { prototype() { }}");
    // test("class Foo { *['prototype']() {} }", "class Foo { *prototype() { }}");
    // test("class Foo { get ['prototype']() {} }", "class Foo { get prototype() { }}");
    // test("class Foo { set ['prototype'](x) {} }", "class Foo { set prototype(x) { }}");
    // test("class Foo { async ['prototype']() {} }", "class Foo { async prototype() { }}");
    // test("class Foo { static ['prototype'] = 0 }", "class Foo { static ['prototype'] = 0;}");
    // test("class Foo { static ['prototype']() {} }", "class Foo { static ['prototype']() { }}");
    // test("class Foo { static *['prototype']() {} }", "class Foo { static *['prototype']() { }}");
    // test(
    // "class Foo { static get ['prototype']() {} }",
    // "class Foo { static get ['prototype']() { }}",
    // );
    // test(
    // "class Foo { static set ['prototype'](x) {} }",
    // "class Foo { static set ['prototype'](x) { }}",
    // );
    // test(
    // "class Foo { static async ['prototype']() {} }",
    // "class Foo { static async ['prototype']() { }}",
    // );
    // test(
    // "class Foo { constructor() {} ['constructor']() {} }",
    // "class Foo { constructor() { } ['constructor']() { }}",
    // );
    // test(
    // "class Foo { static constructor() {} static ['constructor']() {} }",
    // "class Foo { static constructor() { } static constructor() { }}",
    // );
    test("class x { '0' = y }", "class x { 0 = y;}");
    test("class x { '123' = y }", "class x { 123 = y;}");
    // test("class x { ['-123'] = y }", "class x { '-123' = y;}");
    test("class x { '-0' = y }", "class x { '-0' = y;}");
    test("class x { '01' = y }", "class x { '01' = y;}");
    test("class x { '-01' = y }", "class x { '-01' = y;}");
    test("class x { '0x1' = y }", "class x { '0x1' = y;}");
    test("class x { '-0x1' = y }", "class x { '-0x1' = y;}");
    test("class x { '2147483647' = y }", "class x { 2147483647 = y;}");
    test("class x { '2147483648' = y }", "class x { '2147483648' = y;}");
    // test("class x { ['-2147483648'] = y }", "class x { '-2147483648' = y;}");
    test("class x { ['-2147483649'] = y }", "class x { '-2147483649' = y;}");
    test("class Foo { static {} }", "class Foo {}");
    test("class Foo { static { 123 } }", "class Foo {}");
    // test("class Foo { static { /* @__PURE__ */ foo() } }", "class Foo {}");
    test("class Foo { static { foo() } }", "class Foo { static { foo(); }}");
    test("x: break x", "");
    test("x: { break x; foo() }", "");
    // test("y: while (foo()) x: { break x; foo() }", "for (; foo(); ) ;");
    // test("y: while (foo()) x: { break y; foo() }", "y: for (; foo(); ) break y;");
    // test("x: { y: { z: { foo(); break x; } } }", "x: { foo(); break x;}");
    // test("x: { class X { static { new X } } }", "{ class X { static {  new X(); } }}");
    test("(() => {}) ? a : b", "a;");
    // test("x = `a${1 + `b${2}c` + 3}d`", "x = `a1b2c3d`;");
    test("x = 1 ? a : b", "x = a;");
    test("x = 0 ? a : b", "x = b;");
    test("x; 1 ? 0 : ()=>{}; (()=>{})()", "x;");
    test("x; 0 ? ()=>{} : 1; (()=>{})()", "x;");
    test("if (1) 0; else ()=>{}; (()=>{})()", "");
    test("if (0) ()=>{}; else 1; (()=>{})()", "");
    test("var a; while (1) ;", "for (var a;;) ;");
    test("let a; while (1) ;", "let a;for (;;) ;");
    test("const a=0; while (1) ;", "const a = 0;for (;;) ;");
    test("var a; for (var b;;) ;", "for (var a, b;;) ;");
    test("let a; for (let b;;) ;", "let a;for (let b;;) ;");
    test("const a=0; for (const b = 1;;) ;", "const a = 0;for (let b = 1;;) ;");
    test("export var a; while (1) ;", "export var a;for (;;) ;");
    test("export let a; while (1) ;", "export let a;for (;;) ;");
    test("export const a=0; while (1) ;", "export const a = 0;for (;;) ;");
    test("export var a; for (var b;;) ;", "export var a;for (var b;;) ;");
    test("export let a; for (let b;;) ;", "export let a;for (let b;;) ;");
    test("export const a=0; for (const b = 1;;) ;", "export const a = 0;for (let b = 1;;) ;");
    test("var a; for (let b;;) ;", "var a;for (let b;;) ;");
    test("let a; for (const b=0;;) ;", "let a;for (let b = 0;;) ;");
    test("const a=0; for (var b;;) ;", "const a = 0;for (var b;;) ;");
    test("a(); while (1) ;", "for (a();;) ;");
    test("a(); for (b();;) ;", "for (a(), b();;) ;");
    // test("for (; ;) if (x) break;", "for (; !x; ) ;");
    // test("for (; ;) if (!x) break;", "for (; x; ) ;");
    // test("for (; a;) if (x) break;", "for (; a && !x; ) ;");
    // test("for (; a;) if (!x) break;", "for (; a && x; ) ;");
    // test("for (; ;) { if (x) break; y(); }", "for (; !x; ) y();");
    // test("for (; a;) { if (x) break; y(); }", "for (; a && !x; ) y();");
    // test("for (; ;) if (x) break; else y();", "for (; !x; ) y();");
    // test("for (; a;) if (x) break; else y();", "for (; a && !x; ) y();");
    // test("for (; ;) { if (x) break; else y(); z(); }", "for (; !x; ) y(), z();");
    // test("for (; a;) { if (x) break; else y(); z(); }", "for (; a && !x; ) y(), z();");
    // test("for (; ;) if (x) y(); else break;", "for (; x; ) y();");
    // test("for (; ;) if (!x) y(); else break;", "for (; !x; ) y();");
    // test("for (; a;) if (x) y(); else break;", "for (; a && x; ) y();");
    // test("for (; a;) if (!x) y(); else break;", "for (; a && !x; ) y();");
    // test("for (; ;) { if (x) y(); else break; z(); }", "for (; x; ) { y(); z();}");
    // test("for (; a;) { if (x) y(); else break; z(); }", "for (; a && x; ) { y(); z();}");
    // test("while (x) { if (1) break; z(); }", "for (; x; ) break;");
    // test("while (x) { if (1) continue; z(); }", "for (; x; ) ;");
    // test(
    // "foo: while (a) while (x) { if (1) continue foo; z(); }",
    // "foo: for (; a; ) for (; x; ) continue foo;",
    // );
    test("while (x) { y(); if (1) break; z(); }", "for (; x; ) { y(); break;}");
    test("while (x) { y(); if (1) continue; z(); }", "for (; x; ) y();");
    test("while (x) { y(); debugger; if (1) continue; z(); }", "for (; x; ) { y(); debugger; }");
    test("while (x) { let y = z(); if (1) continue; z(); }", "for (; x; ) { let y = z();}");
    test(
        "while (x) { debugger; if (y) { if (1) break; z() } }",
        "for (; x; ) { debugger; if (y) break; }",
    );
    // test("while (x) { debugger; if (y) { if (1) continue; z() } }", "for (; x; ) { debugger; y; }");
    test(
        "while (x) { debugger; if (1) { if (1) break; z() } }",
        "for (; x; ) { debugger; break; }",
    );
    test("while (x) { debugger; if (1) { if (1) continue; z() } }", "for (; x; ) debugger;");
    // test("while (x()) continue", "for (; x(); ) ;");
    test("while (x) { y(); continue }", "for (; x; ) y();");
    test("while (x) { if (y) { z(); continue } }", "for (; x; ) if (y) { z(); continue; }");
    // test(
    // "label: while (x) while (y) { z(); continue label }",
    // "label: for (; x; ) for (; y; ) { z(); continue label;}",
    // );
    // test("while (x) { if (y) continue; z(); }", "for (; x; ) y || z();");
    // test("while (x) { if (y) continue; else z(); w(); }", "for (; x; ) y || (z(), w());");
    // test("while (x) { t(); if (y) continue; z(); }", "for (; x; ) t(), !y && z();");
    // test(
    // "while (x) { t(); if (y) continue; else z(); w(); }",
    // "for (; x; ) t(), !y && (z(), w());",
    // );
    // test("while (x) { debugger; if (y) continue; z(); }", "for (; x; ) { debugger; y || z(); }");
    // test(
    // "while (x) { debugger; if (y) continue; else z(); w(); }",
    // "for (; x; ) { debugger; y || (z(), w());}",
    // );
    // test(
    // "while (x) { if (y) continue; function y() {} }",
    // "for (; x; ) { let y = function() { }; var y = y; }",
    // );
    test("while (x) { if (y) continue; let y }", "for (; x; ) { if (y) continue; let y; }");
    // test("while (x) { if (y) continue; var y }", "for (; x; ) if (!y) var y; ");
    test("console.log(undefined)", "console.log(void 0);");
    test("console.log(+undefined)", "console.log(NaN);");
    test("console.log(undefined + undefined)", "console.log(void 0 + void 0);");
    test("const x = undefined", "const x = void 0;");
    test("let x = undefined", "let x;");
    test("var x = undefined", "var x = void 0;");
    // test("function foo(a) { if (!a) return undefined; a() }", "function foo(a) { a && a(); }");
    test("delete undefined", "delete undefined;");
    test("undefined--", "undefined--;");
    test("undefined++", "undefined++;");
    test("--undefined", "--undefined;");
    test("++undefined", "++undefined;");
    test("undefined = 1", "undefined = 1;");
    test("[undefined] = 1", "[undefined] = 1;");
    test("({x: undefined} = 1)", "({ x: undefined } = 1);");
    // test("with (x) y(undefined); z(undefined)", "with (x) y(undefined);z(void 0);");
    // test(
    // "with (x) while (i) y(undefined); z(undefined)",
    // "with (x) for (; i; ) y(undefined);z(void 0);",
    // );
    test("x['y']", "x.y;");
    test("x['y z']", "x['y z'];");
    test("x?.['y']", "x?.y;");
    test("x?.['y z']", "x?.['y z'];");
    test("x?.['y']()", "x?.y();");
    test("x?.['y z']()", "x?.['y z']();");
    test("x['y' + 'z']", "x.yz;");
    // test("x?.['y' + 'z']", "x?.['yz'];");
    test("x['0']", "x[0];");
    test("x['123']", "x[123];");
    test("x['-123']", "x[-123];");
    test("x['-0']", "x['-0'];");
    test("x['01']", "x['01'];");
    test("x['-01']", "x['-01'];");
    test("x['0x1']", "x['0x1'];");
    test("x['-0x1']", "x['-0x1'];");
    test("x['2147483647']", "x[2147483647];");
    test("x['2147483648']", "x['2147483648'];");
    // test("x['-2147483648']", "x[-2147483648];");
    test("x['-2147483649']", "x['-2147483649'];");
    test("while(1) { while (1) {} }", "for (;;) for (;;) ;");
    test("while(1) { const x = y; }", "for (;;) { let x = y;}");
    test("while(1) { let x; }", "for (;;) { let x;}");
    // test("while(1) { var x; }", "for (;;) var x;");
    test("while(1) { class X {} }", "for (;;) { class X { }}");
    // test("while(1) { function x() {} }", "for (;;) var x = function() { };");
    test("while(1) { function* x() {} }", "for (;;) { function* x() { }}");
    test("while(1) { async function x() {} }", "for (;;) { async function x() { }}");
    test("while(1) { async function* x() {} }", "for (;;) { async function* x() { }}");
    test(
        "function _() { x(); switch (y) { case z: return w; } }",
        "function _() { switch (x(), y) { case z:  return w; }}",
    );
    test(
        "function _() { if (t) { x(); switch (y) { case z: return w; } } }",
        "function _() { if (t) switch (x(), y) { case z:  return w; } }",
    );
    test("a = '' + 0", "a = '0';");
    test("a = 0 + ''", "a = '0';");
    test("a = '' + b", "a = '' + b;");
    test("a = b + ''", "a = b + '';");
    // test("a = [] + 0", "a = '0';");
    // test("a = 0 + []", "a = '0';");
    test("a = [] + b", "a = [] + b;");
    test("a = b + []", "a = b + [];");
    test("a = [b] + 0", "a = [b] + 0;");
    test("a = 0 + [b]", "a = 0 + [b];");
    // test("a = [1, 2] + ''", "a = '1,2';");
    // test("a = [1, 0, 2] + ''", "a = '1,0,2';");
    // test("a = [1, null, 2] + ''", "a = '1,,2';");
    // test("a = [1, undefined, 2] + ''", "a = '1,,2';");
    // test("a = [1, true, 2] + ''", "a = '1,true,2';");
    // test("a = [1, false, 2] + ''", "a = '1,false,2';");
    test("a = [1, , 2] + ''", "a = [1, , 2] + '';");
    test("a = [1, , ,] + ''", "a = [1, , ,] + '';");
    // test("a = {} + 0", "a = '[object Object]0';");
    // test("a = 0 + {}", "a = '0[object Object]';");
    test("a = {} + b", "a = {} + b;");
    test("a = b + {}", "a = b + {};");
    test("a = {toString:()=>1} + 0", "a = { toString: () => 1 } + 0;");
    test("a = 0 + {toString:()=>1}", "a = 0 + { toString: () => 1 };");
    // test("a = '' + `${b}`", "a = `${b}`;");
    // test("a = `${b}` + ''", "a = `${b}`;");
    // test("a = '' + typeof b", "a = typeof b;");
    // test("a = typeof b + ''", "a = typeof b;");
    // test("a = [] + `${b}`", "a = `${b}`;");
    // test("a = `${b}` + []", "a = `${b}`;");
    // test("a = [] + typeof b", "a = typeof b;");
    // test("a = typeof b + []", "a = typeof b;");
    // test("a = [b] + `${b}`", "a = [b] + `${b}`;");
    // test("a = `${b}` + [b]", "a = `${b}` + [b];");
    // test("a = {} + `${b}`", "a = `[object Object]${b}`;");
    // test("a = `${b}` + {}", "a = `${b}[object Object]`;");
    test("a = {} + typeof b", "a = {} + typeof b;");
    test("a = typeof b + {}", "a = typeof b + {};");
    test("a = {toString:()=>1} + `${b}`", "a = { toString: () => 1 } + `${b}`;");
    test("a = `${b}` + {toString:()=>1}", "a = `${b}` + { toString: () => 1 };");
    test("a = '' + false", "a = 'false';");
    test("a = '' + true", "a = 'true';");
    test("a = false + ''", "a = 'false';");
    test("a = true + ''", "a = 'true';");
    // test("a = 1 + false + ''", "a = 1 + false + '';");
    // test("a = 0 + true + ''", "a = 0 + true + '';");
    test("a = '' + null", "a = 'null';");
    test("a = null + ''", "a = 'null';");
    test("a = '' + undefined", "a = 'undefined';");
    test("a = undefined + ''", "a = 'undefined';");
    test("a = '' + 0n", "a = '0';");
    test("a = '' + 1n", "a = '1';");
    test("a = '' + 123n", "a = '123';");
    test("a = '' + 1_2_3n", "a = '123';");
    // test("a = '' + 0b0n", "a = '' + 0b0n;");
    // test("a = '' + 0o0n", "a = '' + 0o0n;");
    // test("a = '' + 0x0n", "a = '' + 0x0n;");
    // test("a = '' + /a\\b/ig", "a = '/a\\\\b/ig';");
    // test("a = /a\\b/ig + ''", "a = '/a\\\\b/ig';");
    // test("a = '' + ''.constructor", "a = 'function String() { [native code] }';");
    // test("a = ''.constructor + ''", "a = 'function String() { [native code] }';");
    // test("a = '' + /./.constructor", "a = 'function RegExp() { [native code] }';");
    // test("a = /./.constructor + ''", "a = 'function RegExp() { [native code] }';");
    test("''.length++", "''.length++;");
    test("''.length = a", "''.length = a;");
    test("a = ''.len", "a = ''.len;");
    test("a = [].length", "a = 0;");
    test("a = ''.length", "a = 0;");
    test("a = ``.length", "a = 0;");
    test("a = b``.length", "a = b``.length;");
    test("a = 'abc'.length", "a = 3;");
    test("a = 'ȧḃċ'.length", "a = 3;");
    test("a = '👯‍♂️'.length", "a = 5;");
    test("a = 'abc'[-1]", "a = 'abc'[-1];");
    // test("a = 'abc'[-0]", "a = 'a';");
    // test("a = 'abc'[0]", "a = 'a';");
    // test("a = 'abc'[2]", "a = 'c';");
    test("a = 'abc'[3]", "a = 'abc'[3];");
    test("a = 'abc'[NaN]", "a = 'abc'[NaN];");
    test("a = 'abc'[-1e100]", "a = 'abc'[-1e100];");
    test("a = 'abc'[1e100]", "a = 'abc'[1e100];");
    test("a = 'abc'[-Infinity]", "a = 'abc'[-Infinity];");
    test("a = 'abc'[Infinity]", "a = 'abc'[Infinity];");
    test("a = !(b == c)", "a = b != c;");
    test("a = !(b != c)", "a = b == c;");
    test("a = !(b === c)", "a = b !== c;");
    test("a = !(b !== c)", "a = b === c;");
    test("function _() { if (!(a, b)) return c }", "function _() { if (a, !b) return c; }");
    test("a = !(b < c)", "a = !(b < c);");
    test("a = !(b > c)", "a = !(b > c);");
    test("a = !(b <= c)", "a = !(b <= c);");
    test("a = !(b >= c)", "a = !(b >= c);");
    test("a = !!b", "a = !!b;");
    test("a = !!!b", "a = !b;");
    test("a = !!-b", "a = !!-b;");
    test("a = !!void b", "a = !!void b;");
    test("a = !!delete b", "a = delete b;");
    test("a = !!(b + c)", "a = !!(b + c);");
    test("a = !!(b == c)", "a = b == c;");
    test("a = !!(b != c)", "a = b != c;");
    test("a = !!(b === c)", "a = b === c;");
    test("a = !!(b !== c)", "a = b !== c;");
    test("a = !!(b < c)", "a = b < c;");
    test("a = !!(b > c)", "a = b > c;");
    test("a = !!(b <= c)", "a = b <= c;");
    test("a = !!(b >= c)", "a = b >= c;");
    test("a = !!(b in c)", "a = b in c;");
    test("a = !!(b instanceof c)", "a = b instanceof c;");
    test("a = !!(b && c)", "a = !!(b && c);");
    test("a = !!(b || c)", "a = !!(b || c);");
    test("a = !!(b ?? c)", "a = !!(b ?? c);");
    test("a = !!(!b && c)", "a = !!(!b && c);");
    test("a = !!(!b || c)", "a = !!(!b || c);");
    test("a = !!(!b ?? c)", "a = !b;");
    test("a = !!(b && !c)", "a = !!(b && !c);");
    test("a = !!(b || !c)", "a = !!(b || !c);");
    test("a = !!(b ?? !c)", "a = !!(b ?? !c);");
    // test("a = !!(!b && !c)", "a = !b && !c;");
    // test("a = !!(!b || !c)", "a = !b || !c;");
    test("a = !!(!b ?? !c)", "a = !b;");
    test("a = !!(b, c)", "a = (b, !!c);");
    test("a = Boolean(b); var Boolean", "a = Boolean(b);var Boolean;");
    test("a = Boolean()", "a = !1;");
    test("a = Boolean(b)", "a = !!b;");
    test("a = Boolean(!b)", "a = !b;");
    test("a = Boolean(!!b)", "a = !!b;");
    test("a = Boolean(b ? true : false)", "a = !!b;");
    test("a = Boolean(b ? false : true)", "a = !b;");
    // test("a = Boolean(b ? c > 0 : c < 0)", "a = b ? c > 0 : c < 0;");
    // test("a = Boolean((b | c) !== 0)", "a = !!(b | c);");
    // test("a = Boolean(b ? (c | d) !== 0 : (d | e) !== 0)", "a = !!(b ? c | d : d | e);");
    test("a = Number(x)", "a = Number(x);");
    test("a = Number(0n)", "a = Number(0n);");
    test("a = Number(false); var Number", "a = Number(!1);var Number;");
    test("a = Number(0xFFFF_FFFF_FFFF_FFFFn)", "a = Number(0xFFFFFFFFFFFFFFFFn);");
    test("a = Number()", "a = 0;");
    test("a = Number(-123)", "a = -123;");
    test("a = Number(false)", "a = 0;");
    test("a = Number(true)", "a = 1;");
    test("a = Number(undefined)", "a = NaN;");
    test("a = Number(null)", "a = 0;");
    // test("a = Number(b ? !c : !d)", "a = +(b ? !c : !d);");
    // test("a = String(x)", "a = String(x);");
    test("a = String('x'); var String", "a = String('x');var String;");
    test("a = String()", "a = '';");
    test("a = String('x')", "a = 'x';");
    // test("a = String(b ? 'x' : 'y')", "a = b ? 'x' : 'y';");
    test("a = BigInt(x)", "a = BigInt(x);");
    test("a = BigInt(0n); var BigInt", "a = BigInt(0n);var BigInt;");
    test("a = BigInt()", "a = BigInt();");
    test("a = BigInt('0')", "a = BigInt('0');");
    test("a = BigInt(0n)", "a = 0n;");
    // test("a = BigInt(b ? 0n : 1n)", "a = b ? 0n : 1n;");
    test("a = 'xy'.charCodeAt()", "a = 120;");
    test("a = 'xy'.charCodeAt(0)", "a = 120;");
    test("a = 'xy'.charCodeAt(1)", "a = 121;");
    test("a = 'xy'.charCodeAt(-1)", "a = NaN;");
    // test("a = 'xy'.charCodeAt(2)", "a = NaN;");
    // test("a = '🧀'.charCodeAt()", "a = 55358;");
    // test("a = '🧀'.charCodeAt(0)", "a = 55358;");
    // test("a = '🧀'.charCodeAt(1)", "a = 56768;");
    // test("a = '🧀'.charCodeAt(-1)", "a = NaN;");
    // test("a = '🧀'.charCodeAt(2)", "a = NaN;");
    test("a = 'xy'.charCodeAt(NaN)", "a = 'xy'.charCodeAt(NaN);");
    test("a = 'xy'.charCodeAt(-Infinity)", "a = 'xy'.charCodeAt(-Infinity);");
    test("a = 'xy'.charCodeAt(Infinity)", "a = 'xy'.charCodeAt(Infinity);");
    test("a = 'xy'.charCodeAt(0.5)", "a = 'xy'.charCodeAt(0.5);");
    test("a = 'xy'.charCodeAt(1e99)", "a = NaN;");
    test("a = 'xy'.charCodeAt('1')", "a = 121;");
    test("a = 'xy'.charCodeAt(1, 2)", "a = 121;");
    test("a = String.fromCharCode()", "a = '';");
    test("a = String.fromCharCode(0)", "a = '\\0';");
    test("a = String.fromCharCode(120)", "a = 'x';");
    test("a = String.fromCharCode(120, 121)", "a = 'xy';");
    // test("a = String.fromCharCode(55358, 56768)", "a = '🧀';");
    test("a = String.fromCharCode(0x10000)", "a = '\\0';");
    test("a = String.fromCharCode(0x10078, 0x10079)", "a = 'xy';");
    test("a = String.fromCharCode(0x1_0000_FFFF)", "a = '￿';");
    test("a = String.fromCharCode(NaN)", "a = '\\0';");
    test("a = String.fromCharCode(-Infinity)", "a = '\\0';");
    test("a = String.fromCharCode(Infinity)", "a = '\\0';");
    test("a = String.fromCharCode(null)", "a = '\\0';");
    test("a = String.fromCharCode(undefined)", "a = '\\0';");
    test("a = String.fromCharCode('123')", "a = '{';");
    test("a = String.fromCharCode(x)", "a = String.fromCharCode(x);");
    test("a = String.fromCharCode('x')", "a = '\\0';");
    test("a = String.fromCharCode('0.5')", "a = '\\0';");
    test("a = false.toString()", "a = 'false';");
    test("a = true.toString()", "a = 'true';");
    test("a = 'xy'.toString()", "a = 'xy';");
    test("a = 0 .toString()", "a = '0';");
    test("a = (-0).toString()", "a = '0';");
    test("a = 123 .toString()", "a = '123';");
    test("a = (-123).toString()", "a = '-123';");
    test("a = NaN.toString()", "a = 'NaN';");
    test("a = Infinity.toString()", "a = 'Infinity';");
    test("a = (-Infinity).toString()", "a = '-Infinity';");
    // test("a = /a\\b/ig.toString()", "a = '/a\\\\b/ig';");
    test("a = 100 .toString(0)", "a = 100 .toString(0);");
    test("a = 100 .toString(1)", "a = 100 .toString(1);");
    test("a = 100 .toString(2)", "a = '1100100';");
    test("a = 100 .toString(5)", "a = '400';");
    test("a = 100 .toString(8)", "a = '144';");
    test("a = 100 .toString(13)", "a = '79';");
    test("a = 100 .toString(16)", "a = '64';");
    test("a = 10000 .toString(19)", "a = '18d6';");
    test("a = 10000 .toString(23)", "a = 'iki';");
    test("a = 1000000 .toString(29)", "a = '1c01m';");
    test("a = 1000000 .toString(31)", "a = '12hi2';");
    test("a = 1000000 .toString(36)", "a = 'lfls';");
    // test("a = (-1000000).toString(36)", "a = '-lfls';");
    test("a = 0 .toString(36)", "a = '0';");
    test("a = (-0).toString(36)", "a = '0';");
    test("a = false.toString(b)", "a = (!1).toString(b);");
    test("a = true.toString(b)", "a = (!0).toString(b);");
    test("a = 'xy'.toString(b)", "a = 'xy'.toString(b);");
    test("a = 123 .toString(b)", "a = 123 .toString(b);");
    test("a = 0.5.toString()", "a = '0.5';");
    test("a = 1e99.toString(b)", "a = 1e99.toString(b);");
    test("a = /./.toString(b)", "a = /./.toString(b);");
    test("1 ? a() : b()", "a();");
    test("0 ? a() : b()", "b();");
    test("a ? a : b", "a || b;");
    test("a ? b : a", "a && b;");
    test("a.x ? a.x : b", "a.x ? a.x : b;");
    test("a.x ? b : a.x", "a.x ? b : a.x;");
    test("a ? b() : c()", "a ? b() : c();");
    test("!a ? b() : c()", "a ? c() : b();");
    test("!!a ? b() : c()", "a ? b() : c();");
    test("!!!a ? b() : c()", "a ? c() : b();");
    test("if (1) a(); else b()", "a();");
    test("if (0) a(); else b()", "b();");
    test("if (a) b(); else c()", "a ? b() : c();");
    test("if (!a) b(); else c()", "a ? c() : b();");
    test("if (!!a) b(); else c()", "a ? b() : c();");
    test("if (!!!a) b(); else c()", "a ? c() : b();");
    test("if (1) a()", "a();");
    test("if (0) a()", "");
    test("if (a) b()", "a && b();");
    test("if (!a) b()", "a || b();");
    test("if (!!a) b()", "a && b();");
    test("if (!!!a) b()", "a || b();");
    test("if (1) {} else a()", "");
    test("if (0) {} else a()", "a();");
    test("if (a) {} else b()", "a || b();");
    test("if (!a) {} else b()", "a && b();");
    test("if (!!a) {} else b()", "a || b();");
    test("if (!!!a) {} else b()", "a && b();");
    test("if (a) {} else throw b", "if (!a) throw b;");
    test("if (!a) {} else throw b", "if (a) throw b;");
    test("a(); if (b) throw c", "if (a(), b) throw c;");
    test("if (a) if (b) throw c", "if (a && b) throw c;");
    test("if (true) { let a = b; if (c) throw d }", "{ let a = b; if (c) throw d;}");
    // test("if (true) { if (a) throw b; if (c) throw d }", "if (a) throw b;if (c) throw d;");
    // test("if (false) throw a; else { let b = c; if (d) throw e }", "{ let b = c; if (d) throw e;}");
    // test(
    // "if (false) throw a; else { if (b) throw c; if (d) throw e }",
    // "if (b) throw c;if (d) throw e;",
    // );
    test(
        "if (a) { if (b) throw c; else { let d = e; if (f) throw g } }",
        "if (a) { if (b) throw c; { let d = e; if (f) throw g; }}",
    );
    test(
        "if (a) { if (b) throw c; else if (d) throw e; else if (f) throw g }",
        "if (a) { if (b) throw c; if (d) throw e; if (f) throw g;}",
    );
    test("a = b ? true : false", "a = !!b;");
    test("a = b ? false : true", "a = !b;");
    test("a = !b ? true : false", "a = !b;");
    test("a = !b ? false : true", "a = !!b;");
    test("a = b == c ? true : false", "a = b == c;");
    test("a = b != c ? true : false", "a = b != c;");
    test("a = b === c ? true : false", "a = b === c;");
    test("a = b !== c ? true : false", "a = b !== c;");
    test("a ? b(c) : b(d)", "a ? b(c) : b(d);");
    // test("let a; a ? b(c) : b(d)", "let a; a ? b(c) : b(d);");
    test("let a, b; a ? b(c) : b(d)", "let a, b;b(a ? c : d);");
    test("let a, b; a ? b(c, 0) : b(d)", "let a, b; a ? b(c, 0) : b(d);");
    test("let a, b; a ? b(c) : b(d, 0)", "let a, b; a ? b(c) : b(d, 0);");
    test("let a, b; a ? b(c, 0) : b(d, 1)", "let a, b; a ? b(c, 0) : b(d, 1);");
    test("let a, b; a ? b(c, 0) : b(d, 0)", "let a, b;b(a ? c : d, 0);");
    test("let a, b; a ? b(...c) : b(d)", "let a, b; a ? b(...c) : b(d);");
    test("let a, b; a ? b(c) : b(...d)", "let a, b; a ? b(c) : b(...d);");
    test("let a, b; a ? b(...c) : b(...d)", "let a, b;b(...a ? c : d);");
    test("let a, b; a ? b(a) : b(c)", "let a, b;b(a || c);");
    test("let a, b; a ? b(c) : b(a)", "let a, b;b(a && c);");
    test("let a, b; a ? b(...a) : b(...c)", "let a, b;b(...a || c);");
    test("let a, b; a ? b(...c) : b(...a)", "let a, b;b(...a && c);");
    test("let a; a.x ? b(c) : b(d)", "let a;a.x ? b(c) : b(d);");
    test("let a, b; a.x ? b(c) : b(d)", "let a, b; a.x ? b(c) : b(d);");
    // test("let a, b; a ? b.y(c) : b.y(d)", "let a, b; a ? b.y(c) : b.y(d);");
    test("let a, b; a.x ? b.y(c) : b.y(d)", "let a, b; a.x ? b.y(c) : b.y(d);");
    test("a ? b : c ? b : d", "a || c ? b : d;");
    test("a ? b ? c : d : d", "a && b ? c : d;");
    test("a ? c : (b, c)", "a || b, c;");
    test("a ? (b, c) : c", "a && b, c;");
    test("a ? c : (b, d)", "a ? c : (b, d);");
    test("a ? (b, c) : d", "a ? (b, c) : d;");
    test("a ? b || c : c", "a && b || c;");
    test("a ? b || c : d", "a ? b || c : d;");
    test("a ? b && c : c", "a ? b && c : c;");
    test("a ? c : b && c", "(a || b) && c;");
    test("a ? c : b && d", "a ? c : b && d;");
    test("a ? c : b || c", "a ? c : b || c;");
    test("a = b == null ? c : b", "a = b == null ? c : b;");
    test("a = b != null ? b : c", "a = b == null ? c : b;");
    test("let b; a = b == null ? c : b", "let b; a = b ?? c;");
    test("let b; a = b != null ? b : c", "let b; a = b ?? c;");
    test("let b; a = b == null ? b : c", "let b; a = b == null ? b : c;");
    test("let b; a = b != null ? c : b", "let b; a = b == null ? b : c;");
    test("let b; a = null == b ? c : b", "let b; a = b ?? c;");
    test("let b; a = null != b ? b : c", "let b; a = b ?? c;");
    test("let b; a = null == b ? b : c", "let b; a = b == null ? b : c;");
    test("let b; a = null != b ? c : b", "let b; a = b == null ? b : c;");
    test("let b; a = b.x == null ? c : b.x", "let b; a = b.x == null ? c : b.x;");
    test("let b; a = b.x != null ? b.x : c", "let b; a = b.x == null ? c : b.x;");
    test("let b; a = null == b.x ? c : b.x", "let b; a = b.x == null ? c : b.x;");
    test("let b; a = null != b.x ? b.x : c", "let b; a = b.x == null ? c : b.x;");
    test("let b; a = b === null ? c : b", "let b; a = b === null ? c : b;");
    test("let b; a = b !== null ? b : c", "let b; a = b === null ? c : b;");
    test("let b; a = null === b ? c : b", "let b; a = b === null ? c : b;");
    test("let b; a = null !== b ? b : c", "let b; a = b === null ? c : b;");
    test("let b; a = null === b || b === undefined ? c : b", "let b; a = b ?? c;");
    test("let b; a = b !== undefined && b !== null ? b : c", "let b; a = b ?? c;");
    // test("a(b ? 0 : 0)", "a((b, 0));");
    // test("a(b ? +0 : -0)", "a(b ? 0 : -0);");
    // test("a(b ? +0 : 0)", "a((b, 0));");
    // test("a(b ? -0 : 0)", "a(b ? -0 : 0);");
    test("a ? b : b", "a, b;");
    // test("let a; a ? b : b", "let a; b;");
    test("a ? -b : -b", "a, -b;");
    test("a ? b.c : b.c", "a, b.c;");
    test("a ? b?.c : b?.c", "a, b?.c;");
    test("a ? b[c] : b[c]", "a, b[c];");
    test("a ? b() : b()", "a, b();");
    test("a ? b?.() : b?.()", "a, b?.();");
    test("a ? b?.[c] : b?.[c]", "a, b?.[c];");
    test("a ? b == c : b == c", "a, b == c;");
    test("a ? b.c(d + e[f]) : b.c(d + e[f])", "a, b.c(d + e[f]);");
    // test("a ? -b : !b", "a ? -b : b;");
    test("a ? b() : b(c)", "a ? b() : b(c);");
    test("a ? b(c) : b(d)", "a ? b(c) : b(d);");
    test("a ? b?.c : b.c", "a ? b?.c : b.c;");
    test("a ? b?.() : b()", "a ? b?.() : b();");
    test("a ? b?.[c] : b[c]", "a ? b?.[c] : b[c];");
    test("a ? b == c : b != c", "a ? b == c : b != c;");
    test("a ? b.c(d + e[f]) : b.c(d + e[g])", "a ? b.c(d + e[f]) : b.c(d + e[g]);");
    test("(a, b) ? c : d", "a, b ? c : d;");
    test(
        "function _() { return a && ((b && c) && (d && e)) }",
        "function _() { return a && b && c && d && e; }",
    );
    test(
        "function _() { return a || ((b || c) || (d || e)) }",
        "function _() { return a || b || c || d || e; }",
    );
    test(
        "function _() { return a ?? ((b ?? c) ?? (d ?? e)) }",
        "function _() { return a ?? b ?? c ?? d ?? e; }",
    );
    test("if (a) if (b) if (c) d", "a && b && c && d;");
    test("if (!a) if (!b) if (!c) d", "a || b || c || d;");
    test(
        "function _() { let a, b, c; return a != null ? a : b != null ? b : c }",
        "function _() { let a, b, c;return a ?? b ?? c; }",
    );
    test(
        "function _() { if (a) return c; if (b) return d; }",
        "function _() { if (a) return c;if (b) return d; }",
    );
    test(
        "function _() { if (a) return c; if (b) return c; }",
        "function _() { if (a || b) return c; }",
    );
    test(
        "function _() { if (a) return c; if (b) return; }",
        "function _() { if (a) return c;if (b) return; }",
    );
    test(
        "function _() { if (a) return; if (b) return c; }",
        "function _() { if (a) return;if (b) return c; }",
    );
    test("function _() { if (a) return; if (b) return; }", "function _() { if (a || b) return; }");
    test("if (a) throw c; if (b) throw d;", "if (a) throw c;if (b) throw d;");
    test("if (a) throw c; if (b) throw c;", "if (a || b) throw c;");
    // test("while (x) { if (a) break; if (b) break; }", "for (; x && !(a || b); ) ;");
    // test("while (x) { if (a) continue; if (b) continue; }", "for (; x; ) a || b;");
    // test(
    // "while (x) { debugger; if (a) break; if (b) break; }",
    // "for (; x; ) { debugger; if (a || b) break;}",
    // );
    // test(
    // "while (x) { debugger; if (a) continue; if (b) continue; }",
    // "for (; x; ) { debugger; a || b;}",
    // );
    // test(
    // "x: while (x) y: while (y) { if (a) break x; if (b) break y; }",
    // "x: for (; x; ) y: for (; y; ) { if (a) break x; if (b) break y;}",
    // );
    // test(
    // "x: while (x) y: while (y) { if (a) continue x; if (b) continue y; }",
    // "x: for (; x; ) y: for (; y; ) { if (a) continue x; if (b) continue y;}",
    // );
    // test(
    // "x: while (x) y: while (y) { if (a) break x; if (b) break x; }",
    // "x: for (; x; ) for (; y; ) if (a || b) break x;",
    // );
    // test(
    // "x: while (x) y: while (y) { if (a) continue x; if (b) continue x; }",
    // "x: for (; x; ) for (; y; ) if (a || b) continue x;",
    // );
    // test(
    // "x: while (x) y: while (y) { if (a) break y; if (b) break y; }",
    // "for (; x; ) y: for (; y; ) if (a || b) break y;",
    // );
    // test(
    // "x: while (x) y: while (y) { if (a) continue y; if (b) continue y; }",
    // "for (; x; ) y: for (; y; ) if (a || b) continue y;",
    // );
    test("if (x ? y : 0) foo()", "x && y && foo();");
    test("if (x ? y : 1) foo()", "(!x || y) && foo();");
    test("if (x ? 0 : y) foo()", "!x && y && foo();");
    test("if (x ? 1 : y) foo()", "(x || y) && foo();");
    test("if (x ? y : 0) ; else foo()", "x && y || foo();");
    test("if (x ? y : 1) ; else foo()", "!x || y || foo();");
    test("if (x ? 0 : y) ; else foo()", "!x && y || foo();");
    test("if (x ? 1 : y) ; else foo()", "x || y || foo();");
    test("(x ? y : 0) && foo();", "x && y && foo();");
    test("(x ? y : 1) && foo();", "(!x || y) && foo();");
    test("(x ? 0 : y) && foo();", "!x && y && foo();");
    test("(x ? 1 : y) && foo();", "(x || y) && foo();");
    test("(x ? y : 0) || foo();", "x && y || foo();");
    test("(x ? y : 1) || foo();", "!x || y || foo();");
    test("(x ? 0 : y) || foo();", "!x && y || foo();");
    test("(x ? 1 : y) || foo();", "x || y || foo();");
    test("if (!!a || !!b) throw 0", "if (a || b) throw 0;");
    test("if (!!a && !!b) throw 0", "if (a && b) throw 0;");
    test("if (!!a ? !!b : !!c) throw 0", "if (a ? b : c) throw 0;");
    test("if ((a + b) !== 0) throw 0", "if (a + b !== 0) throw 0;");
    test("if ((a | b) !== 0) throw 0", "if (a | b) throw 0;");
    test("if ((a & b) !== 0) throw 0", "if (a & b) throw 0;");
    test("if ((a ^ b) !== 0) throw 0", "if (a ^ b) throw 0;");
    test("if ((a << b) !== 0) throw 0", "if (a << b) throw 0;");
    test("if ((a >> b) !== 0) throw 0", "if (a >> b) throw 0;");
    test("if ((a >>> b) !== 0) throw 0", "if (a >>> b) throw 0;");
    // test("if (+a !== 0) throw 0", "if (+a != 0) throw 0;");
    // test("if (~a !== 0) throw 0", "if (~a) throw 0;");
    test("if (0 != (a + b)) throw 0", "if (a + b != 0) throw 0;");
    test("if (0 != (a | b)) throw 0", "if (a | b) throw 0;");
    test("if (0 != (a & b)) throw 0", "if (a & b) throw 0;");
    test("if (0 != (a ^ b)) throw 0", "if (a ^ b) throw 0;");
    test("if (0 != (a << b)) throw 0", "if (a << b) throw 0;");
    test("if (0 != (a >> b)) throw 0", "if (a >> b) throw 0;");
    test("if (0 != (a >>> b)) throw 0", "if (a >>> b) throw 0;");
    // test("if (0 != +a) throw 0", "if (+a != 0) throw 0;");
    // test("if (0 != ~a) throw 0", "if (~a) throw 0;");
    test("if ((a + b) === 0) throw 0", "if (a + b === 0) throw 0;");
    test("if ((a | b) === 0) throw 0", "if (!(a | b)) throw 0;");
    test("if ((a & b) === 0) throw 0", "if (!(a & b)) throw 0;");
    test("if ((a ^ b) === 0) throw 0", "if (!(a ^ b)) throw 0;");
    test("if ((a << b) === 0) throw 0", "if (!(a << b)) throw 0;");
    test("if ((a >> b) === 0) throw 0", "if (!(a >> b)) throw 0;");
    test("if ((a >>> b) === 0) throw 0", "if (!(a >>> b)) throw 0;");
    // test("if (+a === 0) throw 0", "if (+a == 0) throw 0;");
    // test("if (~a === 0) throw 0", "if (!~a) throw 0;");
    test("if (0 == (a + b)) throw 0", "if (a + b == 0) throw 0;");
    test("if (0 == (a | b)) throw 0", "if (!(a | b)) throw 0;");
    test("if (0 == (a & b)) throw 0", "if (!(a & b)) throw 0;");
    test("if (0 == (a ^ b)) throw 0", "if (!(a ^ b)) throw 0;");
    test("if (0 == (a << b)) throw 0", "if (!(a << b)) throw 0;");
    test("if (0 == (a >> b)) throw 0", "if (!(a >> b)) throw 0;");
    test("if (0 == (a >>> b)) throw 0", "if (!(a >>> b)) throw 0;");
    // test("if (0 == +a) throw 0", "if (+a == 0) throw 0;");
    // test("if (0 == ~a) throw 0", "if (!~a) throw 0;");
    test(
        "function _() { if (a) { if (b) return c } else return d }",
        "function _() { if (a) { if (b) return c;} else return d; }",
    );
    test(
        "function _() { if (a) while (1) { if (b) return c } else return d }",
        "function _() { if (a) { for (;;) if (b) return c;} else return d; }",
    );
    test(
        "function _() { if (a) for (;;) { if (b) return c } else return d }",
        "function _() { if (a) { for (;;) if (b) return c;} else return d; }",
    );
    test(
        "function _() { if (a) for (x in y) { if (b) return c } else return d }",
        "function _() { if (a) { for (x in y) if (b) return c;} else return d; }",
    );
    test(
        "function _() { if (a) for (x of y) { if (b) return c } else return d }",
        "function _() { if (a) { for (x of y) if (b) return c;} else return d; }",
    );
    test(
        "function _() { if (a) with (x) { if (b) return c } else return d }",
        "function _() { if (a) { with (x) if (b) return c;} else return d; }",
    );
    test(
        "function _() { if (a) x: { if (b) break x } else return c }",
        "function _() { if (a) { x: if (b) break x;} else return c; }",
    );
    test(
        "function _() { let a; return a != null ? a.b : undefined }",
        "function _ () { let a;return a?.b; }",
    );
    test(
        "function _() { let a; return a != null ? a[b] : undefined }",
        "function _ () { let a;return a?.[b]; }",
    );
    test(
        "function _() { let a; return a != null ? a(b) : undefined }",
        "function _ () { let a;return a?.(b); }",
    );
    test(
        "function _() { let a; return a == null ? undefined : a.b }",
        "function _ () { let a;return a?.b; }",
    );
    test(
        "function _() { let a; return a == null ? undefined : a[b] }",
        "function _ () { let a;return a?.[b]; }",
    );
    test(
        "function _() { let a; return a == null ? undefined : a(b) }",
        "function _ () { let a;return a?.(b); }",
    );
    test(
        "function _() { let a; return null != a ? a.b : undefined }",
        "function _ () { let a;return a?.b; }",
    );
    test(
        "function _() { let a; return null != a ? a[b] : undefined }",
        "function _ () { let a;return a?.[b]; }",
    );
    test(
        "function _() { let a; return null != a ? a(b) : undefined }",
        "function _ () { let a;return a?.(b); }",
    );
    test(
        "function _() { let a; return null == a ? undefined : a.b }",
        "function _ () { let a;return a?.b; }",
    );
    test(
        "function _() { let a; return null == a ? undefined : a[b] }",
        "function _ () { let a;return a?.[b]; }",
    );
    test(
        "function _() { let a; return null == a ? undefined : a(b) }",
        "function _ () { let a;return a?.(b); }",
    );
    test(
        "function _() { return a != null ? a.b : undefined }",
        "function _ () { return a == null ? void 0 : a.b; }",
    );
    test(
        "function _() { let a; return a != null ? a.b : null }",
        "function _ () { let a;return a == null ? null : a.b; }",
    );
    test(
        "function _() { let a; return a != null ? b.a : undefined }",
        "function _ () { let a;return a == null ? void 0 : b.a; }",
    );
    test(
        "function _() { let a; return a != 0 ? a.b : undefined }",
        "function _ () { let a;return a == 0 ? void 0 : a.b; }",
    );
    test(
        "function _() { let a; return a !== null ? a.b : undefined }",
        "function _ () { let a;return a === null ? void 0 : a.b; }",
    );
    test(
        "function _() { let a; return a != undefined ? a.b : undefined }",
        "function _ () { let a;return a?.b; }",
    );
    test(
        "function _() { let a; return a != null ? a?.b : undefined }",
        "function _ () { let a;return a?.b; }",
    );
    test(
        "function _() { let a; return a != null ? a.b.c[d](e) : undefined }",
        "function _ () { let a;return a?.b.c[d](e); }",
    );
    test(
        "function _() { let a; return a != null ? a?.b.c[d](e) : undefined }",
        "function _ () { let a;return a?.b.c[d](e); }",
    );
    test(
        "function _() { let a; return a != null ? a.b.c?.[d](e) : undefined }",
        "function _ () { let a;return a?.b.c?.[d](e); }",
    );
    test(
        "function _() { let a; return a != null ? a?.b.c?.[d](e) : undefined }",
        "function _ () { let a;return a?.b.c?.[d](e); }",
    );
}

#[test]
#[ignore]
fn test_ignored1() {
    test("a != null && a.b()", "a?.b();");
    test("a == null || a.b()", "a?.b();");
    test("null != a && a.b()", "a?.b();");
    test("null == a || a.b()", "a?.b();");
    test("a == null && a.b()", "a == null && a.b();");
    test("a != null || a.b()", "a != null || a.b();");
    test("null == a && a.b()", "a == null && a.b();");
    test("null != a || a.b()", "a != null || a.b();");
    test("x = a != null && a.b()", "x = a != null && a.b();");
    test("x = a == null || a.b()", "x = a == null || a.b();");
    test("if (a != null) a.b()", "a?.b();");
    test("if (a == null) ; else a.b()", "a?.b();");
    test("if (a == null) a.b()", "a == null && a.b();");
    test("if (a != null) ; else a.b()", "a != null || a.b();");
    test("x(y ?? 1)", "x(y ?? 1);");
    test("x(y.z ?? 1)", "x(y.z ?? 1);");
    test("x(y[z] ?? 1)", "x(y[z] ?? 1);");
    test("x(0 ?? 1)", "x(0);");
    test("x(0n ?? 1)", "x(0n);");
    test("x('' ?? 1)", "x('');");
    test("x(/./ ?? 1)", "x(/./);");
    test("x({} ?? 1)", "x({});");
    test("x((() => {}) ?? 1)", "x(() => {});");
    test("x(class {} ?? 1)", "x(class {});");
    test("x(function() {} ?? 1)", "x(function() {});");
    test("x(null ?? 1)", "x(1);");
    test("x(undefined ?? 1)", "x(1);");
    test("x(void y ?? 1)", "x(void y ?? 1);");
    test("x(-y ?? 1)", "x(-y);");
    test("x(+y ?? 1)", "x(+y);");
    test("x(!y ?? 1)", "x(!y);");
    test("x(~y ?? 1)", "x(~y);");
    test("x(--y ?? 1)", "x(--y);");
    test("x(++y ?? 1)", "x(++y);");
    test("x(y-- ?? 1)", "x(y--);");
    test("x(y++ ?? 1)", "x(y++);");
    test("x(delete y ?? 1)", "x(delete y);");
    test("x(typeof y ?? 1)", "x(typeof y);");
    test("x((y, 0) ?? 1)", "x((y, 0));");
    test("x((y, !z) ?? 1)", "x((y, !z));");
    test("x((y, null) ?? 1)", "x((y, null ?? 1));");
    test("x((y, void z) ?? 1)", "x((y, void z ?? 1));");
    test("x((y + z) ?? 1)", "x(y + z);");
    test("x((y - z) ?? 1)", "x(y - z);");
    test("x((y * z) ?? 1)", "x(y * z);");
    test("x((y / z) ?? 1)", "x(y / z);");
    test("x((y % z) ?? 1)", "x(y % z);");
    test("x((y ** z) ?? 1)", "x(y ** z);");
    test("x((y << z) ?? 1)", "x(y << z);");
    test("x((y >> z) ?? 1)", "x(y >> z);");
    test("x((y >>> z) ?? 1)", "x(y >>> z);");
    test("x((y | z) ?? 1)", "x(y | z);");
    test("x((y & z) ?? 1)", "x(y & z);");
    test("x((y ^ z) ?? 1)", "x(y ^ z);");
    test("x((y < z) ?? 1)", "x(y < z);");
    test("x((y > z) ?? 1)", "x(y > z);");
    test("x((y <= z) ?? 1)", "x(y <= z);");
    test("x((y >= z) ?? 1)", "x(y >= z);");
    test("x((y == z) ?? 1)", "x(y == z);");
    test("x((y != z) ?? 1)", "x(y != z);");
    test("x((y === z) ?? 1)", "x(y === z);");
    test("x((y !== z) ?? 1)", "x(y !== z);");
    test("x((y || z) ?? 1)", "x((y || z) ?? 1);");
    test("x((y && z) ?? 1)", "x((y && z) ?? 1);");
    test("x((y ?? z) ?? 1)", "x(y ?? z ?? 1);");
}

#[test]
#[ignore]
fn test_ignored2() {
    test("y(x && false)", "y(x && false);");
    test("y(x || false)", "y(x || false);");
    test("y(!(x && false))", "y(!(x && false));");
    test("y(!(x || false))", "y(!x);");
    test("if (x && false) y", "x;");
    test("if (x || false) y", "x && y;");
    test("if (x && false) y; else z", "x, z;");
    test("if (x || false) y; else z", "x ? y : z;");
    test("y(x && false ? y : z)", "y((x, z));");
    test("y(x || false ? y : z)", "y(x ? y : z);");
    test("while (false) x()", "for (; false; ) x();");
    test("for (; false; ) x()", "for (; false; ) x();");
    test("y(x && '')", "y(x && '');");
    test("y(x || '')", "y(x || '');");
    test("y(!(x && ''))", "y(!(x && false));");
    test("y(!(x || ''))", "y(!x);");
    test("if (x && '') y", "x;");
    test("if (x || '') y", "x && y;");
    test("if (x && '') y; else z", "x, z;");
    test("if (x || '') y; else z", "x ? y : z;");
    test("y(x && '' ? y : z)", "y((x, z));");
    test("y(x || '' ? y : z)", "y(x ? y : z);");
    test("while ('') x()", "for (; false; ) x();");
    test("for (; ''; ) x()", "for (; false; ) x();");
    test("y(x && 0)", "y(x && 0);");
    test("y(x || 0)", "y(x || 0);");
    test("y(!(x && 0))", "y(!(x && false));");
    test("y(!(x || 0))", "y(!x);");
    test("if (x && 0) y", "x;");
    test("if (x || 0) y", "x && y;");
    test("if (x && 0) y; else z", "x, z;");
    test("if (x || 0) y; else z", "x ? y : z;");
    test("y(x && 0 ? y : z)", "y((x, z));");
    test("y(x || 0 ? y : z)", "y(x ? y : z);");
    test("while (0) x()", "for (; false; ) x();");
    test("for (; 0; ) x()", "for (; false; ) x();");
    test("y(x && 0n)", "y(x && 0n);");
    test("y(x || 0n)", "y(x || 0n);");
    test("y(!(x && 0n))", "y(!(x && false));");
    test("y(!(x || 0n))", "y(!x);");
    test("if (x && 0n) y", "x;");
    test("if (x || 0n) y", "x && y;");
    test("if (x && 0n) y; else z", "x, z;");
    test("if (x || 0n) y; else z", "x ? y : z;");
    test("y(x && 0n ? y : z)", "y((x, z));");
    test("y(x || 0n ? y : z)", "y(x ? y : z);");
    test("while (0n) x()", "for (; false; ) x();");
    test("for (; 0n; ) x()", "for (; false; ) x();");
    test("y(x && null)", "y(x && null);");
    test("y(x || null)", "y(x || null);");
    test("y(!(x && null))", "y(!(x && false));");
    test("y(!(x || null))", "y(!x);");
    test("if (x && null) y", "x;");
    test("if (x || null) y", "x && y;");
    test("if (x && null) y; else z", "x, z;");
    test("if (x || null) y; else z", "x ? y : z;");
    test("y(x && null ? y : z)", "y((x, z));");
    test("y(x || null ? y : z)", "y(x ? y : z);");
    test("while (null) x()", "for (; false; ) x();");
    test("for (; null; ) x()", "for (; false; ) x();");
    test("y(x && void 0)", "y(x && void 0);");
    test("y(x || void 0)", "y(x || void 0);");
    test("y(!(x && void 0))", "y(!(x && false));");
    test("y(!(x || void 0))", "y(!x);");
    test("if (x && void 0) y", "x;");
    test("if (x || void 0) y", "x && y;");
    test("if (x && void 0) y; else z", "x, z;");
    test("if (x || void 0) y; else z", "x ? y : z;");
    test("y(x && void 0 ? y : z)", "y((x, z));");
    test("y(x || void 0 ? y : z)", "y(x ? y : z);");
    test("while (void 0) x()", "for (; false; ) x();");
    test("for (; void 0; ) x()", "for (; false; ) x();");
    test("y(x && true)", "y(x && true);");
    test("y(x || true)", "y(x || true);");
    test("y(!(x && true))", "y(!x);");
    test("y(!(x || true))", "y(!(x || true));");
    test("if (x && true) y", "x && y;");
    test("if (x || true) y", "x, y;");
    test("if (x && true) y; else z", "x ? y : z;");
    test("if (x || true) y; else z", "x, y;");
    test("y(x && true ? y : z)", "y(x ? y : z);");
    test("y(x || true ? y : z)", "y((x, y));");
    test("while (true) x()", "for (;;) x();");
    test("for (; true; ) x()", "for (;;) x();");
    test("y(x && ' ')", "y(x && ' ');");
    test("y(x || ' ')", "y(x || ' ');");
    test("y(!(x && ' '))", "y(!x);");
    test("y(!(x || ' '))", "y(!(x || true));");
    test("if (x && ' ') y", "x && y;");
    test("if (x || ' ') y", "x, y;");
    test("if (x && ' ') y; else z", "x ? y : z;");
    test("if (x || ' ') y; else z", "x, y;");
    test("y(x && ' ' ? y : z)", "y(x ? y : z);");
    test("y(x || ' ' ? y : z)", "y((x, y));");
    test("while (' ') x()", "for (;;) x();");
    test("for (; ' '; ) x()", "for (;;) x();");
    test("y(x && 1)", "y(x && 1);");
    test("y(x || 1)", "y(x || 1);");
    test("y(!(x && 1))", "y(!x);");
    test("y(!(x || 1))", "y(!(x || true));");
    test("if (x && 1) y", "x && y;");
    test("if (x || 1) y", "x, y;");
    test("if (x && 1) y; else z", "x ? y : z;");
    test("if (x || 1) y; else z", "x, y;");
    test("y(x && 1 ? y : z)", "y(x ? y : z);");
    test("y(x || 1 ? y : z)", "y((x, y));");
    test("while (1) x()", "for (;;) x();");
    test("for (; 1; ) x()", "for (;;) x();");
    test("y(x && 1n)", "y(x && 1n);");
    test("y(x || 1n)", "y(x || 1n);");
    test("y(!(x && 1n))", "y(!x);");
    test("y(!(x || 1n))", "y(!(x || true));");
    test("if (x && 1n) y", "x && y;");
    test("if (x || 1n) y", "x, y;");
    test("if (x && 1n) y; else z", "x ? y : z;");
    test("if (x || 1n) y; else z", "x, y;");
    test("y(x && 1n ? y : z)", "y(x ? y : z);");
    test("y(x || 1n ? y : z)", "y((x, y));");
    test("while (1n) x()", "for (;;) x();");
    test("for (; 1n; ) x()", "for (;;) x();");
    test("y(x && /./)", "y(x && /./);");
    test("y(x || /./)", "y(x || /./);");
    test("y(!(x && /./))", "y(!x);");
    test("y(!(x || /./))", "y(!(x || true));");
    test("if (x && /./) y", "x && y;");
    test("if (x || /./) y", "x, y;");
    test("if (x && /./) y; else z", "x ? y : z;");
    test("if (x || /./) y; else z", "x, y;");
    test("y(x && /./ ? y : z)", "y(x ? y : z);");
    test("y(x || /./ ? y : z)", "y((x, y));");
    test("while (/./) x()", "for (;;) x();");
    test("for (; /./; ) x()", "for (;;) x();");
    test("y(x && (() => {}))", "y(x && (() => {}));");
    test("y(x || (() => {}))", "y(x || (() => {}));");
    test("y(!(x && (() => {})))", "y(!x);");
    test("y(!(x || (() => {})))", "y(!(x || true));");
    test("if (x && (() => {})) y", "x && y;");
    test("if (x || (() => {})) y", "x, y;");
    test("if (x && (() => {})) y; else z", "x ? y : z;");
    test("if (x || (() => {})) y; else z", "x, y;");
    test("y(x && (() => {}) ? y : z)", "y(x ? y : z);");
    test("y(x || (() => {}) ? y : z)", "y((x, y));");
    test("while ((() => {})) x()", "for (;;) x();");
    test("for (; (() => {}); ) x()", "for (;;) x();");
    test("y(x && function() {})", "y(x && function() {});");
    test("y(x || function() {})", "y(x || function() {});");
    test("y(!(x && function() {}))", "y(!x);");
    test("y(!(x || function() {}))", "y(!(x || true));");
    test("if (x && function() {}) y", "x && y;");
    test("if (x || function() {}) y", "x, y;");
    test("if (x && function() {}) y; else z", "x ? y : z;");
    test("if (x || function() {}) y; else z", "x, y;");
    test("y(x && function() {} ? y : z)", "y(x ? y : z);");
    test("y(x || function() {} ? y : z)", "y((x, y));");
    test("while (function() {}) x()", "for (;;) x();");
    test("for (; function() {}; ) x()", "for (;;) x();");
    test("y(x && [1, 2])", "y(x && [1, 2]);");
    test("y(x || [1, 2])", "y(x || [1, 2]);");
    test("y(!(x && [1, 2]))", "y(!x);");
    test("y(!(x || [1, 2]))", "y(!(x || true));");
    test("if (x && [1, 2]) y", "x && y;");
    test("if (x || [1, 2]) y", "x, y;");
    test("if (x && [1, 2]) y; else z", "x ? y : z;");
    test("if (x || [1, 2]) y; else z", "x, y;");
    test("y(x && [1, 2] ? y : z)", "y(x ? y : z);");
    test("y(x || [1, 2] ? y : z)", "y((x, y));");
    test("while ([1, 2]) x()", "for (;;) x();");
    test("for (; [1, 2]; ) x()", "for (;;) x();");
    test("y(x && { a: 0 })", "y(x && { a: 0 });");
    test("y(x || { a: 0 })", "y(x || { a: 0 });");
    test("y(!(x && { a: 0 }))", "y(!x);");
    test("y(!(x || { a: 0 }))", "y(!(x || true));");
    test("if (x && { a: 0 }) y", "x && y;");
    test("if (x || { a: 0 }) y", "x, y;");
    test("if (x && { a: 0 }) y; else z", "x ? y : z;");
    test("if (x || { a: 0 }) y; else z", "x, y;");
    test("y(x && { a: 0 } ? y : z)", "y(x ? y : z);");
    test("y(x || { a: 0 } ? y : z)", "y((x, y));");
    test("while ({ a: 0 }) x()", "for (;;) x();");
    test("for (; { a: 0 }; ) x()", "for (;;) x();");
    test("y(x && void foo())", "y(x && void foo());");
    test("y(x || void foo())", "y(x || void foo());");
    test("y(!(x && void foo()))", "y(!(x && void foo()));");
    test("y(!(x || void foo()))", "y(!(x || void foo()));");
    test("if (x || void foo()) y", "(x || void foo()) && y;");
    test("if (x || void foo()) y; else z", "x || void foo() ? y : z;");
    test("y(x || void foo() ? y : z)", "y(x || void foo() ? y : z);");
    test("while (void foo()) x()", "for (; void foo(); ) x();");
    test("for (; void foo(); ) x()", "for (; void foo(); ) x();");
    test("y(x && typeof foo())", "y(x && typeof foo());");
    test("y(x || typeof foo())", "y(x || typeof foo());");
    test("y(!(x || typeof foo()))", "y(!(x || typeof foo()));");
    test("y(!(x && typeof foo()))", "y(!(x && typeof foo()));");
    test("if (x && typeof foo()) y", "x && typeof foo() && y;");
    test("if (x && typeof foo()) y; else z", "x && typeof foo() ? y : z;");
    test("y(x && typeof foo() ? y : z)", "y(x && typeof foo() ? y : z);");
    test("while (typeof foo()) x()", "for (; typeof foo(); ) x();");
    test("for (; typeof foo(); ) x()", "for (; typeof foo(); ) x();");
    test("y(x && [foo()])", "y(x && [foo()]);");
    test("y(x || [foo()])", "y(x || [foo()]);");
    test("y(!(x || [foo()]))", "y(!(x || [foo()]));");
    test("y(!(x && [foo()]))", "y(!(x && [foo()]));");
    test("if (x && [foo()]) y", "x && [foo()] && y;");
    test("if (x && [foo()]) y; else z", "x && [foo()] ? y : z;");
    test("y(x && [foo()] ? y : z)", "y(x && [foo()] ? y : z);");
    test("while ([foo()]) x()", "for (; [foo()]; ) x();");
    test("for (; [foo()]; ) x()", "for (; [foo()]; ) x();");
    test("y(x && { [foo()]: 0 })", "y(x && { [foo()]: 0 });");
    test("y(x || { [foo()]: 0 })", "y(x || { [foo()]: 0 });");
    test("y(!(x || { [foo()]: 0 }))", "y(!(x || { [foo()]: 0 }));");
    test("y(!(x && { [foo()]: 0 }))", "y(!(x && { [foo()]: 0 }));");
    test("if (x && { [foo()]: 0 }) y", "x && { [foo()]: 0 } && y;");
    test("if (x && { [foo()]: 0 }) y; else z", "x && { [foo()]: 0 } ? y : z;");
    test("y(x && { [foo()]: 0 } ? y : z)", "y(x && { [foo()]: 0 } ? y : z);");
    test("while ({ [foo()]: 0 }) x()", "for (; { [foo()]: 0 }; ) x();");
    test("for (; { [foo()]: 0 }; ) x()", "for (; { [foo()]: 0 }; ) x();");
}

#[test]
#[ignore]
fn test_ignored3() {
    test("function foo() { x(); return; }", "function foo() { x();}");
    test("let foo = function() { x(); return; }", "let foo = function() { x();};");
    test("let foo = () => { x(); return; }", "let foo = () => { x();};");
    test("function foo() { x(); return y; }", "function foo() { return x(), y;}");
    test("let foo = function() { x(); return y; }", "let foo = function() { return x(), y;};");
    test("let foo = () => { x(); return y; }", "let foo = () => (x(), y);");
    test("x(); return;", "x();return;");
    test(
        "function foo() { a = b; if (a) return a; if (b) c = b; return c; }",
        "function foo() { return a = b, a || (b && (c = b), c);}",
    );
    test(
        "function foo() { a = b; if (a) return; if (b) c = b; return c; }",
        "function foo() { if (a = b, !a) return b && (c = b), c;}",
    );
    test("function foo() { if (!a) return b; return c; }", "function foo() { return a ? c : b;}");
    test("if (1) return a(); else return b()", "return a();");
    test("if (0) return a(); else return b()", "return b();");
    test("if (a) return b(); else return c()", "return a ? b() : c();");
    test("if (!a) return b(); else return c()", "return a ? c() : b();");
    test("if (!!a) return b(); else return c()", "return a ? b() : c();");
    test("if (!!!a) return b(); else return c()", "return a ? c() : b();");
    test("if (1) return a(); return b()", "return a();");
    test("if (0) return a(); return b()", "return b();");
    test("if (a) return b(); return c()", "return a ? b() : c();");
    test("if (!a) return b(); return c()", "return a ? c() : b();");
    test("if (!!a) return b(); return c()", "return a ? b() : c();");
    test("if (!!!a) return b(); return c()", "return a ? c() : b();");
    test("if (a) return b; else return c; return d;", "return a ? b : c;");
    test("function x() { if (y) return; z(); }", "function x() { y || z();}");
    test("function x() { if (y) return; else z(); w(); }", "function x() { y || (z(), w());}");
    test("function x() { t(); if (y) return; z(); }", "function x() { t(), !y && z();}");
    test(
        "function x() { t(); if (y) return; else z(); w(); }",
        "function x() { t(), !y && (z(), w());}",
    );
    test("function x() { debugger; if (y) return; z(); }", "function x() { debugger; y || z();}");
    test(
        "function x() { debugger; if (y) return; else z(); w(); }",
        "function x() { debugger; y || (z(), w());}",
    );
    test("function x() { if (y) { if (z) return; } }", "function x() { y && z;}");
    test(
        "function x() { if (y) { if (z) return; w(); } }",
        "function x() { if (y) { if (z) return; w(); }}",
    );
    test("function foo(x) { if (!x.y) {} else return x }", "function foo(x) { if (x.y) return x;}");
    test(
        "function foo(x) { if (!x.y) return undefined; return x }",
        "function foo(x) { if (x.y) return x;}",
    );
    test(
        "function x() { if (y) return; function y() {} }",
        "function x() { if (y) return; function y() { }}",
    );
    test("function x() { if (y) return; let y }", "function x() { if (y) return; let y;}");
    test("function x() { if (y) return; var y }", "function x() { if (!y) var y;}");
    test(
        "function foo() { a = b; if (a) throw a; if (b) c = b; throw c; }",
        "function foo() { throw a = b, a || (b && (c = b), c);}",
    );
    test("function foo() { if (!a) throw b; throw c; }", "function foo() { throw a ? c : b;}");
    test("if (1) throw a(); else throw b()", "throw a();");
    test("if (0) throw a(); else throw b()", "throw b();");
    test("if (a) throw b(); else throw c()", "throw a ? b() : c();");
    test("if (!a) throw b(); else throw c()", "throw a ? c() : b();");
    test("if (!!a) throw b(); else throw c()", "throw a ? b() : c();");
    test("if (!!!a) throw b(); else throw c()", "throw a ? c() : b();");
    test("if (1) throw a(); throw b()", "throw a();");
    test("if (0) throw a(); throw b()", "throw b();");
    test("if (a) throw b(); throw c()", "throw a ? b() : c();");
    test("if (!a) throw b(); throw c()", "throw a ? c() : b();");
    test("if (!!a) throw b(); throw c()", "throw a ? b() : c();");
    test("if (!!!a) throw b(); throw c()", "throw a ? c() : b();");
}

#[test]
#[ignore]
fn test_ignored4() {
    test("const a = undefined", "const a = void 0;");
    test("let a = undefined", "let a;");
    test("let {} = undefined", "let {} = void 0;");
    test("let [] = undefined", "let [] = void 0;");
    test("var a = undefined", "var a = void 0;");
    test("var {} = undefined", "var {} = void 0;");
    test("var [] = undefined", "var [] = void 0;");
    test("x = foo(1, ...[], 2)", "x = foo(1, 2);");
    test("x = foo(1, ...2, 3)", "x = foo(1, ...2, 3);");
    test("x = foo(1, ...[2], 3)", "x = foo(1, 2, 3);");
    test("x = foo(1, ...[2, 3], 4)", "x = foo(1, 2, 3, 4);");
    test("x = foo(1, ...[2, ...y, 3], 4)", "x = foo(1, 2, ...y, 3, 4);");
    test("x = foo(1, ...{a, b}, 4)", "x = foo(1, ...{ a, b }, 4);");
    test("x = foo(1, ...[,2,,], 3)", "x = foo(1, void 0, 2, void 0, 3);");
    test("x = new foo(1, ...[], 2)", "x = new foo(1, 2);");
    test("x = new foo(1, ...2, 3)", "x = new foo(1, ...2, 3);");
    test("x = new foo(1, ...[2], 3)", "x = new foo(1, 2, 3);");
    test("x = new foo(1, ...[2, 3], 4)", "x = new foo(1, 2, 3, 4);");
    test("x = new foo(1, ...[2, ...y, 3], 4)", "x = new foo(1, 2, ...y, 3, 4);");
    test("x = new foo(1, ...{a, b}, 4)", "x = new foo(1, ...{ a, b }, 4);");
    test("x = new foo(1, ...[,2,,], 3)", "x = new foo(1, void 0, 2, void 0, 3);");
    test("x = [1, ...[], 2]", "x = [1, 2];");
    test("x = [1, ...2, 3]", "x = [1, ...2, 3];");
    test("x = [1, ...[2], 3]", "x = [1, 2, 3];");
    test("x = [1, ...[2, 3], 4]", "x = [1, 2, 3, 4];");
    test("x = [1, ...[2, ...y, 3], 4]", "x = [1, 2, ...y, 3, 4];");
    test("x = [1, ...{a, b}, 4]", "x = [1, ...{ a, b }, 4];");
    test("x = [1, ...[,2,,], 3]", "x = [1, void 0, 2, void 0, 3];");
    test("x = {['y']: z}", "x = { y: z };");
    test("x = {['y']() {}}", "x = { y() {} };");
    test("x = {get ['y']() {}}", "x = { get y() {} };");
    test("x = {set ['y'](z) {}}", "x = { set y(z) {} };");
    test("x = {async ['y']() {}}", "x = { async y() {} };");
    test("({['y']: z} = x)", "({ y: z } = x);");
    test("x = {a, ...{}, b}", "x = { a, b };");
    test("x = {a, ...b, c}", "x = { a, ...b, c };");
    test("x = {a, ...{b}, c}", "x = { a, b, c };");
    test("x = {a, ...{b() {}}, c}", "x = { a, b() {}, c };");
    test("x = {a, ...{b, c}, d}", "x = { a, b, c, d };");
    test("x = {a, ...{b, ...y, c}, d}", "x = { a, b, ...y, c, d };");
    test("x = {a, ...[b, c], d}", "x = { a, ...[b, c], d };");
    test("x = {a, ...{[b]: c}, d}", "x = { a, [b]: c, d };");
    test("x = {a, ...{[b]() {}}, c}", "x = { a, [b]() {}, c };");
    test(
        "x = {a, ...{b, get c() { return y++ }, d}, e}",
        "x = { a, b, ...{ get c() { return y++;}, d }, e };",
    );
    test(
        "x = {a, ...{b, set c(_) { throw _ }, d}, e}",
        "x = { a, b, ...{ set c(_) { throw _;}, d }, e };",
    );
    test("x = {a, ...{b, __proto__: c, d}, e}", "x = { a, b, ...{ __proto__: c, d }, e };");
    test("x = {a, ...{b, ['__proto__']: c, d}, e}", "x = { a, b, ['__proto__']: c, d, e };");
    test("x = {a, ...{b, __proto__() {}, c}, d}", "x = { a, b, __proto__() {}, c, d };");
    test("x = {a, ...true, b}", "x = { a, b };");
    test("x = {a, ...null, b}", "x = { a, b };");
    test("x = {a, ...void 0, b}", "x = { a, b };");
    test("x = {a, ...123, b}", "x = { a, b };");
    test("x = {a, ...123n, b}", "x = { a, b };");
    test("x = {a, .../x/, b}", "x = { a, b };");
    test("x = {a, ...function(){}, b}", "x = { a, b };");
    test("x = {a, ...()=>{}, b}", "x = { a, b };");
    test("x = {a, ...'123', b}", "x = { a, ...'123', b };");
    test("x = {a, ...[1, 2, 3], b}", "x = { a, ...[1, 2, 3], b };");
    test("x = {a, ...(()=>{})(), b}", "x = { a, .../* @__PURE__ */ (() => {})(), b };");
    test("x = {['y']: z}.y", "x = { y: z }.y;");
    test("x = {['y']: z}.y; var z", "x = z;var z;");
    test("x = {foo: foo(), y: 1}.y", "x = { foo: foo(), y: 1 }.y;");
    test("x = {foo: /* @__PURE__ */ foo(), y: 1}.y", "x = 1;");
    test("x = {__proto__: null}.y", "x = void 0;");
    test("x = {__proto__: null, y: 1}.y", "x = 1;");
    test("x = {__proto__: null}.__proto__", "x = void 0;");
    test("x = {['__proto__']: null}.y", "x = { ['__proto__']: null }.y;");
    test("x = {['__proto__']: null, y: 1}.y", "x = { ['__proto__']: null, y: 1 }.y;");
    test("x = {['__proto__']: null}.__proto__", "x = { ['__proto__']: null }.__proto__;");
    test("x = {y: 1}?.y", "x = 1;");
    test("x = {y: 1}?.['y']", "x = 1;");
    test("x = {y: {z: 1}}?.y.z", "x = 1;");
    test("x = {y: {z: 1}}?.y?.z", "x = { z: 1 }?.z;");
    test("x = {y() {}}?.y()", "x = { y() {} }.y();");
    test("function f(x) { return {x}.x`` }", "function f(x) { return { x }.x``;}");
    test("function f(x) { return (0, {x}.x)`` }", "function f(x) { return x``;}");
    test("var a = () => {}", "var a = () => {};");
    test("var a = () => 123", "var a = () => 123;");
    test("var a = () => void 0", "var a = () => {};");
    test("var a = () => undefined", "var a = () => {};");
    test("var a = () => {return}", "var a = () => {};");
    test("var a = () => {return 123}", "var a = () => 123;");
    test("var a = () => {throw 123}", "var a = () => { throw 123;};");
    test("var a = (() => {})()", "var a = /* @__PURE__ */ (() => {})();");
    test("(() => {})()", "");
    test("(() => a())()", "a();");
    test("(() => { a() })()", "a();");
    test("(() => { return a() })()", "a();");
    test("(() => { let b = a; b() })()", "a();");
    test("(() => { let b = a; return b() })()", "a();");
    test("(async () => {})()", "");
    test("(async () => { a() })()", "(async () => a())();");
    test("(async () => { let b = a; b() })()", "(async () => a())();");
    test("var a = (function() {})()", "var a = /* @__PURE__ */ function() {}();");
    test("(function() {})()", "");
    test("(function*() {})()", "");
    test("(async function() {})()", "");
    test("(function() { a() })()", "(function() { a();})();");
    test("(function*() { a() })()", "(function* () { a();})();");
    test("(async function() { a() })()", "(async function() { a();})();");
    test("(() => x)()", "x;");
    test("/* @__PURE__ */ (() => x)()", "");
    test("/* @__PURE__ */ (() => x)(y, z)", "y, z;");
    test("_ = `a${x}b${y}c`", "_ = `a${x}b${y}c`;");
    test("_ = `a${x}b${'y'}c`", "_ = `a${x}byc`;");
    test("_ = `a${'x'}b${y}c`", "_ = `axb${y}c`;");
    test("_ = `a${'x'}b${'y'}c`", "_ = `axbyc`;");
    test("tag`a${x}b${y}c`", "tag`a${x}b${y}c`;");
    test("tag`a${x}b${'y'}c`", "tag`a${x}b${'y'}c`;");
    test("tag`a${'x'}b${y}c`", "tag`a${'x'}b${y}c`;");
    test("tag`a${'x'}b${'y'}c`", "tag`a${'x'}b${'y'}c`;");
    test("(1, x)``", "x``;");
    test("(1, x.y)``", "(0, x.y)``;");
    test("(1, x[y])``", "(0, x[y])``;");
    test("(true && x)``", "x``;");
    test("(true && x.y)``", "(0, x.y)``;");
    test("(true && x[y])``", "(0, x[y])``;");
    test("(false || x)``", "x``;");
    test("(false || x.y)``", "(0, x.y)``;");
    test("(false || x[y])``", "(0, x[y])``;");
    test("(null ?? x)``", "x``;");
    test("(null ?? x.y)``", "(0, x.y)``;");
    test("(null ?? x[y])``", "(0, x[y])``;");
    test("function f(a) { let c = a.b; return c`` }", "function f(a) { return (0, a.b)``;}");
    test(
        "function f(a) { let c = a.b; return c`${x}` }",
        "function f(a) { return (0, a.b)`${x}`;}",
    );
    test("return typeof (123, x)", "return typeof (0, x);");
    test("return typeof (123, x.y)", "return typeof x.y;");
    test("return typeof (123, x); var x", "return typeof x;var x;");
    test("return typeof (true && x)", "return typeof (0, x);");
    test("return typeof (true && x.y)", "return typeof x.y;");
    test("return typeof (true && x); var x", "return typeof x;var x;");
    test("return typeof (false || x)", "return typeof (0, x);");
    test("return typeof (false || x.y)", "return typeof x.y;");
    test("return typeof (false || x); var x", "return typeof x;var x;");
    test("return typeof x !== 'undefined'", "return typeof x < 'u';");
    test("return typeof x != 'undefined'", "return typeof x < 'u';");
    test("return 'undefined' !== typeof x", "return typeof x < 'u';");
    test("return 'undefined' != typeof x", "return typeof x < 'u';");
    test("return typeof x === 'undefined'", "return typeof x > 'u';");
    test("return typeof x == 'undefined'", "return typeof x > 'u';");
    test("return 'undefined' === typeof x", "return typeof x > 'u';");
    test("return 'undefined' == typeof x", "return typeof x > 'u';");
    test("return typeof x === y", "return typeof x === y;");
    test("return typeof x !== y", "return typeof x !== y;");
    test("return y === typeof x", "return y === typeof x;");
    test("return y !== typeof x", "return y !== typeof x;");
    test("return typeof x === 'string'", "return typeof x == 'string';");
    test("return typeof x !== 'string'", "return typeof x != 'string';");
    test("return 'string' === typeof x", "return typeof x == 'string';");
    test("return 'string' !== typeof x", "return typeof x != 'string';");
    test("return a === 0", "return a === 0;");
    test("return a !== 0", "return a !== 0;");
    test("return +a === 0", "return +a == 0;");
    test("return +a !== 0", "return +a != 0;");
    test("return -a === 0", "return -a === 0;");
    test("return -a !== 0", "return -a !== 0;");
    test("return a === ''", "return a === '';");
    test("return a !== ''", "return a !== '';");
    test("return (a + '!') === 'a!'", "return a + '!' == 'a!';");
    test("return (a + '!') !== 'a!'", "return a + '!' != 'a!';");
    test("return (a += '!') === 'a!'", "return (a += '!') == 'a!';");
    test("return (a += '!') !== 'a!'", "return (a += '!') != 'a!';");
    test("return a === false", "return a === false;");
    test("return a === true", "return a === true;");
    test("return a !== false", "return a !== false;");
    test("return a !== true", "return a !== true;");
    test("return !a === false", "return !!a;");
    test("return !a === true", "return !a;");
    test("return !a !== false", "return !a;");
    test("return !a !== true", "return !!a;");
    test("return false === !a", "return !!a;");
    test("return true === !a", "return !a;");
    test("return false !== !a", "return !a;");
    test("return true !== !a", "return !!a;");
    test("return a === !b", "return a === !b;");
    test("return a === !b", "return a === !b;");
    test("return a !== !b", "return a !== !b;");
    test("return a !== !b", "return a !== !b;");
    test("return !a === !b", "return !a == !b;");
    test("return !a === !b", "return !a == !b;");
    test("return !a !== !b", "return !a != !b;");
    test("return !a !== !b", "return !a != !b;");
    test("return (a, -1n) !== -1", "return a, -1n !== -1;");
    test("return (a, ~1n) !== -1", "return a, ~1n !== -1;");
    test("return (a -= 1n) !== -1", "return (a -= 1n) !== -1;");
    test("return (a *= 1n) !== -1", "return (a *= 1n) !== -1;");
    test("return (a **= 1n) !== -1", "return (a **= 1n) !== -1;");
    test("return (a /= 1n) !== -1", "return (a /= 1n) !== -1;");
    test("return (a %= 1n) !== -1", "return (a %= 1n) !== -1;");
    test("return (a &= 1n) !== -1", "return (a &= 1n) !== -1;");
    test("return (a |= 1n) !== -1", "return (a |= 1n) !== -1;");
    test("return (a ^= 1n) !== -1", "return (a ^= 1n) !== -1;");
    test("return -(a, b)", "return a, -b;");
    test("return +(a, b)", "return a, +b;");
    test("return ~(a, b)", "return a, ~b;");
    test("return !(a, b)", "return a, !b;");
    test("return void (a, b)", "return a, void b;");
    test("return typeof (a, b)", "return typeof (a, b);");
    test("return delete (a, b)", "return delete (a, b);");
    test("(a, b) && c", "a, b && c;");
    test("(a, b) == c", "a, b == c;");
    test("(a, b) + c", "a, b + c;");
    test("a && (b, c)", "a && (b, c);");
    test("a == (b, c)", "a == (b, c);");
    test("a + (b, c)", "a + (b, c);");
    test("x = +5", "x = 5;");
    test("x = -5", "x = -5;");
    test("x = ~5", "x = -6;");
    test("x = !5", "x = false;");
    test("x = typeof 5", "x = 'number';");
    test("x = +''", "x = 0;");
    test("x = +[]", "x = 0;");
    test("x = +{}", "x = NaN;");
    test("x = +/1/", "x = NaN;");
    test("x = +[1]", "x = +[1];");
    test("x = +'123'", "x = 123;");
    test("x = +'-123'", "x = -123;");
    test("x = +'0x10'", "x = +'0x10';");
    test("x = +{toString:()=>1}", "x = +{ toString: () => 1 };");
    test("x = +{valueOf:()=>1}", "x = +{ valueOf: () => 1 };");
    test("x = 3 + 6", "x = 9;");
    test("x = 3 - 6", "x = -3;");
    test("x = 3 * 6", "x = 3 * 6;");
    test("x = 3 / 6", "x = 3 / 6;");
    test("x = 3 % 6", "x = 3 % 6;");
    test("x = 3 ** 6", "x = 3 ** 6;");
    test("x = 0 / 0", "x = NaN;");
    test("x = 123 / 0", "x = Infinity;");
    test("x = 123 / -0", "x = -Infinity;");
    test("x = -123 / 0", "x = -Infinity;");
    test("x = -123 / -0", "x = Infinity;");
    test("x = 3 < 6", "x = true;");
    test("x = 3 > 6", "x = false;");
    test("x = 3 <= 6", "x = true;");
    test("x = 3 >= 6", "x = false;");
    test("x = 3 == 6", "x = false;");
    test("x = 3 != 6", "x = true;");
    test("x = 3 === 6", "x = false;");
    test("x = 3 !== 6", "x = true;");
    test("x = 'a' < 'b'", "x = true;");
    test("x = 'a' > 'b'", "x = false;");
    test("x = 'a' <= 'b'", "x = true;");
    test("x = 'a' >= 'b'", "x = false;");
    test("x = 'ab' < 'abc'", "x = true;");
    test("x = 'ab' > 'abc'", "x = false;");
    test("x = 'ab' <= 'abc'", "x = true;");
    test("x = 'ab' >= 'abc'", "x = false;");
    test("x = '𐙩' < 'ﬡ'", "x = true;");
    test("x = '𐙩' > 'ﬡ'", "x = false;");
    test("x = '𐙩' <= 'ﬡ'", "x = true;");
    test("x = '𐙩' >= 'ﬡ'", "x = false;");
    test("x = 3 in 6", "x = 3 in 6;");
    test("x = 3 instanceof 6", "x = 3 instanceof 6;");
    test("x = (3, 6)", "x = 6;");
    test("x = 10 << 0", "x = 10;");
    test("x = 10 << 1", "x = 20;");
    test("x = 10 << 16", "x = 655360;");
    test("x = 10 << 17", "x = 10 << 17;");
    test("x = 10 >> 0", "x = 10;");
    test("x = 10 >> 1", "x = 5;");
    test("x = 10 >>> 0", "x = 10;");
    test("x = 10 >>> 1", "x = 5;");
    test("x = -10 >>> 1", "x = -10 >>> 1;");
    test("x = -1 >>> 0", "x = -1 >>> 0;");
    test("x = -123 >>> 5", "x = -123 >>> 5;");
    test("x = -123 >>> 6", "x = 67108862;");
    test("x = 3 & 6", "x = 2;");
    test("x = 3 | 6", "x = 7;");
    test("x = 3 ^ 6", "x = 5;");
    test("x = 3 && 6", "x = 6;");
    test("x = 3 || 6", "x = 3;");
    test("x = 3 ?? 6", "x = 3;");
    test("(a && b) && c", "a && b && c;");
    test("a && (b && c)", "a && b && c;");
    test("(a || b) && c", "(a || b) && c;");
    test("a && (b || c)", "a && (b || c);");
    test("(a || b) || c", "a || b || c;");
    test("a || (b || c)", "a || b || c;");
    test("(a && b) || c", "a && b || c;");
    test("a || (b && c)", "a || b && c;");
    test("return a === void 0", "return a === void 0;");
    test("return a !== void 0", "return a !== void 0;");
    test("return void 0 === a", "return a === void 0;");
    test("return void 0 !== a", "return a !== void 0;");
    test("return a == void 0", "return a == null;");
    test("return a != void 0", "return a != null;");
    test("return void 0 == a", "return a == null;");
    test("return void 0 != a", "return a != null;");
    test("return a === null || a === undefined", "return a == null;");
    test("return a === null || a !== undefined", "return a === null || a !== void 0;");
    test("return a !== null || a === undefined", "return a !== null || a === void 0;");
    test("return a === null && a === undefined", "return a === null && a === void 0;");
    test("return a.x === null || a.x === undefined", "return a.x === null || a.x === void 0;");
    test("return a === undefined || a === null", "return a == null;");
    test("return a === undefined || a !== null", "return a === void 0 || a !== null;");
    test("return a !== undefined || a === null", "return a !== void 0 || a === null;");
    test("return a === undefined && a === null", "return a === void 0 && a === null;");
    test("return a.x === undefined || a.x === null", "return a.x === void 0 || a.x === null;");
    test("return a !== null && a !== undefined", "return a != null;");
    test("return a !== null && a === undefined", "return a !== null && a === void 0;");
    test("return a === null && a !== undefined", "return a === null && a !== void 0;");
    test("return a !== null || a !== undefined", "return a !== null || a !== void 0;");
    test("return a.x !== null && a.x !== undefined", "return a.x !== null && a.x !== void 0;");
    test("return a !== undefined && a !== null", "return a != null;");
    test("return a !== undefined && a === null", "return a !== void 0 && a === null;");
    test("return a === undefined && a !== null", "return a === void 0 && a !== null;");
    test("return a !== undefined || a !== null", "return a !== void 0 || a !== null;");
    test("return a.x !== undefined && a.x !== null", "return a.x !== void 0 && a.x !== null;");
    test("x = function y() {}", "x = function() {};");
    test("x = function y() { return y }", "x = function y() { return y;};");
    test("x = function y() { return eval('y') }", "x = function y() { return eval('y');};");
    test("x = function y() { if (0) return y }", "x = function() {};");
    test("class x {['y'] = z}", "class x { y = z;}");
    test("class x {['y']() {}}", "class x { y() { }}");
    test("class x {get ['y']() {}}", "class x { get y() { }}");
    test("class x {set ['y'](z) {}}", "class x { set y(z) { }}");
    test("class x {async ['y']() {}}", "class x { async y() { }}");
    test("x = class {['y'] = z}", "x = class { y = z;};");
    test("x = class {['y']() {}}", "x = class { y() { }};");
    test("x = class {get ['y']() {}}", "x = class { get y() { }};");
    test("x = class {set ['y'](z) {}}", "x = class { set y(z) { }};");
    test("x = class {async ['y']() {}}", "x = class { async y() { }};");
    test("x = class y {}", "x = class {};");
    test("x = class y { foo() { return y } }", "x = class y { foo() { return y; }};");
    test("x = class y { foo() { if (0) return y } }", "x = class { foo() { }};");
    test("null", "");
    test("void 0", "");
    test("void 0", "");
    test("false", "");
    test("true", "");
    test("123", "");
    test("123n", "");
    test("'abc'", "'abc';");
    test("0; 'abc'", "");
    test("'abc'; 'use strict'", "'abc';'use strict';");
    test("function f() { 'abc'; 'use strict' }", "function f() { 'abc'; 'use strict';}");
    test("this", "");
    test("/regex/", "");
    test("(function() {})", "");
    test("(() => {})", "");
    test("import.meta", "");
    test("+x", "+x;");
    test("-x", "-x;");
    test("!x", "x;");
    test("~x", "~x;");
    test("++x", "++x;");
    test("--x", "--x;");
    test("x++", "x++;");
    test("x--", "x--;");
    test("void x", "x;");
    test("delete x", "delete x;");
    test("typeof x", "");
    test("typeof x()", "x();");
    test("typeof (0, x)", "x;");
    test("typeof (0 || x)", "x;");
    test("typeof (1 && x)", "x;");
    test("typeof (1 ? x : 0)", "x;");
    test("typeof (0 ? 1 : x)", "x;");
    test("a + b", "a + b;");
    test("a - b", "a - b;");
    test("a * b", "a * b;");
    test("a / b", "a / b;");
    test("a % b", "a % b;");
    test("a ** b", "a ** b;");
    test("a & b", "a & b;");
    test("a | b", "a | b;");
    test("a ^ b", "a ^ b;");
    test("a << b", "a << b;");
    test("a >> b", "a >> b;");
    test("a >>> b", "a >>> b;");
    test("a === b", "a, b;");
    test("a !== b", "a, b;");
    test("a == b", "a == b;");
    test("a != b", "a != b;");
    test("a, b", "a, b;");
    test("a + '' == b", "a + '' == b;");
    test("a + '' != b", "a + '' != b;");
    test("a + '' == b + ''", "a + '', b + '';");
    test("a + '' != b + ''", "a + '', b + '';");
    test("a + '' == (b | c)", "a + '', b | c;");
    test("a + '' != (b | c)", "a + '', b | c;");
    test("typeof a == b + ''", "b + '';");
    test("typeof a != b + ''", "b + '';");
    test("typeof a == 'b'", "");
    test("typeof a != 'b'", "");
    test("Object", "");
    test("Object()", "Object();");
    test("NonObject", "NonObject;");
    test("var bound; unbound", "var bound;unbound;");
    test("var bound; bound", "var bound;");
    test("foo, 123, bar", "foo, bar;");
    test("[[foo,, 123,, bar]]", "foo, bar;");
    test("var bound; [123, unbound, ...unbound, 234]", "var bound;[unbound, ...unbound];");
    test("var bound; [123, bound, ...bound, 234]", "var bound;[...bound];");
    test("({foo, x: 123, [y]: 123, z: z, bar})", "foo, y + '', z, bar;");
    test(
        "var bound; ({x: 123, unbound, ...unbound, [unbound]: null, y: 234})",
        "var bound;({ unbound, ...unbound, [unbound]: 0 });",
    );
    test(
        "var bound; ({x: 123, bound, ...bound, [bound]: null, y: 234})",
        "var bound;({ ...bound, [bound]: 0 });",
    );
    test(
        "var bound; ({x: 123, bound, ...bound, [bound]: foo(), y: 234})",
        "var bound;({ ...bound, [bound]: foo() });",
    );
    test("console.log(1, foo(), bar())", "console.log(1, foo(), bar());");
    test("/* @__PURE__ */ console.log(1, foo(), bar())", "foo(), bar();");
    test("new TestCase(1, foo(), bar())", "new TestCase(1, foo(), bar());");
    test("/* @__PURE__ */ new TestCase(1, foo(), bar())", "foo(), bar();");
    test("let x = (1, 2)", "let x = 2;");
    test("let x = (y, 2)", "let x = (y, 2);");
    test("let x = (/* @__PURE__ */ foo(bar), 2)", "let x = (bar, 2);");
    test("let x = (2, y)", "let x = y;");
    test("let x = (2, y)()", "let x = y();");
    test("let x = (true && y)()", "let x = y();");
    test("let x = (false || y)()", "let x = y();");
    test("let x = (null ?? y)()", "let x = y();");
    test("let x = (1 ? y : 2)()", "let x = y();");
    test("let x = (0 ? 1 : y)()", "let x = y();");
    test("let x = (2, y.z)", "let x = y.z;");
    test("let x = (2, y.z)()", "let x = (0, y.z)();");
    test("let x = (true && y.z)()", "let x = (0, y.z)();");
    test("let x = (false || y.z)()", "let x = (0, y.z)();");
    test("let x = (null ?? y.z)()", "let x = (0, y.z)();");
    test("let x = (1 ? y.z : 2)()", "let x = (0, y.z)();");
    test("let x = (0 ? 1 : y.z)()", "let x = (0, y.z)();");
    test("let x = (2, y[z])", "let x = y[z];");
    test("let x = (2, y[z])()", "let x = (0, y[z])();");
    test("let x = (true && y[z])()", "let x = (0, y[z])();");
    test("let x = (false || y[z])()", "let x = (0, y[z])();");
    test("let x = (null ?? y[z])()", "let x = (0, y[z])();");
    test("let x = (1 ? y[z] : 2)()", "let x = (0, y[z])();");
    test("let x = (0 ? 1 : y[z])()", "let x = (0, y[z])();");
    test("delete (x)", "delete x;");
    test("delete (x); var x", "delete x;var x;");
    test("delete (x.y)", "delete x.y;");
    test("delete (x[y])", "delete x[y];");
    test("delete (x?.y)", "delete x?.y;");
    test("delete (x?.[y])", "delete x?.[y];");
    test("delete (2, x)", "delete (0, x);");
    test("delete (2, x); var x", "delete (0, x);var x;");
    test("delete (2, x.y)", "delete (0, x.y);");
    test("delete (2, x[y])", "delete (0, x[y]);");
    test("delete (2, x?.y)", "delete (0, x?.y);");
    test("delete (2, x?.[y])", "delete (0, x?.[y]);");
    test("delete (true && x)", "delete (0, x);");
    test("delete (false || x)", "delete (0, x);");
    test("delete (null ?? x)", "delete (0, x);");
    test("delete (1 ? x : 2)", "delete (0, x);");
    test("delete (0 ? 1 : x)", "delete (0, x);");
    test("delete (NaN)", "delete NaN;");
    test("delete (Infinity)", "delete Infinity;");
    test("delete (-Infinity)", "delete -Infinity;");
    test("delete (1, NaN)", "delete (0, NaN);");
    test("delete (1, Infinity)", "delete (0, Infinity);");
    test("delete (1, -Infinity)", "delete -Infinity;");
    test("foo ? 1 : 2", "foo;");
    test("foo ? 1 : bar", "foo || bar;");
    test("foo ? bar : 2", "foo && bar;");
    test("foo ? bar : baz", "foo ? bar : baz;");
    test("foo && bar", "foo && bar;");
    test("var foo; foo && bar", "var foo;foo && bar;");
    test("var bar; foo && bar", "var bar;foo;");
    test("var foo, bar; foo && bar", "var foo, bar;");
    test("foo || bar", "foo || bar;");
    test("var foo; foo || bar", "var foo;foo || bar;");
    test("var bar; foo || bar", "var bar;foo;");
    test("var foo, bar; foo || bar", "var foo, bar;");
    test("foo ?? bar", "foo ?? bar;");
    test("var foo; foo ?? bar", "var foo;foo ?? bar;");
    test("var bar; foo ?? bar", "var bar;foo;");
    test("var foo, bar; foo ?? bar", "var foo, bar;");
    test("tag`a${b}c${d}e`", "tag`a${b}c${d}e`;");
    test("`a${b}c${d}e`", "`${b}${d}`;");
    test("`stuff ${x} ${1}`", "`${x}`;");
    test("`stuff ${1} ${y}`", "`${y}`;");
    test("`stuff ${x} ${y}`", "`${x}${y}`;");
    test("`stuff ${x ? 1 : 2} ${y}`", "x, `${y}`;");
    test("`stuff ${x} ${y ? 1 : 2}`", "`${x}`, y;");
    test("`stuff ${x} ${y ? 1 : 2} ${z}`", "`${x}`, y, `${z}`;");
    test("'a' + b + 'c' + d", "'' + b + d;");
    test("a + 'b' + c + 'd'", "a + '' + c;");
    test("a + b + 'c' + 'd'", "a + b + '';");
    test("'a' + 'b' + c + d", "'' + c + d;");
    test("(a + '') + (b + '')", "a + (b + '');");
    test("with (a) []", "with (a) ;");
    test("var a; with (b) a", "var a;with (b) a;");
    test(
        "function wrapper(arg0, arg1) {var x = 1; return x}",
        "function wrapper(arg0, arg1) { var x = 1; return x;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = 1; return x}",
        "function wrapper(arg0, arg1) { return 1;}",
    );
    test(
        "function wrapper(arg0, arg1) {const x = 1; return x}",
        "function wrapper(arg0, arg1) { return 1;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = 1; if (false) x++; return x}",
        "function wrapper(arg0, arg1) { return 1;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = 1; if (true) x++; return x}",
        "function wrapper(arg0, arg1) { let x = 1; return x++, x;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = 1; return x + x}",
        "function wrapper(arg0, arg1) { let x = 1; return x + x;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = 1; return +x}",
        "function wrapper(arg0, arg1) { return +1;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = 1; return -x}",
        "function wrapper(arg0, arg1) { return -1;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = 1; return !x}",
        "function wrapper(arg0, arg1) { return !1;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = 1; return ~x}",
        "function wrapper(arg0, arg1) { return ~1;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = 1; return void x}",
        "function wrapper(arg0, arg1) { let x = 1;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = 1; return typeof x}",
        "function wrapper(arg0, arg1) { return typeof 1;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = 1; return `<${x}>`}",
        "function wrapper(arg0, arg1) { return `<1>`;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = 1n; return `<${x}>`}",
        "function wrapper(arg0, arg1) { return `<1>`;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = null; return `<${x}>`}",
        "function wrapper(arg0, arg1) { return `<null>`;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = undefined; return `<${x}>`}",
        "function wrapper(arg0, arg1) { return `<undefined>`;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = false; return `<${x}>`}",
        "function wrapper(arg0, arg1) { return `<false>`;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = true; return `<${x}>`}",
        "function wrapper(arg0, arg1) { return `<true>`;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = 1; return x + 2}",
        "function wrapper(arg0, arg1) { return 1 + 2;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = 1; return 2 + x}",
        "function wrapper(arg0, arg1) { return 2 + 1;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = 1; return x + arg0}",
        "function wrapper(arg0, arg1) { return 1 + arg0;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = 1; return arg0 + x}",
        "function wrapper(arg0, arg1) { return arg0 + 1;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = 1; return x + fn()}",
        "function wrapper(arg0, arg1) { return 1 + fn();}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = 1; return fn() + x}",
        "function wrapper(arg0, arg1) { return fn() + 1;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = 1; return x + undef}",
        "function wrapper(arg0, arg1) { return 1 + undef;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = 1; return undef + x}",
        "function wrapper(arg0, arg1) { return undef + 1;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = fn(); return x + 2}",
        "function wrapper(arg0, arg1) { return fn() + 2;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = fn(); return 2 + x}",
        "function wrapper(arg0, arg1) { return 2 + fn();}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = fn(); return x + arg0}",
        "function wrapper(arg0, arg1) { return fn() + arg0;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = fn(); return arg0 + x}",
        "function wrapper(arg0, arg1) { let x = fn(); return arg0 + x;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = fn(); return x + fn2()}",
        "function wrapper(arg0, arg1) { return fn() + fn2();}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = fn(); return fn2() + x}",
        "function wrapper(arg0, arg1) { let x = fn(); return fn2() + x;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = fn(); return x + undef}",
        "function wrapper(arg0, arg1) { return fn() + undef;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = fn(); return undef + x}",
        "function wrapper(arg0, arg1) { let x = fn(); return undef + x;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = 1; ++x}",
        "function wrapper(arg0, arg1) { let x = 1; ++x;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = 1; --x}",
        "function wrapper(arg0, arg1) { let x = 1; --x;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = 1; x++}",
        "function wrapper(arg0, arg1) { let x = 1; x++;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = 1; x--}",
        "function wrapper(arg0, arg1) { let x = 1; x--;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = 1; delete x}",
        "function wrapper(arg0, arg1) { let x = 1; delete x;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = 1; x = 2}",
        "function wrapper(arg0, arg1) { let x = 1; x = 2;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = 1; x += 2}",
        "function wrapper(arg0, arg1) { let x = 1; x += 2;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = 1; x ||= 2}",
        "function wrapper(arg0, arg1) { let x = 1; x ||= 2;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = 1; arg0 = x}",
        "function wrapper(arg0, arg1) { arg0 = 1;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = 1; arg0 += x}",
        "function wrapper(arg0, arg1) { arg0 += 1;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = 1; arg0 ||= x}",
        "function wrapper(arg0, arg1) { arg0 ||= 1;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = fn(); arg0 = x}",
        "function wrapper(arg0, arg1) { arg0 = fn();}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = fn(); arg0 += x}",
        "function wrapper(arg0, arg1) { let x = fn(); arg0 += x;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = fn(); arg0 ||= x}",
        "function wrapper(arg0, arg1) { let x = fn(); arg0 ||= x;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = 1; y.z = x}",
        "function wrapper(arg0, arg1) { let x = 1; y.z = x;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = 1; y.z += x}",
        "function wrapper(arg0, arg1) { let x = 1; y.z += x;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = 1; y.z ||= x}",
        "function wrapper(arg0, arg1) { let x = 1; y.z ||= x;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = fn(); y.z = x}",
        "function wrapper(arg0, arg1) { let x = fn(); y.z = x;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = fn(); y.z += x}",
        "function wrapper(arg0, arg1) { let x = fn(); y.z += x;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = fn(); y.z ||= x}",
        "function wrapper(arg0, arg1) { let x = fn(); y.z ||= x;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = arg0; return x ? y : z;}",
        "function wrapper(arg0, arg1) { return arg0 ? y : z;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = arg0; return arg1 ? x : y;}",
        "function wrapper(arg0, arg1) { return arg1 ? arg0 : y;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = arg0; return arg1 ? y : x;}",
        "function wrapper(arg0, arg1) { return arg1 ? y : arg0;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = arg0; return x || y;}",
        "function wrapper(arg0, arg1) { return arg0 || y;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = arg0; return x && y;}",
        "function wrapper(arg0, arg1) { return arg0 && y;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = arg0; return x ?? y;}",
        "function wrapper(arg0, arg1) { return arg0 ?? y;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = arg0; return arg1 || x;}",
        "function wrapper(arg0, arg1) { return arg1 || arg0;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = arg0; return arg1 && x;}",
        "function wrapper(arg0, arg1) { return arg1 && arg0;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = arg0; return arg1 ?? x;}",
        "function wrapper(arg0, arg1) { return arg1 ?? arg0;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = arg0; return y ? x : z;}",
        "function wrapper(arg0, arg1) { let x = arg0; return y ? x : z;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = arg0; return y ? z : x;}",
        "function wrapper(arg0, arg1) { let x = arg0; return y ? z : x;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = arg0; return (arg1 ? 1 : 2) ? x : 3;}",
        "function wrapper(arg0, arg1) { return arg0;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = arg0; return (arg1 ? 1 : 2) ? 3 : x;}",
        "function wrapper(arg0, arg1) { let x = arg0; return 3;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = arg0; return (arg1 ? y : 1) ? x : 2;}",
        "function wrapper(arg0, arg1) { let x = arg0; return !arg1 || y ? x : 2;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = arg0; return (arg1 ? 1 : y) ? x : 2;}",
        "function wrapper(arg0, arg1) { let x = arg0; return arg1 || y ? x : 2;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = arg0; return (arg1 ? y : 1) ? 2 : x;}",
        "function wrapper(arg0, arg1) { let x = arg0; return !arg1 || y ? 2 : x;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = arg0; return (arg1 ? 1 : y) ? 2 : x;}",
        "function wrapper(arg0, arg1) { let x = arg0; return arg1 || y ? 2 : x;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = arg0; return y || x;}",
        "function wrapper(arg0, arg1) { let x = arg0; return y || x;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = arg0; return y && x;}",
        "function wrapper(arg0, arg1) { let x = arg0; return y && x;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = arg0; return y ?? x;}",
        "function wrapper(arg0, arg1) { let x = arg0; return y ?? x;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = fn(); return x ? arg0 : y;}",
        "function wrapper(arg0, arg1) { return fn() ? arg0 : y;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = fn(); return arg0 ? x : y;}",
        "function wrapper(arg0, arg1) { let x = fn(); return arg0 ? x : y;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = fn(); return arg0 ? y : x;}",
        "function wrapper(arg0, arg1) { let x = fn(); return arg0 ? y : x;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = fn(); return x || arg0;}",
        "function wrapper(arg0, arg1) { return fn() || arg0;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = fn(); return x && arg0;}",
        "function wrapper(arg0, arg1) { return fn() && arg0;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = fn(); return x ?? arg0;}",
        "function wrapper(arg0, arg1) { return fn() ?? arg0;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = fn(); return arg0 || x;}",
        "function wrapper(arg0, arg1) { let x = fn(); return arg0 || x;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = fn(); return arg0 && x;}",
        "function wrapper(arg0, arg1) { let x = fn(); return arg0 && x;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = fn(); return arg0 ?? x;}",
        "function wrapper(arg0, arg1) { let x = fn(); return arg0 ?? x;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = fn(); let y = x[prop]; let z = y.val; throw z}",
        "function wrapper(arg0, arg1) { throw fn()[prop].val;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = fn(), y = x[prop], z = y.val; throw z}",
        "function wrapper(arg0, arg1) { throw fn()[prop].val;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = 0; let y = ++x; return y}",
        "function wrapper(arg0, arg1) { let x = 0; return ++x;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = 0; let y = x; return [x, y]}",
        "function wrapper(arg0, arg1) { let x = 0; return [x, x];}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = 0; let y = ++x; return [x, y]}",
        "function wrapper(arg0, arg1) { let x = 0, y = ++x; return [x, y];}",
    );
    test("function wrapper(arg0, arg1) { let x = 0; let y = {valueOf() { x = 1 }}; let z = x; return [y == 1, z]}", "function wrapper(arg0, arg1) { let x = 0, y = { valueOf() { x = 1; } }, z = x; return [y == 1, z];}");
    test(
        "function wrapper(arg0, arg1) { let x = arg0; return [...x];}",
        "function wrapper(arg0, arg1) { return [...arg0];}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = arg0; return [x, ...arg1];}",
        "function wrapper(arg0, arg1) { return [arg0, ...arg1];}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = arg0; return [...arg1, x];}",
        "function wrapper(arg0, arg1) { let x = arg0; return [...arg1, x];}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = arg0; return arg1(...x);}",
        "function wrapper(arg0, arg1) { return arg1(...arg0);}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = arg0; return arg1(x, ...arg1);}",
        "function wrapper(arg0, arg1) { return arg1(arg0, ...arg1);}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = arg0; return arg1(...arg1, x);}",
        "function wrapper(arg0, arg1) { let x = arg0; return arg1(...arg1, x);}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = arg0; arg1(x);}",
        "function wrapper(arg0, arg1) { arg1(arg0);}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = arg0; throw x;}",
        "function wrapper(arg0, arg1) { throw arg0;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = arg0; return x;}",
        "function wrapper(arg0, arg1) { return arg0;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = arg0; if (x) return 1;}",
        "function wrapper(arg0, arg1) { if (arg0) return 1;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = arg0; switch (x) { case 0: return 1; }}",
        "function wrapper(arg0, arg1) { switch (arg0) { case 0:  return 1; }}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = arg0; let y = x; return y + y;}",
        "function wrapper(arg0, arg1) { let y = arg0; return y + y;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = arg0; do {} while (x);}",
        "function wrapper(arg0, arg1) { let x = arg0; do ; while (x);}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = arg0; while (x) return 1;}",
        "function wrapper(arg0, arg1) { let x = arg0; for (; x; ) return 1;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = arg0; for (; x; ) return 1;}",
        "function wrapper(arg0, arg1) { let x = arg0; for (; x; ) return 1;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = arg0; return arg1?.[x];}",
        "function wrapper(arg0, arg1) { return arg1?.[arg0];}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = arg0; return arg1?.(x);}",
        "function wrapper(arg0, arg1) { return arg1?.(arg0);}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = fn(); return arg1?.[x];}",
        "function wrapper(arg0, arg1) { let x = fn(); return arg1?.[x];}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = fn(); return arg1?.(x);}",
        "function wrapper(arg0, arg1) { let x = fn(); return arg1?.(x);}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = arg0; return arg1?.a === x;}",
        "function wrapper(arg0, arg1) { let x = arg0; return arg1?.a === x;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = arg0; return arg1?.[0] === x;}",
        "function wrapper(arg0, arg1) { let x = arg0; return arg1?.[0] === x;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = arg0; return arg1?.(0) === x;}",
        "function wrapper(arg0, arg1) { let x = arg0; return arg1?.(0) === x;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = arg0; return arg1?.a[x];}",
        "function wrapper(arg0, arg1) { let x = arg0; return arg1?.a[x];}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = arg0; return arg1?.a(x);}",
        "function wrapper(arg0, arg1) { let x = arg0; return arg1?.a(x);}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = arg0; return arg1?.[a][x];}",
        "function wrapper(arg0, arg1) { let x = arg0; return arg1?.[a][x];}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = arg0; return arg1?.[a](x);}",
        "function wrapper(arg0, arg1) { let x = arg0; return arg1?.[a](x);}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = arg0; return arg1?.(a)[x];}",
        "function wrapper(arg0, arg1) { let x = arg0; return arg1?.(a)[x];}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = arg0; return arg1?.(a)(x);}",
        "function wrapper(arg0, arg1) { let x = arg0; return arg1?.(a)(x);}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = arg0; return {x};}",
        "function wrapper(arg0, arg1) { return { x: arg0 };}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = arg0; return {x: y, y: x};}",
        "function wrapper(arg0, arg1) { let x = arg0; return { x: y, y: x };}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = arg0; return {x: arg1, y: x};}",
        "function wrapper(arg0, arg1) { return { x: arg1, y: arg0 };}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = arg0; return {[x]: 0};}",
        "function wrapper(arg0, arg1) { return { [arg0]: 0 };}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = arg0; return {[y]: x};}",
        "function wrapper(arg0, arg1) { let x = arg0; return { [y]: x };}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = arg0; return {[arg1]: x};}",
        "function wrapper(arg0, arg1) { let x = arg0; return { [arg1]: x };}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = arg0; return {y() {}, x};}",
        "function wrapper(arg0, arg1) { return { y() { }, x: arg0 };}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = arg0; return {[y]() {}, x};}",
        "function wrapper(arg0, arg1) { let x = arg0; return { [y]() { }, x };}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = arg0; return {...x};}",
        "function wrapper(arg0, arg1) { return { ...arg0 };}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = arg0; return {...x, y};}",
        "function wrapper(arg0, arg1) { return { ...arg0, y };}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = arg0; return {x, ...y};}",
        "function wrapper(arg0, arg1) { return { x: arg0, ...y };}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = arg0; return {...y, x};}",
        "function wrapper(arg0, arg1) { let x = arg0; return { ...y, x };}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = arg0; return `a${x}b${y}c`;}",
        "function wrapper(arg0, arg1) { return `a${arg0}b${y}c`;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = arg0; return `a${y}b${x}c`;}",
        "function wrapper(arg0, arg1) { let x = arg0; return `a${y}b${x}c`;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = arg0; return `a${arg1}b${x}c`;}",
        "function wrapper(arg0, arg1) { return `a${arg1}b${arg0}c`;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = arg0; return x`y`;}",
        "function wrapper(arg0, arg1) { return arg0`y`;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = arg0; return y`a${x}b`;}",
        "function wrapper(arg0, arg1) { let x = arg0; return y`a${x}b`;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = arg0; return arg1`a${x}b`;}",
        "function wrapper(arg0, arg1) { return arg1`a${arg0}b`;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = 'x'; return `a${x}b`;}",
        "function wrapper(arg0, arg1) { return `axb`;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = arg0; return import(x);}",
        "function wrapper(arg0, arg1) { return import(arg0);}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = arg0; return [import(y), x];}",
        "function wrapper(arg0, arg1) { let x = arg0; return [import(y), x];}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = arg0; return [import(arg1), x];}",
        "function wrapper(arg0, arg1) { return [import(arg1), arg0];}",
    );
    test(
        "function wrapper(arg0, arg1) {return async () => { let x = arg0; await x; };}",
        "function wrapper(arg0, arg1) { return async () => { await arg0; };}",
    );
    test(
        "function wrapper(arg0, arg1) {return async () => { let x = arg0; await y; return x; };}",
        "function wrapper(arg0, arg1) { return async () => { let x = arg0; return await y, x; };}",
    );
    test("function wrapper(arg0, arg1) {return async () => { let x = arg0; await arg1; return x; };}", "function wrapper(arg0, arg1) { return async () => { let x = arg0; return await arg1, x; };}");
    test(
        "function wrapper(arg0, arg1) {return function* () { let x = arg0; yield x; };}",
        "function wrapper(arg0, arg1) { return function* () { yield arg0; };}",
    );
    test(
        "function wrapper(arg0, arg1) {return function* () { let x = arg0; yield; return x; };}",
        "function wrapper(arg0, arg1) { return function* () { let x = arg0; return yield, x; };}",
    );
    test(
        "function wrapper(arg0, arg1) {return function* () { let x = arg0; yield y; return x; };}",
        "function wrapper(arg0, arg1) { return function* () { let x = arg0; return yield y, x; };}",
    );
    test("function wrapper(arg0, arg1) {return function* () { let x = arg0; yield arg1; return x; };}", "function wrapper(arg0, arg1) { return function* () { let x = arg0; return yield arg1, x; };}");
    test(
        "function wrapper(arg0, arg1) { let x = arg0; x()}",
        "function wrapper(arg0, arg1) { arg0();}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = arg0; (0, x)()}",
        "function wrapper(arg0, arg1) { arg0();}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = arg0.foo; x.bar()}",
        "function wrapper(arg0, arg1) { arg0.foo.bar();}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = arg0.foo; x[bar]()}",
        "function wrapper(arg0, arg1) { arg0.foo[bar]();}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = arg0.foo; x()}",
        "function wrapper(arg0, arg1) { let x = arg0.foo; x();}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = arg0[foo]; x()}",
        "function wrapper(arg0, arg1) { let x = arg0[foo]; x();}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = arg0?.foo; x()}",
        "function wrapper(arg0, arg1) { let x = arg0?.foo; x();}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = arg0?.[foo]; x()}",
        "function wrapper(arg0, arg1) { let x = arg0?.[foo]; x();}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = arg0.foo; (0, x)()}",
        "function wrapper(arg0, arg1) { let x = arg0.foo; x();}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = arg0[foo]; (0, x)()}",
        "function wrapper(arg0, arg1) { let x = arg0[foo]; x();}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = arg0?.foo; (0, x)()}",
        "function wrapper(arg0, arg1) { let x = arg0?.foo; x();}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = arg0?.[foo]; (0, x)()}",
        "function wrapper(arg0, arg1) { let x = arg0?.[foo]; x();}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = arg0(); arg1() + x}",
        "function wrapper(arg0, arg1) { let x = arg0(); arg1() + x;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = arg0(); /* @__PURE__ */ arg1() + x}",
        "function wrapper(arg0, arg1) { let x = arg0(); /* @__PURE__ */ arg1() + x;}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = /* @__PURE__ */ arg0(); arg1() + x}",
        "function wrapper(arg0, arg1) { let x = /* @__PURE__ */ arg0(); arg1() + x;}",
    );
    test(
    "function wrapper(arg0, arg1) { let x = /* @__PURE__ */ arg0(); /* @__PURE__ */ arg1() + x}",
    "function wrapper(arg0, arg1) { /* @__PURE__ */ arg1() + /* @__PURE__ */ arg0();}",
  );
    test("if (1) a(); else { ; }", "a();");
    test("if (1) a(); else { b() }", "a();");
    test("if (1) a(); else { const b = c }", "a();");
    test("if (1) a(); else { let b }", "a();");
    test("if (1) a(); else { throw b }", "a();");
    test("if (1) a(); else { return b }", "a();");
    test("b: { if (x) a(); else { break b } }", "b: if (x) a(); else break b;");
    test("b: { if (1) a(); else { break b } }", "a();");
    test("b: { if (0) a(); else { break b } }", "");
    test(
        "b: while (1) if (x) a(); else { continue b }",
        "b: for (;;) if (x) a(); else continue b;",
    );
    test("b: while (1) if (1) a(); else { continue b }", "for (;;) a();");
    test("b: while (1) if (0) a(); else { continue b }", "b: for (;;) continue b;");
    test("if (1) a(); else { class b {} }", "a();");
    test("if (1) a(); else { debugger }", "a();");
    test("if (1) a(); else { switch (1) { case 1: b() } }", "a();");
    test("if (0) { let a = 1} else a()", "a();");
    test("if (1) { let a = 1} else a()", "{ let a = 1;}");
    test("if (0) a(); else { let a = 1}", "{ let a = 1;}");
    test("if (1) a(); else { let a = 1}", "a();");
    test("if (1) a(); else { var a = b }", "if (1) a(); else var a;");
    test("if (1) a(); else { var [a] = b }", "if (1) a(); else var a;");
    test("if (1) a(); else { var {x: a} = b }", "if (1) a(); else var a;");
    test("if (1) a(); else { var [] = b }", "a();");
    test("if (1) a(); else { var {} = b }", "a();");
    test("if (1) a(); else { function a() {} }", "if (1) a(); else var a;");
    test("if (1) a(); else { for(;;){var a} }", "if (1) a(); else for (;;) var a;");
    test("if (1) { a(); b() } else { var a; var b; }", "if (1) a(), b(); else var a, b;");
    test("if (1) a(); else { switch (1) { case 1: case 2: var a } }", "if (1) a(); else var a;");
    test("import 'x' assert {'type': 'json'}", "import 'x' assert { type: 'json' };");
    test("import 'x' assert {'ty pe': 'json'}", "import 'x' assert { 'ty pe': 'json' };");
    test(
        "import(x ? 'y' : 'z', {assert: {'a': 'b'}})",
        "x ? import('y', { assert: { a: 'b' } }) : import('z', { assert: { a: 'b' } });",
    );
    test(
        "import(x ? 'y' : 'z', {assert: {'a a': 'b'}})",
        "x ? import('y', { assert: { 'a a': 'b' } }) : import('z', { assert: { 'a a': 'b' } });",
    );
    test("import 'x' with {'type': 'json'}", "import 'x' with { type: 'json' };");
    test("import 'x' with {'ty pe': 'json'}", "import 'x' with { 'ty pe': 'json' };");
    test(
        "import(x ? 'y' : 'z', {with: {'a': 'b'}})",
        "x ? import('y', { with: { a: 'b' } }) : import('z', { with: { a: 'b' } });",
    );
    test(
        "import(x ? 'y' : 'z', {with: {'a a': 'b'}})",
        "x ? import('y', { with: { 'a a': 'b' } }) : import('z', { with: { 'a a': 'b' } });",
    );
    test(
        "try { throw 0 } catch (e) { console.log(0) }",
        "try { throw 0;} catch { console.log(0);}",
    );
    test(
        "try { throw 0 } catch (e) { console.log(0, e) }",
        "try { throw 0;} catch (e) { console.log(0, e);}",
    );
    test("try { throw 0 } catch (e) { 0 && console.log(0, e) }", "try { throw 0;} catch {}");
    test(
        "try { thrower() } catch ([a]) { console.log(0) }",
        "try { thrower();} catch ([a]) { console.log(0);}",
    );
    test(
        "try { thrower() } catch ({ a }) { console.log(0) }",
        "try { thrower();} catch ({ a }) { console.log(0);}",
    );
    test(
        "try { throw 1 } catch (x) { y(x); var x = 2; y(x) }",
        "try { throw 1;} catch (x) { y(x); var x = 2; y(x);}",
    );
    test(
        "try { throw 1 } catch (x) { var x = 2; y(x) }",
        "try { throw 1;} catch (x) { var x = 2; y(x);}",
    );
    test("try { throw 1 } catch (x) { var x = 2 }", "try { throw 1;} catch (x) { var x = 2;}");
    test("try { throw 1 } catch (x) { eval('x') }", "try { throw 1;} catch (x) { eval('x');}");
    test(
        "if (y) try { throw 1 } catch (x) {} else eval('x')",
        "if (y) try { throw 1;} catch {}else eval('x');",
    );
    test("try { throw 0 } catch (e) { foo() }", "try { throw 0;} catch { foo();}");
    test("try {} catch (e) { var foo }", "try {} catch { var foo;}");
    test("try {} catch (e) { foo() }", "");
    test("try {} catch (e) { foo() } finally {}", "");
    test("try {} finally { foo() }", "foo();");
    test("try {} catch (e) { foo() } finally { bar() }", "bar();");
    test("try {} finally { var x = foo() }", "var x = foo();");
    test("try {} catch (e) { foo() } finally { var x = bar() }", "var x = bar();");
    test("try {} finally { let x = foo() }", "{ let x = foo();}");
    test("try {} catch (e) { foo() } finally { let x = bar() }", "{ let x = bar();}");
    test("using x = {}", "using x = {};");
    test("using x = null", "const x = null;");
    test("using x = undefined", "const x = void 0;");
    test("using x = (foo, y)", "using x = (foo, y);");
    test("using x = (foo, null)", "const x = (foo, null);");
    test("using x = (foo, undefined)", "const x = (foo, void 0);");
    test("using x = null, y = undefined", "const x = null, y = void 0;");
    test("using x = null, y = z", "using x = null, y = z;");
    test("using x = z, y = undefined", "using x = z, y = void 0;");
}
