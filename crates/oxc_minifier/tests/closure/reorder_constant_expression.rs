//! [PeepholeReorderConstantExpression](https://github.com/google/closure-compiler/blob/master/test/com/google/javascript/jscomp/PeepholeReorderConstantExpressionTest.java)

use crate::expect;

#[test]
fn symmetric_operations() {
    set1_tests_same("==");
    set2_tests("==");
    set3_tests("==");

    set1_tests_same("!=");
    set2_tests("!=");
    set3_tests("!=");

    set1_tests_same("===");
    set2_tests("===");
    set3_tests("===");

    set1_tests_same("!==");
    set2_tests("!==");
    set3_tests("!==");

    // TODO: need to check operator precedence
    set1_tests_same("*");
    set2_tests("*");
    set3_tests("*");
}

#[test]
fn reorder_constant_doesnt_add_parens() {
    expect("a % b * 4", "a%b*4");
    expect("a * b * 4", "a*b*4");
}

fn set1_tests_same(op: &str) {
    set1_tests(op, op);
}

// This set has a mutable on the right and an Immutable on the left.
// Applies for relational and symmetric operations.
fn set1_tests(op1: &str, op2: &str) {
    expect(&format!("a {op1} 0"), &format!("0{op2}a"));
    expect(&format!("a {op1} '0'"), &format!("'0'{op1}a"));
    expect(&format!("a  {op1}  ''"), &format!("''{op2}a"));
    expect(&format!("a {op1} -1.0"), &format!("-1.0{op2}a"));

    expect(&format!("function f(a){{a {op1} 0}}"), &format!("function f(a){{0{op2}a}}"));
    expect(&format!("f() {op1} 0"), &format!("0{op2}f()"));
    expect(&format!("(a + b) {op1} 0"), &format!("0{op2}(a+b)"));
    expect(&format!("(a + 1) {op1} 0"), &format!("0{op2}(a+1)"));

    expect(&format!("x++ {op1} 0"), &format!("0{op2}x++"));
    expect(
        &format!("x = 0; function f(){{x++; return x}}; f() {op1} 0"),
        &format!("x=0;function f(){{x++;return x}}0{op2}f()"),
    );
}

// This set has a mutable on the right and an Immutable on the left.
// Applies only for symmetric operations.
fn set2_tests(op: &str) {
    expect(&format!("a {op} NaN"), &format!("NaN{op}a"));
    // expect(&format!("a {op} Infinity"), &format!("(1/0){op}a"));

    expect(&format!("NaN {op} a"), &format!("NaN{op}a"));
    // expect(&format!("Infinity {op} a"), &format!("(1/0){op}a"));
}

// This set has an the immutable on the left already, or both non-immutable.
fn set3_tests(op: &str) {
    expect(&format!("0 {op} a"), &format!("0{op}a"));
    expect(&format!("'0' {op} a"), &format!("'0'{op}a"));
    expect(&format!("'' {op} a"), &format!("''{op}a"));
    expect(&format!("-1.0 {op} a"), &format!("-1.0{op}a"));

    expect(&format!("0 {op} 1"), &format!("0{op}1"));

    expect(&format!("a {op} b"), &format!("a{op}b"));
}
