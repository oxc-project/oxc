#[cfg(test)]
mod performance_tests {
    use crate::lexer::{Kind, Lexer};
    use oxc_allocator::Allocator;
    use oxc_span::SourceType;
    use std::time::Instant;

    const TEST_JS_CODE: &str = "function test() { let a = 1; let b = 2; return a + b; }";

    #[test] 
    fn test_lexer_performance_baseline() {
        let allocator = Allocator::default();
        let source_type = SourceType::default();
        
        let start = Instant::now();
        const ITERATIONS: usize = 1000;
        
        for _ in 0..ITERATIONS {
            let mut lexer = Lexer::new_for_benchmarks(&allocator, TEST_JS_CODE, source_type);
            let mut token_count = 0;
            while lexer.next_token().kind() != Kind::Eof {
                token_count += 1;
            }
            std::hint::black_box(token_count);
        }
        
        let elapsed = start.elapsed();
        println!("Lexer baseline: {} iterations in {:?} ({:.2}µs per iteration)", 
                 ITERATIONS, elapsed, elapsed.as_micros() as f64 / ITERATIONS as f64);
        
        // Test passes if it runs without panic
        assert!(elapsed.as_millis() < 1000); // Should complete in less than 1 second
    }
}