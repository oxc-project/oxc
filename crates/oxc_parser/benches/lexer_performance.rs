//! Benchmarks for lexer performance improvements.
//!
//! This module contains benchmarks to measure the performance impact of SIMD and branchless optimizations.

use criterion2::{black_box, criterion_group, criterion_main, Criterion};
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;

// Sample JavaScript code for benchmarking
const JAVASCRIPT_SAMPLES: &[(&str, &str)] = &[
    (
        "identifiers",
        "const myVariable = 42; let anotherIdentifier = true; var someObject = { property: 'value' };",
    ),
    (
        "numbers",
        "const a = 123; const b = 0x1A2B; const c = 0b1010; const d = 0o755; const e = 3.14159; const f = 1e10;",
    ),
    (
        "strings",
        r#"const str1 = "Hello, world!"; const str2 = 'Single quotes'; const str3 = `Template string ${variable}`;"#,
    ),
    (
        "whitespace_heavy",
        "   const   a   =   42   ;   \n   let   b   =   'hello'   ;   \n   var   c   =   {   prop   :   123   }   ;   ",
    ),
    (
        "mixed_content",
        r#"
        function fibonacci(n) {
            if (n <= 1) return n;
            return fibonacci(n - 1) + fibonacci(n - 2);
        }
        
        const numbers = [0, 1, 2, 3, 5, 8, 13, 21, 34, 55];
        const strings = ["hello", "world", "from", "JavaScript"];
        
        for (let i = 0; i < numbers.length; i++) {
            console.log(`Fibonacci of ${numbers[i]} is ${fibonacci(numbers[i])}`);
        }
        "#,
    ),
    (
        "large_object",
        r#"
        const largeObject = {
            property1: "value1",
            property2: 42,
            property3: true,
            property4: null,
            property5: undefined,
            property6: [1, 2, 3, 4, 5],
            property7: {
                nested1: "nested_value1",
                nested2: 123,
                nested3: false,
                nested4: {
                    deep1: "deep_value1",
                    deep2: 456,
                    deep3: true
                }
            },
            property8: function() { return "function_result"; },
            property9: () => "arrow_function_result",
            property10: 3.14159265359
        };
        "#,
    ),
];

fn bench_lexer_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("lexer_performance");
    
    for (name, code) in JAVASCRIPT_SAMPLES {
        group.bench_function(*name, |b| {
            b.iter(|| {
                let allocator = Allocator::default();
                let source_type = SourceType::mjs();
                let parser = Parser::new(&allocator, black_box(code), source_type);
                
                // Just tokenize, don't parse
                let mut lexer = parser.lexer;
                let mut token_count = 0;
                
                loop {
                    let token = lexer.next_token();
                    token_count += 1;
                    if token.kind.is_eof() {
                        break;
                    }
                }
                
                black_box(token_count)
            });
        });
    }
    
    group.finish();
}

fn bench_specific_optimizations(c: &mut Criterion) {
    let mut group = c.benchmark_group("specific_optimizations");
    
    // Benchmark identifier scanning
    group.bench_function("identifier_scanning", |b| {
        let code = "a".repeat(1000);
        b.iter(|| {
            let allocator = Allocator::default();
            let source_type = SourceType::mjs();
            let parser = Parser::new(&allocator, black_box(&code), source_type);
            let mut lexer = parser.lexer;
            black_box(lexer.next_token())
        });
    });
    
    // Benchmark numeric literal parsing
    group.bench_function("numeric_parsing", |b| {
        let code = "123 456 789 0x1A2B 0b1010101 0o755 3.14159 1e10 ";
        b.iter(|| {
            let allocator = Allocator::default();
            let source_type = SourceType::mjs();
            let parser = Parser::new(&allocator, black_box(code), source_type);
            let mut lexer = parser.lexer;
            
            let mut tokens = Vec::new();
            loop {
                let token = lexer.next_token();
                if token.kind.is_eof() {
                    break;
                }
                tokens.push(token);
            }
            black_box(tokens)
        });
    });
    
    // Benchmark string literal parsing
    group.bench_function("string_parsing", |b| {
        let code = r#""This is a long string with some content that should trigger SIMD optimizations when scanning for quotes and escape sequences""#;
        b.iter(|| {
            let allocator = Allocator::default();
            let source_type = SourceType::mjs();
            let parser = Parser::new(&allocator, black_box(code), source_type);
            let mut lexer = parser.lexer;
            black_box(lexer.next_token())
        });
    });
    
    // Benchmark whitespace skipping
    group.bench_function("whitespace_skipping", |b| {
        let code = format!("{}const a = 42;", " ".repeat(1000));
        b.iter(|| {
            let allocator = Allocator::default();
            let source_type = SourceType::mjs();
            let parser = Parser::new(&allocator, black_box(&code), source_type);
            let mut lexer = parser.lexer;
            
            let mut tokens = Vec::new();
            loop {
                let token = lexer.next_token();
                if token.kind.is_eof() {
                    break;
                }
                tokens.push(token);
            }
            black_box(tokens)
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_lexer_performance,
    bench_specific_optimizations
);
criterion_main!(benches);