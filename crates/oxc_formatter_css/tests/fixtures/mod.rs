//! Fixture tests for cases the Prettier conformance suite does NOT cover
//! (discovered while wiring the crate into oxfmt — plan Step 5).
//!
//! Every `.css`/`.scss`/`.less` file under `fixtures/format/` is formatted
//! with each option set from the nearest `options.json` and snapshotted next
//! to it. Expected outputs were verified against `prettier@3.8.3` by hand;
//! when adding a fixture, do the same (`npx prettier@3.8.3 --parser <v>`).

use std::path::Path;

use oxc_allocator::Allocator;
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
        // `@prettier-placeholder-N-id` markers in value/selector position.
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
    use oxc_formatter_core::{Document, EmbeddedContext, FormatOptions, Printer};

    let allocator = Allocator::default();
    let group_id_builder = oxc_formatter_core::UniqueGroupIdBuilder::default();
    let ctx = EmbeddedContext {
        allocator: &allocator,
        group_id_builder: &group_id_builder,
        dispatcher: None,
    };
    let elements =
        oxc_formatter_css::format_to_ir(&ctx, source, options).expect("format should succeed");
    let document = Document::new(elements, Vec::new());
    document.propagate_expand();
    let (elements, tailwind_classes) = document.into_elements_and_tailwind_classes();
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

/// Any parse error must surface as `Err` (the oxfmt Prettier-fallback
/// trigger), including raffia's recoverable ones — EXCEPT top-level
/// declarations, which postcss accepts (see `top-level-declaration.scss`).
#[test]
fn parse_error_is_err() {
    let allocator = Allocator::default();
    let css = CssFormatOptions::default();
    let scss = CssFormatOptions { variant: CssVariant::Scss, ..css };
    for (source, options) in [
        // Unclosed block (postcss also rejects this).
        ("a {\n  color: red;\n", css),
        // IE star hack: postcss tolerates it, raffia does not -> fallback.
        ("a { *zoom: 1; }", css),
        // css-in-js `${}` markers in value position...
        ("a { color: @prettier-placeholder-0-id; }", scss),
        // ...and in selector position stay errors in the STANDALONE entry
        // (`format_to_ir` tolerates them via the raffia fork option;
        // see `fixtures/embedded/`).
        (".a-@prettier-placeholder-0-id {\n}", scss),
        // `2N-1` with a glued minus is invalid An+B for raffia
        // (postcss-selector-parser accepts and lowercases it).
        ("a:nth-child(2N-1) { color: red; }", css),
    ] {
        assert!(format(&allocator, source, options).is_err(), "{source:?} should fail to format");
    }
}
