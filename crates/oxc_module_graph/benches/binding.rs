use std::path::PathBuf;

use compact_str::CompactString;
use criterion::{Criterion, criterion_group, criterion_main};
use oxc_module_graph::types::{
    ExportsKind, ImportKind, ImportRecordIdx, ImportRecordMeta, LocalExport, ModuleIdx,
    NamedImport, ResolvedImportRecord, StarExportEntry, SymbolRef, WrapKind,
};
use oxc_module_graph::{
    LinkConfig, ModuleGraph, NormalModule, SideEffects, build_resolved_exports,
    match_imports_collect,
};
use oxc_syntax::symbol::SymbolId;
use rustc_hash::FxHashMap;

fn sym(module: usize, id: u32) -> SymbolRef {
    SymbolRef::new(ModuleIdx::from_usize(module), SymbolId::from_raw_unchecked(id))
}

/// Build a graph with `module_count` modules, each with `exports_per_module` exports
/// and star-export chains between consecutive modules.
fn build_star_export_graph(module_count: usize, exports_per_module: usize) -> ModuleGraph {
    let mut graph = ModuleGraph::new();

    for m in 0..module_count {
        let idx = ModuleIdx::from_usize(m);
        let default_ref = sym(m, exports_per_module as u32);
        let ns_ref = sym(m, exports_per_module as u32 + 1);

        let mut named_exports = FxHashMap::default();
        for e in 0..exports_per_module {
            let name = CompactString::new(format!("export_{e}"));
            named_exports.insert(
                name.clone(),
                LocalExport { exported_name: name, local_symbol: sym(m, e as u32) },
            );
        }

        // Star export to next module (except last)
        let star_export_entries = if m + 1 < module_count {
            vec![StarExportEntry {
                module_request: CompactString::new(format!("./mod_{}", m + 1)),
                resolved_module: Some(ModuleIdx::from_usize(m + 1)),
                span: oxc_span::Span::default(),
            }]
        } else {
            vec![]
        };

        let module = NormalModule {
            idx,
            path: PathBuf::from(format!("mod_{m}.js")),
            has_module_syntax: true,
            exports_kind: ExportsKind::Esm,
            has_top_level_await: false,
            side_effects: SideEffects::True,
            has_lazy_export: false,
            execution_order_sensitive: false,
            named_exports,
            named_imports: FxHashMap::default(),
            import_records: Vec::new(),
            star_export_entries,
            indirect_export_entries: Vec::new(),
            default_export_ref: default_ref,
            namespace_object_ref: ns_ref,
            wrap_kind: WrapKind::None,
            original_wrap_kind: WrapKind::None,
            wrapper_ref: None,
            required_by_other_module: false,
            resolved_exports: FxHashMap::default(),
            has_dynamic_exports: false,
            is_tla_or_contains_tla: false,
            propagated_side_effects: false,
            exec_order: u32::MAX,
        };

        // Ensure symbol slots
        graph.add_normal_module(module);
        let total_syms = exports_per_module + 2; // exports + default + ns
        graph.ensure_module_symbol_capacity(idx, total_syms);
        for e in 0..exports_per_module {
            graph.set_symbol_name(idx, SymbolId::from_usize(e), format!("export_{e}"));
        }
    }

    graph.set_entries(vec![ModuleIdx::from_usize(0)]);
    graph
}

/// Build a graph with modules that import from each other for match_imports bench.
fn build_import_graph(module_count: usize, imports_per_module: usize) -> ModuleGraph {
    let mut graph = ModuleGraph::new();

    // First create all modules with exports
    for m in 0..module_count {
        let idx = ModuleIdx::from_usize(m);
        let default_ref = sym(m, (imports_per_module + imports_per_module) as u32);
        let ns_ref = sym(m, (imports_per_module + imports_per_module + 1) as u32);

        let mut named_exports = FxHashMap::default();
        for e in 0..imports_per_module {
            let name = CompactString::new(format!("item_{e}"));
            named_exports.insert(
                name.clone(),
                LocalExport {
                    exported_name: name,
                    local_symbol: sym(m, (imports_per_module + e) as u32),
                },
            );
        }

        // Import from next module (wrapping around)
        let target = (m + 1) % module_count;
        let mut named_imports = FxHashMap::default();
        let mut import_records = Vec::new();

        import_records.push(ResolvedImportRecord {
            specifier: CompactString::new(format!("./mod_{target}")),
            resolved_module: Some(ModuleIdx::from_usize(target)),
            kind: ImportKind::Static,
            namespace_ref: ns_ref,
            meta: ImportRecordMeta::empty(),
            is_type_only: false,
        });

        for i in 0..imports_per_module {
            let local_sym = sym(m, i as u32);
            named_imports.insert(
                local_sym,
                NamedImport {
                    imported_name: CompactString::new(format!("item_{i}")),
                    local_symbol: local_sym,
                    record_idx: ImportRecordIdx::from_usize(0),
                    is_type: false,
                },
            );
        }

        let module = NormalModule {
            idx,
            path: PathBuf::from(format!("mod_{m}.js")),
            has_module_syntax: true,
            exports_kind: ExportsKind::Esm,
            has_top_level_await: false,
            side_effects: SideEffects::True,
            has_lazy_export: false,
            execution_order_sensitive: false,
            named_exports,
            named_imports,
            import_records,
            star_export_entries: Vec::new(),
            indirect_export_entries: Vec::new(),
            default_export_ref: default_ref,
            namespace_object_ref: ns_ref,
            wrap_kind: WrapKind::None,
            original_wrap_kind: WrapKind::None,
            wrapper_ref: None,
            required_by_other_module: false,
            resolved_exports: FxHashMap::default(),
            has_dynamic_exports: false,
            is_tla_or_contains_tla: false,
            propagated_side_effects: false,
            exec_order: u32::MAX,
        };

        graph.add_normal_module(module);
        let total_syms = imports_per_module * 2 + 2;
        graph.ensure_module_symbol_capacity(idx, total_syms);
        for s in 0..total_syms {
            graph.set_symbol_name(idx, SymbolId::from_usize(s), format!("sym_{m}_{s}"));
        }
    }

    graph.set_entries(vec![ModuleIdx::from_usize(0)]);
    graph
}

fn bench_build_resolved_exports(c: &mut Criterion) {
    c.bench_function("build_resolved_exports_1000_modules", |b| {
        let graph = build_star_export_graph(1000, 10);
        b.iter(|| build_resolved_exports(&graph));
    });
}

fn bench_match_imports(c: &mut Criterion) {
    c.bench_function("match_imports_1000_modules_10_imports", |b| {
        b.iter_with_setup(
            || {
                let mut graph = build_import_graph(1000, 10);
                // First build resolved exports and store them
                let resolved = build_resolved_exports(&graph);
                for (idx, exports) in resolved {
                    if let Some(m) = graph.normal_module_mut(idx) {
                        m.resolved_exports = exports;
                    }
                }
                graph
            },
            |graph| {
                let mut config = LinkConfig::default();
                match_imports_collect(&graph, &mut config)
            },
        );
    });
}

criterion_group!(benches, bench_build_resolved_exports, bench_match_imports,);
criterion_main!(benches);
