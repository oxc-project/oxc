//! [PeepholeReorderConstantExpression](https://github.com/google/closure-compiler/blob/master/test/com/google/javascript/jscomp/PeepholeReorderConstantExpressionTest.java)

use crate::test;

#[test]
#[ignore]
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
#[ignore]
fn reorder_constant_doesnt_add_parens() {
    test("a % b * 4", "a%b*4");
    test("a * b * 4", "a*b*4");
}

fn set1_tests_same(op: &str) {
    set1_tests(op, op);
}

// This set has a mutable on the right and an Immutable on the left.
// Applies for relational and symmetric operations.
fn set1_tests(op1: &str, op2: &str) {
    test(&format!("a {op1} 0"), &format!("0{op2}a"));
    test(&format!("a {op1} '0'"), &format!("'0'{op1}a"));
    test(&format!("a  {op1}  ''"), &format!("''{op2}a"));
    test(&format!("a {op1} -1.0"), &format!("-1.0{op2}a"));

    test(&format!("function f(a){{a {op1} 0}}"), &format!("function f(a){{0{op2}a}}"));
    test(&format!("f() {op1} 0"), &format!("0{op2}f()"));
    test(&format!("(a + b) {op1} 0"), &format!("0{op2}(a+b)"));
    test(&format!("(a + 1) {op1} 0"), &format!("0{op2}(a+1)"));

    test(&format!("x++ {op1} 0"), &format!("0{op2}x++"));
    test(
        &format!("x = 0; function f(){{x++; return x}}; f() {op1} 0"),
        &format!("x=0;function f(){{x++;return x}}0{op2}f()"),
    );
}

// This set has a mutable on the right and an Immutable on the left.
// Applies only for symmetric operations.
fn set2_tests(op: &str) {
    test(&format!("a {op} NaN"), &format!("NaN{op}a"));
    // expect(&format!("a {op} Infinity"), &format!("(1/0){op}a"));

    test(&format!("NaN {op} a"), &format!("NaN{op}a"));
    // expect(&format!("Infinity {op} a"), &format!("(1/0){op}a"));
}

// This set has an the immutable on the left already, or both non-immutable.
fn set3_tests(op: &str) {
    test(&format!("0 {op} a"), &format!("0{op}a"));
    test(&format!("'0' {op} a"), &format!("'0'{op}a"));
    test(&format!("'' {op} a"), &format!("''{op}a"));
    test(&format!("-1.0 {op} a"), &format!("-1.0{op}a"));

    test(&format!("0 {op} 1"), &format!("0{op}1"));

    test(&format!("a {op} b"), &format!("a{op}b"));
}
