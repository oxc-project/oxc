use std::time::Instant;
use oxc_allocator::Allocator;
use oxc_parser::lexer::{Kind, Lexer};
use oxc_span::SourceType;

const TEST_JS_CODE: &str = "function test() { let a = 1; let b = 2; return a + b; }";

fn benchmark_lexer(source_text: &str, name: &str) -> std::time::Duration {
    let allocator = Allocator::default();
    let source_type = SourceType::default();
    
    let start = Instant::now();
    
    // Run lexer multiple times to get meaningful measurements
    const ITERATIONS: usize = 10000;
    
    for _ in 0..ITERATIONS {
        let mut lexer = Lexer::new_for_benchmarks(&allocator, source_text, source_type);
        let mut token_count = 0;
        while lexer.next_token().kind() != Kind::Eof {
            token_count += 1;
        }
        // Prevent optimization from removing the loop
        std::hint::black_box(token_count);
    }
    
    let elapsed = start.elapsed();
    println!("{}: {} iterations in {:?} ({:.2}µs per iteration)", 
             name, ITERATIONS, elapsed, elapsed.as_micros() as f64 / ITERATIONS as f64);
    elapsed
}

fn main() {
    println!("Lexer Performance Test");
    println!("======================");
    
    // Test with the simple sample
    benchmark_lexer(TEST_JS_CODE, "Simple JavaScript");
    
    // Test with pure ASCII identifiers
    let ascii_ids = "function test() { let a = b + c; return d; }";
    benchmark_lexer(ascii_ids, "ASCII identifiers only");
    
    // Test with string literals
    let strings = "\"hello\" \"world\" \"test string with spaces\" \"another string\"";
    benchmark_lexer(strings, "String literals");
    
    // Test with numbers
    let numbers = "1 2 3 4.5 6.7 8.9 0.123 456.789 1000000";
    benchmark_lexer(numbers, "Numbers");
    
    // Test with comments
    let comments = "// comment 1\n/* block comment */ // another";
    benchmark_lexer(comments, "Comments");
    
    // Test with mixed punctuation
    let punctuation = "{}[]().,;:?!+-*/=%<>&|^~";
    benchmark_lexer(punctuation, "Punctuation");
    
    println!("\nTesting complete!");
}