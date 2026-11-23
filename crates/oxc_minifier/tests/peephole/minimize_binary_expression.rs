use crate::{test, test_same};

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

    // `(a | 0) OP n` -> `a OP n` and `n OP (a | 0)` -> `n OP a`.
    fold("(a | 0) << 1", "a << 1");
    fold("1 << (a | 0)", "1 << a");
    fold("(a | 0) >> 1", "a >> 1");
    fold("1 >> (a | 0)", "1 >> a");
    fold("(a | 0) >>> 1", "a >>> 1");
    fold("1 >>> (a | 0)", "1 >>> a");
    fold("(a | 0) | 1", "a | 1");
    fold("1 | (a | 0)", "1 | a");
    fold("(a | 0) ^ 1", "a ^ 1");
    fold("1 ^ (a | 0)", "1 ^ a");
    fold("(a | 0) & 1", "a & 1");
    fold("1 & (a | 0)", "1 & a");
    fold("(0 | a) ^ 1", "a ^ 1");
    fold("1 ^ (0 | a)", "1 ^ a");
    fold("(a | 0) ^ (b | 0)", "(a | 0) ^ b");
    fold("(b = 1) ^ (a | 0)", "(b = 1) ^ a");

    fold_same("(a | 0) ^ (b = 1)");
    fold_same("(a | 1) ^ 2");
    fold_same("2 ^ (a | 1)");
    fold_same("(a | 0) ^ 1n");
    fold_same("1n ^ (a | 0)");
    fold_same("(a | 0) + 1");

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

#[test]
fn bitwise_or_zero_preserves_coercion_order() {
    // The shift can throw a RangeError, but `a | 0` must throw first if `a` is a BigInt.
    test_same(
        "export function f(a, b) { return (a | 0) ^ ((1n << (b ? 1073741824n : 1073741825n)) ? 1 : 2) }",
    );
    // `instanceof` can run a proxy trap while walking the prototype chain.
    test_same(
        "export function f(a, p) { return (a | 0) ^ ({ __proto__: p } instanceof Object ? 1 : 2) }",
    );
}
