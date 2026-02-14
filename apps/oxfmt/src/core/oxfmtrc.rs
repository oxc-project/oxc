use std::path::Path;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use oxc_formatter::{
    ArrowParentheses, AttributePosition, BracketSameLine, BracketSpacing, CustomGroupDefinition,
    EmbeddedLanguageFormatting, Expand, FormatOptions, ImportModifier, ImportSelector, IndentStyle,
    IndentWidth, LineEnding, LineWidth, QuoteProperties, QuoteStyle, Semicolons,
    SortImportsOptions, SortOrder, TailwindcssOptions, TrailingCommas,
};
use oxc_toml::Options as TomlFormatterOptions;

use super::{FormatFileStrategy, utils};

/// Configuration options for the Oxfmt.
///
/// Most options are the same as Prettier's options, but not all of them.
/// In addition, some options are our own extensions.
#[derive(Debug, Default, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct Oxfmtrc {
    #[serde(flatten)]
    pub format_config: FormatConfig,
    /// File-specific overrides.
    /// When a file matches multiple overrides, the later override takes precedence (array order matters).
    ///
    /// - Default: `[]`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overrides: Option<Vec<OxfmtOverrideConfig>>,
    /// Ignore files matching these glob patterns.
    /// Patterns are based on the location of the Oxfmt configuration file.
    ///
    /// - Default: `[]`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignore_patterns: Option<Vec<String>>,
}

// ---

#[derive(Debug, Default, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct OxfmtOverrideConfig {
    /// Glob patterns to match files for this override.
    /// All patterns are relative to the Oxfmt configuration file.
    pub files: Vec<String>,
    /// Glob patterns to exclude from this override.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude_files: Option<Vec<String>>,
    /// Format options to apply for matched files.
    #[serde(default)]
    pub options: FormatConfig,
}

// ---

// NOTE: All fields are typed as `Option` to distinguish between user-specified values and defaults.
#[derive(Debug, Default, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct FormatConfig {
    // ============================================================================================
    // Prettier compatible options, also used by `oxc_formatter` and TOML formatter
    // ============================================================================================
    /// Indent lines with tabs instead of spaces.
    ///
    /// - Default: `false`
    /// - Overrides `.editorconfig.indent_style`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_tabs: Option<bool>,
    /// Specify the number of spaces per indentation-level.
    ///
    /// - Default: `2`
    /// - Overrides `.editorconfig.indent_size`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tab_width: Option<u8>,
    /// Which end of line characters to apply.
    ///
    /// NOTE: `"auto"` is not supported.
    ///
    /// - Default: `"lf"`
    /// - Overrides `.editorconfig.end_of_line`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_of_line: Option<EndOfLineConfig>,
    /// Specify the line length that the printer will wrap on.
    ///
    /// If you don't want line wrapping when formatting Markdown, you can set the `proseWrap` option to disable it.
    ///
    /// - Default: `100`
    /// - Overrides `.editorconfig.max_line_length`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub print_width: Option<u16>,

    /// Use single quotes instead of double quotes.
    ///
    /// For JSX, you can set the `jsxSingleQuote` option.
    ///
    /// - Default: `false`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub single_quote: Option<bool>,
    /// Use single quotes instead of double quotes in JSX.
    ///
    /// - Default: `false`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jsx_single_quote: Option<bool>,
    /// Change when properties in objects are quoted.
    ///
    /// - Default: `"as-needed"`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quote_props: Option<QuotePropsConfig>,
    /// Print trailing commas wherever possible in multi-line comma-separated syntactic structures.
    ///
    /// A single-line array, for example, never gets trailing commas.
    ///
    /// - Default: `"all"`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trailing_comma: Option<TrailingCommaConfig>,
    /// Print semicolons at the ends of statements.
    ///
    /// - Default: `true`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub semi: Option<bool>,
    /// Include parentheses around a sole arrow function parameter.
    ///
    /// - Default: `"always"`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arrow_parens: Option<ArrowParensConfig>,
    /// Print spaces between brackets in object literals.
    ///
    /// - Default: `true`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bracket_spacing: Option<bool>,
    /// Put the `>` of a multi-line HTML (HTML, JSX, Vue, Angular) element at the end of the last line,
    /// instead of being alone on the next line (does not apply to self closing elements).
    ///
    /// - Default: `false`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bracket_same_line: Option<bool>,
    /// How to wrap object literals when they could fit on one line or span multiple lines.
    ///
    /// By default, formats objects as multi-line if there is a newline prior to the first property.
    /// Authors can use this heuristic to contextually improve readability, though it has some downsides.
    ///
    /// - Default: `"preserve"`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub object_wrap: Option<ObjectWrapConfig>,
    /// Enforce single attribute per line in HTML, Vue, and JSX.
    ///
    /// - Default: `false`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub single_attribute_per_line: Option<bool>,

    // NOTE: These experimental options are not yet supported.
    // Just be here to report error if they are used.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(skip)]
    pub experimental_operator_position: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(skip)]
    pub experimental_ternaries: Option<bool>,

    /// Control whether to format embedded parts (For example, CSS-in-JS, or JS-in-Vue, etc.) in the file.
    ///
    /// NOTE: XXX-in-JS support is incomplete.
    ///
    /// - Default: `"auto"`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedded_language_formatting: Option<EmbeddedLanguageFormattingConfig>,

    // ============================================================================================
    // Prettier compatible options and only used by Prettier
    // ============================================================================================
    /// How to wrap prose.
    ///
    /// By default, formatter will not change wrapping in markdown text since some services use a linebreak-sensitive renderer, e.g. GitHub comments and BitBucket.
    /// To wrap prose to the print width, change this option to "always".
    /// If you want to force all prose blocks to be on a single line and rely on editor/viewer soft wrapping instead, you can use "never".
    ///
    /// - Default: `"preserve"`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prose_wrap: Option<ProseWrapConfig>,
    /// Specify the global whitespace sensitivity for HTML, Vue, Angular, and Handlebars.
    ///
    /// - Default: `"css"`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html_whitespace_sensitivity: Option<HtmlWhitespaceSensitivityConfig>,
    /// Whether or not to indent the code inside `<script>` and `<style>` tags in Vue files.
    ///
    /// - Default: `false`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vue_indent_script_and_style: Option<bool>,

    // ============================================================================================
    // Below are our own extensions, handled by Oxfmt
    // ============================================================================================
    /// Whether to insert a final newline at the end of the file.
    ///
    /// - Default: `true`
    /// - Overrides `.editorconfig.insert_final_newline`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub insert_final_newline: Option<bool>,

    /// Experimental: Sort import statements.
    ///
    /// Using the similar algorithm as [eslint-plugin-perfectionist/sort-imports](https://perfectionist.dev/rules/sort-imports).
    /// For details, see each field's documentation.
    ///
    /// - Default: Disabled
    #[serde(skip_serializing_if = "Option::is_none")]
    pub experimental_sort_imports: Option<SortImportsConfig>,

    /// Experimental: Sort `package.json` keys.
    ///
    /// The algorithm is NOT compatible with [prettier-plugin-sort-packagejson](https://github.com/matzkoh/prettier-plugin-packagejson).
    /// But we believe it is clearer and easier to navigate.
    /// For details, see each field's documentation.
    ///
    /// - Default: `true`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub experimental_sort_package_json: Option<SortPackageJsonUserConfig>,

    /// Experimental: Sort Tailwind CSS classes.
    ///
    /// Using the same algorithm as [prettier-plugin-tailwindcss](https://github.com/tailwindlabs/prettier-plugin-tailwindcss).
    /// Option names omit the `tailwind` prefix used in the original plugin (e.g., `config` instead of `tailwindConfig`).
    /// For details, see each field's documentation.
    ///
    /// - Default: Disabled
    #[serde(skip_serializing_if = "Option::is_none")]
    pub experimental_tailwindcss: Option<TailwindcssConfig>,
}

impl FormatConfig {
    /// Resolve relative tailwind paths (`config`, `stylesheet`) to absolute paths.
    /// Otherwise, the plugin tries to resolve the Prettier's configuration file, not Oxfmt's.
    /// <https://github.com/tailwindlabs/prettier-plugin-tailwindcss/blob/125a8bc77639529a5a0c7e4e8a02174d7ed2d70b/src/config.ts#L50-L54>
    pub fn resolve_tailwind_paths(&mut self, base_dir: &Path) {
        let Some(ref mut tw) = self.experimental_tailwindcss else {
            return;
        };

        for path_field in [&mut tw.config, &mut tw.stylesheet] {
            let Some(path_str) = path_field.as_ref() else {
                continue;
            };

            let path = Path::new(path_str);
            if path.is_relative() {
                *path_field = Some(
                    utils::normalize_relative_path(base_dir, path).to_string_lossy().to_string(),
                );
            }
        }
    }

    /// Merge another `FormatConfig`, overwriting only fields that are `Some<T>`.
    ///
    /// # Panics
    /// Panics if serialization/deserialization fails,
    /// which should never happen for valid `FormatConfig` structs.
    pub fn merge(&mut self, other: &Self) {
        let base = serde_json::to_value(&*self).unwrap();
        let overlay = serde_json::to_value(other).unwrap();
        let merged = json_deep_merge(base, overlay);
        *self = serde_json::from_value(merged).unwrap();
    }

    /// Convert to `OxfmtOptions`.
    ///
    /// # Errors
    /// Returns error if any option value is invalid
    pub fn into_oxfmt_options(self) -> Result<OxfmtOptions, String> {
        // Not yet supported options:
        // [Prettier] experimentalOperatorPosition: "start" | "end"
        // [Prettier] experimentalTernaries: boolean
        if self.experimental_operator_position.is_some() {
            return Err("Unsupported option: `experimentalOperatorPosition`".to_string());
        }
        if self.experimental_ternaries.is_some() {
            return Err("Unsupported option: `experimentalTernaries`".to_string());
        }

        // All values are based on defaults from `FormatOptions::default()`
        let mut format_options = FormatOptions::default();

        // [Prettier] useTabs: boolean
        if let Some(use_tabs) = self.use_tabs {
            format_options.indent_style =
                if use_tabs { IndentStyle::Tab } else { IndentStyle::Space };
        }

        // [Prettier] tabWidth: number
        if let Some(width) = self.tab_width {
            format_options.indent_width =
                IndentWidth::try_from(width).map_err(|e| format!("Invalid tabWidth: {e}"))?;
        }

        // [Prettier] endOfLine: "lf" | "cr" | "crlf" | "auto"
        // NOTE: "auto" is not supported
        if let Some(ending) = self.end_of_line {
            format_options.line_ending = match ending {
                EndOfLineConfig::Lf => LineEnding::Lf,
                EndOfLineConfig::Crlf => LineEnding::Crlf,
                EndOfLineConfig::Cr => LineEnding::Cr,
            };
        }

        // [Prettier] printWidth: number
        if let Some(width) = self.print_width {
            format_options.line_width =
                LineWidth::try_from(width).map_err(|e| format!("Invalid printWidth: {e}"))?;
        }

        // [Prettier] singleQuote: boolean
        if let Some(single_quote) = self.single_quote {
            format_options.quote_style =
                if single_quote { QuoteStyle::Single } else { QuoteStyle::Double };
        }

        // [Prettier] jsxSingleQuote: boolean
        if let Some(jsx_single_quote) = self.jsx_single_quote {
            format_options.jsx_quote_style =
                if jsx_single_quote { QuoteStyle::Single } else { QuoteStyle::Double };
        }

        // [Prettier] quoteProps: "as-needed" | "consistent" | "preserve"
        if let Some(props) = self.quote_props {
            format_options.quote_properties = match props {
                QuotePropsConfig::AsNeeded => QuoteProperties::AsNeeded,
                QuotePropsConfig::Consistent => QuoteProperties::Consistent,
                QuotePropsConfig::Preserve => QuoteProperties::Preserve,
            };
        }

        // [Prettier] trailingComma: "all" | "es5" | "none"
        if let Some(commas) = self.trailing_comma {
            format_options.trailing_commas = match commas {
                TrailingCommaConfig::All => TrailingCommas::All,
                TrailingCommaConfig::Es5 => TrailingCommas::Es5,
                TrailingCommaConfig::None => TrailingCommas::None,
            };
        }

        // [Prettier] semi: boolean
        if let Some(semi) = self.semi {
            format_options.semicolons =
                if semi { Semicolons::Always } else { Semicolons::AsNeeded };
        }

        // [Prettier] arrowParens: "avoid" | "always"
        if let Some(parens) = self.arrow_parens {
            format_options.arrow_parentheses = match parens {
                ArrowParensConfig::Avoid => ArrowParentheses::AsNeeded,
                ArrowParensConfig::Always => ArrowParentheses::Always,
            };
        }

        // [Prettier] bracketSpacing: boolean
        if let Some(spacing) = self.bracket_spacing {
            format_options.bracket_spacing = BracketSpacing::from(spacing);
        }

        // [Prettier] bracketSameLine: boolean
        if let Some(same_line) = self.bracket_same_line {
            format_options.bracket_same_line = BracketSameLine::from(same_line);
        }

        // [Prettier] singleAttributePerLine: boolean
        if let Some(single_attribute_per_line) = self.single_attribute_per_line {
            format_options.attribute_position = if single_attribute_per_line {
                AttributePosition::Multiline
            } else {
                AttributePosition::Auto
            };
        }

        // [Prettier] objectWrap: "preserve" | "collapse"
        if let Some(object_wrap) = self.object_wrap {
            format_options.expand = match object_wrap {
                ObjectWrapConfig::Preserve => Expand::Auto,
                ObjectWrapConfig::Collapse => Expand::Never,
            };
        }

        // [Prettier] embeddedLanguageFormatting: "auto" | "off"
        if let Some(embedded_language_formatting) = self.embedded_language_formatting {
            format_options.embedded_language_formatting = match embedded_language_formatting {
                EmbeddedLanguageFormattingConfig::Auto => EmbeddedLanguageFormatting::Auto,
                EmbeddedLanguageFormattingConfig::Off => EmbeddedLanguageFormatting::Off,
            };
        }

        // Below are our own extensions

        if let Some(config) = self.experimental_sort_imports {
            let mut sort_imports = SortImportsOptions::default();

            if let Some(v) = config.partition_by_newline {
                sort_imports.partition_by_newline = v;
            }
            if let Some(v) = config.partition_by_comment {
                sort_imports.partition_by_comment = v;
            }
            if let Some(v) = config.sort_side_effects {
                sort_imports.sort_side_effects = v;
            }
            if let Some(v) = config.order {
                sort_imports.order = match v {
                    SortOrderConfig::Asc => SortOrder::Asc,
                    SortOrderConfig::Desc => SortOrder::Desc,
                };
            }
            if let Some(v) = config.ignore_case {
                sort_imports.ignore_case = v;
            }
            if let Some(v) = config.newlines_between {
                sort_imports.newlines_between = v;
            }
            if let Some(v) = config.internal_pattern {
                sort_imports.internal_pattern = v;
            }
            if let Some(v) = config.groups {
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
                                // Record the boundary between the previous group and this one.
                                // `pending_override` is
                                // - `Some(bool)` if a marker preceded this group
                                // - or `None` (= use global `newlines_between`) otherwise
                                // For the very first group (`groups.is_empty()`),
                                // there is no preceding boundary, so we skip this entirely.
                                newline_boundary_overrides.push(pending_override.take());
                            }
                            groups.push(other.into_vec());
                        }
                    }
                }

                if pending_override.is_some() {
                    return Err("Invalid `sortImports` configuration: `{ \"newlinesBetween\" }` marker cannot appear at the end of `groups`".to_string());
                }

                sort_imports.groups = groups;
                sort_imports.newline_boundary_overrides = newline_boundary_overrides;
            }

            if sort_imports.partition_by_newline
                && sort_imports.newline_boundary_overrides.iter().any(Option::is_some)
            {
                return Err("Invalid `sortImports` configuration: `partitionByNewline` and per-group `{ \"newlinesBetween\" }` markers cannot be used together".to_string());
            }
            if let Some(v) = config.custom_groups {
                sort_imports.custom_groups = v
                    .into_iter()
                    .map(|c| CustomGroupDefinition {
                        group_name: c.group_name,
                        element_name_pattern: c.element_name_pattern,
                        selector: c.selector.as_deref().and_then(ImportSelector::parse),
                        modifiers: c
                            .modifiers
                            .unwrap_or_default()
                            .iter()
                            .filter_map(|s| ImportModifier::parse(s))
                            .collect(),
                    })
                    .collect();
            }

            // `partition_by_newline: true` and `newlines_between: true` cannot be used together
            if sort_imports.partition_by_newline && sort_imports.newlines_between {
                return Err("Invalid `sortImports` configuration: `partitionByNewline: true` and `newlinesBetween: true` cannot be used together".to_string());
            }

            format_options.experimental_sort_imports = Some(sort_imports);
        }

        if let Some(config) = self.experimental_tailwindcss {
            format_options.experimental_tailwindcss = Some(TailwindcssOptions {
                config: config.config,
                stylesheet: config.stylesheet,
                functions: config.functions.unwrap_or_default(),
                attributes: config.attributes.unwrap_or_default(),
                preserve_whitespace: config.preserve_whitespace.unwrap_or(false),
                preserve_duplicates: config.preserve_duplicates.unwrap_or(false),
            });
        }

        // Currently, there is a no options for TOML formatter
        let toml_options = build_toml_options(&format_options);

        let sort_package_json = self.experimental_sort_package_json.map_or_else(
            || Some(SortPackageJsonConfig::default().to_sort_options()),
            |c| c.to_sort_options(),
        );

        let insert_final_newline = self.insert_final_newline.unwrap_or(true);

        Ok(OxfmtOptions { format_options, toml_options, sort_package_json, insert_final_newline })
    }
}

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

#[derive(Debug, Clone, Copy, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum EndOfLineConfig {
    Lf,
    Crlf,
    Cr,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum QuotePropsConfig {
    AsNeeded,
    Consistent,
    Preserve,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum TrailingCommaConfig {
    All,
    Es5,
    None,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ArrowParensConfig {
    Always,
    Avoid,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ObjectWrapConfig {
    Preserve,
    Collapse,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum EmbeddedLanguageFormattingConfig {
    Auto,
    Off,
}

// ---

#[derive(Debug, Clone, Copy, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ProseWrapConfig {
    Always,
    Never,
    Preserve,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum HtmlWhitespaceSensitivityConfig {
    Css,
    Strict,
    Ignore,
}

// ---

#[derive(Debug, Clone, Default, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct SortImportsConfig {
    /// Enables the empty line to separate imports into logical groups.
    ///
    /// When `true`, formatter will not sort imports if there is an empty line between them.
    /// This helps maintain the defined order of logically separated groups of members.
    ///
    /// ```js
    /// import { b1, b2 } from 'b'
    ///
    /// import { a } from 'a'
    /// import { c } from 'c'
    /// ```
    ///
    /// - Default: `false`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub partition_by_newline: Option<bool>,
    /// Enables the use of comments to separate imports into logical groups.
    ///
    /// When `true`, all comments will be treated as delimiters, creating partitions.
    ///
    /// ```js
    /// import { b1, b2 } from 'b'
    /// // PARTITION
    /// import { a } from 'a'
    /// import { c } from 'c'
    /// ```
    ///
    /// - Default: `false`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub partition_by_comment: Option<bool>,
    /// Specifies whether side effect imports should be sorted.
    ///
    /// By default, sorting side-effect imports is disabled for security reasons.
    ///
    /// - Default: `false`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_side_effects: Option<bool>,
    /// Specifies whether to sort items in ascending or descending order.
    ///
    /// - Default: `"asc"`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<SortOrderConfig>,
    /// Specifies whether sorting should be case-sensitive.
    ///
    /// - Default: `true`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignore_case: Option<bool>,
    /// Specifies whether to add newlines between groups.
    ///
    /// When `false`, no newlines are added between groups.
    ///
    /// - Default: `true`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub newlines_between: Option<bool>,
    /// Specifies a prefix for identifying internal imports.
    ///
    /// This is useful for distinguishing your own modules from external dependencies.
    ///
    /// - Default: `["~/", "@/"]`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub internal_pattern: Option<Vec<String>>,
    /// Specifies a list of predefined import groups for sorting.
    ///
    /// Each import will be assigned a single group specified in the groups option (or the `unknown` group if no match is found).
    /// The order of items in the `groups` option determines how groups are ordered.
    ///
    /// Within a given group, members will be sorted according to the type, order, ignoreCase, etc. options.
    ///
    /// Individual groups can be combined together by placing them in an array.
    /// The order of groups in that array does not matter.
    /// All members of the groups in the array will be sorted together as if they were part of a single group.
    ///
    /// Predefined groups are characterized by a single selector and potentially multiple modifiers.
    /// You may enter modifiers in any order, but the selector must always come at the end.
    ///
    /// The list of selectors is sorted from most to least important:
    /// - `type` — TypeScript type imports.
    /// - `side-effect-style` — Side effect style imports.
    /// - `side-effect` — Side effect imports.
    /// - `style` — Style imports.
    /// - `index` — Main file from the current directory.
    /// - `sibling` — Modules from the same directory.
    /// - `parent` — Modules from the parent directory.
    /// - `subpath` — Node.js subpath imports.
    /// - `internal` — Your internal modules.
    /// - `builtin` — Node.js Built-in Modules.
    /// - `external` — External modules installed in the project.
    /// - `import` — Any import.
    ///
    /// The list of modifiers is sorted from most to least important:
    /// - `side-effect` — Side effect imports.
    /// - `type` — TypeScript type imports.
    /// - `value` — Value imports.
    /// - `default` — Imports containing the default specifier.
    /// - `wildcard` — Imports containing the wildcard (`* as`) specifier.
    /// - `named` — Imports containing at least one named specifier.
    ///
    /// - Default: See below
    /// ```json
    /// [
    ///   "type-import",
    ///   ["value-builtin", "value-external"],
    ///   "type-internal",
    ///   "value-internal",
    ///   ["type-parent", "type-sibling", "type-index"],
    ///   ["value-parent", "value-sibling", "value-index"],
    ///   "unknown",
    /// ]
    /// ```
    #[serde(skip_serializing_if = "Option::is_none")]
    pub groups: Option<Vec<SortGroupItemConfig>>,
    /// Define your own groups for matching very specific imports.
    ///
    /// The `customGroups` list is ordered: The first definition that matches an element will be used.
    /// Custom groups have a higher priority than any predefined group.
    ///
    /// If you want a predefined group to take precedence over a custom group,
    /// you must write a custom group definition that does the same as what the predefined group does, and put it first in the list.
    ///
    /// If you specify multiple conditions like `elementNamePattern`, `selector`, and `modifiers`,
    /// all conditions must be met for an import to match the custom group (AND logic).
    ///
    /// - Default: `[]`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_groups: Option<Vec<CustomGroupItemConfig>>,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum SortOrderConfig {
    Asc,
    Desc,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(untagged)]
pub enum SortGroupItemConfig {
    /// A `{ "newlinesBetween": bool }` marker object that overrides the global `newlinesBetween`
    /// setting for the boundary between the previous and next groups.
    NewlinesBetween(NewlinesBetweenMarker),
    /// A single group name string (e.g. `"value-builtin"`).
    Single(String),
    /// Multiple group names treated as one group (e.g. `["value-builtin", "value-external"]`).
    Multiple(Vec<String>),
}

/// A marker object for overriding `newlinesBetween` at a specific group boundary.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct NewlinesBetweenMarker {
    pub newlines_between: bool,
}

impl SortGroupItemConfig {
    fn into_vec(self) -> Vec<String> {
        match self {
            Self::Single(s) => vec![s],
            Self::Multiple(v) => v,
            Self::NewlinesBetween(_) => {
                unreachable!("NewlinesBetween markers should be handled before calling into_vec")
            }
        }
    }
}

#[derive(Debug, Default, Clone, Eq, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct CustomGroupItemConfig {
    /// Name of the custom group, used in the `groups` option.
    pub group_name: String,
    /// List of glob patterns to match import sources for this group.
    pub element_name_pattern: Vec<String>,
    /// Selector to match the import kind.
    ///
    /// Possible values: `"type"`, `"side-effect-style"`, `"side-effect"`, `"style"`, `"index"`,
    /// `"sibling"`, `"parent"`, `"subpath"`, `"internal"`, `"builtin"`, `"external"`, `"import"`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selector: Option<String>,
    /// Modifiers to match the import characteristics.
    /// All specified modifiers must be present (AND logic).
    ///
    /// Possible values: `"side-effect"`, `"type"`, `"value"`, `"default"`, `"wildcard"`, `"named"`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modifiers: Option<Vec<String>>,
}

// ---

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(untagged)]
pub enum SortPackageJsonUserConfig {
    Bool(bool),
    Object(SortPackageJsonConfig),
}

impl Default for SortPackageJsonUserConfig {
    fn default() -> Self {
        Self::Bool(true)
    }
}

impl SortPackageJsonUserConfig {
    /// Convert to `sort_package_json::SortOptions`.
    /// Returns `None` if sorting is disabled.
    pub fn to_sort_options(&self) -> Option<sort_package_json::SortOptions> {
        match self {
            Self::Bool(false) => None,
            Self::Bool(true) => Some(SortPackageJsonConfig::default().to_sort_options()),
            Self::Object(config) => Some(config.to_sort_options()),
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct SortPackageJsonConfig {
    /// Sort the `scripts` field alphabetically.
    ///
    /// - Default: `false`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_scripts: Option<bool>,
}

impl SortPackageJsonConfig {
    fn to_sort_options(&self) -> sort_package_json::SortOptions {
        sort_package_json::SortOptions {
            sort_scripts: self.sort_scripts.unwrap_or(false),
            // Small optimization: Prettier will reformat anyway
            pretty: false,
        }
    }
}

// ---

#[derive(Debug, Clone, Default, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct TailwindcssConfig {
    /// Path to your Tailwind CSS configuration file (v3).
    ///
    /// NOTE: Paths are resolved relative to the Oxfmt configuration file.
    ///
    /// - Default: Automatically find `"tailwind.config.js"`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<String>,
    /// Path to your Tailwind CSS stylesheet (v4).
    ///
    /// NOTE: Paths are resolved relative to the Oxfmt configuration file.
    ///
    /// - Default: Installed Tailwind CSS's `theme.css`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stylesheet: Option<String>,

    /// List of custom function names whose arguments should be sorted (exact match).
    ///
    /// NOTE: Regex patterns are not yet supported.
    ///
    /// - Default: `[]`
    /// - Example: `["clsx", "cn", "cva", "tw"]`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub functions: Option<Vec<String>>,
    /// List of additional attributes to sort beyond `class` and `className` (exact match).
    ///
    /// NOTE: Regex patterns are not yet supported.
    ///
    /// - Default: `[]`
    /// - Example: `["myClassProp", ":class"]`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attributes: Option<Vec<String>>,

    /// Preserve whitespace around classes.
    ///
    /// - Default: `false`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preserve_whitespace: Option<bool>,
    /// Preserve duplicate classes.
    ///
    /// - Default: `false`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preserve_duplicates: Option<bool>,
}

// ---

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

/// Syncs resolved `FormatOptions` values into the raw config JSON.
/// This ensures `external_formatter`(Prettier) receives the same options that `oxc_formatter` uses.
///
/// Only options that meet one of these criteria need to be mapped:
/// - 1. Different defaults between Prettier and oxc_formatter
///   - e.g. `printWidth`: Prettier: 80, Oxfmt: 100
/// - 2. Can be set via `.editorconfig` (values won't be in raw config JSON)
///   - `max_line_length` -> `printWidth`
///   - `end_of_line` -> `endOfLine`
///   - `indent_style` -> `useTabs`
///   - `indent_size` -> `tabWidth`
///
/// This function should be called once during config caching.
/// For strategy-specific options (plugin flags), use [`finalize_external_options()`] separately.
pub fn sync_external_options(options: &FormatOptions, config: &mut Value) {
    let Some(obj) = config.as_object_mut() else {
        return;
    };

    // vs Prettier defaults and `.editorconfig` values
    obj.insert("printWidth".to_string(), Value::from(options.line_width.value()));

    // vs `.editorconfig` values
    obj.insert(
        "useTabs".to_string(),
        Value::from(match options.indent_style {
            IndentStyle::Tab => true,
            IndentStyle::Space => false,
        }),
    );
    obj.insert("tabWidth".to_string(), Value::from(options.indent_width.value()));
    obj.insert(
        "endOfLine".to_string(),
        Value::from(match options.line_ending {
            LineEnding::Lf => "lf",
            LineEnding::Crlf => "crlf",
            LineEnding::Cr => "cr",
        }),
    );

    // Any other fields are preserved as-is.
    // - e.g. `htmlWhitespaceSensitivity`, `vueIndentScriptAndStyle`, etc.
    //   - Defined in `Oxfmtrc`, but only used by Prettier
    // - e.g. `plugins`
    //   - It does not mean plugin works correctly with Oxfmt
    //   - Oxfmt still not aware of any plugin-defined languages
    // Other options defined independently by plugins are also left as they are.
}

/// Parsers that can embed JS/TS code and benefit from Tailwind plugin
#[cfg(feature = "napi")]
static TAILWIND_PARSERS: phf::Set<&'static str> = phf::phf_set! {
    "html",
    "vue",
    "angular",
    "glimmer",
};

/// Parsers that can embed JS/TS code and benefit from oxfmt plugin.
/// For now, expressions are not supported.
/// - e.g. `__vue_expression` in `vue`
/// - e.g. `__ng_directive` in `angular`
#[cfg(feature = "napi")]
static OXFMT_PARSERS: phf::Set<&'static str> = phf::phf_set! {
    // "html",
    // "vue",
    // "markdown",
    // "mdx",
};

/// Finalizes external options by adding plugin-specific flags based on the formatting strategy.
/// This should be called during `resolve()` after getting cached config.
///
/// - `_useTailwindPlugin`: Flag for JS side to load Tailwind plugin
/// - `_oxfmtPluginOptionsJson`: Bundled options for `prettier-plugin-oxfmt`
///
/// Also removes Prettier-unaware options to minimize payload size.
pub fn finalize_external_options(config: &mut Value, strategy: &FormatFileStrategy) {
    let Some(obj) = config.as_object_mut() else {
        return;
    };

    // Determine if Tailwind plugin should be used based on config and strategy
    let use_tailwind = obj.contains_key("experimentalTailwindcss")
        && match strategy {
            FormatFileStrategy::OxcFormatter { .. } => true,
            #[cfg(feature = "napi")]
            FormatFileStrategy::ExternalFormatter { parser_name, .. } => {
                TAILWIND_PARSERS.contains(parser_name)
            }
            _ => false,
        };

    // Add Tailwind plugin flag and map options
    // See: https://github.com/tailwindlabs/prettier-plugin-tailwindcss#options
    if use_tailwind {
        if let Some(tailwind) =
            obj.get("experimentalTailwindcss").and_then(|v| v.as_object()).cloned()
        {
            for (src, dst) in [
                ("config", "tailwindConfig"),
                ("stylesheet", "tailwindStylesheet"),
                ("functions", "tailwindFunctions"),
                ("attributes", "tailwindAttributes"),
                ("preserveWhitespace", "tailwindPreserveWhitespace"),
                ("preserveDuplicates", "tailwindPreserveDuplicates"),
            ] {
                if let Some(value) = tailwind.get(src).cloned() {
                    obj.insert(dst.to_string(), value);
                }
            }
        }
        obj.insert("_useTailwindPlugin".to_string(), Value::Number(1.into()));
    }

    // Build oxfmt plugin options JSON for js-in-xxx parsers
    #[cfg(feature = "napi")]
    if let FormatFileStrategy::ExternalFormatter { parser_name, .. } = strategy
        && OXFMT_PARSERS.contains(parser_name)
    {
        let mut oxfmt_plugin_options = serde_json::Map::new();

        for key in [
            "printWidth",
            "useTabs",
            "tabWidth",
            "endOfLine",
            "singleQuote",
            "bracketSpacing",
            "bracketSameLine",
            "semi",
            "trailingComma",
            "arrowParens",
            "quoteProps",
            "jsxSingleQuote",
            "experimentalSortImports",
            "experimentalTailwindcss",
        ] {
            if let Some(value) = obj.get(key) {
                oxfmt_plugin_options.insert(key.to_string(), value.clone());
            }
        }

        // In embedded contexts, final newline is useless
        oxfmt_plugin_options.insert("insertFinalNewline".to_string(), false.into());

        if let Ok(json_str) = serde_json::to_string(&Value::Object(oxfmt_plugin_options)) {
            obj.insert("_oxfmtPluginOptionsJson".to_string(), Value::String(json_str));
        }
    }

    // To minimize payload size, remove Prettier unaware options
    for key in [
        "experimentalSortImports",
        "experimentalTailwindcss",
        "experimentalSortPackageJson",
        "insertFinalNewline",
        "overrides",
        "ignorePatterns",
    ] {
        obj.remove(key);
    }
}

// ---

/// Merge two JSON values recursively.
/// - Overlay values overwrite base values
/// - `null` values in overlay reset the field to default (via `Option<T>` → `None`)
///
/// All Prettier options are flat, but some our options are nested.
fn json_deep_merge(base: Value, overlay: Value) -> Value {
    match (base, overlay) {
        (Value::Object(mut base_map), Value::Object(overlay_map)) => {
            for (key, overlay_value) in overlay_map {
                let merged = if let Some(base_value) = base_map.remove(&key) {
                    json_deep_merge(base_value, overlay_value)
                } else {
                    overlay_value
                };
                base_map.insert(key, merged);
            }
            Value::Object(base_map)
        }
        (_base, overlay) => overlay,
    }
}

// ---

#[cfg(test)]
mod tests {
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
        let oxfmt_options = config.into_oxfmt_options().unwrap();

        assert!(oxfmt_options.format_options.indent_style.is_tab());
        assert_eq!(oxfmt_options.format_options.indent_width.value(), 4);
        assert_eq!(oxfmt_options.format_options.line_width.value(), 100);
        assert!(!oxfmt_options.format_options.quote_style.is_double());
        assert!(oxfmt_options.format_options.semicolons.is_as_needed());

        let sort_imports = oxfmt_options.format_options.experimental_sort_imports.unwrap();
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
        let oxfmt_options = config.into_oxfmt_options().unwrap();

        // Should use defaults
        assert!(oxfmt_options.format_options.indent_style.is_space());
        assert_eq!(oxfmt_options.format_options.indent_width.value(), 2);
        assert_eq!(oxfmt_options.format_options.line_width.value(), 100);
        assert_eq!(oxfmt_options.format_options.experimental_sort_imports, None);
    }

    #[test]
    fn test_empty_config() {
        let config: FormatConfig = serde_json::from_str("{}").unwrap();
        let oxfmt_options = config.into_oxfmt_options().unwrap();

        // Should use defaults
        assert!(oxfmt_options.format_options.indent_style.is_space());
        assert_eq!(oxfmt_options.format_options.indent_width.value(), 2);
        assert_eq!(oxfmt_options.format_options.line_width.value(), 100);
        assert_eq!(oxfmt_options.format_options.experimental_sort_imports, None);
    }

    #[test]
    fn test_arrow_parens_normalization() {
        // Test "avoid" -> "as-needed" normalization
        let config: FormatConfig = serde_json::from_str(r#"{"arrowParens": "avoid"}"#).unwrap();
        let oxfmt_options = config.into_oxfmt_options().unwrap();
        assert!(oxfmt_options.format_options.arrow_parentheses.is_as_needed());

        // Test "always" remains unchanged
        let config: FormatConfig = serde_json::from_str(r#"{"arrowParens": "always"}"#).unwrap();
        let oxfmt_options = config.into_oxfmt_options().unwrap();
        assert!(oxfmt_options.format_options.arrow_parentheses.is_always());
    }

    #[test]
    fn test_object_wrap_normalization() {
        // Test "preserve" -> "auto" normalization
        let config: FormatConfig = serde_json::from_str(r#"{"objectWrap": "preserve"}"#).unwrap();
        let oxfmt_options = config.into_oxfmt_options().unwrap();
        assert_eq!(oxfmt_options.format_options.expand, Expand::Auto);

        // Test "collapse" -> "never" normalization
        let config: FormatConfig = serde_json::from_str(r#"{"objectWrap": "collapse"}"#).unwrap();
        let oxfmt_options = config.into_oxfmt_options().unwrap();
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
        let oxfmt_options = config.into_oxfmt_options().unwrap();
        let sort_imports = oxfmt_options.format_options.experimental_sort_imports.unwrap();
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
        let oxfmt_options = config.into_oxfmt_options().unwrap();
        let sort_imports = oxfmt_options.format_options.experimental_sort_imports.unwrap();
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
        let oxfmt_options = config.into_oxfmt_options().unwrap();
        let sort_imports = oxfmt_options.format_options.experimental_sort_imports.unwrap();
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
        assert!(config.into_oxfmt_options().is_ok());
        let config: FormatConfig = serde_json::from_str(
            r#"{
                "experimentalSortImports": {
                    "partitionByNewline": true,
                    "newlinesBetween": true
                }
            }"#,
        )
        .unwrap();
        assert!(config.into_oxfmt_options().is_err_and(|e| e.contains("newlinesBetween")));

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
        let oxfmt_options = config.into_oxfmt_options().unwrap();
        let sort_imports = oxfmt_options.format_options.experimental_sort_imports.unwrap();
        assert_eq!(sort_imports.groups.len(), 5);
        assert_eq!(sort_imports.groups[0], vec!["builtin".to_string()]);
        assert_eq!(sort_imports.groups[1], vec!["external".to_string(), "internal".to_string()]);
        assert_eq!(sort_imports.groups[4], vec!["index".to_string()]);

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
        let oxfmt_options = config.into_oxfmt_options().unwrap();
        let sort_imports = oxfmt_options.format_options.experimental_sort_imports.unwrap();
        assert_eq!(sort_imports.groups.len(), 3);
        assert_eq!(sort_imports.groups[0], vec!["builtin".to_string()]);
        assert_eq!(sort_imports.groups[1], vec!["external".to_string()]);
        assert_eq!(sort_imports.groups[2], vec!["parent".to_string()]);
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
        assert!(config.into_oxfmt_options().is_err_and(|e| e.contains("start")));

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
        assert!(config.into_oxfmt_options().is_err_and(|e| e.contains("end")));

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
        assert!(config.into_oxfmt_options().is_err_and(|e| e.contains("consecutive")));

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
        assert!(config.into_oxfmt_options().is_err_and(|e| e.contains("partitionByNewline")));
    }
}

#[cfg(test)]
mod tests_sync_external_options {
    use super::*;

    #[test]
    fn test_sync_external_options_defaults() {
        let json_string = r"{}";
        let mut raw_config: Value = serde_json::from_str(json_string).unwrap();
        let config: FormatConfig = serde_json::from_str(json_string).unwrap();
        let oxfmt_options = config.into_oxfmt_options().unwrap();

        sync_external_options(&oxfmt_options.format_options, &mut raw_config);

        let obj = raw_config.as_object().unwrap();
        assert_eq!(obj.get("printWidth").unwrap(), 100);
    }

    #[test]
    fn test_sync_external_options_with_user_values() {
        let json_string = r#"{
            "printWidth": 80,
            "ignorePatterns": ["*.min.js"],
            "experimentalSortImports": { "order": "asc" }
        }"#;
        let mut raw_config: Value = serde_json::from_str(json_string).unwrap();
        let config: FormatConfig = serde_json::from_str(json_string).unwrap();
        let oxfmt_options = config.into_oxfmt_options().unwrap();

        sync_external_options(&oxfmt_options.format_options, &mut raw_config);

        let obj = raw_config.as_object().unwrap();
        // User-specified value is preserved via FormatOptions
        assert_eq!(obj.get("printWidth").unwrap(), 80);
        // oxfmt extensions are preserved (for caching)
        // They will be removed later by `finalize_external_options()`
        assert!(obj.contains_key("ignorePatterns"));
        assert!(obj.contains_key("experimentalSortImports"));
    }

    #[test]
    fn test_overrides_parsing() {
        let json = r#"{
            "tabWidth": 2,
            "overrides": [
                {
                    "files": ["*.test.js"],
                    "options": { "tabWidth": 4 }
                },
                {
                    "files": ["*.md", "*.html"],
                    "excludeFiles": ["*.min.js"],
                    "options": { "printWidth": 80 }
                }
            ]
        }"#;

        let config: Oxfmtrc = serde_json::from_str(json).unwrap();
        assert!(config.overrides.is_some());

        let overrides = config.overrides.unwrap();
        assert_eq!(overrides.len(), 2);

        // First override: single file pattern
        assert_eq!(overrides[0].files, vec!["*.test.js"]);
        assert!(overrides[0].exclude_files.is_none());
        assert_eq!(overrides[0].options.tab_width, Some(4));

        // Second override: multiple file patterns with exclude
        assert_eq!(overrides[1].files, vec!["*.md", "*.html"]);
        assert_eq!(overrides[1].exclude_files, Some(vec!["*.min.js".to_string()]));
        assert_eq!(overrides[1].options.print_width, Some(80));
    }

    #[test]
    fn test_sync_external_options_preserves_overrides() {
        let json_string = r#"{
            "tabWidth": 2,
            "overrides": [
                { "files": ["*.test.js"], "options": { "tabWidth": 4 } }
            ]
        }"#;
        let mut raw_config: Value = serde_json::from_str(json_string).unwrap();
        let oxfmtrc: Oxfmtrc = serde_json::from_str(json_string).unwrap();
        let oxfmt_options = oxfmtrc.format_config.into_oxfmt_options().unwrap();

        sync_external_options(&oxfmt_options.format_options, &mut raw_config);

        let obj = raw_config.as_object().unwrap();
        // Overrides are preserved (for caching)
        // They will be removed later by `finalize_external_options()`
        assert!(obj.contains_key("overrides"));
    }

    #[test]
    fn test_finalize_external_options_removes_oxfmt_extensions() {
        use std::path::PathBuf;

        use oxc_span::SourceType;

        let json_string = r#"{
            "tabWidth": 2,
            "overrides": [
                { "files": ["*.test.js"], "options": { "tabWidth": 4 } }
            ],
            "ignorePatterns": ["*.min.js"],
            "experimentalSortImports": { "order": "asc" }
        }"#;
        let mut raw_config: Value = serde_json::from_str(json_string).unwrap();

        let strategy = super::super::FormatFileStrategy::OxcFormatter {
            path: PathBuf::from("test.js"),
            source_type: SourceType::mjs(),
        };
        finalize_external_options(&mut raw_config, &strategy);

        let obj = raw_config.as_object().unwrap();
        // oxfmt extensions are removed by finalize_external_options
        assert!(!obj.contains_key("overrides"));
        assert!(!obj.contains_key("ignorePatterns"));
        assert!(!obj.contains_key("experimentalSortImports"));
    }
}

#[cfg(test)]
mod tests_json_deep_merge {
    use super::*;

    #[test]
    fn test_json_deep_merge() {
        use serde_json::json;

        // Primitives: overlay wins
        let base = json!({ "semi": true, "tabWidth": 2 });
        let overlay = json!({ "semi": false });
        let merged = json_deep_merge(base, overlay);
        assert_eq!(merged, json!({ "semi": false, "tabWidth": 2 }));

        // Nested objects: deep merge
        let base = json!({ "experimentalSortImports": { "order": "asc", "ignoreCase": true } });
        let overlay = json!({ "experimentalSortImports": { "order": "desc" } });
        let merged = json_deep_merge(base, overlay);
        assert_eq!(
            merged,
            json!({ "experimentalSortImports": { "order": "desc", "ignoreCase": true } })
        );

        // Null overwrites value (but in practice, None is skipped during serialization)
        let base = json!({ "semi": false, "tabWidth": 4 });
        let overlay = json!({ "semi": null });
        let merged = json_deep_merge(base, overlay);
        assert_eq!(merged, json!({ "semi": null, "tabWidth": 4 }));
    }
}
