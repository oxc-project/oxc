use super::LintServiceOptions;
use crate::fixer::{Fixer, Message};
use crate::{
    Linter,
    loader::{JavaScriptSource, LINT_PARTIAL_LOADER_EXT, PartialLoader},
    module_record::ModuleRecord,
    utils::read_to_string,
};
use indexmap::IndexSet;
use oxc_allocator::Allocator;
use oxc_diagnostics::{DiagnosticSender, DiagnosticService, Error, OxcDiagnostic};
use oxc_parser::{ParseOptions, Parser};
use oxc_resolver::Resolver;
use oxc_semantic::{Semantic, SemanticBuilder};
use oxc_span::{CompactStr, SourceType, VALID_EXTENSIONS};
use rayon::iter::ParallelDrainRange;
use rayon::{Scope, iter::IntoParallelRefIterator, prelude::ParallelIterator};
use rustc_hash::{FxBuildHasher, FxHashMap, FxHashSet};
use self_cell::self_cell;
use smallvec::SmallVec;
use std::borrow::Cow;
use std::mem::take;
use std::rc::Rc;
use std::sync::mpsc;
use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    sync::Arc,
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

struct SectionContent<'a> {
    source: JavaScriptSource<'a>,
    /// None if the parsing failed. The corresponding item with the same index in `ResolvedModule::section_module_records` would contain the errors.
    semantic: Option<Semantic<'a>>,
}

type SectionContents<'a> = SmallVec<[SectionContent<'a>; 1]>;

struct ModuleContentOwner {
    source_text: String,
    allocator: Allocator,
}

self_cell! {
    struct ModuleContent {
        owner: ModuleContentOwner,
        // in the same order as resolvedModule.records
        #[not_covariant]
        dependent: SectionContents,
    }
}

// Safety: dependent borrows from owner. They're safe to be sent together.
unsafe impl Send for ModuleContent {}

struct ResolvedModuleRecord {
    module_record: Arc<ModuleRecord>,
    requested_module_paths: Vec<(/*specifier*/ CompactStr, Arc<OsStr>)>,
}

#[derive(Default)]
struct ResolvedModule {
    section_module_records: SmallVec<[Result<ResolvedModuleRecord, Vec<OxcDiagnostic>>; 1]>,
    content: Option<ModuleContent>,
}

struct ModuleResolveOutput {
    path: Arc<OsStr>,
    resolved_module: ResolvedModule,
}

struct EntryModule {
    path: Arc<OsStr>,
    section_module_records: SmallVec<[Result<Arc<ModuleRecord>, Vec<OxcDiagnostic>>; 1]>,
    content: ModuleContent,
}
impl EntryModule {
    fn from_resolved_module(path: Arc<OsStr>, resolved_module: ResolvedModule) -> Option<Self> {
        let content = resolved_module.content?;
        Some(Self {
            path,
            section_module_records: resolved_module
                .section_module_records
                .into_iter()
                .map(|record_result| match record_result {
                    Ok(record) => Ok(record.module_record),
                    Err(err) => Err(err.clone()),
                })
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

    fn resolve_modules<'a>(
        &'a mut self,
        scope: &Scope<'a>,
        check_syntax_errors: bool,
        tx_error: &'a DiagnosticSender,
        on_entry: impl Fn(&'a Self, EntryModule) + Send + Sync + Clone + 'a,
    ) {
        if self.resolver.is_none() {
            self.paths.par_iter().for_each(|path| {
                let output = self.process_path(Arc::clone(&path), check_syntax_errors, tx_error);
                let entry =
                    EntryModule::from_resolved_module(output.path, output.resolved_module).unwrap();
                on_entry(self, entry);
            });
            return;
        }
        self.paths.par_sort_unstable_by(|a, b| Path::new(b).cmp(&Path::new(a)));
        let me: &Self = self;
        let mut modules_by_path =
            FxHashMap::<Arc<OsStr>, SmallVec<[Arc<ModuleRecord>; 1]>>::default();
        let mut encountered_paths =
            FxHashSet::<Arc<OsStr>>::with_capacity_and_hasher(me.paths.len(), FxBuildHasher);
        let entry_group_size = rayon::current_num_threads();
        let mut entry_modules: Vec<EntryModule> = Vec::with_capacity(entry_group_size);

        let mut module_relationships =
            Vec::<(Arc<OsStr>, SmallVec<[Vec<(/*specifier*/ CompactStr, Arc<OsStr>)>; 1]>)>::new();

        let (tx_resolve_output, rx_resolve_output) = mpsc::channel::<ModuleResolveOutput>();

        let mut group_start = 0usize;
        while group_start < me.paths.len() {
            let mut unresolved_module_count = 0;
            while unresolved_module_count < entry_group_size && group_start < me.paths.len() {
                let path = &me.paths[group_start];
                group_start += 1;
                if encountered_paths.insert(path.clone()) {
                    unresolved_module_count += 1;
                    let path = path.clone();
                    let tx_resolve_output = tx_resolve_output.clone();
                    scope.spawn(move |_| {
                        tx_resolve_output
                            .send(me.process_path(path, check_syntax_errors, tx_error))
                            .unwrap();
                    })
                }
            }
            while unresolved_module_count > 0 {
                let Ok(ModuleResolveOutput { path, mut resolved_module }) =
                    rx_resolve_output.try_recv()
                else {
                    rayon::yield_now();
                    continue;
                };
                unresolved_module_count -= 1;
                let records: SmallVec<[Arc<ModuleRecord>; 1]> = resolved_module
                    .section_module_records
                    .iter()
                    .filter_map(|resolved_module_record| {
                        Some(Arc::clone(&resolved_module_record.as_ref().ok()?.module_record))
                    })
                    .collect();

                modules_by_path.insert(Arc::clone(&path), records);

                for record_result in &resolved_module.section_module_records {
                    let Ok(record) = record_result.as_ref() else {
                        continue;
                    };
                    for (_, dep_path) in &record.requested_module_paths {
                        if encountered_paths.insert(Arc::clone(dep_path)) {
                            scope.spawn({
                                let tx_resolve_output = tx_resolve_output.clone();
                                let dep_path = dep_path.clone();
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
                            unresolved_module_count += 1;
                        }
                    }
                }

                module_relationships.push((
                    Arc::clone(&path),
                    resolved_module
                        .section_module_records
                        .iter_mut()
                        .filter_map(|record_result| {
                            Some(take(&mut record_result.as_mut().ok()?.requested_module_paths))
                        })
                        .collect(),
                ));

                if let Some(entry_module) = EntryModule::from_resolved_module(path, resolved_module)
                {
                    entry_modules.push(entry_module);
                }
            }

            module_relationships.par_drain(..).for_each(|(path, requested_module_paths)| {
                if requested_module_paths.is_empty() {
                    return;
                }
                let records = &modules_by_path[&path];
                assert_eq!(records.len(), requested_module_paths.len());
                for (record, requested_module_paths) in
                    records.iter().zip(requested_module_paths.into_iter())
                {
                    let mut loaded_modules = record.loaded_modules.write().unwrap();
                    for (specifier, dep_path) in requested_module_paths {
                        // TODO: revise how to store multiple sections in loaded_modules
                        let Some(dep_module_record) = modules_by_path[&dep_path].first() else {
                            continue;
                        };
                        loaded_modules.insert(specifier, Arc::clone(dep_module_record));
                    }
                }
            });
            for entry in entry_modules.drain(..) {
                let on_entry = on_entry.clone();
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
            self.resolve_modules(scope, true, tx_error, |me, mut entry_module| {
                entry_module.content.with_dependent_mut(|owner, sections| {
                    // If there are fixes, we will accumulate all of them and write to the file at the end.
                    // This means we do not write multiple times to the same file if there are multiple sources
                    // in the same file (for example, multiple scripts in an `.astro` file).
                    let mut new_source_text = Cow::from(owner.source_text.as_str());
                    // This is used to keep track of the cumulative offset from applying fixes.
                    // Otherwise, spans for fixes will be incorrect due to varying size of the
                    // source code after each fix.
                    let mut fix_offset: i32 = 0;

                    let path = Path::new(&entry_module.path);

                    assert_eq!(entry_module.section_module_records.len(), sections.len());
                    for (record_result, section) in
                        entry_module.section_module_records.into_iter().zip(sections.drain(..))
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

        let mut messages = Mutex::new(Vec::<Message<'a>>::new());
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
    ) -> ModuleResolveOutput {
        let Some(ext) = Path::new(&path).extension().and_then(OsStr::to_str) else {
            return ModuleResolveOutput { path, resolved_module: ResolvedModule::default() };
        };
        let Some(source_type_and_text) = self.get_source_type_and_text(Path::new(&path), ext)
        else {
            return ModuleResolveOutput { path, resolved_module: ResolvedModule::default() };
        };

        let (source_type, source_text) = match source_type_and_text {
            Ok(source_text) => source_text,
            Err(e) => {
                tx_error.send(Some((Path::new(&path).to_path_buf(), vec![e]))).unwrap();
                return ModuleResolveOutput { path, resolved_module: ResolvedModule::default() };
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

        ModuleResolveOutput {
            path,
            resolved_module: ResolvedModule {
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

        let mut requested_module_paths: Vec<(CompactStr, Arc<OsStr>)> = vec![];

        // If import plugin is enabled.
        if let Some(resolver) = &self.resolver {
            // Retrieve all dependent modules from this module.
            let dir = path.parent().unwrap();
            requested_module_paths = module_record
                .requested_modules
                .par_iter()
                .filter_map(|(specifier, _)| {
                    let resolution = resolver.resolve(dir, specifier).ok()?;
                    Some((specifier.clone(), Arc::<OsStr>::from(resolution.path().as_os_str())))
                })
                .collect();
        }
        Ok((ResolvedModuleRecord { module_record, requested_module_paths }, semantic))
    }
}
