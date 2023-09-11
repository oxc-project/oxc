#![no_main]

use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;

libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        if s.chars().all(|s| !s.is_control()) {
            let allocator = Allocator::default();
            let source_type = SourceType::default().with_typescript(true).with_jsx(true);
            let _ = Parser::new(&allocator, &s, source_type).parse();
        }
    }
});
