use std::{env::current_dir, fs, path::Path};

use oxc_allocator::Allocator;
use oxc_formatter::{
    ArrowParentheses, BracketSameLine, BracketSpacing, FormatOptions, Formatter, IndentStyle,
    IndentWidth, LineWidth, QuoteStyle, Semicolons, TrailingCommas, get_parse_options,
};
use oxc_parser::Parser;
use oxc_span::SourceType;

type OptionSet = serde_json::Map<String, serde_json::Value>;

/// Resolve format options for a test file by walking up the directory tree
fn resolve_options(test_file: &Path) -> Vec<OptionSet> {
    let mut current_dir = test_file.parent();

    // Walk up the directory tree looking for options.json
    while let Some(dir) = current_dir {
        let options_file = dir.join("options.json");
        if options_file.exists() {
            if let Ok(content) = fs::read_to_string(&options_file)
                && let Ok(option_sets) = serde_json::from_str::<Vec<OptionSet>>(&content)
            {
                return option_sets;
            }
            break;
        }

        // Stop at fixtures directory
        if dir.ends_with("fixtures") {
            break;
        }

        current_dir = dir.parent();
    }

    // Default: single option set with default options (empty map)
    vec![serde_json::Map::new()]
}

/// Parse JSON options into FormatOptions
fn parse_format_options(json: &OptionSet) -> FormatOptions {
    let mut options = FormatOptions::default();

    for (key, value) in json {
        match key.as_str() {
            "semi" => {
                if let Some(b) = value.as_bool() {
                    options.semicolons = if b { Semicolons::Always } else { Semicolons::AsNeeded };
                }
            }
            "singleQuote" => {
                if let Some(b) = value.as_bool() {
                    options.quote_style = if b { QuoteStyle::Single } else { QuoteStyle::Double };
                }
            }
            "jsxSingleQuote" => {
                if let Some(b) = value.as_bool() {
                    options.jsx_quote_style =
                        if b { QuoteStyle::Single } else { QuoteStyle::Double };
                }
            }
            "arrowParens" => {
                if let Some(s) = value.as_str() {
                    options.arrow_parentheses = match s {
                        "always" => ArrowParentheses::Always,
                        "avoid" => ArrowParentheses::AsNeeded,
                        _ => options.arrow_parentheses,
                    };
                }
            }
            "trailingComma" => {
                if let Some(s) = value.as_str() {
                    options.trailing_commas = match s {
                        "none" => TrailingCommas::None,
                        "es5" => TrailingCommas::Es5,
                        "all" => TrailingCommas::All,
                        _ => options.trailing_commas,
                    };
                }
            }
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
                    options.indent_style = if b { IndentStyle::Tab } else { IndentStyle::Space };
                }
            }
            "bracketSpacing" => {
                if let Some(b) = value.as_bool() {
                    options.bracket_spacing = BracketSpacing::from(b);
                }
            }
            "bracketSameLine" | "jsxBracketSameLine" => {
                if let Some(b) = value.as_bool() {
                    options.bracket_same_line = BracketSameLine::from(b);
                }
            }
            _ => {}
        }
    }

    options
}

/// Format options to a readable string for snapshot display
fn format_options_display(json: &OptionSet) -> String {
    if json.is_empty() {
        return "{}".to_string();
    }

    let mut parts: Vec<_> = json.iter().map(|(k, v)| format!("{k}: {v}")).collect();
    parts.sort();
    format!("{{ {} }}", parts.join(", "))
}

/// Format a source file with given options
fn format_source(source_text: &str, source_type: SourceType, options: FormatOptions) -> String {
    let allocator = Allocator::default();
    let ret =
        Parser::new(&allocator, source_text, source_type).with_options(get_parse_options()).parse();
    assert!(ret.errors.is_empty());

    let formatter = Formatter::new(&allocator, options);
    formatter.build(&ret.program)
}

/// Generate snapshot for a test file
fn generate_snapshot(path: &Path, source_text: &str) -> String {
    let source_type = SourceType::from_path(path).unwrap();
    let option_sets = resolve_options(path);

    let mut snapshot = String::new();
    snapshot.push_str("==================== Input ====================\n");
    snapshot.push_str(source_text);
    snapshot.push('\n');

    snapshot.push_str("==================== Output ====================\n");

    // Test both `printWidth` for Prettier default `80` and our default `100`.
    // If already specified, test that value as well.
    let option_sets = option_sets.into_iter().flat_map(|original_option_json| {
        let mut option_json_w80 = original_option_json.clone();
        option_json_w80.insert("printWidth".to_string(), serde_json::Value::Number(80.into()));
        let mut option_json_default = original_option_json.clone();
        option_json_default.insert(
            "printWidth".to_string(),
            serde_json::Value::Number(LineWidth::default().value().into()),
        );

        if original_option_json.contains_key("printWidth") {
            vec![original_option_json, option_json_w80, option_json_default]
        } else {
            vec![option_json_w80, option_json_default]
        }
    });

    for option_json in option_sets {
        let options_line = format_options_display(&option_json);
        let separator = "-".repeat(options_line.len());

        if !options_line.is_empty() {
            snapshot.push_str(&separator);
            snapshot.push('\n');
            snapshot.push_str(&options_line);
            snapshot.push('\n');
            snapshot.push_str(&separator);
            snapshot.push('\n');
        }

        let options = parse_format_options(&option_json);
        let formatted = format_source(source_text, source_type, options);
        snapshot.push_str(&formatted);
        snapshot.push('\n');
    }

    snapshot.push_str("===================== End =====================\n");

    snapshot
}

/// Helper function to run a test for a single file
fn test_file(path: &Path) {
    let source_text = fs::read_to_string(path).unwrap();
    let snapshot = generate_snapshot(path, &source_text);
    let snapshot_path = current_dir().unwrap().join(path.parent().unwrap());
    let snapshot_name = path.file_name().unwrap().to_str().unwrap();

    insta::with_settings!({
        snapshot_path => snapshot_path,
        prepend_module_to_snapshot => false,
        snapshot_suffix => "",
        omit_expression => true,

    }, {
        insta::assert_snapshot!(snapshot_name, snapshot);
    });
}

// Include auto-generated test functions from build.rs
include!(concat!(env!("OUT_DIR"), "/generated_tests.rs"));
