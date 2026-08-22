use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use indexmap::IndexSet;
use oxc_diagnostics::{OxcDiagnostic, Severity};
use rustc_hash::{FxBuildHasher, FxHashMap};

use crate::{
    AllowWarnDeny, ConfigStore, Message, ModuleRecord, PossibleFixes,
    context::{RuleLabel, build_diagnostic},
    disable_directives::DisableDirectives,
    rules::RuleEnum,
};

/// All loaded modules per path. Multisegment filetypes like astro can have many modules
pub type ProjectModules<'a> = FxHashMap<&'a Path, &'a [Arc<ModuleRecord>]>;

/// Contains all of the state and context specific to this lint rule
///
/// Includes information like the project's modules, disable directives, config and paths
/// Unlike the per-file [`super::LintContext`], severity can differ per directory and should be derived from [`ProjectLintContext::target_paths`]
pub struct ProjectLintContext<'a> {
    modules: &'a ProjectModules<'a>,
    disable_directives: &'a Mutex<FxHashMap<PathBuf, DisableDirectives>>,
    config: &'a ConfigStore,
    paths_set: &'a IndexSet<Arc<OsStr>, FxBuildHasher>,
}

impl<'a> ProjectLintContext<'a> {
    pub fn new(
        modules: &'a ProjectModules<'a>,
        disable_directives: &'a Mutex<FxHashMap<PathBuf, DisableDirectives>>,
        config: &'a ConfigStore,
        paths_set: &'a IndexSet<Arc<OsStr>, FxBuildHasher>,
    ) -> Self {
        Self { modules, disable_directives, config, paths_set }
    }

    pub fn modules(&self) -> &ProjectModules<'a> {
        self.modules
    }

    pub fn disable_directives(&self, path: &Path) -> Option<DisableDirectives> {
        self.disable_directives.lock().expect("disable_directives poisoned").get(path).cloned()
    }

    /// finds severity and configuration per path for the enabled rule, using
    /// `extract` to access configuration
    pub fn target_paths<T: Copy>(
        &self,
        extract: impl Fn(&RuleEnum) -> Option<&T>,
    ) -> FxHashMap<PathBuf, (AllowWarnDeny, T)> {
        self.paths_set
            .iter()
            .filter_map(|path| {
                let path = Path::new(path.as_ref());
                let resolved = self.config.resolve(path);
                let (severity, rule_config) = resolved
                    .rules
                    .iter()
                    .find_map(|(rule, severity)| Some((severity, extract(rule)?)))?;
                Some((path.to_path_buf(), (*severity, *rule_config)))
            })
            .collect()
    }

    /// Checks suppression status at `path`, returning `None` if disabled.
    /// Otherwise, shifts a [`OxcDiagnostic`] by `section_offset` for multi
    /// segment files (eg astro) and applies `severity` and `rule`
    pub fn finalize_diagnostic(
        &self,
        path: &Path,
        diagnostic: OxcDiagnostic,
        rule: RuleLabel,
        severity: AllowWarnDeny,
        section_offset: u32,
    ) -> Option<OxcDiagnostic> {
        let mut message = Message::new(diagnostic, PossibleFixes::None);
        if section_offset != 0 {
            message.move_offset(section_offset);
        }
        let directives = self.disable_directives(path);
        build_diagnostic(
            message.error,
            message.span,
            rule,
            Severity::from(severity),
            directives.as_ref(),
        )
    }
}

/// Turns project diagnostics into LSP [`Message`]s, attaching disable pragma when `with_ignore_fixes` is `true`
/// attaches disable directives quick fixes to messages for the LSP when
pub fn diagnostics_to_messages(
    diagnostics: FxHashMap<PathBuf, Vec<OxcDiagnostic>>,
    with_ignore_fixes: bool,
    linted_source_text: &FxHashMap<PathBuf, String>,
) -> Vec<Message> {
    diagnostics
        .into_iter()
        .flat_map(|(path, diagnostics)| {
            let source_text = with_ignore_fixes.then(|| linted_source_text.get(&path)).flatten();
            diagnostics.into_iter().map(move |diagnostic| {
                let mut message = Message::new(diagnostic, PossibleFixes::None);
                if let Some(source_text) = &source_text {
                    message.add_ignore_fix(0, source_text);
                }
                message
            })
        })
        .collect()
}
