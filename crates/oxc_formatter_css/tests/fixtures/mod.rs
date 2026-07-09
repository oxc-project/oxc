//! Fixture tests for cases the Prettier conformance suite does NOT cover.
//!
//! Expected outputs were verified against `prettier` by hand;
//! when adding a fixture, do the same (`npx prettier@<oxfmt-bundle-version> --parser <variant>`).

use std::path::Path;

use oxc_allocator::{Allocator, ArenaVec};
use oxc_formatter_core::{
    IndentStyle, IndentWidth, LineEnding, LineWidth,
    test_support::{FixtureFormatter, OptionSet, build_fixture_snapshot},
};
use oxc_formatter_css::{CssFormatOptions, CssVariant, format};

struct CssHarness;

impl FixtureFormatter for CssHarness {
    type Options = CssFormatOptions;

    fn parse_options(json: &OptionSet) -> Self::Options {
        let mut options = CssFormatOptions::default();

        for (key, value) in json {
            match key.as_str() {
                "printWidth" => {
                    if let Some(n) = value.as_u64()
                        && let Ok(width) = LineWidth::try_from(u16::try_from(n).unwrap())
                    {
                        options.line_width = width;
                    }
                }
                "tabWidth" => {
                    if let Some(n) = value.as_u64()
                        && let Ok(width) = IndentWidth::try_from(u8::try_from(n).unwrap())
                    {
                        options.indent_width = width;
                    }
                }
                "useTabs" => {
                    if let Some(b) = value.as_bool() {
                        options.indent_style =
                            if b { IndentStyle::Tab } else { IndentStyle::Space };
                    }
                }
                "endOfLine" => {
                    if let Some(s) = value.as_str() {
                        options.line_ending = match s {
                            "lf" => LineEnding::Lf,
                            "crlf" => LineEnding::Crlf,
                            "cr" => LineEnding::Cr,
                            _ => LineEnding::default(),
                        };
                    }
                }
                "singleQuote" => {
                    if let Some(b) = value.as_bool() {
                        options.single_quote = b.into();
                    }
                }
                "trailingComma" => {
                    if let Some(s) = value.as_str() {
                        options.trailing_commas = match s {
                            "none" => oxc_formatter_css::TrailingCommas::Never,
                            _ => oxc_formatter_css::TrailingCommas::Always,
                        };
                    }
                }
                _ => {}
            }
        }

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
        format(&allocator, source, options, None)
            .expect("format should succeed")
            .print()
            .expect("print should succeed")
            .into_code()
    }
}

/// Format through `format_to_ir` and print the raw IR, mirroring what the
/// oxfmt dispatcher + parent template printing do (minus `${}` substitution).
fn format_embedded(source: &str, options: CssFormatOptions) -> String {
    use oxc_formatter_core::{
        Document, EmbeddedContext, FormatElement, FormatOptions, Printer, TextWidth,
    };

    let allocator = Allocator::default();
    let group_id_builder = oxc_formatter_core::UniqueGroupIdBuilder::default();
    let ctx = EmbeddedContext {
        allocator: &allocator,
        group_id_builder: &group_id_builder,
        dispatcher: None,
    };
    let embedded =
        oxc_formatter_css::format_to_ir(&ctx, source, options).expect("format should succeed");
    let document = Document::new(embedded.ir, Vec::new());
    document.propagate_expand();
    let (elements, tailwind_classes) = document.into_elements_and_tailwind_classes();
    // Simulate the host: replace each typed placeholder with the canonical
    // sentinel (the real host substitutes `${expr}`; tests have no expressions).
    // The printer `debug_assert`s on any surviving `EmbedPlaceholder`.
    let elements = ArenaVec::from_iter_in(
        elements.iter().map(|element| match element {
            FormatElement::EmbedPlaceholder(index) => {
                let text = allocator.alloc_str(&std::format!("`PLACEHOLDER-{index}`"));
                FormatElement::Text {
                    text,
                    width: TextWidth::from_text(text, options.indent_width),
                }
            }
            other => other.clone(),
        }),
        &ctx.allocator,
    )
    .into_arena_slice();
    let mut code =
        Printer::with_capacity(source.len(), options.as_print_options(), &tailwind_classes)
            .print(elements)
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
        assert!(
            format(&allocator, source, options, None).is_err(),
            "{source:?} should fail to format"
        );
    }
}
