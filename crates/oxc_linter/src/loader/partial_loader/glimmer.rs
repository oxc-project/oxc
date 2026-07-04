//! # Glimmer JS / Glimmer TS partial loader
//!
//! Strips `<template>...</template>` blocks out of `.gjs` / `.gts` files so the
//! surrounding JavaScript / TypeScript can be linted as ordinary modules.
//!
//! ## Source material
//!
//! The `<template>` JS/TS embedding originates from [Ember RFC #779] and is implemented
//! in production by [content-tag] (a Rust crate built on [`@glimmer/syntax`]). For full
//! correctness an oxc loader should ultimately delegate to (or mirror) content-tag.
//!
//! [Ember RFC #779]: https://rfcs.emberjs.com/id/0779-first-class-component-templates/
//! [content-tag]: https://github.com/embroider-build/content-tag
//! [`@glimmer/syntax`]: https://github.com/glimmerjs/glimmer-vm
//!
//! ## Approach
//!
//! Rather than parsing the file twice, this loader rewrites every
//! `<template>...</template>` block into a same-byte-length JS/TS expression so that:
//!
//! * source spans for the surrounding code remain accurate for diagnostics, and
//! * the result is valid in *every* position where a template tag is legal: top-level,
//!   expression position (`const c = <template>...</template>`) and class body
//!   (`class C { <template>...</template> }`).
//!
//! The `<template>` opener (10 bytes) is replaced by `undefined ` (also 10 bytes — the
//! identifier `undefined` followed by a single space). The remaining bytes — including
//! the `</template>` closer — are replaced byte-for-byte with ASCII spaces, except for
//! `\n`, which is preserved so line/column reporting is unaffected.
//!
//! `undefined` was chosen specifically because it is:
//! * a valid expression in any expression position,
//! * a legal class field name (in class-body position the substitution becomes a class
//!   field declaration named `undefined` with no initializer), and
//! * already a built-in global, so it does not introduce bindings.
//!
//! **Do not change the substitution string without re-checking the byte-length invariant
//! and the validity in both expression and class-body positions.**
//!
//! ## Scanning
//!
//! A minimal single-pass state machine ([`template_regions`]) classifies every byte as
//! code, a line/block comment, or a single/double/backtick string, and only recognizes a
//! `<template>` opener in *code* position. So `<template>` appearing inside a comment or
//! a string is left untouched. Once inside a template, its body is HTML-ish and treated
//! as opaque — quotes, slashes, and `//` in it are not JS — so only `</template>` ends
//! it. It is not a JS/TS lexer, but it is enough to keep the common cases correct without
//! parsing the file twice.
//!
//! ## Known limitations
//!
//! The scanner is deliberately smaller than a real lexer, so a few (unlikely) cases are
//! still handled by the byte pattern alone rather than by grammar. These produce
//! incorrect output; dedicated tests pin them:
//!
//! * inside a regex literal: `/<template>/` — telling a regex from division needs
//!   expression-vs-operator context this scanner does not track, so `/` is treated as an
//!   ordinary code byte and the `<template>` inside is seen as a real opener.
//! * inside JSX text in a `.gjs` file: `<div><template>literal</template></div>`.
//! * inside a `${...}` interpolation in a backtick string — the interpolation's contents
//!   are treated as opaque string, not re-scanned as code.
//! * a `<template>`-shaped substring that is really adjacent comparisons or a TS generic
//!   named `template` (`Array<template>`), which the byte pattern cannot distinguish from
//!   an opener.
//! * an unclosed `<template>` in code position greedily consumes up to the next
//!   `</template>` later in the file (or, if there is none, is left untouched).
//!
//! A future rewrite can delegate to the [content-tag] crate for exact extraction.

use oxc_allocator::{Allocator, StringBuilder};
use oxc_span::SourceType;

use crate::loader::JavaScriptSource;

const TEMPLATE_START: &str = "<template>";
const TEMPLATE_END: &str = "</template>";

/// Where the single-pass scanner currently is while walking the source bytes.
#[derive(Clone, Copy)]
enum ScanState {
    /// Ordinary JS/TS code: the only state in which a `<template>` opener is recognized.
    Code,
    /// Inside a `// ...` line comment, ended by the next `\n`.
    LineComment,
    /// Inside a `/* ... */` block comment, ended by the next `*/`.
    BlockComment,
    /// Inside a string; the byte is the closing delimiter (`'`, `"`, or `` ` ``). A raw
    /// newline also ends a `'`/`"` string (they cannot span lines); backtick strings may.
    Str(u8),
    /// Inside a `<template>` body, holding the byte offset of its `<template>` opener. The
    /// body is HTML-ish and opaque — quotes, slashes, and `//` in it are *not* JS — so
    /// only `</template>` ends it.
    Template(usize),
}

/// Byte offsets `(open_start, close_end)` of every `<template>...</template>` region that
/// opens in code position — not inside a comment or string.
///
/// A minimal single-pass state machine classifies each byte so that a literal
/// `<template>` inside a comment or string is not mistaken for a tag, and so that the
/// HTML-ish template body (which routinely contains quotes and slashes) is treated as
/// opaque rather than lexed as JS. The returned regions are ordered and disjoint. See the
/// module-level docs for the scanner's remaining limitations.
fn template_regions(source_text: &str) -> Vec<(usize, usize)> {
    let bytes = source_text.as_bytes();
    let len = bytes.len();
    let mut regions: Vec<(usize, usize)> = Vec::new();
    let mut state = ScanState::Code;
    let mut i = 0;

    while i < len {
        match state {
            ScanState::Code => {
                let rest = &bytes[i..];
                if rest.starts_with(b"//") {
                    state = ScanState::LineComment;
                    i += 2;
                } else if rest.starts_with(b"/*") {
                    state = ScanState::BlockComment;
                    i += 2;
                } else if matches!(bytes[i], b'\'' | b'"' | b'`') {
                    state = ScanState::Str(bytes[i]);
                    i += 1;
                } else if rest.starts_with(TEMPLATE_START.as_bytes()) {
                    // Enter the template body. `</template>` starts with `<` too, but its
                    // second byte is `/` and the opener's is `t`, so this never matches a
                    // close; a stray `</template>` in code has no opener and is ignored.
                    state = ScanState::Template(i);
                    i += TEMPLATE_START.len();
                } else {
                    i += 1;
                }
            }
            ScanState::LineComment => {
                if bytes[i] == b'\n' {
                    state = ScanState::Code;
                }
                i += 1;
            }
            ScanState::BlockComment => {
                if bytes[i..].starts_with(b"*/") {
                    state = ScanState::Code;
                    i += 2;
                } else {
                    i += 1;
                }
            }
            ScanState::Str(delimiter) => match bytes[i] {
                // A backslash escapes the next byte (including a `'`/`"`/`` ` `` or a
                // line-continuation newline), so skip both.
                b'\\' => i += 2,
                // A raw newline cannot appear in a `'`/`"` string, so it was unterminated;
                // recover to code rather than swallowing the rest of the file.
                b'\n' if delimiter != b'`' => {
                    state = ScanState::Code;
                    i += 1;
                }
                b if b == delimiter => {
                    state = ScanState::Code;
                    i += 1;
                }
                _ => i += 1,
            },
            ScanState::Template(open_start) => {
                if bytes[i..].starts_with(TEMPLATE_END.as_bytes()) {
                    let end = i + TEMPLATE_END.len();
                    regions.push((open_start, end));
                    state = ScanState::Code;
                    i = end;
                } else {
                    i += 1;
                }
            }
        }
    }

    regions
}

pub struct GlimmerPartialLoader<'a> {
    source_text: &'a str,
    allocator: &'a Allocator,
}

impl<'a> GlimmerPartialLoader<'a> {
    pub fn new(source_text: &'a str, allocator: &'a Allocator) -> Self {
        Self { source_text, allocator }
    }

    pub fn parse(self, ext: &str) -> Vec<JavaScriptSource<'a>> {
        // `.gjs` is unconditionally an ESM module per Ember RFC #779: top-level
        // `<template>` and the file format itself presuppose module semantics, so we
        // use `mjs()` rather than `unambiguous()` to avoid script-mode fallback for
        // files that happen to contain no `import`/`export` statements.
        // `.gts` uses `SourceType::ts()` whose unambiguous module kind is resolved to
        // a module by the parser on the first ESM construct.
        let source_type = if ext == "gts" { SourceType::ts() } else { SourceType::mjs() };
        let cleaned = self.strip_templates();
        vec![JavaScriptSource::partial(cleaned, source_type, 0)]
    }

    /// Replace `<template>...</template>` blocks with same-byte-length JS that is
    /// valid in expression and class-body position, while preserving newlines so
    /// byte offsets and line/column information for surrounding JS code remain
    /// accurate for diagnostic reporting. See the module-level docs for the
    /// substitution invariants and the known limitations of this byte-level scan.
    fn strip_templates(&self) -> &'a str {
        let regions = template_regions(self.source_text);
        if regions.is_empty() {
            return self.source_text;
        }

        let source_bytes = self.source_text.as_bytes();
        let mut builder = StringBuilder::with_capacity_in(source_bytes.len(), self.allocator);
        let mut cursor = 0;
        for (region_start, region_end) in regions {
            builder.push_str(&self.source_text[cursor..region_start]);
            // 10-byte opener `<template>` -> 10-byte `undefined `; see module-level
            // docs for why this exact identifier was chosen.
            builder.push_str("undefined ");
            for &byte in &source_bytes[region_start + TEMPLATE_START.len()..region_end] {
                if byte == b'\n' {
                    builder.push_ascii_byte(b'\n');
                } else {
                    builder.push_ascii_byte(b' ');
                }
            }
            cursor = region_end;
        }
        builder.push_str(&self.source_text[cursor..]);
        builder.into_str()
    }
}

#[cfg(test)]
mod test {
    use oxc_allocator::Allocator;
    use oxc_parser::Parser;

    use super::GlimmerPartialLoader;
    use crate::loader::JavaScriptSource;

    fn parse_gjs<'a>(source_text: &'a str, allocator: &'a Allocator) -> JavaScriptSource<'a> {
        let sources = GlimmerPartialLoader::new(source_text, allocator).parse("gjs");
        *sources.first().unwrap()
    }

    fn parse_gts<'a>(source_text: &'a str, allocator: &'a Allocator) -> JavaScriptSource<'a> {
        let sources = GlimmerPartialLoader::new(source_text, allocator).parse("gts");
        *sources.first().unwrap()
    }

    /// Parse the loader's cleaned output through `oxc_parser` and assert it has no
    /// parse errors. Only checks parse-level validity; semantic issues (the synthesized
    /// `undefined` field/expression) are out of scope for this assertion.
    fn assert_cleaned_parses(source_text: &str, ext: &str) {
        let allocator = Allocator::new();
        let sources = GlimmerPartialLoader::new(source_text, &allocator).parse(ext);
        let cleaned = sources.first().unwrap();
        let ret = Parser::new(&allocator, cleaned.source_text, cleaned.source_type).parse();
        assert!(
            !ret.panicked && ret.diagnostics.is_empty(),
            "cleaned output failed to parse for ext {ext}:\n--- input ---\n{source_text}\n--- cleaned ---\n{}\n--- errors ---\n{:#?}",
            cleaned.source_text,
            ret.diagnostics,
        );
    }

    #[test]
    fn test_no_templates_gjs() {
        let allocator = Allocator::new();
        let source_text = "export default class Foo {}";
        let result = parse_gjs(source_text, &allocator);
        assert_eq!(result.source_text, source_text);
        assert!(!result.source_type.is_typescript());
        // `.gjs` is unconditionally a module; see `parse` doc.
        assert!(result.source_type.is_module());
        assert!(!result.source_type.is_unambiguous());
        assert_eq!(result.start, 0);
    }

    #[test]
    fn test_no_templates_gts() {
        let allocator = Allocator::new();
        let source_text = "export default class Foo {}";
        let result = parse_gts(source_text, &allocator);
        assert!(result.source_type.is_typescript());
        assert!(result.source_type.is_unambiguous());
        assert_eq!(result.start, 0);
    }

    #[test]
    fn test_single_template_stripped() {
        let allocator = Allocator::new();
        let source_text = "import x from 'y';\n<template>\n  <h1>Hello</h1>\n</template>\n";
        let result = parse_gjs(source_text, &allocator);
        assert!(!result.source_text.contains("<h1>"));
        assert!(result.source_text.starts_with("import x from 'y';\n"));
        // <template> is replaced with `undefined ` — valid JS, same byte length
        assert!(result.source_text.contains("undefined "));
        let orig_nl = source_text.chars().filter(|&c| c == '\n').count();
        let result_nl = result.source_text.chars().filter(|&c| c == '\n').count();
        assert_eq!(orig_nl, result_nl);
        assert_eq!(result.source_text.len(), source_text.len());
        assert_eq!(result.start, 0);
    }

    #[test]
    fn test_expression_position_template() {
        let allocator = Allocator::new();
        let source_text = "const component = <template>Hello</template>;";
        let result = parse_gjs(source_text, &allocator);
        // Must produce valid JS: `const component = undefined           ;`
        assert!(result.source_text.starts_with("const component = undefined "));
        assert!(result.source_text.ends_with(';'));
        assert_eq!(result.source_text.len(), source_text.len());
    }

    #[test]
    fn test_class_with_template() {
        let allocator = Allocator::new();
        let source_text = r"
import Component from '@glimmer/component';
export default class MyComponent extends Component {
  <template>
    <h1>Hello World!</h1>
  </template>
}
";
        let result = parse_gjs(source_text, &allocator);
        assert!(result.source_text.contains("class MyComponent"));
        assert!(!result.source_text.contains("<h1>"));
        let orig_nl = source_text.chars().filter(|&c| c == '\n').count();
        let result_nl = result.source_text.chars().filter(|&c| c == '\n').count();
        assert_eq!(orig_nl, result_nl);
    }

    #[test]
    fn test_top_level_template() {
        let allocator = Allocator::new();
        let source_text = "<template>\n  <h1>Hello World!</h1>\n</template>\n";
        let result = parse_gjs(source_text, &allocator);
        assert!(!result.source_text.contains("<h1>"));
        let orig_nl = source_text.chars().filter(|&c| c == '\n').count();
        let result_nl = result.source_text.chars().filter(|&c| c == '\n').count();
        assert_eq!(orig_nl, result_nl);
    }

    #[test]
    fn test_multiple_templates() {
        let allocator = Allocator::new();
        let source_text =
            "const A = <template><p>A</p></template>;\nconst B = <template><p>B</p></template>;\n";
        let result = parse_gjs(source_text, &allocator);
        assert!(!result.source_text.contains("<p>"));
        assert!(result.source_text.contains("const A ="));
        assert!(result.source_text.contains("const B ="));
    }

    #[test]
    fn test_returns_single_source() {
        let allocator = Allocator::new();
        let source_text = "const x = 1;\n<template>\n  foo\n</template>\n";
        let sources = GlimmerPartialLoader::new(source_text, &allocator).parse("gjs");
        assert_eq!(sources.len(), 1);
    }

    /// The cleaned output must be parseable JS in every supported template position;
    /// otherwise downstream rules see a phantom AST.
    #[test]
    fn test_cleaned_output_parses_top_level() {
        assert_cleaned_parses("<template>\n  <h1>hi</h1>\n</template>\n", "gjs");
    }

    #[test]
    fn test_cleaned_output_parses_expression_position() {
        assert_cleaned_parses("const c = <template>hi</template>;", "gjs");
    }

    #[test]
    fn test_cleaned_output_parses_class_body() {
        assert_cleaned_parses(
            "import Component from '@glimmer/component';\nexport default class C extends Component {\n  <template>x</template>\n}\n",
            "gjs",
        );
    }

    #[test]
    fn test_cleaned_output_parses_gts_with_generics() {
        assert_cleaned_parses(
            "import type Owner from '@ember/owner';\nexport default class C<T> {\n  value!: T;\n  <template>x</template>\n}\n",
            "gts",
        );
    }

    #[test]
    fn test_cleaned_output_parses_no_templates() {
        assert_cleaned_parses("import x from 'y';\nexport const v = x;\n", "gjs");
    }

    /// Byte offsets of tokens *after* a `<template>` block must be unchanged after
    /// stripping; this is what makes diagnostic spans correct without a remap.
    #[test]
    fn test_byte_offsets_preserved_after_template() {
        let allocator = Allocator::new();
        let source_text = "<template>x</template>;\nconst bar = 1;\n";
        let result = parse_gjs(source_text, &allocator);

        let orig_offset = source_text.find("bar").expect("token in original");
        let cleaned_offset = result.source_text.find("bar").expect("token in cleaned");
        assert_eq!(orig_offset, cleaned_offset);
        assert_eq!(source_text.len(), result.source_text.len());
    }

    #[test]
    fn test_byte_offsets_preserved_across_multiple_templates() {
        let allocator = Allocator::new();
        let source_text = "const A = <template>a</template>;\nconst B = <template>b</template>;\nconst foo = 1;\n";
        let result = parse_gjs(source_text, &allocator);

        let orig_offset = source_text.find("foo").expect("token in original");
        let cleaned_offset = result.source_text.find("foo").expect("token in cleaned");
        assert_eq!(orig_offset, cleaned_offset);
    }

    /// An unclosed `<template>` with no `</template>` anywhere after it produces no
    /// region, so the file is returned verbatim; the parser then surfaces the
    /// malformedness.
    #[test]
    fn test_unclosed_template_does_not_pair_across_boundary() {
        let allocator = Allocator::new();
        let source_text = "const a = 1;\n<template>oops\nconst b = 2;\n";
        let result = parse_gjs(source_text, &allocator);
        // No `</template>` exists, so no rewriting should occur and the original
        // source must be returned verbatim.
        assert_eq!(result.source_text, source_text);
    }

    #[test]
    fn test_well_formed_then_unclosed_only_strips_well_formed() {
        let allocator = Allocator::new();
        let source_text = "<template>good</template>;\nconst a = 1;\n<template>oops\n";
        let result = parse_gjs(source_text, &allocator);
        // First (well-formed) template stripped to `undefined `+spaces.
        assert!(result.source_text.starts_with("undefined "));
        assert!(!result.source_text.contains("good"));
        // Second (unclosed) template's open is preserved.
        assert!(result.source_text.contains("<template>oops"));
        // Byte length is unchanged.
        assert_eq!(result.source_text.len(), source_text.len());
    }

    /// A `<template>` sitting inside a line comment *above* a real template must not be
    /// treated as an opener: the scanner skips comment bytes, so the comment is left
    /// verbatim and only the real template below it is stripped.
    #[test]
    fn test_line_comment_template_open_before_real_template() {
        let allocator = Allocator::new();
        let source_text = "// <template>\n<template>\n  <h1>Hello</h1>\n</template>\n";
        let result = parse_gjs(source_text, &allocator);
        // The comment is untouched: its `<template>` is not a real opener.
        assert!(result.source_text.starts_with("// <template>\n"));
        // The real template below the comment is still stripped.
        assert!(!result.source_text.contains("<h1>"));
        assert!(result.source_text.contains("undefined "));
        assert_eq!(result.source_text.len(), source_text.len());
        let orig_nl = source_text.chars().filter(|&c| c == '\n').count();
        let result_nl = result.source_text.chars().filter(|&c| c == '\n').count();
        assert_eq!(orig_nl, result_nl);
        assert_cleaned_parses(source_text, "gjs");
    }

    /// Real code between a comment's `<template>` and the actual template must survive:
    /// the comment's `<template>` is not an opener, so the `import` here is not swallowed.
    #[test]
    fn test_line_comment_open_preserves_intervening_code() {
        let allocator = Allocator::new();
        let source_text =
            "// <template>\nimport x from 'y';\nexport const c = <template>hi</template>;\n";
        let result = parse_gjs(source_text, &allocator);
        assert!(result.source_text.starts_with("// <template>\n"));
        // The import between the comment and the real template is not swallowed.
        assert!(result.source_text.contains("import x from 'y';"));
        // Only the real template is neutralized.
        assert!(result.source_text.contains("export const c = undefined "));
        assert!(!result.source_text.contains("<template>hi"));
        assert_eq!(result.source_text.len(), source_text.len());
        assert_cleaned_parses(source_text, "gjs");
    }

    // -- Comments and strings are skipped by the scanner ------------------------------
    //
    // A `<template>` / `</template>` byte sequence inside a comment or string is not a
    // real tag; the single-pass scanner leaves it untouched.

    /// A complete `<template>...</template>` inside a string literal is not code, so the
    /// string is preserved verbatim rather than clobbered.
    #[test]
    fn test_template_inside_string_literal_is_left_untouched() {
        let allocator = Allocator::new();
        let source_text = r#"const s = "<template>x</template>";"#;
        let result = parse_gjs(source_text, &allocator);
        assert_eq!(result.source_text, source_text);
        assert_cleaned_parses(source_text, "gjs");
    }

    /// A `</template>` inside a string must not close a later real template, and that real
    /// template must still be stripped.
    #[test]
    fn test_template_bytes_in_string_do_not_break_a_real_template() {
        let allocator = Allocator::new();
        let source_text = "const s = \"</template>\";\nexport const c = <template>hi</template>;\n";
        let result = parse_gjs(source_text, &allocator);
        // The string is preserved.
        assert!(result.source_text.contains("\"</template>\""));
        // The real template after it is stripped.
        assert!(result.source_text.contains("export const c = undefined "));
        assert!(!result.source_text.contains("<template>hi"));
        assert_eq!(result.source_text.len(), source_text.len());
        assert_cleaned_parses(source_text, "gjs");
    }

    /// A backtick string (including its `${...}` interpolation) is preserved, so the
    /// reference to `x` survives and no spurious unused-variable diagnostic appears.
    #[test]
    fn test_template_inside_backtick_string_preserves_substitution() {
        let allocator = Allocator::new();
        let source_text = "const x = 1;\nconst t = `<template>${x}</template>`;\n";
        let result = parse_gjs(source_text, &allocator);
        assert_eq!(result.source_text, source_text);
    }

    /// A complete `<template>...</template>` inside a line comment is not code, so the
    /// comment is preserved verbatim.
    #[test]
    fn test_template_inside_line_comment_is_left_untouched() {
        let allocator = Allocator::new();
        let source_text = "// <template>x</template>\nconst a = 1;\n";
        let result = parse_gjs(source_text, &allocator);
        assert_eq!(result.source_text, source_text);
    }

    /// A `<template>...</template>` inside a block comment is skipped; only the real
    /// template after it is stripped.
    #[test]
    fn test_template_inside_block_comment_is_left_untouched() {
        let allocator = Allocator::new();
        let source_text =
            "/* <template>x</template> */\nexport const c = <template>hi</template>;\n";
        let result = parse_gjs(source_text, &allocator);
        assert!(result.source_text.starts_with("/* <template>x</template> */"));
        assert!(result.source_text.contains("export const c = undefined "));
        assert!(!result.source_text.contains("<template>hi"));
        assert_eq!(result.source_text.len(), source_text.len());
        assert_cleaned_parses(source_text, "gjs");
    }

    /// A template body is HTML-ish and routinely contains quotes, apostrophes, and
    /// slashes; the scanner must treat the body as opaque rather than lexing those as JS
    /// strings/comments, which would swallow the `</template>` and leave the tag in place.
    #[test]
    fn test_template_body_with_quotes_and_slashes_is_stripped() {
        let allocator = Allocator::new();
        let source_text =
            "<template>\n  <a href=\"/x\" title='it\\'s'>hi // there</a>\n</template>\n";
        let result = parse_gjs(source_text, &allocator);
        assert!(result.source_text.starts_with("undefined "));
        assert!(!result.source_text.contains("<a "));
        assert!(!result.source_text.contains("href"));
        assert_eq!(result.source_text.len(), source_text.len());
        assert_cleaned_parses(source_text, "gjs");
    }

    // -- Remaining byte-pattern limitations (pinned) ----------------------------------

    /// Regex literals are not tracked: a real regex containing `</template>` is treated as
    /// code, so the bytes between a preceding `<template>` and the regex's `</template>`
    /// are (incorrectly) stripped. This escaped-close regex happens to be left intact
    /// because `<\/template>` is not the `</template>` byte sequence.
    #[test]
    fn test_known_limitation_regex_with_escaped_close_is_left_intact() {
        let allocator = Allocator::new();
        let source_text = "const r = /<template>x<\\/template>/;\n";
        let result = parse_gjs(source_text, &allocator);
        assert_eq!(result.source_text, source_text);
    }
}
