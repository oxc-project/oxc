use crate::test_snapshot;

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
        ],
    );
}
