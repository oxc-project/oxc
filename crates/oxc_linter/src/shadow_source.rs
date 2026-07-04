//! Shadow source construction for files parsed by an external (JS) parser.
//!
//! Custom parsers (e.g. `ember-eslint-parser`) produce ASTs containing node types which
//! cannot be represented in Oxc's AST (e.g. `GlimmerTemplate`), so such files cannot be
//! parsed by the native parser directly. To run native rules on them anyway, a "shadow
//! source" is built: the original source text with each custom node's span (a "masked
//! region", reported from JS side) replaced in-place by a valid JS placeholder of the
//! same byte length:
//!
//! - Expression position: a template literal `` ` ` `` filled with whitespace.
//! - Class element position (parent is a `ClassBody`): a `static { }` block,
//!   or pure whitespace if the region is too short for one.
//!
//! Newlines inside the region are preserved, so both byte offsets and line numbers of
//! everything outside masked regions are identical between the original and the shadow.
//! Native diagnostics and fixes therefore map back to the original file with no source
//! mapping at all; diagnostics inside masked regions are discarded by the caller.
//!
//! Usage occurring only inside a region is injected into the placeholder as `${ref}`
//! interpolations (or `ref;` statements in a `static` block), so that native rules see
//! it. A ref is a simple expression reported from JS side: a variable name (per the
//! parser's scope manager, e.g. a component used only inside an Ember `<template>` -
//! keeps `no-unused-vars` correct), `this` (keeps `class-methods-use-this` correct),
//! or `this.#name` (keeps `no-unused-private-class-members` correct).

use oxc_span::Span;

/// Input for native linting of a file parsed by an external (JS) parser.
///
/// Produced by [`crate::Linter::run_with_js_parser`], consumed by the lint service runtime.
#[derive(Debug)]
pub struct ShadowLintInput {
    /// Shadow source text. Same byte length as the original source text, and identical
    /// to it outside masked regions, so spans of native diagnostics map back to the
    /// original file unchanged.
    pub source_text: String,
    /// Byte spans of the masked regions. Native diagnostics whose span intersects one of
    /// these are discarded (they refer to placeholder code, not real source). Sorted by
    /// start offset.
    pub masked_spans: Vec<Span>,
}

/// A masked region of the source text, with spans converted to UTF-8 byte offsets.
#[derive(Debug)]
pub struct MaskedRegion {
    /// Byte span of the region in the original source text
    pub span: Span,
    /// `true` if the region is a class element (its parent node is a `ClassBody`)
    pub class_member: bool,
    /// Expressions to inject into the placeholder: variable names referenced inside
    /// the region but declared outside all regions, `this`, or `this.#name`
    /// (see module doc)
    pub refs: Vec<String>,
}

/// Byte length of `static{` + `}`
const STATIC_BLOCK_DELIMS_LEN: usize = 8;

/// Build the shadow source for `source_text` with the given masked regions.
///
/// Returns `None` if the regions are invalid (out of bounds, overlapping, unsorted,
/// or not on UTF-8 character boundaries) - the caller then skips native linting.
///
/// The returned string always has exactly the same byte length as `source_text`,
/// and is identical to it outside the masked regions.
pub fn build_shadow_source(source_text: &str, regions: &[MaskedRegion]) -> Option<String> {
    let mut shadow = Vec::with_capacity(source_text.len());
    let mut cursor: usize = 0;

    for region in regions {
        let start = region.span.start as usize;
        let end = region.span.end as usize;
        // Regions must be sorted, non-overlapping, in bounds, non-empty,
        // and on character boundaries
        if start < cursor
            || end <= start
            || end > source_text.len()
            || !source_text.is_char_boundary(start)
            || !source_text.is_char_boundary(end)
        {
            return None;
        }

        shadow.extend_from_slice(&source_text.as_bytes()[cursor..start]);
        write_placeholder(&mut shadow, &source_text.as_bytes()[start..end], region);
        cursor = end;
    }

    shadow.extend_from_slice(&source_text.as_bytes()[cursor..]);

    debug_assert_eq!(shadow.len(), source_text.len());
    // SAFETY-free conversion: all placeholder bytes are ASCII, and bytes outside
    // placeholders are copied verbatim from `source_text`, so `shadow` is valid UTF-8.
    // Use the checked conversion anyway; it cannot fail.
    String::from_utf8(shadow).ok()
}

/// Write the placeholder for one masked region to `out`.
///
/// The placeholder has exactly `original.len()` bytes. Newlines in `original` are kept
/// at the same positions (except where delimiters overwrite them - see below); every
/// other byte becomes a space, a delimiter, or part of an injected ref.
fn write_placeholder(out: &mut Vec<u8>, original: &[u8], region: &MaskedRegion) {
    let len = original.len();

    // Map each original byte: keep newlines (each byte of a multi-byte character
    // becomes a space, preserving byte length; a lone `\r` is fine in all contexts
    // the placeholder produces).
    let placeholder_start = out.len();
    out.extend(original.iter().map(|&b| if b == b'\n' || b == b'\r' { b } else { b' ' }));
    let placeholder = &mut out[placeholder_start..];

    if region.class_member {
        // Class element position: `static{ ... }` block, with refs as `name;` statements.
        // If the region is too short for `static{}`, or writing the delimiters would
        // overwrite a newline (shifting line numbers of everything after the region),
        // leave it as pure whitespace - valid between class elements.
        if len < STATIC_BLOCK_DELIMS_LEN
            || placeholder[..7].iter().any(|&b| b == b'\n' || b == b'\r')
            || matches!(placeholder[len - 1], b'\n' | b'\r')
        {
            return;
        }
        placeholder[..7].copy_from_slice(b"static{");
        placeholder[len - 1] = b'}';
        inject_refs(&mut placeholder[7..len - 1], &region.refs, RefStyle::Statement);
    } else {
        // Expression position: template literal, with refs as `${name}` interpolations.
        // A 1-byte region can't hold one; use `0` (a valid expression) instead.
        if len == 1 {
            placeholder[0] = b'0';
            return;
        }
        placeholder[0] = b'`';
        placeholder[len - 1] = b'`';
        inject_refs(&mut placeholder[1..len - 1], &region.refs, RefStyle::Interpolation);
    }
}

#[derive(Clone, Copy)]
enum RefStyle {
    /// `${name}` - inside a template literal
    Interpolation,
    /// `name;` - inside a `static` block
    Statement,
}

/// Overwrite runs of spaces in `interior` with refs (`${name}` or `name;`).
///
/// Each ref needs a contiguous run of spaces (a newline cannot appear inside `${...}`,
/// and preserved newlines must not be overwritten). Refs that don't fit are dropped -
/// this only degrades `no-unused-vars` accuracy, never correctness of the placeholder.
fn inject_refs(interior: &mut [u8], refs: &[String], style: RefStyle) {
    let mut pos = 0;
    for name in refs {
        let needed = match style {
            RefStyle::Interpolation => name.len() + 3,
            RefStyle::Statement => name.len() + 1,
        };

        // Find the next run of `needed` consecutive spaces at or after `pos`
        let Some(run_start) = find_space_run(&interior[pos..], needed) else { return };
        let run_start = pos + run_start;

        let slot = &mut interior[run_start..run_start + needed];
        match style {
            RefStyle::Interpolation => {
                slot[0] = b'$';
                slot[1] = b'{';
                slot[2..needed - 1].copy_from_slice(name.as_bytes());
                slot[needed - 1] = b'}';
            }
            RefStyle::Statement => {
                slot[..needed - 1].copy_from_slice(name.as_bytes());
                slot[needed - 1] = b';';
            }
        }
        pos = run_start + needed;
    }
}

/// Find the start of the first run of at least `needed` consecutive spaces in `bytes`.
fn find_space_run(bytes: &[u8], needed: usize) -> Option<usize> {
    debug_assert!(needed > 0);
    let mut run_start = 0;
    let mut run_len = 0;
    for (i, &b) in bytes.iter().enumerate() {
        if b == b' ' {
            if run_len == 0 {
                run_start = i;
            }
            run_len += 1;
            if run_len == needed {
                return Some(run_start);
            }
        } else {
            run_len = 0;
        }
    }
    None
}

#[cfg(test)]
#[expect(clippy::cast_possible_truncation)]
mod tests {
    use super::*;

    fn region(start: u32, end: u32, class_member: bool, refs: &[&str]) -> MaskedRegion {
        MaskedRegion {
            span: Span::new(start, end),
            class_member,
            refs: refs.iter().map(ToString::to_string).collect(),
        }
    }

    #[test]
    fn no_regions_returns_original() {
        let source = "const a = 1;\n";
        let shadow = build_shadow_source(source, &[]).unwrap();
        assert_eq!(shadow, source);
    }

    #[test]
    fn expression_position_becomes_template_literal() {
        let source = "const t = <template>hi</template>;\n";
        let start = source.find('<').unwrap() as u32;
        let end = source.rfind('>').unwrap() as u32 + 1;
        let shadow = build_shadow_source(source, &[region(start, end, false, &[])]).unwrap();
        assert_eq!(shadow.len(), source.len());
        let expected = format!("const t = `{}`;\n", " ".repeat((end - start) as usize - 2));
        assert_eq!(shadow, expected);
    }

    #[test]
    fn class_member_becomes_static_block() {
        let source = "class A {\n  <template>hi</template>\n}\n";
        let start = source.find('<').unwrap() as u32;
        let end = source.rfind("e>").unwrap() as u32 + 2;
        let shadow = build_shadow_source(source, &[region(start, end, true, &[])]).unwrap();
        assert_eq!(shadow.len(), source.len());
        let expected = format!(
            "class A {{\n  static{{{}}}\n}}\n",
            " ".repeat((end - start) as usize - STATIC_BLOCK_DELIMS_LEN)
        );
        assert_eq!(shadow, expected);
    }

    #[test]
    fn newlines_preserved() {
        let source = "x(<template>\n  a\n  b\n</template>);\n";
        let start = source.find('<').unwrap() as u32;
        let end = source.rfind('>').unwrap() as u32 + 1;
        let shadow = build_shadow_source(source, &[region(start, end, false, &[])]).unwrap();
        assert_eq!(shadow.len(), source.len());
        // Newlines are at identical byte positions
        for (i, byte) in source.bytes().enumerate() {
            if byte == b'\n' {
                assert_eq!(shadow.as_bytes()[i], b'\n');
            }
        }
        assert!(shadow.starts_with("x(`"));
        assert!(shadow.ends_with("`);\n"));
    }

    #[test]
    fn refs_injected_as_interpolations() {
        let source = "const t = <template>{{foo}} {{bar}}</template>;\n";
        let start = source.find('<').unwrap() as u32;
        let end = source.rfind('>').unwrap() as u32 + 1;
        let shadow =
            build_shadow_source(source, &[region(start, end, false, &["foo", "bar"])]).unwrap();
        assert_eq!(shadow.len(), source.len());
        assert!(shadow.contains("${foo}"));
        assert!(shadow.contains("${bar}"));
        assert!(shadow.starts_with("const t = `"));
        assert!(shadow.ends_with("`;\n"));
    }

    #[test]
    fn refs_injected_as_statements_in_static_block() {
        let source = "class A {\n  <template>{{foo}}</template>\n}\n";
        let start = source.find('<').unwrap() as u32;
        let end = source.rfind("e>").unwrap() as u32 + 2;
        let shadow = build_shadow_source(source, &[region(start, end, true, &["foo"])]).unwrap();
        assert_eq!(shadow.len(), source.len());
        assert!(shadow.contains("static{foo;"));
        assert!(shadow.contains('}'));
    }

    #[test]
    fn this_and_private_refs_injected() {
        // `this` and `this.#name` refs (reported for `this` / private-name usage inside
        // the region) are spliced like variable refs, in both placeholder styles
        let source =
            "class A {\n  m() {\n    return <template>{{this.foo}} x</template>;\n  }\n}\n";
        let start = source.find('<').unwrap() as u32;
        let end = source.rfind('>').unwrap() as u32 + 1;
        let shadow =
            build_shadow_source(source, &[region(start, end, false, &["this", "this.#count"])])
                .unwrap();
        assert_eq!(shadow.len(), source.len());
        assert!(shadow.contains("${this}"));
        assert!(shadow.contains("${this.#count}"));

        let source = "class A {\n  <template>{{this.foo}} some text</template>\n}\n";
        let start = source.find('<').unwrap() as u32;
        let end = source.rfind("e>").unwrap() as u32 + 2;
        let shadow =
            build_shadow_source(source, &[region(start, end, true, &["this", "this.#count"])])
                .unwrap();
        assert_eq!(shadow.len(), source.len());
        assert!(shadow.contains("static{this;"));
        assert!(shadow.contains("this.#count;"));
    }

    #[test]
    fn refs_that_do_not_fit_are_dropped() {
        let source = "x(<t>a</t>);";
        let start = source.find('<').unwrap() as u32;
        let end = source.rfind('>').unwrap() as u32 + 1;
        let shadow = build_shadow_source(
            source,
            &[region(start, end, false, &["extremelyLongVariableName"])],
        )
        .unwrap();
        assert_eq!(shadow.len(), source.len());
        assert!(!shadow.contains("${"));
    }

    #[test]
    fn multibyte_content_blanked_to_same_byte_length() {
        let source = "const t = <template>héllo 🎉</template>;";
        let start = source.find('<').unwrap() as u32;
        let end = source.rfind('>').unwrap() as u32 + 1;
        let shadow = build_shadow_source(source, &[region(start, end, false, &[])]).unwrap();
        assert_eq!(shadow.len(), source.len());
        assert!(shadow.is_ascii());
    }

    #[test]
    fn multiple_regions() {
        let source = "const a = <t>x</t>;\nconst b = <t>y</t>;\n";
        let r1_start = source.find("<t>x").unwrap() as u32;
        let r1_end = source.find(";\n").unwrap() as u32;
        let r2_start = source.find("<t>y").unwrap() as u32;
        let r2_end = source.rfind('>').unwrap() as u32 + 1;
        let shadow = build_shadow_source(
            source,
            &[region(r1_start, r1_end, false, &[]), region(r2_start, r2_end, false, &[])],
        )
        .unwrap();
        assert_eq!(shadow.len(), source.len());
        let mask = format!("`{}`", " ".repeat("<t>x</t>".len() - 2));
        assert_eq!(shadow, format!("const a = {mask};\nconst b = {mask};\n"));
    }

    #[test]
    fn invalid_regions_rejected() {
        let source = "const a = 1;";
        // Out of bounds
        assert!(build_shadow_source(source, &[region(5, 100, false, &[])]).is_none());
        // Empty
        assert!(build_shadow_source(source, &[region(5, 5, false, &[])]).is_none());
        // Overlapping
        assert!(
            build_shadow_source(source, &[region(2, 8, false, &[]), region(6, 10, false, &[])])
                .is_none()
        );
        // Unsorted
        assert!(
            build_shadow_source(source, &[region(6, 10, false, &[]), region(2, 5, false, &[])])
                .is_none()
        );
        // Not on a char boundary
        let source = "const a = '🎉';";
        let emoji_start = source.find('🎉').unwrap() as u32;
        assert!(
            build_shadow_source(source, &[region(emoji_start + 1, emoji_start + 3, false, &[])])
                .is_none()
        );
    }

    #[test]
    fn class_member_too_short_for_static_block_becomes_whitespace() {
        let source = "class A { <t></t> }";
        let start = source.find('<').unwrap() as u32;
        let end = source.rfind('>').unwrap() as u32 + 1;
        let shadow = build_shadow_source(source, &[region(start, end, true, &[])]).unwrap();
        assert_eq!(shadow, "class A {         }");
    }
}
