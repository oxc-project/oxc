//! <https://github.com/google/closure-compiler/blob/master/test/com/google/javascript/jscomp/PeepholeFoldConstantsTest.java>

use crate::test;

#[test]
fn undefined_comparison1() {
    test("undefined == undefined", "!0");
}
