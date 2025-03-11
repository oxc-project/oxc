use std::{
    borrow::Cow,
    ffi::OsStr,
    fs,
    mem::take,
    path::{Path, PathBuf},
    rc::Rc,
    sync::{Arc, mpsc},
};

use indexmap::IndexSet;
use rayon::iter::ParallelDrainRange;
use rayon::{Scope, iter::IntoParallelRefIterator, prelude::ParallelIterator};
use rustc_hash::{FxBuildHasher, FxHashMap, FxHashSet};
use self_cell::self_cell;
use smallvec::SmallVec;

use oxc_allocator::Allocator;
use oxc_diagnostics::{DiagnosticSender, DiagnosticService, Error, OxcDiagnostic};
use oxc_parser::{ParseOptions, Parser};
use oxc_resolver::Resolver;
use oxc_semantic::{Semantic, SemanticBuilder};
use oxc_span::{CompactStr, SourceType, VALID_EXTENSIONS};

use super::LintServiceOptions;
use crate::{
    Fixer, Linter, Message,
    loader::{JavaScriptSource, LINT_PARTIAL_LOADER_EXT, PartialLoader},
    module_record::ModuleRecord,
    utils::read_to_string,
};

pub struct Runtime {
    cwd: Box<Path>,
    /// All paths to lint
    paths: IndexSet<Arc<OsStr>, FxBuildHasher>,
    pub(super) linter: Linter,
    resolver: Option<Resolver>,

    #[cfg(test)]
    pub(super) test_source: std::sync::RwLock<Option<String>>,
}

/// Output of `Runtime::process_path`
struct ModuleProcessOutput {
    /// All paths in `Runtime` are stored as `OsStr`, because `OsStr` hash is faster
    /// than `Path` - go checkout their source code.
    path: Arc<OsStr>,
    processed_module: ProcessedModule,
}

/// A module processed from a path
#[derive(Default)]
struct ProcessedModule {
    /// Module records of source sections, or diagnostics if parsing failed on that section.
    ///
    /// Modules with special extensions such as .vue could contain multiple source sections (see `PartialLoader::PartialLoader`).
    /// Plain ts/js modules have one section. Using `SmallVec` to avoid allocations for plain modules.
    section_module_records: SmallVec<[Result<ResolvedModuleRecord, Vec<OxcDiagnostic>>; 1]>,

    /// Source code and semantic of the module.
    ///
    /// This value is required for linter to run on the module. If import plugin is enabled,
    /// dependencies are also processed as `ProcessedModule` to construct the module graph, but
    /// not for linting. For these modules, `content` is set to `None`.
    content: Option<ModuleContent>,
}

struct ResolvedModuleRequest {
    specifier: CompactStr,
    resolved_requested_path: Arc<OsStr>,
}

/// ModuleRecord with all specifiers in import statements resolved to real paths.
struct ResolvedModuleRecord {
    module_record: Arc<ModuleRecord>,
    resolved_module_requests: Vec<ResolvedModuleRequest>,
}

self_cell! {
    struct ModuleContent {
        owner: ModuleContentOwner,
        #[not_covariant]
        dependent: SectionContents,
    }
}
// Safety: dependent borrows from owner. They're safe to be sent together.
unsafe impl Send for ModuleContent {}

struct ModuleContentOwner {
    source_text: String,
    allocator: Allocator,
}

/// source text and semantic for each source section. They are in the same order as `ProcessedModule.section_module_records`
type SectionContents<'a> = SmallVec<[SectionContent<'a>; 1]>;
struct SectionContent<'a> {
    source: JavaScriptSource<'a>,
    /// None if section parsing failed. The corresponding item with the same index in
    /// `ProcessedModule.section_module_records` would be `Err(Vec<OxcDiagnostic>)`.
    semantic: Option<Semantic<'a>>,
}

/// A module with its source text and semantic, ready to be linted.
///
/// A `ModuleWithContent` is generated for each path in `runtime.paths`. It's basically the same
/// as `ProcessedModule`, except `content` is non-Option.
struct ModuleToLint {
    path: Arc<OsStr>,
    section_module_records: SmallVec<[Result<Arc<ModuleRecord>, Vec<OxcDiagnostic>>; 1]>,
    content: ModuleContent,
}
impl ModuleToLint {
    fn from_processed_module(path: Arc<OsStr>, processed_module: ProcessedModule) -> Option<Self> {
        processed_module.content.map(|content| Self {
            path,
            section_module_records: processed_module
                .section_module_records
                .into_iter()
                .map(|record_result| record_result.map(|ok| ok.module_record))
                .collect(),
            content,
        })
    }
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
            #[cfg(test)]
            test_source: std::sync::RwLock::new(None),
        }
    }

    fn get_resolver(tsconfig_path: Option<PathBuf>) -> Resolver {
        use oxc_resolver::{ResolveOptions, TsconfigOptions, TsconfigReferences};
        let tsconfig = tsconfig_path.and_then(|path| {
            path.is_file().then_some(TsconfigOptions {
                config_file: path,
                references: TsconfigReferences::Auto,
            })
        });
        let extension_alias = tsconfig.as_ref().map_or_else(Vec::new, |_| {
            vec![
                (".js".into(), vec![".js".into(), ".ts".into()]),
                (".mjs".into(), vec![".mjs".into(), ".mts".into()]),
                (".cjs".into(), vec![".cjs".into(), ".cts".into()]),
            ]
        });
        Resolver::new(ResolveOptions {
            extensions: VALID_EXTENSIONS.iter().map(|ext| format!(".{ext}")).collect(),
            main_fields: vec!["module".into(), "main".into()],
            condition_names: vec!["module".into(), "import".into()],
            extension_alias,
            tsconfig,
            ..ResolveOptions::default()
        })
    }

    #[cfg_attr(not(test), expect(clippy::unused_self))]
    fn get_source_type_and_text(
        &self,
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

        #[cfg(test)]
        if let (true, Some(test_source)) =
            (self.paths.contains(path.as_os_str()), &*self.test_source.read().unwrap())
        {
            return Some(Ok((source_type, test_source.clone())));
        }
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

    /// Prepare entry modules for linting.
    ///
    /// `on_module_to_lint` is called for each entry modules in `self.paths` when it's ready for linting,
    /// which means all its dependencies are resolved if import plugin is enabled.
    fn resolve_modules<'a>(
        &'a mut self,
        scope: &Scope<'a>,
        check_syntax_errors: bool,
        tx_error: &'a DiagnosticSender,
        on_module_to_lint: impl Fn(&'a Self, ModuleToLint) + Send + Sync + Clone + 'a,
    ) {
        if self.resolver.is_none() {
            self.paths.par_iter().for_each(|path| {
                let output = self.process_path(Arc::clone(path), check_syntax_errors, tx_error);
                let entry =
                    ModuleToLint::from_processed_module(output.path, output.processed_module)
                        .unwrap();
                on_module_to_lint(self, entry);
            });
            return;
        }
        // The goal of code below is to construct the module graph bootstrapped by the entry modules (`self.paths`),
        // and call `on_entry` when all dependencies of that entry is resolved. We want to call `on_entry` for each
        // entry as soon as possible, so that the memory for source texts and semantics can be released early.

        // Sorting paths to make deeper paths appear first.
        // Consider a typical scenario:
        //
        // - src/index.js
        // - src/a/foo.js
        // - src/b/bar.js
        // ..... (thousands of sources)
        // - src/very/deep/path/baz.js
        //
        // All paths above are in `self.paths`. `src/index.js`, the entrypoint of the application, references
        // almost all the other paths as its direct or indirect dependencies.
        //
        // If we construct the module graph starting from `src/index.js`, contents (sources and semantics) of
        // all these paths must stay in memory (because they are both entries and part of `src/index.js` dependencies)
        // until the last dependency is processed.
        // The more efficient way is to start from "leaf" modules: their dependencies are ready earlier, thus we
        // can run lint on them and then released their content earlier.
        //
        // But it's impossible to know which ones are "leaf" modules before parsing even starts. Here we assume
        // deeper paths are more likely to be leaf modules  (src/very/deep/path/baz.js is likely to have
        // fewer dependencies than src/index.js).
        // This heuristic is not always true, but it works well enough for real world codebases.
        self.paths.par_sort_unstable_by(|a, b| Path::new(b).cmp(Path::new(a)));

        // The general idea is processing `self.paths` and their dependencies in groups. We start from a group of modules
        // in `self.paths` that is small enough to hold in memory but big enough to make use of the rayon thread pool.
        // We build the module graph from one group, run lint on them, drop sources and semantics but keep the module
        // graph, and then move on to the next group.
        // This size is empirical based on AFFiNE@97cc814a.
        let group_size = rayon::current_num_threads() * 4;

        // Stores modules that belongs to `self.paths` in current group.
        // They are passed to `on_module_to_lint` at the end of each group.
        let mut modules_to_lint: Vec<ModuleToLint> = Vec::with_capacity(group_size);

        // Set self to immutable reference so it can be shared among spawned tasks.
        let me: &Self = self;

        // The module graph keyed by module paths. It is looked up when populating `loaded_modules`.
        // The values are module records of sections (check the docs of `ProcessedModule.section_module_records`)
        // Its entries are kept across groups because modules discovered in former groups could be referenced by modules in latter groups.
        let mut modules_by_path =
            FxHashMap::<Arc<OsStr>, SmallVec<[Arc<ModuleRecord>; 1]>>::with_capacity_and_hasher(
                me.paths.len(),
                FxBuildHasher,
            );

        // `encountered_paths` prevents duplicated processing.
        // It is a superset of keys of `modules_by_path` as it also contains paths that are queued to process.
        let mut encountered_paths =
            FxHashSet::<Arc<OsStr>>::with_capacity_and_hasher(me.paths.len(), FxBuildHasher);

        // Resolved module requests from modules in current group.
        // This is used to populate `loaded_modules` at the end of each group.
        let mut module_paths_and_resolved_requests =
            Vec::<(Arc<OsStr>, SmallVec<[Vec<ResolvedModuleRequest>; 1]>)>::new();

        // There are two sets of threads: threads for the graph and threads for the modules.
        // - The graph thread is the one thread that calls `resolve_modules`. It's the only thread that updates the module graph, so no need for locks.
        // - Module threads accept paths and produces `ModuleProcessOutput` (the logic is in `self.process_path`). They are isolated to each
        //   other and paralleled in the rayon thread pool.

        // This channel is for posting `ModuleProcessOutput` from module threads to the graph thread.
        let (tx_process_output, rx_process_output) = mpsc::channel::<ModuleProcessOutput>();

        // The cursor of `self.paths` that points to the start path of the next group.
        let mut group_start = 0usize;

        // The group loop. Each iteration of this loop processes a group of modules.
        while group_start < me.paths.len() {
            // How many modules are queued but not processed in this group.
            let mut pending_module_count = 0;

            // Bootstrap the group by processing modules to be linted.
            while pending_module_count < group_size && group_start < me.paths.len() {
                let path = &me.paths[group_start];
                group_start += 1;

                // Check if this module to be linted is already processed as a dependency in former groups
                if encountered_paths.insert(Arc::clone(path)) {
                    pending_module_count += 1;
                    let path = Arc::clone(path);
                    let tx_process_output = tx_process_output.clone();
                    scope.spawn(move |_| {
                        tx_process_output
                            .send(me.process_path(path, check_syntax_errors, tx_error))
                            .unwrap();
                    });
                }
            }

            // Loop until all queued modules in this group are processed.
            // Each iteration adds one module to the module graph.
            while pending_module_count > 0 {
                let Ok(ModuleProcessOutput { path, mut processed_module }) =
                    // Most heavy-lifting is done in the module threads. The graph thread would be mostly idle if it
                    // only updates the graph and blocks on awaiting `rx_process_output`.
                    // To avoid this waste, the graph module peeks the `rx_process_output` without blocking, and ...
                    rx_process_output.try_recv()
                else {
                    // yield if `rx_process_output` is empty, giving rayon chances to dispatch module processing or linting to this thread.
                    rayon::yield_now();
                    continue;
                };
                pending_module_count -= 1;

                // Spawns tasks for processing dependencies to module threads
                for record_result in &processed_module.section_module_records {
                    let Ok(record) = record_result.as_ref() else {
                        continue;
                    };
                    for request in &record.resolved_module_requests {
                        let dep_path = &request.resolved_requested_path;
                        if encountered_paths.insert(Arc::clone(dep_path)) {
                            scope.spawn({
                                let tx_resolve_output = tx_process_output.clone();
                                let dep_path = Arc::clone(dep_path);
                                move |_| {
                                    tx_resolve_output
                                        .send(me.process_path(
                                            dep_path,
                                            check_syntax_errors,
                                            tx_error,
                                        ))
                                        .unwrap();
                                }
                            });
                            pending_module_count += 1;
                        }
                    }
                }

                // Populate this module to `modules_by_path`
                modules_by_path.insert(
                    Arc::clone(&path),
                    processed_module
                        .section_module_records
                        .iter()
                        .filter_map(|resolved_module_record| {
                            Some(Arc::clone(&resolved_module_record.as_ref().ok()?.module_record))
                        })
                        .collect(),
                );

                // We want to write to `loaded_modules` when the dependencies of this module are processed, but it's hard
                // to track when that happens, so here we store dependency relationships in `module_paths_and_resolved_requests`,
                // and use it to populate `loaded_modules` after `pending_module_count` reaches 0. That's when all dependencies
                // in this group are processed.
                module_paths_and_resolved_requests.push((
                    Arc::clone(&path),
                    processed_module
                        .section_module_records
                        .iter_mut()
                        .filter_map(|record_result| {
                            Some(take(&mut record_result.as_mut().ok()?.resolved_module_requests))
                        })
                        .collect(),
                ));

                // This module has `content` which means it's one of `self.paths`.
                // Store it to `modules_to_lint`
                if let Some(entry_module) =
                    ModuleToLint::from_processed_module(path, processed_module)
                {
                    modules_to_lint.push(entry_module);
                }
            } // while pending_module_count > 0

            // Now all dependencies in this group are processed.
            // Writing to `loaded_modules` based on `module_paths_and_resolved_requests`
            module_paths_and_resolved_requests.par_drain(..).for_each(|(path, requested_module_paths)| {
                if requested_module_paths.is_empty() {
                    return;
                }
                let records = &modules_by_path[&path];
                assert_eq!(
                    records.len(), requested_module_paths.len(),
                    "This is an internal logic error. Please file an issue at https://github.com/oxc-project/oxc/issues",
                );
                for (record, requested_module_paths) in
                    records.iter().zip(requested_module_paths.into_iter())
                {
                    let mut loaded_modules = record.loaded_modules.write().unwrap();
                    for request in requested_module_paths {
                        // TODO: revise how to store multiple sections in loaded_modules
                        let Some(dep_module_record) =
                            modules_by_path[&request.resolved_requested_path].last()
                        else {
                            continue;
                        };
                        loaded_modules.insert(request.specifier, Arc::clone(dep_module_record));
                    }
                }
            });
            #[expect(clippy::iter_with_drain)]
            for entry in modules_to_lint.drain(..) {
                let on_entry = on_module_to_lint.clone();
                scope.spawn(move |_| {
                    on_entry(me, entry);
                });
            }
        }
    }

    // clippy: the source field is checked and assumed to be less than 4GB, and
    // we assume that the fix offset will not exceed 2GB in either direction
    #[expect(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    pub(super) fn run(&mut self, tx_error: &DiagnosticSender) {
        rayon::scope(|scope| {
            self.resolve_modules(scope, true, tx_error, |me, mut module_to_lint| {
                module_to_lint.content.with_dependent_mut(|owner, sections| {
                    // If there are fixes, we will accumulate all of them and write to the file at the end.
                    // This means we do not write multiple times to the same file if there are multiple sources
                    // in the same file (for example, multiple scripts in an `.astro` file).
                    let mut new_source_text = Cow::from(owner.source_text.as_str());
                    // This is used to keep track of the cumulative offset from applying fixes.
                    // Otherwise, spans for fixes will be incorrect due to varying size of the
                    // source code after each fix.
                    let mut fix_offset: i32 = 0;

                    let path = Path::new(&module_to_lint.path);

                    assert_eq!(module_to_lint.section_module_records.len(), sections.len());
                    for (record_result, section) in
                        module_to_lint.section_module_records.into_iter().zip(sections.drain(..))
                    {
                        let mut messages = match record_result {
                            Ok(module_record) => me.linter.run(
                                path,
                                Rc::new(section.semantic.unwrap()),
                                Arc::clone(&module_record),
                            ),
                            Err(errors) => {
                                errors.into_iter().map(|err| Message::new(err, None)).collect()
                            }
                        };

                        let source_text = section.source.source_text;
                        if me.linter.options().fix.is_some() {
                            let fix_result = Fixer::new(source_text, messages).fix();
                            if fix_result.fixed {
                                // write to file, replacing only the changed part
                                let start =
                                    section.source.start.saturating_add_signed(fix_offset) as usize;
                                let end = start + source_text.len();
                                new_source_text
                                    .to_mut()
                                    .replace_range(start..end, &fix_result.fixed_code);
                                let old_code_len = source_text.len() as u32;
                                let new_code_len = fix_result.fixed_code.len() as u32;
                                fix_offset += new_code_len as i32;
                                fix_offset -= old_code_len as i32;
                            }
                            messages = fix_result.messages;
                        }

                        if !messages.is_empty() {
                            let errors = messages.into_iter().map(Into::into).collect();
                            let path = path.strip_prefix(&me.cwd).unwrap_or(path);
                            let diagnostics = DiagnosticService::wrap_diagnostics(
                                path,
                                &owner.source_text,
                                section.source.start,
                                errors,
                            );
                            tx_error.send(Some(diagnostics)).unwrap();
                        }
                    }
                    // If the new source text is owned, that means it was modified,
                    // so we write the new source text to the file.
                    if let Cow::Owned(new_source_text) = new_source_text {
                        fs::write(path, new_source_text).unwrap();
                    }
                });
            });
        });
    }

    #[cfg(test)]
    pub(super) fn run_source<'a>(
        &mut self,
        allocator: &'a Allocator,
        source_text: &str,
        check_syntax_errors: bool,
        tx_error: &DiagnosticSender,
    ) -> Vec<Message<'a>> {
        use oxc_allocator::CloneIn;
        use std::sync::Mutex;

        *self.test_source.write().unwrap() = Some(source_text.to_owned());

        let messages = Mutex::new(Vec::<Message<'a>>::new());
        rayon::scope(|scope| {
            self.resolve_modules(scope, check_syntax_errors, tx_error, |me, mut module| {
                module.content.with_dependent_mut(|_owner, dependent| {
                    assert_eq!(module.section_module_records.len(), dependent.len());
                    for (record_result, section) in
                        module.section_module_records.into_iter().zip(dependent.drain(..))
                    {
                        messages.lock().unwrap().extend(
                            match record_result {
                                Ok(module_record) => me.linter.run(
                                    Path::new(&module.path),
                                    Rc::new(section.semantic.unwrap()),
                                    Arc::clone(&module_record),
                                ),
                                Err(errors) => {
                                    errors.into_iter().map(|err| Message::new(err, None)).collect()
                                }
                            }
                            .into_iter()
                            .map(|message| message.clone_in(allocator)),
                        );
                    }
                });
            });
        });
        messages.into_inner().unwrap()
    }

    fn process_path(
        &self,
        path: Arc<OsStr>,
        check_syntax_errors: bool,
        tx_error: &DiagnosticSender,
    ) -> ModuleProcessOutput {
        let Some(ext) = Path::new(&path).extension().and_then(OsStr::to_str) else {
            return ModuleProcessOutput { path, processed_module: ProcessedModule::default() };
        };
        let Some(source_type_and_text) = self.get_source_type_and_text(Path::new(&path), ext)
        else {
            return ModuleProcessOutput { path, processed_module: ProcessedModule::default() };
        };

        let (source_type, source_text) = match source_type_and_text {
            Ok(source_text) => source_text,
            Err(e) => {
                tx_error.send(Some((Path::new(&path).to_path_buf(), vec![e]))).unwrap();
                return ModuleProcessOutput { path, processed_module: ProcessedModule::default() };
            }
        };
        let mut records = SmallVec::<[Result<ResolvedModuleRecord, Vec<OxcDiagnostic>>; 1]>::new();
        let mut module_content: Option<ModuleContent> = None;
        let allocator = Allocator::default();
        if self.paths.contains(&path) {
            module_content =
                Some(ModuleContent::new(ModuleContentOwner { source_text, allocator }, |owner| {
                    let mut section_contents = SmallVec::new();
                    records = self.process_source(
                        Path::new(&path),
                        ext,
                        check_syntax_errors,
                        source_type,
                        owner.source_text.as_str(),
                        &owner.allocator,
                        Some(&mut section_contents),
                    );
                    section_contents
                }));
        } else {
            records = self.process_source(
                Path::new(&path),
                ext,
                check_syntax_errors,
                source_type,
                source_text.as_str(),
                &allocator,
                None,
            );
        }

        ModuleProcessOutput {
            path,
            processed_module: ProcessedModule {
                section_module_records: records,
                content: module_content,
            },
        }
    }

    #[expect(clippy::too_many_arguments)]
    fn process_source<'a>(
        &self,
        path: &Path,
        ext: &str,
        check_syntax_errors: bool,
        source_type: SourceType,
        source_text: &'a str,
        allocator: &'a Allocator,
        mut out_sections: Option<&mut SectionContents<'a>>,
    ) -> SmallVec<[Result<ResolvedModuleRecord, Vec<OxcDiagnostic>>; 1]> {
        let section_sources = PartialLoader::parse(ext, source_text)
            .unwrap_or_else(|| vec![JavaScriptSource::partial(source_text, source_type, 0)]);

        let mut section_module_records = SmallVec::<
            [Result<ResolvedModuleRecord, Vec<OxcDiagnostic>>; 1],
        >::with_capacity(section_sources.len());
        for section_source in section_sources {
            match self.process_source_section(
                path,
                allocator,
                section_source.source_text,
                section_source.source_type,
                check_syntax_errors,
            ) {
                Ok((record, semantic)) => {
                    section_module_records.push(Ok(record));
                    if let Some(sections) = &mut out_sections {
                        sections.push(SectionContent {
                            source: section_source,
                            semantic: Some(semantic),
                        });
                    }
                }
                Err(err) => {
                    section_module_records.push(Err(err));
                    if let Some(sections) = &mut out_sections {
                        sections.push(SectionContent { source: section_source, semantic: None });
                    }
                }
            }
        }
        section_module_records
    }

    fn process_source_section<'a>(
        &self,
        path: &Path,
        allocator: &'a Allocator,
        source_text: &'a str,
        source_type: SourceType,
        check_syntax_errors: bool,
    ) -> Result<(ResolvedModuleRecord, Semantic<'a>), Vec<OxcDiagnostic>> {
        let ret = Parser::new(allocator, source_text, source_type)
            .with_options(ParseOptions {
                parse_regular_expression: true,
                allow_return_outside_function: true,
                ..ParseOptions::default()
            })
            .parse();

        if !ret.errors.is_empty() {
            return Err(if ret.is_flow_language { vec![] } else { ret.errors });
        };

        let semantic_ret = SemanticBuilder::new()
            .with_cfg(true)
            .with_scope_tree_child_ids(true)
            .with_build_jsdoc(true)
            .with_check_syntax_error(check_syntax_errors)
            .build(allocator.alloc(ret.program));

        if !semantic_ret.errors.is_empty() {
            return Err(semantic_ret.errors);
        };

        let mut semantic = semantic_ret.semantic;
        semantic.set_irregular_whitespaces(ret.irregular_whitespaces);

        let module_record = Arc::new(ModuleRecord::new(path, &ret.module_record, &semantic));

        let mut resolved_module_requests: Vec<ResolvedModuleRequest> = vec![];

        // If import plugin is enabled.
        if let Some(resolver) = &self.resolver {
            // Retrieve all dependent modules from this module.
            let dir = path.parent().unwrap();
            resolved_module_requests = module_record
                .requested_modules
                .keys()
                .filter_map(|specifier| {
                    let resolution = resolver.resolve(dir, specifier).ok()?;
                    Some(ResolvedModuleRequest {
                        specifier: specifier.clone(),
                        resolved_requested_path: Arc::<OsStr>::from(resolution.path().as_os_str()),
                    })
                })
                .collect();
        }
        Ok((ResolvedModuleRecord { module_record, resolved_module_requests }, semantic))
    }
}
