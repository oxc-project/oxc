static MAX_SAFE_FLOAT: f64 = 9_007_199_254_740_991_f64;
static NEG_MAX_SAFE_FLOAT: f64 = -9_007_199_254_740_991_f64;

static MAX_SAFE_INT: i64 = 9_007_199_254_740_991_i64;
static NEG_MAX_SAFE_INT: i64 = -9_007_199_254_740_991_i64;

use crate::test;

// wrap with a function call so it doesn't get removed.
fn fold(source_text: &str, expected: &str) {
    let source_text = format!("NOOP({source_text})");
    let expected = format!("NOOP({expected})");
    test(&source_text, &expected);
}

fn fold_same(source_text: &str) {
    fold(source_text, source_text);
}

fn test_same(source_text: &str) {
    test(source_text, source_text);
}

#[test]
fn test_comparison() {
    fold("(1, 2) !== 2", "!1");
    fold_same("({} <= {})");
    fold_same("({} >= {})");
    fold_same("({} > {})");
    fold_same("({} < {})");
    fold_same("([] <= [])");
    fold_same("([] >= [])");
    fold_same("([] > [])");
    fold_same("([] < [])");
}

#[test]
fn undefined_comparison1() {
    fold("undefined == undefined", "!0");
    fold("undefined == null", "!0");
    fold("undefined == void 0", "!0");

    fold("undefined == 0", "!1");
    fold("undefined == 1", "!1");
    fold("undefined == 'hi'", "!1");
    fold("undefined == true", "!1");
    fold("undefined == false", "!1");

    fold("undefined === undefined", "!0");
    fold("undefined === null", "!1");
    fold("undefined === void 0", "!0");

    fold("undefined == this", "this == null");
    fold("undefined == x", "x == null");

    fold("undefined != undefined", "!1");
    fold("undefined != null", "!1");
    fold("undefined != void 0", "!1");

    fold("undefined != 0", "!0");
    fold("undefined != 1", "!0");
    fold("undefined != 'hi'", "!0");
    fold("undefined != true", "!0");
    fold("undefined != false", "!0");

    fold("undefined !== undefined", "!1");
    fold("undefined !== void 0", "!1");
    fold("undefined !== null", "!0");

    fold("undefined != this", "this != null");
    fold("undefined != x", "x != null");

    fold("undefined < undefined", "!1");
    fold("undefined > undefined", "!1");
    fold("undefined >= undefined", "!1");
    fold("undefined <= undefined", "!1");

    fold("0 < undefined", "!1");
    fold("true > undefined", "!1");
    fold("'hi' >= undefined", "!1");
    fold("null <= undefined", "!1");

    fold("undefined < 0", "!1");
    fold("undefined > true", "!1");
    fold("undefined >= 'hi'", "!1");
    fold("undefined <= null", "!1");

    fold("null == undefined", "!0");
    fold("0 == undefined", "!1");
    fold("1 == undefined", "!1");
    fold("'hi' == undefined", "!1");
    fold("true == undefined", "!1");
    fold("false == undefined", "!1");
    fold("null === undefined", "!1");
    fold("void 0 === undefined", "!0");

    fold("undefined == NaN", "!1");
    fold("NaN == undefined", "!1");
    fold("undefined == Infinity", "!1");
    fold("Infinity == undefined", "!1");
    fold("undefined == -Infinity", "!1");
    fold("-Infinity == undefined", "!1");
    fold("({}) == undefined", "!1");
    fold("undefined == ({})", "!1");
    fold("([]) == undefined", "!1");
    fold("undefined == ([])", "!1");
    fold("(/a/g) == undefined", "!1");
    fold("undefined == (/a/g)", "!1");
    fold("(function(){}) == undefined", "!1");
    fold("undefined == (function(){})", "!1");

    fold("undefined != NaN", "!0");
    fold("NaN != undefined", "!0");
    fold("undefined != Infinity", "!0");
    fold("Infinity != undefined", "!0");
    fold("undefined != -Infinity", "!0");
    fold("-Infinity != undefined", "!0");
    fold("({}) != undefined", "!0");
    fold("undefined != ({})", "!0");
    fold("([]) != undefined", "!0");
    fold("undefined != ([])", "!0");
    fold("(/a/g) != undefined", "!0");
    fold("undefined != (/a/g)", "!0");
    fold("(function(){}) != undefined", "!0");
    fold("undefined != (function(){})", "!0");

    fold("this == undefined", "this == null");
    fold("x == undefined", "x == null");
}

#[test]
fn test_undefined_comparison2() {
    fold("\"123\" !== void 0", "!0");
    fold("\"123\" === void 0", "!1");

    fold("void 0 !== \"123\"", "!0");
    fold("void 0 === \"123\"", "!1");
}

#[test]
fn test_undefined_comparison3() {
    fold("\"123\" !== undefined", "!0");
    fold("\"123\" === undefined", "!1");

    fold("undefined !== \"123\"", "!0");
    fold("undefined === \"123\"", "!1");
}

#[test]
fn test_null_comparison1() {
    fold("null == undefined", "!0");
    fold("null == null", "!0");
    fold("null == void 0", "!0");

    fold("null == 0", "!1");
    fold("null == 1", "!1");
    fold("null == 0n", "!1");
    fold("null == 1n", "!1");
    fold("null == 'hi'", "!1");
    fold("null == true", "!1");
    fold("null == false", "!1");

    fold("null === undefined", "!1");
    fold("null === null", "!0");
    fold("null === void 0", "!1");
    fold_same("x===null");

    fold_same("this==null");
    fold_same("x==null");

    fold("null != undefined", "!1");
    fold("null != null", "!1");
    fold("null != void 0", "!1");

    fold("null != 0", "!0");
    fold("null != 1", "!0");
    fold("null != 0n", "!0");
    fold("null != 1n", "!0");
    fold("null != 'hi'", "!0");
    fold("null != true", "!0");
    fold("null != false", "!0");

    fold("null !== undefined", "!0");
    fold("null !== void 0", "!0");
    fold("null !== null", "!1");

    fold_same("this!=null");
    fold_same("x!=null");

    fold("null < null", "!1");
    fold("null > null", "!1");
    fold("null >= null", "!0");
    fold("null <= null", "!0");

    fold("0 < null", "!1");
    fold("0 > null", "!1");
    fold("0 >= null", "!0");
    fold("0n < null", "!1");
    fold("0n > null", "!1");
    fold("0n >= null", "!0");
    fold("true > null", "!0");
    fold("'hi' < null", "!1");
    fold("'hi' >= null", "!1");
    fold("null <= null", "!0");

    fold("null < 0", "!1");
    fold("null < 0n", "!1");
    fold("null > true", "!1");
    fold("null < 'hi'", "!1");
    fold("null >= 'hi'", "!1");
    fold("null <= null", "!0");

    fold("null == null", "!0");
    fold("0 == null", "!1");
    fold("1 == null", "!1");
    fold("'hi' == null", "!1");
    fold("true == null", "!1");
    fold("false == null", "!1");
    fold("null === null", "!0");
    fold("void 0 === null", "!1");

    fold("null == NaN", "!1");
    fold("NaN == null", "!1");
    fold("null == Infinity", "!1");
    fold("Infinity == null", "!1");
    fold("null == -Infinity", "!1");
    fold("-Infinity == null", "!1");
    fold("({}) == null", "!1");
    fold("null == ({})", "!1");
    fold("([]) == null", "!1");
    fold("null == ([])", "!1");
    fold("(/a/g) == null", "!1");
    fold("null == (/a/g)", "!1");
    fold("(function(){}) == null", "!1");
    fold("null == (function(){})", "!1");

    fold("null != NaN", "!0");
    fold("NaN != null", "!0");
    fold("null != Infinity", "!0");
    fold("Infinity != null", "!0");
    fold("null != -Infinity", "!0");
    fold("-Infinity != null", "!0");
    fold("({}) != null", "!0");
    fold("null != ({})", "!0");
    fold("([]) != null", "!0");
    fold("null != ([])", "!0");
    fold("(/a/g) != null", "!0");
    fold("null != (/a/g)", "!0");
    fold("(function(){}) != null", "!0");
    fold("null != (function(){})", "!0");

    fold_same("({a:f()})==null");
    fold_same("[f()]==null");

    fold_same("this==null");
    fold_same("x==null");
}

#[test]
fn test_boolean_boolean_comparison() {
    fold_same("!x == !y");
    fold_same("!x < !y");
    fold("!x!==!y", "!x != !y");

    fold_same("!x == !x"); // foldable
    fold_same("!x <! x"); // foldable
    fold("!x !== !x", "!x != !x"); // foldable
}

#[test]
fn test_boolean_number_comparison() {
    fold_same("!x==+y");
    fold_same("!x<=+y");
    fold_same("!x !== +y");
}

#[test]
fn test_number_boolean_comparison() {
    fold_same("+x==!y");
    fold_same("+x<=!y");
    fold_same("+x === !y");
}

#[test]
fn test_boolean_string_comparison() {
    fold_same("!x==''+y");
    fold_same("!x<=''+y");
    fold_same("!x !== '' + y");
}

#[test]
fn test_string_boolean_comparison() {
    fold_same("''+x==!y");
    fold_same("''+x<=!y");
    fold_same("'' + x === !y");
}

#[test]
fn test_number_number_comparison() {
    fold("1 > 1", "!1");
    fold("2 == 3", "!1");
    fold("3.6 === 3.6", "!0");
    fold_same("+x > +y");
    fold_same("+x == +y");
    fold("+x === +y", "+x == +y");
    fold_same("+x > +x"); // foldable to false
    fold_same("+x == +x");
    fold("+x === +x", "+x == +x");
}

#[test]
fn test_string_string_comparison() {
    fold("'a' < 'b'", "!0");
    fold("'a' <= 'b'", "!0");
    fold("'a' > 'b'", "!1");
    fold("'a' >= 'b'", "!1");
    fold("+'a' < +'b'", "!1");
    fold_same("typeof a < 'a'");
    fold_same("'a' >= typeof a");
    fold_same("typeof a < typeof a");
    fold_same("typeof a >= typeof a");
    fold("typeof 3 > typeof 4", "!1");
    fold("typeof function() {} < typeof function() {}", "!1");
    fold("'a' == 'a'", "!0");
    fold("'b' != 'a'", "!0");
    fold_same("typeof a != 'number'");
    fold_same("typeof a != 'unknown'"); // IE
    fold("'a' === 'a'", "!0");
    fold("'b' !== 'a'", "!0");
    fold_same("'' + x <= '' + y");
    fold_same("'' + x != '' + y");
    fold("'' + x === '' + y", "'' + x == '' + y");

    fold_same("'' + x <= '' + x"); // potentially foldable
    fold_same("'' + x != '' + x"); // potentially foldable
    fold("'' + x === '' + x", "'' + x == '' + x"); // potentially foldable

    test(r#"if ("string" !== "\u000Bstr\u000Bing\u000B") {}"#, "");
}

#[test]
fn test_number_string_comparison() {
    fold("1 < '2'", "!0");
    fold("2 > '1'", "!0");
    fold("123 > '34'", "!0");
    fold("NaN >= 'NaN'", "!1");
    fold("1 == '2'", "!1");
    fold("1 != '1'", "!1");
    fold("NaN == 'NaN'", "!1");
    fold("1 === '1'", "!1");
    fold("1 !== '1'", "!0");
    fold_same("+x>''+y");
    fold_same("+x==''+y");
    fold_same("+x !== '' + y");
}

#[test]
fn test_string_number_comparison() {
    fold("'1' < 2", "!0");
    fold("'2' > 1", "!0");
    fold("'123' > 34", "!0");
    fold("'NaN' < NaN", "!1");
    fold("'1' == 2", "!1");
    fold("'1' != 1", "!1");
    fold("'NaN' == NaN", "!1");
    fold("'1' === 1", "!1");
    fold("'1' !== 1", "!0");
    fold_same("''+x<+y");
    fold_same("''+x==+y");
    fold_same("'' + x === +y");
}

#[test]
fn test_nan_comparison() {
    fold("NaN < 1", "!1");
    fold("NaN <= 1", "!1");
    fold("NaN > 1", "!1");
    fold("NaN >= 1", "!1");
    fold("NaN < 1n", "!1");
    fold("NaN <= 1n", "!1");
    fold("NaN > 1n", "!1");
    fold("NaN >= 1n", "!1");

    fold("NaN < NaN", "!1");
    fold("NaN >= NaN", "!1");
    fold("NaN == NaN", "!1");
    fold("NaN === NaN", "!1");

    fold("NaN < null", "!1");
    fold("null >= NaN", "!1");
    fold("NaN == null", "!1");
    fold("null != NaN", "!0");
    fold("null === NaN", "!1");

    fold("NaN < undefined", "!1");
    fold("undefined >= NaN", "!1");
    fold("NaN == undefined", "!1");
    fold("undefined != NaN", "!0");
    fold("undefined === NaN", "!1");

    fold_same("NaN<x");
    fold_same("x>=NaN");
    fold("NaN==x", "x==NaN");
    fold_same("x!=NaN");
    fold("NaN === x", "x === NaN");
    fold_same("x !== NaN");
    fold("NaN==foo()", "foo()==NaN");
}

#[test]
fn test_object_comparison1() {
    fold("!new Date()", "!1");
    fold("!!new Date()", "!0");
    fold_same("!new Date(foo)");

    fold("new Date() == null", "!1");
    fold("new Date() == undefined", "!1");
    fold("new Date() != null", "!0");
    fold("new Date() != undefined", "!0");
    fold("null == new Date()", "!1");
    fold("undefined == new Date()", "!1");
    fold("null != new Date()", "!0");
    fold("undefined != new Date()", "!0");
    fold("new Date(foo) != undefined", "new Date(foo) != null");
}

#[test]
fn js_typeof() {
    fold("x = typeof 1n", "x = \"bigint\"");
    fold("x = typeof 1", "x = \"number\"");
    fold("x = typeof 'foo'", "x = \"string\"");
    fold("x = typeof true", "x = \"boolean\"");
    fold("x = typeof false", "x = \"boolean\"");
    fold("x = typeof null", "x = \"object\"");
    fold("x = typeof undefined", "x = \"undefined\"");
    fold("x = typeof void 0", "x = \"undefined\"");
    fold("x = typeof []", "x = \"object\"");
    fold("x = typeof [1]", "x = \"object\"");
    fold("x = typeof [1,[]]", "x = \"object\"");
    fold("x = typeof {}", "x = \"object\"");
    test("var a, b; NOOP(x = typeof (a === b))", "var a, b; NOOP(x = \"boolean\")");
    test("var foo; NOOP(x = typeof { foo })", "var foo; NOOP(x = \"object\")");
    fold("x = typeof function() {}", "x = 'function'");
    fold("x = typeof (() => {})", "x = 'function'");
    fold("x = typeof class{}", "x = \"function\"");
    fold_same("x = typeof foo"); // no sideeffect, but we don't know the result

    fold_same("x = typeof[1,[foo()]]");
    fold_same("x = typeof{bathwater:baby()}");
    fold_same("x = typeof class { static { foo() } }");
}

#[test]
fn test_fold_unary() {
    fold_same("!foo()");
    fold_same("~foo()");
    fold_same("-foo()");

    fold("a=!true", "a=!1");
    fold("a=!10", "a=!1");
    fold("a=!false", "a=!0");
    fold_same("a=!foo()");
    fold_same("a = !!void b");

    fold("a=-0", "a=-0");
    fold("a=-(0)", "a=-0");
    fold_same("a=-Infinity");
    fold("a=-NaN", "a=NaN");
    fold_same("a=-foo()");
    fold("-undefined", "NaN");
    fold("-null", "-0");
    fold("-NaN", "NaN");

    fold("a=+true", "a=1");
    fold("a=+10", "a=10");
    fold("a=+false", "a=0");
    fold_same("a=+foo()");
    fold_same("a=+f");
    fold("a=+(f?true:false)", "a=+!!f");
    fold("a=+(f?!0:!1)", "a=+!!f");
    fold_same("a=+(f?(foo, !0):(bar, !1))");
    fold("a=+0", "a=0");
    fold("a=+Infinity", "a=Infinity");
    fold("a=+NaN", "a=NaN");
    fold("a=+-7", "a=-7");
    fold("a=+.5", "a=.5");

    fold("a=~~0", "a=0");
    fold("a=~~10", "a=10");
    fold("a=~-7", "a=6");
    fold_same("a=~~foo()");
    fold("a=~0xffffffff", "a=0");
    fold("a=~~0xffffffff", "a=-1");
    fold("a=~.5", "a=-1");

    fold("a=+[]", "a=0");
    fold_same("a=+[...foo]");
    fold("a=+[,]", "a=0");
    fold("a=+[0]", "a=0");
    fold("a=+['0x10']", "a=16");
    fold("a=+[[]]", "a=0");
    fold("a=+[0, 1]", "a=NaN");
    test_same("var foo; NOOP(a=+[0, ...foo])"); // can be either `a=0` or `a=NaN` (also `...foo` may have a side effect)
    test("var foo; NOOP(a=+[0, ...[foo ? 'foo': ''], 1])", "var foo; NOOP(a=NaN)");

    fold("a=+[false]", "a=NaN"); // `+"false"`
    fold("a=+[true]", "a=NaN"); // `+"true"`
    fold("a=+[undefined]", "a=0"); // `+""`
    fold("a=+[null]", "a=0"); // `+""`
}

#[test]
fn test_fold_unary_big_int() {
    fold("-(1n)", "-1n");
    fold("- -1n", "1n");
    fold("!1n", "!1");
    fold("~0n", "-1n");

    fold("~-1n", "0n");
    fold("~~1n", "1n");

    fold("~0x3n", "-4n");
    fold("~0b11n", "-4n");
}

#[test]
fn test_unary_ops_string_compare() {
    fold_same("a = -1");
    fold("a = ~0", "a = -1");
    fold("a = ~1", "a = -2");
    fold("a = ~101", "a = -102");

    fold("a = ~1.1", "a = -2");
    fold("a = ~0x3", "a = -4"); // Hexadecimal number
    fold("a = ~9", "a = -10"); // Despite `-10` is longer than `~9`, the compiler still folds it.
    fold_same("a = ~b");
    fold("a = ~NaN", "a = -1");
    fold("a = ~-Infinity", "a = -1");
    fold("x = ~2147483658.0", "x = 2147483637");
    fold("x = ~-2147483658", "x = -2147483639");
}

#[test]
fn test_fold_logical_op() {
    fold("x = true && x", "x = x");
    fold("x = [foo()] && x", "x = (foo(),x)");

    fold("x = false && x", "x = !1");
    fold("x = true || x", "x = !0");
    fold("x = false || x", "x = x");
    fold("x = 0 && x", "x = 0");
    fold("x = 3 || x", "x = 3");
    fold("x = 0n && x", "x = 0n");
    fold("x = 3n || x", "x = 3n");
    fold("x = false || 0", "x = 0");

    // unfoldable, because the right-side may be the result
    fold("a = x && true", "a=x && !0");
    fold("a = x && false", "a=x && !1");
    fold("a = x || 3", "a=x || 3");
    fold("a = x || false", "a=x || !1");
    fold("a = b ? c : x || false", "a=b ? c : x || !1");
    fold("a = b ? x || false : c", "a=b ? x || !1 : c");
    fold("a = b ? c : x && true", "a=b ? c : x && !0");
    fold("a = b ? x && true : c", "a=b ? x && !0 : c");

    fold("a = x || false ? b : c", "a = x ? b : c");
    fold("a = x && true ? b : c", "a = x ? b : c");

    fold("x = foo() || true || bar()", "x = foo() || !0");
    fold("x = foo() || true && bar()", "x = foo() || bar()");
    fold("x = foo() || false && bar()", "x = foo() || !1");
    fold("x = foo() && false && bar()", "x = foo() && !1");
    fold("x = foo() && false || bar()", "x = (foo(), bar())");
    fold("x = foo() || false || bar()", "x = foo() || bar()");
    fold("x = foo() && true && bar()", "x = foo() && bar()");
    fold("x = foo() || true || bar()", "x = foo() || !0");
    fold("x = foo() && false && bar()", "x = foo() && !1");
    fold("x = foo() && 0 && bar()", "x = foo() && 0");
    fold("x = foo() && 1 && bar()", "x = foo() && bar()");
    fold("x = foo() || 0 || bar()", "x = foo() || bar()");
    fold("x = foo() || 1 || bar()", "x = foo() || 1");
    fold("x = foo() && 0n && bar()", "x = foo() && 0n");
    fold("x = foo() && 1n && bar()", "x = foo() && bar()");
    fold("x = foo() || 0n || bar()", "x = foo() || bar()");
    fold("x = foo() || 1n || bar()", "x = foo() || 1n");
    fold_same("x = foo() || bar() || baz()");
    fold_same("x = foo() && bar() && baz()");

    fold("0 || b()", "b()");
    fold("1 && b()", "b()");
    fold("a() && (1 && b())", "a() && b()");
    fold("(a() && 1) && b()", "a() && b()");

    fold("(x || '') || y", "x || y");
    fold("false || (x || '')", "x || ''");
    fold("(x && 1) && y", "x && y");
    fold("true && (x && 1)", "x && 1");

    // Really not foldable, because it would change the type of the
    // expression if foo() returns something truthy but not true.
    // Cf. FoldConstants.tryFoldAndOr().
    // An example would be if foo() is 1 (truthy) and bar() is 0 (falsey):
    // (1 && true) || 0 == true
    // 1 || 0 == 1, but true =/= 1
    fold("x = foo() && true || bar()", "x = foo() && !0 || bar()");
    fold("foo() && true || bar()", "foo() && !0 || bar()");

    test("var y; x = (true && y)()", "var y; x = y()");
    test("var y; x = (true && y.z)()", "var y; x = (0, y.z)()");
    test("var y; x = (false || y)()", "var y; x = y()");
    test("var y; x = (false || y.z)()", "var y; x = (0, y.z)()");
}

#[test]
fn test_fold_logical_op2() {
    fold("x = function(){} && x", "x=x");
    fold("x = true && function(){}", "x=function(){}");
    fold("x = [(function(){alert(x)})()] && x", "x=((function(){alert(x)})(),x)");
}

#[test]
fn test_fold_nullish_coalesce() {
    // fold if left is null/undefined
    fold("null ?? 1", "1");
    fold("undefined ?? false", "!1");
    fold("(a(), null) ?? 1", "(a(), 1)");

    fold("x = [foo()] ?? x", "x = [foo()]");

    // short circuit on all non nullish LHS
    fold("x = false ?? x", "x = !1");
    fold("x = true ?? x", "x = !0");
    fold("x = 0 ?? x", "x = 0");
    fold("x = 3 ?? x", "x = 3");

    // unfoldable, because the right-side may be the result
    fold("a = x ?? true", "a = x ?? !0");
    fold("a = x ?? false", "a = x ?? !1");
    fold_same("a = x ?? 3");
    fold("a = b ? c : x ?? false", "a = b ? c : x ?? !1");
    fold("a = b ? x ?? false : c", "a = b ? x ?? !1 : c");

    // folded, but not here.
    fold("a = x ?? false ? b : c", "a = x ?? !1 ? b : c");
    fold("a = x ?? true ? b : c", "a = x ?? !0 ? b : c");

    fold("x = foo() ?? true ?? bar()", "x = foo() ?? !0 ?? bar()");
    fold("x = foo() ?? (true && bar())", "x = foo() ?? bar()");
    fold("x = (foo() || false) ?? bar()", "x = (foo() || !1) ?? bar()");

    fold("a() ?? (1 ?? b())", "a() ?? 1");
    fold("(a() ?? 1) ?? b()", "a() ?? 1 ?? b()");

    test_same("var y; x = (y ?? 1)()"); // can compress to "var y; x = y()" if y is not null or undefined
    test_same("var y; x = (y.z ?? 1)()"); // "var y; x = (0, y.z)()" if y is not null or undefined
    test("var y; x = (null ?? y)()", "var y; x = y()");
    test("var y; x = (null ?? y.z)()", "var y; x = (0, y.z)()");
}

#[test]
fn test_fold_void() {
    fold_same("void 0");
    fold("void 1", "void 0");
    fold_same("void x");
    fold_same("void x()");
}

#[test]
fn test_fold_opt_chain() {
    // can't fold when optional part may execute
    fold_same("a = x?.y");
    fold_same("a = x?.()");

    // fold args of optional call
    fold("x = foo() ?. (true && bar())", "x = foo() ?.(bar())");
    fold("a() ?. (1 ?? b())", "a() ?. (1)");

    // test("({a})?.a.b.c.d()?.x.y.z", "a.b.c.d()?.x.y.z");

    fold("x = undefined?.y", "x = void 0");
    fold("x = null?.y", "x = void 0");
    fold("x = undefined?.[foo]", "x = void 0");
    fold("x = null?.[foo]", "x = void 0");
    fold("x = undefined?.()", "x = void 0");
    fold("x = null?.()", "x = void 0");
}

#[test]
fn test_fold_bitwise_op() {
    fold("x = 1 & 1", "x = 1");
    fold("x = 1 & 2", "x = 0");
    fold("x = 3 & 1", "x = 1");
    fold("x = 3 & 3", "x = 3");

    fold("x = 1 | 1", "x = 1");
    fold("x = 1 | 2", "x = 3");
    fold("x = 3 | 1", "x = 3");
    fold("x = 3 | 3", "x = 3");

    fold("x = 1 ^ 1", "x = 0");
    fold("x = 1 ^ 2", "x = 3");
    fold("x = 3 ^ 1", "x = 2");
    fold("x = 3 ^ 3", "x = 0");

    fold("x = -1 & 0", "x = 0");
    fold("x = 0 & -1", "x = 0");
    fold("x = 1 & 4", "x = 0");
    fold("x = 2 & 3", "x = 2");

    // make sure we fold only when we are supposed to -- not when doing so would
    // lose information or when it is performed on nonsensical arguments.
    fold("x = 1 & 1.1", "x = 1");
    fold("x = 1.1 & 1", "x = 1");
    fold("x = 1 & 3000000000", "x = 0");
    fold("x = 3000000000 & 1", "x = 0");

    // Try some cases with | as well
    fold("x = 1 | 4", "x = 5");
    fold("x = 1 | 3", "x = 3");
    fold("x = 1 | 1.1", "x = 1");
    // test_same("x = 1 | 3e9");

    // these cases look strange because bitwise OR converts unsigned numbers to be signed
    fold("x = 1 | 3000000001", "x = -1294967295");
    fold("x = 4294967295 | 0", "x = -1");

    fold("x = -1 | 0", "x = -1");
}

#[test]
fn test_fold_bitwise_op2() {
    fold("x = y & 1 & 1", "x = y & 1");
    fold("x = y & 1 & 2", "x = y & 0");
    fold("x = y & 3 & 1", "x = y & 1");
    fold("x = 3 & y & 1", "x = y & 1");
    fold("x = y & 3 & 3", "x = y & 3");
    fold("x = 3 & y & 3", "x = y & 3");

    fold("x = y | 1 | 1", "x = y | 1");
    fold("x = y | 1 | 2", "x = y | 3");
    fold("x = y | 3 | 1", "x = y | 3");
    fold("x = 3 | y | 1", "x = y | 3");
    fold("x = y | 3 | 3", "x = y | 3");
    fold("x = 3 | y | 3", "x = y | 3");

    fold("x = y ^ 1 ^ 1", "x = y ^ 0");
    fold("x = y ^ 1 ^ 2", "x = y ^ 3");
    fold("x = y ^ 3 ^ 1", "x = y ^ 2");
    fold("x = 3 ^ y ^ 1", "x = y ^ 2");
    fold("x = y ^ 3 ^ 3", "x = y ^ 0");
    fold("x = 3 ^ y ^ 3", "x = y ^ 0");

    fold("x = Infinity | NaN", "x=0");
    fold("x = 12 | NaN", "x=12");
}

#[test]
fn test_fold_bitwise_op_additional() {
    fold("x = null & 1", "x = 0");
    fold_same("x = (2 ** 31 - 1) | 1");
    fold_same("x = (2 ** 31) | 1");

    // https://github.com/oxc-project/oxc/issues/7944
    fold_same("(x - 1) & 1");
    fold_same("(y >> 3) & 7");
    fold("(y & 3) & 7", "y & 3");
    fold_same("(y | 3) & 7");
    fold("y | 3 & 7", "y | 3");
}

#[test]
fn test_fold_bitwise_not() {
    fold("~undefined", "-1");
    fold("~null", "-1");
    fold("~false", "-1");
    fold("~true", "-2");
    fold("~'1'", "-2");
    fold("~'-1'", "0");
    fold("~{}", "-1");
}

#[test]
fn test_fold_bit_shifts() {
    fold("x = 1 << 0", "x=1");
    fold("x = -1 << 0", "x=-1");
    fold("x = 1 << 1", "x=2");
    fold("x = 3 << 1", "x=6");
    fold("x = 1 << 8", "x=256");

    fold("x = 1 >> 0", "x=1");
    fold("x = -1 >> 0", "x=-1");
    fold("x = 1 >> 1", "x=0");
    fold("x = 2 >> 1", "x=1");
    fold("x = 5 >> 1", "x=2");
    fold("x = 127 >> 3", "x=15");
    fold("x = 3 >> 1", "x=1");
    fold("x = 3 >> 2", "x=0");
    fold("x = 10 >> 1", "x=5");
    fold("x = 10 >> 2", "x=2");
    fold("x = 10 >> 5", "x=0");

    fold("x = 10 >>> 1", "x=5");
    fold("x = 10 >>> 2", "x=2");
    fold("x = 10 >>> 5", "x=0");
    fold_same("x = -1 >>> 1");
    fold_same("x = -1 >>> 0");
    fold_same("x = -2 >>> 0");
    fold("x = 0x90000000 >>> 28", "x=9");

    fold("x = 0xffffffff << 0", "x=-1");
    fold("x = 0xffffffff << 4", "x=-16");
    fold("1 << 32", "1");
    fold("1 << -1", "1<<-1");
    fold("1 >> 32", "1");

    // Regression on #6161, ported from <https://github.com/tc39/test262/blob/05c45a4c430ab6fee3e0c7f0d47d8a30d8876a6d/test/language/expressions/unsigned-right-shift/S9.6_A2.2.js>.
    fold("-2147483647 >>> 0", "2147483649");
    fold("-2147483648 >>> 0", "2147483648");
    fold("-2147483649 >>> 0", "2147483647");
    fold("-4294967295 >>> 0", "1");
    fold("-4294967296 >>> 0", "0");
    fold("-4294967297 >>> 0", "4294967295");
    fold("4294967295 >>> 0", "4294967295");
    fold("4294967296 >>> 0", "0");
    fold("4294967297 >>> 0", "1");
    fold("8589934591 >>> 0", "4294967295");
    fold("8589934592 >>> 0", "0");
    fold("8589934593 >>> 0", "1");

    fold("x = -1 << 1", "x = -2");
    fold("x = -1 << 8", "x = -256");
    fold("x = -1 >> 1", "x = -1");
    fold("x = -2 >> 1", "x = -1");
    fold("x = -1 >> 0", "x = -1");
}

#[test]
fn test_string_add() {
    fold("x = 'a' + 'bc'", "x = 'abc'");
    fold("x = 'a' + 5", "x = 'a5'");
    fold("x = 5 + 'a'", "x = '5a'");
    fold("x = 'a' + 5n", "x = 'a5'");
    fold("x = 5n + 'a'", "x = '5a'");
    fold("x = 'a' + ''", "x = 'a'");
    fold("x = 'a' + foo()", "x = 'a'+foo()");
    fold("x = foo() + 'a' + 'b'", "x = foo()+'ab'");
    fold("x = (foo() + 'a') + 'b'", "x = foo()+'ab'"); // believe it!
    fold("x = foo() + 'a' + 'b' + 'cd' + bar()", "x = foo()+'abcd'+bar()");
    fold("x = foo() + 2 + 'b'", "x = foo()+2+\"b\""); // don't fold!

    fold("x = foo() + 'a' + 2", "x = foo()+\"a2\"");
    fold("x = '' + null", "x = 'null'");
    fold("x = true + '' + false", "x = 'truefalse'");
    fold("x = '' + []", "x = ''");
    fold("x = foo() + 'a' + 1 + 1", "x = foo() + 'a11'");
    fold("x = 1 + 1 + 'a'", "x = '2a'");
    fold("x = 1 + 1 + 'a'", "x = '2a'");
    fold("x = 'a' + (1 + 1)", "x = 'a2'");
    // fold("x = '_' + p1 + '_' + ('' + p2)", "x = '_' + p1 + '_' + p2");
    fold("x = 'a' + ('_' + 1 + 1)", "x = 'a_11'");
    fold("x = 'a' + ('_' + 1) + 1", "x = 'a_11'");
    // fold("x = 1 + (p1 + '_') + ('' + p2)", "x = 1 + (p1 + '_') + p2");
    // fold("x = 1 + p1 + '_' + ('' + p2)", "x = 1 + p1 + '_' + p2");
    fold("x = 1 + 'a' + p1", "x = '1a' + p1");
    // fold("x = (p1 + (p2 + 'a')) + 'b'", "x = (p1 + (p2 + 'ab'))");
    // fold("'a' + ('b' + p1) + 1", "'ab' + p1 + 1");
    // fold("x = 'a' + ('b' + p1 + 'c')", "x = 'ab' + (p1 + 'c')");
    fold("void 0 + ''", "'undefined'");

    fold("`${a}` + `${b}`", "`${a}${b}`");
    fold("`${a}` + `${b}b`", "`${a}${b}b`");
    fold("`${a}` + `b${b}`", "`${a}b${b}`");
    fold("`${a}a` + `${b}`", "`${a}a${b}`");
    fold("`${a}a` + `${b}b`", "`${a}a${b}b`");
    fold("`${a}a` + `b${b}`", "`${a}ab${b}`");
    fold("`a${a}` + `${b}`", "`a${a}${b}`");
    fold("`a${a}` + `${b}b`", "`a${a}${b}b`");
    fold("`a${a}` + `b${b}`", "`a${a}b${b}`");
    fold("foo() + `${a}` + `${b}`", "foo() + `${a}${b}`");

    fold_same("x = 'a' + (4 + p1 + 'a')");
    fold_same("x = p1 / 3 + 4");
    fold_same("foo() + 3 + 'a' + foo()");
    fold_same("x = 'a' + ('b' + p1 + p2)");
    fold_same("x = 1 + ('a' + p1)");
    fold_same("x = p1 + '' + p2");
    fold_same("x = 'a' + (1 + p1)");
    fold_same("x = (p2 + 'a') + (1 + p1)");
    fold_same("x = (p2 + 'a') + (1 + p1 + p2)");
    fold_same("x = (p2 + 'a') + (1 + (p1 + p2))");
}

#[test]
fn test_fold_arithmetic() {
    fold("1n+ +1n", "1n + +1n");
    fold("1n- -1n", "1n - -1n");
    fold("a- -b", "a - -b");
}

#[test]
fn test_fold_arithmetic_infinity() {
    fold("x=-Infinity-2", "x=-Infinity");
    fold("x=Infinity-2", "x=Infinity");
    fold("x=Infinity*5", "x=Infinity");
    fold("x = Infinity ** 2", "x = Infinity");
    fold("x = Infinity ** -2", "x = 0");

    fold("x = Infinity % Infinity", "x = NaN");
    fold("x = Infinity % 0", "x = NaN");
}

#[test]
fn test_fold_add() {
    fold("x = 10 + 20", "x = 30");
    fold_same("x = y + 10 + 20");
    fold("x = 1 + null", "x = 1");
    fold("x = null + 1", "x = 1");
}

#[test]
fn test_fold_sub() {
    fold("x = 10 - 20", "x = -10");
}

#[test]
fn test_fold_multiply() {
    fold_same("x = 2.25 * 3");
    fold_same("z = x * y");
    fold_same("x = y * 5");
    // test("x = null * undefined", "x = NaN");
    // test("x = null * 1", "x = 0");
    // test("x = (null - 1) * 2", "x = -2");
    // test("x = (null + 1) * 2", "x = 2");
    // test("x = y + (z * 24 * 60 * 60 * 1000)", "x = y + z * 864E5");
    fold("x = y + (z & 24 & 60 & 60 & 1000)", "x = y + (z & 8)");
    fold("x = -1 * -1", "x = 1");
    fold("x = 1 * -1", "x = -1");
    fold("x = 255 * 255", "x = 65025");
    fold("x = -255 * 255", "x = -65025");
    fold("x = -255 * -255", "x = 65025");
    fold_same("x = 256 * 255");
}

#[test]
fn test_fold_division() {
    fold("x = Infinity / Infinity", "x = NaN");
    fold("x = Infinity / 0", "x = Infinity");
    fold("x = 1 / 0", "x = Infinity");
    fold("x = 0 / 0", "x = NaN");
    fold_same("x = 2 / 4");
    fold_same("x = y / 2 / 4");
}

#[test]
fn test_fold_remainder() {
    fold_same("x = 3 % 2");
    fold_same("x = 3 % -2");
    fold_same("x = -1 % 3");
    fold("x = 1 % 0", "x = NaN");
    fold("x = 0 % 0", "x = NaN");
}

#[test]
fn test_fold_exponential() {
    fold_same("x = 2 ** 3");
    fold_same("x = 2 ** -3");
    fold_same("x = 2 ** 55");
    fold_same("x = 3 ** -1");
    fold_same("x = (-1) ** 0.5");
    fold("x = (-0) ** 3", "x = -0");
    fold_same("x = null ** 0");
}

#[test]
fn test_fold_shift_left() {
    fold("1 << 3", "8");
    fold("1.2345 << 0", "1");
    fold_same("1 << 24");
}

#[test]
fn test_fold_shift_right() {
    fold("2147483647 >> -32.1", "2147483647");
}

#[test]
fn test_fold_shift_right_zero_fill() {
    fold("10 >>> 1", "5");
    fold_same("-1 >>> 0");
}

#[test]
fn test_fold_left() {
    fold("(+x - 1) + 2", "x - 1 + 2"); // not yet
    fold("(+x & 1) & 2", "x & 0");
}

#[test]
fn test_fold_array_length() {
    // Can fold
    fold("x = [].length", "x = 0");
    fold("x = [1,2,3].length", "x = 3");
    // test("x = [a,b].length", "x = 2");
    fold("x = 'abc'['length']", "x = 3");

    // Not handled yet
    fold("x = [,,1].length", "x = 3");

    // Cannot fold
    fold("x = [foo(), 0].length", "x = [foo(),0].length");
    fold_same("x = y.length");
    fold_same("[...[1, 2, 3]].length");
}

#[test]
fn test_fold_string_length() {
    // Can fold basic strings.
    fold("x = ''.length", "x = 0");
    fold("x = '123'.length", "x = 3");

    // Test Unicode escapes are accounted for.
    fold("x = '123\\u01dc'.length", "x = 4");
}

#[test]
fn test_fold_instance_of() {
    // Non object types are never instances of anything.
    fold("64 instanceof Object", "!1");
    fold("64 instanceof Number", "!1");
    fold("'' instanceof Object", "!1");
    fold("'' instanceof String", "!1");
    fold("true instanceof Object", "!1");
    fold("true instanceof Boolean", "!1");
    fold("!0 instanceof Object", "!1");
    fold("!0 instanceof Boolean", "!1");
    fold("false instanceof Object", "!1");
    fold("null instanceof Object", "!1");
    fold("undefined instanceof Object", "!1");
    fold("NaN instanceof Object", "!1");
    fold("Infinity instanceof Object", "!1");

    // Array and object literals are known to be objects.
    fold("[] instanceof Object", "!0");
    fold("({}) instanceof Object", "!0");

    // These cases is foldable, but no handled currently.
    fold_same("new Foo() instanceof Object");
    // These would require type information to fold.
    fold_same("[] instanceof Foo");
    fold_same("({}) instanceof Foo");

    fold("(function() {}) instanceof Object", "!0");

    // An unknown value should never be folded.
    fold_same("x instanceof Foo");
    test_same("var x; foo(x instanceof Object)");
    fold_same("x instanceof Object");
    fold_same("0 instanceof Foo");
}

#[test]
fn test_fold_instance_of_additional() {
    fold("(typeof {}) instanceof Object", "!1");
    fold("(+{}) instanceof Number", "!1");
    fold_same("({ __proto__: null }) instanceof Object");
    fold("/foo/ instanceof Object", "!0");
    fold("(() => {}) instanceof Object", "!0");
    fold("(function(){}) instanceof Object", "!0");
    fold("(class{}) instanceof Object", "!0");
}

#[test]
fn test_fold_left_child_op() {
    fold("x & Infinity & 2", "x & 0");
    fold_same("x - Infinity - 2"); // FIXME: want "x-Infinity"
    fold_same("x - 1 + Infinity");
    fold_same("x - 2 + 1");
    fold_same("x - 2 + 3");
    fold_same("1 + x - 2 + 1");
    fold_same("1 + x - 2 + 3");
    fold_same("1 + x - 2 + 3 - 1");
    fold_same("f(x)-0");
    fold_same("x-0-0"); // FIXME: want x - 0
    fold_same("x+2-2+2");
    fold_same("x+2-2+2-2");
    fold_same("x-2+2");
    fold_same("x-2+2-2");
    fold_same("x-2+2-2+2");

    fold_same("1+x-0-na_n");
    fold_same("1+f(x)-0-na_n");
    fold_same("1+x-0+na_n");
    fold_same("1+f(x)-0+na_n");

    fold_same("1+x+na_n"); // unfoldable
    fold_same("x+2-2"); // unfoldable
    fold_same("x+2"); // nothing to do
    fold_same("x-2"); // nothing to do
}

#[test]
fn test_associative_fold_constants_with_variables() {
    // mul and add should not fold
    fold_same("alert(x * 12 * 20)");
    fold_same("alert(12 * x * 20)");
    fold_same("alert(x + 12 + 20)");
    fold_same("alert(12 + x + 20)");
    fold("alert(x & 12 & 20)", "alert(x & 4)");
    fold("alert(12 & x & 20)", "alert(x & 4)");
}

#[test]
fn test_to_number() {
    fold("x = +''", "x = 0");
    fold("x = +'+Infinity'", "x = Infinity");
    fold("x = +'-Infinity'", "x = -Infinity");

    for op in ["", "+", "-"] {
        for s in ["inf", "infinity", "INFINITY", "InFiNiTy"] {
            fold(&format!("x = +'{op}{s}'"), "x = NaN");
        }
    }
}

#[test]
fn test_number_constructor() {
    fold("Number(undefined)", "NaN");
    fold("Number(void 0)", "NaN");
    fold("Number(null)", "0");
    fold("Number(true)", "1");
    fold("Number(false)", "0");
    fold("Number('a')", "NaN");
    fold("Number('1')", "1");
    test_same("var Number; NOOP(Number(1))");
}

#[test]
fn test_fold_useless_string_addition() {
    fold_same("typeof foo");
    fold_same("typeof foo + '123'");
    fold("typeof foo + ''", "typeof foo");
    fold("'' + typeof foo", "typeof foo");
    fold("typeof foo + ``", "typeof foo");
    fold("`` + typeof foo", "typeof foo");
    fold("typeof foo + []", "typeof foo");
    fold("[] + typeof foo", "typeof foo");
    fold("(foo ? 'a' : 'b') + ''", "foo ? 'a' : 'b'");
    fold_same("typeof foo - ''");
}

#[test]
fn test_fold_same_typeof() {
    fold("typeof foo === typeof bar", "typeof foo == typeof bar");
    fold("typeof foo !== typeof bar", "typeof foo != typeof bar");
    fold("typeof foo.bar === typeof foo.bar", "typeof foo.bar == typeof foo.bar");
    fold("typeof foo.bar !== typeof foo.bar", "typeof foo.bar != typeof foo.bar");
}

#[test]
fn test_fold_invalid_typeof_comparison() {
    fold("typeof foo == 123", "!1");
    fold("typeof foo == '123'", "!1");
    fold("typeof foo === null", "!1");
    fold("typeof foo === undefined", "!1");
    fold("typeof foo !== 123", "!0");
    fold("typeof foo !== '123'", "!0");
    fold("typeof foo != null", "!0");
    fold("typeof foo != undefined", "!0");
    fold("typeof foo === 'string'", "typeof foo == 'string'");
    fold("typeof foo === 'number'", "typeof foo == 'number'");
}

#[test]
fn test_issue_8782() {
    fold("+(void unknown())", "+void unknown()");
}

#[test]
fn test_inline_values_in_template_literal() {
    fold("`foo${1}`", "'foo1'");
    fold("`foo${1}bar`", "'foo1bar'");
    fold("`foo${1}bar${2}baz`", "'foo1bar2baz'");
    fold("`foo${1}bar${2}baz${3}qux`", "'foo1bar2baz3qux'");
    fold("`foo${1}${i}`", "`foo1${i}`");
    fold("`foo${'${}'}`", "'foo${}'");
    fold("`foo${'${}'}${i}`", "`foo\\${}${i}`");
    fold_same("foo`foo${1}bar`");
}

mod bigint {
    use super::{
        MAX_SAFE_FLOAT, MAX_SAFE_INT, NEG_MAX_SAFE_FLOAT, NEG_MAX_SAFE_INT, fold, fold_same,
    };

    #[test]
    fn test_fold_bitwise_op_with_big_int() {
        fold("x = 1n & 1n", "x = 1n");
        fold("x = 1n & 2n", "x = 0n");
        fold("x = 3n & 1n", "x = 1n");
        fold("x = 3n & 3n", "x = 3n");

        fold("x = 1n | 1n", "x = 1n");
        fold("x = 1n | 2n", "x = 3n");
        fold("x = 1n | 3n", "x = 3n");
        fold("x = 3n | 1n", "x = 3n");
        fold("x = 3n | 3n", "x = 3n");
        fold("x = 1n | 4n", "x = 5n");

        fold("x = 1n ^ 1n", "x = 0n");
        fold("x = 1n ^ 2n", "x = 3n");
        fold("x = 3n ^ 1n", "x = 2n");
        fold("x = 3n ^ 3n", "x = 0n");

        fold("x = -1n & 0n", "x = 0n");
        fold("x = 0n & -1n", "x = 0n");
        fold("x = 1n & 4n", "x = 0n");
        fold("x = 2n & 3n", "x = 2n");

        fold("x = 1n & 3000000000n", "x = 0n");
        fold("x = 3000000000n & 1n", "x = 0n");

        // bitwise OR does not affect the sign of a bigint
        fold("x = 1n | 3000000001n", "x = 3000000001n");
        fold("x = 4294967295n | 0n", "x = 4294967295n");

        fold("x = y & 1n & 1n", "x = y & 1n");
        fold("x = y & 1n & 2n", "x = y & 0n");
        fold("x = y & 3n & 1n", "x = y & 1n");
        fold("x = 3n & y & 1n", "x = y & 1n");
        fold("x = y & 3n & 3n", "x = y & 3n");
        fold("x = 3n & y & 3n", "x = y & 3n");

        fold("x = y | 1n | 1n", "x = y | 1n");
        fold("x = y | 1n | 2n", "x = y | 3n");
        fold("x = y | 3n | 1n", "x = y | 3n");
        fold("x = 3n | y | 1n", "x = y | 3n");
        fold("x = y | 3n | 3n", "x = y | 3n");
        fold("x = 3n | y | 3n", "x = y | 3n");

        fold("x = y ^ 1n ^ 1n", "x = y ^ 0n");
        fold("x = y ^ 1n ^ 2n", "x = y ^ 3n");
        fold("x = y ^ 3n ^ 1n", "x = y ^ 2n");
        fold("x = 3n ^ y ^ 1n", "x = y ^ 2n");
        fold("x = y ^ 3n ^ 3n", "x = y ^ 0n");
        fold("x = 3n ^ y ^ 3n", "x = y ^ 0n");

        // TypeError: Cannot mix BigInt and other types
        fold_same("1n & 1");
        fold_same("1n | 1");
        fold_same("1n ^ 1");
    }

    #[test]
    fn test_bigint_number_comparison() {
        fold("1n < 2", "!0");
        fold("1n > 2", "!1");
        fold("1n == 1", "!0");
        fold("1n == 2", "!1");

        // comparing with decimals is allowed
        fold("1n < 1.1", "!0");
        fold("1n < 1.9", "!0");
        fold("1n < 0.9", "!1");
        fold("-1n < -1.1", "!1");
        fold("-1n < -1.9", "!1");
        fold("-1n < -0.9", "!0");
        fold("1n > 1.1", "!1");
        fold("1n > 0.9", "!0");
        fold("-1n > -1.1", "!0");
        fold("-1n > -0.9", "!1");

        // Don't fold unsafely large numbers because there might be floating-point error
        fold(&format!("0n > {MAX_SAFE_INT}"), "!1");
        fold(&format!("0n < {MAX_SAFE_INT}"), "!0");
        fold(&format!("0n > {NEG_MAX_SAFE_INT}"), "!0");
        fold(&format!("0n < {NEG_MAX_SAFE_INT}"), "!1");
        fold(&format!("0n > {MAX_SAFE_FLOAT}"), "!1");
        fold(&format!("0n < {MAX_SAFE_FLOAT}"), "!0");
        fold(&format!("0n > {NEG_MAX_SAFE_FLOAT}"), "!0");
        fold(&format!("0n < {NEG_MAX_SAFE_FLOAT}"), "!1");

        // comparing with Infinity is allowed
        fold("1n < Infinity", "!0");
        fold("1n > Infinity", "!1");
        fold("1n < -Infinity", "!1");
        fold("1n > -Infinity", "!0");

        // null is interpreted as 0 when comparing with bigint
        fold("1n < null", "!1");
        fold("1n > null", "!0");
    }

    #[test]
    fn test_bigint_string_comparison() {
        fold("1n < '2'", "!0");
        fold("2n > '1'", "!0");
        fold("123n > '34'", "!0");
        fold("1n == '1'", "!0");
        fold("1n == '2'", "!1");
        fold("1n != '1'", "!1");
        fold("1n === '1'", "!1");
        fold("1n !== '1'", "!0");
    }

    #[test]
    fn test_string_bigint_comparison() {
        fold("'1' < 2n", "!0");
        fold("'2' > 1n", "!0");
        fold("'123' > 34n", "!0");
        fold("'1' == 1n", "!0");
        fold("'1' == 2n", "!1");
        fold("'1' != 1n", "!1");
        fold("'1' === 1n", "!1");
        fold("'1' !== 1n", "!0");
    }

    #[test]
    fn test_object_bigint_comparison() {
        fold_same("{ valueOf: function() { return 0n; } } != 0n");
        fold_same("{ toString: function() { return '0'; } } != 0n");
    }

    #[test]
    fn test_fold_object_spread() {
        fold_same("({ z, ...a })");
        let result = "({ z })";
        fold("({ z, ...[] })", result);
        fold("({ z, ...{} })", result);
        fold("({ z, ...undefined })", result);
        fold("({ z, ...void 0 })", result);
        fold("({ z, ...null })", result);
        fold("({ z, ...true })", result);
        fold("({ z, ...!0 })", result);
        fold("({ z, ...!1 })", result);
        fold("({ z, ...1 })", result);
        fold("({ z, ...1n })", result);
        fold("({ z, .../asdf/ })", result);
        fold("({ z, ...()=>{} })", result);
        fold("({ z, ...function(){} })", result);
        fold_same("({ z, ...'abc' })");
        fold("({ a: 0, ...{ b: 1 } })", "({ a: 0, b: 1 })");
        fold("({ a: 0, ...{ b: 1, ...{ c: 2 } } })", "({ a: 0, b: 1, c: 2 })");
        fold("({ a: 0, ...{ a: 1 } })", "({ a: 0, a: 1 })"); // can be fold to `({ a: 1 })`
        fold("({ a: foo(), ...{ a: bar() } })", "({ a: foo(), a: bar() })"); // can be fold to `({ a: (foo(), bar()) })`
        fold_same("({ ...{ get a() { return 0 } } })");
        fold("({ ...{ __proto__: null } })", "({})");
        fold("({ ...{ '__proto__': null } })", "({})");
        fold_same("({ a: foo(), ...{ __proto__: bar() }, b: baz() })"); // can be folded to `({ a: foo(), b: (bar(), baz()) })`
        fold("({ ...{ __proto__() {} } })", "({ __proto__() {} })");
        fold("({ ...{ ['__proto__']: null } })", "({ ['__proto__']: null })");
    }
}
