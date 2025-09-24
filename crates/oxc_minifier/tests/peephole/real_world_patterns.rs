use crate::{test, test_same};

/// Focused minification tests for specific patterns and edge cases
/// Tests demonstrate actual oxc minifier capabilities

#[test]
fn test_function_call_optimization() {
    // String constructors - as standalone expressions they get eliminated
    test("String(42)", "");
    test("String(true)", "");
    test("String(null)", "");

    // In return context they are optimized
    test("return String(42)", "return '42'");
    test("return String(true)", "return 'true'");
    test("return String(null)", "return 'null'");

    // Boolean constructors get optimized
    test("return Boolean(1)", "return !0");
    test("return Boolean(0)", "return !1");
    test("return Boolean('')", "return !1");

    // Number constructors - eliminated as unused expressions
    test("Number('42')", "");
    test("Number(true)", "");
}

#[test]
fn test_property_access_optimization() {
    // Bracket to dot notation
    test("obj['property']", "obj.property");
    test("obj['validName123']", "obj.validName123");
    test("obj['$special']", "obj.$special");

    // Cases that should NOT be optimized
    test_same("obj['123invalid']"); // starts with number
    test_same("obj['key-with-dash']"); // contains dash
    test_same("obj['key with space']"); // contains space
}

#[test]
fn test_logical_operator_optimization() {
    // Short-circuit evaluation
    test("true && foo()", "foo()");
    test("false && foo()", "");
    test("true || foo()", "");
    test("false || foo()", "foo()");

    // Boolean constants - as standalone expressions get eliminated
    test("!!true", "");
    test("!!false", "");
    test("!true", "");
    test("!false", "");

    // In return context they are optimized
    test("return !!true", "return !0");
    test("return !!false", "return !1");
    test("return !true", "return !1");
    test("return !false", "return !0");
}

#[test]
fn test_conditional_optimization() {
    // Ternary with constants
    test("true ? foo() : bar()", "foo()");
    test("false ? foo() : bar()", "bar()");

    // If statements with constants
    test("if (true) foo();", "foo();");
    test("if (false) foo();", "");
    test("if (true) foo(); else bar();", "foo();");
    test("if (false) foo(); else bar();", "bar();");
}

#[test]
fn test_assignment_optimization() {
    // Self-assignment patterns
    test("x = x + 1", "x += 1");
    test("x = x - 1", "--x");
    test("x = x * 2", "x *= 2");
    test("x = x + y", "x += y"); // this IS optimized
}

#[test]
fn test_string_concatenation() {
    // String literals - these become unused expressions and get eliminated
    test("'hello ' + 'world'", "");
    test("'count: ' + 42", "");
    test("42 + ' items'", "");

    // In return context they are optimized
    test("return 'hello ' + 'world'", "return 'hello world'");
    test("return 'count: ' + 42", "return 'count: 42'");
    test("return 42 + ' items'", "return '42 items'");

    // Side effects are preserved but strings might be optimized
    test("getValue() + 'suffix'", "getValue() + ''");
    test("'prefix' + sideEffect()", "'' + sideEffect();");
}

#[test]
fn test_optimization_boundaries() {
    // Cases where optimization should be careful
    test_same("eval('code')"); // should never optimize eval
    test("with (obj) { prop = value; }", "with(obj) prop = value;"); // braces removed but preserved
    test_same("delete obj.prop"); // side effects
    test_same("obj.method()"); // potential side effects
}
