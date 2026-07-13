//! String utility functions.

/// Compare two strings for equality in a `const` context.
///
/// `str`'s `==` operator is not `const` (it goes through the non-`const` [`PartialEq`] impl),
/// so this byte-for-byte comparison stands in where a compile-time string equality is needed.
///
/// Do not use this except in const context - standard `==` ([`PartialEq`]) has much better performance.
///
/// ```
/// use oxc_data_structures::str::const_str_eq;
///
/// const _: () = assert!(const_str_eq("foo", "foo"));
/// const _: () = assert!(!const_str_eq("foo", "bar"));
/// ```
pub const fn const_str_eq(a: &str, b: &str) -> bool {
    let a = a.as_bytes();
    let b = b.as_bytes();

    if a.len() != b.len() {
        return false;
    }

    let mut i = 0;
    while i < a.len() {
        if a[i] != b[i] {
            return false;
        }
        i += 1;
    }

    true
}

#[cfg(test)]
mod tests {
    use super::const_str_eq;

    #[test]
    fn equal_and_unequal() {
        assert!(const_str_eq("", ""));
        assert!(const_str_eq("foo", "foo"));
        assert!(const_str_eq("foobar", "foobar"));

        assert!(!const_str_eq("", "foo"));
        assert!(!const_str_eq("foo", ""));
        assert!(!const_str_eq("f", "foo"));
        assert!(!const_str_eq("foo", "f"));
        // Differing lengths
        assert!(!const_str_eq("foo", "foobar"));
        // Same length, differing byte
        assert!(!const_str_eq("foo", "for"));
        assert!(!const_str_eq("foo", "bar"));
    }

    #[test]
    fn usable_in_const_context() {
        const EQ: bool = const_str_eq("foo", "foo");
        const NE: bool = const_str_eq("foo", "bar");
        assert_eq!((EQ, NE), (true, false));
    }
}
