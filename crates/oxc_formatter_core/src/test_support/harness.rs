//! Language-agnostic fixture-test runtime.
//!
//! Implements the snapshot-test machinery shared by every formatter crate's
//! `tests/fixtures/mod.rs`:
//!
//! - walks up to find an `options.json` and parses it into `OptionSet`s
//! - drives one format pass per option-set × per printWidth (80 + 100)
//! - assembles the canonical `==== Input ==== ... ==== Output ==== ...` snapshot
//! - hands the result to `insta::assert_snapshot!`
//!
//! Each formatter crate provides language-specific behavior by implementing
//! [`FixtureFormatter`].

use std::{
    env::current_dir,
    fs,
    path::{Path, PathBuf},
};

use crate::LineWidth;

/// A single `options.json` entry: a JSON object of per-test format options.
pub type OptionSet = serde_json::Map<String, serde_json::Value>;

/// Per-language hook for the fixture harness.
pub trait FixtureFormatter {
    /// The typed format-option struct for this language.
    type Options: Clone;

    /// Build typed options from a parsed `options.json` fragment.
    fn parse_options(json: &OptionSet) -> Self::Options;

    /// Format `source` (the contents of the fixture file at `path`) using `options`.
    /// `path` is passed for variant detection (e.g. `.json` vs `.jsonc`).
    fn format(source: &str, path: &Path, options: &Self::Options) -> String;
}

/// Resolves format options for `test_file` by walking up the directory tree
/// looking for `options.json`. Stops at the first one found, or at the `fixtures`
/// directory, whichever comes first.
///
/// Returns a single empty option-set when no file is found.
#[must_use]
pub fn resolve_options(test_file: &Path) -> Vec<OptionSet> {
    let mut current_dir = test_file.parent();

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

        if dir.ends_with("fixtures") {
            break;
        }

        current_dir = dir.parent();
    }

    vec![serde_json::Map::new()]
}

/// Renders an `OptionSet` as `{ key: value, ... }` for the snapshot header.
#[must_use]
pub fn format_options_display(json: &OptionSet) -> String {
    if json.is_empty() {
        return "{}".to_string();
    }
    let mut parts: Vec<_> = json.iter().map(|(k, v)| format!("{k}: {v}")).collect();
    parts.sort();
    format!("{{ {} }}", parts.join(", "))
}

/// Generates the canonical snapshot body for `path`/`source_text`.
///
/// For each resolved option-set, exercises Prettier's default (`printWidth: 80`) and
/// oxc's default (`LineWidth::default()`). If the option-set pins a non-default width,
/// that variant is emitted first, giving 3 rows total; pinning to one of the defaults
/// is treated as a no-op to avoid emitting an identical duplicate row.
fn generate_snapshot<F: FixtureFormatter>(path: &Path, source_text: &str) -> String {
    let option_sets = resolve_options(path);

    let mut snapshot = String::new();
    snapshot.push_str("==================== Input ====================\n");
    snapshot.push_str(source_text);
    snapshot.push('\n');

    snapshot.push_str("==================== Output ====================\n");

    let option_sets = option_sets.into_iter().flat_map(|original| {
        // Always exercise Prettier's default (80) and oxc's default
        // (`LineWidth::default()`). If the original pins a non-default `printWidth`
        // we also emit a row for it; pinning to 80 or 100 is a no-op since the
        // default rows already cover that width.
        const PRETTIER_DEFAULT_WIDTH: u64 = 80;
        let oxc_default_width = u64::from(LineWidth::default().value());

        let pinned = original.get("printWidth").and_then(serde_json::Value::as_u64);

        let mut rows = Vec::with_capacity(3);
        if let Some(w) = pinned
            && w != PRETTIER_DEFAULT_WIDTH
            && w != oxc_default_width
        {
            rows.push(original.clone());
        }

        let mut with_prettier_width = original.clone();
        with_prettier_width.insert(
            "printWidth".to_string(),
            serde_json::Value::Number(PRETTIER_DEFAULT_WIDTH.into()),
        );
        rows.push(with_prettier_width);

        let mut with_oxc_width = original;
        with_oxc_width
            .insert("printWidth".to_string(), serde_json::Value::Number(oxc_default_width.into()));
        rows.push(with_oxc_width);

        rows
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

        let options = F::parse_options(&option_json);
        let formatted = F::format(source_text, path, &options);
        snapshot.push_str(&formatted);
        snapshot.push('\n');
    }

    snapshot.push_str("===================== End =====================\n");

    snapshot
}

/// Materialized snapshot for `insta::assert_snapshot!`.
///
/// `insta` records the call site of the macro into the snapshot header (`source:` /
/// `assertion_line:`). We deliberately do NOT call the macro here so each consumer
/// crate can invoke it from its own `tests/fixtures/mod.rs`, keeping the `source:`
/// path stable when the harness is shared across crates. This matters in CI where
/// `INSTA_REQUIRE_FULL_MATCH=1` makes the header part of the equality check.
pub struct FixtureSnapshot {
    /// Snapshot body — feed to `insta::assert_snapshot!`.
    pub body: String,
    /// Directory the `.snap` file should live in (sibling to the fixture).
    pub path: PathBuf,
    /// Base filename for the `.snap` file (matches the fixture's filename).
    pub name: String,
}

/// Reads `path` and builds its [`FixtureSnapshot`].
///
/// # Panics
/// Panics if the fixture cannot be read.
pub fn build_fixture_snapshot<F: FixtureFormatter>(path: &Path) -> FixtureSnapshot {
    let source_text = fs::read_to_string(path).unwrap();
    let body = generate_snapshot::<F>(path, &source_text);
    let snap_dir = current_dir().unwrap().join(path.parent().unwrap());
    let snap_name = path.file_name().unwrap().to_string_lossy().into_owned();
    FixtureSnapshot { body, path: snap_dir, name: snap_name }
}
