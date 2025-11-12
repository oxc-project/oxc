use std::{
    borrow::Cow,
    ffi::OsStr,
    fs,
    hash::BuildHasherDefault,
    mem::take,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, mpsc},
};

use indexmap::IndexSet;
use rayon::iter::ParallelDrainRange;
use rayon::{
    Scope,
    iter::IntoParallelRefIterator,
    prelude::{ParallelIterator, ParallelSliceMut},
};
use rustc_hash::{FxBuildHasher, FxHashMap, FxHashSet, FxHasher};
use self_cell::self_cell;
use smallvec::SmallVec;

use oxc_allocator::{Allocator, AllocatorGuard, AllocatorPool};
use oxc_diagnostics::{DiagnosticSender, DiagnosticService, Error, OxcDiagnostic};
use oxc_parser::{ParseOptions, Parser};
use oxc_resolver::Resolver;
use oxc_semantic::{Semantic, SemanticBuilder};
use oxc_span::{CompactStr, SourceType, VALID_EXTENSIONS};

use crate::{
    Fixer, Linter, Message, PossibleFixes,
    context::ContextSubHost,
    disable_directives::DisableDirectives,
    loader::{JavaScriptSource, LINT_PARTIAL_LOADER_EXTENSIONS, PartialLoader},
    module_record::ModuleRecord,
    utils::read_to_arena_str,
};

use super::LintServiceOptions;

type ModulesByPath =
    papaya::HashMap<Arc<OsStr>, SmallVec<[Arc<ModuleRecord>; 1]>, BuildHasherDefault<FxHasher>>;

pub struct Runtime {
    cwd: Box<Path>,
    pub(super) linter: Linter,
    resolver: Option<Resolver>,

    allocator_pool: AllocatorPool,

    /// The module graph keyed by module paths. It is looked up when populating `loaded_modules`.
    /// The values are module records of sections (check the docs of `ProcessedModule.section_module_records`)
    /// Its entries are kept across groups because modules discovered in former groups could be referenced by modules in latter groups.
    ///
    /// `ModuleRecord` is a cyclic data structure.
    /// To make sure all `ModuleRecord` gets dropped after `Runtime` is dropped,
    /// `modules_by_path` must own `ModuleRecord` with `Arc`, all other references must use `Weak<ModuleRecord>`.
    modules_by_path: ModulesByPath,
    /// Collected disable directives from linted files
    disable_directives_map: Arc<Mutex<FxHashMap<PathBuf, DisableDirectives>>>,
}

/// Output of `Runtime::process_path`
struct ModuleProcessOutput<'alloc_pool> {
    /// All paths in `Runtime` are stored as `OsStr`, because `OsStr` hash is faster
    /// than `Path` - go checkout their source code.
    path: Arc<OsStr>,
    processed_module: ProcessedModule<'alloc_pool>,
}

/// A module processed from a path
#[derive(Default)]
struct ProcessedModule<'alloc_pool> {
    /// Module records of source sections, or diagnostics if parsing failed on that section.
    ///
    /// Modules with special extensions such as .vue could contain multiple source sections (see `PartialLoader::PartialLoader`).
    /// Plain ts/js modules have one section. Using `SmallVec` to avoid allocations for plain modules.
    section_module_records: SmallVec<[Result<ResolvedModuleRecord, Vec<OxcDiagnostic>>; 1]>,

    /// Source code and semantic of the module.
    ///
    /// This value is required for linter to run on the module.  There are two cases where `content` is `None`:
    /// - Import plugin is enabled and the module is a dependency, which is processed only to construct the module graph, not for linting.
    /// - Couldn't get the source text of the module to lint, e.g. the file doesn't exist or the source isn't valid utf-8.
    ///
    /// Note that `content` is `Some` even if parsing is unsuccessful as long as the source to lint is valid utf-8.
    /// It is designed this way to cover the case where some but not all the sections fail to parse.
    content: Option<ModuleContent<'alloc_pool>>,
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
    struct ModuleContent<'alloc_pool> {
        owner: AllocatorGuard<'alloc_pool>,
        #[not_covariant]
        dependent: ModuleContentDependent,
    }
}
struct ModuleContentDependent<'a> {
    source_text: &'a str,
    section_contents: SectionContents<'a>,
}

// Safety: dependent borrows from owner. They're safe to be sent together.
unsafe impl Send for ModuleContent<'_> {}

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
/// as [`ProcessedModule`], except `content` is non-Option.
struct ModuleToLint<'alloc_pool> {
    path: Arc<OsStr>,
    section_module_records: SmallVec<[Result<Arc<ModuleRecord>, Vec<OxcDiagnostic>>; 1]>,
    content: ModuleContent<'alloc_pool>,
}
impl<'alloc_pool> ModuleToLint<'alloc_pool> {
    fn from_processed_module(
        path: Arc<OsStr>,
        processed_module: ProcessedModule<'alloc_pool>,
    ) -> Option<Self> {
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

/// A simple trait for the `Runtime` to load and save file from a filesystem
/// The `Runtime` uses OsFileSystem as a default
/// The Tester and `oxc_language_server` would like to provide the content from memory
pub trait RuntimeFileSystem {
    /// reads the content of a file path
    ///
    /// # Errors
    /// When no valid path is provided or the content is not valid UTF-8 Stream
    fn read_to_arena_str<'a>(
        &'a self,
        path: &Path,
        allocator: &'a Allocator,
    ) -> Result<&'a str, std::io::Error>;

    /// write a file to the file system
    ///
    /// # Errors
    /// When the program does not have write permission for the file system
    fn write_file(&self, path: &Path, content: &str) -> Result<(), std::io::Error>;
}

pub struct OsFileSystem;

impl RuntimeFileSystem for OsFileSystem {
    fn read_to_arena_str<'a>(
        &self,
        path: &Path,
        allocator: &'a Allocator,
    ) -> Result<&'a str, std::io::Error> {
        read_to_arena_str(path, allocator)
    }

    fn write_file(&self, path: &Path, content: &str) -> Result<(), std::io::Error> {
        fs::write(path, content)
    }
}

impl Runtime {
    pub(super) fn new(linter: Linter, options: LintServiceOptions) -> Self {
        // If global thread pool wasn't already initialized, do it now.
        // This "locks" config for the thread pool, which ensures `rayon::current_num_threads()`
        // cannot change from now on.
        //
        // Initializing the thread pool without specifying `num_threads` produces a threadpool size
        // based on `std::thread::available_parallelism`. However, Rayon's docs state that:
        // > In the future, the default behavior may change to dynamically add or remove threads as needed.
        // https://docs.rs/rayon/1.11.0/rayon/struct.ThreadPoolBuilder.html#method.num_threads
        //
        // However, I (@overlookmotel) assume that would be considered a breaking change,
        // so we don't have to worry about it until Rayon v2.
        // When Rayon v2 is released and we upgrade to it, we'll need to revisit this and make sure
        // we still guarantee that thread count is locked.
        //
        // If thread pool was already initialized, this won't do anything.
        // `build_global` will return `Err` in that case, but we can ignore it.
        // That just means the config (and so number of threads) is already locked.
        // https://docs.rs/rayon/1.11.0/rayon/struct.ThreadPoolBuilder.html#method.build_global
        let _ = rayon::ThreadPoolBuilder::new().build_global();

        let thread_count = rayon::current_num_threads();

        // If an external linter is used (JS plugins), we must use fixed-size allocators,
        // for compatibility with raw transfer
        let allocator_pool = if linter.has_external_linter() {
            AllocatorPool::new_fixed_size(thread_count)
        } else {
            AllocatorPool::new(thread_count)
        };

        let resolver = options.cross_module.then(|| {
            Self::get_resolver(options.tsconfig.or_else(|| Some(options.cwd.join("tsconfig.json"))))
        });

        Self {
            allocator_pool,
            cwd: options.cwd,
            linter,
            resolver,
            modules_by_path: papaya::HashMap::builder()
                .hasher(BuildHasherDefault::default())
                .resize_mode(papaya::ResizeMode::Blocking)
                .build(),
            disable_directives_map: Arc::new(Mutex::new(FxHashMap::default())),
        }
    }

    pub fn set_disable_directives_map(
        &mut self,
        map: Arc<Mutex<FxHashMap<PathBuf, DisableDirectives>>>,
    ) {
        self.disable_directives_map = map;
    }

    fn get_resolver(tsconfig_path: Option<PathBuf>) -> Resolver {
        use oxc_resolver::{
            ResolveOptions, TsconfigDiscovery, TsconfigOptions, TsconfigReferences,
        };
        let tsconfig = tsconfig_path.and_then(|path| {
            path.is_file().then_some(TsconfigDiscovery::Manual(TsconfigOptions {
                config_file: path,
                references: TsconfigReferences::Auto,
            }))
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

    fn get_source_type_and_text<'a>(
        file_system: &'a (dyn RuntimeFileSystem + Sync + Send),
        path: &Path,
        ext: &str,
        allocator: &'a Allocator,
    ) -> Option<Result<(SourceType, &'a str), Error>> {
        let source_type = SourceType::from_path(path);
        let not_supported_yet =
            source_type.as_ref().is_err_and(|_| !LINT_PARTIAL_LOADER_EXTENSIONS.contains(&ext));
        if not_supported_yet {
            return None;
        }

        let mut source_type = source_type.unwrap_or_default();
        // Treat JS and JSX files to maximize chance of parsing files.
        if source_type.is_javascript() {
            source_type = source_type.with_jsx(true);
        }

        let file_result = file_system.read_to_arena_str(path, allocator).map_err(|e| {
            Error::new(OxcDiagnostic::error(format!(
                "Failed to open file {} with error \"{e}\"",
                path.display()
            )))
        });
        Some(match file_result {
            Ok(source_text) => Ok((source_type, source_text)),
            Err(e) => Err(e),
        })
    }

    /// Prepare entry modules for linting.
    ///
    /// `on_module_to_lint` is called for each entry modules in `paths` when it's ready for linting,
    /// which means all its dependencies are resolved if import plugin is enabled.
    fn resolve_modules<'a>(
        &'a self,
        file_system: &'a (dyn RuntimeFileSystem + Sync + Send),
        paths: &'a IndexSet<Arc<OsStr>, FxBuildHasher>,
        scope: &Scope<'a>,
        check_syntax_errors: bool,
        tx_error: Option<&'a DiagnosticSender>,
        on_module_to_lint: impl Fn(&'a Self, ModuleToLint) + Send + Sync + Clone + 'a,
    ) {
        if self.resolver.is_none() {
            paths.par_iter().for_each(|path| {
                let output =
                    self.process_path(file_system, paths, path, check_syntax_errors, tx_error);
                let Some(entry) =
                    ModuleToLint::from_processed_module(output.path, output.processed_module)
                else {
                    return;
                };
                on_module_to_lint(self, entry);
            });
            return;
        }
        // The goal of code below is to construct the module graph bootstrapped by the entry modules (`paths`),
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
        // All paths above are in `paths`. `src/index.js`, the entrypoint of the application, references
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

        // Create a sorted copy of paths for processing
        let mut sorted_paths: Vec<_> = paths.iter().cloned().collect();
        sorted_paths.par_sort_unstable_by(|a, b| Path::new(b).cmp(Path::new(a)));

        // The general idea is processing `sorted_paths` and their dependencies in groups. We start from a group of modules
        // in `sorted_paths` that is small enough to hold in memory but big enough to make use of the rayon thread pool.
        // We build the module graph from one group, run lint on them, drop sources and semantics but keep the module
        // graph, and then move on to the next group.
        // This size is empirical based on AFFiNE@97cc814a.
        let group_size = rayon::current_num_threads() * 4;

        // Stores modules that belongs to `self.paths` in current group.
        // They are passed to `on_module_to_lint` at the end of each group.
        let mut modules_to_lint: Vec<ModuleToLint> = Vec::with_capacity(group_size);

        // Set self to immutable reference so it can be shared among spawned tasks.
        let me: &Self = self;

        // `encountered_paths` prevents duplicated processing.
        // It is a superset of keys of `modules_by_path` as it also contains paths that are queued to process.
        let mut encountered_paths =
            FxHashSet::<Arc<OsStr>>::with_capacity_and_hasher(sorted_paths.len(), FxBuildHasher);

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

        // The cursor of `sorted_paths` that points to the start path of the next group.
        let mut group_start = 0usize;

        // The group loop. Each iteration of this loop processes a group of modules.
        while group_start < sorted_paths.len() {
            // How many modules are queued but not processed in this group.
            let mut pending_module_count = 0;

            // Bootstrap the group by processing modules to be linted.
            while pending_module_count < group_size && group_start < sorted_paths.len() {
                let path = &sorted_paths[group_start];
                group_start += 1;

                // Check if this module to be linted is already processed as a dependency in former groups
                if encountered_paths.insert(Arc::clone(path)) {
                    pending_module_count += 1;
                    let path = Arc::clone(path);
                    let tx_process_output = tx_process_output.clone();
                    scope.spawn(move |_| {
                        tx_process_output
                            .send(me.process_path(
                                file_system,
                                paths,
                                &path,
                                check_syntax_errors,
                                tx_error,
                            ))
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
                                let tx_process_output = tx_process_output.clone();
                                let dep_path = Arc::clone(dep_path);
                                move |_| {
                                    tx_process_output
                                        .send(me.process_path(
                                            file_system,
                                            paths,
                                            &dep_path,
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
                self.modules_by_path.pin().insert(
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
                let modules_by_path = self.modules_by_path.pin();
                let records = modules_by_path.get(&path).unwrap();
                assert_eq!(
                    records.len(), requested_module_paths.len(),
                    "This is an internal logic error. Please file an issue at https://github.com/oxc-project/oxc/issues",
                );
                for (record, requested_module_paths) in
                    records.iter().zip(requested_module_paths.into_iter())
                {
                    let mut loaded_modules = record.write_loaded_modules();
                    for request in requested_module_paths {
                        // TODO: revise how to store multiple sections in loaded_modules
                        let Some(dep_module_record) =
                            modules_by_path.get(&request.resolved_requested_path).unwrap().last()
                        else {
                            continue;
                        };
                        loaded_modules.insert(request.specifier, Arc::downgrade(dep_module_record));
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

    pub(super) fn run(
        &self,
        file_system: &(dyn RuntimeFileSystem + Sync + Send),
        paths: Vec<Arc<OsStr>>,
        tx_error: &DiagnosticSender,
    ) {
        self.modules_by_path.pin().reserve(paths.len());
        let paths_set: IndexSet<Arc<OsStr>, FxBuildHasher> = paths.into_iter().collect();

        rayon::scope(|scope| {
            self.resolve_modules(
                file_system,
                &paths_set,
                scope,
                true,
                Some(tx_error),
                move |me, mut module_to_lint| {
                    module_to_lint.content.with_dependent_mut(|allocator_guard, dep| {
                        // If there are fixes, we will accumulate all of them and write to the file at the end.
                        // This means we do not write multiple times to the same file if there are multiple sources
                        // in the same file (for example, multiple scripts in an `.astro` file).
                        let mut new_source_text = Cow::from(dep.source_text);

                        let path = Path::new(&module_to_lint.path);

                        assert_eq!(
                            module_to_lint.section_module_records.len(),
                            dep.section_contents.len()
                        );

                        let context_sub_hosts: Vec<ContextSubHost<'_>> = module_to_lint
                            .section_module_records
                            .into_iter()
                            .zip(dep.section_contents.drain(..))
                            .filter_map(|(record_result, section)| match record_result {
                                Ok(module_record) => {
                                    Some(ContextSubHost::new_with_framework_options(
                                        section.semantic.unwrap(),
                                        Arc::clone(&module_record),
                                        section.source.start,
                                        section.source.framework_options,
                                    ))
                                }
                                Err(messages) => {
                                    if !messages.is_empty() {
                                        let diagnostics = DiagnosticService::wrap_diagnostics(
                                            &me.cwd,
                                            path,
                                            dep.source_text,
                                            messages,
                                        );
                                        tx_error.send(diagnostics).unwrap();
                                    }
                                    None
                                }
                            })
                            .collect();

                        if context_sub_hosts.is_empty() {
                            return;
                        }

                        let (mut messages, disable_directives) = me
                            .linter
                            .run_with_disable_directives(path, context_sub_hosts, allocator_guard);

                        // Store the disable directives for this file
                        if let Some(disable_directives) = disable_directives {
                            me.disable_directives_map
                                .lock()
                                .expect("disable_directives_map mutex poisoned")
                                .insert(path.to_path_buf(), disable_directives);
                        }

                        if me.linter.options().fix.is_some() {
                            let fix_result = Fixer::new(
                                dep.source_text,
                                messages,
                                SourceType::from_path(path).ok().map(|st| {
                                    if st.is_javascript() { st.with_jsx(true) } else { st }
                                }),
                            )
                            .fix();
                            if fix_result.fixed {
                                // write to file, replacing only the changed part
                                let start = 0;
                                let end = start + dep.source_text.len();
                                new_source_text
                                    .to_mut()
                                    .replace_range(start..end, &fix_result.fixed_code);
                            }
                            messages = fix_result.messages;
                        }

                        if !messages.is_empty() {
                            let errors = messages.into_iter().map(Into::into).collect();
                            let diagnostics = DiagnosticService::wrap_diagnostics(
                                &me.cwd,
                                path,
                                dep.source_text,
                                errors,
                            );
                            tx_error.send(diagnostics).unwrap();
                        }

                        // If the new source text is owned, that means it was modified,
                        // so we write the new source text to the file.
                        if let Cow::Owned(new_source_text) = &new_source_text {
                            file_system.write_file(path, new_source_text).unwrap();
                        }
                    });
                },
            );
        });
    }

    // language_server: the language server needs line and character position
    // the struct not using `oxc_diagnostic::Error, because we are just collecting information
    // and returning it to the client to let him display it.
    pub(super) fn run_source(
        &self,
        file_system: &(dyn RuntimeFileSystem + Sync + Send),
        paths: Vec<Arc<OsStr>>,
    ) -> Vec<Message> {
        use std::sync::Mutex;

        self.modules_by_path.pin().reserve(paths.len());
        let paths_set: IndexSet<Arc<OsStr>, FxBuildHasher> = paths.into_iter().collect();

        let messages = Mutex::new(Vec::<Message>::new());
        rayon::scope(|scope| {
            self.resolve_modules(
                file_system,
                &paths_set,
                scope,
                true,
                None,
                |me, mut module_to_lint| {
                    module_to_lint.content.with_dependent_mut(
                    |allocator_guard, ModuleContentDependent { source_text: _, section_contents }| {
                        assert_eq!(
                            module_to_lint.section_module_records.len(),
                            section_contents.len()
                        );

                        let context_sub_hosts: Vec<ContextSubHost<'_>> = module_to_lint
                            .section_module_records
                            .into_iter()
                            .zip(section_contents.drain(..))
                            .filter_map(|(record_result, section)| match record_result {
                                Ok(module_record) => {
                                    Some(ContextSubHost::new_with_framework_options(
                                        section.semantic.unwrap(),
                                        Arc::clone(&module_record),
                                        section.source.start,
                                        section.source.framework_options,
                                    ))
                                }
                                Err(diagnostics) => {
                                    if !diagnostics.is_empty() {
                                        messages.lock().unwrap().extend(
                                            diagnostics.into_iter().map(|diagnostic| {
                                                Message::new(diagnostic, PossibleFixes::None)
                                            }),
                                        );
                                    }
                                    None
                                }
                            })
                            .collect();

                        if context_sub_hosts.is_empty() {
                            return;
                        }

                        let path = Path::new(&module_to_lint.path);
                        let (section_messages, disable_directives) = me
                            .linter
                            .run_with_disable_directives(path, context_sub_hosts, allocator_guard);

                        if let Some(disable_directives) = disable_directives {
                            me.disable_directives_map
                                .lock()
                                .expect("disable_directives_map mutex poisoned")
                                .insert(path.to_path_buf(), disable_directives);
                        }

                        messages.lock().unwrap().extend(
                            section_messages
                        );
                    },
                );
                },
            );
        });

        messages.into_inner().unwrap()
    }

    #[cfg(test)]
    pub(super) fn run_test_source(
        &self,
        file_system: &(dyn RuntimeFileSystem + Sync + Send),
        paths: Vec<Arc<OsStr>>,
        check_syntax_errors: bool,
        tx_error: &DiagnosticSender,
    ) -> Vec<Message> {
        use std::sync::Mutex;

        self.modules_by_path.pin().reserve(paths.len());
        let paths_set: IndexSet<Arc<OsStr>, FxBuildHasher> = paths.into_iter().collect();

        let messages = Mutex::new(Vec::<Message>::new());
        rayon::scope(|scope| {
            self.resolve_modules(file_system, &paths_set, scope, check_syntax_errors, Some(tx_error), |me, mut module| {
                module.content.with_dependent_mut(
                    |allocator_guard, ModuleContentDependent { source_text: _, section_contents }| {
                        assert_eq!(module.section_module_records.len(), section_contents.len());

                        let context_sub_hosts: Vec<ContextSubHost<'_>> = module
                            .section_module_records
                            .into_iter()
                            .zip(section_contents.drain(..))
                            .filter_map(|(record_result, section)| match record_result {
                                Ok(module_record) => Some(ContextSubHost::new_with_framework_options(
                                    section.semantic.unwrap(),
                                    Arc::clone(&module_record),
                                    section.source.start,
                                    section.source.framework_options
                                )),
                                Err(errors) => {
                                    if !errors.is_empty() {
                                        messages
                                            .lock()
                                            .unwrap()
                                            .extend(errors
                                        .into_iter()
                                        .map(|err| Message::new(err, PossibleFixes::None))
                                    );
                                    }
                                    None
                                }
                            })
                            .collect();

                        if context_sub_hosts.is_empty() {
                            return;
                        }

                        messages.lock().unwrap().extend(
                            me.linter.run(
                                Path::new(&module.path),
                                context_sub_hosts,
                                allocator_guard
                            )
                            ,
                        );
                    },
                );
            });
        });
        messages.into_inner().unwrap()
    }

    fn process_path<'a>(
        &'a self,
        file_system: &'a (dyn RuntimeFileSystem + Sync + Send),
        paths: &IndexSet<Arc<OsStr>, FxBuildHasher>,
        path: &Arc<OsStr>,
        check_syntax_errors: bool,
        tx_error: Option<&DiagnosticSender>,
    ) -> ModuleProcessOutput<'a> {
        let processed_module = self
            .process_path_to_module(file_system, paths, path, check_syntax_errors, tx_error)
            .unwrap_or_default();
        ModuleProcessOutput { path: Arc::clone(path), processed_module }
    }

    fn process_path_to_module<'a>(
        &'a self,
        file_system: &'a (dyn RuntimeFileSystem + Sync + Send),
        paths: &IndexSet<Arc<OsStr>, FxBuildHasher>,
        path: &Arc<OsStr>,
        check_syntax_errors: bool,
        tx_error: Option<&DiagnosticSender>,
    ) -> Option<ProcessedModule<'a>> {
        let ext = Path::new(path).extension().and_then(OsStr::to_str)?;

        if SourceType::from_path(Path::new(path))
            .as_ref()
            .is_err_and(|_| !LINT_PARTIAL_LOADER_EXTENSIONS.contains(&ext))
        {
            return None;
        }

        let allocator_guard = self.allocator_pool.get();

        if paths.contains(path) {
            let mut records =
                SmallVec::<[Result<ResolvedModuleRecord, Vec<OxcDiagnostic>>; 1]>::new();

            let module_content = ModuleContent::try_new(allocator_guard, |allocator_guard| {
                let allocator = &**allocator_guard;

                let Some(stt) =
                    Self::get_source_type_and_text(file_system, Path::new(path), ext, allocator)
                else {
                    return Err(());
                };

                let (source_type, source_text) = match stt {
                    Ok(v) => v,
                    Err(e) => {
                        if let Some(tx_error) = tx_error {
                            tx_error.send(vec![e]).unwrap();
                        }
                        return Err(());
                    }
                };

                let mut section_contents = SmallVec::new();
                records = self.process_source(
                    Path::new(path),
                    ext,
                    check_syntax_errors,
                    source_type,
                    source_text,
                    allocator,
                    Some(&mut section_contents),
                );

                Ok(ModuleContentDependent { source_text, section_contents })
            });
            let module_content = module_content.ok()?;

            Some(ProcessedModule { section_module_records: records, content: Some(module_content) })
        } else {
            let allocator = &*allocator_guard;

            let stt = Self::get_source_type_and_text(file_system, Path::new(path), ext, allocator)?;

            let (source_type, source_text) = match stt {
                Ok(v) => v,
                Err(e) => {
                    if let Some(tx_error) = tx_error {
                        tx_error.send(vec![e]).unwrap();
                    }
                    return None;
                }
            };

            let records = self.process_source(
                Path::new(path),
                ext,
                check_syntax_errors,
                source_type,
                source_text,
                allocator,
                None,
            );

            Some(ProcessedModule { section_module_records: records, content: None })
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
                    let err: Vec<OxcDiagnostic> = err
                        .into_iter()
                        .map(|mut diagnostic| {
                            if let Some(labels) = &mut diagnostic.labels {
                                for label in labels.iter_mut() {
                                    label.set_span_offset(
                                        label.offset() + section_source.start as usize,
                                    );
                                }
                            }
                            diagnostic
                        })
                        .collect();

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
        }

        let semantic_ret = SemanticBuilder::new()
            .with_cfg(true)
            .with_scope_tree_child_ids(true)
            .with_check_syntax_error(check_syntax_errors)
            .build(allocator.alloc(ret.program));

        if !semantic_ret.errors.is_empty() {
            return Err(semantic_ret.errors);
        }

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
