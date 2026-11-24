use std::path::Path;

use oxc_allocator::Allocator;
use oxc_formatter_tests::{FixtureFormatter, InputVariant, OptionValue, build_fixture_snapshot};
use oxc_formatter_toml::{TomlFormatOptions, TrailingCommas, format};

struct TomlHarness;

impl FixtureFormatter for TomlHarness {
    type Options = TomlFormatOptions;
    /// Every character but whitespace and commas:
    /// formatting only moves those, so a dropped comment, key or value changes it.
    /// Commas and whitespace inside strings and comments are not covered.
    type Fingerprint = String;

    fn apply_option(options: &mut Self::Options, key: &str, value: &OptionValue) -> bool {
        match key {
            "trailingComma" => {
                if let Some(s) = value.as_str() {
                    options.trailing_commas =
                        if s == "none" { TrailingCommas::Never } else { TrailingCommas::Always };
                }
            }
            _ => return false,
        }
        true
    }

    fn format(
        source: &str,
        _path: &Path,
        options: &Self::Options,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let allocator = Allocator::default();
        Ok(format(&allocator, source, *options)?.print()?.into_code())
    }

    fn fingerprint(source: &str, _path: &Path, _options: &Self::Options) -> String {
        source.chars().filter(|c| !c.is_whitespace() && *c != ',').collect()
    }

    fn input_variants(_path: &Path) -> &'static [InputVariant] {
        // A lone CR is not a TOML line break, but a syntax error
        &[InputVariant::Crlf, InputVariant::Bom]
    }
}

fn test_file(path: &Path) {
    // `insta::assert_snapshot!` is invoked from this file so the snapshot's
    // `source:` header records this consumer crate, not the shared harness.
    let snap = build_fixture_snapshot::<TomlHarness>(path);
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

/// `oxc_toml::parse()` is fault-tolerant, so any parse error must surface as `Err`.
#[test]
fn parse_error_is_err() {
    for source in ["a = ", "b = = 1", "[t"] {
        assert!(
            format(&Allocator::default(), source, TomlFormatOptions::default()).is_err(),
            "{source:?} should fail to format"
        );
    }
}

/// A whitespace-only file formats to empty (Prettier does the same).
#[test]
fn whitespace_only_is_empty() {
    for (source, expected) in
        [("", ""), (" \n", ""), ("\u{feff}", "\u{feff}"), ("\u{feff} \n", "\u{feff}")]
    {
        let formatted = TomlHarness::format(source, Path::new(""), &TomlFormatOptions::default())
            .expect("should format");
        assert_eq!(formatted, expected, "{source:?}");
    }
}

/// A lone CR is not a TOML line break, so `endOfLine: "cr"` prints LF.
/// (Not a fixture: the harness asserts that `cr` output holds no LF.)
#[test]
fn cr_line_ending_prints_lf() {
    let options = TomlFormatOptions {
        line_ending: oxc_formatter_core::LineEnding::Cr,
        ..TomlFormatOptions::default()
    };
    let formatted =
        TomlHarness::format("a=1\n[t]\nb=[1,2]\n", Path::new(""), &options).expect("should format");
    assert_eq!(formatted, "a = 1\n[t]\nb = [1, 2]\n");
}
