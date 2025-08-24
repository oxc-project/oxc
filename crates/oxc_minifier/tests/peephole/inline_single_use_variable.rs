use super::test_same;

#[test]
fn test_inline_single_use_variable() {
    test_same("function wrapper(arg0, arg1) {using x = foo; return x}");
    test_same("async function wrapper(arg0, arg1) { await using x = foo; return x}");
}
