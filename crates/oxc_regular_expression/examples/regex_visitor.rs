#![expect(clippy::print_stdout)]
//! # Regular Expression AST Visitor Example
//!
//! This example demonstrates how to traverse and analyze regular expression ASTs
//! using Oxc's visitor pattern. It shows how to implement a custom visitor to
//! inspect various nodes in a parsed regular expression.
//!
//! ## Features
//!
//! - Parse complex regular expression patterns
//! - Implement custom visitor for AST traversal
//! - Track entering and leaving each AST node
//! - Display node types and their source spans
//! - Analyze the structure of regex patterns
//!
//! ## The Visitor Pattern
//!
//! The visitor pattern allows you to traverse the AST and perform operations
//! on different node types. This is useful for:
//! - Code analysis and linting
//! - AST transformations
//! - Pattern matching and extraction
//! - Educational exploration of regex structure
//!
//! ## Example Pattern
//!
//! The example uses a complex GitHub URL matching pattern that demonstrates:
//! - Alternation (`|`)
//! - Grouping with `()`
//! - Character classes `[]`
//! - Quantifiers `+`, `*`, `{n,m}`
//! - Anchors `^`, `$`
//! - Escape sequences `\/`, `\.`
//!
//! ## Usage
//!
//! Simply run:
//! ```bash
//! cargo run -p oxc_regular_expression --example regex_visitor
//! ```

use oxc_allocator::Allocator;
use oxc_regular_expression::{
    LiteralParser, Options,
    visit::{RegExpAstKind, Visit},
};
use oxc_span::GetSpan;

/// Custom visitor that tracks AST traversal and analyzes the regex structure
struct RegexAnalysisVisitor {
    depth: usize,
    node_count: usize,
    group_count: usize,
    character_class_count: usize,
    quantifier_count: usize,
}

impl RegexAnalysisVisitor {
    fn new() -> Self {
        Self {
            depth: 0,
            node_count: 0,
            group_count: 0,
            character_class_count: 0,
            quantifier_count: 0,
        }
    }

    fn indent(&self) -> String {
        "  ".repeat(self.depth)
    }

    fn print_statistics(&self) {
        println!();
        println!("📊 AST Analysis Statistics:");
        println!("═══════════════════════════");
        println!("Total nodes visited: {}", self.node_count);
        println!("Groups found: {}", self.group_count);
        println!("Character classes found: {}", self.character_class_count);
        println!("Quantifiers found: {}", self.quantifier_count);
    }
}

impl Visit<'_> for RegexAnalysisVisitor {
    fn enter_node(&mut self, kind: RegExpAstKind) {
        self.node_count += 1;

        // Analyze specific node types
        match &kind {
            RegExpAstKind::CapturingGroup(_) | RegExpAstKind::IgnoreGroup(_) => {
                self.group_count += 1;
                println!("{}🔍 Entering group at {:?}", self.indent(), kind.span());
            }
            RegExpAstKind::CharacterClass(_) => {
                self.character_class_count += 1;
                println!("{}📋 Entering character class at {:?}", self.indent(), kind.span());
            }
            RegExpAstKind::Quantifier(_) => {
                self.quantifier_count += 1;
                println!("{}🔢 Entering quantifier at {:?}", self.indent(), kind.span());
            }
            RegExpAstKind::Alternative(_) => {
                println!("{}🔀 Entering alternative at {:?}", self.indent(), kind.span());
            }
            RegExpAstKind::Character(_) => {
                println!("{}📝 Character at {:?}", self.indent(), kind.span());
            }
            RegExpAstKind::Dot(_) => {
                println!("{}🎯 Dot (any character) at {:?}", self.indent(), kind.span());
            }
            _ => {
                println!("{}➤ Entering {:?} at {:?}", self.indent(), kind, kind.span());
            }
        }

        self.depth += 1;
    }

    fn leave_node(&mut self, kind: RegExpAstKind) {
        self.depth -= 1;

        // Only show leave messages for complex nodes to reduce noise
        match &kind {
            RegExpAstKind::CapturingGroup(_)
            | RegExpAstKind::IgnoreGroup(_)
            | RegExpAstKind::CharacterClass(_)
            | RegExpAstKind::Alternative(_) => {
                println!("{}⬅ Leaving {:?} at {:?}", self.indent(), kind, kind.span());
            }
            _ => {} // Skip leave messages for simple nodes
        }
    }
}

/// Main entry point for the regex visitor example
fn main() {
    println!("Oxc Regular Expression AST Visitor Example");
    println!("==========================================");

    // Complex GitHub URL matching pattern for demonstration
    let source_pattern = r"(https?:\/\/github\.com\/(([^\s]+)\/([^\s]+))\/([^\s]+\/)?(issues|pull)\/([0-9]+))|(([^\s]+)\/([^\s]+))?#([1-9][0-9]*)($|[\s\:\;\-\(\=])";

    println!("Pattern to analyze:");
    println!("/{source_pattern}/");
    println!();
    println!("This pattern matches GitHub URLs and issue references with parts for:");
    println!("• Full GitHub URLs (https://github.com/owner/repo/issues/123)");
    println!("• Short references (owner/repo#123)");
    println!("• Various separators and boundaries");
    println!();
    println!("{}", "─".repeat(80));
    println!();

    // Parse the regular expression
    let allocator = Allocator::default();
    let parser = LiteralParser::new(&allocator, source_pattern, None, Options::default());

    match parser.parse() {
        Ok(pattern) => {
            println!("✅ Successfully parsed regex pattern");
            println!();
            println!("🔍 AST Traversal:");
            println!("{}", "─".repeat(50));

            // Create our custom visitor and traverse the AST
            let mut visitor = RegexAnalysisVisitor::new();
            visitor.visit_pattern(&pattern);

            println!("{}", "─".repeat(50));
            visitor.print_statistics();

            println!();
            println!("💡 The visitor pattern demonstrated:");
            println!("   • How to traverse regex AST nodes");
            println!("   • Tracking entry and exit from nodes");
            println!("   • Analyzing pattern structure");
            println!("   • Counting different node types");
            println!();
            println!("🚀 You can extend this visitor to:");
            println!("   • Validate regex patterns");
            println!("   • Extract specific information");
            println!("   • Transform or optimize patterns");
            println!("   • Generate documentation or reports");
        }
        Err(error) => {
            println!("❌ Failed to parse regex pattern:");
            let error = error.with_source_code(format!("/{source_pattern}/"));
            println!("{error:?}");
            println!();
            println!("💡 Try with a simpler pattern or fix the syntax error");
        }
    }
}
