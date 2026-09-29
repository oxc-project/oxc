use crate::test;

// wrap with a function call so it doesn't get removed.
fn fold(source_text: &str, expected: &str) {
    let source_text = format!("NOOP({source_text})");
    let expected = format!("NOOP({expected})");
    test(&source_text, &expected);
}

fn fold_same(source_text: &str) {
    fold(source_text, source_text);
}

#[test]
fn minimize_bitwise_binary_expr() {
    // `(a OP b) | 0` -> `a OP b`.
    fold_same("a << b | 0");
    fold_same("0 | (a << b)");
    fold_same("a >> b | 0");
    fold_same("0 | (a >> b)");
    fold_same("a >>> b | 0");
    fold_same("0 | (a >>> b)");
    fold_same("(-1 >>> a) | b");
    fold_same("a | b | 0");
    fold_same("0 | (a | b)");
    fold_same("a ^ b | 0");
    fold_same("0 | (a ^ b)");
    fold_same("a & b | 0");
    fold_same("0 | (a & b)");

    // `(a | 0) OP b` and `a OP (b | 0)`.
    fold("((1 | a) | 0) << b", "(a | 1) << b");
    fold("a << ((1 | b) | 0)", "a << (b | 1)");
    fold("((1 | a) | 0) >> b", "(a | 1) >> b");
    fold("a >> ((1 | b) | 0)", "a >> (b | 1)");
    fold("((1 | a) | 0) >>> b", "(a | 1) >>> b");
    fold("a >>> ((1 | b) | 0)", "a >>> (b | 1)");
    fold("((1 | a) | 0) | b", "(a | 1) | b");
    fold("a | ((1 | b) | 0)", "a | b | 1");
    fold("((1 | a) | 0) ^ b", "(a | 1) ^ b");
    fold("a ^ ((1 | b) | 0)", "a ^ (b | 1)");
    fold("((1 | a) | 0) & b", "(a | 1) & b");
    fold("a & ((1 | b) | 0)", "a & (b | 1)");

    fold_same("(a | 0) << b");
    fold_same("a << (b | 0)");
    fold_same("(a | 0) >> b");
    fold_same("a >> (b | 0)");
    fold_same("(a | 0) >>> b");
    fold_same("a >>> (b | 0)");
    fold_same("(a | 0) | b");
    fold("a | (b | 0)", "a | b | 0");
    fold_same("(a | 0) ^ b");
    fold_same("a ^ (b | 0)");
    fold_same("(a | 0) & b");
    fold_same("a & (b | 0)");

    // `a OP 0` -> `a | 0`
    fold("a << 0", "a | 0");
    fold("a >> 0", "a | 0");
    fold("a ^ 0", "a | 0");

    // `a & 0xffffffff` -> `a | 0`.
    fold("a & 0xffffffff", "a | 0");
    fold("4294967295 & a", "0 | a");
    fold("a & 4294967294.9999999", "a | 0");
    fold_same("a & 4294967294.999999");

    fold_same("a + 0");
    fold_same("a | 1");
    fold_same("a & 1");
    fold_same("a << 1");
    fold_same("a | (b + 0)");
    fold_same("a | ~b");
    fold_same("~a | 0");
    fold_same("0 | ~a");
}
