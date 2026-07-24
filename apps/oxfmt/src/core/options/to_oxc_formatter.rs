use rustc_hash::FxHashSet;

use oxc_formatter::{
    ArrayExpand, ArrayLinePattern, ArrowParentheses, AttributePosition, BracketSameLine,
    BracketSpacing, CustomGroupDefinition, EmbeddedLanguageFormatting, Expand, GroupEntry,
    ImportModifier, ImportSelector, JsFormatOptions, QuoteProperties, QuoteStyle, Semicolons,
    SortImportsOptions, SortOrder, SortTailwindcssOptions, TrailingCommas,
};

use super::{
    super::oxfmtrc::{
        ArrayWrapConfig, ArrayWrapMode, ArrowParensConfig, CustomGroupItemConfig,
        EmbeddedLanguageFormattingConfig, FormatConfig, HtmlWhitespaceSensitivityConfig,
        JsdocUserConfig, ObjectWrapConfig, QuotePropsConfig, SortGroupItemConfig,
        SortImportsUserConfig, SortOrderConfig, SortTailwindcssUserConfig, TrailingCommaConfig,
    },
    to_core_options::to_core_options,
};

/// Convert `FormatConfig` into validated `JsFormatOptions` for `oxc_formatter`.
///
/// # Errors
/// Returns error if any option value is invalid.
pub fn to_oxc_formatter(config: &FormatConfig) -> Result<JsFormatOptions, String> {
    let core = to_core_options(config)?;

    let mut format_options = JsFormatOptions {
        indent_style: core.indent_style,
        indent_width: core.indent_width,
        line_width: core.line_width,
        line_ending: core.line_ending,
        ..JsFormatOptions::default()
    };

    // NOTE: Not yet supported options:
    // [Prettier] experimentalOperatorPosition: "start" | "end"
    // [Prettier] experimentalTernaries: boolean
    // These are rejected at deserialize time so they never reach here.

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

    // [Prettier] htmlWhitespaceSensitivity: "css" | "strict" | "ignore"
    if let Some(sensitivity) = config.html_whitespace_sensitivity {
        format_options.html_whitespace_sensitivity_ignore =
            matches!(sensitivity, HtmlWhitespaceSensitivityConfig::Ignore);
    }

    // [Prettier] embeddedLanguageFormatting: "auto" | "off"
    if let Some(embedded_language_formatting) = config.embedded_language_formatting {
        format_options.embedded_language_formatting = match embedded_language_formatting {
            EmbeddedLanguageFormattingConfig::Auto => EmbeddedLanguageFormatting::Auto,
            EmbeddedLanguageFormattingConfig::Off => EmbeddedLanguageFormatting::Off,
        };
    }

    // Below are our own extensions

    if let Some((array_expand, array_line_pattern)) = to_array_wrap(config)? {
        format_options.array_expand = array_expand;
        format_options.array_line_pattern = array_line_pattern;
    }

    format_options.sort_imports = to_sort_imports(config)?;

    if let Some(tw_config) =
        config.sort_tailwindcss.clone().and_then(SortTailwindcssUserConfig::into_config)
    {
        // `config` / `stylesheet` / `preserve_duplicates` are JS-sorter-only
        // and travel through `to_prettier::inject_tailwind_plugin_payload`,
        // not the Rust formatter options.
        format_options.sort_tailwindcss = Some(SortTailwindcssOptions {
            functions: tw_config.functions.unwrap_or_default(),
            attributes: tw_config.attributes.unwrap_or_default(),
            preserve_whitespace: tw_config.preserve_whitespace.unwrap_or(false),
        });
    }

    format_options.jsdoc = to_jsdoc(config)?;

    Ok(format_options)
}

/// Parse and validate `arrayWrap` into [`ArrayExpand`] and an optional [`ArrayLinePattern`].
///
/// Parsing is the validation here, so this is shared by
/// both [`to_oxc_formatter()`] (build) and [`super::validate::validate()`] (gate).
///
/// # Errors
/// Returns an error if the `arrayWrap` configuration is invalid.
pub(super) fn to_array_wrap(
    config: &FormatConfig,
) -> Result<Option<(ArrayExpand, Option<ArrayLinePattern>)>, String> {
    let Some(array_wrap) = &config.array_wrap else {
        return Ok(None);
    };

    Ok(Some(match array_wrap {
        ArrayWrapConfig::Mode(ArrayWrapMode::Preserve) => (ArrayExpand::Preserve, None),
        ArrayWrapConfig::Mode(ArrayWrapMode::Collapse) => (ArrayExpand::Never, None),
        ArrayWrapConfig::Options(options) => {
            if options.min_elements_to_wrap.is_none() && options.line_pattern.is_none() {
                return Err(
                    "Invalid `arrayWrap` value.\nExpected at least one of `minElementsToWrap` or `linePattern`.".to_string(),
                );
            }

            let line_pattern = options
                .line_pattern
                .as_ref()
                .map(|line_pattern| {
                    line_pattern
                        .parse()
                        .map_err(|err| format!("Invalid `arrayWrap.linePattern` value.\n{err}"))
                })
                .transpose()?;

            // A pattern without a threshold applies to arrays kept expanded by preserve
            let array_expand = options
                .min_elements_to_wrap
                .map_or(ArrayExpand::Preserve, ArrayExpand::ForceAboveThreshold);

            (array_expand, line_pattern)
        }
    }))
}

/// Parse and validate `sortImports` into [`SortImportsOptions`].
///
/// Parsing is the validation here, so this is shared by
/// both [`to_oxc_formatter()`] (build) and [`super::validate::validate()`] (gate).
///
/// # Errors
/// Returns an error if the `sortImports` configuration is invalid.
pub(super) fn to_sort_imports(config: &FormatConfig) -> Result<Option<SortImportsOptions>, String> {
    let Some(sort_imports_config) =
        config.sort_imports.clone().and_then(SortImportsUserConfig::into_config)
    else {
        return Ok(None);
    };

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

    Ok(Some(sort_imports))
}

/// Parse and validate `jsdoc` into [`oxc_formatter::JsdocOptions`].
///
/// Shared by both [`to_oxc_formatter()`] (build) and [`super::validate::validate()`] (gate).
///
/// # Errors
/// Returns an error if `lineWrappingStyle` / `commentLineStrategy` is invalid.
pub(super) fn to_jsdoc(
    config: &FormatConfig,
) -> Result<Option<oxc_formatter::JsdocOptions>, String> {
    let Some(jsdoc_config) = config.jsdoc.clone().and_then(JsdocUserConfig::into_config) else {
        return Ok(None);
    };

    let mut opts = oxc_formatter::JsdocOptions::default();
    if let Some(v) = jsdoc_config.capitalize_descriptions {
        opts.capitalize_descriptions = v;
    }
    if let Some(v) = jsdoc_config.description_with_dot {
        opts.description_with_dot = v;
    }
    if let Some(v) = jsdoc_config.add_default_to_description {
        opts.add_default_to_description = v;
    }
    if let Some(v) = jsdoc_config.prefer_code_fences {
        opts.prefer_code_fences = v;
    }
    if let Some(ref v) = jsdoc_config.line_wrapping_style {
        opts.line_wrapping_style = match v.as_str() {
            "greedy" => oxc_formatter::LineWrappingStyle::Greedy,
            "balance" => oxc_formatter::LineWrappingStyle::Balance,
            other => {
                return Err(format!(
                    "Invalid jsdoc lineWrappingStyle: {other:?}. Expected \"greedy\" or \"balance\"."
                ));
            }
        };
    }
    if let Some(ref v) = jsdoc_config.comment_line_strategy {
        opts.comment_line_strategy = match v.as_str() {
            "singleLine" => oxc_formatter::CommentLineStrategy::SingleLine,
            "multiline" => oxc_formatter::CommentLineStrategy::Multiline,
            "keep" => oxc_formatter::CommentLineStrategy::Keep,
            other => {
                return Err(format!(
                    "Invalid jsdoc commentLineStrategy: {other:?}. Expected \"singleLine\", \"multiline\", or \"keep\"."
                ));
            }
        };
    }
    if let Some(v) = jsdoc_config.separate_tag_groups {
        opts.separate_tag_groups = v;
    }
    if let Some(v) = jsdoc_config.separate_returns_from_param {
        opts.separate_returns_from_param = v;
    }
    if let Some(v) = jsdoc_config.bracket_spacing {
        opts.bracket_spacing = v;
    }
    if let Some(v) = jsdoc_config.description_tag {
        opts.description_tag = v;
    }
    if let Some(v) = jsdoc_config.keep_unparsable_example_indent {
        opts.keep_unparsable_example_indent = v;
    }

    Ok(Some(opts))
}

// ---

#[cfg(test)]
mod tests {
    use oxc_formatter::{Expand, GroupName};

    use super::super::validate::validate;
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
        let format_options = to_oxc_formatter(&config).unwrap();

        assert!(format_options.indent_style.is_tab());
        assert_eq!(format_options.indent_width.value(), 4);
        assert_eq!(format_options.line_width.value(), 100);
        assert!(!format_options.quote_style.is_double());
        assert!(format_options.semicolons.is_as_needed());

        let sort_imports = format_options.sort_imports.unwrap();
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
        let format_options = to_oxc_formatter(&config).unwrap();

        // Should use defaults
        assert!(format_options.indent_style.is_space());
        assert_eq!(format_options.indent_width.value(), 2);
        assert_eq!(format_options.line_width.value(), 100);
        assert_eq!(format_options.sort_imports, None);
    }

    #[test]
    fn test_empty_config() {
        let config: FormatConfig = serde_json::from_str("{}").unwrap();
        let format_options = to_oxc_formatter(&config).unwrap();

        // Should use defaults
        assert!(format_options.indent_style.is_space());
        assert_eq!(format_options.indent_width.value(), 2);
        assert_eq!(format_options.line_width.value(), 100);
        assert_eq!(format_options.sort_imports, None);
    }

    #[test]
    fn test_arrow_parens_normalization() {
        // Test "avoid" -> "as-needed" normalization
        let config: FormatConfig = serde_json::from_str(r#"{"arrowParens": "avoid"}"#).unwrap();
        let format_options = to_oxc_formatter(&config).unwrap();
        assert!(format_options.arrow_parentheses.is_as_needed());

        // Test "always" remains unchanged
        let config: FormatConfig = serde_json::from_str(r#"{"arrowParens": "always"}"#).unwrap();
        let format_options = to_oxc_formatter(&config).unwrap();
        assert!(format_options.arrow_parentheses.is_always());
    }

    #[test]
    fn test_object_wrap_normalization() {
        // Test "preserve" -> "auto" normalization
        let config: FormatConfig = serde_json::from_str(r#"{"objectWrap": "preserve"}"#).unwrap();
        let format_options = to_oxc_formatter(&config).unwrap();
        assert_eq!(format_options.expand, Expand::Auto);

        // Test "collapse" -> "never" normalization
        let config: FormatConfig = serde_json::from_str(r#"{"objectWrap": "collapse"}"#).unwrap();
        let format_options = to_oxc_formatter(&config).unwrap();
        assert_eq!(format_options.expand, Expand::Never);
    }

    #[test]
    fn test_array_wrap_normalization() {
        // Test default (no option) -> Auto (Prettier behavior)
        let config: FormatConfig = serde_json::from_str("{}").unwrap();
        let format_options = to_oxc_formatter(&config).unwrap();
        assert_eq!(format_options.array_expand, ArrayExpand::Auto);

        // Test "preserve" -> Preserve
        let config: FormatConfig = serde_json::from_str(r#"{"arrayWrap": "preserve"}"#).unwrap();
        let format_options = to_oxc_formatter(&config).unwrap();
        assert_eq!(format_options.array_expand, ArrayExpand::Preserve);

        // Test "collapse" -> Never
        let config: FormatConfig = serde_json::from_str(r#"{"arrayWrap": "collapse"}"#).unwrap();
        let format_options = to_oxc_formatter(&config).unwrap();
        assert_eq!(format_options.array_expand, ArrayExpand::Never);

        // Test { minElementsToWrap: 2 } -> ForceAboveThreshold(2)
        let config: FormatConfig =
            serde_json::from_str(r#"{"arrayWrap": {"minElementsToWrap": 2}}"#).unwrap();
        let format_options = to_oxc_formatter(&config).unwrap();
        assert_eq!(format_options.array_expand, ArrayExpand::ForceAboveThreshold(2));
        assert_eq!(format_options.array_line_pattern, None);

        // Test { minElementsToWrap: 2, linePattern: "2 1" } -> ForceAboveThreshold(2) + pattern
        let config: FormatConfig = serde_json::from_str(
            r#"{"arrayWrap": {"minElementsToWrap": 2, "linePattern": "2 1"}}"#,
        )
        .unwrap();
        let format_options = to_oxc_formatter(&config).unwrap();
        assert_eq!(format_options.array_expand, ArrayExpand::ForceAboveThreshold(2));
        assert_eq!(format_options.array_line_pattern, Some("2 1".parse().unwrap()));

        // Test { linePattern: "3" } alone -> Preserve + pattern
        let config: FormatConfig =
            serde_json::from_str(r#"{"arrayWrap": {"linePattern": "3"}}"#).unwrap();
        let format_options = to_oxc_formatter(&config).unwrap();
        assert_eq!(format_options.array_expand, ArrayExpand::Preserve);
        assert_eq!(format_options.array_line_pattern, Some("3".parse().unwrap()));

        // Test {} -> error
        let config: FormatConfig = serde_json::from_str(r#"{"arrayWrap": {}}"#).unwrap();
        let err = to_oxc_formatter(&config).unwrap_err();
        assert!(err.contains("minElementsToWrap"), "unexpected error: {err}");

        // Test invalid linePattern -> error
        let config: FormatConfig =
            serde_json::from_str(r#"{"arrayWrap": {"linePattern": "2 x"}}"#).unwrap();
        let err = to_oxc_formatter(&config).unwrap_err();
        assert!(err.contains("linePattern"), "unexpected error: {err}");

        // Test zero in linePattern -> error
        let config: FormatConfig =
            serde_json::from_str(r#"{"arrayWrap": {"linePattern": "0"}}"#).unwrap();
        let err = to_oxc_formatter(&config).unwrap_err();
        assert!(err.contains("linePattern"), "unexpected error: {err}");
    }

    #[test]
    fn test_sort_imports_config() {
        let config: FormatConfig = serde_json::from_str(
            r#"{
            "experimentalSortImports": {}
        }"#,
        )
        .unwrap();
        let format_options = to_oxc_formatter(&config).unwrap();
        let sort_imports = format_options.sort_imports.unwrap();
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
        let format_options = to_oxc_formatter(&config).unwrap();
        let sort_imports = format_options.sort_imports.unwrap();
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
        let format_options = to_oxc_formatter(&config).unwrap();
        let sort_imports = format_options.sort_imports.unwrap();
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
        assert!(to_oxc_formatter(&config).is_ok());
        let config: FormatConfig = serde_json::from_str(
            r#"{
                "experimentalSortImports": {
                    "partitionByNewline": true,
                    "newlinesBetween": true
                }
            }"#,
        )
        .unwrap();
        assert!(to_oxc_formatter(&config).is_err_and(|e| e.contains("newlinesBetween")));

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
        let format_options = to_oxc_formatter(&config).unwrap();
        let sort_imports = format_options.sort_imports.unwrap();
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
        let format_options = to_oxc_formatter(&config).unwrap();
        let sort_imports = format_options.sort_imports.unwrap();
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
        assert!(to_oxc_formatter(&config).is_err_and(|e| e.contains("start")));

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
        assert!(to_oxc_formatter(&config).is_err_and(|e| e.contains("end")));

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
        assert!(to_oxc_formatter(&config).is_err_and(|e| e.contains("consecutive")));

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
        assert!(to_oxc_formatter(&config).is_err_and(|e| e.contains("partitionByNewline")));
    }

    #[test]
    fn test_bool_for_object_options() {
        let config: FormatConfig = serde_json::from_str(r#"{"sortImports": true}"#).unwrap();
        assert!(to_oxc_formatter(&config).unwrap().sort_imports.is_some());

        let config: FormatConfig = serde_json::from_str(r#"{"sortImports": false}"#).unwrap();
        assert!(to_oxc_formatter(&config).unwrap().sort_imports.is_none());

        let config: FormatConfig = serde_json::from_str(r#"{"sortTailwindcss": true}"#).unwrap();
        assert!(to_oxc_formatter(&config).unwrap().sort_tailwindcss.is_some());

        let config: FormatConfig = serde_json::from_str(r#"{"sortTailwindcss": false}"#).unwrap();
        assert!(to_oxc_formatter(&config).unwrap().sort_tailwindcss.is_none());

        let config: FormatConfig = serde_json::from_str(r#"{"jsdoc": true}"#).unwrap();
        assert!(to_oxc_formatter(&config).unwrap().jsdoc.is_some());

        let config: FormatConfig = serde_json::from_str(r#"{"jsdoc": false}"#).unwrap();
        assert!(to_oxc_formatter(&config).unwrap().jsdoc.is_none());
    }

    #[test]
    fn validate_matches_build_validation() {
        // Valid config: both build and validate succeed.
        let config: FormatConfig =
            serde_json::from_str(r#"{ "printWidth": 80, "sortImports": true }"#).unwrap();
        assert!(validate(&config).is_ok());
        assert!(to_oxc_formatter(&config).is_ok());

        // Core range error (valid u16, but outside `LineWidth` bounds).
        let config: FormatConfig = serde_json::from_str(r#"{ "printWidth": 1000 }"#).unwrap();
        assert!(validate(&config).is_err());
        assert!(to_oxc_formatter(&config).is_err());

        // JS-specific error (sortImports) must be caught by `validate` too,
        // not just by building `JsFormatOptions`.
        let config: FormatConfig = serde_json::from_str(
            r#"{ "experimentalSortImports": { "groups": [{ "newlinesBetween": false }, "builtin"] } }"#,
        )
        .unwrap();
        assert!(validate(&config).is_err_and(|e| e.contains("start")));
        assert!(to_oxc_formatter(&config).is_err_and(|e| e.contains("start")));

        // JS-specific error (jsdoc enum).
        let config: FormatConfig =
            serde_json::from_str(r#"{ "jsdoc": { "lineWrappingStyle": "bogus" } }"#).unwrap();
        assert!(validate(&config).is_err_and(|e| e.contains("lineWrappingStyle")));
    }
}
