use std::path::Path;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::core::utils;

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

    /// Sort import statements.
    ///
    /// Using the similar algorithm as [eslint-plugin-perfectionist/sort-imports](https://perfectionist.dev/rules/sort-imports).
    /// For details, see each field's documentation.
    ///
    /// - Default: Disabled
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(alias = "experimentalSortImports")]
    pub sort_imports: Option<SortImportsConfig>,

    /// Sort `package.json` keys.
    ///
    /// The algorithm is NOT compatible with [prettier-plugin-sort-packagejson](https://github.com/matzkoh/prettier-plugin-packagejson).
    /// But we believe it is clearer and easier to navigate.
    /// For details, see each field's documentation.
    ///
    /// - Default: `true`
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(alias = "experimentalSortPackageJson")]
    pub sort_package_json: Option<SortPackageJsonUserConfig>,

    /// Sort Tailwind CSS classes.
    ///
    /// Using the same algorithm as [prettier-plugin-tailwindcss](https://github.com/tailwindlabs/prettier-plugin-tailwindcss).
    /// Option names omit the `tailwind` prefix used in the original plugin (e.g., `config` instead of `tailwindConfig`).
    /// For details, see each field's documentation.
    ///
    /// - Default: Disabled
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(alias = "experimentalTailwindcss")]
    pub sort_tailwindcss: Option<SortTailwindcssConfig>,
}

impl FormatConfig {
    /// Resolve relative tailwind paths (`config`, `stylesheet`) to absolute paths.
    /// Otherwise, the plugin tries to resolve the Prettier's configuration file, not Oxfmt's.
    /// <https://github.com/tailwindlabs/prettier-plugin-tailwindcss/blob/125a8bc77639529a5a0c7e4e8a02174d7ed2d70b/src/config.ts#L50-L54>
    pub fn resolve_tailwind_paths(&mut self, base_dir: &Path) {
        let Some(ref mut tw) = self.sort_tailwindcss else {
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
    /// - `side_effect_style` — Side effect style imports.
    /// - `side_effect` — Side effect imports.
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
    /// - `side_effect` — Side effect imports.
    /// - `type` — TypeScript type imports.
    /// - `value` — Value imports.
    /// - `default` — Imports containing the default specifier.
    /// - `wildcard` — Imports containing the wildcard (`* as`) specifier.
    /// - `named` — Imports containing at least one named specifier.
    ///
    /// - Default: See below
    /// ```json
    /// [
    ///   "builtin",
    ///   "external",
    ///   ["internal", "subpath"],
    ///   ["parent", "sibling", "index"],
    ///   "style",
    ///   "unknown"
    /// ]
    /// ```
    ///
    /// Also, you can override the global `newlinesBetween` setting for specific group boundaries
    /// by including a `{ "newlinesBetween": boolean }` marker object in the `groups` list at the desired position.
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
    pub(super) fn into_vec(self) -> Vec<String> {
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
    /// Possible values: `"type"`, `"side_effect_style"`, `"side_effect"`, `"style"`, `"index"`,
    /// `"sibling"`, `"parent"`, `"subpath"`, `"internal"`, `"builtin"`, `"external"`, `"import"`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selector: Option<String>,
    /// Modifiers to match the import characteristics.
    /// All specified modifiers must be present (AND logic).
    ///
    /// Possible values: `"side_effect"`, `"type"`, `"value"`, `"default"`, `"wildcard"`, `"named"`
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
    pub(super) fn to_sort_options(&self) -> sort_package_json::SortOptions {
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
pub struct SortTailwindcssConfig {
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
