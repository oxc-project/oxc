use std::ops::Deref;

use crate::{ast::Pattern, Parser, ParserOptions};
use oxc_allocator::Allocator;
use oxc_span::cmp::ContentEq;

fn parse<'a>(alloc: &'a Allocator, regex: &'a str) -> Pattern<'a> {
    Parser::new(alloc, regex, ParserOptions::default()).parse().unwrap()
}

fn test_cases() -> impl Iterator<Item = (&'static str, &'static str)> {
    use std::iter;
    let cases = vec!["abc", "^abc?", "[a-z]", "a{1,2}", "a{1,}", "a(?=b)"];

    // [1, 2, 3]
    let first = cases.iter();
    // [2, 3, 1]
    let second = first.clone().skip(1).chain(iter::once(&cases[0]));

    first.zip(second).copied()
}
#[test]
fn test_content_eq() {
    use std::iter;
    let cases = vec!["abc", "^abc?", "[a-z]", "a{1,2}", "a{1,}", "a(?=b)"];

    let alloc = Allocator::default();
    // [1, 2, 3]
    let first = cases.iter();
    // [2, 3, 1]
    let second = first.clone().skip(1).chain(iter::once(&cases[0]));
    for (first_source, second_source) in first.zip(second) {
        let first = parse(&alloc, first_source);
        let second = parse(&alloc, second_source);
        assert!(
            first.content_eq(&first),
            "Content for pattern /{first_source}/ should be equal to itself."
        );
        assert!(
            !first.content_eq(&second),
            "Content for patterns /{first_source}/ and /{second_source}/ should not be equal."
        );
    }
}
