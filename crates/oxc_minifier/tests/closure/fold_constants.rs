//! <https://github.com/google/closure-compiler/blob/master/test/com/google/javascript/jscomp/PeepholeFoldConstantsTest.java>

use crate::{test, test_same};

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
    // test("'hi' >= undefined", "!1");
    test("null <= undefined", "!1");

    test("undefined < 0", "!1");
    test("undefined > true", "!1");
    // test("undefined >= 'hi'", "!1");
    test("undefined <= null", "!1");

    test("null == undefined", "!0");
    test("0 == undefined", "!1");
    test("1 == undefined", "!1");
    // test("'hi' == undefined", "!1");
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
    // test("- -1n", "1n");
    // test("!1n", "false");
    // test("~0n", "-1n");
}

#[test]
fn test_unary_ops_string_compare() {
    test_same("a=-1");
    test("a = ~0", "a=-1");
    test("a = ~1", "a=-2");
    test("a = ~101", "a=-102");
}
