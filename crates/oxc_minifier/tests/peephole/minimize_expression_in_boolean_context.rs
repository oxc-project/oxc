use crate::test;

fn test_boolean(source_text: &str, expected: &str) {
    test(format!("for(;{source_text};);").as_str(), format!("for(;{expected};);").as_str());
}

#[test]
fn test_minimize_expression_in_boolean_context() {
    test_boolean("!!a", "a");
    test_boolean("!!a ? b : c", "a ? b : c");
    test_boolean("!!!a", "!a");
    test_boolean("Boolean(!!a)", "a");
    test_boolean("((a | +b) !== 0)", "a | +b");
    test_boolean("(a | +b) === 0", "!(a | +b)");
    test_boolean("!!a && !!b", "a && b");
    test_boolean("!!a || !!b", "a || b");
    test_boolean("anything || (0, false)", "anything");
    test_boolean("a ? !!b : !!c", "a ? b : c");
    test_boolean("foo, !!bar", "foo, bar");
    test_boolean("anything1 ? (0, true) : anything2", "anything1 || anything2");
    test_boolean("anything1 ? (0, false) : anything2", "!anything1 && anything2");
    test_boolean("anything1 ? anything2 : (0, true)", "!anything1 || anything2");
    test_boolean("anything1 ? anything2 : (0, false)", "anything1 && anything2");
    test_boolean("+a === 0", "+a == 0");
}

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
