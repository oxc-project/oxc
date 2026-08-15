#[cfg(feature = "napi")]
use oxc_formatter::SortTailwindcssOptions;
use oxc_formatter::{
    ArrowParentheses, AttributePosition, BracketSameLine, BracketSpacing, CommentLineStrategy,
    CustomGroupDefinition, Expand, GroupEntry, ImportModifier, ImportSelector, JsFormatOptions,
    JsdocOptions, LineWrappingStyle, OperatorPosition, QuoteProperties, QuoteStyle, Semicolons,
    SortImportsOptions, SortOrder, TrailingCommas,
};
use oxc_formatter_core::{CoreFormatOptions, FormatOptions};

#[cfg(feature = "napi")]
use super::super::oxfmtrc::SortTailwindcssUserConfig;
use super::super::oxfmtrc::{
    ArrowParensConfig, CommentLineStrategyConfig, FormatConfig, HtmlWhitespaceSensitivityConfig,
    ImportModifierConfig, ImportSelectorConfig, JsdocUserConfig, LineWrappingStyleConfig,
    ObjectWrapConfig, OperatorPositionConfig, QuotePropsConfig, SortGroupItemConfig,
    SortImportsUserConfig, SortOrderConfig, TrailingCommaConfig,
};

/// Convert `FormatConfig` into `JsFormatOptions` for `oxc_formatter`.
///
/// NOTE: Pure field translation:
/// `core` and `sort_imports` are the validation gate's artifacts ([`super::validate::validate()`]), so this cannot fail.
pub fn to_oxc_formatter(
    config: &FormatConfig,
    core_options: CoreFormatOptions,
    sort_imports: Option<SortImportsOptions>,
) -> JsFormatOptions {
    let mut format_options = JsFormatOptions::default();
    format_options.apply_core(core_options);

    // NOTE: [Prettier] experimentalTernaries is not yet supported;
    // rejected at deserialize time (`oxfmtrc::reject_experimental_ternaries`) so it never reaches here.

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

    // [Prettier] experimentalOperatorPosition: "start" | "end"
    if let Some(position) = config.experimental_operator_position {
        format_options.operator_position = match position {
            OperatorPositionConfig::Start => OperatorPosition::Start,
            OperatorPositionConfig::End => OperatorPosition::End,
        };
    }

    // [Prettier] htmlWhitespaceSensitivity: "css" | "strict" | "ignore"
    if let Some(sensitivity) = config.html_whitespace_sensitivity {
        format_options.html_whitespace_sensitivity_ignore =
            matches!(sensitivity, HtmlWhitespaceSensitivityConfig::Ignore);
    }

    // Below are our own extensions

    format_options.sort_imports = sort_imports;
    format_options.jsdoc = to_jsdoc(config);
    // napi only, like the CSS mapper: collection itself normalizes whitespace,
    // so enabling it without the JS-side sorter would apply half the feature
    // (normalized but unsorted classes).
    #[cfg(feature = "napi")]
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

    format_options
}

/// Derive [`SortImportsOptions`] from the resolved config;
/// the gate ([`super::validate::validate()`]) runs it once, like `to_core_options`.
///
/// NOTE: Combination validity is a property of the resolved config
/// (overrides deep-merge field-wise, so two individually valid configs can compose into an invalid one),
/// which is why none of these checks can run at deserialize time.
/// Each rule lives with its owner and this function only converts and invokes them:
/// marker grammar on `SortGroupItemConfig` (the flat-list syntax's type),
/// combination / reference invariants on `SortImportsOptions` (the formatter's type),
/// enumerated values (selector / modifiers) at deserialize.
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
    if let Some(v) = sort_imports_config.custom_groups {
        sort_imports.custom_groups = v
            .into_iter()
            .map(|cg| CustomGroupDefinition {
                group_name: cg.group_name,
                element_name_pattern: cg.element_name_pattern,
                selector: cg.selector.map(to_import_selector),
                modifiers: cg
                    .modifiers
                    .unwrap_or_default()
                    .into_iter()
                    .map(to_import_modifier)
                    .collect(),
            })
            .collect();
    }
    if let Some(v) = sort_imports_config.groups {
        SortGroupItemConfig::validate_markers(&v)
            .map_err(|e| format!("Invalid `sortImports` configuration: {e}"))?;

        let mut groups = Vec::new();
        let mut newline_boundary_overrides: Vec<Option<bool>> = Vec::new();
        let mut pending_override: Option<bool> = None;
        for item in v {
            match item {
                SortGroupItemConfig::NewlinesBetween(marker) => {
                    // `validate_markers` ruled out leading/adjacent markers,
                    // so no override can be pending here.
                    debug_assert!(pending_override.is_none());
                    pending_override = Some(marker.newlines_between);
                }
                other => {
                    if !groups.is_empty() {
                        newline_boundary_overrides.push(pending_override.take());
                    }
                    groups.push(
                        other.into_vec().iter().map(|name| GroupEntry::parse(name)).collect(),
                    );
                }
            }
        }

        sort_imports.groups = groups;
        sort_imports.newline_boundary_overrides = newline_boundary_overrides;
    }

    sort_imports.validate().map_err(|e| format!("Invalid `sortImports` configuration: {e}"))?;

    Ok(Some(sort_imports))
}

/// Pure field translation (unknown values are rejected at deserialize time).
fn to_import_selector(config: ImportSelectorConfig) -> ImportSelector {
    match config {
        ImportSelectorConfig::Type => ImportSelector::Type,
        ImportSelectorConfig::SideEffectStyle => ImportSelector::SideEffectStyle,
        ImportSelectorConfig::SideEffect => ImportSelector::SideEffect,
        ImportSelectorConfig::Style => ImportSelector::Style,
        ImportSelectorConfig::Index => ImportSelector::Index,
        ImportSelectorConfig::Sibling => ImportSelector::Sibling,
        ImportSelectorConfig::Parent => ImportSelector::Parent,
        ImportSelectorConfig::Subpath => ImportSelector::Subpath,
        ImportSelectorConfig::Internal => ImportSelector::Internal,
        ImportSelectorConfig::Builtin => ImportSelector::Builtin,
        ImportSelectorConfig::External => ImportSelector::External,
        ImportSelectorConfig::Import => ImportSelector::Import,
    }
}

/// Pure field translation (unknown values are rejected at deserialize time).
fn to_import_modifier(config: ImportModifierConfig) -> ImportModifier {
    match config {
        ImportModifierConfig::SideEffect => ImportModifier::SideEffect,
        ImportModifierConfig::Type => ImportModifier::Type,
        ImportModifierConfig::Value => ImportModifier::Value,
        ImportModifierConfig::Default => ImportModifier::Default,
        ImportModifierConfig::Wildcard => ImportModifier::Wildcard,
        ImportModifierConfig::Named => ImportModifier::Named,
    }
}

/// Convert `jsdoc` into [`oxc_formatter::JsdocOptions`].
///
/// Enumerated options are validated at deserialize time, so this cannot fail.
pub(super) fn to_jsdoc(config: &FormatConfig) -> Option<JsdocOptions> {
    let jsdoc_config = config.jsdoc.clone().and_then(JsdocUserConfig::into_config)?;

    let mut opts = JsdocOptions::default();
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
    if let Some(v) = jsdoc_config.line_wrapping_style {
        opts.line_wrapping_style = match v {
            LineWrappingStyleConfig::Greedy => LineWrappingStyle::Greedy,
            LineWrappingStyleConfig::Balance => LineWrappingStyle::Balance,
        };
    }
    if let Some(v) = jsdoc_config.comment_line_strategy {
        opts.comment_line_strategy = match v {
            CommentLineStrategyConfig::SingleLine => CommentLineStrategy::SingleLine,
            CommentLineStrategyConfig::Multiline => CommentLineStrategy::Multiline,
            CommentLineStrategyConfig::Keep => CommentLineStrategy::Keep,
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

    Some(opts)
}

// ---

#[cfg(test)]
mod tests {
    use oxc_formatter::{Expand, GroupEntry, GroupName};

    use super::super::validate::validate;
    use super::*;

    /// Production shape: the gate validates/derives, then the infallible mapper builds.
    fn build(config: &FormatConfig) -> Result<JsFormatOptions, String> {
        let validated = validate(config)?;
        Ok(to_oxc_formatter(config, validated.core, validated.sort_imports))
    }

    /// The config enums mirror `oxc_formatter`'s (which deliberately carries no
    /// serde/schemars dependency). A config-side variant without a formatter
    /// counterpart cannot compile (the mapper match forces it), but a formatter-side
    /// addition missing its mirror is silent — this round-trip turns that into a failure.
    #[test]
    fn config_enums_mirror_formatter_enums() {
        for selector in ImportSelector::ALL_SELECTORS {
            let config: ImportSelectorConfig =
                serde_json::from_value(serde_json::json!(selector.name())).unwrap_or_else(|_| {
                    panic!("selector `{}` is missing from ImportSelectorConfig", selector.name())
                });
            assert_eq!(to_import_selector(config), *selector);
        }
        for modifier in ImportModifier::ALL_MODIFIERS {
            let config: ImportModifierConfig =
                serde_json::from_value(serde_json::json!(modifier.name())).unwrap_or_else(|_| {
                    panic!("modifier `{}` is missing from ImportModifierConfig", modifier.name())
                });
            assert_eq!(to_import_modifier(config), *modifier);
        }
    }

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
        let format_options = build(&config).unwrap();

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
        let format_options = build(&config).unwrap();

        // Should use defaults
        assert!(format_options.indent_style.is_space());
        assert_eq!(format_options.indent_width.value(), 2);
        assert_eq!(format_options.line_width.value(), 100);
        assert_eq!(format_options.sort_imports, None);
    }

    #[test]
    fn test_empty_config() {
        let config: FormatConfig = serde_json::from_str("{}").unwrap();
        let format_options = build(&config).unwrap();

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
        let format_options = build(&config).unwrap();
        assert!(format_options.arrow_parentheses.is_as_needed());

        // Test "always" remains unchanged
        let config: FormatConfig = serde_json::from_str(r#"{"arrowParens": "always"}"#).unwrap();
        let format_options = build(&config).unwrap();
        assert!(format_options.arrow_parentheses.is_always());
    }

    #[test]
    fn test_object_wrap_normalization() {
        // Test "preserve" -> "auto" normalization
        let config: FormatConfig = serde_json::from_str(r#"{"objectWrap": "preserve"}"#).unwrap();
        let format_options = build(&config).unwrap();
        assert_eq!(format_options.expand, Expand::Auto);

        // Test "collapse" -> "never" normalization
        let config: FormatConfig = serde_json::from_str(r#"{"objectWrap": "collapse"}"#).unwrap();
        let format_options = build(&config).unwrap();
        assert_eq!(format_options.expand, Expand::Never);
    }

    #[test]
    fn test_sort_imports_config() {
        let config: FormatConfig = serde_json::from_str(
            r#"{
            "experimentalSortImports": {}
        }"#,
        )
        .unwrap();
        let format_options = build(&config).unwrap();
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
        let format_options = build(&config).unwrap();
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
        let format_options = build(&config).unwrap();
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
        assert!(build(&config).is_ok());
        let config: FormatConfig = serde_json::from_str(
            r#"{
                "experimentalSortImports": {
                    "partitionByNewline": true,
                    "newlinesBetween": true
                }
            }"#,
        )
        .unwrap();
        assert!(build(&config).is_err_and(|e| e.contains("newlinesBetween")));

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
        let format_options = build(&config).unwrap();
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
        let format_options = build(&config).unwrap();
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
        assert!(build(&config).is_err_and(|e| e.contains("start")));

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
        assert!(build(&config).is_err_and(|e| e.contains("end")));

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
        assert!(build(&config).is_err_and(|e| e.contains("consecutive")));

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
        assert!(build(&config).is_err_and(|e| e.contains("partitionByNewline")));
    }

    #[test]
    fn test_bool_for_object_options() {
        let config: FormatConfig = serde_json::from_str(r#"{"sortImports": true}"#).unwrap();
        assert!(build(&config).unwrap().sort_imports.is_some());

        let config: FormatConfig = serde_json::from_str(r#"{"sortImports": false}"#).unwrap();
        assert!(build(&config).unwrap().sort_imports.is_none());

        // Tailwind collection maps napi-only
        let config: FormatConfig = serde_json::from_str(r#"{"sortTailwindcss": true}"#).unwrap();
        #[cfg(feature = "napi")]
        assert!(build(&config).unwrap().sort_tailwindcss.is_some());
        #[cfg(not(feature = "napi"))]
        assert!(build(&config).unwrap().sort_tailwindcss.is_none());

        let config: FormatConfig = serde_json::from_str(r#"{"sortTailwindcss": false}"#).unwrap();
        assert!(build(&config).unwrap().sort_tailwindcss.is_none());

        let config: FormatConfig = serde_json::from_str(r#"{"jsdoc": true}"#).unwrap();
        assert!(build(&config).unwrap().jsdoc.is_some());

        let config: FormatConfig = serde_json::from_str(r#"{"jsdoc": false}"#).unwrap();
        assert!(build(&config).unwrap().jsdoc.is_none());
    }

    #[test]
    fn validate_matches_build_validation() {
        // Valid config: both build and validate succeed.
        let config: FormatConfig =
            serde_json::from_str(r#"{ "printWidth": 80, "sortImports": true }"#).unwrap();
        assert!(validate(&config).is_ok());
        assert!(build(&config).is_ok());

        // Core range error (valid u16, but outside `LineWidth` bounds).
        let config: FormatConfig = serde_json::from_str(r#"{ "printWidth": 1000 }"#).unwrap();
        assert!(validate(&config).is_err());
        assert!(build(&config).is_err());

        // JS-specific error (sortImports) must be caught by `validate` too,
        // not just by building `JsFormatOptions`.
        let config: FormatConfig = serde_json::from_str(
            r#"{ "experimentalSortImports": { "groups": [{ "newlinesBetween": false }, "builtin"] } }"#,
        )
        .unwrap();
        assert!(validate(&config).is_err_and(|e| e.contains("start")));
        assert!(build(&config).is_err_and(|e| e.contains("start")));

        // JS-specific error (jsdoc enum) is rejected at deserialize time,
        // so neither `validate` nor `to_oxc_formatter` can ever see it.
        assert!(
            serde_json::from_str::<FormatConfig>(
                r#"{ "jsdoc": { "lineWrappingStyle": "bogus" } }"#
            )
            .is_err()
        );
    }
}
