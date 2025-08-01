#![expect(clippy::print_stdout)]
//! # Regular Expression Literal Parser Example
//!
//! This example demonstrates how to parse JavaScript regular expression literals using
//! Oxc's regular expression parser. It shows parsing of various regex patterns and flags,
//! including both valid and invalid expressions for educational purposes.
//!
//! ## Features
//!
//! - Parse regex patterns with various flags (i, g, m, u, v, etc.)
//! - Handle Unicode patterns and escapes
//! - Parse character classes and quantifiers
//! - Support for named capture groups and backreferences
//! - Comprehensive error reporting for invalid patterns
//! - Examples of both valid and invalid regex patterns
//!
//! ## Pattern Examples
//!
//! The example includes patterns demonstrating:
//! - Basic matching: `ab`, `abc`
//! - Flags: `i` (ignore case), `g` (global), `u` (unicode), `v` (unicodesets)
//! - Unicode: emoji, escape sequences
//! - Alternation: `a|b`
//! - Quantifiers: `{0}`, `{1,2}`, `{3,}`
//! - Assertions: lookahead, lookbehind
//! - Character classes: `[abc]`, `[a&&b]`, `[a--b]`
//! - Named groups: `(?<name>pattern)`
//!
//! ## Usage
//!
//! Simply run:
//! ```bash
//! cargo run -p oxc_regular_expression --example parse_literal
//! ```

use oxc_allocator::Allocator;
use oxc_regular_expression::{LiteralParser, Options};

/// Main entry point for the regex literal parsing example
fn main() {
    println!("Oxc Regular Expression Parser Example");
    println!("=====================================");
    println!("Parsing various regex patterns to demonstrate parser capabilities");
    println!();

    let allocator = Allocator::default();

    // Collection of test patterns with descriptions
    struct TestCase {
        pattern: &'static str,
        flags: &'static str,
        description: &'static str,
        should_fail: bool,
    }

    let test_cases = [
        TestCase {
            pattern: r"ab",
            flags: "",
            description: "Simple literal match",
            should_fail: false,
        },
        TestCase {
            pattern: r"abc",
            flags: "i",
            description: "Case-insensitive match",
            should_fail: false,
        },
        TestCase {
            pattern: r"abcd",
            flags: "igv",
            description: "Multiple flags (ignore case, global, unicodesets)",
            should_fail: false,
        },
        TestCase {
            pattern: r"emo👈🏻ji",
            flags: "u",
            description: "Unicode emoji pattern",
            should_fail: false,
        },
        TestCase {
            pattern: r"ab|c",
            flags: "i",
            description: "Alternation pattern",
            should_fail: false,
        },
        TestCase {
            pattern: r"a|b+|c",
            flags: "i",
            description: "Alternation with quantifier",
            should_fail: false,
        },
        TestCase {
            pattern: r"a{0}|b{1,2}|c{3,}",
            flags: "i",
            description: "Various quantifiers",
            should_fail: false,
        },
        TestCase {
            pattern: r"(?=a)|(?<=b)|(?!c)|(?<!d)",
            flags: "i",
            description: "Lookahead and lookbehind assertions",
            should_fail: false,
        },
        TestCase {
            pattern: r"\n\cM\0\x41\.",
            flags: "",
            description: "Escape sequences",
            should_fail: false,
        },
        TestCase {
            pattern: r"\n\cM\0\x41\u1234\.",
            flags: "u",
            description: "Unicode escape sequences",
            should_fail: false,
        },
        TestCase {
            pattern: r"\n\cM\0\x41\u{1f600}\.",
            flags: "u",
            description: "Unicode code point escapes",
            should_fail: false,
        },
        TestCase {
            pattern: r"a\k<f>x\1c",
            flags: "u",
            description: "Named backreference",
            should_fail: false,
        },
        TestCase {
            pattern: r"(cg)(?<n>cg)(?:g)",
            flags: "",
            description: "Capture groups and named groups",
            should_fail: false,
        },
        TestCase {
            pattern: r"{3}",
            flags: "",
            description: "Invalid quantifier (should fail)",
            should_fail: true,
        },
        TestCase {
            pattern: r"Em🥹j",
            flags: "",
            description: "Emoji in pattern",
            should_fail: false,
        },
        TestCase {
            pattern: r"^(?=ab)\b(?!cd)(?<=ef)\B(?<!gh)$",
            flags: "",
            description: "Complex assertions",
            should_fail: false,
        },
        TestCase {
            pattern: r"^(?<!ab)$",
            flags: "",
            description: "Lookbehind at start",
            should_fail: false,
        },
        TestCase {
            pattern: r"a)",
            flags: "",
            description: "Unmatched closing parenthesis (should fail)",
            should_fail: true,
        },
        TestCase {
            pattern: r"c]",
            flags: "",
            description: "Unmatched closing bracket",
            should_fail: false,
        },
        TestCase {
            pattern: r"[abc]",
            flags: "",
            description: "Character class",
            should_fail: false,
        },
        TestCase {
            pattern: r"[|\]]",
            flags: "",
            description: "Special characters in class",
            should_fail: false,
        },
        TestCase {
            pattern: r"[a&&b]",
            flags: "v",
            description: "Character class intersection (unicodesets)",
            should_fail: false,
        },
        TestCase {
            pattern: r"[a--b]",
            flags: "v",
            description: "Character class subtraction (unicodesets)",
            should_fail: false,
        },
        TestCase {
            pattern: r"[a&&&]",
            flags: "v",
            description: "Invalid intersection (should fail)",
            should_fail: true,
        },
        TestCase {
            pattern: r"[a---]",
            flags: "v",
            description: "Invalid subtraction (should fail)",
            should_fail: true,
        },
        TestCase {
            pattern: r"[^a--b--c]",
            flags: "v",
            description: "Negated class with subtraction",
            should_fail: false,
        },
        TestCase {
            pattern: r"[a[b[c[d[e[f[g[h[i[j[k[l]]]]]]]]]]]]",
            flags: "v",
            description: "Nested character classes",
            should_fail: false,
        },
        TestCase {
            pattern: r"[\q{abc|d|e|}]",
            flags: "v",
            description: "String literals in class (unicodesets)",
            should_fail: false,
        },
        TestCase {
            pattern: r"\p{Basic_Emoji}",
            flags: "v",
            description: "Unicode property (unicodesets)",
            should_fail: false,
        },
        TestCase {
            pattern: r"\p{Basic_Emoji}",
            flags: "u",
            description: "Unicode property with u flag (should fail)",
            should_fail: true,
        },
        TestCase {
            pattern: r"[[^\q{}]]",
            flags: "v",
            description: "Invalid string literal (should fail)",
            should_fail: true,
        },
        TestCase {
            pattern: r"(?<a>)(?<a>)",
            flags: "",
            description: "Duplicate named groups (should fail)",
            should_fail: true,
        },
        TestCase {
            pattern: r"(?noname)",
            flags: "v",
            description: "Invalid group syntax (should fail)",
            should_fail: true,
        },
        TestCase {
            pattern: r"[\bb]",
            flags: "",
            description: "Backspace in character class",
            should_fail: false,
        },
        TestCase {
            pattern: r"a{2,1}",
            flags: "v",
            description: "Invalid quantifier range (should fail)",
            should_fail: true,
        },
    ];

    let mut success_count = 0;
    let mut failure_count = 0;
    let mut unexpected_results = 0;

    for (i, test_case) in test_cases.iter().enumerate() {
        let literal = format!("/{}/{}", test_case.pattern, test_case.flags);

        println!("Test #{}: {}", i + 1, test_case.description);
        println!("Pattern: {literal}");

        // Create parser with offset for error reporting (accounting for leading `/`)
        let parser = LiteralParser::new(
            &allocator,
            test_case.pattern,
            Some(test_case.flags),
            Options { pattern_span_offset: 1, ..Options::default() },
        );

        let result = parser.parse();

        match result {
            Ok(pattern) => {
                if test_case.should_fail {
                    println!("⚠️  Unexpected success (expected failure)");
                    unexpected_results += 1;
                } else {
                    println!("✅ Success");
                    success_count += 1;
                }

                // Show a condensed view of the AST for successful parses
                println!("AST: {pattern:#?}");
            }
            Err(error) => {
                if test_case.should_fail {
                    println!("✅ Expected failure");
                    failure_count += 1;
                } else {
                    println!("❌ Unexpected failure");
                    unexpected_results += 1;
                }

                let error = error.clone().with_source_code(literal);
                println!("Error: {error:?}");
            }
        }

        println!("{}", "─".repeat(70));
        println!();
    }

    // Summary statistics
    println!("📊 Parsing Summary:");
    println!("═══════════════════");
    println!("Total patterns tested: {}", test_cases.len());
    println!("✅ Successful parses: {success_count}");
    println!("❌ Expected failures: {failure_count}");
    if unexpected_results > 0 {
        println!("⚠️  Unexpected results: {unexpected_results}");
    }

    println!();
    println!("💡 This example demonstrates the parser's capability to handle");
    println!("   various regex patterns and provide detailed error reporting");
    println!("   for invalid patterns. The parser supports modern JavaScript");
    println!("   regex features including Unicode and unicodesets modes.");
}
