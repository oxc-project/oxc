use crate::test;

#[test]
fn test_try_fold_in_boolean_context() {
    test("if (!!a);", "a");
    test("while (!!a);", "for (;a;);");
    test("do; while (!!a);", "do; while (a);");
    test("for (;!!a;);", "for (;a;);");
    test("!!a ? b : c", "a ? b : c");
    test("if (!!!a);", "a");
    test("Boolean(!!a)", "a");
    test("if ((a | +b) !== 0);", "a | +b");
    test("if ((a | +b) === 0);", "a | +b");
    test("if (!!a && !!b);", "a && b");
    test("if (!!a || !!b);", "a || b");
    test("if (anything || (0, false));", "anything");
    test("if (a ? !!b : !!c);", "a ? b : c");
    test("if (anything1 ? (0, true) : anything2);", "anything1 || anything2");
    test("if (anything1 ? (0, false) : anything2);", "!anything1 && anything2");
    test("if (anything1 ? anything2 : (0, true));", "!anything1 || anything2");
    test("if (anything1 ? anything2 : (0, false));", "anything1 && anything2");
    test("if(!![]);", "");
    test("if (+a === 0) { b } else { c }", "+a == 0 ? b : c"); // should not be folded to `a ? b : c` (`+a` might be NaN)
    test("if (foo, !!bar) { let baz }", "if (foo, bar) { let baz }");
}
