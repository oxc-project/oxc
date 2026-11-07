use std::alloc::{GlobalAlloc, Layout, System};

pub use criterion::*;

/// Newline/indentation workloads shared by lexer and parser benchmarks.
pub fn newline_cases() -> impl Iterator<Item = (&'static str, String)> {
    let mut cases = [
        ("lf_indent_0", 0),
        ("lf_indent_1", 1),
        ("lf_indent_2", 2),
        ("lf_indent_4", 4),
        ("lf_indent_6", 6),
        ("lf_indent_7", 7),
        ("lf_indent_8", 8),
        ("lf_indent_9", 9),
        ("lf_indent_10", 10),
        ("lf_indent_12", 12),
        ("lf_indent_14", 14),
        ("lf_indent_15", 15),
        ("lf_indent_16", 16),
        ("lf_indent_17", 17),
        ("lf_indent_31", 31),
        ("lf_indent_32", 32),
        ("lf_indent_33", 33),
        ("lf_indent_64", 64),
    ]
    .map(|(name, len)| (name, format!("x;\n{}", " ".repeat(len)).repeat(128)))
    .to_vec();
    cases.extend([
        ("lf_tabs_1", "x;\n\t".repeat(128)),
        ("lf_tabs_4", "x;\n\t\t\t\t".repeat(128)),
        ("lf_tabs_16", format!("x;\n{}", "\t".repeat(16)).repeat(128)),
        ("lf_mixed_4", "x;\n \t \t".repeat(128)),
        ("lf_mixed_32", format!("x;\n{}", " \t".repeat(16)).repeat(128)),
        ("crlf_indent_4", "x;\r\n    ".repeat(128)),
        ("cr_indent_4", "x;\r    ".repeat(128)),
        ("lf_blank_lines", "x;\n\n\n    ".repeat(128)),
        ("crlf_blank_lines", "x;\r\n\r\n    ".repeat(128)),
        ("mixed_blank_lines", "x;\n \t\r\n\t\r    ".repeat(128)),
        ("lf_nbsp", "x;\n \u{a0} ".repeat(128)),
        ("lf_line_separator", "x;\n\u{2028}    ".repeat(128)),
        ("lf_paragraph_separator", "x;\n\u{2029}    ".repeat(128)),
        ("lf_vt_ff", "x;\n\u{b}\u{c}    ".repeat(128)),
        ("block_trailing", "x; /* ordinary */\n    y;\n".repeat(128)),
        ("block_leading", "/* leading */\n    x;\n".repeat(128)),
        ("block_between_newlines", "x;\n/* block */\n    y;\n".repeat(128)),
        ("block_multiple", "x; /* first */ /* second */\n    y;\n".repeat(128)),
        ("block_multiline", "x; /* first\nsecond */\n    y;\n".repeat(128)),
        ("block_legal", "x; /* @license retained */\n    y;\n".repeat(128)),
        ("block_pure", "x; /* #__PURE__ */\n    f();\n".repeat(128)),
        (
            "block_no_side_effects",
            "x; /* #__NO_SIDE_EFFECTS__ */\n    function f() {}\n".repeat(128),
        ),
        ("line_comment_crlf", "x; // line\r\n    y;\n".repeat(128)),
        ("line_comment_crlf_leading", "x = // line\r\n    1;\n".repeat(128)),
        ("asi_return", "function f(){ return\n    x; }\n".repeat(128)),
        ("asi_update", "x\n++y;\n".repeat(128)),
        ("asi_async", "async\nfunction f() {}\n".repeat(128)),
        ("no_newline_control", "x; y;".repeat(128)),
        ("eof_after_lf", "x;\n".into()),
        ("eof_after_crlf", "x;\r\n".into()),
        ("eof_indent_15", format!("x;\n{}", " ".repeat(15))),
        ("eof_indent_16", format!("x;\n{}", " ".repeat(16))),
        ("eof_indent_17", format!("x;\n{}", " ".repeat(17))),
    ]);
    // Keep LF and one-space controls at equal lengths with the same token sequences.
    // These sources avoid syntax whose meaning depends on line breaks.
    for (newline_name, spaces_name, source, repetitions) in [
        ("guard_call_lf", "guard_call_spaces", "f(\n(x)\n,\ny\n)\n;\n", 128),
        ("guard_array_lf", "guard_array_spaces", "x =\n[\nx\n,\ny\n]\n;\n", 128),
        ("guard_block_lf", "guard_block_spaces", "function f()\n{\nx;\n}\n", 128),
        ("guard_object_colon_lf", "guard_object_colon_spaces", "x = {\nkey\n:\ny\n};\n", 128),
        ("guard_tilde_lf", "guard_tilde_spaces", "x =\n~x;\n", 128),
        ("fallback_quote_digit_lf", "fallback_quote_digit_spaces", "\"text\";\n123;\n", 128),
        ("guard_final_identifier_lf", "guard_final_identifier_spaces", "x;\nq", 1),
        ("guard_final_semicolon_lf", "guard_final_semicolon_spaces", "x;\n;", 1),
    ] {
        let newline_source = source.repeat(repetitions);
        let spaces_source =
            newline_source.chars().map(|c| if c == '\n' { ' ' } else { c }).collect();
        cases.push((newline_name, newline_source));
        cases.push((spaces_name, spaces_source));
    }
    cases.into_iter()
}

#[global_allocator]
static GLOBAL: NeverGrowInPlaceAllocator = NeverGrowInPlaceAllocator;

/// Global allocator for use in benchmarks.
///
/// A thin wrapper around Rust's default [`System`] allocator. It passes through `alloc`
/// and `dealloc` methods to [`System`], but does not implement [`GlobalAlloc::realloc`].
///
/// Rationale for this is:
///
/// `realloc` for default [`System`] allocator calls `libc::realloc`, which may either:
/// 1. allow the allocation to grow in place. or
/// 2. create a new allocation, and copy memory from old allocation to the new one.
///
/// Whether allocations can grow in place or not depends on the state of the operating system's
/// memory tables, and so is inherently non-deterministic. Using default `System` allocator
/// therefore produces large and unpredictable variance in benchmarks.
///
/// By not providing a `realloc` method, this custom allocator delegates to the default
/// [`GlobalAlloc::realloc`] implementation which *never* grows in place.
/// It therefore represents the "worse case scenario" for memory allocation performance.
/// This behavior is consistent and predictable, and therefore stabilizes benchmark results.
struct NeverGrowInPlaceAllocator;

// SAFETY: Methods simply delegate to `System` allocator
#[expect(clippy::undocumented_unsafe_blocks)]
unsafe impl GlobalAlloc for NeverGrowInPlaceAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) };
    }
}
