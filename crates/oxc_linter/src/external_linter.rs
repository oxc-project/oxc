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

pub type ExternalLinterForgetBufferCb = Arc<Box<dyn Fn(u32) -> Result<(), String> + Send + Sync>>;

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
    k: usize,
    load_plugin_on_workers: Box<[ExternalLinterLoadPluginCb]>,
    setup_rule_configs_on_workers: Box<[ExternalLinterSetupRuleConfigsCb]>,
    lint_file_on_workers: Box<[ExternalLinterLintFileCb]>,
    forget_buffer_on_workers: Box<[ExternalLinterForgetBufferCb]>,
    create_workspace_on_workers: Box<[ExternalLinterCreateWorkspaceCb]>,
    destroy_workspace_on_workers: Box<[ExternalLinterDestroyWorkspaceCb]>,
}

impl ExternalLinter {
    /// Create an [`ExternalLinter`] with `K` callback slots (one per JS isolate).
    ///
    /// All slices must have the same non-zero length `K`. `K == 1` is today's
    /// main-thread callback path.
    ///
    /// # Panics
    ///
    /// Panics if any slice is empty or the slice lengths do not match.
    pub fn new(
        load_plugin_on_workers: Box<[ExternalLinterLoadPluginCb]>,
        setup_rule_configs_on_workers: Box<[ExternalLinterSetupRuleConfigsCb]>,
        lint_file_on_workers: Box<[ExternalLinterLintFileCb]>,
        forget_buffer_on_workers: Box<[ExternalLinterForgetBufferCb]>,
        create_workspace_on_workers: Box<[ExternalLinterCreateWorkspaceCb]>,
        destroy_workspace_on_workers: Box<[ExternalLinterDestroyWorkspaceCb]>,
    ) -> Self {
        let k = load_plugin_on_workers.len();
        assert!(k >= 1, "ExternalLinter requires at least one worker");
        assert_eq!(
            setup_rule_configs_on_workers.len(),
            k,
            "setup_rule_configs callback count must equal K"
        );
        assert_eq!(lint_file_on_workers.len(), k, "lint_file callback count must equal K");
        assert_eq!(forget_buffer_on_workers.len(), k, "forget_buffer callback count must equal K");
        assert_eq!(
            create_workspace_on_workers.len(),
            k,
            "create_workspace callback count must equal K"
        );
        assert_eq!(
            destroy_workspace_on_workers.len(),
            k,
            "destroy_workspace callback count must equal K"
        );
        Self {
            k,
            load_plugin_on_workers,
            setup_rule_configs_on_workers,
            lint_file_on_workers,
            forget_buffer_on_workers,
            create_workspace_on_workers,
            destroy_workspace_on_workers,
        }
    }

    /// Load a JS plugin on every worker and return the first worker's result.
    ///
    /// # Errors
    ///
    /// Returns an error if any worker fails to load the plugin, or if workers
    /// register different `name` / `offset` / `rule_names`.
    pub fn load_plugin(
        &self,
        plugin_url: String,
        plugin_name: Option<String>,
        plugin_name_is_alias: bool,
        workspace_uri: Option<String>,
    ) -> Result<LoadPluginResult, String> {
        if self.k == 1 {
            return (self.load_plugin_on_workers[0])(
                plugin_url,
                plugin_name,
                plugin_name_is_alias,
                workspace_uri,
            );
        }
        let first = (self.load_plugin_on_workers[0])(
            plugin_url.clone(),
            plugin_name.clone(),
            plugin_name_is_alias,
            workspace_uri.clone(),
        )?;
        for cb in &self.load_plugin_on_workers[1..self.k - 1] {
            let other = cb(
                plugin_url.clone(),
                plugin_name.clone(),
                plugin_name_is_alias,
                workspace_uri.clone(),
            )?;
            if load_plugin_mismatch(&first, &other) {
                return Err("JS worker rule registration mismatch".to_string());
            }
        }
        let other = (self.load_plugin_on_workers[self.k - 1])(
            plugin_url,
            plugin_name,
            plugin_name_is_alias,
            workspace_uri,
        )?;
        if load_plugin_mismatch(&first, &other) {
            return Err("JS worker rule registration mismatch".to_string());
        }
        Ok(first)
    }

    /// Send rule options JSON to every worker.
    ///
    /// # Errors
    ///
    /// Returns the first worker error, if any.
    pub fn setup_rule_configs(&self, options_json: String) -> Result<(), String> {
        let last = self.k - 1;
        for cb in &self.setup_rule_configs_on_workers[..last] {
            cb(options_json.clone())?;
        }
        (self.setup_rule_configs_on_workers[last])(options_json)
    }

    /// Route `lint_file` to the worker that owns `allocator`'s buffer (`buffer_id % K`).
    ///
    /// # Errors
    ///
    /// Returns an error if the owning worker's callback fails.
    ///
    /// # Panics
    ///
    /// Panics if `allocator` is not a fixed-size arena.
    pub fn lint_file(
        &self,
        file_path: String,
        rule_ids: Vec<u32>,
        options_ids: Vec<u32>,
        settings_json: String,
        globals_json: String,
        workspace_uri: Option<String>,
        allocator: &Allocator,
    ) -> Result<Vec<LintFileResult>, String> {
        assert!(
            allocator.is_fixed_size(),
            "ExternalLinter::lint_file requires a fixed-size allocator"
        );

        #[cfg(all(target_pointer_width = "64", target_endian = "little"))]
        {
            // SAFETY: `is_fixed_size` was checked above.
            let buffer_id = unsafe { allocator.fixed_size_buffer_id() };
            let owner = (buffer_id as usize) % self.k;
            (self.lint_file_on_workers[owner])(
                file_path,
                rule_ids,
                options_ids,
                settings_json,
                globals_json,
                workspace_uri,
                allocator,
            )
        }

        #[cfg(not(all(target_pointer_width = "64", target_endian = "little")))]
        {
            (self.lint_file_on_workers[0])(
                file_path,
                rule_ids,
                options_ids,
                settings_json,
                globals_json,
                workspace_uri,
                allocator,
            )
        }
    }

    /// Drop a cached raw-transfer buffer on the worker that owns `buffer_id`.
    ///
    /// # Errors
    ///
    /// Returns an error if the owning worker's callback fails.
    pub fn forget_buffer(&self, buffer_id: u32) -> Result<(), String> {
        let owner = (buffer_id as usize) % self.k;
        (self.forget_buffer_on_workers[owner])(buffer_id)
    }

    /// Create a JS workspace on every worker.
    ///
    /// # Errors
    ///
    /// Returns the first worker error, if any.
    pub fn create_workspace(&self, workspace_uri: String) -> Result<(), String> {
        let last = self.k - 1;
        for cb in &self.create_workspace_on_workers[..last] {
            cb(workspace_uri.clone())?;
        }
        (self.create_workspace_on_workers[last])(workspace_uri)
    }

    /// Destroy a JS workspace on every worker.
    ///
    /// # Errors
    ///
    /// Returns the first worker error, if any.
    pub fn destroy_workspace(&self, workspace_uri: String) -> Result<(), String> {
        let last = self.k - 1;
        for cb in &self.destroy_workspace_on_workers[..last] {
            cb(workspace_uri.clone())?;
        }
        (self.destroy_workspace_on_workers[last])(workspace_uri)
    }
}

fn load_plugin_mismatch(first: &LoadPluginResult, other: &LoadPluginResult) -> bool {
    other.rule_names != first.rule_names || other.offset != first.offset || other.name != first.name
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

#[cfg(test)]
mod tests {
    use std::sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    };

    use oxc_allocator::Allocator;

    use super::*;

    fn unused_load() -> ExternalLinterLoadPluginCb {
        Arc::new(Box::new(|_, _, _, _| panic!("load_plugin should not be called")))
    }

    fn unused_setup() -> ExternalLinterSetupRuleConfigsCb {
        Arc::new(Box::new(|_| panic!("setup_rule_configs should not be called")))
    }

    fn unused_lint() -> ExternalLinterLintFileCb {
        Arc::new(Box::new(|_, _, _, _, _, _, _| panic!("lint_file should not be called")))
    }

    fn unused_forget() -> ExternalLinterForgetBufferCb {
        Arc::new(Box::new(|_| panic!("forget_buffer should not be called")))
    }

    fn unused_create() -> ExternalLinterCreateWorkspaceCb {
        Arc::new(Box::new(|_| panic!("create_workspace should not be called")))
    }

    fn unused_destroy() -> ExternalLinterDestroyWorkspaceCb {
        Arc::new(Box::new(|_| panic!("destroy_workspace should not be called")))
    }

    fn plugin_ok(name: &str, offset: usize, rules: &[&str]) -> LoadPluginResult {
        LoadPluginResult {
            name: name.to_string(),
            offset,
            rule_names: rules.iter().map(|s| (*s).to_string()).collect(),
        }
    }

    fn load_stub(result: LoadPluginResult, calls: Arc<AtomicUsize>) -> ExternalLinterLoadPluginCb {
        Arc::new(Box::new(move |_, _, _, _| {
            calls.fetch_add(1, Ordering::SeqCst);
            Ok(result.clone())
        }))
    }

    fn setup_stub(
        calls: Arc<AtomicUsize>,
        err: Option<String>,
    ) -> ExternalLinterSetupRuleConfigsCb {
        Arc::new(Box::new(move |_| {
            calls.fetch_add(1, Ordering::SeqCst);
            match &err {
                Some(message) => Err(message.clone()),
                None => Ok(()),
            }
        }))
    }

    fn lint_stub(called: Arc<Mutex<Vec<u32>>>) -> ExternalLinterLintFileCb {
        Arc::new(Box::new(move |_, _, _, _, _, _, allocator| {
            #[cfg(all(target_pointer_width = "64", target_endian = "little"))]
            {
                // SAFETY: routing tests only pass fixed-size allocators.
                let buffer_id = unsafe { allocator.fixed_size_buffer_id() };
                called.lock().expect("lint_file stub mutex poisoned").push(buffer_id);
            }
            #[cfg(not(all(target_pointer_width = "64", target_endian = "little")))]
            {
                let _ = allocator;
                called.lock().expect("lint_file stub mutex poisoned").push(0);
            }
            Ok(Vec::new())
        }))
    }

    fn forget_stub(called: Arc<Mutex<Vec<u32>>>) -> ExternalLinterForgetBufferCb {
        Arc::new(Box::new(move |buffer_id| {
            called.lock().expect("forget_buffer stub mutex poisoned").push(buffer_id);
            Ok(())
        }))
    }

    fn linter_from_slots(
        load: Vec<ExternalLinterLoadPluginCb>,
        setup: Vec<ExternalLinterSetupRuleConfigsCb>,
        lint: Vec<ExternalLinterLintFileCb>,
        forget: Vec<ExternalLinterForgetBufferCb>,
        create: Vec<ExternalLinterCreateWorkspaceCb>,
        destroy: Vec<ExternalLinterDestroyWorkspaceCb>,
    ) -> ExternalLinter {
        ExternalLinter::new(
            load.into_boxed_slice(),
            setup.into_boxed_slice(),
            lint.into_boxed_slice(),
            forget.into_boxed_slice(),
            create.into_boxed_slice(),
            destroy.into_boxed_slice(),
        )
    }

    fn call_lint_file(
        linter: &ExternalLinter,
        allocator: &Allocator,
    ) -> Result<Vec<LintFileResult>, String> {
        linter.lint_file(
            "file.js".to_string(),
            Vec::new(),
            Vec::new(),
            "{}".to_string(),
            "{}".to_string(),
            None,
            allocator,
        )
    }

    #[test]
    fn load_plugin_calls_all_k_and_returns_first() {
        let calls0 = Arc::new(AtomicUsize::new(0));
        let calls1 = Arc::new(AtomicUsize::new(0));
        let first = plugin_ok("demo", 3, &["a", "b"]);
        let linter = linter_from_slots(
            vec![
                load_stub(first.clone(), Arc::clone(&calls0)),
                load_stub(first.clone(), Arc::clone(&calls1)),
            ],
            vec![unused_setup(), unused_setup()],
            vec![unused_lint(), unused_lint()],
            vec![unused_forget(), unused_forget()],
            vec![unused_create(), unused_create()],
            vec![unused_destroy(), unused_destroy()],
        );

        let result = linter
            .load_plugin("file:///plugin.js".into(), Some("demo".into()), false, None)
            .unwrap();
        assert_eq!(calls0.load(Ordering::SeqCst), 1);
        assert_eq!(calls1.load(Ordering::SeqCst), 1);
        assert_eq!(result.name, first.name);
        assert_eq!(result.offset, first.offset);
        assert_eq!(result.rule_names, first.rule_names);
    }

    #[test]
    fn load_plugin_mismatch_is_error() {
        let first = plugin_ok("demo", 3, &["a"]);
        let other = plugin_ok("demo", 3, &["b"]);
        let linter = linter_from_slots(
            vec![
                load_stub(first, Arc::new(AtomicUsize::new(0))),
                load_stub(other, Arc::new(AtomicUsize::new(0))),
            ],
            vec![unused_setup(), unused_setup()],
            vec![unused_lint(), unused_lint()],
            vec![unused_forget(), unused_forget()],
            vec![unused_create(), unused_create()],
            vec![unused_destroy(), unused_destroy()],
        );

        let err = linter
            .load_plugin("file:///plugin.js".into(), Some("demo".into()), false, None)
            .unwrap_err();
        assert_eq!(err, "JS worker rule registration mismatch");
    }

    #[test]
    fn setup_rule_configs_calls_all_k() {
        let calls0 = Arc::new(AtomicUsize::new(0));
        let calls1 = Arc::new(AtomicUsize::new(0));
        let linter = linter_from_slots(
            vec![unused_load(), unused_load()],
            vec![setup_stub(Arc::clone(&calls0), None), setup_stub(Arc::clone(&calls1), None)],
            vec![unused_lint(), unused_lint()],
            vec![unused_forget(), unused_forget()],
            vec![unused_create(), unused_create()],
            vec![unused_destroy(), unused_destroy()],
        );

        linter.setup_rule_configs("{}".into()).unwrap();
        assert_eq!(calls0.load(Ordering::SeqCst), 1);
        assert_eq!(calls1.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn setup_rule_configs_worker_error_is_error() {
        let calls0 = Arc::new(AtomicUsize::new(0));
        let calls1 = Arc::new(AtomicUsize::new(0));
        let linter = linter_from_slots(
            vec![unused_load(), unused_load()],
            vec![
                setup_stub(Arc::clone(&calls0), None),
                setup_stub(Arc::clone(&calls1), Some("bad options".into())),
            ],
            vec![unused_lint(), unused_lint()],
            vec![unused_forget(), unused_forget()],
            vec![unused_create(), unused_create()],
            vec![unused_destroy(), unused_destroy()],
        );

        let err = linter.setup_rule_configs("{}".into()).unwrap_err();
        assert_eq!(err, "bad options");
        assert_eq!(calls0.load(Ordering::SeqCst), 1);
        assert_eq!(calls1.load(Ordering::SeqCst), 1);
    }

    #[test]
    #[should_panic(expected = "ExternalLinter::lint_file requires a fixed-size allocator")]
    fn lint_file_panics_on_non_fixed_size_allocator() {
        let linter = linter_from_slots(
            vec![unused_load()],
            vec![unused_setup()],
            vec![unused_lint()],
            vec![unused_forget()],
            vec![unused_create()],
            vec![unused_destroy()],
        );
        let allocator = Allocator::new();
        let _ = call_lint_file(&linter, &allocator);
    }

    #[cfg(all(target_pointer_width = "64", target_endian = "little"))]
    #[test]
    fn lint_file_routes_by_buffer_id_mod_k() {
        use oxc_allocator::AllocatorPool;

        let called0 = Arc::new(Mutex::new(Vec::new()));
        let called1 = Arc::new(Mutex::new(Vec::new()));
        let linter = linter_from_slots(
            vec![unused_load(), unused_load()],
            vec![unused_setup(), unused_setup()],
            vec![lint_stub(Arc::clone(&called0)), lint_stub(Arc::clone(&called1))],
            vec![unused_forget(), unused_forget()],
            vec![unused_create(), unused_create()],
            vec![unused_destroy(), unused_destroy()],
        );

        let pool = AllocatorPool::new_fixed_size(2);
        let a0 = pool.get();
        let a1 = pool.get();
        // SAFETY: allocators came from `new_fixed_size`.
        let id0 = unsafe { a0.fixed_size_buffer_id() };
        // SAFETY: allocators came from `new_fixed_size`.
        let id1 = unsafe { a1.fixed_size_buffer_id() };

        call_lint_file(&linter, &a0).unwrap();
        call_lint_file(&linter, &a1).unwrap();

        let got0 = called0.lock().expect("mutex").clone();
        let got1 = called1.lock().expect("mutex").clone();
        for id in [id0, id1] {
            let owner = (id as usize) % 2;
            if owner == 0 {
                assert!(got0.contains(&id), "buffer {id} should route to worker 0");
                assert!(!got1.contains(&id), "buffer {id} should not route to worker 1");
            } else {
                assert!(got1.contains(&id), "buffer {id} should route to worker 1");
                assert!(!got0.contains(&id), "buffer {id} should not route to worker 0");
            }
        }
    }

    #[cfg(all(target_pointer_width = "64", target_endian = "little"))]
    #[test]
    fn lint_file_k1_uses_the_only_callback() {
        use oxc_allocator::AllocatorPool;

        let called = Arc::new(Mutex::new(Vec::new()));
        let linter = linter_from_slots(
            vec![unused_load()],
            vec![unused_setup()],
            vec![lint_stub(Arc::clone(&called))],
            vec![unused_forget()],
            vec![unused_create()],
            vec![unused_destroy()],
        );

        let pool = AllocatorPool::new_fixed_size(1);
        let allocator = pool.get();
        // SAFETY: allocator came from `new_fixed_size`.
        let id = unsafe { allocator.fixed_size_buffer_id() };
        call_lint_file(&linter, &allocator).unwrap();
        assert_eq!(*called.lock().expect("mutex"), vec![id]);
    }

    #[test]
    fn forget_buffer_routes_by_buffer_id_mod_k() {
        let called0 = Arc::new(Mutex::new(Vec::new()));
        let called1 = Arc::new(Mutex::new(Vec::new()));
        let linter = linter_from_slots(
            vec![unused_load(), unused_load()],
            vec![unused_setup(), unused_setup()],
            vec![unused_lint(), unused_lint()],
            vec![forget_stub(Arc::clone(&called0)), forget_stub(Arc::clone(&called1))],
            vec![unused_create(), unused_create()],
            vec![unused_destroy(), unused_destroy()],
        );

        linter.forget_buffer(4).unwrap();
        linter.forget_buffer(5).unwrap();
        assert_eq!(*called0.lock().expect("mutex"), vec![4]);
        assert_eq!(*called1.lock().expect("mutex"), vec![5]);
    }
}
