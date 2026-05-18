use std::{fmt::Debug, sync::Arc};

use serde::{Deserialize, Serialize, Serializer, ser::SerializeMap};

use oxc_allocator::Allocator;
use oxc_ast_visit::utf8_to_utf16::Utf8ToUtf16;
use oxc_span::Span;

use crate::{
    config::{OxlintEnv, OxlintGlobals},
    context::ContextHost,
    fixer::{CompositeFix, Fix, MergeFixesError},
};

pub type ExternalLinterCreateWorkspaceCb =
    Arc<Box<dyn Fn(String) -> Result<(), String> + Send + Sync>>;

pub type ExternalLinterDestroyWorkspaceCb =
    Arc<Box<dyn Fn(String) -> Result<(), String> + Send + Sync>>;

pub type ExternalLinterLoadPluginCb = Arc<
    Box<
        dyn Fn(
                // File URL to load plugin from
                String,
                // Plugin name (either alias or package name).
                // If is package name, it is pre-normalized.
                Option<String>,
                // `true` if plugin name is an alias (takes priority over name that plugin defines itself)
                bool,
                // Workspace URI (e.g. `file:///path/to/workspace`).
                // `None` in CLI mode (single workspace), `Some` in LSP mode.
                Option<String>,
            ) -> Result<LoadPluginResult, String>
            + Send
            + Sync,
    >,
>;

pub type ExternalLinterSetupRuleConfigsCb =
    Arc<Box<dyn Fn(String) -> Result<(), String> + Send + Sync>>;

pub type ExternalLinterLintFileCb = Arc<
    Box<
        dyn Fn(
                // File path of file to lint
                String,
                // Rule IDs
                Vec<u32>,
                // Options IDs
                Vec<u32>,
                // Settings JSON
                String,
                // Globals JSON
                String,
                // Workspace URI (e.g. `file:///path/to/workspace`).
                // `None` in CLI mode (single workspace), `Some` in LSP mode.
                Option<String>,
                // Allocator
                &Allocator,
            ) -> Result<Vec<LintFileResult>, String>
            + Sync
            + Send,
    >,
>;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoadPluginResult {
    pub name: String,
    pub offset: usize,
    pub rule_names: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LintFileResult {
    pub rule_index: u32,
    pub message: String,
    pub start: u32,
    pub end: u32,
    pub fixes: Option<Vec<JsFix>>,
    pub suggestions: Option<Vec<JsSuggestion>>,
}

/// Fix in form sent from JS to Rust.
///
/// `start` and `end` can be -1, so these fields are `i64`s instead of `u32`s, to accommodate both negative numbers
/// and the full range of `u32`.
///
/// ESLint's `unicode-bom` rule produces a fix `{ range: [-1, 0], text: "" }` to remove a BOM.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JsFix {
    pub start: i64,
    pub end: i64,
    pub text: String,
}

/// Suggestion in form sent from JS to Rust.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JsSuggestion {
    pub message: String,
    pub fixes: Vec<JsFix>,
}

const BOM: &str = "\u{feff}";
#[expect(clippy::cast_possible_truncation)]
const BOM_LEN: u32 = BOM.len() as u32;

/// Convert a `Vec<JsFix>` to a single [`Fix`], including converting spans from UTF-16 to UTF-8.
pub fn convert_and_merge_js_fixes(
    fixes: Vec<JsFix>,
    source_text: &str,
    span_converter: &Utf8ToUtf16,
    has_bom: bool,
) -> Result<Fix, MergeFixesError> {
    // JS should send `None` instead of `Some([])`
    debug_assert!(!fixes.is_empty());

    let is_single = fixes.len() == 1;

    let mut invalid_span = None;
    let mut fixes = fixes.into_iter().map(|fix| {
        // `start` and `end` can be `-1` to mean before a BOM.
        // We also need to handle values which are out of range of `u32`.
        // These are very rare cases, so handle them in a `#[cold]` function.
        let mut negative_or_out_of_range_offset = false;
        let start = u32::try_from(fix.start).unwrap_or_else(|_| {
            negative_or_out_of_range_offset = true;
            0
        });
        let end = u32::try_from(fix.end).unwrap_or_else(|_| {
            negative_or_out_of_range_offset = true;
            0
        });

        if negative_or_out_of_range_offset {
            return convert_negative_or_out_of_range_fix(
                fix,
                span_converter,
                has_bom,
                &mut invalid_span,
            );
        }

        // Convert span from UTF-16 to UTF-8.
        let mut span = Span::new(start, end);
        span_converter.convert_span_back(&mut span);

        Fix::new(fix.text, span)
    });

    let res = if is_single {
        #[expect(clippy::missing_panics_doc, reason = "infallible")]
        let fix = fixes.next().unwrap();

        // Same validation logic as in `CompositeFix::merge_fixes_fallible`.
        // We use `source_text.get(start, end).is_none()` instead of just `end > source_text.len()`
        // to also check that `start` and `end` are on UTF-8 character boundaries.
        // It's possible for offsets not to be on UTF-8 character boundaries if the original UTF-16 offset
        // was in middle of a surrogate pair (2 x UTF-16 characters, 1 x 4-byte UTF-8 character).
        if fix.span.start > fix.span.end {
            Err(MergeFixesError::NegativeRange(fix.span))
        } else if source_text.get(fix.span.start as usize..fix.span.end as usize).is_none() {
            // `end..end` matches the error from `CompositeFix::merge_fixes_fallible`
            Err(MergeFixesError::InvalidRange(fix.span.end, fix.span.end))
        } else {
            Ok(fix)
        }
    } else {
        CompositeFix::merge_fixes_fallible(fixes.collect(), source_text)
    };

    // If any `JsFix` had invalid `start` or `end`, we need to produce an error.
    // These are very rare cases, so handle them in a `#[cold]` function.
    if let Some(span) = invalid_span { create_invalid_offset_error(span) } else { res }
}

/// Convert `JsFix` to `Fix` where either `start` or `end` is out of range of `u32`.
///
/// This means either:
/// * -1 = before the BOM - valid if file has a BOM, invalid if not.
/// * Any other negative offset = invalid.
/// * Offset > `u32::MAX` = invalid.
///
/// Convert offsets from UTF-16 to UTF-8.
/// * If file has a BOM, adjust -1 offsets manually to be before the BOM.
/// * If file doesn't have a BOM, or offsets are out of range, set `invalid_span` to the span of the fix
///   (without the BOM-adjustment). `convert_and_merge_js_fixes` will return an error.
///
/// -1 and invalid offsets are very rare cases, so handling them is in this separate `#[cold]` function.
#[cold]
fn convert_negative_or_out_of_range_fix(
    fix: JsFix,
    span_converter: &Utf8ToUtf16,
    has_bom: bool,
    invalid_span: &mut Option<Span>,
) -> Fix {
    // Detect if either `start` or `end` is out of range, and convert illegal offsets, or valid -1 offsets to 0
    let mut is_invalid = false;
    let mut convert_offset = |offset| {
        if offset < 0 {
            // Only -1 is valid, and only if file has a BOM
            if offset != -1 || !has_bom {
                is_invalid = true;
            }
            0
        } else if let Ok(offset) = u32::try_from(offset) {
            offset
        } else {
            is_invalid = true;
            0
        }
    };

    let start = convert_offset(fix.start);
    let end = convert_offset(fix.end);

    // Convert offsets from UTF-16 to UTF-8
    let mut span = Span::new(start, end);
    span_converter.convert_span_back(&mut span);

    if is_invalid {
        *invalid_span = Some(span);
    } else {
        // Adjust offsets which were -1 to be before the BOM
        if fix.start == -1 {
            span.start -= BOM_LEN;
        }
        if fix.end == -1 {
            span.end -= BOM_LEN;
        }
    }

    Fix::new(fix.text, span)
}

/// Create an error for a fix which had invalid `start` or `end`.
///
/// This is a very rare case, so handling this is in this separate `#[cold]` function.
#[cold]
fn create_invalid_offset_error(span: Span) -> Result<Fix, MergeFixesError> {
    Err(MergeFixesError::InvalidRange(span.start, span.end))
}

#[derive(Clone)]
pub struct ExternalLinter {
    pub(crate) load_plugin: ExternalLinterLoadPluginCb,
    pub(crate) setup_rule_configs: ExternalLinterSetupRuleConfigsCb,
    pub(crate) lint_file: ExternalLinterLintFileCb,
    pub create_workspace: ExternalLinterCreateWorkspaceCb,
    pub destroy_workspace: ExternalLinterDestroyWorkspaceCb,
}

impl ExternalLinter {
    pub fn new(
        load_plugin: ExternalLinterLoadPluginCb,
        setup_rule_configs: ExternalLinterSetupRuleConfigsCb,
        lint_file: ExternalLinterLintFileCb,
        create_workspace: ExternalLinterCreateWorkspaceCb,
        destroy_workspace: ExternalLinterDestroyWorkspaceCb,
    ) -> Self {
        Self { load_plugin, setup_rule_configs, lint_file, create_workspace, destroy_workspace }
    }
}

impl Debug for ExternalLinter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExternalLinter").finish()
    }
}

/// Struct for serializing globals and envs to send to JS plugins.
///
/// Serializes as `{ "globals": { "React": "readonly" }, "envs": { "browser": true } }`.
/// `envs` only includes the environments that are enabled, so all properties are `true`.
#[derive(Serialize)]
pub struct GlobalsAndEnvs<'c> {
    globals: &'c OxlintGlobals,
    envs: EnabledEnvs<'c>,
}

impl<'c> GlobalsAndEnvs<'c> {
    pub fn new(ctx_host: &'c ContextHost<'_>) -> Self {
        Self { globals: ctx_host.globals(), envs: EnabledEnvs(ctx_host.env()) }
    }
}

struct EnabledEnvs<'c>(&'c OxlintEnv);

impl Serialize for EnabledEnvs<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;

        for env_name in self.0.iter() {
            map.serialize_entry(env_name, &true)?;
        }

        map.end()
    }
}
