//! Printing for [`crate::OpaqueRegion`]s: source ranges that are not JavaScript.
//!
//! The parsed AST holds a placeholder node at each region's span (see [`crate::OpaqueRegion`]
//! for how the caller arranges that). Every format site that can host one asks [`write_opaque`]
//! first, so the region prints as its own language instead of as the placeholder.

use oxc_formatter_core::{
    LINE_TERMINATORS, arena_cow_str, dispatch_fragment_ir, normalize_newlines,
};
use oxc_span::Span;

use crate::{formatter::prelude::*, write};

/// Print the opaque region occupying exactly `span`, returning whether there was one.
///
/// Falls back to the verbatim source range whenever the region cannot be formatted, which
/// covers an absent dispatcher (the pure Rust build has none), a language nothing serves,
/// and a child that declined. Emitting the placeholder instead would destroy the source, so
/// there is deliberately no path that prints the parsed text here.
pub fn write_opaque(span: Span, f: &mut JsFormatter<'_, '_>) -> bool {
    let Some(region) = f.context().opaque_region_at(span) else {
        return false;
    };

    let source = f.source_text().text_for(&span);
    if let Some(ir) = dispatch_fragment_ir(f, region.language, source, None) {
        f.write_elements(ir);
    } else {
        // The IR only supports `\n` as a line break; the printer re-emits the configured
        // line ending at the end.
        let normalized = normalize_newlines(source, LINE_TERMINATORS);
        write!(f, [text(arena_cow_str(&normalized, f))]);
    }
    true
}

#[cfg(test)]
mod tests {
    use oxc_allocator::Allocator;
    use oxc_span::SourceType;

    use oxc_formatter_core::{FormatSession, InputKind};

    use crate::{
        JsFormatOptions, OpaqueRegion, format_program_with_opaque_regions, parse_for_format,
    };

    /// Format `original` after parsing `substituted`, with `regions` marking the ranges that
    /// differ between the two. Mirrors what a host with non-JavaScript regions must do.
    fn format(original: &str, substituted: &str, regions: &[OpaqueRegion<'_>]) -> String {
        assert_eq!(
            original.len(),
            substituted.len(),
            "a placeholder must match its region's byte length, or every later span shifts"
        );
        let allocator = Allocator::default();
        let source_type = SourceType::ts();
        let parsed = parse_for_format(&allocator, substituted, source_type);
        assert!(
            parsed.diagnostics.is_empty(),
            "substituted text must parse: {:?}",
            parsed.diagnostics
        );
        let program = allocator.alloc(parsed.program);
        // A service-less session: no dispatcher, so every region takes the verbatim path.
        let session = FormatSession::new(&allocator, InputKind::PhysicalFile);
        format_program_with_opaque_regions(
            &session,
            program,
            original,
            regions,
            JsFormatOptions::default(),
        )
        .print()
        .unwrap()
        .into_code()
    }

    /// No dispatcher is installed here, so every region takes the verbatim fallback. That is
    /// the same path the pure Rust build always takes.
    #[test]
    fn region_prints_its_source_not_the_placeholder() {
        // `<<<HERE>>>` and `__________` are both 10 bytes.
        let region = |start: u32| OpaqueRegion {
            span: oxc_span::Span::new(start, start + 10),
            language: "unserved",
        };

        // Expression position.
        assert_eq!(
            format("const a = <<<HERE>>>;\n", "const a = __________;\n", &[region(10)]),
            "const a = <<<HERE>>>;\n"
        );
        // Statement position. The region is a whole statement in another language, so it
        // keeps its own terminator rather than gaining a JavaScript semicolon.
        assert_eq!(
            format("<<<HERE>>>;\nconst b = 1;\n", "__________;\nconst b = 1;\n", &[region(0)]),
            "<<<HERE>>>\nconst b = 1;\n"
        );
        // Class-member position, which `PropertyDefinition` rather than the identifier takes.
        assert_eq!(
            format("class A {\n  <<<HERE>>>\n}\n", "class A {\n  __________\n}\n", &[region(12)]),
            "class A {\n  <<<HERE>>>\n}\n"
        );
    }

    #[test]
    fn surrounding_code_is_still_formatted() {
        assert_eq!(
            format(
                "const a = [1,2];\nconst b = <<<HERE>>>;\n",
                "const a = [1,2];\nconst b = __________;\n",
                &[OpaqueRegion { span: oxc_span::Span::new(27, 37), language: "unserved" }]
            ),
            "const a = [1, 2];\nconst b = <<<HERE>>>;\n"
        );
    }

    #[test]
    fn multiple_regions_are_each_resolved() {
        let source = "const a = <<<AAA>>>;\nconst b = <<<BBB>>>;\n";
        let substituted = "const a = _________;\nconst b = _________;\n";
        let regions = [
            OpaqueRegion { span: oxc_span::Span::new(10, 19), language: "unserved" },
            OpaqueRegion { span: oxc_span::Span::new(31, 40), language: "unserved" },
        ];
        assert_eq!(format(source, substituted, &regions), source);
    }

    /// The contract in the other direction: a region whose span reaches a position nothing
    /// intercepts prints the placeholder, destroying that range. This is why the caller must
    /// verify each region against the AST and decline to format when one does not line up.
    #[test]
    fn a_misplaced_region_is_not_rescued_here() {
        // A binding is not one of the intercepted positions, so the placeholder survives.
        let regions = [OpaqueRegion { span: oxc_span::Span::new(6, 12), language: "unserved" }];
        assert_eq!(
            format("const aaaaaa = 1;\n", "const ______ = 1;\n", &regions),
            "const ______ = 1;\n"
        );
    }
}
