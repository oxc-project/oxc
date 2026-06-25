use crate::{test, test_same};

#[test]
fn minimize_duplicate_nots() {
    test("!x", "x");
    test("!!x", "x");
    test("!!!x", "x");
    test("!!!!x", "x");
    test("!!!(x && y)", "x && y");
    test("var k = () => { !!x; }", "var k = () => { x }");

    test_same("var k = !!x;");
    test_same("function k () { return !!x; }");
    test("var k = () => { return !!x; }", "var k = () => !!x");
    test_same("var k = () => !!x;");
}

#[test]
fn minimize_nots_with_binary_expressions() {
    test("!(x === undefined)", "x");
    test("!(typeof(x) === 'undefined')", "");
    test("!(typeof(x()) === 'undefined')", "x()");
    test("!(x === void 0)", "x");
    test("!!delete x.y", "delete x.y");
    test("!!!delete x.y", "delete x.y");
    test("!!!!delete x.y", "delete x.y");
    test("var k = !!(foo instanceof bar)", "var k = foo instanceof bar");
    test("!(a === 1 ? void 0 : a.b)", "a !== 1 && a.b;");
    test("!(a, b)", "a, b");
}
