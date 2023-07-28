//! <https://github.com/google/closure-compiler/blob/master/test/com/google/javascript/jscomp/PeepholeFoldConstantsTest.java>

use crate::{test, test_same, test_without_compress_booleans as test_wcb};

#[test]
fn undefined_comparison1() {
    test("undefined == undefined", "!0");
    test("undefined == null", "!0");
    test("undefined == void 0", "!0");

    test("undefined == 0", "!1");
    test("undefined == 1", "!1");
    test("undefined == 'hi'", "!1");
    test("undefined == true", "!1");
    test("undefined == false", "!1");

    test("undefined === undefined", "!0");
    test("undefined === null", "!1");
    test("undefined === void 0", "!0");

    // origin was `test_same("undefined == this");`
    test("undefined == this", "void 0==this");
    // origin was `test_same("undefined == x");`
    test("undefined == x", "void 0==x");

    test("undefined != undefined", "!1");
    test("undefined != null", "!1");
    test("undefined != void 0", "!1");

    test("undefined != 0", "!0");
    test("undefined != 1", "!0");
    test("undefined != 'hi'", "!0");
    test("undefined != true", "!0");
    test("undefined != false", "!0");

    test("undefined !== undefined", "!1");
    test("undefined !== void 0", "!1");
    test("undefined !== null", "!0");

    // origin was `test_same("undefined != this");`
    test("undefined != this", "void 0!=this");
    // origin was `test_same("undefined != x");`
    test("undefined != x", "void 0!=x");

    test("undefined < undefined", "!1");
    test("undefined > undefined", "!1");
    test("undefined >= undefined", "!1");
    test("undefined <= undefined", "!1");

    test("0 < undefined", "!1");
    test("true > undefined", "!1");
    test("'hi' >= undefined", "!1");
    test("null <= undefined", "!1");

    test("undefined < 0", "!1");
    test("undefined > true", "!1");
    test("undefined >= 'hi'", "!1");
    test("undefined <= null", "!1");

    test("null == undefined", "!0");
    test("0 == undefined", "!1");
    test("1 == undefined", "!1");
    test("'hi' == undefined", "!1");
    test("true == undefined", "!1");
    test("false == undefined", "!1");
    test("null === undefined", "!1");
    test("void 0 === undefined", "!0");

    test("undefined == NaN", "!1");
    test("NaN == undefined", "!1");
    test("undefined == Infinity", "!1");
    test("Infinity == undefined", "!1");
    test("undefined == -Infinity", "!1");
    test("-Infinity == undefined", "!1");
    test("({}) == undefined", "!1");
    test("undefined == ({})", "!1");
    test("([]) == undefined", "!1");
    test("undefined == ([])", "!1");
    test("(/a/g) == undefined", "!1");
    test("undefined == (/a/g)", "!1");
    test("(function(){}) == undefined", "!1");
    test("undefined == (function(){})", "!1");

    test("undefined != NaN", "!0");
    test("NaN != undefined", "!0");
    test("undefined != Infinity", "!0");
    test("Infinity != undefined", "!0");
    test("undefined != -Infinity", "!0");
    test("-Infinity != undefined", "!0");
    test("({}) != undefined", "!0");
    test("undefined != ({})", "!0");
    test("([]) != undefined", "!0");
    test("undefined != ([])", "!0");
    test("(/a/g) != undefined", "!0");
    test("undefined != (/a/g)", "!0");
    test("(function(){}) != undefined", "!0");
    test("undefined != (function(){})", "!0");

    // origin was `test_same("this == undefined");`
    test("this == undefined", "this==void 0");
    // origin was `test_same("x == undefined");`
    test("x == undefined", "x==void 0");
}

#[test]
fn test_undefined_comparison2() {
    test("'123' !== void 0", "!0");
    test("'123' === void 0", "!1");

    test("void 0 !== '123'", "!0");
    test("void 0 === '123'", "!1");
}

#[test]
fn test_undefined_comparison3() {
    test("'123' !== undefined", "!0");
    test("'123' === undefined", "!1");

    test("undefined !== '123'", "!0");
    test("undefined === '123'", "!1");
}

#[test]
fn test_null_comparison1() {
    test_wcb("null == undefined", "true");
    test_wcb("null == null", "true");
    test_wcb("null == void 0", "true");

    test_wcb("null == 0", "false");
    test_wcb("null == 1", "false");
    test_wcb("null == 0n", "false");
    test_wcb("null == 1n", "false");
    test_wcb("null == 'hi'", "false");
    test_wcb("null == true", "false");
    test_wcb("null == false", "false");

    test_wcb("null === undefined", "false");
    test_wcb("null === null", "true");
    test_wcb("null === void 0", "false");
    test_same("null===x");

    test_same("null==this");
    test_same("null==x");

    test_wcb("null != undefined", "false");
    test_wcb("null != null", "false");
    test_wcb("null != void 0", "false");

    test_wcb("null != 0", "true");
    test_wcb("null != 1", "true");
    test_wcb("null != 0n", "true");
    test_wcb("null != 1n", "true");
    test_wcb("null != 'hi'", "true");
    test_wcb("null != true", "true");
    test_wcb("null != false", "true");

    test_wcb("null !== undefined", "true");
    test_wcb("null !== void 0", "true");
    test_wcb("null !== null", "false");

    test_same("null!=this");
    test_same("null!=x");

    test_wcb("null < null", "false");
    test_wcb("null > null", "false");
    test_wcb("null >= null", "true");
    test_wcb("null <= null", "true");

    test_wcb("0 < null", "false");
    test_wcb("0 > null", "false");
    test_wcb("0 >= null", "true");
    test_wcb("0n < null", "false");
    test_wcb("0n > null", "false");
    test_wcb("0n >= null", "true");
    test_wcb("true > null", "true");
    test_wcb("'hi' < null", "false");
    test_wcb("'hi' >= null", "false");
    test_wcb("null <= null", "true");

    test_wcb("null < 0", "false");
    test_wcb("null < 0n", "false");
    test_wcb("null > true", "false");
    test_wcb("null < 'hi'", "false");
    test_wcb("null >= 'hi'", "false");
    test_wcb("null <= null", "true");

    test_wcb("null == null", "true");
    test_wcb("0 == null", "false");
    test_wcb("1 == null", "false");
    test_wcb("'hi' == null", "false");
    test_wcb("true == null", "false");
    test_wcb("false == null", "false");
    test_wcb("null === null", "true");
    test_wcb("void 0 === null", "false");

    test_wcb("null == NaN", "false");
    test_wcb("NaN == null", "false");
    test_wcb("null == Infinity", "false");
    test_wcb("Infinity == null", "false");
    test_wcb("null == -Infinity", "false");
    test_wcb("-Infinity == null", "false");
    test_wcb("({}) == null", "false");
    test_wcb("null == ({})", "false");
    test_wcb("([]) == null", "false");
    test_wcb("null == ([])", "false");
    test_wcb("(/a/g) == null", "false");
    test_wcb("null == (/a/g)", "false");
    test_wcb("(function(){}) == null", "false");
    test_wcb("null == (function(){})", "false");

    test_wcb("null != NaN", "true");
    test_wcb("NaN != null", "true");
    test_wcb("null != Infinity", "true");
    test_wcb("Infinity != null", "true");
    test_wcb("null != -Infinity", "true");
    test_wcb("-Infinity != null", "true");
    test_wcb("({}) != null", "true");
    test_wcb("null != ({})", "true");
    test_wcb("([]) != null", "true");
    test_wcb("null != ([])", "true");
    test_wcb("(/a/g) != null", "true");
    test_wcb("null != (/a/g)", "true");
    test_wcb("(function(){}) != null", "true");
    test_wcb("null != (function(){})", "true");

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
    test_wcb("!x !== +y", "true");
}

#[test]
fn test_number_boolean_comparison() {
    test_same("+x==!y");
    test_same("+x<=!y");
    test_wcb("+x === !y", "false");
}

#[test]
fn test_boolean_string_comparison() {
    test_same("!x==''+y");
    test_same("!x<=''+y");
    test_wcb("!x !== '' + y", "true");
}

#[test]
fn test_string_boolean_comparison() {
    test_same("''+x==!y");
    test_same("''+x<=!y");
    test_wcb("'' + x === !y", "false");
}

#[test]
fn test_string_string_comparison() {
    test("'a' < 'b'", "!0");
    test("'a' <= 'b'", "!0");
    test("'a' > 'b'", "!1");
    test("'a' >= 'b'", "!1");
    test("+'a' < +'b'", "!1");
    test_same("typeof a<'a'");
    test_same("'a'>=typeof a");
    test("typeof a < typeof a", "!1");
    test("typeof a >= typeof a", "!0");
    test("typeof 3 > typeof 4", "!1");
    test("typeof function() {} < typeof function() {}", "!1");
    test("'a' == 'a'", "!0");
    test("'b' != 'a'", "!0");
    test_same("'undefined'==typeof a");
    test_same("typeof a!='number'");
    test_same("'undefined'==typeof a");
    test_same("'undefined'==typeof a");
    test("typeof a == typeof a", "!0");
    test("'a' === 'a'", "!0");
    test("'b' !== 'a'", "!0");
    test("typeof a === typeof a", "!0");
    test("typeof a !== typeof a", "!1");
    test_same("''+x<=''+y");
    test_same("''+x!=''+y");
    test_same("''+x===''+y");

    test_same("''+x<=''+x"); // potentially foldable
    test_same("''+x!=''+x"); // potentially foldable
    test_same("''+x===''+x"); // potentially foldable
}

#[test]
fn test_number_string_comparison() {
    test_wcb("1 < '2'", "true");
    test_wcb("2 > '1'", "true");
    test_wcb("123 > '34'", "true");
    test_wcb("NaN >= 'NaN'", "false");
    test_wcb("1 == '2'", "false");
    test_wcb("1 != '1'", "false");
    test_wcb("NaN == 'NaN'", "false");
    test_wcb("1 === '1'", "false");
    test_wcb("1 !== '1'", "true");
    test_same("+x>''+y");
    test_same("+x==''+y");
    test_wcb("+x !== '' + y", "true");
}

#[test]
fn test_string_number_comparison() {
    test_wcb("'1' < 2", "true");
    test_wcb("'2' > 1", "true");
    test_wcb("'123' > 34", "true");
    test_wcb("'NaN' < NaN", "false");
    test_wcb("'1' == 2", "false");
    test_wcb("'1' != 1", "false");
    test_wcb("'NaN' == NaN", "false");
    test_wcb("'1' === 1", "false");
    test_wcb("'1' !== 1", "true");
    test_same("''+x<+y");
    test_same("''+x==+y");
    test_wcb("'' + x === +y", "false");
}

#[test]
fn test_bigint_number_comparison() {
    test_wcb("1n < 2", "true");
    test_wcb("1n > 2", "false");
    test_wcb("1n == 1", "true");
    test_wcb("1n == 2", "false");

    // comparing with decimals is allowed
    test_wcb("1n < 1.1", "true");
    test_wcb("1n < 1.9", "true");
    test_wcb("1n < 0.9", "false");
    test_wcb("-1n < -1.1", "false");
    test_wcb("-1n < -1.9", "false");
    test_wcb("-1n < -0.9", "true");
    test_wcb("1n > 1.1", "false");
    test_wcb("1n > 0.9", "true");
    test_wcb("-1n > -1.1", "true");
    test_wcb("-1n > -0.9", "false");

    // Don't fold unsafely large numbers because there might be floating-point error
    let max_safe_int = 9_007_199_254_740_991_i64;
    let neg_max_safe_int = -9_007_199_254_740_991_i64;
    let max_safe_float = 9_007_199_254_740_991_f64;
    let neg_max_safe_float = -9_007_199_254_740_991_f64;
    test_wcb(&format!("0n > {max_safe_int}"), "false");
    test_wcb(&format!("0n < {max_safe_int}"), "true");
    test_wcb(&format!("0n > {neg_max_safe_int}"), "true");
    test_wcb(&format!("0n < {neg_max_safe_int}"), "false");
    test_wcb(&format!("0n > {max_safe_float}"), "false");
    test_wcb(&format!("0n < {max_safe_float}"), "true");
    test_wcb(&format!("0n > {neg_max_safe_float}"), "true");
    test_wcb(&format!("0n < {neg_max_safe_float}"), "false");

    // comparing with Infinity is allowed
    test_wcb("1n < Infinity", "true");
    test_wcb("1n > Infinity", "false");
    test_wcb("1n < -Infinity", "false");
    test_wcb("1n > -Infinity", "true");

    // null is interpreted as 0 when comparing with bigint
    test_wcb("1n < null", "false");
    test_wcb("1n > null", "true");
}

#[test]
fn test_bigint_string_comparison() {
    test_wcb("1n < '2'", "true");
    test_wcb("2n > '1'", "true");
    test_wcb("123n > '34'", "true");
    test_wcb("1n == '1'", "true");
    test_wcb("1n == '2'", "false");
    test_wcb("1n != '1'", "false");
    test_wcb("1n === '1'", "false");
    test_wcb("1n !== '1'", "true");
}

#[test]
fn test_string_bigint_comparison() {
    test_wcb("'1' < 2n", "true");
    test_wcb("'2' > 1n", "true");
    test_wcb("'123' > 34n", "true");
    test_wcb("'1' == 1n", "true");
    test_wcb("'1' == 2n", "false");
    test_wcb("'1' != 1n", "false");
    test_wcb("'1' === 1n", "false");
    test_wcb("'1' !== 1n", "true");
}

#[test]
fn test_nan_comparison() {
    test_wcb("NaN < 1", "false");
    test_wcb("NaN <= 1", "false");
    test_wcb("NaN > 1", "false");
    test_wcb("NaN >= 1", "false");
    test_wcb("NaN < 1n", "false");
    test_wcb("NaN <= 1n", "false");
    test_wcb("NaN > 1n", "false");
    test_wcb("NaN >= 1n", "false");

    test_wcb("NaN < NaN", "false");
    test_wcb("NaN >= NaN", "false");
    test_wcb("NaN == NaN", "false");
    test_wcb("NaN === NaN", "false");

    test_wcb("NaN < null", "false");
    test_wcb("null >= NaN", "false");
    test_wcb("NaN == null", "false");
    test_wcb("null != NaN", "true");
    test_wcb("null === NaN", "false");

    test_wcb("NaN < undefined", "false");
    test_wcb("undefined >= NaN", "false");
    test_wcb("NaN == undefined", "false");
    test_wcb("undefined != NaN", "true");
    test_wcb("undefined === NaN", "false");

    test_same("NaN<x");
    test_same("x>=NaN");
    test_same("NaN==x");
    test_same("x!=NaN");
    test_wcb("NaN === x", "false");
    test_wcb("x !== NaN", "true");
    test_same("NaN==foo()");
}

#[test]
fn js_typeof() {
    test("x = typeof 1", "x='number'");
    test("x = typeof 'foo'", "x='string'");
    test("x = typeof true", "x='boolean'");
    test("x = typeof false", "x='boolean'");
    test("x = typeof null", "x='object'");
    test("x = typeof undefined", "x='undefined'");
    test("x = typeof void 0", "x='undefined'");
    test("x = typeof []", "x='object'");
    test("x = typeof [1]", "x='object'");
    test("x = typeof [1,[]]", "x='object'");
    test("x = typeof {}", "x='object'");
    test("x = typeof function() {}", "x='function'");

    test_same("x=typeof [1,[foo()]]");
    test_same("x=typeof {bathwater:baby()}");
}

#[test]
fn unary_ops() {
    // TODO: need to port
    // These cases are handled by PeepholeRemoveDeadCode in closure-compiler.
    // test_same("!foo()");
    // test_same("~foo()");
    // test_same("-foo()");

    // These cases are handled here.
    test("a=!true", "a=!!0");
    test("a=!10", "a=!1");
    test("a=!false", "a=!!1");
    test_same("a=!foo()");
    test("a=-0", "a=-0");
    test("a=-(0)", "a=-0");
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
    // test("a=+(f?true:false)", "a=+(f?1:0)"); // TODO(johnlenz): foldable
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
fn unary_with_big_int() {
    test("-(1n)", "-1n");
    test("- -1n", "1n");
    test_wcb("!1n", "false");
    test("~0n", "-1n");
}

#[test]
fn test_unary_ops_string_compare() {
    test_same("a=-1");
    test("a = ~0", "a=-1");
    test("a = ~1", "a=-2");
    test("a = ~101", "a=-102");
}

#[test]
fn test_fold_logical_op() {
    test("x = true && x", "x=x");
    test("x = [foo()] && x", "x=([foo()],x)");

    test("x = false && x", "x=!1");
    test("x = true || x", "x=!0");
    test("x = false || x", "x=x");
    test("x = 0 && x", "x=0");
    test("x = 3 || x", "x=3");
    test("x = 0n && x", "x=0n");
    test("x = 3n || x", "x=3n");
    test("x = false || 0", "x=0");

    // unfoldable, because the right-side may be the result
    test("a = x && true", "a=x&&!0");
    test("a = x && false", "a=x&&!1");
    test("a = x || 3", "a=x||3");
    test("a = x || false", "a=x||!1");
    test("a = b ? c : x || false", "a=b?c:x||!1");
    test("a = b ? x || false : c", "a=b?x||!1:c");
    test("a = b ? c : x && true", "a=b?c:x&&!0");
    test("a = b ? x && true : c", "a=b?x&&!0:c");

    // folded, but not here.
    test_wcb("a = x || false ? b : c", "a=x||false?b:c");
    test_wcb("a = x && true ? b : c", "a=x&&true?b:c");

    test("x = foo() || true || bar()", "x=foo()||!0");
    test("x = foo() || true && bar()", "x=foo()||bar()");
    test("x = foo() || false && bar()", "x=foo()||!1");
    test("x = foo() && false && bar()", "x=foo()&&!1");
    test("x = foo() && false || bar()", "x=(foo()&&!1,bar())");
    test("x = foo() || false || bar()", "x=foo()||bar()");
    test("x = foo() && true && bar()", "x=foo()&&bar()");
    test("x = foo() || true || bar()", "x=foo()||!0");
    test("x = foo() && false && bar()", "x=foo()&&!1");
    test("x = foo() && 0 && bar()", "x=foo()&&0");
    test("x = foo() && 1 && bar()", "x=foo()&&bar()");
    test("x = foo() || 0 || bar()", "x=foo()||bar()");
    test("x = foo() || 1 || bar()", "x=foo()||1");
    test("x = foo() && 0n && bar()", "x=foo()&&0n");
    test("x = foo() && 1n && bar()", "x=foo()&&bar()");
    test("x = foo() || 0n || bar()", "x=foo()||bar()");
    test("x = foo() || 1n || bar()", "x=foo()||1n");
    test_same("x=foo()||bar()||baz()");
    test_same("x=foo()&&bar()&&baz()");

    test("0 || b()", "b()");
    test("1 && b()", "b()");
    test("a() && (1 && b())", "a()&&b()");
    test("(a() && 1) && b()", "a()&&b()");

    test("(x || '') || y;", "x||y");
    test("false || (x || '');", "x||''");
    test("(x && 1) && y;", "x&&y");
    test("true && (x && 1);", "x&&1");

    // Really not foldable, because it would change the type of the
    // expression if foo() returns something truthy but not true.
    // Cf. FoldConstants.tryFoldAndOr().
    // An example would be if foo() is 1 (truthy) and bar() is 0 (falsey):
    // (1 && true) || 0 == true
    // 1 || 0 == 1, but true =/= 1
    test_wcb("x=foo()&&true||bar()", "x=foo()&&true||bar()");
    test_wcb("foo()&&true||bar()", "foo()&&true||bar()");
}

#[test]
fn test_fold_logical_op2() {
    test("x = function(){} && x", "x=x");
    test("x = true && function(){}", "x=function(){}");
    test("x = [(function(){alert(x)})()] && x", "x=([function(){alert(x)}()],x)");
}

#[test]
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
