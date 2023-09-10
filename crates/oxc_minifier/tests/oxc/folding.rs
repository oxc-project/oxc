use crate::{test, test_snapshot};

#[test]
fn addition_folding() {
    test("1 + 1", "2");
    test("1 + 1 + 1", "3");
    test("0 + true", "1");
    test("x+''", "x+''");
}

#[test]
fn addition_folding_snapshots() {
    test_snapshot(
        "addition_folding",
        [
            "let x = 1 + 1;",
            "function foo() { return 1 + 1; }",
            "'' + true",
            "'' + false",
            "'' + null",
            "false + null",
            "'1' + '1'",
            "NaN + NaN",
            "'' + NaN",
            // identifiers
            "let x = 1; let y = x + 1;",
            "var x = 1; x + 1 === 2",
            "var y = 1; 1 + y === 2",
            "null - Number(1)"
        ],
    );
}
