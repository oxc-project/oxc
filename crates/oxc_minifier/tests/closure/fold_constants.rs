//! <https://github.com/google/closure-compiler/blob/master/test/com/google/javascript/jscomp/PeepholeFoldConstantsTest.java>

use crate::{test, test_same};

#[test]
fn undefined_comparison1() {
    test("undefined == undefined", "!0");
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
    // test("a=!true", "a=false");
    // test("a=!10", "a=false");
    // test("a=!false", "a=true");
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
