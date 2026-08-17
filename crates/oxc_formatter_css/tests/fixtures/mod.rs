//! Fixture tests for cases the Prettier conformance suite does NOT cover.
//!
//! Expected outputs were verified against `prettier` by hand;
//! when adding a fixture, do the same (`npx prettier@<oxfmt-bundle-version> --parser <variant>`).

use std::path::Path;

use oxc_allocator::{Allocator, ArenaVec};
use oxc_formatter_css::{CssFormatOptions, CssVariant, format};
use oxc_formatter_tests::{FixtureFormatter, OptionSet, build_fixture_snapshot};

mod options;
use options::apply_css_options;

struct CssHarness;

impl FixtureFormatter for CssHarness {
    type Options = CssFormatOptions;

    fn parse_options(json: &OptionSet) -> Self::Options {
        let mut options = CssFormatOptions::default();
        apply_css_options(&mut options, json);
        options
    }

    fn format(source: &str, path: &Path, options: &Self::Options) -> String {
        // The dialect comes from the fixture extension, like oxfmt's classifier.
        let variant = match path.extension().and_then(|e| e.to_str()) {
            Some("scss") => CssVariant::Scss,
            Some("less") => CssVariant::Less,
            _ => CssVariant::Css,
        };
        let options = CssFormatOptions { variant, ..*options };

        // Fixtures under `embedded/` exercise the dispatcher entry point
        // (`format_to_ir`, css-in-js), which tolerates
        // `` `PLACEHOLDER-N` `` markers in value/selector position.
        if path.components().any(|c| c.as_os_str() == "embedded") {
            return format_embedded(source, options);
        }

        let allocator = Allocator::default();
        format(&allocator, source, options)
            .expect("format should succeed")
            .print()
            .expect("print should succeed")
            .into_code()
    }
}

/// Format through `format_to_ir` and print the raw IR, mirroring what the
/// oxfmt dispatcher + parent template printing do (minus `${}` substitution).
fn format_embedded(source: &str, options: CssFormatOptions) -> String {
    use oxc_formatter_core::{Document, FormatElement, FormatOptions, TextWidth};

    let allocator = Allocator::default();
    let session =
        oxc_formatter_core::FormatSession::new(&allocator, oxc_formatter_core::InputKind::Fragment);
    let embedded = oxc_formatter_css::format_to_ir(
        &session, source, options, /* template_placeholders */ true,
    )
    .expect("format should succeed");
    // Simulate the host: replace each typed placeholder with the canonical
    // sentinel (the real host substitutes `${expr}`; tests have no expressions).
    // The printer `debug_assert`s on any surviving `EmbedPlaceholder`.
    let elements = ArenaVec::from_iter_in(
        embedded.ir.iter().map(|element| match element {
            FormatElement::EmbedPlaceholder(index) => {
                let text = allocator.alloc_str(&std::format!("`PLACEHOLDER-{index}`"));
                FormatElement::Text {
                    text,
                    width: TextWidth::from_text(text, options.indent_width),
                }
            }
            other => other.clone(),
        }),
        &session.allocator(),
    );
    let mut code = Document::new(elements, Vec::new())
        .print(source.len(), options.as_print_options())
        .expect("print should succeed")
        .into_code();
    // The embedded entry point emits no trailing newline (the parent owns it);
    // add one so snapshots stay diff-friendly.
    code.push('\n');
    code
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

/// A leading BOM is preserved (Prettier does the same).
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

/// Any parse error must surface as `Err` from the standalone `format()` entry,
/// including oxc-css-parser's recoverable ones (top-level declarations are invalid here
/// too — only the embedded `format_to_ir` entry tolerates them, see
/// `embedded/scss/top-level-declaration.scss`).
#[test]
fn parse_error_is_err() {
    let allocator = Allocator::default();
    let css = CssFormatOptions::default();
    let scss = CssFormatOptions { variant: CssVariant::Scss, ..css };
    for (source, options) in [
        // Top-level declaration: valid only as an embedded css-in-js fragment
        // (`format_to_ir`); standalone files must reject it like Dart Sass does.
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
        // `2N-1` with a glued minus is invalid An+B for oxc-css-parser
        // (postcss-selector-parser accepts and lowercases it).
        ("a:nth-child(2N-1) { color: red; }", css),
    ] {
        assert!(format(&allocator, source, options).is_err(), "{source:?} should fail to format");
    }
}
