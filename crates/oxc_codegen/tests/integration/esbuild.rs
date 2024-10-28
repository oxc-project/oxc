//! Tests ported from `esbuild`
//! * <https://github.com/evanw/esbuild/blob/main/internal/js_printer/js_printer_test.go>
//! * <https://github.com/evanw/esbuild/blob/main/internal/js_parser/js_parser_test.go>

use crate::tester::{test, test_minify};

// NOTE: These values are aligned with terser, not esbuild.
#[test]
fn test_number() {
    // Check "1eN"
    // TODO FIXME
    // test("x = 1e-100", "x = 1e-100;\n");
    // test("x = 1e-4", "x = 1e-4;\n");
    // test("x = 1e-3", "x = 1e-3;\n");
    // test("x = 1e-2", "x = 1e-2;\n");
    // test("x = 1e-1", "x = 1e-1;\n");
    // test("x = 1e0", "x = 1e0;\n");
    // test("x = 1e1", "x = 1e1;\n");
    // test("x = 1e2", "x = 1e2;\n");
    // test("x = 1e3", "x = 1e3;\n");
    // test("x = 1e4", "x = 1e4;\n");
    // test("x = 1e100", "x = 1e100;\n");
    test_minify("x = 1e-100", "x=1e-100;");
    test_minify("x = 1e-5", "x=1e-5;");
    test_minify("x = 1e-4", "x=1e-4;");
    test_minify("x = 1e-3", "x=.001;");
    test_minify("x = 1e-2", "x=.01;");
    test_minify("x = 1e-1", "x=.1;");
    test_minify("x = 1e0", "x=1;");
    test_minify("x = 1e1", "x=10;");
    test_minify("x = 1e2", "x=100;");
    test_minify("x = 1e3", "x=1e3;");
    test_minify("x = 1e4", "x=1e4;");
    test_minify("x = 1e100", "x=1e100;");

    // Check "12eN"
    // TODO FIXME
    // test("x = 12e-100", "x = 12e-100;\n");
    // test("x = 12e-5", "x = 12e-5;\n");
    // test("x = 12e-4", "x = 12e-4;\n");
    // test("x = 12e-3", "x = 12e-3;\n");
    // test("x = 12e-2", "x = 12e-2;\n");
    // test("x = 12e-1", "x = 12e-1;\n");
    // test("x = 12e0", "x = 12e0;\n");
    // test("x = 12e1", "x = 12e1;\n");
    // test("x = 12e2", "x = 12e2;\n");
    // test("x = 12e3", "x = 12e3;\n");
    // test("x = 12e4", "x = 12e4;\n");
    // test("x = 12e100", "x = 12e100;\n");
    test_minify("x = 12e-100", "x=1.2e-99;");
    test_minify("x = 12e-6", "x=12e-6;");
    test_minify("x = 12e-5", "x=12e-5;");
    test_minify("x = 12e-4", "x=.0012;");
    test_minify("x = 12e-3", "x=.012;");
    test_minify("x = 12e-2", "x=.12;");
    test_minify("x = 12e-1", "x=1.2;");
    test_minify("x = 12e0", "x=12;");
    test_minify("x = 12e1", "x=120;");
    test_minify("x = 12e2", "x=1200;");
    test_minify("x = 12e3", "x=12e3;");
    test_minify("x = 12e4", "x=12e4;");
    test_minify("x = 12e100", "x=12e100;");

    // Check cases for "A.BeX" => "ABeY" simplification
    // TODO FIXME
    // test("x = 123456789", "x = 123456789;\n");
    // test("x = 1123456789", "x = 1123456789;\n");
    // test("x = 10123456789", "x = 10123456789;\n");
    // test("x = 100123456789", "x = 100123456789;\n");
    // test("x = 1000123456789", "x = 1000123456789;\n");
    // test("x = 10000123456789", "x = 10000123456789;\n");
    // test("x = 100000123456789", "x = 100000123456789;\n");
    // test("x = 1000000123456789", "x = 1000000123456789;\n");
    // test("x = 10000000123456789", "x = 10000000123456789;\n");
    // test("x = 100000000123456789", "x = 100000000123456789;\n");
    // test("x = 1000000000123456789", "x = 1000000000123456789;\n");
    // test("x = 10000000000123456789", "x = 10000000000123456789;\n");
    // test("x = 100000000000123456789", "x = 100000000000123456789;\n");

    // Check numbers around the ends of various integer ranges. These were
    // crashing in the WebAssembly build due to a bug in the Go runtime.

    // int32
    test_minify("x = 0x7fff_ffff", "x=2147483647;");
    test_minify("x = 0x8000_0000", "x=2147483648;");
    test_minify("x = 0x8000_0001", "x=2147483649;");
    test_minify("x = -0x7fff_ffff", "x=-2147483647;");
    test_minify("x = -0x8000_0000", "x=-2147483648;");
    test_minify("x = -0x8000_0001", "x=-2147483649;");

    // uint32
    test_minify("x = 0xffff_ffff", "x=4294967295;");
    test_minify("x = 0x1_0000_0000", "x=4294967296;");
    test_minify("x = 0x1_0000_0001", "x=4294967297;");
    test_minify("x = -0xffff_ffff", "x=-4294967295;");
    test_minify("x = -0x1_0000_0000", "x=-4294967296;");
    test_minify("x = -0x1_0000_0001", "x=-4294967297;");

    // int64
    test_minify("x = 0x7fff_ffff_ffff_fdff", "x=0x7ffffffffffffc00;");
    test_minify("x = 0x8000_0000_0000_0000", "x=0x8000000000000000;");
    test_minify("x = 0x8000_0000_0000_3000", "x=0x8000000000003000;");
    test_minify("x = -0x7fff_ffff_ffff_fdff", "x=-0x7ffffffffffffc00;");
    test_minify("x = -0x8000_0000_0000_0000", "x=-0x8000000000000000;");
    test_minify("x = -0x8000_0000_0000_3000", "x=-0x8000000000003000;");

    // uint64
    test_minify("x = 0xffff_ffff_ffff_fbff", "x=0xfffffffffffff800;");
    test_minify("x = 0x1_0000_0000_0000_0000", "x=0x10000000000000000;");
    test_minify("x = 0x1_0000_0000_0000_1000", "x=0x10000000000001000;");
    test_minify("x = -0xffff_ffff_ffff_fbff", "x=-0xfffffffffffff800;");
    test_minify("x = -0x1_0000_0000_0000_0000", "x=-0x10000000000000000;");
    test_minify("x = -0x1_0000_0000_0000_1000", "x=-0x10000000000001000;");

    // Check the hex vs. decimal decision boundary when minifying
    // TODO FIXME
    // test("x = 999999999999", "x = 999999999999;\n");
    // test("x = 1000000000001", "x = 1000000000001;\n");
    // test("x = 0x0FFF_FFFF_FFFF_FF80", "x = 0x0FFF_FFFF_FFFF_FF80;\n");
    // test("x = 0x1000_0000_0000_0000", "x = 0x1000_0000_0000_0000;\n");
    // test("x = 0xFFFF_FFFF_FFFF_F000", "x = 0xFFFF_FFFF_FFFF_F000;\n");
    // test("x = 0xFFFF_FFFF_FFFF_F800", "x = 0xFFFF_FFFF_FFFF_F800;\n");
    // test("x = 0xFFFF_FFFF_FFFF_FFFF", "x = 0xFFFF_FFFF_FFFF_FFFF;\n");
    test_minify("x = 999999999999", "x=999999999999;");
    test_minify("x = 1000000000001", "x=0xe8d4a51001;");
    test_minify("x = 0x0FFF_FFFF_FFFF_FF80", "x=0xfffffffffffff80;");
    test_minify("x = 0x1000_0000_0000_0000", "x=0x1000000000000000;");
    test_minify("x = 0xFFFF_FFFF_FFFF_F000", "x=0xfffffffffffff000;");
    test_minify("x = 0xFFFF_FFFF_FFFF_F800", "x=0xfffffffffffff800;");
    test_minify("x = 0xFFFF_FFFF_FFFF_FFFF", "x=0x10000000000000000;");

    // Check printing a space in between a number and a subsequent "."
    test_minify("x = 0.0001 .y", "x=1e-4.y;");
    test_minify("x = 0.001 .y", "x=.001.y;");
    test_minify("x = 0.01 .y", "x=.01.y;");
    test_minify("x = 0.1 .y", "x=.1.y;");
    test_minify("x = 0 .y", "x=0 .y;");
    test_minify("x = 10 .y", "x=10 .y;");
    test_minify("x = 100 .y", "x=100 .y;");
    test_minify("x = 1000 .y", "x=1e3.y;");
    test_minify("x = 12345 .y", "x=12345 .y;");
    test_minify("x = 0xFFFF_0000_FFFF_0000 .y", "x=0xffff0000ffff0000.y;");
}

#[test]
fn test_array() {
    test("[]", "[];\n");
    test("[,]", "[,];\n");
    test("[,,]", "[, ,];\n");
}

#[test]
fn test_splat() {
    test("[...(a, b)]", "[...(a, b)];\n");
    test("x(...(a, b))", "x(...(a, b));\n");
    test("({...(a, b)})", "({ ...(a, b) });\n");
}

#[test]
fn test_new() {
    test("new x", "new x();\n");
    test("new x()", "new x();\n");
    test("new (x)", "new x();\n");
    test("new (x())", "new (x())();\n");
    test("new (new x())", "new new x()();\n");
    test("new (x + x)", "new (x + x)();\n");
    test("(new x)()", "new x()();\n");

    test("new foo().bar", "new foo().bar;\n");
    test("new (foo().bar)", "new (foo()).bar();\n");
    test("new (foo()).bar", "new (foo()).bar();\n");
    test("new foo()[bar]", "new foo()[bar];\n");
    test("new (foo()[bar])", "new (foo())[bar]();\n");
    test("new (foo())[bar]", "new (foo())[bar]();\n");

    test("new (import('foo').bar)", "new (import(\"foo\")).bar();\n");
    test("new (import('foo')).bar", "new (import(\"foo\")).bar();\n");
    test("new (import('foo')[bar])", "new (import(\"foo\"))[bar]();\n");
    test("new (import('foo'))[bar]", "new (import(\"foo\"))[bar]();\n");

    // test_minify("new x", "new x;");
    // test_minify("new x.y", "new x.y;");
    // test_minify("(new x).y", "new x().y;");
    // test_minify("new x().y", "new x().y;");
    // test_minify("new x() + y", "new x+y;");
    // test_minify("new x() ** 2", "new x**2;");

    // Test preservation of Webpack-specific comments
    test(
        "new Worker(// webpackFoo: 1\n // webpackBar: 2\n 'path');",
        "new Worker(\n\t// webpackFoo: 1\n\t// webpackBar: 2\n\t\"path\"\n);\n",
    );
    test(
        "new Worker(/* webpackFoo: 1 */ /* webpackBar: 2 */ 'path');",
        "new Worker(\n\t/* webpackFoo: 1 */\n\t/* webpackBar: 2 */\n\t\"path\"\n);\n",
    );
    test(
        "new Worker(\n    /* multi\n     * line\n     * webpackBar: */ 'path');",
        "new Worker(\n\t/* multi\n\t* line\n\t* webpackBar: */\n\t\"path\"\n);\n",
    );
    test(
        "new Worker(/* webpackFoo: 1 */ 'path' /* webpackBar:2 */);",
        "new Worker(\n\t/* webpackFoo: 1 */\n\t\"path\"\n\t/* webpackBar:2 */\n);\n",
    );
    test(
        "new Worker(/* webpackFoo: 1 */ 'path' /* webpackBar:2 */ ,);",
        "new Worker(\n\t/* webpackFoo: 1 */\n\t\"path\"\n);\n",
    ); // Not currently handled
    test(
        "new Worker(/* webpackFoo: 1 */ 'path', /* webpackBar:2 */ );",
        "new Worker(\n\t/* webpackFoo: 1 */\n\t\"path\"\n\t/* webpackBar:2 */\n);\n",
    );
    test( "new Worker(new URL('path', /* webpackFoo: these can go anywhere */ import.meta.url))",
    "new Worker(new URL(\n\t\"path\",\n\t/* webpackFoo: these can go anywhere */\n\timport.meta.url\n));\n");

    // non-webpack comments
    test("new Worker(/* before */ foo)", "new Worker(\n\t/* before */\n\tfoo\n);\n");
    test("new Worker(/* before */ 'foo')", "new Worker(\n\t/* before */\n\t\"foo\"\n);\n");
    test("new Worker(foo /* after */)", "new Worker(\n\tfoo\n\t/* after */\n);\n");
    test("new Worker('foo' /* after */)", "new Worker(\n\t\"foo\"\n\t/* after */\n);\n");
}

#[test]
fn test_call() {
    test("x()()()", "x()()();\n");
    test("x().y()[z]()", "x().y()[z]();\n");
    test("(--x)();", "(--x)();\n");
    test("(x--)();", "(x--)();\n");

    test("eval(x)", "eval(x);\n");
    test("eval?.(x)", "eval?.(x);\n");
    test("(eval)(x)", "eval(x);\n");
    test("(eval)?.(x)", "eval?.(x);\n");

    test("eval(x, y)", "eval(x, y);\n");
    test("eval?.(x, y)", "eval?.(x, y);\n");
    test("(1, eval)(x)", "(1, eval)(x);\n");
    test("(1, eval)?.(x)", "(1, eval)?.(x);\n");
    // testMangle(t, "(1 ? eval : 2)(x)", "(0, eval)(x);\n");
    // testMangle(t, "(1 ? eval : 2)?.(x)", "eval?.(x);\n");

    test_minify("eval?.(x)", "eval?.(x);");
    test_minify("eval(x,y)", "eval(x,y);");
    test_minify("eval?.(x,y)", "eval?.(x,y);");
    test_minify("(1, eval)(x)", "(1,eval)(x);");
    test_minify("(1, eval)?.(x)", "(1,eval)?.(x);");
    // testMangleMinify(t, "(1 ? eval : 2)(x)", "(0,eval)(x);");
    // testMangleMinify(t, "(1 ? eval : 2)?.(x)", "eval?.(x);");

    // Webpack-specific comments
    test(
        "require(// webpackFoo: 1\n // webpackBar: 2\n 'path');",
        "require(\n\t// webpackFoo: 1\n\t// webpackBar: 2\n\t\"path\"\n);\n",
    );
    test( "require(// webpackFoo: 1\n // webpackBar: 2\n 'path', {type: 'module'});",
    "require(\n\t// webpackFoo: 1\n\t// webpackBar: 2\n\t\"path\",\n\t{ type: \"module\" }\n);\n");
    test(
        "require(/* webpackFoo: 1 */ /* webpackBar: 2 */ 'path');",
        "require(\n\t/* webpackFoo: 1 */\n\t/* webpackBar: 2 */\n\t\"path\"\n);\n",
    );
    test( "require(/* webpackFoo: 1 */ /* webpackBar: 2 */ 'path', {type: 'module'});",
        "require(\n\t/* webpackFoo: 1 */\n\t/* webpackBar: 2 */\n\t\"path\",\n\t{ type: \"module\" }\n);\n");
    test(
        "require(\n    /* multi\n     * line\n     * webpackBar: */ 'path');",
        "require(\n\t/* multi\n\t* line\n\t* webpackBar: */\n\t\"path\"\n);\n",
    );
    test(
        "require(/* webpackFoo: 1 */ 'path' /* webpackBar:2 */);",
        "require(\n\t/* webpackFoo: 1 */\n\t\"path\"\n\t/* webpackBar:2 */\n);\n",
    );
    test(
        "require(/* webpackFoo: 1 */ 'path' /* webpackBar:2 */ ,);",
        "require(\n\t/* webpackFoo: 1 */\n\t\"path\"\n);\n",
    ); // Not currently handled
    test(
        "require(/* webpackFoo: 1 */ 'path', /* webpackBar:2 */ );",
        "require(\n\t/* webpackFoo: 1 */\n\t\"path\"\n\t/* webpackBar:2 */\n);\n",
    );
    test( "require(/* webpackFoo: 1 */ 'path', { type: 'module' } /* webpackBar:2 */ );", "require(\n\t/* webpackFoo: 1 */\n\t\"path\",\n\t{ type: \"module\" }\n\t/* webpackBar:2 */\n);\n");

    // non-webpack comments
    test("require(/* before */ foo)", "require(\n\t/* before */\n\tfoo\n);\n");
    test("require(/* before */ 'foo')", "require(\n\t/* before */\n\t\"foo\"\n);\n");
    test("require(foo /* after */)", "require(\n\tfoo\n\t/* after */\n);\n");
    test("require('foo' /* after */)", "require(\n\t\"foo\"\n\t/* after */\n);\n");
}

#[test]
fn test_member() {
    test("x.y[z]", "x.y[z];\n");
    test("((x+1).y+1)[z]", "((x + 1).y + 1)[z];\n");
}

#[test]
fn test_comma() {
    test("1, 2, 3", "1, 2, 3;\n");
    test("(1, 2), 3", "1, 2, 3;\n");
    test("1, (2, 3)", "1, 2, 3;\n");
    test("a ? (b, c) : (d, e)", "a ? (b, c) : (d, e);\n");
    test("let x = (a, b)", "let x = (a, b);\n");
    test("(x = a), b", "x = a, b;\n");
    test("x = (a, b)", "x = (a, b);\n");
    test("x((1, 2))", "x((1, 2));\n");
}

#[test]
fn test_unary() {
    test("+(x--)", "+x--;\n");
    test("-(x++)", "-x++;\n");
}

#[test]
fn test_nullish() {
    // "??" can't directly contain "||" or "&&"
    test("(a && b) ?? c", "(a && b) ?? c;\n");
    test("(a || b) ?? c", "(a || b) ?? c;\n");
    test("a ?? (b && c)", "a ?? (b && c);\n");
    test("a ?? (b || c)", "a ?? (b || c);\n");

    // "||" and "&&" can't directly contain "??"
    test("a && (b ?? c)", "a && (b ?? c);\n");
    test("a || (b ?? c)", "a || (b ?? c);\n");
    test("(a ?? b) && c", "(a ?? b) && c;\n");
    test("(a ?? b) || c", "(a ?? b) || c;\n");
}

#[test]
fn test_string() {
    test("let x = ''", "let x = \"\";\n");
    test("let x = '\\b'", "let x = \"\\b\";\n");
    test("let x = '\\f'", "let x = \"\\f\";\n");
    test("let x = '\t'", "let x = \"\t\";\n");
    test("let x = '\\v'", "let x = \"\\v\";\n");
    test("let x = '\\n'", "let x = \"\\n\";\n");
    test("let x = '\\''", "let x = \"'\";\n");
    test("let x = '\\\"'", "let x = \"\\\"\";\n");
    test("let x = '\\'\"'", "let x = \"'\\\"\";\n");
    test("let x = '\\\\'", "let x = \"\\\\\";\n");
    test("let x = '\x00'", "let x = \"\\0\";\n");
    test("let x = '\x00!'", "let x = \"\\0!\";\n");
    test("let x = '\x001'", "let x = \"\\x001\";\n");
    test("let x = '\\0'", "let x = \"\\0\";\n");
    test("let x = '\\0!'", "let x = \"\\0!\";\n");
    test("let x = '\x07'", "let x = \"\\x07\";\n");
    test("let x = '\x07!'", "let x = \"\\x07!\";\n");
    test("let x = '\x071'", "let x = \"\\x071\";\n");
    test("let x = '\\7'", "let x = \"\\x07\";\n");
    test("let x = '\\7!'", "let x = \"\\x07!\";\n");
    test("let x = '\\01'", "let x = \"\x01\";\n");
    test("let x = '\x10'", "let x = \"\x10\";\n");
    test("let x = '\\x10'", "let x = \"\x10\";\n");
    test("let x = '\x1B'", "let x = \"\\x1B\";\n");
    test("let x = '\\x1B'", "let x = \"\\x1B\";\n");
    test("let x = '\u{ABCD}'", "let x = \"\u{ABCD}\";\n");
    test("let x = '\\uABCD'", "let x = \"\u{ABCD}\";\n");
    // test( "let x = '\U000123AB'", "let x = \"\U000123AB\";\n");
    // test( "let x = '\\u{123AB}'", "let x = \"\U000123AB\";\n");
    // test( "let x = '\\uD808\\uDFAB'", "let x = \"\U000123AB\";\n");
    // test( "let x = '\\uD808'", "let x = \"\\uD808\";\n");
    // test( "let x = '\\uD808X'", "let x = \"\\uD808X\";\n");
    // test( "let x = '\\uDFAB'", "let x = \"\\uDFAB\";\n");
    // test( "let x = '\\uDFABX'", "let x = \"\\uDFABX\";\n");

    // test( "let x = '\\x80'", "let x = \"\U00000080\";\n");
    // test( "let x = '\\xFF'", "let x = \"\U000000FF\";\n");
    // test( "let x = '\\xF0\\x9F\\x8D\\x95'", "let x = \"\U000000F0\U0000009F\U0000008D\U00000095\";\n");
    // test( "let x = '\\uD801\\uDC02\\uDC03\\uD804'", "let x = \"\U00010402\\uDC03\\uD804\";\n");
}

#[test]
fn test_template() {
    test("let x = `\\0`", "let x = `\\0`;\n");
    test("let x = `\\x01`", "let x = `\\x01`;\n");
    test("let x = `\\0${0}`", "let x = `\\0${0}`;\n");
    test("let x = `\\x01${0}`", "let x = `\\x01${0}`;\n");
    test("let x = `${0}\\0`", "let x = `${0}\\0`;\n");
    test("let x = `${0}\\x01`", "let x = `${0}\\x01`;\n");
    test("let x = `${0}\\0${1}`", "let x = `${0}\\0${1}`;\n");
    test("let x = `${0}\\x01${1}`", "let x = `${0}\\x01${1}`;\n");

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
    test("let x = `$\\{y}`", "let x = `$\\{y}`;\n");

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
    // test_minify("new tag`x`", "new tag`x`;");
    // test_minify("new (tag`x`)", "new tag`x`;");
    // test_minify("new tag()`x`", "new tag()`x`;");
    // test_minify("(new tag)`x`", "new tag()`x`;");

    test("new tag`${x}`", "new tag`${x}`();\n");
    test("new (tag`${x}`)", "new tag`${x}`();\n");
    test("new tag()`${x}`", "new tag()`${x}`;\n");
    test("(new tag)`${x}`", "new tag()`${x}`;\n");
    // test_minify("new tag`${x}`", "new tag`${x}`;");
    // test_minify("new (tag`${x}`)", "new tag`${x}`;");
    // test_minify("new tag()`${x}`", "new tag()`${x}`;");
    // test_minify("(new tag)`${x}`", "new tag()`${x}`;");
}

#[test]
fn test_object() {
    test("let x = {'(':')'}", "let x = { \"(\": \")\" };\n");
    test("({})", "({});\n");
    test("({}.x)", "({}).x;\n");
    test("({} = {})", "({} = {});\n");
    test("(x, {} = {})", "x, {} = {};\n");
    test("let x = () => ({})", "let x = () => ({});\n");
    test("let x = () => ({}.x)", "let x = () => ({}).x;\n");
    test("let x = () => ({} = {})", "let x = () => ({} = {});\n");
    test("let x = () => (x, {} = {})", "let x = () => (x, {} = {});\n");

    // "{ __proto__: __proto__ }" must not become "{ __proto__ }"
    test(
        "function foo(__proto__) { return { __proto__: __proto__ } }",
        "function foo(__proto__) {\n\treturn { __proto__: __proto__ };\n}\n",
    );
    test(
        "function foo(__proto__) { return { '__proto__': __proto__ } }",
        "function foo(__proto__) {\n\treturn { \"__proto__\": __proto__ };\n}\n",
    );
    test(
        "function foo(__proto__) { return { ['__proto__']: __proto__ } }",
        "function foo(__proto__) {\n\treturn { [\"__proto__\"]: __proto__ };\n}\n",
    );
    test(
        "import { __proto__ } from 'foo'; let foo = () => ({ __proto__: __proto__ })",
        "import { __proto__ } from \"foo\";\nlet foo = () => ({ __proto__: __proto__ });\n",
    );
    test(
        "import { __proto__ } from 'foo'; let foo = () => ({ '__proto__': __proto__ })",
        "import { __proto__ } from \"foo\";\nlet foo = () => ({ \"__proto__\": __proto__ });\n",
    );
    test(
        "import { __proto__ } from 'foo'; let foo = () => ({ ['__proto__']: __proto__ })",
        "import { __proto__ } from \"foo\";\nlet foo = () => ({ [\"__proto__\"]: __proto__ });\n",
    );

    // Don't use ES6+ features (such as a shorthand or computed property name) in ES5
    // testTarget(
    // t,
    // 5,
    // "function foo(__proto__) { return { __proto__ } }",
    // "function foo(__proto__) {\n  return { __proto__: __proto__ };\n}\n",
    // );
}

#[test]
fn test_for() {
    // Make sure "in" expressions are forbidden in the right places
    test("for ((a in b);;);", "for ((a in b);;);\n");
    test("for (a ? b : (c in d);;);", "for (a ? b : (c in d);;);\n");
    test("for ((a ? b : c in d).foo;;);", "for ((a ? b : c in d).foo;;);\n");
    test("for (var x = (a in b);;);", "for (var x = (a in b);;);\n");
    test("for (x = (a in b);;);", "for (x = (a in b);;);\n");
    test("for (x == (a in b);;);", "for (x == (a in b);;);\n");
    test("for (1 * (x == a in b);;);", "for (1 * (x == a in b);;);\n");
    test("for (a ? b : x = (c in d);;);", "for (a ? b : x = (c in d);;);\n");
    test("for (var x = y = (a in b);;);", "for (var x = y = (a in b);;);\n");
    test("for ([a in b];;);", "for ([a in b];;);\n");
    test("for (x(a in b);;);", "for (x(a in b);;);\n");
    test("for (x[a in b];;);", "for (x[a in b];;);\n");
    test("for (x?.[a in b];;);", "for (x?.[a in b];;);\n");
    test("for ((x => a in b);;);", "for ((x) => (a in b);;);\n");

    // Make sure for-of loops with commas are wrapped in parentheses
    test("for (let a in b, c);", "for (let a in b, c);\n");
    test("for (let a of (b, c));", "for (let a of (b, c));\n");
}

#[test]
fn test_function() {
    test("function foo(a = (b, c), ...d) {}", "function foo(a = (b, c), ...d) {}\n");
    test(
        "function foo({[1 + 2]: a = 3} = {[1 + 2]: 3}) {}",
        "function foo({ [1 + 2]: a = 3 } = { [1 + 2]: 3 }) {}\n",
    );
    test(
        "function foo([a = (1, 2), ...[b, ...c]] = [1, [2, 3]]) {}",
        "function foo([a = (1, 2), ...[b, ...c]] = [1, [2, 3]]) {}\n",
    );
    test("function foo([] = []) {}", "function foo([] = []) {}\n");
    test("function foo([,] = [,]) {}", "function foo([,] = [,]) {}\n");
    test("function foo([,,] = [,,]) {}", "function foo([, ,] = [, ,]) {}\n");
}

#[test]
#[ignore]
fn test_comments_and_parentheses() {
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
    test("for ((/* foo */ let).x of y) ;", "for (\n  /* foo */\n  (let).x of y\n) ;\n");
    test("for (/* foo */ (let).x of y) ;", "for (\n  /* foo */\n  (let).x of y\n) ;\n");
    test(
        "function *x() { yield (/* foo */ y) }",
        "function* x() {\n  yield (\n    /* foo */\n    y\n  );\n}\n",
    );
}

#[test]
fn test_pure_comment() {
    test("function* foo() {}", "function* foo() {}\n");
    test("(function* () {})", "(function* () {});\n");
    test("(function* foo() {})", "(function* foo() {});\n");

    test("new (function() {})", "new function() {}();\n");
    test("new (function() {})()", "new function() {}();\n");
    test("/*@__PURE__*/new (function() {})()", "/*@__PURE__*/ new function() {}();\n");

    test("export default (function() { foo() })", "export default (function() {\n\tfoo();\n});\n");
    test(
        "export default (function() { foo() })()",
        "export default (function() {\n\tfoo();\n})();\n",
    );
    test(
        "export default /*@__PURE__*/(function() { foo() })()",
        "export default /*@__PURE__*/ (function() {\n\tfoo();\n})();\n",
    );
}

#[test]
fn test_generator() {
    test("function* foo() {}", "function* foo() {}\n");
    test("(function* () {})", "(function* () {});\n");
    test("(function* foo() {})", "(function* foo() {});\n");

    test("class Foo { *foo() {} }", "class Foo {\n\t*foo() {}\n}\n");
    test("class Foo { static *foo() {} }", "class Foo {\n\tstatic *foo() {}\n}\n");
    test("class Foo { *[foo]() {} }", "class Foo {\n\t*[foo]() {}\n}\n");
    test("class Foo { static *[foo]() {} }", "class Foo {\n\tstatic *[foo]() {}\n}\n");

    test("(class { *foo() {} })", "(class {\n\t*foo() {}\n});\n");
    test("(class { static *foo() {} })", "(class {\n\tstatic *foo() {}\n});\n");
    test("(class { *[foo]() {} })", "(class {\n\t*[foo]() {}\n});\n");
    test("(class { static *[foo]() {} })", "(class {\n\tstatic *[foo]() {}\n});\n");
}

#[test]
fn test_arrow() {
    test("() => {}", "() => {};\n");
    test("x => (x, 0)", "(x) => (x, 0);\n");
    test("x => {y}", "(x) => {\n\ty;\n};\n");
    test("(a = (b, c), ...d) => {}", "(a = (b, c), ...d) => {};\n");
    test(
        "({[1 + 2]: a = 3} = {[1 + 2]: 3}) => {}",
        "({ [1 + 2]: a = 3 } = { [1 + 2]: 3 }) => {};\n",
    );
    test(
        "([a = (1, 2), ...[b, ...c]] = [1, [2, 3]]) => {}",
        "([a = (1, 2), ...[b, ...c]] = [1, [2, 3]]) => {};\n",
    );
    test("([] = []) => {}", "([] = []) => {};\n");
    test("([,] = [,]) => {}", "([,] = [,]) => {};\n");
    test("([,,] = [,,]) => {}", "([, ,] = [, ,]) => {};\n");
    test("a = () => {}", "a = () => {};\n");
    test("a || (() => {})", "a || (() => {});\n");
    test("({a = b, c = d}) => {}", "({ a = b, c = d }) => {};\n");
    test("([{a = b, c = d} = {}] = []) => {}", "([{ a = b, c = d } = {}] = []) => {};\n");
    test("({a: [b = c] = []} = {}) => {}", "({ a: [b = c] = [] } = {}) => {};\n");

    // These are not arrow functions but initially look like one
    test("(a = b, c)", "a = b, c;\n");
    test("([...a = b])", "[...a = b];\n");
    test("([...a, ...b])", "[...a, ...b];\n");
    test("({a: b, c() {}})", "({\n\ta: b,\n\tc() {}\n});\n");
    test("({a: b, get c() {}})", "({\n\ta: b,\n\tget c() {}\n});\n");
    test("({a: b, set c(x) {}})", "({\n\ta: b,\n\tset c(x) {}\n});\n");
}

#[test]
fn test_class() {
    test("class Foo extends (a, b) {}", "class Foo extends (a, b) {}\n");
    test("class Foo { get foo() {} }", "class Foo {\n\tget foo() {}\n}\n");
    test("class Foo { set foo(x) {} }", "class Foo {\n\tset foo(x) {}\n}\n");
    test("class Foo { static foo() {} }", "class Foo {\n\tstatic foo() {}\n}\n");
    test("class Foo { static get foo() {} }", "class Foo {\n\tstatic get foo() {}\n}\n");
    test("class Foo { static set foo(x) {} }", "class Foo {\n\tstatic set foo(x) {}\n}\n");
}

#[test]
fn test_auto_accessors() {
    test(
        "class Foo { accessor x; static accessor y }",
        "class Foo {\n\taccessor x;\n\tstatic accessor y;\n}\n",
    );
    test(
        "class Foo { accessor [x]; static accessor [y] }",
        "class Foo {\n\taccessor [x];\n\tstatic accessor [y];\n}\n",
    );
    test_minify(
        "class Foo { accessor x; static accessor y }",
        "class Foo{accessor x;static accessor y}",
    );
    test_minify(
        "class Foo { accessor [x]; static accessor [y] }",
        "class Foo{accessor[x];static accessor[y]}",
    );
}

#[test]
fn test_private_identifiers() {
    test(
        "class Foo { #foo; foo() { return #foo in this } }",
        "class Foo {\n\t#foo;\n\tfoo() {\n\t\treturn #foo in this;\n\t}\n}\n",
    );
    // FIXME
    // test_minify(
    // "class Foo { #foo; foo() { return #foo in this } }",
    // "class Foo{#foo;foo(){return#foo in this}}",
    // );
}

#[test]
fn test_decorators() {
    let source = "class Foo {\n@w\nw; @x x; @a1\n@b1@b2\n@c1@c2@c3\ny = @y1 @y2 class {}; @a1\n@b1@b2\n@c1@c2@c3 z =\n@z1\n@z2\nclass {}}";
    let expect = "class Foo {\n\t@w w;\n\t@x x;\n\t@a1 @b1 @b2 @c1 @c2 @c3 y = @y1 @y2 class {};\n\t@a1 @b1 @b2 @c1 @c2 @c3 z = @z1 @z2 class {};\n}\n";
    test(source, expect);
    // test_minify( example, "class Foo{@w w;@x x;@a1@b1@b2@c1@c2@c3 y=@y1@y2 class{};@a1@b1@b2@c1@c2@c3 z=@z1@z2 class{}}");
}

#[test]
fn test_import() {
    test("import('path');", "import(\"path\");\n"); // The semicolon must not be a separate statement

    test(
        "import(// webpackFoo: 1\n // webpackBar: 2\n 'path');",
        "import(\n\t// webpackFoo: 1\n\t// webpackBar: 2\n\t\"path\"\n);\n",
    );
    test( "import(// webpackFoo: 1\n // webpackBar: 2\n 'path', {type: 'module'});", "import(\n\t// webpackFoo: 1\n\t// webpackBar: 2\n\t\"path\",\n\t{ type: \"module\" }\n);\n");
    test(
        "import(/* webpackFoo: 1 */ /* webpackBar: 2 */ 'path');",
        "import(\n\t/* webpackFoo: 1 */\n\t/* webpackBar: 2 */\n\t\"path\"\n);\n",
    );
    test( "import(/* webpackFoo: 1 */ /* webpackBar: 2 */ 'path', {type: 'module'});",
        "import(\n\t/* webpackFoo: 1 */\n\t/* webpackBar: 2 */\n\t\"path\",\n\t{ type: \"module\" }\n);\n");
    test(
        "import(\n    /* multi\n     * line\n     * webpackBar: */ 'path');",
        "import(\n\t/* multi\n\t* line\n\t* webpackBar: */\n\t\"path\"\n);\n",
    );
    test(
        "import(/* webpackFoo: 1 */ 'path' /* webpackBar:2 */);",
        "import(\n\t/* webpackFoo: 1 */\n\t\"path\"\n\t/* webpackBar:2 */\n);\n",
    );
    test(
        "import(/* webpackFoo: 1 */ 'path' /* webpackBar:2 */ ,);",
        "import(\n\t/* webpackFoo: 1 */\n\t\"path\"\n);\n",
    ); // Not currently handled
    test(
        "import(/* webpackFoo: 1 */ 'path', /* webpackBar:2 */ );",
        "import(\n\t/* webpackFoo: 1 */\n\t\"path\"\n\t/* webpackBar:2 */\n);\n",
    );
    test( "import(/* webpackFoo: 1 */ 'path', { type: 'module' } /* webpackBar:2 */ );", "import(\n\t/* webpackFoo: 1 */\n\t\"path\",\n\t{ type: \"module\" }\n\t/* webpackBar:2 */\n);\n");
    test( "import(new URL('path', /* webpackFoo: these can go anywhere */ import.meta.url))",
    "import(new URL(\n\t\"path\",\n\t/* webpackFoo: these can go anywhere */\n\timport.meta.url\n));\n");

    // non-webpack comments
    test("import(/* before */ foo)", "import(\n\t/* before */\n\tfoo\n);\n");
    test("import(/* before */ 'foo')", "import(\n\t/* before */\n\t\"foo\"\n);\n");
    test("import(foo /* after */)", "import(\n\tfoo\n\t/* after */\n);\n");
    test("import('foo' /* after */)", "import(\n\t\"foo\"\n\t/* after */\n);\n");
}

#[test]
fn test_export_default() {
    test("export default function() {}", "export default function() {}\n");
    test("export default function foo() {}", "export default function foo() {}\n");
    test("export default async function() {}", "export default async function() {}\n");
    test("export default async function foo() {}", "export default async function foo() {}\n");
    test("export default class {}", "export default class {}\n");
    test("export default class foo {}", "export default class foo {}\n");

    test("export default (function() {})", "export default (function() {});\n");
    test("export default (function foo() {})", "export default (function foo() {});\n");
    test("export default (async function() {})", "export default (async function() {});\n");
    test("export default (async function foo() {})", "export default (async function foo() {});\n");
    test("export default (class {})", "export default (class {});\n");
    test("export default (class foo {})", "export default (class foo {});\n");

    test(
        "export default (function() {}.toString())",
        "export default (function() {}).toString();\n",
    );
    test(
        "export default (function foo() {}.toString())",
        "export default (function foo() {}).toString();\n",
    );
    test(
        "export default (async function() {}.toString())",
        "export default (async function() {}).toString();\n",
    );
    test(
        "export default (async function foo() {}.toString())",
        "export default (async function foo() {}).toString();\n",
    );
    test("export default (class {}.toString())", "export default (class {}).toString();\n");
    test("export default (class foo {}.toString())", "export default (class foo {}).toString();\n");

    test_minify("export default function() {}", "export default function(){}");
    test_minify("export default function foo() {}", "export default function foo(){}");
    test_minify("export default async function() {}", "export default async function(){}");
    test_minify("export default async function foo() {}", "export default async function foo(){}");
    test_minify("export default class {}", "export default class{}");
    test_minify("export default class foo {}", "export default class foo{}");
}

#[test]
fn test_whitespace() {
    test("- -x", "- -x;\n");
    test("+ -x", "+-x;\n");
    test("- +x", "-+x;\n");
    test("+ +x", "+ +x;\n");
    test("- --x", "- --x;\n");
    test("+ --x", "+--x;\n");
    test("- ++x", "-++x;\n");
    test("+ ++x", "+ ++x;\n");

    test_minify("- -x", "- -x;");
    test_minify("+ -x", "+-x;");
    test_minify("- +x", "-+x;");
    test_minify("+ +x", "+ +x;");
    test_minify("- --x", "- --x;");
    test_minify("+ --x", "+--x;");
    test_minify("- ++x", "-++x;");
    test_minify("+ ++x", "+ ++x;");

    test_minify("x - --y", "x- --y;");
    test_minify("x + --y", "x+--y;");
    test_minify("x - ++y", "x-++y;");
    test_minify("x + ++y", "x+ ++y;");

    test_minify("x-- > y", "x-- >y;");
    test_minify("x < !--y", "x<! --y;");
    test_minify("x > !--y", "x>!--y;");
    test_minify("!--y", "!--y;");

    test_minify("1 + -0", "1+-0;");
    test_minify("1 - -0", "1- -0;");
    test_minify("1 + -Infinity", "1+-Infinity;");
    test_minify("1 - -Infinity", "1- -Infinity;");

    test_minify("/x/ / /y/", "/x// /y/;");
    test_minify("/x/ + Foo", "/x/+Foo;");
    test_minify("/x/ instanceof Foo", "/x/ instanceof Foo;");
    test_minify("[x] instanceof Foo", "[x]instanceof Foo;");

    test_minify("throw x", "throw x;");
    test_minify("throw typeof x", "throw typeof x;");
    test_minify("throw delete x", "throw delete x;");
    test_minify("throw function(){}", "throw function(){};");

    test_minify("x in function(){}", "x in function(){};");
    test_minify("x instanceof function(){}", "x instanceof function(){};");
    test_minify("π in function(){}", "π in function(){};");
    test_minify("π instanceof function(){}", "π instanceof function(){};");

    test_minify("()=>({})", "()=>({});");
    test_minify("()=>({}[1])", "()=>({})[1];");
    // test_minify("()=>({}+0)", "()=>\"[object Object]0\";");
    test_minify("()=>function(){}", "()=>function(){};");

    test_minify("(function(){})", "(function(){});");
    test_minify("(class{})", "(class{});");
    test_minify("({})", "({});");
}

// #[test]#[ignore] fn TestMangle(t *testing.T) {
// testMangle(t, "let x = '\\n'", "let x = `\n`;\n");
// testMangle(t, "let x = `\n`", "let x = `\n`;\n");
// testMangle(t, "let x = '\\n${}'", "let x = \"\\n${}\";\n");
// testMangle(t, "let x = `\n\\${}`", "let x = \"\\n${}\";\n");
// testMangle(t, "let x = `\n\\${}${y}\\${}`", "let x = `\n\\${}${y}\\${}`;\n");
// }

#[test]
#[ignore]
fn minify() {
    test_minify("0.1", ".1;");
    test_minify("1.2", "1.2;");

    test_minify("() => {}", "()=>{};");
    test_minify("(a) => {}", "a=>{};");
    test_minify("(...a) => {}", "(...a)=>{};");
    test_minify("(a = 0) => {}", "(a=0)=>{};");
    test_minify("(a, b) => {}", "(a,b)=>{};");

    test("true ** 2", "true ** 2;\n");
    test("false ** 2", "false ** 2;\n");
    test_minify("true ** 2", "true**2;");
    test_minify("false ** 2", "false**2;");
    // testMangle(t, "true ** 2", "(!0) ** 2;\n");
    // testMangle(t, "false ** 2", "(!1) ** 2;\n");

    test_minify("import a from 'path'", "import a from\"path\";");
    test_minify("import * as ns from 'path'", "import*as ns from\"path\";");
    test_minify("import {a, b as c} from 'path'", "import{a,b as c}from\"path\";");
    test_minify("import {a, ' ' as c} from 'path'", "import{a,\" \"as c}from\"path\";");

    test_minify("export * as ns from 'path'", "export*as ns from\"path\";");
    test_minify("export * as ' ' from 'path'", "export*as\" \"from\"path\";");
    test_minify("export {a, b as c} from 'path'", "export{a,b as c}from\"path\";");
    test_minify("export {' ', '-' as ';'} from 'path'", "export{\" \",\"-\"as\";\"}from\"path\";");
    test_minify("let a, b; export {a, b as c}", "let a,b;export{a,b as c};");
    test_minify("let a, b; export {a, b as ' '}", "let a,b;export{a,b as\" \"};");

    // Print some strings using template literals when minifying
    test("x = '\\n'", "x = \"\\n\";\n");
    // testMangle(t, "x = '\\n'", "x = `\n`;\n");
    // testMangle(t, "x = {'\\n': 0}", "x = { \"\\n\": 0 };\n");
    // testMangle(t, "x = class{'\\n' = 0}", "x = class {\n  \"\\n\" = 0;\n};\n");
    // testMangle(t, "class Foo{'\\n' = 0}", "class Foo {\n  \"\\n\" = 0;\n}\n");

    // Special identifiers must not be minified
    test_minify("exports", "exports;");
    test_minify("require", "require;");
    test_minify("module", "module;");

    // Comment statements must not affect their surroundings when minified
    test_minify("//!single\nthrow 1 + 2", "//!single\nthrow 1+2;");
    test_minify("/*!multi-\nline*/\nthrow 1 + 2", "/*!multi-\nline*/throw 1+2;");
}

#[test]
#[ignore]
fn test_es5() {
    // testTargetMangle(t, 5, "foo('a\\n\\n\\nb')", "foo(\"a\\n\\n\\nb\");;\n");
    // testTargetMangle(t, 2015, "foo('a\\n\\n\\nb')", "foo(`a\n\n\nb`);\n");

    // testTarget(t, 5, "foo({a, b})", "foo({ a: a, b: b });\n");
    // testTarget(t, 2015, "foo({a, b})", "foo({ a, b });\n");

    // testTarget(t, 5, "x => x", "(function(x) {\n  return x;\n});\n");
    // testTarget(t, 2015, "x => x", "(x) => x;\n");

    // testTarget(t, 5, "() => {}", "(function() {\n});\n");
    // testTarget(t, 2015, "() => {}", "() => {\n};\n");

    // testTargetMinify(t, 5, "x => x", "(function(x){return x});");
    // testTargetMinify(t, 2015, "x => x", "x=>x;");

    // testTargetMinify(t, 5, "() => {}", "(function(){});");
    // testTargetMinify(t, 2015, "() => {}", "()=>{};");
}

#[test]
fn test_ascii_only() {
    test("let π = 'π'", "let π = \"π\";\n");
    test("let π_ = 'π'", "let π_ = \"π\";\n");
    test("let _π = 'π'", "let _π = \"π\";\n");
    // testASCII(t, "let π = 'π'", "let \\u03C0 = \"\\u03C0\";\n");
    // testASCII(t, "let π_ = 'π'", "let \\u03C0_ = \"\\u03C0\";\n");
    // testASCII(t, "let _π = 'π'", "let _\\u03C0 = \"\\u03C0\";\n");

    test("let 貓 = '🐈'", "let 貓 = \"🐈\";\n");
    test("let 貓abc = '🐈'", "let 貓abc = \"🐈\";\n");
    test("let abc貓 = '🐈'", "let abc貓 = \"🐈\";\n");
    // testASCII(t, "let 貓 = '🐈'", "let \\u8C93 = \"\\u{1F408}\";\n");
    // testASCII(t, "let 貓abc = '🐈'", "let \\u8C93abc = \"\\u{1F408}\";\n");
    // testASCII(t, "let abc貓 = '🐈'", "let abc\\u8C93 = \"\\u{1F408}\";\n");

    // Test a character outside the BMP
    test("var 𐀀", "var 𐀀;\n");
    test("var \\u{10000}", "var 𐀀;\n");
    // testASCII(t, "var 𐀀", "var \\u{10000};\n");
    // testASCII(t, "var \\u{10000}", "var \\u{10000};\n");
    // testTargetASCII(t, 2015, "'𐀀'", "\"\\u{10000}\";\n");
    // testTargetASCII(t, 5, "'𐀀'", "\"\\uD800\\uDC00\";\n");
    // testTargetASCII(t, 2015, "x.𐀀", "x[\"\\u{10000}\"];\n");
    // testTargetASCII(t, 5, "x.𐀀", "x[\"\\uD800\\uDC00\"];\n");

    // Escapes should use consistent case
    // testASCII(
    // t,
    // "var \\u{100a} = {\\u100A: '\\u100A'}",
    // "var \\u100A = { \\u100A: \"\\u100A\" };\n",
    // );
    // testASCII(
    // t,
    // "var \\u{1000a} = {\\u{1000A}: '\\u{1000A}'}",
    // "var \\u{1000A} = { \"\\u{1000A}\": \"\\u{1000A}\" };\n",
    // );

    // These characters should always be escaped
    // test( "let x = '\u2028'", "let x = \"\\u2028\";\n");
    // test( "let x = '\u2029'", "let x = \"\\u2029\";\n");
    // test( "let x = '\uFEFF'", "let x = \"\\uFEFF\";\n");

    // There should still be a space before "extends"
    // testASCII(t, "class 𐀀 extends π {}", "class \\u{10000} extends \\u03C0 {\n}\n");
    // testASCII(t, "(class 𐀀 extends π {})", "(class \\u{10000} extends \\u03C0 {\n});\n");
    // test_minifyASCII(t, "class 𐀀 extends π {}", "class \\u{10000} extends \\u03C0{}");
    // test_minifyASCII(t, "(class 𐀀 extends π {})", "(class \\u{10000} extends \\u03C0{});");
}

#[test]
fn test_jsx() {
    test("<a/>", "<a />;\n");
    test("<A/>", "<A />;\n");
    test("<a.b/>", "<a.b />;\n");
    test("<A.B/>", "<A.B />;\n");
    test("<a-b/>", "<a-b />;\n");
    test("<a:b/>", "<a:b />;\n");
    test("<a></a>", "<a></a>;\n");
    test("<a b></a>", "<a b></a>;\n");

    test("<a b={true}></a>", "<a b={true}></a>;\n");
    test("<a b='x'></a>", "<a b=\"x\"></a>;\n");
    test("<a b=\"x\"></a>", "<a b=\"x\"></a>;\n");
    test("<a b={'x'}></a>", "<a b={\"x\"}></a>;\n");
    test("<a b={`'`}></a>", "<a b={`'`}></a>;\n");
    test("<a b={`\"`}></a>", "<a b={`\"`}></a>;\n");
    test("<a b={`'\"`}></a>", "<a b={`'\"`}></a>;\n");
    test("<a b=\"&quot;\"></a>", "<a b=\"&quot;\"></a>;\n");
    test("<a b=\"&amp;\"></a>", "<a b=\"&amp;\"></a>;\n");

    test("<a>x</a>", "<a>x</a>;\n");
    test("<a>x\ny</a>", "<a>x\ny</a>;\n");
    test("<a>{'x'}{'y'}</a>", "<a>{\"x\"}{\"y\"}</a>;\n");
    test("<a> x</a>", "<a> x</a>;\n");
    test("<a>x </a>", "<a>x </a>;\n");
    test("<a>&#10;</a>", "<a>&#10;</a>;\n");
    test("<a>&amp;</a>", "<a>&amp;</a>;\n");
    test("<a>&lt;</a>", "<a>&lt;</a>;\n");
    test("<a>&gt;</a>", "<a>&gt;</a>;\n");
    test("<a>&#123;</a>", "<a>&#123;</a>;\n");
    test("<a>&#125;</a>", "<a>&#125;</a>;\n");

    test("<a><x/></a>", "<a><x /></a>;\n");
    test("<a><x/><y/></a>", "<a><x /><y /></a>;\n");
    test("<a>b<c/>d</a>", "<a>b<c />d</a>;\n");

    test("<></>", "<></>;\n");
    test("<>x<y/>z</>", "<>x<y />z</>;\n");

    // JSX elements as JSX attribute values
    test("<a b=<c/>/>", "<a b=<c /> />;\n");
    test("<a b=<>c</>/>", "<a b=<>c</> />;\n");
    test("<a b=<>{c}</>/>", "<a b=<>{c}</> />;\n");
    test("<a b={<c/>}/>", "<a b={<c />} />;\n");
    test("<a b={<>c</>}/>", "<a b={<>c</>} />;\n");
    test("<a b={<>{c}</>}/>", "<a b={<>{c}</>} />;\n");

    // These can't be escaped because JSX lacks a syntax for escapes
    // testJSXASCII(t, "<π/>", "<π />;\n");
    // testJSXASCII(t, "<π.𐀀/>", "<π.𐀀 />;\n");
    // testJSXASCII(t, "<𐀀.π/>", "<𐀀.π />;\n");
    // testJSXASCII(t, "<π>x</π>", "<π>x</π>;\n");
    // testJSXASCII(t, "<𐀀>x</𐀀>", "<𐀀>x</𐀀>;\n");
    // testJSXASCII(t, "<a π/>", "<a π />;\n");
    // testJSXASCII(t, "<a 𐀀/>", "<a 𐀀 />;\n");

    // JSX text is deliberately not printed as ASCII when JSX preservation is
    // enabled. This is because:
    //
    // a) The JSX specification doesn't say how JSX text is supposed to be interpreted
    // b) Enabling JSX preservation means that JSX will be transformed again anyway
    // c) People do very weird/custom things with JSX that "preserve" shouldn't break
    //
    // See also: https://github.com/evanw/esbuild/issues/3605
    // testJSXASCII(t, "<a b='π'/>", "<a b='π' />;\n");
    // testJSXASCII(t, "<a b='𐀀'/>", "<a b='𐀀' />;\n");
    // testJSXASCII(t, "<a>π</a>", "<a>π</a>;\n");
    // testJSXASCII(t, "<a>𐀀</a>", "<a>𐀀</a>;\n");

    // testJSXMinify(t, "<a b c={x,y} d='true'/>", "<a b c={(x,y)}d='true'/>;");
    // testJSXMinify(t, "<a><b/><c/></a>", "<a><b/><c/></a>;");
    // testJSXMinify(t, "<a> x <b/> y </a>", "<a> x <b/> y </a>;");
    // testJSXMinify(t, "<a>{' x '}{'<b/>'}{' y '}</a>", "<a>{\" x \"}{\"<b/>\"}{\" y \"}</a>;");
}

#[test]
fn test_jsx_single_line() {
    test("<x/>", "<x />;\n");
    test("<x y/>", "<x y />;\n");
    test("<x\n/>", "<x />;\n");
    // test("<x\ny/>", "<x\n\ty\n/>;\n");
    // test("<x y\n/>", "<x\n\ty\n/>;\n");
    // test("<x\n{...y}/>", "<x\n\t{...y}\n/>;\n");

    test_minify("<x/>", "<x/>;");
    test_minify("<x y/>", "<x y/>;");
    test_minify("<x\n/>", "<x/>;");
    test_minify("<x\ny/>", "<x y/>;");
    test_minify("<x y\n/>", "<x y/>;");
    test_minify("<x\n{...y}/>", "<x{...y}/>;");
}

#[test]
#[ignore]
fn test_avoid_slash_script() {
    // Positive cases
    test("x = '</script'", "x = \"<\\/script\";\n");
    test("x = `</script`", "x = `<\\/script`;\n");
    test("x = `</SCRIPT`", "x = `<\\/SCRIPT`;\n");
    test("x = `</ScRiPt`", "x = `<\\/ScRiPt`;\n");
    test("x = `</script${y}`", "x = `<\\/script${y}`;\n");
    test("x = `${y}</script`", "x = `${y}<\\/script`;\n");
    test_minify("x = 1 < /script/.exec(y).length", "x=1< /script/.exec(y).length;");
    test_minify("x = 1 < /SCRIPT/.exec(y).length", "x=1< /SCRIPT/.exec(y).length;");
    test_minify("x = 1 < /ScRiPt/.exec(y).length", "x=1< /ScRiPt/.exec(y).length;");
    test_minify("x = 1 << /script/.exec(y).length", "x=1<< /script/.exec(y).length;");
    test("//! </script\n//! >/script\n//! /script", "//! <\\/script\n//! >/script\n//! /script\n");
    test("//! </SCRIPT\n//! >/SCRIPT\n//! /SCRIPT", "//! <\\/SCRIPT\n//! >/SCRIPT\n//! /SCRIPT\n");
    test("//! </ScRiPt\n//! >/ScRiPt\n//! /ScRiPt", "//! <\\/ScRiPt\n//! >/ScRiPt\n//! /ScRiPt\n");
    test("/*! </script \n </script */", "/*! <\\/script \n <\\/script */\n");
    test("/*! </SCRIPT \n </SCRIPT */", "/*! <\\/SCRIPT \n <\\/SCRIPT */\n");
    test("/*! </ScRiPt \n </ScRiPt */", "/*! <\\/ScRiPt \n <\\/ScRiPt */\n");
    test(
        "String.raw`</script`",
        "import { __template } from \"<runtime>\";\nvar _a;\nString.raw(_a || (_a = __template([\"<\\/script\"])));\n",
    );
    test(
        "String.raw`</script${a}`",
        "import { __template } from \"<runtime>\";\nvar _a;\nString.raw(_a || (_a = __template([\"<\\/script\", \"\"])), a);\n",
    );
    test(
        "String.raw`${a}</script`",
        "import { __template } from \"<runtime>\";\nvar _a;\nString.raw(_a || (_a = __template([\"\", \"<\\/script\"])), a);\n",
    );
    test(
        "String.raw`</SCRIPT`",
        "import { __template } from \"<runtime>\";\nvar _a;\nString.raw(_a || (_a = __template([\"<\\/SCRIPT\"])));\n",
    );
    test(
        "String.raw`</ScRiPt`",
        "import { __template } from \"<runtime>\";\nvar _a;\nString.raw(_a || (_a = __template([\"<\\/ScRiPt\"])));\n",
    );

    // Negative cases
    test("x = '</'", "x = \"</\";\n");
    test("x = '</ script'", "x = \"</ script\";\n");
    test("x = '< /script'", "x = \"< /script\";\n");
    test("x = '/script>'", "x = \"/script>\";\n");
    test("x = '<script>'", "x = \"<script>\";\n");
    test_minify("x = 1 < / script/.exec(y).length", "x=1</ script/.exec(y).length;");
    test_minify("x = 1 << / script/.exec(y).length", "x=1<</ script/.exec(y).length;");
}

#[test]
fn test_infinity() {
    test("x = Infinity", "x = Infinity;\n");
    test("x = -Infinity", "x = -Infinity;\n");
    test("x = (Infinity).toString", "x = Infinity.toString;\n");
    test("x = (-Infinity).toString", "x = (-Infinity).toString;\n");
    test("x = (Infinity) ** 2", "x = Infinity ** 2;\n");
    test("x = (-Infinity) ** 2", "x = (-Infinity) ** 2;\n");
    test("x = ~Infinity", "x = ~Infinity;\n");
    test("x = ~-Infinity", "x = ~-Infinity;\n");
    test("x = Infinity * y", "x = Infinity * y;\n");
    test("x = Infinity / y", "x = Infinity / y;\n");
    test("x = y * Infinity", "x = y * Infinity;\n");
    test("x = y / Infinity", "x = y / Infinity;\n");
    test("throw Infinity", "throw Infinity;\n");

    test_minify("x = Infinity", "x=Infinity;");
    test_minify("x = -Infinity", "x=-Infinity;");
    test_minify("x = (Infinity).toString", "x=Infinity.toString;");
    test_minify("x = (-Infinity).toString", "x=(-Infinity).toString;");
    test_minify("x = (Infinity) ** 2", "x=Infinity**2;");
    test_minify("x = (-Infinity) ** 2", "x=(-Infinity)**2;");
    test_minify("x = ~Infinity", "x=~Infinity;");
    test_minify("x = ~-Infinity", "x=~-Infinity;");
    test_minify("x = Infinity * y", "x=Infinity*y;");
    test_minify("x = Infinity / y", "x=Infinity/y;");
    test_minify("x = y * Infinity", "x=y*Infinity;");
    test_minify("x = y / Infinity", "x=y/Infinity;");
    test_minify("throw Infinity", "throw Infinity;");

    // testMangle(t, "x = Infinity", "x = 1 / 0;\n");
    // testMangle(t, "x = -Infinity", "x = -1 / 0;\n");
    // testMangle(t, "x = (Infinity).toString", "x = (1 / 0).toString;\n");
    // testMangle(t, "x = (-Infinity).toString", "x = (-1 / 0).toString;\n");
    // testMangle(t, "x = Infinity ** 2", "x = (1 / 0) ** 2;\n");
    // testMangle(t, "x = (-Infinity) ** 2", "x = (-1 / 0) ** 2;\n");
    // testMangle(t, "x = Infinity * y", "x = 1 / 0 * y;\n");
    // testMangle(t, "x = Infinity / y", "x = 1 / 0 / y;\n");
    // testMangle(t, "x = y * Infinity", "x = y * (1 / 0);\n");
    // testMangle(t, "x = y / Infinity", "x = y / (1 / 0);\n");
    // testMangle(t, "throw Infinity", "throw 1 / 0;\n");

    // testMangleMinify(t, "x = Infinity", "x=1/0;");
    // testMangleMinify(t, "x = -Infinity", "x=-1/0;");
    // testMangleMinify(t, "x = (Infinity).toString", "x=(1/0).toString;");
    // testMangleMinify(t, "x = (-Infinity).toString", "x=(-1/0).toString;");
    // testMangleMinify(t, "x = Infinity ** 2", "x=(1/0)**2;");
    // testMangleMinify(t, "x = (-Infinity) ** 2", "x=(-1/0)**2;");
    // testMangleMinify(t, "x = Infinity * y", "x=1/0*y;");
    // testMangleMinify(t, "x = Infinity / y", "x=1/0/y;");
    // testMangleMinify(t, "x = y * Infinity", "x=y*(1/0);");
    // testMangleMinify(t, "x = y / Infinity", "x=y/(1/0);");
    // testMangleMinify(t, "throw Infinity", "throw 1/0;");
}

#[test]
fn test_binary_operator_visitor() {
    // Make sure the inner "/*b*/" comment doesn't disappear due to weird binary visitor stuff
    // testMangle(t, "x = (0, /*a*/ (0, /*b*/ (0, /*c*/ 1 == 2) + 3) * 4)", "x = /*a*/\n/*b*/\n(/*c*/\n!1 + 3) * 4;\n");

    // Make sure deeply-nested ASTs don't cause a stack overflow
    let x = format!("x = f(){};\n", " + f()".repeat(10_000));
    test(&x, &x);

    let x = format!("x = f(){};\n", " && f()".repeat(10_000));
    test(&x, &x);

    let x = format!("x = f(){};\n", " && f() + f()".repeat(10_000));
    test(&x, &x);
}

// See: https://github.com/tc39/proposal-explicit-resource-management
#[test]
fn test_using() {
    test("using x = y", "using x = y;\n");
    test("using x = y, z = _", "using x = y, z = _;\n");
    test_minify("using x = y", "using x=y;");
    test_minify("using x = y, z = _", "using x=y,z=_;");

    test("await using x = y", "await using x = y;\n");
    test("await using x = y, z = _", "await using x = y, z = _;\n");
    test_minify("await using x = y", "await using x=y;");
    test_minify("await using x = y, z = _", "await using x=y,z=_;");
}

#[test]
fn test_preserve_optional_chain_parentheses() {
    test("a?.b.c", "a?.b.c;\n");
    test("(a?.b).c", "(a?.b).c;\n");
    test("a?.b.c.d", "a?.b.c.d;\n");
    test("(a?.b.c).d", "(a?.b.c).d;\n");
    test("a?.b[c]", "a?.b[c];\n");
    test("(a?.b)[c]", "(a?.b)[c];\n");
    test("a?.b(c)", "a?.b(c);\n");
    test("(a?.b)(c)", "(a?.b)(c);\n");

    test("a?.[b][c]", "a?.[b][c];\n");
    test("(a?.[b])[c]", "(a?.[b])[c];\n");
    test("a?.[b][c][d]", "a?.[b][c][d];\n");
    test("(a?.[b][c])[d]", "(a?.[b][c])[d];\n");
    test("a?.[b].c", "a?.[b].c;\n");
    test("(a?.[b]).c", "(a?.[b]).c;\n");
    test("a?.[b](c)", "a?.[b](c);\n");
    test("(a?.[b])(c)", "(a?.[b])(c);\n");

    test("a?.(b)(c)", "a?.(b)(c);\n");
    test("(a?.(b))(c)", "(a?.(b))(c);\n");
    test("a?.(b)(c)(d)", "a?.(b)(c)(d);\n");
    test("(a?.(b)(c))(d)", "(a?.(b)(c))(d);\n");
    test("a?.(b).c", "a?.(b).c;\n");
    test("(a?.(b)).c", "(a?.(b)).c;\n");
    test("a?.(b)[c]", "a?.(b)[c];\n");
    test("(a?.(b))[c]", "(a?.(b))[c];\n");
}
