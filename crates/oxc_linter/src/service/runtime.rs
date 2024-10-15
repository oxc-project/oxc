use std::{
    borrow::Cow,
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    rc::Rc,
    sync::Arc,
};

use oxc_allocator::Allocator;
use oxc_diagnostics::{DiagnosticSender, DiagnosticService, Error, OxcDiagnostic};
use oxc_parser::{ParseOptions, Parser};
use oxc_resolver::Resolver;
use oxc_semantic::SemanticBuilder;
use oxc_span::{SourceType, VALID_EXTENSIONS};
use rayon::{iter::ParallelBridge, prelude::ParallelIterator};
use rustc_hash::FxHashSet;

use crate::{
    loader::{JavaScriptSource, PartialLoader, LINT_PARTIAL_LOADER_EXT},
    utils::read_to_string,
    Fixer, Linter, Message,
};

use super::{
    module_cache::{ModuleCache, ModuleState},
    LintServiceOptions,
};

pub struct Runtime {
    cwd: Box<Path>,
    /// All paths to lint
    paths: FxHashSet<Box<Path>>,
    pub(super) linter: Linter,
    resolver: Option<Resolver>,
    modules: ModuleCache,
}

impl Runtime {
    pub(super) fn new(linter: Linter, options: LintServiceOptions) -> Self {
        let resolver = options.cross_module.then(|| {
            Self::get_resolver(options.tsconfig.or_else(|| Some(options.cwd.join("tsconfig.json"))))
        });
        Self {
            cwd: options.cwd,
            paths: options.paths.iter().cloned().collect(),
            linter,
            resolver,
            modules: ModuleCache::default(),
        }
    }

    fn get_resolver(tsconfig: Option<PathBuf>) -> Resolver {
        use oxc_resolver::{ResolveOptions, TsconfigOptions, TsconfigReferences};
        let tsconfig = tsconfig.and_then(|path| {
            if path.is_file() {
                Some(TsconfigOptions { config_file: path, references: TsconfigReferences::Auto })
            } else {
                None
            }
        });

        Resolver::new(ResolveOptions {
            extensions: VALID_EXTENSIONS.iter().map(|ext| format!(".{ext}")).collect(),
            condition_names: vec!["module".into(), "require".into()],
            tsconfig,
            ..ResolveOptions::default()
        })
    }

    fn get_source_type_and_text(
        path: &Path,
        ext: &str,
    ) -> Option<Result<(SourceType, String), Error>> {
        let source_type = SourceType::from_path(path);
        let not_supported_yet =
            source_type.as_ref().is_err_and(|_| !LINT_PARTIAL_LOADER_EXT.contains(&ext));
        if not_supported_yet {
            return None;
        }
        let source_type = source_type.unwrap_or_default();
        let file_result = read_to_string(path).map_err(|e| {
            Error::new(OxcDiagnostic::error(format!(
                "Failed to open file {path:?} with error \"{e}\""
            )))
        });
        Some(match file_result {
            Ok(source_text) => Ok((source_type, source_text)),
            Err(e) => Err(e),
        })
    }

    // clippy: the source field is checked and assumed to be less than 4GB, and
    // we assume that the fix offset will not exceed 2GB in either direction
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    pub(super) fn process_path(&self, path: &Path, tx_error: &DiagnosticSender) {
        if self.init_cache_state(path) {
            return;
        }

        let Some(ext) = path.extension().and_then(OsStr::to_str) else {
            self.ignore_path(path);
            return;
        };

        let Some(source_type_and_text) = Self::get_source_type_and_text(path, ext) else {
            self.ignore_path(path);
            return;
        };

        let (source_type, source_text) = match source_type_and_text {
            Ok(source_text) => source_text,
            Err(e) => {
                self.ignore_path(path);
                tx_error.send(Some((path.to_path_buf(), vec![e]))).unwrap();
                return;
            }
        };

        let sources = PartialLoader::parse(ext, &source_text)
            .unwrap_or_else(|| vec![JavaScriptSource::partial(&source_text, source_type, 0)]);

        if sources.is_empty() {
            self.ignore_path(path);
            return;
        }

        // If there are fixes, we will accumulate all of them and write to the file at the end.
        // This means we do not write multiple times to the same file if there are multiple sources
        // in the same file (for example, multiple scripts in an `.astro` file).
        let mut new_source_text = Cow::from(&source_text);
        // This is used to keep track of the cumulative offset from applying fixes.
        // Otherwise, spans for fixes will be incorrect due to varying size of the
        // source code after each fix.
        let mut fix_offset: i32 = 0;

        let mut allocator = Allocator::default();
        for (i, source) in sources.into_iter().enumerate() {
            if i >= 1 {
                allocator.reset();
            }
            let mut messages = self.process_source(
                path,
                &allocator,
                source.source_text,
                source.source_type,
                true,
                tx_error,
            );

            if self.linter.options().fix.is_some() {
                let fix_result = Fixer::new(source.source_text, messages).fix();
                if fix_result.fixed {
                    // write to file, replacing only the changed part
                    let start = source.start.saturating_add_signed(fix_offset) as usize;
                    let end = start + source.source_text.len();
                    new_source_text.to_mut().replace_range(start..end, &fix_result.fixed_code);
                    let old_code_len = source.source_text.len() as u32;
                    let new_code_len = fix_result.fixed_code.len() as u32;
                    fix_offset += new_code_len as i32;
                    fix_offset -= old_code_len as i32;
                }
                messages = fix_result.messages;
            }

            if !messages.is_empty() {
                self.ignore_path(path);
                let errors = messages.into_iter().map(Into::into).collect();
                let path = path.strip_prefix(&self.cwd).unwrap_or(path);
                let diagnostics =
                    DiagnosticService::wrap_diagnostics(path, source.source_text, errors);
                tx_error.send(Some(diagnostics)).unwrap();
            }
        }

        // If the new source text is owned, that means it was modified,
        // so we write the new source text to the file.
        if let Cow::Owned(new_source_text) = new_source_text {
            fs::write(path, new_source_text).unwrap();
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn process_source<'a>(
        &self,
        path: &Path,
        allocator: &'a Allocator,
        source_text: &'a str,
        source_type: SourceType,
        check_syntax_errors: bool,
        tx_error: &DiagnosticSender,
    ) -> Vec<Message<'a>> {
        let ret = Parser::new(allocator, source_text, source_type)
            .with_options(ParseOptions {
                parse_regular_expression: true,
                allow_return_outside_function: true,
                ..ParseOptions::default()
            })
            .parse();

        if !ret.errors.is_empty() {
            return ret.errors.into_iter().map(|err| Message::new(err, None)).collect();
        };

        // Build the module record to unblock other threads from waiting for too long.
        // The semantic model is not built at this stage.
        let semantic_builder = SemanticBuilder::new()
            .with_cfg(true)
            .with_build_jsdoc(true)
            .with_check_syntax_error(check_syntax_errors)
            .build_module_record(path, &ret.program);
        let module_record = semantic_builder.module_record();

        if self.resolver.is_some() {
            self.modules.add_resolved_module(path, Arc::clone(&module_record));

            // Retrieve all dependency modules from this module.
            let dir = path.parent().unwrap();
            module_record
                .requested_modules
                .keys()
                .par_bridge()
                .map_with(self.resolver.as_ref().unwrap(), |resolver, specifier| {
                    resolver.resolve(dir, specifier).ok().map(|r| (specifier, r))
                })
                .flatten()
                .for_each_with(tx_error, |tx_error, (specifier, resolution)| {
                    let path = resolution.path();
                    self.process_path(path, tx_error);
                    let Some(target_module_record_ref) = self.modules.get(path) else {
                        return;
                    };
                    let ModuleState::Resolved(target_module_record) =
                        target_module_record_ref.value()
                    else {
                        return;
                    };
                    // Append target_module to loaded_modules
                    module_record
                        .loaded_modules
                        .insert(specifier.clone(), Arc::clone(target_module_record));
                });

            // The thread is blocked here until all dependent modules are resolved.

            // Resolve and append `star_export_bindings`
            for export_entry in &module_record.star_export_entries {
                let Some(remote_module_record_ref) =
                    export_entry.module_request.as_ref().and_then(|module_request| {
                        module_record.loaded_modules.get(module_request.name())
                    })
                else {
                    continue;
                };
                let remote_module_record = remote_module_record_ref.value();

                // Append both remote `bindings` and `exported_bindings_from_star_export`
                let remote_exported_bindings_from_star_export = remote_module_record
                    .exported_bindings_from_star_export
                    .iter()
                    .flat_map(|r| r.value().clone());
                let remote_bindings = remote_module_record
                    .exported_bindings
                    .keys()
                    .cloned()
                    .chain(remote_exported_bindings_from_star_export)
                    .collect::<Vec<_>>();
                module_record
                    .exported_bindings_from_star_export
                    .entry(remote_module_record.resolved_absolute_path.clone())
                    .or_default()
                    .value_mut()
                    .extend(remote_bindings);
            }

            // Stop if the current module is not marked for lint.
            if !self.paths.contains(path) {
                return vec![];
            }
        }

        let semantic_ret = semantic_builder.build(&ret.program);

        if !semantic_ret.errors.is_empty() {
            return semantic_ret.errors.into_iter().map(|err| Message::new(err, None)).collect();
        };

        let mut semantic = semantic_ret.semantic;
        semantic.set_irregular_whitespaces(ret.irregular_whitespaces);
        self.linter.run(path, Rc::new(semantic))
    }

    pub(super) fn init_cache_state(&self, path: &Path) -> bool {
        if self.resolver.is_none() {
            return false;
        }

        self.modules.init_cache_state(path)
    }

    fn ignore_path(&self, path: &Path) {
        self.resolver.is_some().then(|| self.modules.ignore_path(path));
    }

    pub(super) fn number_of_dependencies(&self) -> usize {
        self.modules.len() - self.paths.len()
    }

    pub(super) fn iter_paths(&self) -> impl Iterator<Item = &Box<Path>> + '_ {
        self.paths.iter()
    }
}
