//! Fixture tests for cases the Prettier conformance suite does NOT cover,
//! plus minimal re-pins of known divergences the suite DOES hit
//! (a divergence needs its own pin; the big conformance file is only a regression net).
//!
//! Expected outputs were verified against `prettier` by hand;
//! when adding a fixture, do the same with the oxfmt-bundled one
//! (`node apps/oxfmt/node_modules/prettier/bin/prettier.cjs --parser <variant>`, see `FORMATTER_POLICY.md` "Verification").

use std::path::Path;

use oxc_allocator::{Allocator, ArenaVec};
use oxc_formatter_css::{
    CssFormatOptions, CssRoot, CssVariant, SingleQuote, TrailingCommas, format, parse_for_format,
};
use oxc_formatter_tests::{FixtureFormatter, InputVariant, OptionValue, build_fixture_snapshot};

struct CssHarness;

/// What formatting must leave unchanged, see `FixtureFormatter::Fingerprint`.
#[derive(Debug, PartialEq)]
struct Fingerprint {
    comments: usize,
}

impl FixtureFormatter for CssHarness {
    type Options = CssFormatOptions;
    type Fingerprint = Fingerprint;

    fn apply_option(options: &mut Self::Options, key: &str, value: &OptionValue) -> bool {
        match key {
            "trailingComma" => {
                if let Some(s) = value.as_str() {
                    options.trailing_commas =
                        if s == "none" { TrailingCommas::Never } else { TrailingCommas::Always };
                }
            }
            "singleQuote" => {
                if let Some(b) = value.as_bool() {
                    options.single_quote = SingleQuote::from(b);
                }
            }
            _ => return false,
        }
        true
    }

    fn format(
        source: &str,
        path: &Path,
        options: &Self::Options,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let options = CssFormatOptions { variant: variant_of(path), ..*options };

        if is_embedded(path) {
            return format_embedded(source, options);
        }

        let allocator = Allocator::default();
        Ok(format(&allocator, source, options)?.print()?.into_code())
    }

    fn fingerprint(source: &str, path: &Path, options: &Self::Options) -> Fingerprint {
        let options = CssFormatOptions { variant: variant_of(path), ..*options };
        let allocator = Allocator::default();
        let parsed = parse_for_format(
            &allocator,
            source,
            options,
            if is_embedded(path) { CssRoot::CssInJsTemplate } else { CssRoot::Stylesheet },
        )
        .expect("source should parse");
        Fingerprint { comments: parsed.comments.len() }
    }

    fn input_variants(path: &Path) -> &'static [InputVariant] {
        // An embedded entry never sees a BOM: a leading U+FEFF there is content
        if is_embedded(path) { &[InputVariant::Crlf, InputVariant::Cr] } else { InputVariant::ALL }
    }
}

/// The dialect comes from the fixture extension, like oxfmt's classifier.
fn variant_of(path: &Path) -> CssVariant {
    match path.extension().and_then(|e| e.to_str()) {
        Some("scss") => CssVariant::Scss,
        Some("less") => CssVariant::Less,
        _ => CssVariant::Css,
    }
}

/// Fixtures under `embedded/` exercise the dispatcher entry point
/// (`format_to_ir`, css-in-js), which tolerates
/// `` `PLACEHOLDER-N` `` markers in value/selector position.
fn is_embedded(path: &Path) -> bool {
    path.components().any(|c| c.as_os_str() == "embedded")
}

/// Format through `format_to_ir` and print the raw IR, mirroring what the
/// oxfmt dispatcher + parent template printing do (minus `${}` substitution).
fn format_embedded(
    source: &str,
    options: CssFormatOptions,
) -> Result<String, Box<dyn std::error::Error>> {
    use oxc_formatter_core::{Document, FormatElement, FormatOptions};

    let allocator = Allocator::default();
    let session =
        oxc_formatter_core::FormatSession::new(&allocator, oxc_formatter_core::InputKind::Fragment);
    let embedded = oxc_formatter_css::format_to_ir(
        &session,
        source,
        options,
        Some(oxc_formatter_css::CssEmbeddedIn::CssInJsTemplate),
    )?;
    // Simulate the host: replace each typed placeholder with the canonical
    // sentinel (the real host substitutes `${expr}`; tests have no expressions).
    // The printer `debug_assert`s on any surviving `EmbedPlaceholder`.
    let elements = ArenaVec::from_iter_in(
        embedded.ir.iter().map(|element| match element {
            FormatElement::EmbedPlaceholder(index) => {
                let text = allocator.alloc_str(&std::format!("`PLACEHOLDER-{index}`"));
                FormatElement::text(text, options.indent_width)
            }
            other => other.clone(),
        }),
        &session.allocator(),
    );
    let mut code = Document::new(elements, Vec::new())
        .print(source.len(), options.as_print_options())?
        .into_code();
    // The embedded entry point emits no trailing newline (the parent owns it);
    // add one so snapshots stay diff-friendly.
    code.push('\n');
    Ok(code)
}

fn test_file(path: &Path) {
    // `insta::assert_snapshot!` is invoked from this file so the snapshot's
    // `source:` header records this consumer crate, not the shared harness.
    let snap = build_fixture_snapshot::<CssHarness>(path);
    insta::with_settings!({
        snapshot_path => snap.path,
        prepend_module_to_snapshot => false,
        snapshot_suffix => "",
        omit_expression => true,
    }, {
        insta::assert_snapshot!(snap.name, snap.body);
    });
}

// Include auto-generated test functions from build.rs
include!(concat!(env!("OUT_DIR"), "/generated_tests.rs"));

// ---

#[test]
fn bom_is_preserved() {
    let allocator = Allocator::default();
    let formatted =
        format(&allocator, "\u{feff}a {\n  color: red;\n}\n", CssFormatOptions::default())
            .expect("BOM input should parse")
            .print()
            .expect("print should succeed")
            .into_code();
    assert_eq!(formatted, "\u{feff}a {\n  color: red;\n}\n");
}

#[test]
fn parse_error_is_err() {
    let allocator = Allocator::default();
    let css = CssFormatOptions::default();
    let scss = CssFormatOptions { variant: CssVariant::Scss, ..css };
    let less = CssFormatOptions { variant: CssVariant::Less, ..css };
    for (source, options) in [
        // Root declaration: `TopLevelDeclaration` in a stylesheet, as opposed to a fragment (`CssRoot`)
        // (dart-sass rejects it; Less parses it like less.js).
        ("display: flex;", scss),
        // EOF/newline-unclosed constructs: oxc-css-parser (0.0.6+) recovers to a
        // valid AST but records the spec parse error, so they bail like every
        // other recoverable error. Prettier rejects all of these too ("Unclosed
        // block" / "Unclosed string" / "Unclosed bracket"); formatting them
        // would corrupt the input (`;` appended inside the unclosed construct).
        ("a {\n  color: red;\n", css),
        ("@media (min-width: 500px) {\n", css),
        ("a { content: \"abc", css),
        ("a {\n  content: \"\n}", css),
        ("a { width: calc(100% - 10px", css),
        ("a { --x: {", css),
        // css-in-js `${}` markers in value position...
        ("a { color: `PLACEHOLDER-0`; }", scss),
        // ...and in selector position stay errors in the STANDALONE entry
        // (`format_to_ir` tolerates them via the oxc-css-parser fork option;
        // see `fixtures/embedded/`).
        (".a-`PLACEHOLDER-0` {\n}", scss),
        // The raw-prelude fallback (`unknown-qualified-rule.css`) fires only
        // when a declaration's value hits a top-level `{` (name + `:` parsed);
        // a selector error with no such head stays a hard error.
        ("a, { color: red; }", css),
        // A stray closing brace (postcss rejects it too, "Unexpected }")
        ("} a { color: red }", css),
        // postcss tolerates this as raw at-rule params (Prettier destroys it to `@media ;`)
        ("@media ( { }", css),
        // Pin for crates/oxc_formatter_css/DIVERGENCES.md#grammar-owner-rejected, one input per shape.
        // CSS: a browser drops the rule
        ("a[id=func(\"foo\")] {}", css),
        ("-Option/root .a {}", css),
        (".a >>> .b {}", css),
        (".icon.is-$(network) {}", css),
        (":not(\".a\") {}", css),
        ("one \"two\" three {}", css),
        ("@import (multiple) \"a.less\";", css),
        ("@import \"a.css\" {}", css),
        ("@import ur l(a.css);", css),
        ("color: red;", css),
        ("--foo: { color: red; }", css),
        ("@charset UTF-8;", css),
        // SCSS: dart-sass rejects (root declarations: `display: flex;` above)
        ("@for $i from 1 end 3 {}", scss),
        ("a { color: @var; }", scss),
        ("@import ur l(a.css);", scss),
        // Less: lessc rejects
        (".a { width: @{min-width}; }", less),
        (".a { width: #{$var}; }", less),
        ("@IMPORT a;", less),
        ("@c: { a: true; }\n.m() when (@c [ a ] = true) {}", less),
        (".a { b: ++1; }", less),
        ("@import ur l(a.css);", less),
    ] {
        assert!(format(&allocator, source, options).is_err(), "{source:?} should fail to format");
    }
}
