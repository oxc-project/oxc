//! Language-agnostic fixture-test runtime.
//!
//! Implements the snapshot-test machinery shared by every formatter crate's `tests/fixtures/mod.rs`:
//!
//! - walks up to find an `options.json` and parses it into typed options (a malformed file or an unknown key fails the test)
//! - drives one format pass per option-set × per printWidth (80 + 100)
//! - re-formats each output and pins any mismatch as a `Not idempotent` section
//!   (idempotency violations are tracked in the snapshot, not asserted)
//! - asserts the consumer's fingerprint of the input equals that of the output (lossless contract)
//! - assembles the canonical `==== Input ==== ... ==== Output ==== ...` snapshot
//! - returns the body for the consumer's `insta::assert_snapshot!`
//!
//! Each formatter crate provides language-specific behavior by implementing [`FixtureFormatter`].

use std::{
    env::current_dir,
    fs,
    path::{Path, PathBuf},
};

use oxc_formatter_core::{
    CoreFormatOptions, FormatOptions, IndentStyle, IndentWidth, LineEnding, LineWidth,
};

/// A single `options.json` entry: a JSON object of per-test format options.
type OptionSet = serde_json::Map<String, serde_json::Value>;

/// The value of one `options.json` key.
pub type OptionValue = serde_json::Value;

/// Applies one core option (`printWidth` / `tabWidth` / `useTabs` / `endOfLine`) onto `options`, `false` for any other key.
/// Invalid values are ignored.
fn apply_core_option<O: FormatOptions>(options: &mut O, key: &str, value: &OptionValue) -> bool {
    let mut core = CoreFormatOptions {
        indent_style: options.indent_style(),
        indent_width: options.indent_width(),
        line_width: options.line_width(),
        line_ending: options.line_ending(),
    };

    match key {
        "printWidth" => {
            if let Some(width) = value
                .as_u64()
                .and_then(|n| u16::try_from(n).ok())
                .and_then(|n| LineWidth::try_from(n).ok())
            {
                core.line_width = width;
            }
        }
        "tabWidth" => {
            if let Some(width) = value
                .as_u64()
                .and_then(|n| u8::try_from(n).ok())
                .and_then(|n| IndentWidth::try_from(n).ok())
            {
                core.indent_width = width;
            }
        }
        "useTabs" => {
            if let Some(b) = value.as_bool() {
                core.indent_style = if b { IndentStyle::Tab } else { IndentStyle::Space };
            }
        }
        "endOfLine" => {
            if let Some(s) = value.as_str() {
                core.line_ending = match s {
                    "lf" => LineEnding::Lf,
                    "crlf" => LineEnding::Crlf,
                    "cr" => LineEnding::Cr,
                    _ => LineEnding::default(),
                };
            }
        }
        _ => return false,
    }

    options.apply_core(core);
    true
}

/// Builds typed options from one `options.json` entry: the core keys here, the rest by [`FixtureFormatter::apply_option`].
/// A key neither takes fails the test, so a typo'd or unmapped key never formats silently with the default.
fn parse_options<F: FixtureFormatter>(json: &OptionSet) -> F::Options {
    let mut options = F::Options::default();
    for (key, value) in json {
        assert!(
            apply_core_option(&mut options, key, value)
                || F::apply_option(&mut options, key, value),
            "unknown option `{key}` in `options.json`"
        );
    }
    options
}

/// Per-language hook for the fixture harness.
pub trait FixtureFormatter {
    /// The typed format-option struct for this language.
    type Options: FormatOptions + Default + Clone;

    /// Source-derived value that formatting must leave unchanged;
    /// the harness compares the input's against the output's.
    /// The harness knows nothing about what it captures, the consumer does
    /// (comment counts, an AST fingerprint, ...).
    type Fingerprint: PartialEq + std::fmt::Debug;

    /// Applies one language-specific `options.json` entry onto `options`, `false` when `key` is not this language's.
    /// The harness applies the core ones itself, and fails the test on a key neither takes.
    fn apply_option(options: &mut Self::Options, key: &str, value: &OptionValue) -> bool;

    /// Format `source` (the contents of the fixture file at `path`) using `options`.
    /// `path` is passed for variant detection (e.g. `.json` vs `.jsonc`).
    fn format(source: &str, path: &Path, options: &Self::Options) -> String;

    /// [`Self::Fingerprint`] of `source`.
    /// Take it from the formatter's own parse (`parse_for_format`), so no test re-implements the parser setup.
    fn fingerprint(source: &str, path: &Path, options: &Self::Options) -> Self::Fingerprint;
}

/// Resolves format options for `test_file` by walking up the directory tree looking for `options.json`.
/// Stops at the first one found, or at the `fixtures` directory, whichever comes first.
///
/// Returns a single empty option-set when no file is found.
///
/// # Panics
/// Panics when the `options.json` found cannot be read or is not an array of objects.
fn resolve_options(test_file: &Path) -> Vec<OptionSet> {
    let mut current_dir = test_file.parent();

    while let Some(dir) = current_dir {
        let options_file = dir.join("options.json");
        if options_file.exists() {
            let content = fs::read_to_string(&options_file)
                .unwrap_or_else(|err| panic!("cannot read {}: {err}", options_file.display()));
            return serde_json::from_str(&content)
                .unwrap_or_else(|err| panic!("invalid {}: {err}", options_file.display()));
        }

        if dir.ends_with("fixtures") {
            break;
        }

        current_dir = dir.parent();
    }

    vec![serde_json::Map::new()]
}

/// Renders an `OptionSet` as `{ key: value, ... }` for the snapshot header.
fn format_options_display(json: &OptionSet) -> String {
    if json.is_empty() {
        return "{}".to_string();
    }
    let mut parts: Vec<_> = json.iter().map(|(k, v)| format!("{k}: {v}")).collect();
    parts.sort();
    format!("{{ {} }}", parts.join(", "))
}

/// Snapshot-body invariant: the configured `endOfLine` is applied to every output line break.
/// The snapshot itself cannot pin this
/// (insta normalizes line endings when storing, and `.gitattributes eol=crlf` re-normalizes the `crlf/` fixtures on checkout),
/// so assert it before the output is embedded.
/// This doubles as the per-crate `endOfLine` option-plumbing check;
/// the conversion itself is pinned by `it_converts_line_endings` in `oxc_formatter_core`
/// (and the builders' debug_asserts keep raw `\r` out of `Text`/`Token`, so a stray `\r` can only come from the printer).
fn assert_line_ending_applied(option_json: &OptionSet, formatted: &str, path: &Path) {
    let Some(eol) = option_json.get("endOfLine").and_then(serde_json::Value::as_str) else {
        return;
    };
    let bytes = formatted.as_bytes();
    let ok = match eol {
        "crlf" => bytes.iter().enumerate().all(|(i, &b)| match b {
            b'\n' => i > 0 && bytes[i - 1] == b'\r',
            b'\r' => bytes.get(i + 1) == Some(&b'\n'),
            _ => true,
        }),
        "cr" => !bytes.contains(&b'\n'),
        _ => !bytes.contains(&b'\r'),
    };
    assert!(ok, "endOfLine: {eol:?} not applied to every line break in {}", path.display());
}

/// Generates the canonical snapshot body for `path`/`source_text`.
///
/// For each resolved option-set,
/// exercises Prettier's default (`printWidth: 80`) and oxc's default (`LineWidth::default()`).
/// If the option-set pins a non-default width, that variant is emitted first, giving 3 rows total;
/// pinning to one of the defaults is treated as a no-op to avoid emitting an identical duplicate row.
fn generate_snapshot<F: FixtureFormatter>(path: &Path, source_text: &str) -> String {
    let option_sets = resolve_options(path);

    let mut snapshot = String::new();
    snapshot.push_str("==================== Input ====================\n");
    snapshot.push_str(source_text);
    snapshot.push('\n');

    snapshot.push_str("==================== Output ====================\n");

    let option_sets = option_sets.into_iter().flat_map(|original| {
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

        let options = parse_options::<F>(&option_json);
        let formatted = F::format(source_text, path, &options);
        assert_line_ending_applied(&option_json, &formatted, path);
        // Lossless contract (`FORMATTER_POLICY.md`, reason (2)): formatting drops nothing the user wrote.
        // What is measured is the consumer's `Fingerprint`.
        // Unlike idempotency this is asserted, not pinned: a loss is inadmissible whatever the snapshot says.
        assert_eq!(
            F::fingerprint(source_text, path, &options),
            F::fingerprint(&formatted, path, &options),
            "fingerprint changed by formatting {} with {options_line}\n  output:\n{formatted}",
            path.display(),
        );

        snapshot.push_str(&formatted);
        snapshot.push('\n');

        // Idempotency: formatting the formatter's own output must be a no-op.
        // A violation is pinned as an extra section rather than asserted,
        // so known non-idempotent fixtures stay visible in their own snapshots
        // (and fixing one shows up as this section disappearing).
        let reformatted = F::format(&formatted, path, &options);
        assert_line_ending_applied(&option_json, &reformatted, path);
        if reformatted != formatted {
            snapshot.push_str("---------- Not idempotent (second pass) ----------\n");
            snapshot.push_str(&reformatted);
            snapshot.push('\n');
        }
    }

    snapshot.push_str("===================== End =====================\n");

    snapshot
}

/// Materialized snapshot for `insta::assert_snapshot!`.
///
/// `insta` records the call site of the macro into the snapshot header (`source:` / `assertion_line:`).
/// We deliberately do NOT call the macro here so each consumer crate can invoke it from its own `tests/fixtures/mod.rs`,
/// keeping the `source:` path stable when the harness is shared across crates.
/// This matters in CI where `INSTA_REQUIRE_FULL_MATCH=1` makes the header part of the equality check.
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
