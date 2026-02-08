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
/// Offsets have 1 added to them on JS side.
/// So an original range of `[0, 10]` becomes `JsFix { start_plus_one: 1, end_plus_one: 11, text: "..." }`.
///
/// This allows offsets which were originally -1, and they can be stored in `u32`s.
///
/// ESLint's `unicode-bom` rule produces a fix `{ range: [-1, 0], text: "" }` to remove a BOM.
/// This becomes `JsFix { start_plus_one: 0, end_plus_one: 1, text: "" }`.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JsFix {
    pub start_plus_one: u32,
    pub end_plus_one: u32,
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

    let mut illegal_bom_fix_span = None;
    let mut fixes = fixes.into_iter().map(|fix| {
        // `start_plus_one` and `end_plus_one` are original `start` and `end` + 1.
        // If either is 0, it means the fix is before a BOM.
        // This is a very rare case, so handle it in a `#[cold]` function.
        if fix.start_plus_one == 0 || fix.end_plus_one == 0 {
            return convert_bom_fix(fix, span_converter, has_bom, &mut illegal_bom_fix_span);
        }

        // Convert span from UTF-16 to UTF-8.
        // Deduct 1 from `start_plus_one` and `end_plus_one` to get original offsets.
        let start = fix.start_plus_one - 1;
        let end = fix.end_plus_one - 1;
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

    // If any `JsFix` had `start_plus_one == 0` or `end_plus_one == 0`, but the file doesn't have a BOM,
    // the fix is invalid. This is a very rare case, so handle it in a `#[cold]` function.
    if let Some(span) = illegal_bom_fix_span { create_illegal_bom_error(span) } else { res }
}

/// Convert `JsFix` to `Fix` where either `start_plus_one` or `end_plus_one` is 0.
/// This means that the fix starts before the BOM.
///
/// Convert offsets from UTF-16 to UTF-8.
/// * If file has a BOM, adjust 0 offsets manually to be before the BOM.
/// * If file doesn't have a BOM, set `illegal_span` to the span of the fix (without the BOM-adjustment).
///   `convert_and_merge_js_fixes` will return an error in this case.
///
/// This is a very rare case, so handling this is in this separate `#[cold]` function.
#[cold]
fn convert_bom_fix(
    fix: JsFix,
    span_converter: &Utf8ToUtf16,
    has_bom: bool,
    illegal_span: &mut Option<Span>,
) -> Fix {
    // Convert span from UTF-16 to UTF-8.
    // Perform conversion with original offsets (`start_plus_one - 1`, `end_plus_one - 1`).
    // Offsets which are 0 are clamped to 0 for this initial conversion, so the UTF-8 offsets point to start of the file,
    // *after* the BOM.
    let start = fix.start_plus_one.saturating_sub(1);
    let end = fix.end_plus_one.saturating_sub(1);
    let mut span = Span::new(start, end);

    span_converter.convert_span_back(&mut span);

    if has_bom {
        // Adjust offsets which were 0 to be before the BOM
        if fix.start_plus_one == 0 {
            span.start -= BOM_LEN;
        }
        if fix.end_plus_one == 0 {
            span.end -= BOM_LEN;
        }
    } else {
        // File doesn't have a BOM, so this is an invalid fix.
        // Set `illegal_span`. `convert_and_merge_js_fixes` will return an error in this case.
        *illegal_span = Some(span);
    }

    Fix::new(fix.text, span)
}

/// Create an error for an invalid fix which had `start_plus_one` or `end_plus_one` of 0,
/// but in a file which doesn't have a BOM.
///
/// This is a very rare case, so handling this is in this separate `#[cold]` function.
#[cold]
fn create_illegal_bom_error(span: Span) -> Result<Fix, MergeFixesError> {
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
