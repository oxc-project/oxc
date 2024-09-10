//! <https://github.com/google/closure-compiler/blob/master/test/com/google/javascript/jscomp/PeepholeFoldConstantsTest.java>

use crate::CompressOptions;

fn test(source_text: &str, expected: &str) {
    let options = CompressOptions {
        remove_syntax: true,
        fold_constants: true,
        ..CompressOptions::all_false()
    };
    crate::test(source_text, expected, options);
}

fn test_same(source_text: &str) {
    test(source_text, source_text);
}

// Oxc

#[test]
fn cjs() {
    // Bail `cjs-module-lexer`.
    test_same("0 && (module.exports = { version });");
}

#[test] // https://github.com/oxc-project/oxc/issues/4341
fn tagged_template() {
    test_same("(1, o.f)()");
    test_same("(1, o.f)``");

    test("(true && o.f)()", "o.f()");
    test_same("(true && o.f)``");

    test("(true ? o.f : false)()", "o.f()");
    test_same("(true ? o.f : false)``");
}

// Google Closure Compiler

#[test]
fn undefined_comparison1() {
    test("undefined == undefined", "true");
    test("undefined == null", "true");
    test("undefined == void 0", "true");

    test("undefined == 0", "false");
    test("undefined == 1", "false");
    test("undefined == 'hi'", "false");
    test("undefined == true", "false");
    test("undefined == false", "false");

    test("undefined === undefined", "true");
    test("undefined === null", "false");
    test("undefined === void 0", "true");

    test_same("undefined == this");
    test_same("undefined == x");

    test("undefined != undefined", "false");
    test("undefined != null", "false");
    test("undefined != void 0", "false");

    test("undefined != 0", "true");
    test("undefined != 1", "true");
    test("undefined != 'hi'", "true");
    test("undefined != true", "true");
    test("undefined != false", "true");

    test("undefined !== undefined", "false");
    test("undefined !== void 0", "false");
    test("undefined !== null", "true");

    test_same("undefined != this");
    test_same("undefined != x");

    test("undefined < undefined", "false");
    test("undefined > undefined", "false");
    test("undefined >= undefined", "false");
    test("undefined <= undefined", "false");

    test("0 < undefined", "false");
    test("true > undefined", "false");
    test("'hi' >= undefined", "false");
    test("null <= undefined", "false");

    test("undefined < 0", "false");
    test("undefined > true", "false");
    test("undefined >= 'hi'", "false");
    test("undefined <= null", "false");

    test("null == undefined", "true");
    test("0 == undefined", "false");
    test("1 == undefined", "false");
    test("'hi' == undefined", "false");
    test("true == undefined", "false");
    test("false == undefined", "false");
    test("null === undefined", "false");
    test("void 0 === undefined", "true");

    test("undefined == NaN", "false");
    test("NaN == undefined", "false");
    test("undefined == Infinity", "false");
    test("Infinity == undefined", "false");
    test("undefined == -Infinity", "false");
    test("-Infinity == undefined", "false");
    test("({}) == undefined", "false");
    test("undefined == ({})", "false");
    test("([]) == undefined", "false");
    test("undefined == ([])", "false");
    test("(/a/g) == undefined", "false");
    test("undefined == (/a/g)", "false");
    test("(function(){}) == undefined", "false");
    test("undefined == (function(){})", "false");

    test("undefined != NaN", "true");
    test("NaN != undefined", "true");
    test("undefined != Infinity", "true");
    test("Infinity != undefined", "true");
    test("undefined != -Infinity", "true");
    test("-Infinity != undefined", "true");
    test("({}) != undefined", "true");
    test("undefined != ({})", "true");
    test("([]) != undefined", "true");
    test("undefined != ([])", "true");
    test("(/a/g) != undefined", "true");
    test("undefined != (/a/g)", "true");
    test("(function(){}) != undefined", "true");
    test("undefined != (function(){})", "true");

    test_same("this == undefined");
    test_same("x == undefined");
}

#[test]
fn test_undefined_comparison2() {
    test("\"123\" !== void 0", "true");
    test("\"123\" === void 0", "false");

    test("void 0 !== \"123\"", "true");
    test("void 0 === \"123\"", "false");
}

#[test]
fn test_undefined_comparison3() {
    test("\"123\" !== undefined", "true");
    test("\"123\" === undefined", "false");

    test("undefined !== \"123\"", "true");
    test("undefined === \"123\"", "false");
}

#[test]
fn test_null_comparison1() {
    test("null == undefined", "true");
    test("null == null", "true");
    test("null == void 0", "true");

    test("null == 0", "false");
    test("null == 1", "false");
    // test("null == 0n", "false");
    // test("null == 1n", "false");
    test("null == 'hi'", "false");
    test("null == true", "false");
    test("null == false", "false");

    test("null === undefined", "false");
    test("null === null", "true");
    test("null === void 0", "false");
    test_same("null===x");

    test_same("null==this");
    test_same("null==x");

    test("null != undefined", "false");
    test("null != null", "false");
    test("null != void 0", "false");

    test("null != 0", "true");
    test("null != 1", "true");
    // test("null != 0n", "true");
    // test("null != 1n", "true");
    test("null != 'hi'", "true");
    test("null != true", "true");
    test("null != false", "true");

    test("null !== undefined", "true");
    test("null !== void 0", "true");
    test("null !== null", "false");

    test_same("null!=this");
    test_same("null!=x");

    test("null < null", "false");
    test("null > null", "false");
    test("null >= null", "true");
    test("null <= null", "true");

    test("0 < null", "false");
    test("0 > null", "false");
    test("0 >= null", "true");
    // test("0n < null", "false");
    // test("0n > null", "false");
    // test("0n >= null", "true");
    test("true > null", "true");
    test("'hi' < null", "false");
    test("'hi' >= null", "false");
    test("null <= null", "true");

    test("null < 0", "false");
    // test("null < 0n", "false");
    test("null > true", "false");
    test("null < 'hi'", "false");
    test("null >= 'hi'", "false");
    test("null <= null", "true");

    test("null == null", "true");
    test("0 == null", "false");
    test("1 == null", "false");
    test("'hi' == null", "false");
    test("true == null", "false");
    test("false == null", "false");
    test("null === null", "true");
    test("void 0 === null", "false");

    test("null == NaN", "false");
    test("NaN == null", "false");
    test("null == Infinity", "false");
    test("Infinity == null", "false");
    test("null == -Infinity", "false");
    test("-Infinity == null", "false");
    test("({}) == null", "false");
    test("null == ({})", "false");
    test("([]) == null", "false");
    test("null == ([])", "false");
    test("(/a/g) == null", "false");
    test("null == (/a/g)", "false");
    test("(function(){}) == null", "false");
    test("null == (function(){})", "false");

    test("null != NaN", "true");
    test("NaN != null", "true");
    test("null != Infinity", "true");
    test("Infinity != null", "true");
    test("null != -Infinity", "true");
    test("-Infinity != null", "true");
    test("({}) != null", "true");
    test("null != ({})", "true");
    test("([]) != null", "true");
    test("null != ([])", "true");
    test("(/a/g) != null", "true");
    test("null != (/a/g)", "true");
    test("(function(){}) != null", "true");
    test("null != (function(){})", "true");

    test_same("({a:f()})==null");
    test_same("null=={a:f()}");
    test_same("[f()]==null");
    test_same("null==[f()]");

    test_same("this==null");
    test_same("x==null");
}

#[test]
fn test_boolean_boolean_comparison() {
    test_same("!x==!y");
    test_same("!x<!y");
    test_same("!x!==!y");

    test_same("!x==!x"); // foldable
    test_same("!x<!x"); // foldable
    test_same("!x!==!x"); // foldable
}

#[test]
fn test_boolean_number_comparison() {
    test_same("!x==+y");
    test_same("!x<=+y");
    test("!x !== +y", "true");
}

#[test]
fn test_number_boolean_comparison() {
    test_same("+x==!y");
    test_same("+x<=!y");
    test("+x === !y", "false");
}

#[test]
fn test_boolean_string_comparison() {
    test_same("!x==''+y");
    test_same("!x<=''+y");
    test("!x !== '' + y", "true");
}

#[test]
fn test_string_boolean_comparison() {
    test_same("''+x==!y");
    test_same("''+x<=!y");
    test("'' + x === !y", "false");
}

#[test]
#[ignore]
fn test_string_string_comparison() {
    test("'a' < 'b'", "true");
    test("'a' <= 'b'", "true");
    test("'a' > 'b'", "false");
    test("'a' >= 'b'", "false");
    test("+'a' < +'b'", "false");
    test_same("typeof a < 'a'");
    test_same("'a' >= typeof a");
    test("typeof a < typeof a", "false");
    test("typeof a >= typeof a", "true");
    test("typeof 3 > typeof 4", "false");
    test("typeof function() {} < typeof function() {}", "false");
    test("'a' == 'a'", "true");
    test("'b' != 'a'", "true");
    test_same("'undefined' == typeof a");
    test_same("typeof a != 'number'");
    test_same("'undefined' == typeof a");
    test_same("'undefined' == typeof a");
    test("typeof a == typeof a", "true");
    test("'a' === 'a'", "true");
    test("'b' !== 'a'", "true");
    test("typeof a === typeof a", "true");
    test("typeof a !== typeof a", "false");
    test_same("'' + x <= '' + y");
    test_same("'' + x != '' + y");
    test_same("'' + x === '' + y");

    test_same("'' + x <= '' + x"); // potentially foldable
    test_same("'' + x != '' + x"); // potentially foldable
    test_same("'' + x === '' + x"); // potentially foldable
}

#[test]
fn test_number_string_comparison() {
    test("1 < '2'", "true");
    test("2 > '1'", "true");
    test("123 > '34'", "true");
    test("NaN >= 'NaN'", "false");
    test("1 == '2'", "false");
    test("1 != '1'", "false");
    test("NaN == 'NaN'", "false");
    test("1 === '1'", "false");
    test("1 !== '1'", "true");
    test_same("+x>''+y");
    test_same("+x==''+y");
    test("+x !== '' + y", "true");
}

#[test]
fn test_string_number_comparison() {
    test("'1' < 2", "true");
    test("'2' > 1", "true");
    test("'123' > 34", "true");
    test("'NaN' < NaN", "false");
    test("'1' == 2", "false");
    test("'1' != 1", "false");
    test("'NaN' == NaN", "false");
    test("'1' === 1", "false");
    test("'1' !== 1", "true");
    test_same("''+x<+y");
    test_same("''+x==+y");
    test("'' + x === +y", "false");
}

#[test]
#[ignore]
fn test_bigint_number_comparison() {
    test("1n < 2", "true");
    test("1n > 2", "false");
    test("1n == 1", "true");
    test("1n == 2", "false");

    // comparing with decimals is allowed
    test("1n < 1.1", "true");
    test("1n < 1.9", "true");
    test("1n < 0.9", "false");
    test("-1n < -1.1", "false");
    test("-1n < -1.9", "false");
    test("-1n < -0.9", "true");
    test("1n > 1.1", "false");
    test("1n > 0.9", "true");
    test("-1n > -1.1", "true");
    test("-1n > -0.9", "false");

    // Don't fold unsafely large numbers because there might be floating-point error
    let max_safe_int = 9_007_199_254_740_991_i64;
    let neg_max_safe_int = -9_007_199_254_740_991_i64;
    let max_safe_float = 9_007_199_254_740_991_f64;
    let neg_max_safe_float = -9_007_199_254_740_991_f64;
    test(&format!("0n > {max_safe_int}"), "false");
    test(&format!("0n < {max_safe_int}"), "true");
    test(&format!("0n > {neg_max_safe_int}"), "true");
    test(&format!("0n < {neg_max_safe_int}"), "false");
    test(&format!("0n > {max_safe_float}"), "false");
    test(&format!("0n < {max_safe_float}"), "true");
    test(&format!("0n > {neg_max_safe_float}"), "true");
    test(&format!("0n < {neg_max_safe_float}"), "false");

    // comparing with Infinity is allowed
    test("1n < Infinity", "true");
    test("1n > Infinity", "false");
    test("1n < -Infinity", "false");
    test("1n > -Infinity", "true");

    // null is interpreted as 0 when comparing with bigint
    test("1n < null", "false");
    test("1n > null", "true");
}

#[test]
#[ignore]
fn test_bigint_string_comparison() {
    test("1n < '2'", "true");
    test("2n > '1'", "true");
    test("123n > '34'", "true");
    test("1n == '1'", "true");
    test("1n == '2'", "false");
    test("1n != '1'", "false");
    test("1n === '1'", "false");
    test("1n !== '1'", "true");
}

#[test]
#[ignore]
fn test_string_bigint_comparison() {
    test("'1' < 2n", "true");
    test("'2' > 1n", "true");
    test("'123' > 34n", "true");
    test("'1' == 1n", "true");
    test("'1' == 2n", "false");
    test("'1' != 1n", "false");
    test("'1' === 1n", "false");
    test("'1' !== 1n", "true");
}

#[test]
fn test_nan_comparison() {
    test("NaN < 1", "false");
    test("NaN <= 1", "false");
    test("NaN > 1", "false");
    test("NaN >= 1", "false");
    // test("NaN < 1n", "false");
    // test("NaN <= 1n", "false");
    // test("NaN > 1n", "false");
    // test("NaN >= 1n", "false");

    test("NaN < NaN", "false");
    test("NaN >= NaN", "false");
    test("NaN == NaN", "false");
    test("NaN === NaN", "false");

    test("NaN < null", "false");
    test("null >= NaN", "false");
    test("NaN == null", "false");
    test("null != NaN", "true");
    test("null === NaN", "false");

    test("NaN < undefined", "false");
    test("undefined >= NaN", "false");
    test("NaN == undefined", "false");
    test("undefined != NaN", "true");
    test("undefined === NaN", "false");

    test_same("NaN<x");
    test_same("x>=NaN");
    test_same("NaN==x");
    test_same("x!=NaN");
    test("NaN === x", "false");
    test("x !== NaN", "true");
    test_same("NaN==foo()");
}

#[test]
fn fold_typeof() {
    test("x = typeof 1", "x = \"number\"");
    test("x = typeof 'foo'", "x = \"string\"");
    test("x = typeof true", "x = \"boolean\"");
    test("x = typeof false", "x = \"boolean\"");
    test("x = typeof null", "x = \"object\"");
    test("x = typeof undefined", "x = \"undefined\"");
    test("x = typeof void 0", "x = \"undefined\"");
    test("x = typeof []", "x = \"object\"");
    test("x = typeof [1]", "x = \"object\"");
    test("x = typeof [1,[]]", "x = \"object\"");
    test("x = typeof {}", "x = \"object\"");
    test("x = typeof function() {}", "x = 'function'");

    test_same("x = typeof[1,[foo()]]");
    test_same("x = typeof{bathwater:baby()}");
}

#[test]
#[ignore]
fn unary_ops() {
    // TODO: need to port
    // These cases are handled by PeepholeRemoveDeadCode in closure-compiler.
    // test_same("!foo()");
    // test_same("~foo()");
    // test_same("-foo()");

    // These cases are handled here.
    test("a=!true", "a=false");
    test("a=!10", "a=false");
    test("a=!false", "a=true");
    test_same("a=!foo()");
    test("a=-0", "a=-0.0");
    test("a=-(0)", "a=-0.0");
    test_same("a=-Infinity");
    test("a=-NaN", "a=NaN");
    test_same("a=-foo()");
    test("a=~~0", "a=0");
    test("a=~~10", "a=10");
    test("a=~-7", "a=6");

    test("a=+true", "a=1");
    test("a=+10", "a=10");
    test("a=+false", "a=0");
    test_same("a=+foo()");
    test_same("a=+f");
    test("a=+(f?true:false)", "a=+(f?1:0)");
    test("a=+0", "a=0");
    test("a=+Infinity", "a=Infinity");
    test("a=+NaN", "a=NaN");
    test("a=+-7", "a=-7");
    test("a=+.5", "a=.5");

    test("a=~0xffffffff", "a=0");
    test("a=~~0xffffffff", "a=-1");
    // test_same("a=~.5", PeepholeFoldConstants.FRACTIONAL_BITWISE_OPERAND);
}

#[test]
#[ignore]
fn unary_with_big_int() {
    test("-(1n)", "-1n");
    test("- -1n", "1n");
    test("!1n", "false");
    test("~0n", "-1n");
}

#[test]
#[ignore]
fn test_unary_ops_string_compare() {
    test_same("a = -1");
    test("a = ~0", "a = -1");
    test("a = ~1", "a = -2");
    test("a = ~101", "a = -102");
}

#[test]
fn test_fold_logical_op() {
    test("x = true && x", "x = x");
    test("x = [foo()] && x", "x = ([foo()],x)");

    test("x = false && x", "x = false");
    test("x = true || x", "x = true");
    test("x = false || x", "x = x");
    test("x = 0 && x", "x = 0");
    test("x = 3 || x", "x = 3");
    test("x = 0n && x", "x = 0n");
    test("x = 3n || x", "x = 3n");
    test("x = false || 0", "x = 0");

    // unfoldable, because the right-side may be the result
    test("a = x && true", "a=x && true");
    test("a = x && false", "a=x && false");
    test("a = x || 3", "a=x || 3");
    test("a = x || false", "a=x || false");
    test("a = b ? c : x || false", "a=b ? c:x || false");
    test("a = b ? x || false : c", "a=b ? x || false:c");
    test("a = b ? c : x && true", "a=b ? c:x && true");
    test("a = b ? x && true : c", "a=b ? x && true:c");

    // folded, but not here.
    test_same("a = x || false ? b : c");
    test_same("a = x && true ? b : c");

    test("x = foo() || true || bar()", "x = foo() || true");
    test("x = foo() || true && bar()", "x = foo() || bar()");
    test("x = foo() || false && bar()", "x = foo() || false");
    test("x = foo() && false && bar()", "x = foo() && false");
    test("x = foo() && false || bar()", "x = (foo() && false,bar())");
    test("x = foo() || false || bar()", "x = foo() || bar()");
    test("x = foo() && true && bar()", "x = foo() && bar()");
    test("x = foo() || true || bar()", "x = foo() || true");
    test("x = foo() && false && bar()", "x = foo() && false");
    test("x = foo() && 0 && bar()", "x = foo() && 0");
    test("x = foo() && 1 && bar()", "x = foo() && bar()");
    test("x = foo() || 0 || bar()", "x = foo() || bar()");
    test("x = foo() || 1 || bar()", "x = foo() || 1");
    test("x = foo() && 0n && bar()", "x = foo() && 0n");
    test("x = foo() && 1n && bar()", "x = foo() && bar()");
    test("x = foo() || 0n || bar()", "x = foo() || bar()");
    test("x = foo() || 1n || bar()", "x = foo() || 1n");
    test_same("x = foo() || bar() || baz()");
    test_same("x = foo() && bar() && baz()");

    test("0 || b()", "b()");
    test("1 && b()", "b()");
    test("a() && (1 && b())", "a() && b()");
    test("(a() && 1) && b()", "a() && b()");

    test("(x || '') || y;", "x || y");
    test("false || (x || '');", "x || ''");
    test("(x && 1) && y;", "x && y");
    test("true && (x && 1);", "x && 1");

    // Really not foldable, because it would change the type of the
    // expression if foo() returns something truthy but not true.
    // Cf. FoldConstants.tryFoldAndOr().
    // An example would be if foo() is 1 (truthy) and bar() is 0 (falsey):
    // (1 && true) || 0 == true
    // 1 || 0 == 1, but true =/= 1
    test_same("x = foo() && true || bar()");
    test_same("foo() && true || bar()");
}

#[test]
fn test_fold_logical_op2() {
    test("x = function(){} && x", "x=x");
    test("x = true && function(){}", "x=function(){}");
    test("x = [(function(){alert(x)})()] && x", "x=([function(){alert(x)}()],x)");
}

#[test]
#[ignore]
fn test_fold_void() {
    test_same("void 0");
    test("void 1", "void 0");
    test("void x", "void 0");
    test_same("void x()");
}

#[test]
fn test_fold_bit_shift() {
    test("x = 1 << 0", "x=1");
    test("x = -1 << 0", "x=-1");
    test("x = 1 << 1", "x=2");
    test("x = 3 << 1", "x=6");
    test("x = 1 << 8", "x=256");

    test("x = 1 >> 0", "x=1");
    test("x = -1 >> 0", "x=-1");
    test("x = 1 >> 1", "x=0");
    test("x = 2 >> 1", "x=1");
    test("x = 5 >> 1", "x=2");
    test("x = 127 >> 3", "x=15");
    test("x = 3 >> 1", "x=1");
    test("x = 3 >> 2", "x=0");
    test("x = 10 >> 1", "x=5");
    test("x = 10 >> 2", "x=2");
    test("x = 10 >> 5", "x=0");

    test("x = 10 >>> 1", "x=5");
    test("x = 10 >>> 2", "x=2");
    test("x = 10 >>> 5", "x=0");
    test("x = -1 >>> 1", "x=2147483647"); // 0x7fffffff
    test("x = -1 >>> 0", "x=4294967295"); // 0xffffffff
    test("x = -2 >>> 0", "x=4294967294"); // 0xfffffffe
    test("x = 0x90000000 >>> 28", "x=9");

    test("x = 0xffffffff << 0", "x=-1");
    test("x = 0xffffffff << 4", "x=-16");
    test("1 << 32", "1<<32");
    test("1 << -1", "1<<-1");
    test("1 >> 32", "1>>32");
}
