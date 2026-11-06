//! Tests that deeply nested (but legal) patterns produce a diagnostic instead of
//! overflowing the stack and aborting the process.

use oxc_allocator::Allocator;
use oxc_regular_expression::{LiteralParser, Options};

#[test]
fn nesting_over_limit_errors_instead_of_overflowing_stack() {
    // 300 nested capturing groups: over the limit in both debug and release builds.
    let allocator = Allocator::default();
    let pattern = format!("{}a{}", "(".repeat(300), ")".repeat(300));
    let result = LiteralParser::new(&allocator, &pattern, Some(""), Options::default()).parse();
    let error = result.expect_err("deeply nested pattern should fail to parse");
    assert!(error.to_string().contains("Nesting depth exceeds"));
}

#[test]
fn nesting_under_limit_parses() {
    // 40 nested capturing groups: under the limit in both debug and release builds.
    let allocator = Allocator::default();
    let pattern = format!("{}a{}", "(".repeat(40), ")".repeat(40));
    let result = LiteralParser::new(&allocator, &pattern, Some(""), Options::default()).parse();
    assert!(result.is_ok(), "40 nested groups should parse: {result:?}");

    let allocator = Allocator::default();
    let pattern = format!("{}a{}", "[".repeat(40), "]".repeat(40));
    // Nested classes are only allowed with `v` (unicodeSets) mode.
    let result = LiteralParser::new(&allocator, &pattern, Some("v"), Options::default()).parse();
    assert!(result.is_ok(), "40 nested classes should parse: {result:?}");
}
