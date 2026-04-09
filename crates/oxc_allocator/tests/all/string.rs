#![cfg(feature = "collections")]
use bumpalo::{collections::String, format, Bump};
use std::fmt::Write;

#[test]
fn format_a_bunch_of_strings() {
    let b = Bump::new();
    let mut s = String::from_str_in("hello", &b);
    for i in 0..1000 {
        write!(&mut s, " {}", i).unwrap();
    }
}

#[test]
fn trailing_comma_in_format_macro() {
    let b = Bump::new();
    let v = format![in &b, "{}{}", 1, 2, ];
    assert_eq!(v, "12");
}

#[test]
fn push_str() {
    let b = Bump::new();
    let mut s = String::new_in(&b);
    s.push_str("abc");
    assert_eq!(s, "abc");
    s.push_str("def");
    assert_eq!(s, "abcdef");
    s.push_str("");
    assert_eq!(s, "abcdef");
    s.push_str(&"x".repeat(4000));
    assert_eq!(s.len(), 4006);
    s.push_str("ghi");
    assert_eq!(s.len(), 4009);
    assert_eq!(&s[s.len() - 5..], "xxghi");
}
