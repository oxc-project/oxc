use rustc_hash::FxHashSet;

use oxc_formatter::{
    ArrowParentheses, AttributePosition, BracketSameLine, BracketSpacing, CustomGroupDefinition,
    EmbeddedLanguageFormatting, Expand, FormatOptions, GroupEntry, ImportModifier, ImportSelector,
    IndentStyle, IndentWidth, LineEnding, LineWidth, QuoteProperties, QuoteStyle, Semicolons,
    SortImportsOptions, SortOrder, SortTailwindcssOptions, TrailingCommas,
};
use oxc_toml::Options as TomlFormatterOptions;

use super::format_config::{
    ArrowParensConfig, CustomGroupItemConfig, EmbeddedLanguageFormattingConfig, EndOfLineConfig,
    FormatConfig, ObjectWrapConfig, QuotePropsConfig, SortGroupItemConfig, SortOrderConfig,
    SortPackageJsonConfig, TrailingCommaConfig,
};

/// Resolved format options from `FormatConfig`.
///
/// Contains `FormatOptions` for `oxc_formatter` plus additional Oxfmt-specific options.
/// All fields here are subject to per-file overrides.
#[derive(Debug, Clone)]
pub struct OxfmtOptions {
    pub format_options: FormatOptions,
    pub toml_options: TomlFormatterOptions,
    pub sort_package_json: Option<sort_package_json::SortOptions>,
    pub insert_final_newline: bool,
}

/// Convert `FormatConfig` into `OxfmtOptions`.
///
/// # Errors
/// Returns error if any option value is invalid
pub fn to_oxfmt_options(config: FormatConfig) -> Result<OxfmtOptions, String> {
    // Not yet supported options:
    // [Prettier] experimentalOperatorPosition: "start" | "end"
    // [Prettier] experimentalTernaries: boolean
    if config.experimental_operator_position.is_some() {
        return Err("Unsupported option: `experimentalOperatorPosition`".to_string());
    }
    if config.experimental_ternaries.is_some() {
        return Err("Unsupported option: `experimentalTernaries`".to_string());
    }

    // All values are based on defaults from `FormatOptions::default()`
    let mut format_options = FormatOptions::default();

    // [Prettier] useTabs: boolean
    if let Some(use_tabs) = config.use_tabs {
        format_options.indent_style = if use_tabs { IndentStyle::Tab } else { IndentStyle::Space };
    }

    // [Prettier] tabWidth: number
    if let Some(width) = config.tab_width {
        format_options.indent_width =
            IndentWidth::try_from(width).map_err(|e| format!("Invalid tabWidth: {e}"))?;
    }

    // [Prettier] endOfLine: "lf" | "cr" | "crlf" | "auto"
    // NOTE: "auto" is not supported
    if let Some(ending) = config.end_of_line {
        format_options.line_ending = match ending {
            EndOfLineConfig::Lf => LineEnding::Lf,
            EndOfLineConfig::Crlf => LineEnding::Crlf,
            EndOfLineConfig::Cr => LineEnding::Cr,
        };
    }

    // [Prettier] printWidth: number
    if let Some(width) = config.print_width {
        format_options.line_width =
            LineWidth::try_from(width).map_err(|e| format!("Invalid printWidth: {e}"))?;
    }

    // [Prettier] singleQuote: boolean
    if let Some(single_quote) = config.single_quote {
        format_options.quote_style =
            if single_quote { QuoteStyle::Single } else { QuoteStyle::Double };
    }

    // [Prettier] jsxSingleQuote: boolean
    if let Some(jsx_single_quote) = config.jsx_single_quote {
        format_options.jsx_quote_style =
            if jsx_single_quote { QuoteStyle::Single } else { QuoteStyle::Double };
    }

    // [Prettier] quoteProps: "as-needed" | "consistent" | "preserve"
    if let Some(props) = config.quote_props {
        format_options.quote_properties = match props {
            QuotePropsConfig::AsNeeded => QuoteProperties::AsNeeded,
            QuotePropsConfig::Consistent => QuoteProperties::Consistent,
            QuotePropsConfig::Preserve => QuoteProperties::Preserve,
        };
    }

    // [Prettier] trailingComma: "all" | "es5" | "none"
    if let Some(commas) = config.trailing_comma {
        format_options.trailing_commas = match commas {
            TrailingCommaConfig::All => TrailingCommas::All,
            TrailingCommaConfig::Es5 => TrailingCommas::Es5,
            TrailingCommaConfig::None => TrailingCommas::None,
        };
    }

    // [Prettier] semi: boolean
    if let Some(semi) = config.semi {
        format_options.semicolons = if semi { Semicolons::Always } else { Semicolons::AsNeeded };
    }

    // [Prettier] arrowParens: "avoid" | "always"
    if let Some(parens) = config.arrow_parens {
        format_options.arrow_parentheses = match parens {
            ArrowParensConfig::Avoid => ArrowParentheses::AsNeeded,
            ArrowParensConfig::Always => ArrowParentheses::Always,
        };
    }

    // [Prettier] bracketSpacing: boolean
    if let Some(spacing) = config.bracket_spacing {
        format_options.bracket_spacing = BracketSpacing::from(spacing);
    }

    // [Prettier] bracketSameLine: boolean
    if let Some(same_line) = config.bracket_same_line {
        format_options.bracket_same_line = BracketSameLine::from(same_line);
    }

    // [Prettier] singleAttributePerLine: boolean
    if let Some(single_attribute_per_line) = config.single_attribute_per_line {
        format_options.attribute_position = if single_attribute_per_line {
            AttributePosition::Multiline
        } else {
            AttributePosition::Auto
        };
    }

    // [Prettier] objectWrap: "preserve" | "collapse"
    if let Some(object_wrap) = config.object_wrap {
        format_options.expand = match object_wrap {
            ObjectWrapConfig::Preserve => Expand::Auto,
            ObjectWrapConfig::Collapse => Expand::Never,
        };
    }

    // [Prettier] embeddedLanguageFormatting: "auto" | "off"
    if let Some(embedded_language_formatting) = config.embedded_language_formatting {
        format_options.embedded_language_formatting = match embedded_language_formatting {
            EmbeddedLanguageFormattingConfig::Auto => EmbeddedLanguageFormatting::Auto,
            EmbeddedLanguageFormattingConfig::Off => EmbeddedLanguageFormatting::Off,
        };
    }

    // Below are our own extensions

    if let Some(sort_imports_config) = config.sort_imports {
        let mut sort_imports = SortImportsOptions::default();

        if let Some(v) = sort_imports_config.partition_by_newline {
            sort_imports.partition_by_newline = v;
        }
        if let Some(v) = sort_imports_config.partition_by_comment {
            sort_imports.partition_by_comment = v;
        }
        if let Some(v) = sort_imports_config.sort_side_effects {
            sort_imports.sort_side_effects = v;
        }
        if let Some(v) = sort_imports_config.order {
            sort_imports.order = match v {
                SortOrderConfig::Asc => SortOrder::Asc,
                SortOrderConfig::Desc => SortOrder::Desc,
            };
        }
        if let Some(v) = sort_imports_config.ignore_case {
            sort_imports.ignore_case = v;
        }
        if let Some(v) = sort_imports_config.newlines_between {
            sort_imports.newlines_between = v;
        }
        if let Some(v) = sort_imports_config.internal_pattern {
            sort_imports.internal_pattern = v;
        }
        // Validate and parse `customGroups` first, since `groups` may refer to custom group names.
        if let Some(v) = sort_imports_config.custom_groups {
            let mut custom_groups = Vec::with_capacity(v.len());
            for cg in v {
                let CustomGroupItemConfig { group_name, element_name_pattern, .. } = cg;
                let selector = match cg.selector.as_deref() {
                    Some(s) => match ImportSelector::parse(s) {
                        Some(parsed) => Some(parsed),
                        None => {
                            return Err(format!(
                                "Invalid `sortImports` configuration: unknown selector: `{s}` in customGroups: `{group_name}`"
                            ));
                        }
                    },
                    None => None,
                };
                let raw_modifiers = cg.modifiers.unwrap_or_default();
                let mut modifiers = Vec::with_capacity(raw_modifiers.len());
                for m in &raw_modifiers {
                    match ImportModifier::parse(m) {
                        Some(parsed) => modifiers.push(parsed),
                        None => {
                            return Err(format!(
                                "Invalid `sortImports` configuration: unknown modifier: `{m}` in customGroups: `{group_name}`"
                            ));
                        }
                    }
                }
                custom_groups.push(CustomGroupDefinition {
                    group_name,
                    element_name_pattern,
                    selector,
                    modifiers,
                });
            }
            sort_imports.custom_groups = custom_groups;
        }
        if let Some(v) = sort_imports_config.groups {
            let custom_group_names: FxHashSet<&str> =
                sort_imports.custom_groups.iter().map(|g| g.group_name.as_str()).collect();
            let mut groups = Vec::new();
            let mut newline_boundary_overrides: Vec<Option<bool>> = Vec::new();
            let mut pending_override: Option<bool> = None;

            for item in v {
                match item {
                    SortGroupItemConfig::NewlinesBetween(marker) => {
                        if groups.is_empty() {
                            return Err("Invalid `sortImports` configuration: `{ \"newlinesBetween\" }` marker cannot appear at the start of `groups`".to_string());
                        }
                        if pending_override.is_some() {
                            return Err("Invalid `sortImports` configuration: consecutive `{ \"newlinesBetween\" }` markers are not allowed in `groups`".to_string());
                        }
                        pending_override = Some(marker.newlines_between);
                    }
                    other => {
                        if !groups.is_empty() {
                            newline_boundary_overrides.push(pending_override.take());
                        }
                        let mut entries = Vec::new();
                        for name in other.into_vec() {
                            let entry = GroupEntry::parse(&name);
                            if let GroupEntry::Custom(ref n) = entry
                                && !custom_group_names.contains(n.as_str())
                            {
                                return Err(format!(
                                    "Invalid `sortImports` configuration: unknown group name `{name}` in `groups`"
                                ));
                            }
                            entries.push(entry);
                        }
                        groups.push(entries);
                    }
                }
            }

            if pending_override.is_some() {
                return Err("Invalid `sortImports` configuration: `{ \"newlinesBetween\" }` marker cannot appear at the end of `groups`".to_string());
            }

            sort_imports.groups = groups;
            sort_imports.newline_boundary_overrides = newline_boundary_overrides;
        }

        sort_imports.validate().map_err(|e| format!("Invalid `sortImports` configuration: {e}"))?;

        format_options.sort_imports = Some(sort_imports);
    }

    if let Some(tw_config) = config.sort_tailwindcss {
        format_options.sort_tailwindcss = Some(SortTailwindcssOptions {
            config: tw_config.config,
            stylesheet: tw_config.stylesheet,
            functions: tw_config.functions.unwrap_or_default(),
            attributes: tw_config.attributes.unwrap_or_default(),
            preserve_whitespace: tw_config.preserve_whitespace.unwrap_or(false),
            preserve_duplicates: tw_config.preserve_duplicates.unwrap_or(false),
        });
    }

    // Currently, there is a no options for TOML formatter
    let toml_options = build_toml_options(&format_options);

    let sort_package_json = config.sort_package_json.map_or_else(
        || Some(SortPackageJsonConfig::default().to_sort_options()),
        |c| c.to_sort_options(),
    );

    let insert_final_newline = config.insert_final_newline.unwrap_or(true);

    Ok(OxfmtOptions { format_options, toml_options, sort_package_json, insert_final_newline })
}

// ---

/// Build `toml` formatter options from `FormatOptions`.
/// Use the same options as `prettier-plugin-toml`.
/// <https://github.com/un-ts/prettier/blob/7a4346d5dbf6b63987c0f81228fc46bb12f8692f/packages/toml/src/index.ts#L27-L31>
fn build_toml_options(format_options: &FormatOptions) -> TomlFormatterOptions {
    TomlFormatterOptions {
        column_width: format_options.line_width.value() as usize,
        indent_string: if format_options.indent_style.is_tab() {
            "\t".to_string()
        } else {
            " ".repeat(format_options.indent_width.value() as usize)
        },
        array_trailing_comma: !format_options.trailing_commas.is_none(),
        crlf: format_options.line_ending.is_carriage_return_line_feed(),
        // NOTE: Need to align with `oxc_formatter` and Prettier defaults,
        // to make `insertFinalNewline` option work correctly.
        trailing_newline: true,
        ..Default::default()
    }
}

// ---

#[cfg(test)]
mod tests {
    use oxc_formatter::{Expand, GroupName};

    use super::*;

    #[test]
    fn test_config_parsing() {
        let json = r#"{
            "useTabs": true,
            "tabWidth": 4,
            "printWidth": 100,
            "singleQuote": true,
            "semi": false,
            "experimentalSortImports": {
                "partitionByNewline": true,
                "order": "desc",
                "ignoreCase": false,
                "newlinesBetween": false
            }
        }"#;

        let config: FormatConfig = serde_json::from_str(json).unwrap();
        let oxfmt_options = to_oxfmt_options(config).unwrap();

        assert!(oxfmt_options.format_options.indent_style.is_tab());
        assert_eq!(oxfmt_options.format_options.indent_width.value(), 4);
        assert_eq!(oxfmt_options.format_options.line_width.value(), 100);
        assert!(!oxfmt_options.format_options.quote_style.is_double());
        assert!(oxfmt_options.format_options.semicolons.is_as_needed());

        let sort_imports = oxfmt_options.format_options.sort_imports.unwrap();
        assert!(sort_imports.partition_by_newline);
        assert!(sort_imports.order.is_desc());
        assert!(!sort_imports.ignore_case);
        assert!(!sort_imports.newlines_between);
    }

    #[test]
    fn test_ignore_unknown_fields() {
        let config: FormatConfig = serde_json::from_str(
            r#"{
                "unknownField": "someValue",
                "anotherUnknown": 123
            }"#,
        )
        .unwrap();
        let oxfmt_options = to_oxfmt_options(config).unwrap();

        // Should use defaults
        assert!(oxfmt_options.format_options.indent_style.is_space());
        assert_eq!(oxfmt_options.format_options.indent_width.value(), 2);
        assert_eq!(oxfmt_options.format_options.line_width.value(), 100);
        assert_eq!(oxfmt_options.format_options.sort_imports, None);
    }

    #[test]
    fn test_empty_config() {
        let config: FormatConfig = serde_json::from_str("{}").unwrap();
        let oxfmt_options = to_oxfmt_options(config).unwrap();

        // Should use defaults
        assert!(oxfmt_options.format_options.indent_style.is_space());
        assert_eq!(oxfmt_options.format_options.indent_width.value(), 2);
        assert_eq!(oxfmt_options.format_options.line_width.value(), 100);
        assert_eq!(oxfmt_options.format_options.sort_imports, None);
    }

    #[test]
    fn test_arrow_parens_normalization() {
        // Test "avoid" -> "as-needed" normalization
        let config: FormatConfig = serde_json::from_str(r#"{"arrowParens": "avoid"}"#).unwrap();
        let oxfmt_options = to_oxfmt_options(config).unwrap();
        assert!(oxfmt_options.format_options.arrow_parentheses.is_as_needed());

        // Test "always" remains unchanged
        let config: FormatConfig = serde_json::from_str(r#"{"arrowParens": "always"}"#).unwrap();
        let oxfmt_options = to_oxfmt_options(config).unwrap();
        assert!(oxfmt_options.format_options.arrow_parentheses.is_always());
    }

    #[test]
    fn test_object_wrap_normalization() {
        // Test "preserve" -> "auto" normalization
        let config: FormatConfig = serde_json::from_str(r#"{"objectWrap": "preserve"}"#).unwrap();
        let oxfmt_options = to_oxfmt_options(config).unwrap();
        assert_eq!(oxfmt_options.format_options.expand, Expand::Auto);

        // Test "collapse" -> "never" normalization
        let config: FormatConfig = serde_json::from_str(r#"{"objectWrap": "collapse"}"#).unwrap();
        let oxfmt_options = to_oxfmt_options(config).unwrap();
        assert_eq!(oxfmt_options.format_options.expand, Expand::Never);
    }

    #[test]
    fn test_sort_imports_config() {
        let config: FormatConfig = serde_json::from_str(
            r#"{
            "experimentalSortImports": {}
        }"#,
        )
        .unwrap();
        let oxfmt_options = to_oxfmt_options(config).unwrap();
        let sort_imports = oxfmt_options.format_options.sort_imports.unwrap();
        assert!(sort_imports.newlines_between);
        assert!(!sort_imports.partition_by_newline);

        // Test explicit false
        let config: FormatConfig = serde_json::from_str(
            r#"{
                "experimentalSortImports": {
                    "newlinesBetween": false
                }
            }"#,
        )
        .unwrap();
        let oxfmt_options = to_oxfmt_options(config).unwrap();
        let sort_imports = oxfmt_options.format_options.sort_imports.unwrap();
        assert!(!sort_imports.newlines_between);
        assert!(!sort_imports.partition_by_newline);

        // Test explicit true
        let config: FormatConfig = serde_json::from_str(
            r#"{
                "experimentalSortImports": {
                    "newlinesBetween": true
                }
            }"#,
        )
        .unwrap();
        let oxfmt_options = to_oxfmt_options(config).unwrap();
        let sort_imports = oxfmt_options.format_options.sort_imports.unwrap();
        assert!(sort_imports.newlines_between);
        assert!(!sort_imports.partition_by_newline);

        let config: FormatConfig = serde_json::from_str(
            r#"{
                "experimentalSortImports": {
                    "partitionByNewline": true,
                    "newlinesBetween": false
                }
            }"#,
        )
        .unwrap();
        assert!(to_oxfmt_options(config).is_ok());
        let config: FormatConfig = serde_json::from_str(
            r#"{
                "experimentalSortImports": {
                    "partitionByNewline": true,
                    "newlinesBetween": true
                }
            }"#,
        )
        .unwrap();
        assert!(to_oxfmt_options(config).is_err_and(|e| e.contains("newlinesBetween")));

        let config: FormatConfig = serde_json::from_str(
            r#"{
                "experimentalSortImports": {
                    "groups": [
                        "builtin",
                        ["external", "internal"],
                        "parent",
                        "sibling",
                        "index"
                    ]
                }
            }"#,
        )
        .unwrap();
        let oxfmt_options = to_oxfmt_options(config).unwrap();
        let sort_imports = oxfmt_options.format_options.sort_imports.unwrap();
        assert_eq!(sort_imports.groups.len(), 5);
        assert_eq!(
            sort_imports.groups[0],
            vec![GroupEntry::Predefined(GroupName::parse("builtin").unwrap())]
        );
        assert_eq!(
            sort_imports.groups[1],
            vec![
                GroupEntry::Predefined(GroupName::parse("external").unwrap()),
                GroupEntry::Predefined(GroupName::parse("internal").unwrap())
            ]
        );
        assert_eq!(
            sort_imports.groups[4],
            vec![GroupEntry::Predefined(GroupName::parse("index").unwrap())]
        );

        // Test groups with newlinesBetween overrides
        let config: FormatConfig = serde_json::from_str(
            r#"{
                "experimentalSortImports": {
                    "groups": [
                        "builtin",
                        { "newlinesBetween": false },
                        "external",
                        "parent"
                    ]
                }
            }"#,
        )
        .unwrap();
        let oxfmt_options = to_oxfmt_options(config).unwrap();
        let sort_imports = oxfmt_options.format_options.sort_imports.unwrap();
        assert_eq!(sort_imports.groups.len(), 3);
        assert_eq!(
            sort_imports.groups[0],
            vec![GroupEntry::Predefined(GroupName::parse("builtin").unwrap())]
        );
        assert_eq!(
            sort_imports.groups[1],
            vec![GroupEntry::Predefined(GroupName::parse("external").unwrap())]
        );
        assert_eq!(
            sort_imports.groups[2],
            vec![GroupEntry::Predefined(GroupName::parse("parent").unwrap())]
        );
        assert_eq!(sort_imports.newline_boundary_overrides.len(), 2);
        assert_eq!(sort_imports.newline_boundary_overrides[0], Some(false));
        assert_eq!(sort_imports.newline_boundary_overrides[1], None);

        // Test error: newlinesBetween at start of groups
        let config: FormatConfig = serde_json::from_str(
            r#"{
                "experimentalSortImports": {
                    "groups": [
                        { "newlinesBetween": false },
                        "builtin",
                        "external"
                    ]
                }
            }"#,
        )
        .unwrap();
        assert!(to_oxfmt_options(config).is_err_and(|e| e.contains("start")));

        // Test error: newlinesBetween at end of groups
        let config: FormatConfig = serde_json::from_str(
            r#"{
                "experimentalSortImports": {
                    "groups": [
                        "builtin",
                        "external",
                        { "newlinesBetween": true }
                    ]
                }
            }"#,
        )
        .unwrap();
        assert!(to_oxfmt_options(config).is_err_and(|e| e.contains("end")));

        // Test error: consecutive newlinesBetween markers
        let config: FormatConfig = serde_json::from_str(
            r#"{
                "experimentalSortImports": {
                    "groups": [
                        "builtin",
                        { "newlinesBetween": false },
                        { "newlinesBetween": true },
                        "external"
                    ]
                }
            }"#,
        )
        .unwrap();
        assert!(to_oxfmt_options(config).is_err_and(|e| e.contains("consecutive")));

        // Test error: partitionByNewline with per-group newlinesBetween markers
        let config: FormatConfig = serde_json::from_str(
            r#"{
                "experimentalSortImports": {
                    "partitionByNewline": true,
                    "groups": [
                        "builtin",
                        { "newlinesBetween": false },
                        "external"
                    ]
                }
            }"#,
        )
        .unwrap();
        assert!(to_oxfmt_options(config).is_err_and(|e| e.contains("partitionByNewline")));
    }
}
