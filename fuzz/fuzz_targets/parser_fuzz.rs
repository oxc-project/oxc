#![no_main]

use libfuzzer_sys::fuzz_target;
use oxc_allocator::Allocator;
use oxc_parser::{ParseOptions, Parser};
use oxc_span::SourceType;

fuzz_target!(|data: &[u8]| {
    // Skip empty inputs or inputs that are too large
    if data.is_empty() || data.len() > 10_000 {
        return;
    }

    // Try to convert bytes to a valid UTF-8 string
    if let Ok(source) = std::str::from_utf8(data) {
        let allocator = Allocator::default();
        let source_type = SourceType::default();
        let parser_options = ParseOptions::default();
        
        // Parse the input - we don't care if it fails, we just want to ensure
        // the parser doesn't crash or panic on any input
        let _result = Parser::new(&allocator, source, source_type)
            .with_options(parser_options)
            .parse();
            
        // The parser should handle any input gracefully without panicking
    }
});