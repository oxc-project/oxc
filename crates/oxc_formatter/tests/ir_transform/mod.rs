mod sort_imports;

use oxc_formatter::{
    CustomGroupDefinition, FormatOptions, ImportModifier, ImportSelector, QuoteStyle, Semicolons,
    SortImportsOptions, SortOrder,
};
use serde::Deserialize;

pub fn assert_format(code: &str, config_json: &str, expected: &str) {
    // NOTE: Strip leading single `\n` for better test case readability.
    let code = code.strip_prefix('\n').expect("Test code should start with a newline");
    let expected = expected.strip_prefix('\n').expect("Expected code should start with a newline");

    let options = parse_test_config(config_json);

    let actual = format_code(code, &options);
    assert_eq!(
        actual, expected,
        r"
💥 First format does not match expected!
============== input ==============
{code}
============== actual =============
{actual}
============= expected ============
{expected}
============== config =============
{config_json}
"
    );

    // Check idempotency
    let actual2 = format_code(&actual, &options);
    assert_eq!(
        actual2, expected,
        r"
💥 Formatting is not idempotent!
============== input ==============
{actual}
============== actual =============
{actual2}
============= expected ============
{expected}
============== config =============
{config_json}
"
    );
}

/// Simple test config parser for ir_transform tests.
/// Only supports the subset of options used in these tests.
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TestConfig {
    single_quote: Option<bool>,
    semi: Option<bool>,
    experimental_sort_imports: Option<TestSortImportsConfig>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TestCustomGroupDefinition {
    group_name: String,
    #[serde(default)]
    element_name_pattern: Vec<String>,
    selector: Option<String>,
    modifiers: Option<Vec<String>>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TestSortImportsConfig {
    partition_by_newline: Option<bool>,
    partition_by_comment: Option<bool>,
    sort_side_effects: Option<bool>,
    order: Option<String>,
    ignore_case: Option<bool>,
    newlines_between: Option<bool>,
    internal_pattern: Option<Vec<String>>,
    #[serde(default, deserialize_with = "deserialize_groups")]
    groups: Option<ParsedGroups>,
    custom_groups: Option<Vec<TestCustomGroupDefinition>>,
}

#[derive(Debug, Default)]
struct ParsedGroups {
    groups: Vec<Vec<String>>,
    newline_boundary_overrides: Vec<Option<bool>>,
}

fn deserialize_groups<'de, D>(deserializer: D) -> Result<Option<ParsedGroups>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde_json::Value;

    let Some(Value::Array(arr)) = Option::deserialize(deserializer)? else {
        return Ok(None);
    };

    let mut groups = Vec::new();
    let mut newline_boundary_overrides: Vec<Option<bool>> = Vec::new();
    let mut pending_override: Option<bool> = None;

    for item in arr {
        if let Value::Object(obj) = item {
            pending_override = obj.get("newlinesBetween").and_then(Value::as_bool);
        } else {
            if !groups.is_empty() {
                newline_boundary_overrides.push(pending_override.take());
            }
            let group = match item {
                Value::String(s) => vec![s],
                Value::Array(a) => {
                    a.into_iter().filter_map(|v| v.as_str().map(String::from)).collect()
                }
                _ => continue,
            };
            groups.push(group);
        }
    }

    Ok(Some(ParsedGroups { groups, newline_boundary_overrides }))
}

fn parse_test_config(json: &str) -> FormatOptions {
    let config: TestConfig = serde_json::from_str(json).expect("Invalid test config JSON");
    let mut options = FormatOptions::default();

    if let Some(single_quote) = config.single_quote {
        options.quote_style = if single_quote { QuoteStyle::Single } else { QuoteStyle::Double };
    }
    if let Some(semi) = config.semi {
        options.semicolons = if semi { Semicolons::Always } else { Semicolons::AsNeeded };
    }
    if let Some(sort_config) = config.experimental_sort_imports {
        let mut sort_imports = SortImportsOptions::default();
        if let Some(v) = sort_config.partition_by_newline {
            sort_imports.partition_by_newline = v;
        }
        if let Some(v) = sort_config.partition_by_comment {
            sort_imports.partition_by_comment = v;
        }
        if let Some(v) = sort_config.sort_side_effects {
            sort_imports.sort_side_effects = v;
        }
        if let Some(v) = sort_config.order {
            sort_imports.order = match v.as_str() {
                "desc" => SortOrder::Desc,
                _ => SortOrder::Asc,
            };
        }
        if let Some(v) = sort_config.ignore_case {
            sort_imports.ignore_case = v;
        }
        if let Some(v) = sort_config.newlines_between {
            sort_imports.newlines_between = v;
        }
        if let Some(v) = sort_config.internal_pattern {
            sort_imports.internal_pattern = v;
        }
        if let Some(v) = sort_config.groups {
            sort_imports.groups = v.groups;
            sort_imports.newline_boundary_overrides = v.newline_boundary_overrides;
        }
        if let Some(v) = sort_config.custom_groups {
            sort_imports.custom_groups = v
                .into_iter()
                .map(|value| CustomGroupDefinition {
                    group_name: value.group_name,
                    element_name_pattern: value.element_name_pattern,
                    selector: value.selector.as_deref().and_then(ImportSelector::parse),
                    modifiers: value
                        .modifiers
                        .unwrap_or_default()
                        .iter()
                        .filter_map(|s| ImportModifier::parse(s))
                        .collect(),
                })
                .collect();
        }
        options.experimental_sort_imports = Some(sort_imports);
    }

    options
}

fn format_code(code: &str, options: &FormatOptions) -> String {
    use oxc_allocator::Allocator;
    use oxc_formatter::{Formatter, get_parse_options};
    use oxc_parser::Parser;
    use oxc_span::SourceType;

    let allocator = Allocator::new();
    let source_type = SourceType::from_path("dummy.tsx").unwrap();

    let ret = Parser::new(&allocator, code, source_type).with_options(get_parse_options()).parse();

    if let Some(error) = ret.errors.first() {
        panic!("💥 Parser error: {}", error.message);
    }

    Formatter::new(&allocator, options.clone()).build(&ret.program)
}
