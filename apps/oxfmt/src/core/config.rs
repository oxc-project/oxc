use std::path::{Path, PathBuf};

use editorconfig_parser::{
    EditorConfig, EditorConfigProperties, EditorConfigProperty, EndOfLine, IndentStyle,
    MaxLineLength,
};
use oxc_toml::Options as TomlFormatterOptions;
use serde_json::Value;

use oxc_formatter::{
    FormatOptions,
    oxfmtrc::{EndOfLineConfig, OxfmtOptions, Oxfmtrc},
};

use super::{FormatFileStrategy, utils};

/// Resolve config file path from cwd and optional explicit path.
pub fn resolve_oxfmtrc_path(cwd: &Path, config_path: Option<&Path>) -> Option<PathBuf> {
    // If `--config` is explicitly specified, use that path
    if let Some(config_path) = config_path {
        return Some(if config_path.is_absolute() {
            config_path.to_path_buf()
        } else {
            cwd.join(config_path)
        });
    }

    // If `--config` is not specified, search the nearest config file from cwd upwards
    // Support both `.json` and `.jsonc`, but prefer `.json` if both exist
    cwd.ancestors().find_map(|dir| {
        for filename in [".oxfmtrc.json", ".oxfmtrc.jsonc"] {
            let config_path = dir.join(filename);
            if config_path.exists() {
                return Some(config_path);
            }
        }
        None
    })
}

pub fn resolve_editorconfig_path(cwd: &Path) -> Option<PathBuf> {
    // Search the nearest `.editorconfig` from cwd upwards
    cwd.ancestors().map(|dir| dir.join(".editorconfig")).find(|p| p.exists())
}

// ---

/// Resolved options for each file type.
/// Each variant contains only the options needed for that formatter.
pub enum ResolvedOptions {
    /// For JS/TS files formatted by oxc_formatter.
    OxcFormatter {
        format_options: FormatOptions,
        /// For embedded language formatting (e.g., CSS in template literals)
        external_options: Value,
        insert_final_newline: bool,
    },
    /// For TOML files.
    OxfmtToml { toml_options: TomlFormatterOptions, insert_final_newline: bool },
    /// For non-JS files formatted by external formatter (Prettier).
    #[cfg(feature = "napi")]
    ExternalFormatter { external_options: Value, insert_final_newline: bool },
    /// For `package.json` files: optionally sorted then formatted.
    #[cfg(feature = "napi")]
    ExternalFormatterPackageJson {
        external_options: Value,
        sort_package_json: bool,
        insert_final_newline: bool,
    },
}

/// Configuration resolver that derives all config values from a single `serde_json::Value`.
///
/// Priority order: `Oxfmtrc::default()` → `.editorconfig` → user's `.oxfmtrc`
pub struct ConfigResolver {
    /// User's raw config as JSON value.
    /// It contains every possible field, even those not recognized by `Oxfmtrc`.
    /// e.g. `printWidth`: recognized by both `Oxfmtrc` and Prettier
    /// e.g. `vueIndentScriptAndStyle`: not recognized by `Oxfmtrc`, but used by Prettier
    /// e.g. `svelteSortAttributes`: not recognized by Prettier by default
    raw_config: Value,
    /// Parsed `.editorconfig`, if any.
    editorconfig: Option<EditorConfig>,
    /// Cached parsed options after validation.
    /// Used to avoid re-parsing during per-file resolution, if `.editorconfig` is not used.
    /// NOTE: Currently, only `.editorconfig` provides per-file overrides, `.oxfmtrc` does not.
    cached_options: Option<(FormatOptions, OxfmtOptions, Value)>,
}

impl ConfigResolver {
    /// Create a new resolver from a raw JSON config value.
    #[cfg(feature = "napi")]
    pub fn from_value(raw_config: Value) -> Self {
        Self { raw_config, editorconfig: None, cached_options: None }
    }

    /// Create a resolver by loading config from a file path.
    ///
    /// # Errors
    /// Returns error if:
    /// - Config file is specified but not found or invalid
    /// - Config file parsing fails
    pub fn from_config_paths(
        cwd: &Path,
        oxfmtrc_path: Option<&Path>,
        editorconfig_path: Option<&Path>,
    ) -> Result<Self, String> {
        // Read and parse config file, or use empty JSON if not found
        let json_string = match oxfmtrc_path {
            Some(path) => {
                let mut json_string = utils::read_to_string(path)
                    // Do not include OS error, it differs between platforms
                    .map_err(|_| format!("Failed to read {}: File not found", path.display()))?;
                // Strip comments (JSONC support)
                json_strip_comments::strip(&mut json_string).map_err(|err| {
                    format!("Failed to strip comments from {}: {err}", path.display())
                })?;
                json_string
            }
            None => "{}".to_string(),
        };

        // Parse as raw JSON value
        let raw_config: Value = serde_json::from_str(&json_string)
            .map_err(|err| format!("Failed to parse config: {err}"))?;

        let editorconfig = match editorconfig_path {
            Some(path) => {
                let str = utils::read_to_string(path)
                    .map_err(|_| format!("Failed to read {}: File not found", path.display()))?;

                // Use the directory containing `.editorconfig` as the base, not the CLI's cwd.
                // This ensures patterns like `[src/*.ts]` are resolved relative to where `.editorconfig` is located.
                Some(EditorConfig::parse(&str).with_cwd(path.parent().unwrap_or(cwd)))
            }
            None => None,
        };

        Ok(Self { raw_config, editorconfig, cached_options: None })
    }

    /// Validate config and return ignore patterns for file walking.
    ///
    /// Validated options are cached for fast path resolution.
    /// See also [`ConfigResolver::resolve_with_overrides`] for per-file overrides.
    ///
    /// # Errors
    /// Returns error if config deserialization fails.
    pub fn build_and_validate(&mut self) -> Result<Vec<String>, String> {
        let mut oxfmtrc: Oxfmtrc = serde_json::from_value(self.raw_config.clone())
            .map_err(|err| format!("Failed to deserialize Oxfmtrc: {err}"))?;

        // If `.editorconfig` is used, apply its root section first
        // If there are per-file overrides, they will be applied during `resolve()`
        if let Some(editorconfig) = &self.editorconfig
            && let Some(props) =
                editorconfig.sections().iter().find(|s| s.name == "*").map(|s| &s.properties)
        {
            apply_editorconfig(&mut oxfmtrc, props);
        }

        // If not specified, default options are resolved here
        let (format_options, oxfmt_options) = oxfmtrc
            .into_options()
            .map_err(|err| format!("Failed to parse configuration.\n{err}"))?;

        // Apply our resolved defaults to Prettier options too
        // e.g. set `printWidth: 100` if not specified (= Prettier default: 80)
        let mut external_options = self.raw_config.clone();
        Oxfmtrc::populate_prettier_config(&format_options, &mut external_options);

        let ignore_patterns_clone = oxfmt_options.ignore_patterns.clone();

        // NOTE: Save cache for fast path: no per-file overrides
        self.cached_options = Some((format_options, oxfmt_options, external_options));

        Ok(ignore_patterns_clone)
    }

    /// Resolve format options for a specific file.
    pub fn resolve(&self, strategy: &FormatFileStrategy) -> ResolvedOptions {
        let (format_options, oxfmt_options, external_options) = if let Some(editorconfig) =
            &self.editorconfig
            && let Some(props) = get_editorconfig_overrides(editorconfig, strategy.path())
        {
            self.resolve_with_overrides(&props)
        } else {
            // Fast path: no per-file overrides
            // Either:
            // - `.editorconfig` is NOT used
            // - or used but per-file overrides do NOT exist for this file
            self.cached_options
                .clone()
                .expect("`build_and_validate()` must be called before `resolve()`")
        };

        let insert_final_newline = oxfmt_options.insert_final_newline;

        match strategy {
            FormatFileStrategy::OxcFormatter { .. } => ResolvedOptions::OxcFormatter {
                format_options,
                external_options,
                insert_final_newline,
            },
            FormatFileStrategy::OxfmtToml { .. } => ResolvedOptions::OxfmtToml {
                toml_options: build_toml_options(&format_options),
                insert_final_newline,
            },
            #[cfg(feature = "napi")]
            FormatFileStrategy::ExternalFormatter { .. } => {
                ResolvedOptions::ExternalFormatter { external_options, insert_final_newline }
            }
            #[cfg(feature = "napi")]
            FormatFileStrategy::ExternalFormatterPackageJson { .. } => {
                ResolvedOptions::ExternalFormatterPackageJson {
                    external_options,
                    sort_package_json: oxfmt_options.sort_package_json,
                    insert_final_newline,
                }
            }
            #[cfg(not(feature = "napi"))]
            _ => {
                unreachable!("If `napi` feature is disabled, this should not be passed here")
            }
        }
    }

    /// Resolve format options for a specific file with `.editorconfig` overrides.
    /// This is the slow path, for fast path, see [`ConfigResolver::build_and_validate`].
    fn resolve_with_overrides(
        &self,
        props: &EditorConfigProperties,
    ) -> (FormatOptions, OxfmtOptions, Value) {
        let mut oxfmtrc: Oxfmtrc = serde_json::from_value(self.raw_config.clone())
            .expect("`build_and_validate()` should catch this before `resolve()`");

        apply_editorconfig(&mut oxfmtrc, props);

        let (format_options, oxfmt_options) = oxfmtrc
            .into_options()
            .expect("If this fails, there is an issue with editorconfig insertion above");

        // Apply our defaults for Prettier options too
        // e.g. set `printWidth: 100` if not specified (= Prettier default: 80)
        let mut external_options = self.raw_config.clone();
        Oxfmtrc::populate_prettier_config(&format_options, &mut external_options);

        (format_options, oxfmt_options, external_options)
    }
}

// ---

/// Check if `.editorconfig` has per-file overrides for this path.
///
/// Returns `Some(props)` if the resolved properties differ from the root `[*]` section.
/// Returns `None` if no overrides.
///
/// Currently, only the following properties are considered for overrides:
/// - max_line_length
/// - end_of_line
/// - indent_style
/// - indent_size
/// - insert_final_newline
fn get_editorconfig_overrides(
    editorconfig: &EditorConfig,
    path: &Path,
) -> Option<EditorConfigProperties> {
    let sections = editorconfig.sections();

    // No sections, or only root `[*]` section → no overrides
    if sections.is_empty() || matches!(sections, [s] if s.name == "*") {
        return None;
    }

    let resolved = editorconfig.resolve(path);

    // Get the root `[*]` section properties
    let root_props = sections.iter().find(|s| s.name == "*").map(|s| &s.properties);

    // Compare only the properties we care about
    let has_overrides = match root_props {
        Some(root) => {
            resolved.max_line_length != root.max_line_length
                || resolved.end_of_line != root.end_of_line
                || resolved.indent_style != root.indent_style
                || resolved.indent_size != root.indent_size
                || resolved.insert_final_newline != root.insert_final_newline
        }
        // No `[*]` section means any resolved property is an override
        None => {
            resolved.max_line_length != EditorConfigProperty::Unset
                || resolved.end_of_line != EditorConfigProperty::Unset
                || resolved.indent_style != EditorConfigProperty::Unset
                || resolved.indent_size != EditorConfigProperty::Unset
                || resolved.insert_final_newline != EditorConfigProperty::Unset
        }
    };

    if has_overrides { Some(resolved) } else { None }
}

/// Apply `.editorconfig` properties to `Oxfmtrc`.
///
/// Only applies values that are not already set in oxfmtrc.
/// Priority: oxfmtrc default < editorconfig < user's oxfmtrc
///
/// Only properties checked by [`get_editorconfig_overrides`] are applied here.
fn apply_editorconfig(oxfmtrc: &mut Oxfmtrc, props: &EditorConfigProperties) {
    #[expect(clippy::cast_possible_truncation)]
    if oxfmtrc.print_width.is_none()
        && let EditorConfigProperty::Value(MaxLineLength::Number(v)) = props.max_line_length
    {
        oxfmtrc.print_width = Some(v as u16);
    }

    if oxfmtrc.end_of_line.is_none()
        && let EditorConfigProperty::Value(eol) = props.end_of_line
    {
        oxfmtrc.end_of_line = Some(match eol {
            EndOfLine::Lf => EndOfLineConfig::Lf,
            EndOfLine::Cr => EndOfLineConfig::Cr,
            EndOfLine::Crlf => EndOfLineConfig::Crlf,
        });
    }

    if oxfmtrc.use_tabs.is_none()
        && let EditorConfigProperty::Value(style) = props.indent_style
    {
        oxfmtrc.use_tabs = Some(match style {
            IndentStyle::Tab => true,
            IndentStyle::Space => false,
        });
    }

    #[expect(clippy::cast_possible_truncation)]
    if oxfmtrc.tab_width.is_none()
        && let EditorConfigProperty::Value(size) = props.indent_size
    {
        oxfmtrc.tab_width = Some(size as u8);
    }

    if oxfmtrc.insert_final_newline.is_none()
        && let EditorConfigProperty::Value(v) = props.insert_final_newline
    {
        oxfmtrc.insert_final_newline = Some(v);
    }
}

// ---

/// Build `toml` formatter options.
/// The same as `prettier-plugin-toml`.
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
        // Align with `oxc_formatter` and Prettier default
        trailing_newline: true,
        ..Default::default()
    }
}
