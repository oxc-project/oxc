use std::{
    collections::BTreeSet,
    ffi::OsStr,
    io::{ErrorKind, Read, Write, stderr},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use oxc_allocator::Allocator;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

use oxc_diagnostics::{DiagnosticSender, DiagnosticService, OxcDiagnostic, Severity};
use oxc_span::{SourceType, Span};

use super::{AllowWarnDeny, ConfigStore, DisableDirectives, ResolvedLinterState, read_to_string};

use crate::{CompositeFix, FixKind, Fixer, Message, PossibleFixes};

/// State required to initialize the `tsgolint` linter.
#[derive(Debug, Clone)]
pub struct TsGoLintState {
    /// The path to the `tsgolint` executable (at least our best guess at it).
    executable_path: PathBuf,
    /// Current working directory, used for rendering paths in diagnostics.
    cwd: PathBuf,
    /// The configuration store for `tsgolint` (used to resolve configurations outside of `oxc_linter`)
    config_store: ConfigStore,
    /// If `oxlint` will output the diagnostics or not.
    /// When `silent` is true, we do not need to access the file system for nice diagnostics messages.
    silent: bool,
    /// If `true`, request that fixes be returned from `tsgolint`.
    fix: bool,
    /// If `true`, request that suggestions be returned from `tsgolint`.
    fix_suggestions: bool,
    /// If `true`, include TypeScript compiler syntactic and semantic diagnostics.
    type_check: bool,
}

impl TsGoLintState {
    pub fn new(cwd: &Path, config_store: ConfigStore, fix_kind: FixKind) -> Self {
        let executable_path =
            try_find_tsgolint_executable(cwd).unwrap_or(PathBuf::from("tsgolint"));

        TsGoLintState {
            config_store,
            executable_path,
            cwd: cwd.to_path_buf(),
            silent: false,
            fix: fix_kind.contains(FixKind::Fix),
            fix_suggestions: fix_kind.contains(FixKind::Suggestion),
            type_check: false,
        }
    }

    /// Try to create a new TsGoLintState, returning an error if the executable cannot be found.
    ///
    /// # Errors
    /// Returns an error if the tsgolint executable cannot be found.
    pub fn try_new(
        cwd: &Path,
        config_store: ConfigStore,
        fix_kind: FixKind,
    ) -> Result<Self, String> {
        let executable_path = try_find_tsgolint_executable(cwd)?;

        Ok(TsGoLintState {
            config_store,
            executable_path,
            cwd: cwd.to_path_buf(),
            silent: false,
            fix: fix_kind.contains(FixKind::Fix),
            fix_suggestions: fix_kind.contains(FixKind::Suggestion),
            type_check: false,
        })
    }

    /// Set to `true` to skip file system reads.
    /// When `silent` is true, we do not need to access the file system for nice diagnostics messages.
    ///
    /// Default is `false`.
    #[must_use]
    pub fn with_silent(mut self, yes: bool) -> Self {
        self.silent = yes;
        self
    }

    /// Set to `true` to include TypeScript compiler syntactic and semantic diagnostics.
    ///
    /// Default is `false`.
    #[must_use]
    pub fn with_type_check(mut self, yes: bool) -> Self {
        self.type_check = yes;
        self
    }

    /// # Panics
    /// - when `stdin` of subprocess cannot be opened
    /// - when `stdout` of subprocess cannot be opened
    /// - when `tsgolint` process cannot be awaited
    ///
    /// # Errors
    /// A human-readable error message indicating why the linting failed.
    pub fn lint(
        self,
        paths: &[Arc<OsStr>],
        disable_directives_map: Arc<Mutex<FxHashMap<PathBuf, DisableDirectives>>>,
        error_sender: DiagnosticSender,
        file_system: &(dyn crate::RuntimeFileSystem + Sync + Send),
    ) -> Result<(), String> {
        if paths.is_empty() {
            return Ok(());
        }

        let mut resolved_configs: FxHashMap<PathBuf, ResolvedLinterState> = FxHashMap::default();

        let json_input = self.json_input(paths, None, &mut resolved_configs);
        if json_input.configs.is_empty() {
            return Ok(());
        }

        let should_fix = self.fix || self.fix_suggestions;
        let cwd = self.cwd.clone();
        let sender_for_fixes = error_sender.clone();

        let handler = std::thread::spawn(move || {
            let mut child = self.spawn_tsgolint(&json_input)?;

            let stdout = child.stdout.take().expect("Failed to open tsgolint stdout");

            // Process stdout stream in a separate thread to send diagnostics as they arrive
            let stdout_handler = std::thread::spawn(
                move || -> Result<Vec<(PathBuf, String, Vec<Message>)>, String> {
                    let disable_directives_map = disable_directives_map
                        .lock()
                        .expect("disable_directives_map mutex poisoned");

                    let mut diagnostic_handler = DiagnosticHandler::new(
                        self.cwd.clone(),
                        self.silent,
                        should_fix,
                        error_sender,
                    );

                    let msg_iter = TsGoLintMessageStream::new(stdout);

                    for msg in msg_iter {
                        match msg {
                            Ok(TsGoLintMessage::Error(err)) => {
                                return Err(err.error);
                            }
                            Ok(TsGoLintMessage::Diagnostic(tsgolint_diagnostic)) => {
                                match tsgolint_diagnostic {
                                    TsGoLintDiagnostic::Rule(tsgolint_diagnostic) => {
                                        let path = &tsgolint_diagnostic.file_path;

                                        let severity = resolved_configs
                                            .get(path)
                                            .or_else(|| {
                                                debug_assert!(false, "resolved_configs should have an entry for every file we linted {tsgolint_diagnostic:?}");
                                                None
                                            })
                                            .and_then(|resolved_config| {
                                                resolved_config
                                                    .rules
                                                    .iter()
                                                    .find(|(rule, _)| {
                                                        rule.name() == tsgolint_diagnostic.rule
                                                    })
                                                    .map(|(_, status)| *status)
                                            })
                                            .or_else(|| {
                                                debug_assert!(false, "resolved_config should have a matching rule for every diagnostic we received {tsgolint_diagnostic:?}");
                                                None
                                            });
                                        let Some(severity) = severity else {
                                            // If the severity is not found, we should not report
                                            // the diagnostic
                                            continue;
                                        };

                                        if should_skip_diagnostic(
                                            &disable_directives_map,
                                            path,
                                            &tsgolint_diagnostic,
                                        ) {
                                            continue;
                                        }

                                        diagnostic_handler
                                            .handle_rule_diagnostic(tsgolint_diagnostic, severity);
                                    }
                                    TsGoLintDiagnostic::Internal(e) => {
                                        diagnostic_handler.handle_internal_diagnostic(e);
                                    }
                                }
                            }
                            Err(e) => {
                                return Err(e);
                            }
                        }
                    }

                    Ok(diagnostic_handler.into_messages_requiring_fixes())
                },
            );

            // Wait for process to complete and stdout processing to finish
            let exit_status = child.wait().expect("Failed to wait for tsgolint process");
            let stdout_result = stdout_handler.join();

            if !exit_status.success() {
                return Err(
                    if let Some(err) = &stdout_result.ok().and_then(std::result::Result::err) {
                        format!("exit status: {exit_status}, error: {err}")
                    } else {
                        format!("exit status: {exit_status}")
                    },
                );
            }

            match stdout_result {
                Ok(Ok(messages)) => Ok(messages),
                Ok(Err(err)) => Err(format!("exit status: {exit_status}, error: {err}")),
                Err(_) => Err("Failed to join stdout processing thread".to_string()),
            }
        });

        match handler.join() {
            Ok(Ok(messages_requiring_fixes)) => {
                for (path, source_text, messages) in messages_requiring_fixes {
                    let source_type = SourceType::from_path(&path)
                        .ok()
                        .map(|st| if st.is_javascript() { st.with_jsx(true) } else { st });
                    let fix_result = Fixer::new(&source_text, messages, source_type).fix();

                    if fix_result.fixed {
                        file_system
                            .write_file(&path, &fix_result.fixed_code)
                            .expect("Failed to write fixed file");
                    }

                    if !fix_result.messages.is_empty() {
                        let source_for_diagnostics: &str =
                            if fix_result.fixed { &fix_result.fixed_code } else { &source_text };
                        let diagnostics = DiagnosticService::wrap_diagnostics(
                            &cwd,
                            &path,
                            source_for_diagnostics,
                            fix_result.messages.into_iter().map(Into::into).collect(),
                        );
                        sender_for_fixes.send(diagnostics).expect("Failed to send diagnostics");
                    }
                }
                Ok(())
            }
            Ok(Err(err)) => Err(format!("Error running tsgolint: {err:?}")),
            Err(err) => Err(format!("Error running tsgolint: {err:?}")),
        }
    }

    /// Spawn the tsgolint process with the given input.
    fn spawn_tsgolint(&self, json_input: &Payload) -> Result<std::process::Child, String> {
        let mut cmd = std::process::Command::new(&self.executable_path);
        cmd.arg("headless")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(stderr());

        if self.fix {
            cmd.arg("-fix");
        }

        if self.fix_suggestions {
            cmd.arg("-fix-suggestions");
        }

        if let Ok(trace_file) = std::env::var("OXLINT_TSGOLINT_TRACE") {
            cmd.arg(format!("-trace={trace_file}"));
        }
        if let Ok(cpuprof_file) = std::env::var("OXLINT_TSGOLINT_CPU") {
            cmd.arg(format!("-cpuprof={cpuprof_file}"));
        }
        if let Ok(heap_file) = std::env::var("OXLINT_TSGOLINT_HEAP") {
            cmd.arg(format!("-heap={heap_file}"));
        }
        if let Ok(allocs_file) = std::env::var("OXLINT_TSGOLINT_ALLOCS") {
            cmd.arg(format!("-allocs={allocs_file}"));
        }

        let mut child = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => {
                return Err(format!(
                    "Failed to spawn tsgolint from path `{}`, with error: {e}",
                    self.executable_path.display()
                ));
            }
        };

        let mut stdin = child.stdin.take().expect("Failed to open tsgolint stdin");

        let json = serde_json::to_string(json_input).expect("Failed to serialize JSON");
        if let Err(e) = stdin.write_all(json.as_bytes())
            && e.kind() != ErrorKind::BrokenPipe
        {
            return Err(format!("Failed to write to tsgolint stdin: {e}"));
        }
        drop(stdin);

        Ok(child)
    }

    /// # Panics
    /// - when `stdin` of subprocess cannot be opened
    /// - when `stdout` of subprocess cannot be opened
    /// - when `tsgolint` process cannot be awaited
    ///
    /// # Errors
    /// A human-readable error message indicating why the linting failed.
    pub fn lint_source(
        &self,
        path: &Arc<OsStr>,
        file_system: &(dyn crate::RuntimeFileSystem + Sync + Send),
        disable_directives_map: Arc<Mutex<FxHashMap<PathBuf, DisableDirectives>>>,
    ) -> Result<Vec<Message>, String> {
        let mut resolved_configs: FxHashMap<PathBuf, ResolvedLinterState> = FxHashMap::default();
        let mut source_overrides = FxHashMap::default();
        let allocator = Allocator::default();
        let Ok(source_text) = file_system.read_to_arena_str(Path::new(path.as_ref()), &allocator)
        else {
            return Err(format!("Failed to read source text for file: {}", path.to_string_lossy()));
        };

        // Clone source_text to own it for the spawned thread
        let source_text_owned = source_text.to_string();
        source_overrides.insert(path.to_string_lossy().to_string(), source_text_owned.clone());

        let json_input = self.json_input(
            std::slice::from_ref(path),
            Some(source_overrides),
            &mut resolved_configs,
        );
        let path_file_name =
            Path::new(path.as_ref()).file_name().unwrap_or_default().to_os_string();
        let mut child = self.spawn_tsgolint(&json_input)?;
        let handler = std::thread::spawn(move || {
            let stdout = child.stdout.take().expect("Failed to open tsgolint stdout");

            let stdout_handler = std::thread::spawn(move || -> Result<Vec<Message>, String> {
                let disable_directives_map =
                    disable_directives_map.lock().expect("disable_directives_map mutex poisoned");
                let msg_iter = TsGoLintMessageStream::new(stdout);

                let mut result = vec![];

                for msg in msg_iter {
                    match msg {
                        Ok(TsGoLintMessage::Error(err)) => {
                            return Err(err.error);
                        }
                        Ok(TsGoLintMessage::Diagnostic(tsgolint_diagnostic)) => {
                            match tsgolint_diagnostic {
                                TsGoLintDiagnostic::Rule(tsgolint_diagnostic) => {
                                    let path = tsgolint_diagnostic.file_path.clone();
                                    let Some(resolved_config) = resolved_configs.get(&path) else {
                                        // If we don't have a resolved config for this path, skip it. We should always
                                        // have a resolved config though, since we processed them already above.
                                        continue;
                                    };

                                    let severity =
                                        resolved_config.rules.iter().find_map(|(rule, status)| {
                                            if rule.name() == tsgolint_diagnostic.rule {
                                                Some(*status)
                                            } else {
                                                None
                                            }
                                        });
                                    let Some(severity) = severity else {
                                        // If the severity is not found, we should not report the diagnostic
                                        continue;
                                    };

                                    if should_skip_diagnostic(
                                        &disable_directives_map,
                                        &path,
                                        &tsgolint_diagnostic,
                                    ) {
                                        continue;
                                    }

                                    let mut message = Message::from_tsgo_lint_diagnostic(
                                        tsgolint_diagnostic,
                                        &source_text_owned,
                                    );

                                    message.error.severity = if severity == AllowWarnDeny::Deny {
                                        Severity::Error
                                    } else {
                                        Severity::Warning
                                    };

                                    result.push(message);
                                }
                                TsGoLintDiagnostic::Internal(e) => {
                                    let span = e
                                        .file_path
                                        .as_ref()
                                        .is_some_and(|f| {
                                            f.file_name().unwrap_or_default() == path_file_name
                                        })
                                        .then_some(e.span)
                                        .flatten()
                                        .unwrap_or_default();
                                    let mut diagnostic: OxcDiagnostic = e.into();
                                    diagnostic = diagnostic.with_label(span);
                                    result.push(Message::new(diagnostic, PossibleFixes::None));
                                }
                            }
                        }
                        Err(e) => {
                            return Err(e);
                        }
                    }
                }

                Ok(result)
            });

            // Wait for process to complete and stdout processing to finish
            let exit_status = child.wait().expect("Failed to wait for tsgolint process");
            let stdout_result = stdout_handler.join();

            if !exit_status.success() {
                let err_msg = stdout_result.ok().and_then(Result::err).unwrap_or_default();
                return Err(format!(
                    "tsgolint process exited with status: {exit_status}, {err_msg}"
                ));
            }

            match stdout_result {
                Ok(Ok(diagnostics)) => Ok(diagnostics),
                Ok(Err(err)) => Err(err),
                Err(_) => Err("Failed to join stdout processing thread".to_string()),
            }
        });

        match handler.join() {
            Ok(Ok(diagnostics)) => {
                // Successfully ran tsgolint
                Ok(diagnostics)
            }
            Ok(Err(err)) => Err(format!("Error running tsgolint: {err:?}")),
            Err(err) => Err(format!("Error running tsgolint: {err:?}")),
        }
    }

    /// Create a JSON input for STDIN of tsgolint in this format:
    ///
    /// ```json
    /// {
    ///   "files": [
    ///     {
    ///       "file_path": "/absolute/path/to/file.ts",
    ///       "rules": ["rule-1", "another-rule"]
    ///     }
    ///   ]
    /// }
    /// ```
    #[inline]
    fn json_input(
        &self,
        paths: &[Arc<OsStr>],
        source_overrides: Option<FxHashMap<String, String>>,
        resolved_configs: &mut FxHashMap<PathBuf, ResolvedLinterState>,
    ) -> Payload {
        let mut config_groups: FxHashMap<BTreeSet<Rule>, Vec<String>> = FxHashMap::default();

        for path in paths {
            if SourceType::from_path(Path::new(path)).is_ok() {
                let path_buf = PathBuf::from(path);
                let file_path = path.to_string_lossy().to_string();

                let resolved_config = resolved_configs
                    .entry(path_buf.clone())
                    .or_insert_with(|| self.config_store.resolve(&path_buf));

                let rules: BTreeSet<Rule> = resolved_config
                    .rules
                    .iter()
                    .filter_map(|(rule, status)| {
                        if status.is_warn_deny() && rule.is_tsgolint_rule() {
                            let rule_name = rule.name().to_string();
                            let options = match rule.to_configuration() {
                                Some(Ok(config)) => Some(config),
                                Some(Err(_)) | None => None,
                            };
                            Some(Rule { name: rule_name, options })
                        } else {
                            None
                        }
                    })
                    .collect();

                config_groups.entry(rules).or_default().push(file_path);
            }
        }

        Payload {
            version: 2,
            configs: config_groups
                .into_iter()
                .map(|(rules, file_paths)| Config {
                    file_paths,
                    rules: rules.into_iter().collect(),
                })
                .collect(),
            source_overrides,
            report_syntactic: self.type_check,
            report_semantic: self.type_check,
        }
    }
}

/// Represents the input JSON to `tsgolint`, like:
///
/// ```json
/// {
///   "version": 2,
///   "configs": [
///     {
///       "file_paths": ["/absolute/path/to/file.ts", "/another/file.ts"],
///       "rules": [
///         { "name": "rule-1" },
///         { "name": "another-rule" },
///       ]
///     }
///   ]
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payload {
    pub version: i32,
    pub configs: Vec<Config>,
    pub source_overrides: Option<FxHashMap<String, String>>,
    pub report_syntactic: bool,
    pub report_semantic: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Absolute path to the file to lint
    pub file_paths: Vec<String>,
    /// List of rules to apply to this file
    /// Example: `["no-floating-promises"]`
    pub rules: Vec<Rule>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub struct Rule {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<serde_json::Value>,
}

impl PartialOrd for Rule {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Rule {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // First compare by name
        match self.name.cmp(&other.name) {
            std::cmp::Ordering::Equal => {
                // If names are equal, compare by serialized options
                // Serialize to canonical JSON string for comparison
                let self_options = self.options.as_ref().map(|v| serde_json::to_string(v).ok());
                let other_options = other.options.as_ref().map(|v| serde_json::to_string(v).ok());
                self_options.cmp(&other_options)
            }
            other_ordering => other_ordering,
        }
    }
}

/// Diagnostic kind discriminator
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DiagnosticKind {
    Rule = 0,
    Internal = 1,
}

impl Serialize for DiagnosticKind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u8(*self as u8)
    }
}

impl<'de> Deserialize<'de> for DiagnosticKind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = u8::deserialize(deserializer)?;
        match value {
            0 => Ok(DiagnosticKind::Rule),
            1 => Ok(DiagnosticKind::Internal),
            _ => Err(serde::de::Error::custom(format!("Invalid DiagnosticKind value: {value}"))),
        }
    }
}

/// Represents the raw output binary data from `tsgolint`.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TsGoLintDiagnosticPayload {
    pub kind: DiagnosticKind,
    pub range: Option<Range>,
    pub message: RuleMessage,
    pub file_path: Option<String>,
    // Only for kind="rule"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fixes: Vec<Fix>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub suggestions: Vec<Suggestion>,
}

/// Represents the error payload from `tsgolint`.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TsGoLintErrorPayload {
    pub error: String,
}

#[derive(Debug, Clone)]
pub enum TsGoLintMessage {
    Diagnostic(TsGoLintDiagnostic),
    Error(TsGoLintError),
}

#[derive(Debug, Clone)]
pub enum TsGoLintDiagnostic {
    Rule(TsGoLintRuleDiagnostic),
    Internal(TsGoLintInternalDiagnostic),
}

#[derive(Debug, Clone)]
pub struct TsGoLintRuleDiagnostic {
    pub span: Span,
    pub rule: String,
    pub message: RuleMessage,
    pub fixes: Vec<Fix>,
    pub suggestions: Vec<Suggestion>,
    pub file_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct TsGoLintInternalDiagnostic {
    pub message: RuleMessage,
    pub span: Option<Span>,
    pub file_path: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct TsGoLintError {
    pub error: String,
}

impl From<TsGoLintDiagnostic> for OxcDiagnostic {
    fn from(val: TsGoLintDiagnostic) -> Self {
        match val {
            TsGoLintDiagnostic::Rule(d) => d.into(),
            TsGoLintDiagnostic::Internal(d) => d.into(),
        }
    }
}

impl From<TsGoLintRuleDiagnostic> for OxcDiagnostic {
    fn from(val: TsGoLintRuleDiagnostic) -> Self {
        let mut d = OxcDiagnostic::warn(val.message.description)
            .with_label(val.span)
            .with_error_code("typescript-eslint", val.rule);
        if let Some(help) = val.message.help {
            d = d.with_help(help);
        }
        d
    }
}
impl From<TsGoLintInternalDiagnostic> for OxcDiagnostic {
    fn from(val: TsGoLintInternalDiagnostic) -> Self {
        let mut d = OxcDiagnostic::error(val.message.description)
            .with_error_code("typescript", val.message.id);
        if let Some(help) = val.message.help {
            d = d.with_help(help);
        }
        if val.file_path.is_some()
            && let Some(span) = val.span
        {
            d = d.with_label(span);
        }
        d
    }
}

impl Message {
    /// Converts a `TsGoLintDiagnostic` into a `Message` with possible fixes.
    fn from_tsgo_lint_diagnostic(mut val: TsGoLintRuleDiagnostic, source_text: &str) -> Self {
        use std::{borrow::Cow, mem};

        let mut fixes =
            Vec::with_capacity(usize::from(!val.fixes.is_empty()) + val.suggestions.len());

        if !val.fixes.is_empty() {
            let fix_vec = mem::take(&mut val.fixes);
            let fix_vec = fix_vec
                .into_iter()
                .map(|fix| crate::fixer::Fix {
                    content: Cow::Owned(fix.text),
                    span: Span::new(fix.range.pos, fix.range.end),
                    message: None,
                })
                .collect();

            fixes.push(CompositeFix::merge_fixes(fix_vec, source_text));
        }

        let suggestions = mem::take(&mut val.suggestions);
        fixes.extend(suggestions.into_iter().map(|mut suggestion| {
            let last_fix_index = suggestion.fixes.len().wrapping_sub(1);
            let fix_vec = suggestion
                .fixes
                .into_iter()
                .enumerate()
                .map(|(i, fix)| {
                    // Don't clone the message description on last turn of loop
                    let message = if i < last_fix_index {
                        suggestion.message.description.clone()
                    } else {
                        mem::take(&mut suggestion.message.description)
                    };

                    crate::fixer::Fix {
                        content: Cow::Owned(fix.text),
                        span: Span::new(fix.range.pos, fix.range.end),
                        message: Some(Cow::Owned(message)),
                    }
                })
                .collect();

            CompositeFix::merge_fixes(fix_vec, source_text)
        }));

        let possible_fix = if fixes.is_empty() {
            PossibleFixes::None
        } else if fixes.len() == 1 {
            PossibleFixes::Single(fixes.into_iter().next().unwrap())
        } else {
            PossibleFixes::Multiple(fixes)
        };

        Self::new(val.into(), possible_fix)
    }
}

// TODO: Should this be removed and replaced with a `Span`?
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Range {
    pub pos: u32,
    pub end: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuleMessage {
    pub id: String,
    pub description: String,
    pub help: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Fix {
    pub text: String,
    pub range: Range,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Suggestion {
    pub message: RuleMessage,
    pub fixes: Vec<Fix>,
}

#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
#[repr(u8)]
pub enum MessageType {
    Error = 0,
    Diagnostic = 1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidMessageType(pub u8);

impl std::fmt::Display for InvalidMessageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid message type: {}", self.0)
    }
}

impl std::error::Error for InvalidMessageType {}

impl TryFrom<u8> for MessageType {
    type Error = InvalidMessageType;

    fn try_from(value: u8) -> Result<Self, InvalidMessageType> {
        match value {
            0 => Ok(Self::Error),
            1 => Ok(Self::Diagnostic),
            _ => Err(InvalidMessageType(value)),
        }
    }
}

/// Iterator that streams messages from tsgolint stdout.
struct TsGoLintMessageStream {
    stdout: std::process::ChildStdout,
    buffer: Vec<u8>,
}

impl TsGoLintMessageStream {
    fn new(stdout: std::process::ChildStdout) -> TsGoLintMessageStream {
        TsGoLintMessageStream { stdout, buffer: Vec::with_capacity(8192) }
    }
}

impl Iterator for TsGoLintMessageStream {
    type Item = Result<TsGoLintMessage, String>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut read_buf = [0u8; 8192];

        loop {
            // Try to parse a complete message from the existing buffer
            let mut cursor = std::io::Cursor::new(self.buffer.as_slice());

            if cursor.position() < self.buffer.len() as u64 {
                match parse_single_message(&mut cursor) {
                    Ok(message) => {
                        // Successfully parsed a message, remove it from buffer
                        #[expect(clippy::cast_possible_truncation)]
                        self.buffer.drain(..cursor.position() as usize);
                        return Some(Ok(message));
                    }
                    Err(TsGoLintMessageParseError::IncompleteData) => {}
                    Err(e) => {
                        return Some(Err(e.to_string()));
                    }
                }
            }

            // Read more data from stdout
            match self.stdout.read(&mut read_buf) {
                Ok(0) => {
                    return None;
                }
                Ok(n) => {
                    self.buffer.extend_from_slice(&read_buf[..n]);
                }
                Err(e) => {
                    return Some(Err(format!("Failed to read from tsgolint stdout: {e}")));
                }
            }
        }
    }
}

enum TsGoLintMessageParseError {
    IncompleteData,
    InvalidMessageType(InvalidMessageType),
    InvalidErrorPayload(serde_json::Error),
    InvalidDiagnosticPayload(serde_json::Error),
}

impl std::fmt::Display for TsGoLintMessageParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TsGoLintMessageParseError::IncompleteData => write!(f, "Incomplete data"),
            TsGoLintMessageParseError::InvalidMessageType(e) => write!(f, "{e}"),
            TsGoLintMessageParseError::InvalidErrorPayload(e) => {
                write!(f, "Failed to parse tsgolint error payload: {e}")
            }
            TsGoLintMessageParseError::InvalidDiagnosticPayload(e) => {
                write!(f, "Failed to parse tsgolint diagnostic payload: {e}")
            }
        }
    }
}

/// Cache for source text to avoid reading the same file multiple times.
#[derive(Default)]
struct SourceTextCache(FxHashMap<PathBuf, String>);

impl SourceTextCache {
    fn get_or_insert(&mut self, path: &Path) -> &str {
        self.0
            .entry(path.to_path_buf())
            .or_insert_with(|| read_to_string(path).unwrap_or_default())
            .as_str()
    }
}

/// Handles streaming and collecting diagnostics from tsgolint.
struct DiagnosticHandler {
    cwd: PathBuf,
    silent: bool,
    should_fix: bool,
    source_text_cache: SourceTextCache,
    error_sender: DiagnosticSender,
    /// Messages requiring fixes, grouped by file path: messages.
    messages_requiring_fixes: FxHashMap<PathBuf, Vec<Message>>,
}

impl DiagnosticHandler {
    fn new(cwd: PathBuf, silent: bool, should_fix: bool, error_sender: DiagnosticSender) -> Self {
        Self {
            cwd,
            silent,
            should_fix,
            source_text_cache: SourceTextCache::default(),
            error_sender,
            messages_requiring_fixes: FxHashMap::default(),
        }
    }

    fn get_source_text(&mut self, path: &Path) -> &str {
        if self.silent && !self.should_fix {
            // The source text is not needed in silent mode, the diagnostic isn't printed.
            ""
        } else {
            self.source_text_cache.get_or_insert(path)
        }
    }

    fn handle_rule_diagnostic(
        &mut self,
        diagnostic: TsGoLintRuleDiagnostic,
        severity: AllowWarnDeny,
    ) {
        let path = diagnostic.file_path.clone();
        let has_fixes =
            self.should_fix && (!diagnostic.fixes.is_empty() || !diagnostic.suggestions.is_empty());

        if has_fixes {
            // Collect for later fix application
            let mut message =
                Message::from_tsgo_lint_diagnostic(diagnostic, self.get_source_text(&path));
            message.error.severity =
                if severity == AllowWarnDeny::Deny { Severity::Error } else { Severity::Warning };

            let entry = self.messages_requiring_fixes.entry(path).or_default();

            entry.push(message);
        } else {
            // Stream immediately
            self.send_diagnostic(&path, diagnostic.into(), severity);
        }
    }

    fn handle_internal_diagnostic(&mut self, e: TsGoLintInternalDiagnostic) {
        let file_path = e.file_path.clone();
        let oxc_diagnostic: OxcDiagnostic = e.into();

        let diagnostics = if let Some(ref file_path) = file_path {
            let source_text = self.get_source_text(file_path).to_string();
            DiagnosticService::wrap_diagnostics(
                &self.cwd,
                file_path,
                &source_text,
                vec![oxc_diagnostic],
            )
        } else {
            vec![oxc_diagnostic.into()]
        };

        self.error_sender.send(diagnostics).expect("Failed to send diagnostics");
    }

    fn send_diagnostic(
        &mut self,
        path: &Path,
        oxc_diagnostic: OxcDiagnostic,
        severity: AllowWarnDeny,
    ) {
        let source_text = self.get_source_text(path).to_string();
        let oxc_diagnostic = oxc_diagnostic.with_severity(if severity == AllowWarnDeny::Deny {
            Severity::Error
        } else {
            Severity::Warning
        });
        let diagnostics = DiagnosticService::wrap_diagnostics(
            &self.cwd,
            path,
            &source_text,
            vec![oxc_diagnostic],
        );
        self.error_sender.send(diagnostics).expect("Failed to send diagnostics");
    }

    /// Consume the handler and return collected messages requiring fixes.
    fn into_messages_requiring_fixes(self) -> Vec<(PathBuf, String, Vec<Message>)> {
        let Self { messages_requiring_fixes, mut source_text_cache, should_fix, silent, .. } = self;

        messages_requiring_fixes
            .into_iter()
            .map(|(path, messages)| {
                let source_text = source_text_cache.0.remove(&path).unwrap_or_else(|| {
                    if !silent || should_fix {
                        read_to_string(&path).unwrap_or_default()
                    } else {
                        String::new()
                    }
                });
                (path, source_text, messages)
            })
            .collect()
    }
}

fn should_skip_diagnostic(
    disable_directives_map: &FxHashMap<PathBuf, DisableDirectives>,
    path: &Path,
    tsgolint_diagnostic: &TsGoLintRuleDiagnostic,
) -> bool {
    let span = tsgolint_diagnostic.span;

    if let Some(directives) = disable_directives_map.get(path) {
        directives.contains(&tsgolint_diagnostic.rule, span)
            || directives.contains(&format!("typescript-eslint/{}", tsgolint_diagnostic.rule), span)
            || directives
                .contains(&format!("@typescript-eslint/{}", tsgolint_diagnostic.rule), span)
    } else {
        debug_assert!(
            false,
            "disable_directives_map should have an entry for every file we linted"
        );
        false
    }
}

/// Parses a single message from the binary tsgolint output.
// Messages are encoded as follows:
// | Payload Size (uint32 LE) - 4 bytes | Message Type (uint8) - 1 byte | Payload |
fn parse_single_message(
    cursor: &mut std::io::Cursor<&[u8]>,
) -> Result<TsGoLintMessage, TsGoLintMessageParseError> {
    let mut size_bytes = [0u8; 4];
    if cursor.read_exact(&mut size_bytes).is_err() {
        return Err(TsGoLintMessageParseError::IncompleteData);
    }
    let size = u32::from_le_bytes(size_bytes) as usize;

    let mut message_type_byte = [0u8; 1];
    if cursor.read_exact(&mut message_type_byte).is_err() {
        return Err(TsGoLintMessageParseError::IncompleteData);
    }

    let message_type = MessageType::try_from(message_type_byte[0])
        .map_err(TsGoLintMessageParseError::InvalidMessageType)?;

    let mut payload_bytes = vec![0u8; size];
    if cursor.read_exact(&mut payload_bytes).is_err() {
        return Err(TsGoLintMessageParseError::IncompleteData);
    }
    let payload_str = String::from_utf8_lossy(&payload_bytes);

    match message_type {
        MessageType::Error => {
            let error_payload = serde_json::from_str::<TsGoLintErrorPayload>(&payload_str)
                .map_err(TsGoLintMessageParseError::InvalidErrorPayload)?;

            Ok(TsGoLintMessage::Error(TsGoLintError { error: error_payload.error }))
        }
        MessageType::Diagnostic => {
            let diagnostic_payload =
                serde_json::from_str::<TsGoLintDiagnosticPayload>(&payload_str)
                    .map_err(TsGoLintMessageParseError::InvalidDiagnosticPayload)?;

            Ok(TsGoLintMessage::Diagnostic(match diagnostic_payload.kind {
                DiagnosticKind::Rule => TsGoLintDiagnostic::Rule(TsGoLintRuleDiagnostic {
                    rule: diagnostic_payload
                        .rule
                        .expect("Rule name must be present for rule diagnostics"),
                    span: diagnostic_payload.range.map_or_else(
                        || {
                            debug_assert!(false, "Range must be present for rule diagnostics");
                            Span::default()
                        },
                        |range| Span::new(range.pos, range.end),
                    ),
                    message: diagnostic_payload.message,
                    fixes: diagnostic_payload.fixes,
                    suggestions: diagnostic_payload.suggestions,
                    file_path: PathBuf::from(
                        diagnostic_payload
                            .file_path
                            .expect("File path must be present for rule diagnostics"),
                    ),
                }),
                DiagnosticKind::Internal => {
                    TsGoLintDiagnostic::Internal(TsGoLintInternalDiagnostic {
                        message: diagnostic_payload.message,
                        span: diagnostic_payload.range.map(|range| Span::new(range.pos, range.end)),
                        file_path: diagnostic_payload.file_path.map(PathBuf::from),
                    })
                }
            }))
        }
    }
}

/// Tries to find the `tsgolint` executable. In priority order, this will check:
/// 1. The `OXLINT_TSGOLINT_PATH` environment variable.
/// 2. The `tsgolint` binary in the current working directory's `node_modules/.bin` directory.
///
/// # Errors
/// Returns an error if `OXLINT_TSGOLINT_PATH` is set but does not exist or is not a file.
/// Returns an error if the tsgolint executable could not be found.
pub fn try_find_tsgolint_executable(cwd: &Path) -> Result<PathBuf, String> {
    // Check the environment variable first
    if let Ok(path_str) = std::env::var("OXLINT_TSGOLINT_PATH") {
        let path = PathBuf::from(&path_str);
        if path.is_dir() {
            let tsgolint_path = path.join("tsgolint");
            if tsgolint_path.exists() {
                return Ok(tsgolint_path);
            }
            return Err(format!(
                "Failed to find tsgolint executable: OXLINT_TSGOLINT_PATH points to directory '{path_str}' but 'tsgolint' binary not found inside"
            ));
        }
        if path.is_file() {
            return Ok(path);
        }
        return Err(format!(
            "Failed to find tsgolint executable: OXLINT_TSGOLINT_PATH points to '{path_str}' which does not exist"
        ));
    }

    // Executing a sub-command in Windows needs a `cmd` or `ps1` extension.
    // Since `cmd` is the most compatible one with older systems, we use that one first,
    // then check for `exe` which is also common. Bun, for example, does not create a `cmd`
    // file but still produces an `exe` file (https://github.com/oxc-project/oxc/issues/13784).
    #[cfg(windows)]
    let files = &["tsgolint.CMD", "tsgolint.exe"];
    #[cfg(not(windows))]
    let files = &["tsgolint"];

    // Move upwards until we find a `package.json`, then look at `node_modules/.bin/tsgolint`
    let mut current_dir = cwd.to_path_buf();
    loop {
        for file in files {
            let node_modules_bin = current_dir.join("node_modules").join(".bin").join(file);
            if node_modules_bin.exists() {
                return Ok(node_modules_bin);
            }
        }

        // If we reach the root directory, stop searching
        if !current_dir.pop() {
            break;
        }
    }

    // Finally, search in the system PATH
    // This supports package managers that install binaries globally and make them
    // available via PATH
    if let Ok(path_env) = std::env::var("PATH") {
        for dir in std::env::split_paths(&path_env) {
            for file in files {
                let candidate = dir.join(file);
                if candidate.is_file() {
                    return Ok(candidate);
                }
            }
        }
    }

    Err("Failed to find tsgolint executable. You may need to add the `oxlint-tsgolint` package to your project?".to_string())
}

#[cfg(test)]
mod test {
    use oxc_diagnostics::{LabeledSpan, OxcCode, Severity};
    use oxc_span::Span;

    use crate::{
        fixer::{Message, PossibleFixes},
        tsgolint::{Fix, Range, RuleMessage, Suggestion, TsGoLintRuleDiagnostic},
    };

    #[test]
    fn test_message_from_tsgo_lint_diagnostic_basic() {
        let diagnostic = TsGoLintRuleDiagnostic {
            span: Span::new(0, 10),
            rule: "some_rule".into(),
            message: RuleMessage {
                id: "some_id".into(),
                description: "Some description".into(),
                help: Some("Some help".into()),
            },
            fixes: vec![],
            suggestions: vec![],
            file_path: "some/file/path".into(),
        };

        let message = Message::from_tsgo_lint_diagnostic(diagnostic, "Some text over 10 bytes.");

        assert_eq!(message.error.message, "Some description");
        assert_eq!(message.error.severity, Severity::Warning);
        assert_eq!(message.span, Span::new(0, 10));
        assert_eq!(
            message.error.code,
            OxcCode { scope: Some("typescript-eslint".into()), number: Some("some_rule".into()) }
        );
        assert!(message.error.labels.as_ref().is_some());
        assert_eq!(message.error.labels.as_ref().unwrap().len(), 1);
        assert_eq!(message.error.labels.as_ref().unwrap()[0], LabeledSpan::new(None, 0, 10));
        assert_eq!(message.error.help, Some("Some help".into()));
        assert!(message.fixes.is_empty());
    }

    #[test]
    fn test_message_from_tsgo_lint_diagnostic_with_fixes() {
        let diagnostic = TsGoLintRuleDiagnostic {
            span: Span::new(0, 10),
            rule: "some_rule".into(),
            message: RuleMessage {
                id: "some_id".into(),
                description: "Some description".into(),
                help: None,
            },
            fixes: vec![
                Fix { text: "fixed".into(), range: Range { pos: 0, end: 5 } },
                Fix { text: "hello".into(), range: Range { pos: 5, end: 10 } },
            ],
            suggestions: vec![],
            file_path: "some/file/path".into(),
        };

        let message = Message::from_tsgo_lint_diagnostic(diagnostic, "Some text over 10 bytes.");

        assert_eq!(message.fixes.len(), 1);
        assert_eq!(
            message.fixes,
            PossibleFixes::Single(crate::fixer::Fix {
                content: "fixedhello".into(),
                span: Span::new(0, 10),
                message: None,
            })
        );
    }

    #[test]
    fn test_message_from_tsgo_lint_diagnostic_with_multiple_suggestions() {
        let diagnostic = TsGoLintRuleDiagnostic {
            span: Span::new(0, 10),
            rule: "some_rule".into(),
            message: RuleMessage {
                id: "some_id".into(),
                description: "Some description".into(),
                help: None,
            },
            fixes: vec![],
            suggestions: vec![
                Suggestion {
                    message: RuleMessage {
                        id: "some_id".into(),
                        description: "Suggestion 1".into(),
                        help: None,
                    },
                    fixes: vec![Fix { text: "hello".into(), range: Range { pos: 0, end: 5 } }],
                },
                Suggestion {
                    message: RuleMessage {
                        id: "some_id".into(),
                        description: "Suggestion 2".into(),
                        help: None,
                    },
                    fixes: vec![
                        Fix { text: "hello".into(), range: Range { pos: 0, end: 5 } },
                        Fix { text: "world".into(), range: Range { pos: 5, end: 10 } },
                    ],
                },
            ],
            file_path: "some/file/path".into(),
        };

        let message = Message::from_tsgo_lint_diagnostic(diagnostic, "Some text over 10 bytes.");

        assert_eq!(
            message.fixes,
            PossibleFixes::Multiple(vec![
                crate::fixer::Fix {
                    content: "hello".into(),
                    span: Span::new(0, 5),
                    message: Some("Suggestion 1".into()),
                },
                crate::fixer::Fix {
                    content: "helloworld".into(),
                    span: Span::new(0, 10),
                    message: Some("Suggestion 2".into()),
                },
            ])
        );
    }

    #[test]
    fn test_message_from_tsgo_lint_diagnostic_with_fix_and_suggestions() {
        let diagnostic = TsGoLintRuleDiagnostic {
            span: Span::new(0, 10),
            rule: "some_rule".into(),
            message: RuleMessage {
                id: "some_id".into(),
                description: "Some description".into(),
                help: None,
            },
            fixes: vec![Fix { text: "fixed".into(), range: Range { pos: 0, end: 5 } }],
            suggestions: vec![Suggestion {
                message: RuleMessage {
                    id: "some_id".into(),
                    description: "Suggestion 1".into(),
                    help: None,
                },
                fixes: vec![Fix { text: "Suggestion 1".into(), range: Range { pos: 0, end: 5 } }],
            }],
            file_path: "some/file/path".into(),
        };

        let message = Message::from_tsgo_lint_diagnostic(diagnostic, "Some text over 10 bytes.");

        assert_eq!(message.fixes.len(), 2);
        assert_eq!(
            message.fixes,
            PossibleFixes::Multiple(vec![
                crate::fixer::Fix { content: "fixed".into(), span: Span::new(0, 5), message: None },
                crate::fixer::Fix {
                    content: "Suggestion 1".into(),
                    span: Span::new(0, 5),
                    message: Some("Suggestion 1".into()),
                },
            ])
        );
    }

    #[test]
    fn test_diagnostic_payload_deserialize_without_fixes_or_suggestions() {
        use super::TsGoLintDiagnosticPayload;

        // Test payload with both fixes and suggestions omitted
        let json = r#"{
            "kind": 0,
            "range": {"pos": 0, "end": 10},
            "rule": "no-unused-vars",
            "message": {
                "id": "msg_id",
                "description": "Variable is unused",
                "help": null
            },
            "file_path": "test.ts"
        }"#;

        let payload: TsGoLintDiagnosticPayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.fixes.len(), 0);
        assert_eq!(payload.suggestions.len(), 0);
        assert_eq!(payload.rule, Some("no-unused-vars".to_string()));

        // Test payload with only fixes omitted
        let json_with_suggestions = r#"{
            "kind": 0,
            "range": {"pos": 0, "end": 10},
            "rule": "no-unused-vars",
            "message": {
                "id": "msg_id",
                "description": "Variable is unused",
                "help": null
            },
            "suggestions": [
                {
                    "message": {
                        "id": "suggestion_id",
                        "description": "Remove unused variable",
                        "help": null
                    },
                    "fixes": []
                }
            ],
            "file_path": "test.ts"
        }"#;

        let payload: TsGoLintDiagnosticPayload =
            serde_json::from_str(json_with_suggestions).unwrap();
        assert_eq!(payload.fixes.len(), 0);
        assert_eq!(payload.suggestions.len(), 1);

        // Test payload with only suggestions omitted
        let json_with_fixes = r#"{
            "kind": 0,
            "range": {"pos": 0, "end": 10},
            "rule": "no-unused-vars",
            "message": {
                "id": "msg_id",
                "description": "Variable is unused",
                "help": null
            },
            "fixes": [
                {
                    "text": "fixed",
                    "range": {"pos": 0, "end": 5}
                }
            ],
            "file_path": "test.ts"
        }"#;

        let payload: TsGoLintDiagnosticPayload = serde_json::from_str(json_with_fixes).unwrap();
        assert_eq!(payload.fixes.len(), 1);
        assert_eq!(payload.suggestions.len(), 0);
    }

    #[test]
    fn test_btreeset_preserves_rules_with_different_options() {
        use super::Rule;
        use std::collections::BTreeSet;

        // Create two rules with the same name but different options
        let rule1 = Rule {
            name: "no-floating-promises".to_string(),
            options: Some(serde_json::json!({"ignoreVoid": true})),
        };

        let rule2 = Rule {
            name: "no-floating-promises".to_string(),
            options: Some(serde_json::json!({"ignoreVoid": false})),
        };

        let rule3 = Rule { name: "no-floating-promises".to_string(), options: None };

        // Insert into BTreeSet
        let mut rules = BTreeSet::new();
        rules.insert(rule1.clone());
        rules.insert(rule2.clone());
        rules.insert(rule3.clone());

        // All three distinct rules should be preserved
        assert_eq!(rules.len(), 3, "BTreeSet should preserve all rules with different options");

        // Verify all rules are present
        assert!(rules.contains(&rule1), "Rule with ignoreVoid: true should be present");
        assert!(rules.contains(&rule2), "Rule with ignoreVoid: false should be present");
        assert!(rules.contains(&rule3), "Rule with no options should be present");
    }

    #[test]
    fn test_btreeset_deduplicates_identical_rules() {
        use super::Rule;
        use std::collections::BTreeSet;

        let rule1 = Rule {
            name: "no-floating-promises".to_string(),
            options: Some(serde_json::json!({"ignoreVoid": true})),
        };

        let rule2 = Rule {
            name: "no-floating-promises".to_string(),
            options: Some(serde_json::json!({"ignoreVoid": true})),
        };

        let mut rules = BTreeSet::new();
        rules.insert(rule1);
        rules.insert(rule2);

        // Identical rules should be deduplicated
        assert_eq!(rules.len(), 1, "BTreeSet should deduplicate identical rules");
    }
}
