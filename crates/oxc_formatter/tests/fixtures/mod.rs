use std::{env::current_dir, fs, path::Path, str::FromStr};

use oxc_allocator::Allocator;
use oxc_formatter::{
    ArrowParentheses, BracketSameLine, BracketSpacing, FormatOptions, Formatter, IndentStyle,
    IndentWidth, LineWidth, QuoteStyle, Semicolons, TrailingCommas,
};
use oxc_parser::{ParseOptions, Parser};
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
                if let Some(n) = value.as_str()
                    && let Ok(width) = LineWidth::from_str(n)
                {
                    options.line_width = width;
                }
            }
            "tabWidth" => {
                if let Some(n) = value.as_str()
                    && let Ok(width) = IndentWidth::from_str(n)
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
        return String::new();
    }

    let mut parts: Vec<String> = json
        .iter()
        .map(|(k, v)| {
            let value_str = match v {
                serde_json::Value::Bool(b) => b.to_string(),
                serde_json::Value::Number(n) => n.to_string(),
                serde_json::Value::String(s) => format!("\"{s}\""),
                _ => v.to_string(),
            };
            format!("{k}: {value_str}")
        })
        .collect();

    parts.sort();
    parts.join(", ")
}

/// Format a source file with given options
fn format_source(source_text: &str, source_type: SourceType, options: FormatOptions) -> String {
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source_text, source_type)
        .with_options(ParseOptions {
            parse_regular_expression: false,
            allow_v8_intrinsics: true,
            allow_return_outside_function: true,
            preserve_parens: false,
        })
        .parse();

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

    if !option_sets.is_empty() {
        snapshot.push_str("==================== Output ====================\n");
    }

    for option_json in option_sets {
        let options_display = format_options_display(&option_json);
        let options_line = if options_display.is_empty() {
            String::default()
        } else {
            format!("{{ {options_display} }}")
        };
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
