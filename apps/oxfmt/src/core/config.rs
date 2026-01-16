use std::path::{Path, PathBuf};

use editorconfig_parser::{
    EditorConfig, EditorConfigProperties, EditorConfigProperty, EndOfLine, IndentStyle,
    MaxLineLength,
};
use serde_json::Value;
use tracing::instrument;

use oxc_formatter::FormatOptions;
use oxc_toml::Options as TomlFormatterOptions;

use super::{
    FormatFileStrategy,
    oxfmtrc::{EndOfLineConfig, OxfmtOptions, Oxfmtrc, populate_prettier_config},
    utils,
};

/// Resolve config file path from cwd and optional explicit path.
pub fn resolve_oxfmtrc_path(cwd: &Path, config_path: Option<&Path>) -> Option<PathBuf> {
    // If `--config` is explicitly specified, use that path
    if let Some(config_path) = config_path {
        return Some(utils::normalize_relative_path(cwd, config_path));
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
#[derive(Debug)]
pub enum ResolvedOptions {
    /// For JS/TS files formatted by oxc_formatter.
    OxcFormatter {
        format_options: Box<FormatOptions>,
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
        sort_package_json: Option<sort_package_json::SortOptions>,
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
    cached_options: Option<(OxfmtOptions, Value)>,
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
    #[instrument(level = "debug", name = "oxfmt::config::from_config_paths", skip_all)]
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
    #[instrument(level = "debug", name = "oxfmt::config::build_and_validate", skip_all)]
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
        let (oxfmt_options, ignore_patterns) = oxfmtrc
            .into_options()
            .map_err(|err| format!("Failed to parse configuration.\n{err}"))?;

        // Apply our resolved defaults to Prettier options too
        // e.g. set `printWidth: 100` if not specified (= Prettier default: 80)
        let mut external_options = self.raw_config.clone();
        populate_prettier_config(&oxfmt_options.format_options, &mut external_options);

        // NOTE: Save cache for fast path: no per-file overrides
        self.cached_options = Some((oxfmt_options, external_options));

        Ok(ignore_patterns)
    }

    /// Resolve format options for a specific file.
    #[instrument(level = "debug", name = "oxfmt::config::resolve", skip_all, fields(path = %strategy.path().display()))]
    pub fn resolve(&self, strategy: &FormatFileStrategy) -> ResolvedOptions {
        let (oxfmt_options, external_options) = if let Some(editorconfig) = &self.editorconfig
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

        #[cfg(feature = "napi")]
        let OxfmtOptions { format_options, toml_options, sort_package_json, insert_final_newline } =
            oxfmt_options;
        #[cfg(not(feature = "napi"))]
        let OxfmtOptions { format_options, toml_options, insert_final_newline, .. } = oxfmt_options;

        match strategy {
            FormatFileStrategy::OxcFormatter { .. } => ResolvedOptions::OxcFormatter {
                format_options: Box::new(format_options),
                external_options,
                insert_final_newline,
            },
            FormatFileStrategy::OxfmtToml { .. } => {
                ResolvedOptions::OxfmtToml { toml_options, insert_final_newline }
            }
            #[cfg(feature = "napi")]
            FormatFileStrategy::ExternalFormatter { .. } => {
                ResolvedOptions::ExternalFormatter { external_options, insert_final_newline }
            }
            #[cfg(feature = "napi")]
            FormatFileStrategy::ExternalFormatterPackageJson { .. } => {
                ResolvedOptions::ExternalFormatterPackageJson {
                    external_options,
                    sort_package_json,
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
    #[instrument(level = "debug", name = "oxfmt::config::resolve_with_overrides", skip_all)]
    fn resolve_with_overrides(&self, props: &EditorConfigProperties) -> (OxfmtOptions, Value) {
        let mut oxfmtrc: Oxfmtrc = serde_json::from_value(self.raw_config.clone())
            .expect("`build_and_validate()` should catch this before `resolve()`");

        apply_editorconfig(&mut oxfmtrc, props);

        let (oxfmt_options, _) = oxfmtrc
            .into_options()
            .expect("If this fails, there is an issue with editorconfig insertion above");

        // Apply our defaults for Prettier options too
        // e.g. set `printWidth: 100` if not specified (= Prettier default: 80)
        let mut external_options = self.raw_config.clone();
        populate_prettier_config(&oxfmt_options.format_options, &mut external_options);

        (oxfmt_options, external_options)
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
    if oxfmtrc.format_config.print_width.is_none()
        && let EditorConfigProperty::Value(MaxLineLength::Number(v)) = props.max_line_length
    {
        oxfmtrc.format_config.print_width = Some(v as u16);
    }

    if oxfmtrc.format_config.end_of_line.is_none()
        && let EditorConfigProperty::Value(eol) = props.end_of_line
    {
        oxfmtrc.format_config.end_of_line = Some(match eol {
            EndOfLine::Lf => EndOfLineConfig::Lf,
            EndOfLine::Cr => EndOfLineConfig::Cr,
            EndOfLine::Crlf => EndOfLineConfig::Crlf,
        });
    }

    if oxfmtrc.format_config.use_tabs.is_none()
        && let EditorConfigProperty::Value(style) = props.indent_style
    {
        oxfmtrc.format_config.use_tabs = Some(match style {
            IndentStyle::Tab => true,
            IndentStyle::Space => false,
        });
    }

    #[expect(clippy::cast_possible_truncation)]
    if oxfmtrc.format_config.tab_width.is_none()
        && let EditorConfigProperty::Value(size) = props.indent_size
    {
        oxfmtrc.format_config.tab_width = Some(size as u8);
    }

    if oxfmtrc.format_config.insert_final_newline.is_none()
        && let EditorConfigProperty::Value(v) = props.insert_final_newline
    {
        oxfmtrc.format_config.insert_final_newline = Some(v);
    }
}
