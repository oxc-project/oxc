use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    rc::Rc,
    sync::Arc,
};

use oxc_allocator::Allocator;
use oxc_checker::{
    checker::{Checker, CheckerBuilder, CheckerReturn, NodeRef},
    program::{
        FsProgramHost, HostModuleResolution, ProgramEntry, ProgramHost, ProgramStoreBuilder,
        ProgramStoreError,
    },
    types::{CheckerArena, Ty, TypeData, TypeId},
};
use oxc_diagnostics::{DiagnosticSender, DiagnosticService};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::{SourceType, Span};
use oxc_span_checker::GetSpan;
use rustc_hash::FxHashMap;

use crate::{
    AllowWarnDeny, ConfigStore, ContextHost, ContextSubHost, ContextSubHostOptions, LintOptions,
    Message, ModuleRecord, RuleEnum, RuntimeFileSystem,
};

pub fn is_native_type_aware_rule(rule: &RuleEnum) -> bool {
    // TODO: This would actually be part of the generated code, but this is just a proof of concept
    rule.name() == "no-unsafe-unary-minus"
}

pub fn is_type_aware_rule(rule: &RuleEnum) -> bool {
    rule.is_tsgolint_rule() || is_native_type_aware_rule(rule)
}

pub struct TypedApiContext<'a> {
    arena: CheckerArena<'a>,
    types_by_span: FxHashMap<Span, Ty<'a>>,
    type_names: FxHashMap<(Span, TypeId), String>,
}

impl<'a> TypedApiContext<'a> {
    fn new(checker: &CheckerReturn<'a, '_>, entry: &ProgramEntry<'a>) -> Self {
        let arena = checker.arena();
        let mut types_by_span = FxHashMap::default();
        let mut type_names = FxHashMap::default();
        for (node_id, node) in entry.semantic().nodes().iter_enumerated() {
            let span = node.kind().span();
            let node = NodeRef::new(entry.id(), node_id);
            let ty = checker.get_constrained_type_at_location(node);
            let span = Span::new(span.start, span.end);
            types_by_span.insert(span, ty);
            cache_type_names(checker, arena, ty, node, span, &mut type_names);
        }
        Self { arena, types_by_span, type_names }
    }

    #[inline]
    pub fn type_at_span(&self, span: Span) -> Option<Ty<'a>> {
        self.types_by_span.get(&span).copied()
    }

    #[inline]
    pub fn type_data(&self, ty: Ty<'a>) -> TypeData<'a> {
        self.arena.type_data(ty)
    }

    #[inline]
    pub fn type_name(&self, span: Span, ty: Ty<'a>) -> Option<&str> {
        self.type_names.get(&(span, ty.id())).map(String::as_str)
    }
}

fn cache_type_names<'a>(
    checker: &CheckerReturn<'a, '_>,
    arena: CheckerArena<'a>,
    ty: Ty<'a>,
    node: NodeRef,
    span: Span,
    type_names: &mut FxHashMap<(Span, TypeId), String>,
) {
    if type_names.insert((span, ty.id()), checker.type_to_string(ty, node)).is_some() {
        return;
    }
    match arena.type_data(ty) {
        TypeData::Union(union) => {
            for ty in &union.types {
                cache_type_names(checker, arena, *ty, node, span, type_names);
            }
        }
        TypeData::Intersection(intersection) => {
            for ty in &intersection.types {
                cache_type_names(checker, arena, *ty, node, span, type_names);
            }
        }
        _ => {}
    }
}

pub struct NativeTypeAwareRunner {
    cwd: PathBuf,
    config_store: ConfigStore,
}

struct RuntimeProgramHost<'a> {
    file_system: &'a (dyn RuntimeFileSystem + Sync + Send),
    resolver: FsProgramHost,
}

impl<'a> RuntimeProgramHost<'a> {
    fn new(file_system: &'a (dyn RuntimeFileSystem + Sync + Send)) -> Self {
        Self { file_system, resolver: FsProgramHost::new() }
    }
}

impl ProgramHost for RuntimeProgramHost<'_> {
    fn read_source(&self, path: &Path) -> Result<String, ProgramStoreError> {
        let allocator = Allocator::default();
        self.file_system.read_to_arena_str(path, &allocator).map(str::to_owned).map_err(|error| {
            ProgramStoreError::ReadSource { path: path.to_path_buf(), message: error.to_string() }
        })
    }

    fn canonicalize_path(&self, path: &Path) -> PathBuf {
        self.resolver.canonicalize_path(path)
    }

    fn resolve_module(&self, containing_file: &Path, specifier: &str) -> HostModuleResolution {
        self.resolver.resolve_module(containing_file, specifier)
    }
}

impl NativeTypeAwareRunner {
    pub fn new(cwd: PathBuf, config_store: ConfigStore) -> Self {
        Self { cwd, config_store }
    }

    pub fn lint(
        &self,
        files: &[Arc<OsStr>],
        _directives_store: &crate::DirectivesStore,
        tx_error: &DiagnosticSender,
    ) -> Result<(), String> {
        for result in self.lint_with_host(files, FsProgramHost::new())? {
            if result.messages.is_empty() {
                continue;
            }
            let diagnostics = result.messages.into_iter().map(Into::into).collect::<Vec<_>>();
            let wrapped = DiagnosticService::wrap_diagnostics(
                &self.cwd,
                result.path,
                &result.source_text,
                diagnostics,
            );
            tx_error.send(wrapped).map_err(|error| error.to_string())?;
        }
        Ok(())
    }

    pub fn lint_source(
        &self,
        files: &[Arc<OsStr>],
        file_system: &(dyn RuntimeFileSystem + Sync + Send),
    ) -> Result<Vec<Message>, String> {
        Ok(self
            .lint_with_host(files, RuntimeProgramHost::new(file_system))?
            .into_iter()
            .flat_map(|result| result.messages)
            .collect())
    }

    fn lint_with_host<H: ProgramHost>(
        &self,
        files: &[Arc<OsStr>],
        host: H,
    ) -> Result<Vec<NativeLintResult>, String> {
        let selected_files = files
            .iter()
            .map(PathBuf::from)
            .filter(|path| SourceType::from_path(path).is_ok_and(SourceType::is_typescript))
            .collect::<Vec<_>>();

        if !selected_files.iter().any(|path| !self.rules(path).is_empty()) {
            return Ok(Vec::new());
        }

        let allocator = Allocator::default();
        let mut builder = ProgramStoreBuilder::new(&allocator, host);
        for path in &selected_files {
            builder = builder.add_root_file(path);
        }
        let store = builder.build().map_err(|error| error.to_string())?;
        let checker = CheckerBuilder::new().build(&store);

        let mut results = Vec::new();
        for entry in store.entries().iter().filter(|entry| !entry.is_lib()) {
            let Some(selected_path) =
                selected_files.iter().find(|path| same_file(path, entry.path()))
            else {
                continue;
            };
            let resolved = self.config_store.resolve(selected_path);
            let rules = resolved
                .rules
                .iter()
                .filter(|(rule, severity)| {
                    is_native_type_aware_rule(rule) && severity.is_warn_deny()
                })
                .cloned()
                .collect::<Vec<_>>();
            if rules.is_empty() {
                continue;
            }

            let source_type = SourceType::from_path(selected_path).map_err(|e| e.to_string())?;
            let parsed = Parser::new(&allocator, entry.source_text(), source_type).parse();
            if !parsed.diagnostics.is_empty() {
                continue;
            }
            let semantic =
                SemanticBuilder::new_linter().build(allocator.alloc(parsed.program)).semantic;
            let module_record =
                Arc::new(ModuleRecord::new(selected_path, &parsed.module_record, &semantic));
            let sub_host =
                ContextSubHost::new(semantic, module_record, 0, ContextSubHostOptions::default());
            let typed_api = TypedApiContext::new(&checker, entry);
            let ctx_host = Rc::new(
                ContextHost::new(
                    selected_path,
                    vec![sub_host],
                    &allocator,
                    LintOptions::default(),
                    Arc::clone(&resolved.config),
                )
                .with_type_aware(typed_api),
            );

            for (rule, severity) in &rules {
                let ctx = Rc::clone(&ctx_host).spawn(rule, *severity);
                for node in ctx.semantic().nodes().iter() {
                    rule.run::<false>(node, &ctx, None);
                }
            }

            results.push(NativeLintResult {
                path: selected_path.clone(),
                source_text: entry.source_text().to_string(),
                messages: ctx_host.take_diagnostics(),
            });
        }

        Ok(results)
    }

    fn rules(&self, path: &Path) -> Vec<(RuleEnum, AllowWarnDeny)> {
        self.config_store
            .resolve(path)
            .rules
            .iter()
            .filter(|(rule, severity)| is_native_type_aware_rule(rule) && severity.is_warn_deny())
            .cloned()
            .collect()
    }
}

struct NativeLintResult {
    path: PathBuf,
    source_text: String,
    messages: Vec<Message>,
}

fn same_file(left: &Path, right: &Path) -> bool {
    std::fs::canonicalize(left).unwrap_or_else(|_| left.to_path_buf()) == right
}
