use std::path::PathBuf;

use compact_str::CompactString;
use oxc_module_graph::default::SymbolRefDb;
use oxc_module_graph::types::{
    ExportsKind, ImportKind, ImportRecordIdx, ImportRecordMeta, IndirectExportEntry, LocalExport,
    MatchImportKind, ModuleIdx, NamedImport, ResolvedImportRecord, StarExportEntry, SymbolRef,
    WrapKind,
};
use oxc_module_graph::{
    ExportsKindConfig, ExternalModule, ImportHooks, LinkConfig, ModuleGraph, NormalModule,
    SideEffects, WrapModulesConfig, bind_imports_and_exports, build_resolved_exports,
    compute_exec_order, compute_has_dynamic_exports, compute_tla, determine_module_exports_kind,
    determine_safely_merge_cjs_ns, determine_side_effects, find_cycles, match_imports_collect,
    wrap_modules,
};
use rustc_hash::FxHashMap;

fn dummy_symbol_ref(module: ModuleIdx, id: u32) -> SymbolRef {
    use oxc_syntax::symbol::SymbolId;
    SymbolRef::new(module, SymbolId::from_raw_unchecked(id))
}

/// Helper: create a NormalModule with minimal fields.
fn make_normal_module(
    idx: ModuleIdx,
    path: &str,
    named_exports: FxHashMap<CompactString, LocalExport>,
    named_imports: FxHashMap<SymbolRef, NamedImport>,
    import_records: Vec<ResolvedImportRecord>,
    default_export_ref: SymbolRef,
    namespace_object_ref: SymbolRef,
) -> NormalModule {
    NormalModule {
        idx,
        path: PathBuf::from(path),
        has_module_syntax: true,
        exports_kind: ExportsKind::None,
        has_top_level_await: false,
        side_effects: SideEffects::True,
        has_lazy_export: false,
        execution_order_sensitive: false,
        named_exports,
        named_imports,
        import_records,
        star_export_entries: Vec::new(),
        indirect_export_entries: Vec::new(),
        default_export_ref,
        namespace_object_ref,
        wrap_kind: WrapKind::None,
        original_wrap_kind: WrapKind::None,
        wrapper_ref: None,
        required_by_other_module: false,
        resolved_exports: FxHashMap::default(),
        has_dynamic_exports: false,
        is_tla_or_contains_tla: false,
        propagated_side_effects: false,
        exec_order: u32::MAX,
    }
}

#[test]
fn test_module_graph_basic() {
    let mut graph = ModuleGraph::new();

    let idx_a = ModuleIdx::from_usize(0);
    let idx_b = ModuleIdx::from_usize(1);

    // Module A: imports `foo` from B, exports `bar`
    let sym_foo = dummy_symbol_ref(idx_a, 0);
    let sym_bar = dummy_symbol_ref(idx_a, 1);
    let default_ref = dummy_symbol_ref(idx_a, 2);
    let ns_ref = dummy_symbol_ref(idx_a, 3);

    let mut named_imports = FxHashMap::default();
    named_imports.insert(
        sym_foo,
        NamedImport {
            imported_name: CompactString::new("foo"),
            local_symbol: sym_foo,
            record_idx: ImportRecordIdx::from_usize(0),
            is_type: false,
        },
    );

    let mut named_exports = FxHashMap::default();
    named_exports.insert(
        CompactString::new("bar"),
        LocalExport { exported_name: CompactString::new("bar"), local_symbol: sym_bar },
    );

    graph.add_normal_module(NormalModule {
        idx: idx_a,
        path: PathBuf::from("/a.js"),
        has_module_syntax: true,
        exports_kind: ExportsKind::None,
        has_top_level_await: false,
        side_effects: SideEffects::True,
        has_lazy_export: false,
        execution_order_sensitive: false,
        named_exports,
        named_imports,
        import_records: vec![ResolvedImportRecord {
            specifier: CompactString::new("./b"),
            resolved_module: Some(idx_b),
            kind: ImportKind::Static,
            namespace_ref: dummy_symbol_ref(idx_b, 0),
            meta: ImportRecordMeta::empty(),
        }],
        default_export_ref: default_ref,
        namespace_object_ref: ns_ref,
        star_export_entries: vec![],
        indirect_export_entries: vec![],
        wrap_kind: WrapKind::None,
        original_wrap_kind: WrapKind::None,
        wrapper_ref: None,
        required_by_other_module: false,
        resolved_exports: FxHashMap::default(),
        has_dynamic_exports: false,
        is_tla_or_contains_tla: false,
        propagated_side_effects: false,
        exec_order: u32::MAX,
    });

    // Module B: exports `foo`
    let sym_foo_b = dummy_symbol_ref(idx_b, 0);
    let default_ref_b = dummy_symbol_ref(idx_b, 1);
    let ns_ref_b = dummy_symbol_ref(idx_b, 2);

    let mut named_exports_b = FxHashMap::default();
    named_exports_b.insert(
        CompactString::new("foo"),
        LocalExport { exported_name: CompactString::new("foo"), local_symbol: sym_foo_b },
    );

    graph.add_normal_module(make_normal_module(
        idx_b,
        "/b.js",
        named_exports_b,
        FxHashMap::default(),
        vec![],
        default_ref_b,
        ns_ref_b,
    ));

    // Verify basic queries
    assert_eq!(graph.modules_len(), 2);
    let module_a_ref = graph.normal_module(idx_a).unwrap();
    assert!(module_a_ref.has_module_syntax);
    assert_eq!(module_a_ref.idx, idx_a);

    // Verify dependencies via import records
    let deps_a: Vec<ModuleIdx> = graph
        .normal_module(idx_a)
        .unwrap()
        .import_records
        .iter()
        .filter_map(|r| r.resolved_module)
        .collect();
    assert_eq!(deps_a.len(), 1);
    assert_eq!(deps_a[0], idx_b);

    let deps_b: Vec<ModuleIdx> = graph
        .normal_module(idx_b)
        .unwrap()
        .import_records
        .iter()
        .filter_map(|r| r.resolved_module)
        .collect();
    assert_eq!(deps_b.len(), 0);

    // Verify exports
    let a_exports: Vec<String> = module_a_ref.named_exports.keys().map(|k| k.to_string()).collect();
    assert!(a_exports.contains(&"bar".to_string()));

    let module_b_ref = graph.normal_module(idx_b).unwrap();
    let b_exports: Vec<String> = module_b_ref.named_exports.keys().map(|k| k.to_string()).collect();
    assert!(b_exports.contains(&"foo".to_string()));

    // Verify module count
    assert_eq!(graph.modules_len(), 2);
}

#[test]
fn test_symbol_ref_db() {
    let mut db = SymbolRefDb::new();
    db.ensure_modules(2);

    let module_a = ModuleIdx::from_usize(0);
    let module_b = ModuleIdx::from_usize(1);

    let sym_a = db.add_symbol(module_a, "foo".to_string());
    let sym_b = db.add_symbol(module_b, "bar".to_string());

    assert_eq!(db.canonical_ref_for(sym_a), sym_a);
    assert_eq!(db.canonical_ref_for(sym_b), sym_b);

    db.link(sym_a, sym_b);
    assert_eq!(db.canonical_ref_for(sym_a), sym_b);
    assert_eq!(db.canonical_ref_for(sym_b), sym_b);

    assert_eq!(db.symbol_name(sym_a), "foo");
    assert_eq!(db.symbol_name(sym_b), "bar");
}

#[test]
fn test_symbol_ref_db_chain() {
    let mut db = SymbolRefDb::new();
    db.ensure_modules(3);

    let m0 = ModuleIdx::from_usize(0);
    let m1 = ModuleIdx::from_usize(1);
    let m2 = ModuleIdx::from_usize(2);

    let s0 = db.add_symbol(m0, "x".to_string());
    let s1 = db.add_symbol(m1, "y".to_string());
    let s2 = db.add_symbol(m2, "z".to_string());

    db.link(s0, s1);
    db.link(s1, s2);

    assert_eq!(db.canonical_ref_for(s0), s2);
    assert_eq!(db.canonical_ref_for(s1), s2);
    assert_eq!(db.canonical_ref_for(s2), s2);
}

#[test]
fn test_symbol_ref_db_adopts_existing_symbol_ids() {
    use oxc_syntax::symbol::SymbolId;

    let mut db = SymbolRefDb::new();
    let module = ModuleIdx::from_usize(0);

    db.ensure_module_symbol_capacity(module, 3);

    let adopted = SymbolId::from_usize(2);
    db.set_symbol_name(module, adopted, "adopted".to_string());
    db.init_symbol_self_link(module, adopted);

    let adopted_ref = SymbolRef::new(module, adopted);
    assert_eq!(db.canonical_ref_for(adopted_ref), adopted_ref);
    assert_eq!(db.symbol_name(adopted_ref), "adopted");

    let synthetic = db.alloc_synthetic_symbol(module, "synthetic".to_string());
    assert_eq!(synthetic, SymbolRef::new(module, SymbolId::from_usize(3)));
    assert_eq!(db.symbol_name(synthetic), "synthetic");
}

#[test]
fn test_symbol_ref_db_link_uses_canonical_roots() {
    let mut db = SymbolRefDb::new();
    db.ensure_modules(4);

    let m0 = ModuleIdx::from_usize(0);
    let m1 = ModuleIdx::from_usize(1);
    let m2 = ModuleIdx::from_usize(2);
    let m3 = ModuleIdx::from_usize(3);

    let s0 = db.add_symbol(m0, "x".to_string());
    let s1 = db.add_symbol(m1, "y".to_string());
    let s2 = db.add_symbol(m2, "z".to_string());
    let s3 = db.add_symbol(m3, "w".to_string());

    db.link(s0, s1);
    db.link(s1, s2);
    db.link(s0, s3);

    assert_eq!(db.canonical_ref_for(s0), s3);
    assert_eq!(db.canonical_ref_for(s1), s3);
    assert_eq!(db.canonical_ref_for(s2), s3);
    assert_eq!(db.canonical_ref_for(s3), s3);
}

#[test]
fn test_symbol_ref_db_add_symbol_creates_sparse_modules() {
    let mut db = SymbolRefDb::new();
    let module = ModuleIdx::from_usize(2);

    let sym = db.add_symbol(module, "late".to_string());

    assert_eq!(sym, dummy_symbol_ref(module, 0));
    assert_eq!(db.canonical_ref_for(sym), sym);
    assert_eq!(db.symbol_name(sym), "late");
}

#[test]
fn test_module_record_from_syntax() {
    use oxc_allocator::Allocator;
    use oxc_module_graph::types::ModuleRecord;
    use oxc_parser::Parser;
    use oxc_span::SourceType;

    let allocator = Allocator::default();
    let source = r"
        import { foo } from './bar';
        export const baz = foo + 1;
        export default 42;
    ";

    let source_type = SourceType::mjs();
    let ret = Parser::new(&allocator, source, source_type).parse();
    let record = ModuleRecord::from_syntax(&ret.module_record);

    assert!(record.has_module_syntax);
    assert_eq!(record.import_entries.len(), 1);
    assert_eq!(record.import_entries[0].module_request.name(), "./bar");
    assert!(!record.local_export_entries.is_empty());
    assert!(record.export_default.is_some());
}

// --- Builder tests ---

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

#[test]
fn test_builder_basic() {
    use oxc_module_graph::default::ModuleGraphBuilder;

    let entry = fixtures_dir().join("entry.js");
    let result = ModuleGraphBuilder::new().build(&[entry]);

    assert_eq!(result.graph.modules_len(), 3);
    assert!(result.errors.is_empty(), "Unexpected errors: {:?}", result.errors);
}

#[test]
fn test_builder_circular() {
    use oxc_module_graph::default::ModuleGraphBuilder;

    let entry = fixtures_dir().join("circular_a.js");
    let result = ModuleGraphBuilder::new().build(&[entry]);

    assert_eq!(result.graph.modules_len(), 2);
    assert!(result.errors.is_empty());

    let deps_a: Vec<ModuleIdx> = result
        .graph
        .normal_module(ModuleIdx::from_usize(0))
        .unwrap()
        .import_records
        .iter()
        .filter_map(|r| r.resolved_module)
        .collect();
    let deps_b: Vec<ModuleIdx> = result
        .graph
        .normal_module(ModuleIdx::from_usize(1))
        .unwrap()
        .import_records
        .iter()
        .filter_map(|r| r.resolved_module)
        .collect();
    assert_eq!(deps_a.len(), 1);
    assert_eq!(deps_b.len(), 1);
}

#[test]
fn test_builder_reexport() {
    use oxc_module_graph::default::ModuleGraphBuilder;

    let entry = fixtures_dir().join("reexport.js");
    let result = ModuleGraphBuilder::new().build(&[entry]);

    assert_eq!(result.graph.modules_len(), 3);

    let reexport_module = result.graph.normal_module(ModuleIdx::from_usize(0)).unwrap();
    assert!(!reexport_module.star_export_entries.is_empty());
    assert!(!reexport_module.indirect_export_entries.is_empty());
}

#[test]
fn test_builder_tla_detection() {
    use oxc_module_graph::default::ModuleGraphBuilder;

    // Module with top-level await should have has_top_level_await = true
    let entry = fixtures_dir().join("tla.js");
    let result = ModuleGraphBuilder::new().build(&[entry]);

    assert!(result.errors.is_empty(), "Unexpected errors: {:?}", result.errors);
    let module = result.graph.normal_module(ModuleIdx::from_usize(0)).unwrap();
    assert!(module.has_top_level_await, "Expected has_top_level_await to be true for TLA module");

    // Module with await only inside a function should have has_top_level_await = false
    let entry = fixtures_dir().join("no_tla.js");
    let result = ModuleGraphBuilder::new().build(&[entry]);

    assert!(result.errors.is_empty(), "Unexpected errors: {:?}", result.errors);
    let module = result.graph.normal_module(ModuleIdx::from_usize(0)).unwrap();
    assert!(
        !module.has_top_level_await,
        "Expected has_top_level_await to be false for non-TLA module"
    );
}

#[test]
fn test_builder_external_modules() {
    use oxc_module_graph::default::ModuleGraphBuilder;

    let entry = fixtures_dir().join("external_import.js");
    let result = ModuleGraphBuilder::new().build(&[entry]);

    assert!(result.errors.is_empty(), "Unexpected errors: {:?}", result.errors);

    // The entry module itself + 2 external modules (react, lodash).
    // "react" appears twice in import statements but should be deduplicated to one ExternalModule.
    let normal_count = result.graph.normal_modules().count();
    assert_eq!(normal_count, 1, "Expected exactly 1 normal module");

    // Count external modules
    let external_count = result.graph.modules.iter().filter(|m| m.as_external().is_some()).count();
    assert_eq!(external_count, 2, "Expected 2 external modules (react, lodash)");

    // The entry module's import records should all have resolved_module set
    let entry_module = result.graph.normal_module(ModuleIdx::from_usize(0)).unwrap();
    for record in &entry_module.import_records {
        assert!(
            record.resolved_module.is_some(),
            "Expected all import records to have resolved_module, but '{}' has None",
            record.specifier
        );
    }
}

#[test]
fn test_builder_symbol_mapping() {
    use oxc_module_graph::default::ModuleGraphBuilder;

    let entry = fixtures_dir().join("symbol_export.js");
    let result = ModuleGraphBuilder::new().build(&[entry]);

    assert!(result.errors.is_empty(), "Unexpected errors: {:?}", result.errors);

    let module = result.graph.normal_module(ModuleIdx::from_usize(0)).unwrap();

    // Verify exports exist
    assert!(module.named_exports.contains_key("foo"), "Expected 'foo' export");
    assert!(module.named_exports.contains_key("bar"), "Expected 'bar' export");
    assert!(module.named_exports.contains_key("baz"), "Expected 'baz' export");

    // Verify each export's symbol maps to a real name in the symbol db
    for (export_name, local_export) in &module.named_exports {
        let sym_name = result.graph.symbol_name(local_export.local_symbol);
        assert_eq!(
            sym_name, export_name,
            "Expected export '{}' symbol name to match, got '{}'",
            export_name, sym_name
        );
    }

    // Verify that different exports have different SymbolIds
    let foo_sym = module.named_exports["foo"].local_symbol;
    let bar_sym = module.named_exports["bar"].local_symbol;
    let baz_sym = module.named_exports["baz"].local_symbol;
    assert_ne!(foo_sym, bar_sym, "foo and bar should have different symbols");
    assert_ne!(foo_sym, baz_sym, "foo and baz should have different symbols");
    assert_ne!(bar_sym, baz_sym, "bar and baz should have different symbols");
}

#[test]
fn test_builder_reexport_resolved_targets() {
    use oxc_module_graph::default::ModuleGraphBuilder;

    let entry = fixtures_dir().join("reexport.js");
    let result = ModuleGraphBuilder::new().build(&[entry]);

    assert!(result.errors.is_empty(), "Unexpected errors: {:?}", result.errors);

    let reexport_module = result.graph.normal_module(ModuleIdx::from_usize(0)).unwrap();

    // Star exports should have resolved targets (not None)
    for star in &reexport_module.star_export_entries {
        assert!(
            star.resolved_module.is_some(),
            "Expected star export '{}' to have a resolved module target",
            star.module_request
        );
    }

    // Indirect exports should have resolved targets (not None)
    for indirect in &reexport_module.indirect_export_entries {
        assert!(
            indirect.resolved_module.is_some(),
            "Expected indirect export '{}' to have a resolved module target",
            indirect.module_request
        );
    }
}

// --- Binding algorithm tests ---

/// Helper to build a two-module graph for binding tests.
fn two_module_graph_with_binding(export_name: &str, import_name: &str) -> ModuleGraph {
    let mut graph = ModuleGraph::new();
    graph.symbols.ensure_modules(2);

    let idx_a = ModuleIdx::from_usize(0);
    let idx_b = ModuleIdx::from_usize(1);

    // Module B: exports `export_name`
    let sym_b_export = graph.add_symbol(idx_b, export_name.to_string());
    let sym_b_default = graph.add_symbol(idx_b, "__default__".to_string());
    let sym_b_ns = graph.add_symbol(idx_b, "__namespace__".to_string());

    let mut named_exports_b = FxHashMap::default();
    named_exports_b.insert(
        CompactString::new(export_name),
        LocalExport { exported_name: CompactString::new(export_name), local_symbol: sym_b_export },
    );

    // Module A: imports `import_name` from B
    let sym_a_local = graph.add_symbol(idx_a, import_name.to_string());
    let sym_a_default = graph.add_symbol(idx_a, "__default__".to_string());
    let sym_a_ns = graph.add_symbol(idx_a, "__namespace__".to_string());

    let mut named_imports_a = FxHashMap::default();
    named_imports_a.insert(
        sym_a_local,
        NamedImport {
            imported_name: CompactString::new(import_name),
            local_symbol: sym_a_local,
            record_idx: ImportRecordIdx::from_usize(0),
            is_type: false,
        },
    );

    // Add modules in order
    graph.add_normal_module(make_normal_module(
        idx_a,
        "/a.js",
        FxHashMap::default(),
        named_imports_a,
        vec![ResolvedImportRecord {
            specifier: CompactString::new("./b"),
            resolved_module: Some(idx_b),
            kind: ImportKind::Static,
            namespace_ref: dummy_symbol_ref(ModuleIdx::from_usize(0), 0),
            meta: ImportRecordMeta::empty(),
        }],
        sym_a_default,
        sym_a_ns,
    ));

    graph.add_normal_module(make_normal_module(
        idx_b,
        "/b.js",
        named_exports_b,
        FxHashMap::default(),
        Vec::new(),
        sym_b_default,
        sym_b_ns,
    ));

    graph
}

#[test]
fn test_bind_named_import() {
    let mut graph = two_module_graph_with_binding("foo", "foo");

    bind_imports_and_exports(&mut graph);

    let idx_a = ModuleIdx::from_usize(0);
    let idx_b = ModuleIdx::from_usize(1);

    let a_foo = dummy_symbol_ref(idx_a, 0);
    let b_foo = dummy_symbol_ref(idx_b, 0);

    let canonical = graph.canonical_ref(a_foo);
    assert_eq!(canonical, b_foo, "A's foo should link to B's foo");
}

#[test]
fn test_bind_unresolved_import() {
    let mut graph = two_module_graph_with_binding("foo", "bar");

    bind_imports_and_exports(&mut graph);

    let idx_a = ModuleIdx::from_usize(0);
    let a_bar = dummy_symbol_ref(idx_a, 0);
    let canonical = graph.canonical_ref(a_bar);
    assert_eq!(canonical, a_bar, "A's bar should remain unlinked");
}

#[test]
fn test_bind_star_reexport() {
    let mut graph = ModuleGraph::new();
    graph.symbols.ensure_modules(3);

    let idx_a = ModuleIdx::from_usize(0);
    let idx_b = ModuleIdx::from_usize(1);
    let idx_c = ModuleIdx::from_usize(2);

    // Module C: exports "foo"
    let sym_c_foo = graph.add_symbol(idx_c, "foo".to_string());
    let sym_c_default = graph.add_symbol(idx_c, "__default__".to_string());
    let sym_c_ns = graph.add_symbol(idx_c, "__namespace__".to_string());

    let mut exports_c = FxHashMap::default();
    exports_c.insert(
        CompactString::new("foo"),
        LocalExport { exported_name: CompactString::new("foo"), local_symbol: sym_c_foo },
    );

    // Module B: `export * from './c'`
    let sym_b_default = graph.add_symbol(idx_b, "__default__".to_string());
    let sym_b_ns = graph.add_symbol(idx_b, "__namespace__".to_string());

    // Module A: `import { foo } from './b'`
    let sym_a_foo = graph.add_symbol(idx_a, "foo".to_string());
    let sym_a_default = graph.add_symbol(idx_a, "__default__".to_string());
    let sym_a_ns = graph.add_symbol(idx_a, "__namespace__".to_string());

    let mut named_imports_a = FxHashMap::default();
    named_imports_a.insert(
        sym_a_foo,
        NamedImport {
            imported_name: CompactString::new("foo"),
            local_symbol: sym_a_foo,
            record_idx: ImportRecordIdx::from_usize(0),
            is_type: false,
        },
    );

    graph.add_normal_module(make_normal_module(
        idx_a,
        "/a.js",
        FxHashMap::default(),
        named_imports_a,
        vec![ResolvedImportRecord {
            specifier: CompactString::new("./b"),
            resolved_module: Some(idx_b),
            kind: ImportKind::Static,
            namespace_ref: dummy_symbol_ref(ModuleIdx::from_usize(0), 0),
            meta: ImportRecordMeta::empty(),
        }],
        sym_a_default,
        sym_a_ns,
    ));

    let mut module_b = make_normal_module(
        idx_b,
        "/b.js",
        FxHashMap::default(),
        FxHashMap::default(),
        vec![ResolvedImportRecord {
            specifier: CompactString::new("./c"),
            resolved_module: Some(idx_c),
            kind: ImportKind::Static,
            namespace_ref: dummy_symbol_ref(ModuleIdx::from_usize(0), 0),
            meta: ImportRecordMeta::empty(),
        }],
        sym_b_default,
        sym_b_ns,
    );
    module_b.star_export_entries = vec![StarExportEntry {
        module_request: CompactString::new("./c"),
        resolved_module: Some(idx_c),
        span: oxc_span::Span::new(0, 0),
    }];
    graph.add_normal_module(module_b);

    graph.add_normal_module(make_normal_module(
        idx_c,
        "/c.js",
        exports_c,
        FxHashMap::default(),
        Vec::new(),
        sym_c_default,
        sym_c_ns,
    ));

    bind_imports_and_exports(&mut graph);

    let canonical = graph.canonical_ref(sym_a_foo);
    assert_eq!(canonical, sym_c_foo, "A's foo should link through B's star export to C's foo");
}

// --- Graph algorithm tests ---

/// Helper to build a simple graph with no imports/symbols for graph algorithm tests.
fn simple_graph(edges: &[(usize, usize)]) -> ModuleGraph {
    #[expect(clippy::tuple_array_conversions)]
    let max_idx = edges.iter().flat_map(|&(a, b)| [a, b]).max().map_or(0, |m| m + 1);

    let mut graph = ModuleGraph::new();
    graph.symbols.ensure_modules(max_idx);

    for i in 0..max_idx {
        let idx = ModuleIdx::from_usize(i);
        let default_ref = graph.add_symbol(idx, "__default__".to_string());
        let ns_ref = graph.add_symbol(idx, "__namespace__".to_string());

        let import_records: Vec<ResolvedImportRecord> = edges
            .iter()
            .filter(|&&(from, _)| from == i)
            .map(|&(_, to)| ResolvedImportRecord {
                specifier: CompactString::new(format!("./mod{to}")),
                resolved_module: Some(ModuleIdx::from_usize(to)),
                kind: ImportKind::Static,
                namespace_ref: dummy_symbol_ref(ModuleIdx::from_usize(0), 0),
                meta: ImportRecordMeta::empty(),
            })
            .collect();

        graph.add_normal_module(make_normal_module(
            idx,
            &format!("/mod{i}.js"),
            FxHashMap::default(),
            FxHashMap::default(),
            import_records,
            default_ref,
            ns_ref,
        ));
    }

    graph
}

#[test]
fn test_find_cycles_none() {
    let graph = simple_graph(&[(0, 1), (1, 2)]);
    let cycles = find_cycles(&graph);
    assert!(cycles.is_empty(), "DAG should have no cycles");
}

#[test]
fn test_find_cycles_simple() {
    let graph = simple_graph(&[(0, 1), (1, 0)]);
    let cycles = find_cycles(&graph);
    assert_eq!(cycles.len(), 1, "Should find one cycle");
    assert_eq!(cycles[0].len(), 2, "Cycle should have 2 modules");
}

#[test]
fn test_find_cycles_multiple() {
    let graph = simple_graph(&[(0, 1), (1, 0), (2, 3), (3, 2)]);
    let cycles = find_cycles(&graph);
    assert_eq!(cycles.len(), 2, "Should find two cycles");
}

#[test]
fn test_find_cycles_self_loop() {
    let graph = simple_graph(&[(0, 0)]);
    let cycles = find_cycles(&graph);
    assert_eq!(cycles.len(), 1, "Should find self-loop");
    assert_eq!(cycles[0].len(), 1, "Self-loop is a single-module cycle");
}

// --- match_imports with re-export chain following ---

#[test]
fn test_match_imports_reexport_chain() {
    let mut graph = ModuleGraph::new();
    graph.symbols.ensure_modules(3);

    let idx_a = ModuleIdx::from_usize(0);
    let idx_b = ModuleIdx::from_usize(1);
    let idx_c = ModuleIdx::from_usize(2);

    // Module C: local export "foo"
    let sym_c_foo = graph.add_symbol(idx_c, "foo".to_string());
    let sym_c_default = graph.add_symbol(idx_c, "__default__".to_string());
    let sym_c_ns = graph.add_symbol(idx_c, "__namespace__".to_string());

    let mut exports_c = FxHashMap::default();
    exports_c.insert(
        CompactString::new("foo"),
        LocalExport { exported_name: CompactString::new("foo"), local_symbol: sym_c_foo },
    );

    // Module B: imports "foo" from C and re-exports it
    let sym_b_foo = graph.add_symbol(idx_b, "foo".to_string());
    let sym_b_default = graph.add_symbol(idx_b, "__default__".to_string());
    let sym_b_ns = graph.add_symbol(idx_b, "__namespace__".to_string());

    let mut named_imports_b = FxHashMap::default();
    named_imports_b.insert(
        sym_b_foo,
        NamedImport {
            imported_name: CompactString::new("foo"),
            local_symbol: sym_b_foo,
            record_idx: ImportRecordIdx::from_usize(0),
            is_type: false,
        },
    );
    let mut exports_b = FxHashMap::default();
    exports_b.insert(
        CompactString::new("foo"),
        LocalExport { exported_name: CompactString::new("foo"), local_symbol: sym_b_foo },
    );

    // Module A: imports "foo" from B
    let sym_a_foo = graph.add_symbol(idx_a, "foo".to_string());
    let sym_a_default = graph.add_symbol(idx_a, "__default__".to_string());
    let sym_a_ns = graph.add_symbol(idx_a, "__namespace__".to_string());

    let mut named_imports_a = FxHashMap::default();
    named_imports_a.insert(
        sym_a_foo,
        NamedImport {
            imported_name: CompactString::new("foo"),
            local_symbol: sym_a_foo,
            record_idx: ImportRecordIdx::from_usize(0),
            is_type: false,
        },
    );

    graph.add_normal_module(make_normal_module(
        idx_a,
        "/a.js",
        FxHashMap::default(),
        named_imports_a,
        vec![ResolvedImportRecord {
            specifier: CompactString::new("./b"),
            resolved_module: Some(idx_b),
            kind: ImportKind::Static,
            namespace_ref: dummy_symbol_ref(ModuleIdx::from_usize(0), 0),
            meta: ImportRecordMeta::empty(),
        }],
        sym_a_default,
        sym_a_ns,
    ));

    graph.add_normal_module(make_normal_module(
        idx_b,
        "/b.js",
        exports_b,
        named_imports_b,
        vec![ResolvedImportRecord {
            specifier: CompactString::new("./c"),
            resolved_module: Some(idx_c),
            kind: ImportKind::Static,
            namespace_ref: dummy_symbol_ref(ModuleIdx::from_usize(0), 0),
            meta: ImportRecordMeta::empty(),
        }],
        sym_b_default,
        sym_b_ns,
    ));

    graph.add_normal_module(make_normal_module(
        idx_c,
        "/c.js",
        exports_c,
        FxHashMap::default(),
        Vec::new(),
        sym_c_default,
        sym_c_ns,
    ));

    bind_imports_and_exports(&mut graph);

    let canonical = graph.canonical_ref(sym_a_foo);
    assert_eq!(canonical, sym_c_foo, "A's foo should follow re-export chain to C's foo");
}

#[test]
fn test_match_imports_unresolved() {
    let mut graph = two_module_graph_with_binding("foo", "bar");

    bind_imports_and_exports(&mut graph);

    assert!(!graph.binding_errors().is_empty(), "Expected unresolved import error");
}

#[test]
fn test_match_imports_deep_chain() {
    let mut graph = ModuleGraph::new();
    graph.symbols.ensure_modules(4);

    let idx_a = ModuleIdx::from_usize(0);
    let idx_b = ModuleIdx::from_usize(1);
    let idx_c = ModuleIdx::from_usize(2);
    let idx_d = ModuleIdx::from_usize(3);

    // D: local export "x"
    let sym_d_x = graph.add_symbol(idx_d, "x".to_string());
    let sym_d_default = graph.add_symbol(idx_d, "__default__".to_string());
    let sym_d_ns = graph.add_symbol(idx_d, "__namespace__".to_string());

    let mut exports_d = FxHashMap::default();
    exports_d.insert(
        CompactString::new("x"),
        LocalExport { exported_name: CompactString::new("x"), local_symbol: sym_d_x },
    );

    // C: import "x" from D, re-export as "x"
    let sym_c_x = graph.add_symbol(idx_c, "x".to_string());
    let sym_c_default = graph.add_symbol(idx_c, "__default__".to_string());
    let sym_c_ns = graph.add_symbol(idx_c, "__namespace__".to_string());

    let mut named_imports_c = FxHashMap::default();
    named_imports_c.insert(
        sym_c_x,
        NamedImport {
            imported_name: CompactString::new("x"),
            local_symbol: sym_c_x,
            record_idx: ImportRecordIdx::from_usize(0),
            is_type: false,
        },
    );
    let mut exports_c = FxHashMap::default();
    exports_c.insert(
        CompactString::new("x"),
        LocalExport { exported_name: CompactString::new("x"), local_symbol: sym_c_x },
    );

    // B: import "x" from C, re-export as "x"
    let sym_b_x = graph.add_symbol(idx_b, "x".to_string());
    let sym_b_default = graph.add_symbol(idx_b, "__default__".to_string());
    let sym_b_ns = graph.add_symbol(idx_b, "__namespace__".to_string());

    let mut named_imports_b = FxHashMap::default();
    named_imports_b.insert(
        sym_b_x,
        NamedImport {
            imported_name: CompactString::new("x"),
            local_symbol: sym_b_x,
            record_idx: ImportRecordIdx::from_usize(0),
            is_type: false,
        },
    );
    let mut exports_b = FxHashMap::default();
    exports_b.insert(
        CompactString::new("x"),
        LocalExport { exported_name: CompactString::new("x"), local_symbol: sym_b_x },
    );

    // A: import "x" from B
    let sym_a_x = graph.add_symbol(idx_a, "x".to_string());
    let sym_a_default = graph.add_symbol(idx_a, "__default__".to_string());
    let sym_a_ns = graph.add_symbol(idx_a, "__namespace__".to_string());

    let mut named_imports_a = FxHashMap::default();
    named_imports_a.insert(
        sym_a_x,
        NamedImport {
            imported_name: CompactString::new("x"),
            local_symbol: sym_a_x,
            record_idx: ImportRecordIdx::from_usize(0),
            is_type: false,
        },
    );

    graph.add_normal_module(make_normal_module(
        idx_a,
        "/a.js",
        FxHashMap::default(),
        named_imports_a,
        vec![ResolvedImportRecord {
            specifier: CompactString::new("./b"),
            resolved_module: Some(idx_b),
            kind: ImportKind::Static,
            namespace_ref: dummy_symbol_ref(ModuleIdx::from_usize(0), 0),
            meta: ImportRecordMeta::empty(),
        }],
        sym_a_default,
        sym_a_ns,
    ));
    graph.add_normal_module(make_normal_module(
        idx_b,
        "/b.js",
        exports_b,
        named_imports_b,
        vec![ResolvedImportRecord {
            specifier: CompactString::new("./c"),
            resolved_module: Some(idx_c),
            kind: ImportKind::Static,
            namespace_ref: dummy_symbol_ref(ModuleIdx::from_usize(0), 0),
            meta: ImportRecordMeta::empty(),
        }],
        sym_b_default,
        sym_b_ns,
    ));
    graph.add_normal_module(make_normal_module(
        idx_c,
        "/c.js",
        exports_c,
        named_imports_c,
        vec![ResolvedImportRecord {
            specifier: CompactString::new("./d"),
            resolved_module: Some(idx_d),
            kind: ImportKind::Static,
            namespace_ref: dummy_symbol_ref(ModuleIdx::from_usize(0), 0),
            meta: ImportRecordMeta::empty(),
        }],
        sym_c_default,
        sym_c_ns,
    ));
    graph.add_normal_module(make_normal_module(
        idx_d,
        "/d.js",
        exports_d,
        FxHashMap::default(),
        Vec::new(),
        sym_d_default,
        sym_d_ns,
    ));

    bind_imports_and_exports(&mut graph);

    let canonical = graph.canonical_ref(sym_a_x);
    assert_eq!(canonical, sym_d_x, "A's x should follow 3-level chain to D's x");
}

#[test]
fn test_match_imports_circular_reexport() {
    let mut graph = ModuleGraph::new();
    graph.symbols.ensure_modules(3);

    let idx_a = ModuleIdx::from_usize(0);
    let idx_b = ModuleIdx::from_usize(1);
    let idx_c = ModuleIdx::from_usize(2);

    // B: import "x" from C, export "x"
    let sym_b_x = graph.add_symbol(idx_b, "x".to_string());
    let sym_b_default = graph.add_symbol(idx_b, "__default__".to_string());
    let sym_b_ns = graph.add_symbol(idx_b, "__namespace__".to_string());

    let mut named_imports_b = FxHashMap::default();
    named_imports_b.insert(
        sym_b_x,
        NamedImport {
            imported_name: CompactString::new("x"),
            local_symbol: sym_b_x,
            record_idx: ImportRecordIdx::from_usize(0),
            is_type: false,
        },
    );
    let mut exports_b = FxHashMap::default();
    exports_b.insert(
        CompactString::new("x"),
        LocalExport { exported_name: CompactString::new("x"), local_symbol: sym_b_x },
    );

    // C: import "x" from B, export "x"
    let sym_c_x = graph.add_symbol(idx_c, "x".to_string());
    let sym_c_default = graph.add_symbol(idx_c, "__default__".to_string());
    let sym_c_ns = graph.add_symbol(idx_c, "__namespace__".to_string());

    let mut named_imports_c = FxHashMap::default();
    named_imports_c.insert(
        sym_c_x,
        NamedImport {
            imported_name: CompactString::new("x"),
            local_symbol: sym_c_x,
            record_idx: ImportRecordIdx::from_usize(0),
            is_type: false,
        },
    );
    let mut exports_c = FxHashMap::default();
    exports_c.insert(
        CompactString::new("x"),
        LocalExport { exported_name: CompactString::new("x"), local_symbol: sym_c_x },
    );

    // A: import "x" from B
    let sym_a_x = graph.add_symbol(idx_a, "x".to_string());
    let sym_a_default = graph.add_symbol(idx_a, "__default__".to_string());
    let sym_a_ns = graph.add_symbol(idx_a, "__namespace__".to_string());

    let mut named_imports_a = FxHashMap::default();
    named_imports_a.insert(
        sym_a_x,
        NamedImport {
            imported_name: CompactString::new("x"),
            local_symbol: sym_a_x,
            record_idx: ImportRecordIdx::from_usize(0),
            is_type: false,
        },
    );

    graph.add_normal_module(make_normal_module(
        idx_a,
        "/a.js",
        FxHashMap::default(),
        named_imports_a,
        vec![ResolvedImportRecord {
            specifier: CompactString::new("./b"),
            resolved_module: Some(idx_b),
            kind: ImportKind::Static,
            namespace_ref: dummy_symbol_ref(ModuleIdx::from_usize(0), 0),
            meta: ImportRecordMeta::empty(),
        }],
        sym_a_default,
        sym_a_ns,
    ));
    graph.add_normal_module(make_normal_module(
        idx_b,
        "/b.js",
        exports_b,
        named_imports_b,
        vec![ResolvedImportRecord {
            specifier: CompactString::new("./c"),
            resolved_module: Some(idx_c),
            kind: ImportKind::Static,
            namespace_ref: dummy_symbol_ref(ModuleIdx::from_usize(0), 0),
            meta: ImportRecordMeta::empty(),
        }],
        sym_b_default,
        sym_b_ns,
    ));
    graph.add_normal_module(make_normal_module(
        idx_c,
        "/c.js",
        exports_c,
        named_imports_c,
        vec![ResolvedImportRecord {
            specifier: CompactString::new("./b"),
            resolved_module: Some(idx_b),
            kind: ImportKind::Static,
            namespace_ref: dummy_symbol_ref(ModuleIdx::from_usize(0), 0),
            meta: ImportRecordMeta::empty(),
        }],
        sym_c_default,
        sym_c_ns,
    ));

    // Should not hang — cycle detection kicks in.
    bind_imports_and_exports(&mut graph);

    let canonical = graph.canonical_ref(sym_a_x);
    assert_eq!(canonical, sym_a_x, "A's x should remain unlinked due to cycle");
}

// --- Dynamic exports tests ---

fn make_simple_module(
    graph: &mut ModuleGraph,
    idx: ModuleIdx,
    _path: &str,
) -> (SymbolRef, SymbolRef) {
    let default_ref = graph.add_symbol(idx, "__default__".to_string());
    let ns_ref = graph.add_symbol(idx, "__namespace__".to_string());
    (default_ref, ns_ref)
}

#[test]
fn test_dynamic_exports_cjs_module() {
    let mut graph = ModuleGraph::new();
    graph.symbols.ensure_modules(1);

    let idx = ModuleIdx::from_usize(0);
    let (default_ref, ns_ref) = make_simple_module(&mut graph, idx, "/a.js");

    let mut module = make_normal_module(
        idx,
        "/a.js",
        FxHashMap::default(),
        FxHashMap::default(),
        Vec::new(),
        default_ref,
        ns_ref,
    );
    module.exports_kind = ExportsKind::CommonJs;
    graph.add_normal_module(module);

    let dynamic = compute_has_dynamic_exports(&graph);
    assert!(dynamic.contains(&idx), "CJS module should have dynamic exports");
}

#[test]
fn test_dynamic_exports_external_star_target() {
    use oxc_module_graph::ExternalModule;

    let mut graph = ModuleGraph::new();
    graph.symbols.ensure_modules(2);

    let idx_a = ModuleIdx::from_usize(0);
    let idx_ext = ModuleIdx::from_usize(1);

    let (default_a, ns_a) = make_simple_module(&mut graph, idx_a, "/a.js");
    let ns_ext = graph.add_symbol(idx_ext, "__namespace__".to_string());

    // External module
    graph.add_external_module(ExternalModule {
        idx: idx_ext,
        specifier: CompactString::new("external"),
        side_effects: SideEffects::True,
        namespace_ref: ns_ext,
        exec_order: u32::MAX,
    });

    // Module A: export * from 'external'
    let mut module_a = make_normal_module(
        idx_a,
        "/a.js",
        FxHashMap::default(),
        FxHashMap::default(),
        Vec::new(),
        default_a,
        ns_a,
    );
    module_a.star_export_entries = vec![StarExportEntry {
        module_request: CompactString::new("external"),
        resolved_module: Some(idx_ext),
        span: oxc_span::Span::new(0, 0),
    }];
    graph.add_normal_module(module_a);

    let dynamic = compute_has_dynamic_exports(&graph);
    assert!(dynamic.contains(&idx_a), "Star export from external should be dynamic");
}

#[test]
fn test_dynamic_exports_transitive_from_cjs() {
    let mut graph = ModuleGraph::new();
    graph.symbols.ensure_modules(2);

    let idx_a = ModuleIdx::from_usize(0);
    let idx_b = ModuleIdx::from_usize(1);

    let (default_a, ns_a) = make_simple_module(&mut graph, idx_a, "/a.js");
    let (default_b, ns_b) = make_simple_module(&mut graph, idx_b, "/b.js");

    // B is CJS
    let mut module_b = make_normal_module(
        idx_b,
        "/b.js",
        FxHashMap::default(),
        FxHashMap::default(),
        Vec::new(),
        default_b,
        ns_b,
    );
    module_b.exports_kind = ExportsKind::CommonJs;
    graph.add_normal_module(module_b);

    // A: export * from './b'
    let mut module_a = make_normal_module(
        idx_a,
        "/a.js",
        FxHashMap::default(),
        FxHashMap::default(),
        Vec::new(),
        default_a,
        ns_a,
    );
    module_a.star_export_entries = vec![StarExportEntry {
        module_request: CompactString::new("./b"),
        resolved_module: Some(idx_b),
        span: oxc_span::Span::new(0, 0),
    }];
    graph.add_normal_module(module_a);

    let dynamic = compute_has_dynamic_exports(&graph);
    assert!(dynamic.contains(&idx_a), "Transitive star from CJS should be dynamic");
    assert!(dynamic.contains(&idx_b), "CJS module itself should be dynamic");
}

#[test]
fn test_dynamic_exports_pure_esm_not_dynamic() {
    let mut graph = ModuleGraph::new();
    graph.symbols.ensure_modules(2);

    let idx_a = ModuleIdx::from_usize(0);
    let idx_b = ModuleIdx::from_usize(1);

    let (default_a, ns_a) = make_simple_module(&mut graph, idx_a, "/a.js");
    let (default_b, ns_b) = make_simple_module(&mut graph, idx_b, "/b.js");

    graph.add_normal_module(make_normal_module(
        idx_b,
        "/b.js",
        FxHashMap::default(),
        FxHashMap::default(),
        Vec::new(),
        default_b,
        ns_b,
    ));

    let mut module_a = make_normal_module(
        idx_a,
        "/a.js",
        FxHashMap::default(),
        FxHashMap::default(),
        Vec::new(),
        default_a,
        ns_a,
    );
    module_a.star_export_entries = vec![StarExportEntry {
        module_request: CompactString::new("./b"),
        resolved_module: Some(idx_b),
        span: oxc_span::Span::new(0, 0),
    }];
    graph.add_normal_module(module_a);

    let dynamic = compute_has_dynamic_exports(&graph);
    assert!(!dynamic.contains(&idx_a), "Pure ESM should not have dynamic exports");
}

#[test]
fn test_dynamic_exports_cycle_no_infinite_loop() {
    let mut graph = ModuleGraph::new();
    graph.symbols.ensure_modules(2);

    let idx_a = ModuleIdx::from_usize(0);
    let idx_b = ModuleIdx::from_usize(1);

    let (default_a, ns_a) = make_simple_module(&mut graph, idx_a, "/a.js");
    let (default_b, ns_b) = make_simple_module(&mut graph, idx_b, "/b.js");

    // A star-exports from B, B star-exports from A (cycle)
    let mut module_a = make_normal_module(
        idx_a,
        "/a.js",
        FxHashMap::default(),
        FxHashMap::default(),
        Vec::new(),
        default_a,
        ns_a,
    );
    module_a.star_export_entries = vec![StarExportEntry {
        module_request: CompactString::new("./b"),
        resolved_module: Some(idx_b),
        span: oxc_span::Span::new(0, 0),
    }];

    let mut module_b = make_normal_module(
        idx_b,
        "/b.js",
        FxHashMap::default(),
        FxHashMap::default(),
        Vec::new(),
        default_b,
        ns_b,
    );
    module_b.star_export_entries = vec![StarExportEntry {
        module_request: CompactString::new("./a"),
        resolved_module: Some(idx_a),
        span: oxc_span::Span::new(0, 0),
    }];

    graph.add_normal_module(module_a);
    graph.add_normal_module(module_b);

    // Should not hang
    let dynamic = compute_has_dynamic_exports(&graph);
    assert!(!dynamic.contains(&idx_a));
    assert!(!dynamic.contains(&idx_b));
}

// --- TLA tests ---

#[test]
fn test_tla_direct() {
    let mut graph = ModuleGraph::new();
    graph.symbols.ensure_modules(1);

    let idx = ModuleIdx::from_usize(0);
    let (default_ref, ns_ref) = make_simple_module(&mut graph, idx, "/a.js");

    let mut module = make_normal_module(
        idx,
        "/a.js",
        FxHashMap::default(),
        FxHashMap::default(),
        Vec::new(),
        default_ref,
        ns_ref,
    );
    module.has_top_level_await = true;
    graph.add_normal_module(module);

    let tla = compute_tla(&graph);
    assert!(tla.contains(&idx), "Module with TLA should be in result");
}

#[test]
fn test_tla_transitive_static() {
    let mut graph = ModuleGraph::new();
    graph.symbols.ensure_modules(2);

    let idx_a = ModuleIdx::from_usize(0);
    let idx_b = ModuleIdx::from_usize(1);

    let (default_a, ns_a) = make_simple_module(&mut graph, idx_a, "/a.js");
    let (default_b, ns_b) = make_simple_module(&mut graph, idx_b, "/b.js");

    // B has TLA
    let mut module_b = make_normal_module(
        idx_b,
        "/b.js",
        FxHashMap::default(),
        FxHashMap::default(),
        Vec::new(),
        default_b,
        ns_b,
    );
    module_b.has_top_level_await = true;
    graph.add_normal_module(module_b);

    // A statically imports B
    graph.add_normal_module(make_normal_module(
        idx_a,
        "/a.js",
        FxHashMap::default(),
        FxHashMap::default(),
        vec![ResolvedImportRecord {
            specifier: CompactString::new("./b"),
            resolved_module: Some(idx_b),
            kind: ImportKind::Static,
            namespace_ref: dummy_symbol_ref(ModuleIdx::from_usize(0), 0),
            meta: ImportRecordMeta::empty(),
        }],
        default_a,
        ns_a,
    ));

    let tla = compute_tla(&graph);
    assert!(tla.contains(&idx_a), "Static import of TLA module propagates");
    assert!(tla.contains(&idx_b));
}

#[test]
fn test_tla_dynamic_import_not_propagated() {
    let mut graph = ModuleGraph::new();
    graph.symbols.ensure_modules(2);

    let idx_a = ModuleIdx::from_usize(0);
    let idx_b = ModuleIdx::from_usize(1);

    let (default_a, ns_a) = make_simple_module(&mut graph, idx_a, "/a.js");
    let (default_b, ns_b) = make_simple_module(&mut graph, idx_b, "/b.js");

    let mut module_b = make_normal_module(
        idx_b,
        "/b.js",
        FxHashMap::default(),
        FxHashMap::default(),
        Vec::new(),
        default_b,
        ns_b,
    );
    module_b.has_top_level_await = true;
    graph.add_normal_module(module_b);

    // A dynamically imports B
    graph.add_normal_module(make_normal_module(
        idx_a,
        "/a.js",
        FxHashMap::default(),
        FxHashMap::default(),
        vec![ResolvedImportRecord {
            specifier: CompactString::new("./b"),
            resolved_module: Some(idx_b),
            kind: ImportKind::Dynamic,
            namespace_ref: dummy_symbol_ref(ModuleIdx::from_usize(0), 0),
            meta: ImportRecordMeta::empty(),
        }],
        default_a,
        ns_a,
    ));

    let tla = compute_tla(&graph);
    assert!(!tla.contains(&idx_a), "Dynamic import should not propagate TLA");
    assert!(tla.contains(&idx_b));
}

#[test]
fn test_tla_pure_esm_not_tla() {
    let graph = simple_graph(&[(0, 1)]);
    let tla = compute_tla(&graph);
    assert!(tla.is_empty(), "No TLA modules means empty result");
}

#[test]
fn test_tla_cycle_no_infinite_loop() {
    let mut graph = ModuleGraph::new();
    graph.symbols.ensure_modules(2);

    let idx_a = ModuleIdx::from_usize(0);
    let idx_b = ModuleIdx::from_usize(1);

    let (default_a, ns_a) = make_simple_module(&mut graph, idx_a, "/a.js");
    let (default_b, ns_b) = make_simple_module(&mut graph, idx_b, "/b.js");

    // Circular static imports, A has TLA
    let mut module_a = make_normal_module(
        idx_a,
        "/a.js",
        FxHashMap::default(),
        FxHashMap::default(),
        vec![ResolvedImportRecord {
            specifier: CompactString::new("./b"),
            resolved_module: Some(idx_b),
            kind: ImportKind::Static,
            namespace_ref: dummy_symbol_ref(ModuleIdx::from_usize(0), 0),
            meta: ImportRecordMeta::empty(),
        }],
        default_a,
        ns_a,
    );
    module_a.has_top_level_await = true;
    graph.add_normal_module(module_a);

    graph.add_normal_module(make_normal_module(
        idx_b,
        "/b.js",
        FxHashMap::default(),
        FxHashMap::default(),
        vec![ResolvedImportRecord {
            specifier: CompactString::new("./a"),
            resolved_module: Some(idx_a),
            kind: ImportKind::Static,
            namespace_ref: dummy_symbol_ref(ModuleIdx::from_usize(0), 0),
            meta: ImportRecordMeta::empty(),
        }],
        default_b,
        ns_b,
    ));

    let tla = compute_tla(&graph);
    assert!(tla.contains(&idx_a));
    assert!(tla.contains(&idx_b));
}

// --- Exec order tests ---

#[test]
fn test_exec_order_linear_chain() {
    use oxc_module_graph::LinkConfig;

    // A -> B -> C
    let mut graph = simple_graph(&[(0, 1), (1, 2)]);
    graph.set_entries(vec![ModuleIdx::from_usize(0)]);

    let config = LinkConfig::default();
    let result = compute_exec_order(&graph, &config);

    assert_eq!(result.sorted.len(), 3);
    // C first, then B, then A (post-order)
    assert_eq!(result.sorted[0], ModuleIdx::from_usize(2));
    assert_eq!(result.sorted[1], ModuleIdx::from_usize(1));
    assert_eq!(result.sorted[2], ModuleIdx::from_usize(0));
}

#[test]
fn test_exec_order_diamond() {
    use oxc_module_graph::LinkConfig;

    // A -> B, A -> C, B -> D, C -> D
    let mut graph = simple_graph(&[(0, 1), (0, 2), (1, 3), (2, 3)]);
    graph.set_entries(vec![ModuleIdx::from_usize(0)]);

    let config = LinkConfig::default();
    let result = compute_exec_order(&graph, &config);

    assert_eq!(result.sorted.len(), 4);
    // D should be first (deepest dependency)
    assert_eq!(result.sorted[0], ModuleIdx::from_usize(3));
}

#[test]
fn test_exec_order_dynamic_import_excluded() {
    use oxc_module_graph::LinkConfig;

    let mut graph = ModuleGraph::new();
    graph.symbols.ensure_modules(2);

    let idx_a = ModuleIdx::from_usize(0);
    let idx_b = ModuleIdx::from_usize(1);

    let (default_a, ns_a) = make_simple_module(&mut graph, idx_a, "/a.js");
    let (default_b, ns_b) = make_simple_module(&mut graph, idx_b, "/b.js");

    graph.add_normal_module(make_normal_module(
        idx_a,
        "/a.js",
        FxHashMap::default(),
        FxHashMap::default(),
        vec![ResolvedImportRecord {
            specifier: CompactString::new("./b"),
            resolved_module: Some(idx_b),
            kind: ImportKind::Dynamic,
            namespace_ref: dummy_symbol_ref(ModuleIdx::from_usize(0), 0),
            meta: ImportRecordMeta::empty(),
        }],
        default_a,
        ns_a,
    ));
    graph.add_normal_module(make_normal_module(
        idx_b,
        "/b.js",
        FxHashMap::default(),
        FxHashMap::default(),
        Vec::new(),
        default_b,
        ns_b,
    ));

    graph.set_entries(vec![idx_a]);

    let config = LinkConfig { include_dynamic_imports: false, ..Default::default() };
    let result = compute_exec_order(&graph, &config);

    assert_eq!(result.sorted.len(), 1, "Dynamic import excluded");
    assert_eq!(result.sorted[0], idx_a);
}

#[test]
fn test_exec_order_dynamic_import_included() {
    use oxc_module_graph::LinkConfig;

    let mut graph = ModuleGraph::new();
    graph.symbols.ensure_modules(2);

    let idx_a = ModuleIdx::from_usize(0);
    let idx_b = ModuleIdx::from_usize(1);

    let (default_a, ns_a) = make_simple_module(&mut graph, idx_a, "/a.js");
    let (default_b, ns_b) = make_simple_module(&mut graph, idx_b, "/b.js");

    graph.add_normal_module(make_normal_module(
        idx_a,
        "/a.js",
        FxHashMap::default(),
        FxHashMap::default(),
        vec![ResolvedImportRecord {
            specifier: CompactString::new("./b"),
            resolved_module: Some(idx_b),
            kind: ImportKind::Dynamic,
            namespace_ref: dummy_symbol_ref(ModuleIdx::from_usize(0), 0),
            meta: ImportRecordMeta::empty(),
        }],
        default_a,
        ns_a,
    ));
    graph.add_normal_module(make_normal_module(
        idx_b,
        "/b.js",
        FxHashMap::default(),
        FxHashMap::default(),
        Vec::new(),
        default_b,
        ns_b,
    ));

    graph.set_entries(vec![idx_a]);

    let config = LinkConfig { include_dynamic_imports: true, ..Default::default() };
    let result = compute_exec_order(&graph, &config);

    assert_eq!(result.sorted.len(), 2, "Dynamic import included");
}

#[test]
fn test_exec_order_runtime_first() {
    use oxc_module_graph::LinkConfig;

    let mut graph = simple_graph(&[(0, 1)]);

    let idx_rt = ModuleIdx::from_usize(2);
    graph.symbols.ensure_modules(3);
    let (default_rt, ns_rt) = make_simple_module(&mut graph, idx_rt, "/runtime.js");
    graph.add_normal_module(make_normal_module(
        idx_rt,
        "/runtime.js",
        FxHashMap::default(),
        FxHashMap::default(),
        Vec::new(),
        default_rt,
        ns_rt,
    ));

    graph.set_entries(vec![ModuleIdx::from_usize(0)]);
    graph.set_runtime(idx_rt);

    let config = LinkConfig::default();
    let result = compute_exec_order(&graph, &config);

    assert_eq!(result.sorted[0], idx_rt, "Runtime should be first");
}

#[test]
fn test_exec_order_hot_accept_skipped() {
    use oxc_module_graph::LinkConfig;

    let mut graph = ModuleGraph::new();
    graph.symbols.ensure_modules(2);

    let idx_a = ModuleIdx::from_usize(0);
    let idx_b = ModuleIdx::from_usize(1);

    let (default_a, ns_a) = make_simple_module(&mut graph, idx_a, "/a.js");
    let (default_b, ns_b) = make_simple_module(&mut graph, idx_b, "/b.js");

    graph.add_normal_module(make_normal_module(
        idx_a,
        "/a.js",
        FxHashMap::default(),
        FxHashMap::default(),
        vec![ResolvedImportRecord {
            specifier: CompactString::new("./b"),
            resolved_module: Some(idx_b),
            kind: ImportKind::HotAccept,
            namespace_ref: dummy_symbol_ref(ModuleIdx::from_usize(0), 0),
            meta: ImportRecordMeta::empty(),
        }],
        default_a,
        ns_a,
    ));
    graph.add_normal_module(make_normal_module(
        idx_b,
        "/b.js",
        FxHashMap::default(),
        FxHashMap::default(),
        Vec::new(),
        default_b,
        ns_b,
    ));

    graph.set_entries(vec![idx_a]);

    let config = LinkConfig::default();
    let result = compute_exec_order(&graph, &config);

    assert_eq!(result.sorted.len(), 1, "HotAccept should be skipped");
    assert_eq!(result.sorted[0], idx_a);
}

// --- Side effects tests ---

#[test]
fn test_side_effects_has_side_effects() {
    use oxc_module_graph::LinkConfig;

    let graph = simple_graph(&[]);
    let config = LinkConfig::default();
    let se = determine_side_effects(&graph, &config);

    // simple_graph creates modules with SideEffects::True
    for (_, has) in &se {
        assert!(*has, "Default modules should have side effects");
    }
}

#[test]
fn test_side_effects_no_treeshake() {
    use oxc_module_graph::LinkConfig;

    let mut graph = ModuleGraph::new();
    graph.symbols.ensure_modules(1);

    let idx = ModuleIdx::from_usize(0);
    let (default_ref, ns_ref) = make_simple_module(&mut graph, idx, "/a.js");

    let mut module = make_normal_module(
        idx,
        "/a.js",
        FxHashMap::default(),
        FxHashMap::default(),
        Vec::new(),
        default_ref,
        ns_ref,
    );
    module.side_effects = SideEffects::NoTreeshake;
    graph.add_normal_module(module);

    let config = LinkConfig::default();
    let se = determine_side_effects(&graph, &config);
    assert_eq!(se[&idx], true, "NoTreeshake should always have side effects");
}

#[test]
fn test_side_effects_pure_no_deps() {
    use oxc_module_graph::LinkConfig;

    let mut graph = ModuleGraph::new();
    graph.symbols.ensure_modules(1);

    let idx = ModuleIdx::from_usize(0);
    let (default_ref, ns_ref) = make_simple_module(&mut graph, idx, "/a.js");

    let mut module = make_normal_module(
        idx,
        "/a.js",
        FxHashMap::default(),
        FxHashMap::default(),
        Vec::new(),
        default_ref,
        ns_ref,
    );
    module.side_effects = SideEffects::False;
    graph.add_normal_module(module);

    let config = LinkConfig::default();
    let se = determine_side_effects(&graph, &config);
    assert_eq!(se[&idx], false, "Pure module with no deps should be side-effect free");
}

#[test]
fn test_side_effects_transitive() {
    use oxc_module_graph::LinkConfig;

    let mut graph = ModuleGraph::new();
    graph.symbols.ensure_modules(2);

    let idx_a = ModuleIdx::from_usize(0);
    let idx_b = ModuleIdx::from_usize(1);

    let (default_a, ns_a) = make_simple_module(&mut graph, idx_a, "/a.js");
    let (default_b, ns_b) = make_simple_module(&mut graph, idx_b, "/b.js");

    // B has side effects
    graph.add_normal_module(make_normal_module(
        idx_b,
        "/b.js",
        FxHashMap::default(),
        FxHashMap::default(),
        Vec::new(),
        default_b,
        ns_b,
    ));

    // A is pure but imports B
    let mut module_a = make_normal_module(
        idx_a,
        "/a.js",
        FxHashMap::default(),
        FxHashMap::default(),
        vec![ResolvedImportRecord {
            specifier: CompactString::new("./b"),
            resolved_module: Some(idx_b),
            kind: ImportKind::Static,
            namespace_ref: dummy_symbol_ref(ModuleIdx::from_usize(0), 0),
            meta: ImportRecordMeta::empty(),
        }],
        default_a,
        ns_a,
    );
    module_a.side_effects = SideEffects::False;
    graph.add_normal_module(module_a);

    let config = LinkConfig::default();
    let se = determine_side_effects(&graph, &config);
    assert_eq!(se[&idx_a], true, "Pure module importing side-effectful dep is side-effectful");
}

#[test]
fn test_side_effects_custom_hooks() {
    use oxc_module_graph::LinkConfig;
    use oxc_module_graph::hooks::SideEffectsHooks;

    struct AlwaysSideEffects;
    impl SideEffectsHooks for AlwaysSideEffects {
        fn star_export_has_extra_side_effects(
            &self,
            _importer: ModuleIdx,
            _importee: ModuleIdx,
        ) -> bool {
            true
        }
    }

    let mut graph = ModuleGraph::new();
    graph.symbols.ensure_modules(2);

    let idx_a = ModuleIdx::from_usize(0);
    let idx_b = ModuleIdx::from_usize(1);

    let (default_a, ns_a) = make_simple_module(&mut graph, idx_a, "/a.js");
    let (default_b, ns_b) = make_simple_module(&mut graph, idx_b, "/b.js");

    let mut module_b = make_normal_module(
        idx_b,
        "/b.js",
        FxHashMap::default(),
        FxHashMap::default(),
        Vec::new(),
        default_b,
        ns_b,
    );
    module_b.side_effects = SideEffects::False;
    graph.add_normal_module(module_b);

    let mut module_a = make_normal_module(
        idx_a,
        "/a.js",
        FxHashMap::default(),
        FxHashMap::default(),
        Vec::new(),
        default_a,
        ns_a,
    );
    module_a.side_effects = SideEffects::False;
    module_a.star_export_entries = vec![StarExportEntry {
        module_request: CompactString::new("./b"),
        resolved_module: Some(idx_b),
        span: oxc_span::Span::new(0, 0),
    }];
    graph.add_normal_module(module_a);

    let hooks = AlwaysSideEffects;
    let config = LinkConfig { side_effects_hooks: Some(&hooks), ..Default::default() };
    let se = determine_side_effects(&graph, &config);
    assert_eq!(se[&idx_a], true, "Custom hook should make star-export side-effectful");
}

#[test]
fn test_side_effects_cycle_no_infinite_loop() {
    use oxc_module_graph::LinkConfig;

    let mut graph = ModuleGraph::new();
    graph.symbols.ensure_modules(2);

    let idx_a = ModuleIdx::from_usize(0);
    let idx_b = ModuleIdx::from_usize(1);

    let (default_a, ns_a) = make_simple_module(&mut graph, idx_a, "/a.js");
    let (default_b, ns_b) = make_simple_module(&mut graph, idx_b, "/b.js");

    let mut module_a = make_normal_module(
        idx_a,
        "/a.js",
        FxHashMap::default(),
        FxHashMap::default(),
        vec![ResolvedImportRecord {
            specifier: CompactString::new("./b"),
            resolved_module: Some(idx_b),
            kind: ImportKind::Static,
            namespace_ref: dummy_symbol_ref(ModuleIdx::from_usize(0), 0),
            meta: ImportRecordMeta::empty(),
        }],
        default_a,
        ns_a,
    );
    module_a.side_effects = SideEffects::False;
    graph.add_normal_module(module_a);

    let mut module_b = make_normal_module(
        idx_b,
        "/b.js",
        FxHashMap::default(),
        FxHashMap::default(),
        vec![ResolvedImportRecord {
            specifier: CompactString::new("./a"),
            resolved_module: Some(idx_a),
            kind: ImportKind::Static,
            namespace_ref: dummy_symbol_ref(ModuleIdx::from_usize(0), 0),
            meta: ImportRecordMeta::empty(),
        }],
        default_b,
        ns_b,
    );
    module_b.side_effects = SideEffects::False;
    graph.add_normal_module(module_b);

    let config = LinkConfig::default();
    let se = determine_side_effects(&graph, &config);
    // Cycle with both pure should not hang and should be pure
    assert_eq!(se[&idx_a], false);
    assert_eq!(se[&idx_b], false);
}

#[test]
fn test_side_effects_external_module_propagation() {
    use oxc_module_graph::{ExternalModule, LinkConfig};

    let mut graph = ModuleGraph::new();
    graph.symbols.ensure_modules(2);

    let idx_a = ModuleIdx::from_usize(0);
    let idx_ext = ModuleIdx::from_usize(1);

    let (default_a, ns_a) = make_simple_module(&mut graph, idx_a, "/a.js");
    let ns_ext = graph.add_symbol(idx_ext, "__namespace__".to_string());

    // External module with side effects
    graph.add_external_module(ExternalModule {
        idx: idx_ext,
        specifier: CompactString::new("ext-lib"),
        side_effects: SideEffects::True,
        namespace_ref: ns_ext,
        exec_order: u32::MAX,
    });

    // A is pure but imports the external
    let mut module_a = make_normal_module(
        idx_a,
        "/a.js",
        FxHashMap::default(),
        FxHashMap::default(),
        vec![ResolvedImportRecord {
            specifier: CompactString::new("ext-lib"),
            resolved_module: Some(idx_ext),
            kind: ImportKind::Static,
            namespace_ref: dummy_symbol_ref(ModuleIdx::from_usize(0), 0),
            meta: ImportRecordMeta::empty(),
        }],
        default_a,
        ns_a,
    );
    module_a.side_effects = SideEffects::False;
    graph.add_normal_module(module_a);

    let config = LinkConfig::default();
    let se = determine_side_effects(&graph, &config);
    assert_eq!(
        se[&idx_a], true,
        "Pure module importing external with side effects should be side-effectful"
    );
}

// --- Namespace import test ---

#[test]
fn test_namespace_import() {
    let mut graph = ModuleGraph::new();
    graph.symbols.ensure_modules(2);

    let idx_a = ModuleIdx::from_usize(0);
    let idx_b = ModuleIdx::from_usize(1);

    let sym_a_ns_import = graph.add_symbol(idx_a, "ns".to_string());
    let sym_a_default = graph.add_symbol(idx_a, "__default__".to_string());
    let sym_a_ns = graph.add_symbol(idx_a, "__namespace__".to_string());

    let sym_b_default = graph.add_symbol(idx_b, "__default__".to_string());
    let sym_b_ns = graph.add_symbol(idx_b, "__namespace__".to_string());

    let mut named_imports_a = FxHashMap::default();
    named_imports_a.insert(
        sym_a_ns_import,
        NamedImport {
            imported_name: CompactString::new("*"),
            local_symbol: sym_a_ns_import,
            record_idx: ImportRecordIdx::from_usize(0),
            is_type: false,
        },
    );

    graph.add_normal_module(make_normal_module(
        idx_a,
        "/a.js",
        FxHashMap::default(),
        named_imports_a,
        vec![ResolvedImportRecord {
            specifier: CompactString::new("./b"),
            resolved_module: Some(idx_b),
            kind: ImportKind::Static,
            namespace_ref: dummy_symbol_ref(ModuleIdx::from_usize(0), 0),
            meta: ImportRecordMeta::empty(),
        }],
        sym_a_default,
        sym_a_ns,
    ));
    graph.add_normal_module(make_normal_module(
        idx_b,
        "/b.js",
        FxHashMap::default(),
        FxHashMap::default(),
        Vec::new(),
        sym_b_default,
        sym_b_ns,
    ));

    bind_imports_and_exports(&mut graph);

    // A's ns import should link to B's namespace object ref
    let canonical = graph.canonical_ref(sym_a_ns_import);
    assert_eq!(canonical, sym_b_ns, "Namespace import should link to namespace object ref");
}

// ---------------------------------------------------------------------------
// Step-by-step linking workflow (Rolldown integration pattern)
// ---------------------------------------------------------------------------

/// Simulates Rolldown's step-by-step link pipeline:
///
/// 1. Create a `ModuleGraph` manually (no builder)
/// 2. Populate `NormalModule` + `ExternalModule` with all fields
/// 3. Call algorithms in Rolldown's order with interleaved consumer steps
/// 4. Assert results match expected values
#[test]
fn test_stepwise_link_workflow() {
    let mut graph = ModuleGraph::new();

    // Allocate module indices.
    let entry_idx = graph.alloc_module_idx(); // 0: entry.js
    let lib_idx = graph.alloc_module_idx(); // 1: lib.js
    let ext_idx = graph.alloc_module_idx(); // 2: external "react"

    // --- Add symbols ---
    // entry.js: import { foo } from './lib'; export default foo;
    let entry_foo = graph.add_symbol(entry_idx, "foo".into());
    let entry_default = graph.add_symbol(entry_idx, "default".into());
    let entry_ns = graph.add_symbol(entry_idx, "*ns*".into());

    // lib.js: export const foo = 42;
    let lib_foo = graph.add_symbol(lib_idx, "foo".into());
    let lib_default = graph.add_symbol(lib_idx, "default".into());
    let lib_ns = graph.add_symbol(lib_idx, "*ns*".into());

    // react (external): namespace symbol
    let ext_ns = graph.add_symbol(ext_idx, "react_ns".into());

    // --- Populate modules ---

    // entry.js: import { foo } from './lib'; export default foo;
    let mut entry_imports = FxHashMap::default();
    entry_imports.insert(
        entry_foo,
        NamedImport {
            imported_name: CompactString::new("foo"),
            local_symbol: entry_foo,
            record_idx: ImportRecordIdx::from_usize(0),
            is_type: false,
        },
    );
    let mut entry_exports = FxHashMap::default();
    entry_exports.insert(
        CompactString::new("default"),
        LocalExport { exported_name: CompactString::new("default"), local_symbol: entry_default },
    );

    graph.add_normal_module(NormalModule {
        idx: entry_idx,
        path: PathBuf::from("/entry.js"),
        has_module_syntax: true,
        exports_kind: ExportsKind::None,
        has_top_level_await: false,
        side_effects: SideEffects::True,
        has_lazy_export: false,
        execution_order_sensitive: false,
        named_exports: entry_exports,
        named_imports: entry_imports,
        import_records: vec![ResolvedImportRecord {
            specifier: CompactString::new("./lib"),
            resolved_module: Some(lib_idx),
            kind: ImportKind::Static,
            namespace_ref: dummy_symbol_ref(ModuleIdx::from_usize(0), 0),
            meta: ImportRecordMeta::empty(),
        }],
        star_export_entries: vec![],
        indirect_export_entries: vec![],
        default_export_ref: entry_default,
        namespace_object_ref: entry_ns,
        wrap_kind: WrapKind::None,
        original_wrap_kind: WrapKind::None,
        wrapper_ref: None,
        required_by_other_module: false,
        resolved_exports: FxHashMap::default(),
        has_dynamic_exports: false,
        is_tla_or_contains_tla: false,
        propagated_side_effects: false,
        exec_order: u32::MAX,
    });

    // lib.js: export const foo = 42;
    let mut lib_exports = FxHashMap::default();
    lib_exports.insert(
        CompactString::new("foo"),
        LocalExport { exported_name: CompactString::new("foo"), local_symbol: lib_foo },
    );

    graph.add_normal_module(NormalModule {
        idx: lib_idx,
        path: PathBuf::from("/lib.js"),
        has_module_syntax: true,
        exports_kind: ExportsKind::None,
        has_top_level_await: false,
        side_effects: SideEffects::False,
        has_lazy_export: false,
        execution_order_sensitive: false,
        named_exports: lib_exports,
        named_imports: FxHashMap::default(),
        import_records: vec![],
        star_export_entries: vec![],
        indirect_export_entries: vec![],
        default_export_ref: lib_default,
        namespace_object_ref: lib_ns,
        wrap_kind: WrapKind::None,
        original_wrap_kind: WrapKind::None,
        wrapper_ref: None,
        required_by_other_module: false,
        resolved_exports: FxHashMap::default(),
        has_dynamic_exports: false,
        is_tla_or_contains_tla: false,
        propagated_side_effects: false,
        exec_order: u32::MAX,
    });

    // react (external)
    graph.add_external_module(ExternalModule {
        idx: ext_idx,
        specifier: CompactString::new("react"),
        side_effects: SideEffects::True,
        namespace_ref: ext_ns,
        exec_order: u32::MAX,
    });

    graph.set_entries(vec![entry_idx]);

    // --- Step-by-step link pipeline ---

    // Step 1: Execution order
    let mut config = LinkConfig::default();
    let exec = compute_exec_order(&graph, &config);
    #[expect(clippy::cast_possible_truncation)]
    for (i, &idx) in exec.sorted.iter().enumerate() {
        match graph.module_mut(idx) {
            oxc_module_graph::Module::Normal(m) => m.exec_order = i as u32,
            oxc_module_graph::Module::External(m) => m.exec_order = i as u32,
        }
    }

    // Step 2: TLA propagation
    let tla = compute_tla(&graph);
    for &idx in &tla {
        if let Some(m) = graph.normal_module_mut(idx) {
            m.is_tla_or_contains_tla = true;
        }
    }

    // Step 3 (interleaved consumer step): mark CJS modules, etc.
    // (Rolldown would call determine_module_exports_kind here)
    // For this test, no CJS modules to mark.

    // Step 4: Dynamic exports
    let dynamic = compute_has_dynamic_exports(&graph);
    for &idx in &dynamic {
        if let Some(m) = graph.normal_module_mut(idx) {
            m.has_dynamic_exports = true;
        }
    }

    // Step 5: Resolved exports
    let resolved = build_resolved_exports(&graph);
    for (idx, exports) in resolved {
        if let Some(m) = graph.normal_module_mut(idx) {
            m.resolved_exports = exports;
        }
    }

    // Step 6: Match imports
    let (errors, links) = match_imports_collect(&graph, &mut config);

    // Consumer applies links to its own SymbolRefDb (or graph.symbols).
    for (from, to) in &links {
        graph.link_symbols(*from, *to);
    }

    // Step 7: Side effects
    let se = determine_side_effects(&graph, &config);
    for (idx, has) in se {
        if let Some(m) = graph.normal_module_mut(idx) {
            m.propagated_side_effects = has;
        }
    }

    // --- Assertions ---
    assert!(errors.is_empty(), "Expected no binding errors, got: {errors:?}");
    assert!(!links.is_empty(), "Expected at least one link pair");

    // entry's `foo` should resolve to lib's `foo`
    let canonical = graph.canonical_ref(entry_foo);
    assert_eq!(canonical, lib_foo, "entry:foo should resolve to lib:foo");

    // lib.js was marked SideEffects::False, so propagated_side_effects should be false
    let lib_module = graph.normal_module(lib_idx).unwrap();
    assert!(!lib_module.propagated_side_effects, "lib.js should not have propagated side effects");

    // Execution order should be set (not u32::MAX)
    let entry_module = graph.normal_module(entry_idx).unwrap();
    assert_ne!(entry_module.exec_order, u32::MAX, "entry exec_order should be set");
    assert_ne!(lib_module.exec_order, u32::MAX, "lib exec_order should be set");

    // lib should execute before entry (DFS post-order: leaf first)
    assert!(
        lib_module.exec_order < entry_module.exec_order,
        "lib ({}) should execute before entry ({})",
        lib_module.exec_order,
        entry_module.exec_order
    );
}

// ---------------------------------------------------------------------------
// ImportHooks integration
// ---------------------------------------------------------------------------

struct TestImportHooks {
    resolved_count: usize,
    no_match_count: usize,
}

impl ImportHooks for TestImportHooks {
    fn on_resolved(
        &mut self,
        _importer: ModuleIdx,
        _local_symbol: SymbolRef,
        _result: &MatchImportKind,
        _reexport_chain: &[SymbolRef],
    ) {
        self.resolved_count += 1;
    }

    fn on_final_no_match(
        &mut self,
        _target: ModuleIdx,
        _import_name: &str,
    ) -> Option<MatchImportKind> {
        self.no_match_count += 1;
        None // Let the error propagate
    }
}

/// Verifies that `ImportHooks::on_resolved` is called for each import resolution
/// and `on_final_no_match` is called for unresolved imports.
#[test]
fn test_import_hooks_called() {
    let mut graph = ModuleGraph::new();

    let idx_a = graph.alloc_module_idx();
    let idx_b = graph.alloc_module_idx();

    // Symbols for A
    let sym_a_foo = graph.add_symbol(idx_a, "foo".into());
    let sym_a_missing = graph.add_symbol(idx_a, "missing".into());
    let sym_a_default = graph.add_symbol(idx_a, "default".into());
    let sym_a_ns = graph.add_symbol(idx_a, "*ns*".into());

    // Symbols for B
    let sym_b_foo = graph.add_symbol(idx_b, "foo".into());
    let sym_b_default = graph.add_symbol(idx_b, "default".into());
    let sym_b_ns = graph.add_symbol(idx_b, "*ns*".into());

    // A: import { foo, missing } from './b'
    let mut named_imports_a = FxHashMap::default();
    named_imports_a.insert(
        sym_a_foo,
        NamedImport {
            imported_name: CompactString::new("foo"),
            local_symbol: sym_a_foo,
            record_idx: ImportRecordIdx::from_usize(0),
            is_type: false,
        },
    );
    named_imports_a.insert(
        sym_a_missing,
        NamedImport {
            imported_name: CompactString::new("missing"),
            local_symbol: sym_a_missing,
            record_idx: ImportRecordIdx::from_usize(0),
            is_type: false,
        },
    );

    graph.add_normal_module(make_normal_module(
        idx_a,
        "/a.js",
        FxHashMap::default(),
        named_imports_a,
        vec![ResolvedImportRecord {
            specifier: CompactString::new("./b"),
            resolved_module: Some(idx_b),
            kind: ImportKind::Static,
            namespace_ref: dummy_symbol_ref(ModuleIdx::from_usize(0), 0),
            meta: ImportRecordMeta::empty(),
        }],
        sym_a_default,
        sym_a_ns,
    ));

    // B: export const foo = 42; (no "missing" export)
    let mut named_exports_b = FxHashMap::default();
    named_exports_b.insert(
        CompactString::new("foo"),
        LocalExport { exported_name: CompactString::new("foo"), local_symbol: sym_b_foo },
    );
    graph.add_normal_module(make_normal_module(
        idx_b,
        "/b.js",
        named_exports_b,
        FxHashMap::default(),
        vec![],
        sym_b_default,
        sym_b_ns,
    ));

    // Build resolved exports first.
    let resolved = build_resolved_exports(&graph);
    for (idx, exports) in resolved {
        if let Some(m) = graph.normal_module_mut(idx) {
            m.resolved_exports = exports;
        }
    }

    // Run match_imports_collect with hooks.
    let mut hooks = TestImportHooks { resolved_count: 0, no_match_count: 0 };
    let (errors, links) = {
        let mut config = LinkConfig { import_hooks: Some(&mut hooks), ..Default::default() };
        match_imports_collect(&graph, &mut config)
    };
    // Config is dropped, so we can read hooks directly again.

    // on_resolved should be called for both imports (foo: success, missing: NoMatch).
    assert_eq!(hooks.resolved_count, 2, "on_resolved should be called for each import");
    // on_final_no_match should be called for the "missing" import.
    assert_eq!(hooks.no_match_count, 1, "on_final_no_match should be called for unresolved import");

    // One error (unresolved "missing"), one link (foo).
    assert_eq!(errors.len(), 1, "Expected 1 unresolved import error");
    assert!(!links.is_empty(), "Expected at least one link for 'foo'");
}

#[test]
fn test_build_resolved_exports_marks_indirect_reexports_from_commonjs() {
    let mut graph = ModuleGraph::new();

    let idx_a = graph.alloc_module_idx();
    let idx_b = graph.alloc_module_idx();

    let sym_a_default = graph.add_symbol(idx_a, "default".into());
    let sym_a_ns = graph.add_symbol(idx_a, "*ns*".into());

    let sym_b_bar = graph.add_symbol(idx_b, "bar".into());
    let sym_b_default = graph.add_symbol(idx_b, "default".into());
    let sym_b_ns = graph.add_symbol(idx_b, "*ns*".into());

    graph.add_normal_module(make_normal_module(
        idx_a,
        "/a.js",
        FxHashMap::default(),
        FxHashMap::default(),
        vec![ResolvedImportRecord {
            specifier: CompactString::new("./b"),
            resolved_module: Some(idx_b),
            kind: ImportKind::Static,
            namespace_ref: sym_b_ns,
            meta: ImportRecordMeta::empty(),
        }],
        sym_a_default,
        sym_a_ns,
    ));
    graph.normal_module_mut(idx_a).unwrap().indirect_export_entries = vec![IndirectExportEntry {
        exported_name: CompactString::new("bar"),
        imported_name: CompactString::new("bar"),
        module_request: CompactString::new("./b"),
        resolved_module: Some(idx_b),
        span: Default::default(),
    }];

    graph.add_normal_module(make_normal_module(
        idx_b,
        "/b.js",
        FxHashMap::from_iter([(
            CompactString::new("bar"),
            LocalExport { exported_name: CompactString::new("bar"), local_symbol: sym_b_bar },
        )]),
        FxHashMap::default(),
        vec![],
        sym_b_default,
        sym_b_ns,
    ));
    graph.normal_module_mut(idx_b).unwrap().exports_kind = ExportsKind::CommonJs;

    let resolved = build_resolved_exports(&graph);
    let export = resolved.get(&idx_a).and_then(|exports| exports.get("bar")).unwrap();

    assert!(export.came_from_cjs);
    assert_eq!(export.symbol_ref, sym_b_bar);
}

/// Verifies that `on_final_no_match` can override the error by returning a result.
#[test]
fn test_import_hooks_override_no_match() {
    let mut graph = ModuleGraph::new();

    let idx_a = graph.alloc_module_idx();
    let idx_b = graph.alloc_module_idx();

    // Symbols
    let sym_a_missing = graph.add_symbol(idx_a, "missing".into());
    let sym_a_default = graph.add_symbol(idx_a, "default".into());
    let sym_a_ns = graph.add_symbol(idx_a, "*ns*".into());

    let sym_b_default = graph.add_symbol(idx_b, "default".into());
    let sym_b_ns = graph.add_symbol(idx_b, "*ns*".into());
    // A fallback symbol the hook will return.
    let sym_b_fallback = graph.add_symbol(idx_b, "fallback".into());

    // A: import { missing } from './b'
    let mut named_imports_a = FxHashMap::default();
    named_imports_a.insert(
        sym_a_missing,
        NamedImport {
            imported_name: CompactString::new("missing"),
            local_symbol: sym_a_missing,
            record_idx: ImportRecordIdx::from_usize(0),
            is_type: false,
        },
    );

    graph.add_normal_module(make_normal_module(
        idx_a,
        "/a.js",
        FxHashMap::default(),
        named_imports_a,
        vec![ResolvedImportRecord {
            specifier: CompactString::new("./b"),
            resolved_module: Some(idx_b),
            kind: ImportKind::Static,
            namespace_ref: dummy_symbol_ref(ModuleIdx::from_usize(0), 0),
            meta: ImportRecordMeta::empty(),
        }],
        sym_a_default,
        sym_a_ns,
    ));

    // B: no exports (so "missing" won't be found)
    graph.add_normal_module(make_normal_module(
        idx_b,
        "/b.js",
        FxHashMap::default(),
        FxHashMap::default(),
        vec![],
        sym_b_default,
        sym_b_ns,
    ));

    // Build resolved exports.
    let resolved = build_resolved_exports(&graph);
    for (idx, exports) in resolved {
        if let Some(m) = graph.normal_module_mut(idx) {
            m.resolved_exports = exports;
        }
    }

    // Hook that overrides the no-match with a fallback symbol.
    struct OverrideHooks {
        fallback: SymbolRef,
    }
    impl ImportHooks for OverrideHooks {
        fn on_final_no_match(
            &mut self,
            _target: ModuleIdx,
            _import_name: &str,
        ) -> Option<MatchImportKind> {
            Some(MatchImportKind::Normal { symbol_ref: self.fallback })
        }
    }

    let mut hooks = OverrideHooks { fallback: sym_b_fallback };
    let mut config = LinkConfig { import_hooks: Some(&mut hooks), ..Default::default() };

    let (errors, links) = match_imports_collect(&graph, &mut config);

    // No errors because the hook overrode the no-match.
    assert!(errors.is_empty(), "Expected no errors when hook overrides no-match, got: {errors:?}");
    // The link should map sym_a_missing → sym_b_fallback.
    assert_eq!(links.len(), 1);
    assert_eq!(links[0], (sym_a_missing, sym_b_fallback));
}

// --- determine_module_exports_kind tests ---

/// Helper: build a graph where A imports B with a specific ImportKind,
/// and B has a given initial ExportsKind.
fn exports_kind_graph(import_kind: ImportKind, importee_exports_kind: ExportsKind) -> ModuleGraph {
    let mut graph = ModuleGraph::new();
    graph.symbols.ensure_modules(2);

    let idx_a = ModuleIdx::from_usize(0);
    let idx_b = ModuleIdx::from_usize(1);

    let (default_a, ns_a) = make_simple_module(&mut graph, idx_a, "/a.js");
    let (default_b, ns_b) = make_simple_module(&mut graph, idx_b, "/b.js");

    let ns_rec = graph.add_symbol(idx_a, "__import_ns__".to_string());

    graph.add_normal_module(make_normal_module(
        idx_a,
        "/a.js",
        FxHashMap::default(),
        FxHashMap::default(),
        vec![ResolvedImportRecord {
            specifier: CompactString::new("./b"),
            resolved_module: Some(idx_b),
            kind: import_kind,
            namespace_ref: ns_rec,
            meta: ImportRecordMeta::empty(),
        }],
        default_a,
        ns_a,
    ));

    let mut module_b = make_normal_module(
        idx_b,
        "/b.js",
        FxHashMap::default(),
        FxHashMap::default(),
        Vec::new(),
        default_b,
        ns_b,
    );
    module_b.exports_kind = importee_exports_kind;
    graph.add_normal_module(module_b);

    graph
}

#[test]
fn test_exports_kind_static_import_none_becomes_esm() {
    let graph = exports_kind_graph(ImportKind::Static, ExportsKind::None);
    let config = ExportsKindConfig { dynamic_imports_as_require: false, wrap_cjs_entries: false };
    let result = determine_module_exports_kind(&graph, &config);

    let idx_b = ModuleIdx::from_usize(1);
    assert_eq!(
        result.exports_kind_updates.get(&idx_b).copied(),
        Some(ExportsKind::Esm),
        "Static import of None module should set it to Esm"
    );
}

#[test]
fn test_exports_kind_static_import_cjs_unchanged() {
    let graph = exports_kind_graph(ImportKind::Static, ExportsKind::CommonJs);
    let config = ExportsKindConfig { dynamic_imports_as_require: false, wrap_cjs_entries: false };
    let result = determine_module_exports_kind(&graph, &config);

    let idx_b = ModuleIdx::from_usize(1);
    assert!(
        !result.exports_kind_updates.contains_key(&idx_b),
        "Static import of CJS module should not change exports_kind"
    );
}

#[test]
fn test_exports_kind_require_esm_gets_esm_wrap() {
    let graph = exports_kind_graph(ImportKind::Require, ExportsKind::Esm);
    let config = ExportsKindConfig { dynamic_imports_as_require: false, wrap_cjs_entries: false };
    let result = determine_module_exports_kind(&graph, &config);

    let idx_b = ModuleIdx::from_usize(1);
    assert_eq!(
        result.wrap_kind_updates.get(&idx_b).copied(),
        Some(WrapKind::Esm),
        "Require of ESM module should get WrapKind::Esm"
    );
}

#[test]
fn test_exports_kind_require_cjs_gets_cjs_wrap() {
    let graph = exports_kind_graph(ImportKind::Require, ExportsKind::CommonJs);
    let config = ExportsKindConfig { dynamic_imports_as_require: false, wrap_cjs_entries: false };
    let result = determine_module_exports_kind(&graph, &config);

    let idx_b = ModuleIdx::from_usize(1);
    assert_eq!(
        result.wrap_kind_updates.get(&idx_b).copied(),
        Some(WrapKind::Cjs),
        "Require of CJS module should get WrapKind::Cjs"
    );
}

#[test]
fn test_exports_kind_require_none_becomes_cjs() {
    let graph = exports_kind_graph(ImportKind::Require, ExportsKind::None);
    let config = ExportsKindConfig { dynamic_imports_as_require: false, wrap_cjs_entries: false };
    let result = determine_module_exports_kind(&graph, &config);

    let idx_b = ModuleIdx::from_usize(1);
    assert_eq!(
        result.exports_kind_updates.get(&idx_b).copied(),
        Some(ExportsKind::CommonJs),
        "Require of None module should set exports_kind to CommonJs"
    );
    assert_eq!(
        result.wrap_kind_updates.get(&idx_b).copied(),
        Some(WrapKind::Cjs),
        "Require of None module should also set WrapKind::Cjs"
    );
}

#[test]
fn test_exports_kind_dynamic_as_require() {
    let graph = exports_kind_graph(ImportKind::Dynamic, ExportsKind::None);
    let config = ExportsKindConfig { dynamic_imports_as_require: true, wrap_cjs_entries: false };
    let result = determine_module_exports_kind(&graph, &config);

    let idx_b = ModuleIdx::from_usize(1);
    assert_eq!(
        result.exports_kind_updates.get(&idx_b).copied(),
        Some(ExportsKind::CommonJs),
        "Dynamic import with dynamic_imports_as_require should act like require"
    );
    assert_eq!(result.wrap_kind_updates.get(&idx_b).copied(), Some(WrapKind::Cjs),);
}

#[test]
fn test_exports_kind_dynamic_default_no_change() {
    let graph = exports_kind_graph(ImportKind::Dynamic, ExportsKind::None);
    let config = ExportsKindConfig { dynamic_imports_as_require: false, wrap_cjs_entries: false };
    let result = determine_module_exports_kind(&graph, &config);

    let idx_b = ModuleIdx::from_usize(1);
    assert!(
        !result.exports_kind_updates.contains_key(&idx_b),
        "Dynamic import (default) should not change exports_kind"
    );
    assert!(
        !result.wrap_kind_updates.contains_key(&idx_b),
        "Dynamic import (default) should not set wrap_kind"
    );
}

#[test]
fn test_exports_kind_cjs_non_entry_wrapped() {
    // B is CJS, not an entry point → should be wrapped.
    let mut graph = ModuleGraph::new();
    graph.symbols.ensure_modules(1);

    let idx_b = ModuleIdx::from_usize(0);
    let (default_b, ns_b) = make_simple_module(&mut graph, idx_b, "/b.js");

    let mut module_b = make_normal_module(
        idx_b,
        "/b.js",
        FxHashMap::default(),
        FxHashMap::default(),
        Vec::new(),
        default_b,
        ns_b,
    );
    module_b.exports_kind = ExportsKind::CommonJs;
    graph.add_normal_module(module_b);
    // No entries set → B is not an entry.

    let config = ExportsKindConfig { dynamic_imports_as_require: false, wrap_cjs_entries: false };
    let result = determine_module_exports_kind(&graph, &config);

    assert_eq!(
        result.wrap_kind_updates.get(&idx_b).copied(),
        Some(WrapKind::Cjs),
        "CJS non-entry module should be wrapped"
    );
}

#[test]
fn test_exports_kind_cjs_entry_not_wrapped() {
    // B is CJS and an entry point, wrap_cjs_entries = false → NOT wrapped.
    let mut graph = ModuleGraph::new();
    graph.symbols.ensure_modules(1);

    let idx_b = ModuleIdx::from_usize(0);
    let (default_b, ns_b) = make_simple_module(&mut graph, idx_b, "/b.js");

    let mut module_b = make_normal_module(
        idx_b,
        "/b.js",
        FxHashMap::default(),
        FxHashMap::default(),
        Vec::new(),
        default_b,
        ns_b,
    );
    module_b.exports_kind = ExportsKind::CommonJs;
    graph.add_normal_module(module_b);
    graph.set_entries(vec![idx_b]);

    let config = ExportsKindConfig { dynamic_imports_as_require: false, wrap_cjs_entries: false };
    let result = determine_module_exports_kind(&graph, &config);

    assert!(
        !result.wrap_kind_updates.contains_key(&idx_b),
        "CJS entry with wrap_cjs_entries=false should NOT be wrapped"
    );
}

#[test]
fn test_exports_kind_cjs_entry_wrapped() {
    // B is CJS and an entry point, wrap_cjs_entries = true → wrapped.
    let mut graph = ModuleGraph::new();
    graph.symbols.ensure_modules(1);

    let idx_b = ModuleIdx::from_usize(0);
    let (default_b, ns_b) = make_simple_module(&mut graph, idx_b, "/b.js");

    let mut module_b = make_normal_module(
        idx_b,
        "/b.js",
        FxHashMap::default(),
        FxHashMap::default(),
        Vec::new(),
        default_b,
        ns_b,
    );
    module_b.exports_kind = ExportsKind::CommonJs;
    graph.add_normal_module(module_b);
    graph.set_entries(vec![idx_b]);

    let config = ExportsKindConfig { dynamic_imports_as_require: false, wrap_cjs_entries: true };
    let result = determine_module_exports_kind(&graph, &config);

    assert_eq!(
        result.wrap_kind_updates.get(&idx_b).copied(),
        Some(WrapKind::Cjs),
        "CJS entry with wrap_cjs_entries=true should be wrapped"
    );
}

#[test]
fn test_exports_kind_mixed_graph() {
    // A --(static)--> B, B --(require)--> C (all start as None)
    let mut graph = ModuleGraph::new();
    graph.symbols.ensure_modules(3);

    let idx_a = ModuleIdx::from_usize(0);
    let idx_b = ModuleIdx::from_usize(1);
    let idx_c = ModuleIdx::from_usize(2);

    let (default_a, ns_a) = make_simple_module(&mut graph, idx_a, "/a.js");
    let (default_b, ns_b) = make_simple_module(&mut graph, idx_b, "/b.js");
    let (default_c, ns_c) = make_simple_module(&mut graph, idx_c, "/c.js");

    let ns_rec_a = graph.add_symbol(idx_a, "__import_ns_a__".to_string());
    let ns_rec_b = graph.add_symbol(idx_b, "__import_ns_b__".to_string());

    // A imports B (static)
    graph.add_normal_module(make_normal_module(
        idx_a,
        "/a.js",
        FxHashMap::default(),
        FxHashMap::default(),
        vec![ResolvedImportRecord {
            specifier: CompactString::new("./b"),
            resolved_module: Some(idx_b),
            kind: ImportKind::Static,
            namespace_ref: ns_rec_a,
            meta: ImportRecordMeta::empty(),
        }],
        default_a,
        ns_a,
    ));

    // B requires C
    graph.add_normal_module(make_normal_module(
        idx_b,
        "/b.js",
        FxHashMap::default(),
        FxHashMap::default(),
        vec![ResolvedImportRecord {
            specifier: CompactString::new("./c"),
            resolved_module: Some(idx_c),
            kind: ImportKind::Require,
            namespace_ref: ns_rec_b,
            meta: ImportRecordMeta::empty(),
        }],
        default_b,
        ns_b,
    ));

    // C has no imports
    graph.add_normal_module(make_normal_module(
        idx_c,
        "/c.js",
        FxHashMap::default(),
        FxHashMap::default(),
        Vec::new(),
        default_c,
        ns_c,
    ));

    let config = ExportsKindConfig { dynamic_imports_as_require: false, wrap_cjs_entries: true };
    let result = determine_module_exports_kind(&graph, &config);

    // B should become ESM (static import from A)
    assert_eq!(
        result.exports_kind_updates.get(&idx_b).copied(),
        Some(ExportsKind::Esm),
        "B should become ESM from static import"
    );
    // C should become CJS (require from B) and be wrapped
    assert_eq!(
        result.exports_kind_updates.get(&idx_c).copied(),
        Some(ExportsKind::CommonJs),
        "C should become CJS from require"
    );
    assert_eq!(
        result.wrap_kind_updates.get(&idx_c).copied(),
        Some(WrapKind::Cjs),
        "C should be wrapped"
    );
}

// --- wrap_modules tests ---

#[test]
fn test_wrap_propagation_basic() {
    // A requires B(ESM) → C(ESM, static import from B)
    let mut graph = ModuleGraph::new();
    graph.symbols.ensure_modules(3);

    let idx_a = ModuleIdx::from_usize(0);
    let idx_b = ModuleIdx::from_usize(1);
    let idx_c = ModuleIdx::from_usize(2);

    let (default_a, ns_a) = make_simple_module(&mut graph, idx_a, "/a.js");
    let (default_b, ns_b) = make_simple_module(&mut graph, idx_b, "/b.js");
    let (default_c, ns_c) = make_simple_module(&mut graph, idx_c, "/c.js");

    let ns_rec_a = graph.add_symbol(idx_a, "__ns__".to_string());
    let ns_rec_b = graph.add_symbol(idx_b, "__ns__".to_string());

    graph.add_normal_module(make_normal_module(
        idx_a,
        "/a.js",
        FxHashMap::default(),
        FxHashMap::default(),
        vec![ResolvedImportRecord {
            specifier: CompactString::new("./b"),
            resolved_module: Some(idx_b),
            kind: ImportKind::Require,
            namespace_ref: ns_rec_a,
            meta: ImportRecordMeta::empty(),
        }],
        default_a,
        ns_a,
    ));

    let mut module_b = make_normal_module(
        idx_b,
        "/b.js",
        FxHashMap::default(),
        FxHashMap::default(),
        vec![ResolvedImportRecord {
            specifier: CompactString::new("./c"),
            resolved_module: Some(idx_c),
            kind: ImportKind::Static,
            namespace_ref: ns_rec_b,
            meta: ImportRecordMeta::empty(),
        }],
        default_b,
        ns_b,
    );
    module_b.exports_kind = ExportsKind::Esm;
    module_b.wrap_kind = WrapKind::Esm; // Set by determine_module_exports_kind
    graph.add_normal_module(module_b);

    let mut module_c = make_normal_module(
        idx_c,
        "/c.js",
        FxHashMap::default(),
        FxHashMap::default(),
        Vec::new(),
        default_c,
        ns_c,
    );
    module_c.exports_kind = ExportsKind::Esm;
    graph.add_normal_module(module_c);

    let config = WrapModulesConfig {
        on_demand_wrapping: false,
        strict_execution_order: false,
        skip_symbol_creation: false,
    };
    let result = wrap_modules(&mut graph, &config);

    // C should be wrapped ESM because B is wrapped and C is B's dependency.
    assert_eq!(
        result.wrap_kind_updates.get(&idx_c).copied(),
        Some(WrapKind::Esm),
        "C should be wrapped ESM via propagation from B"
    );
}

#[test]
fn test_wrap_cjs_dep_always_wrapped() {
    // A(ESM) imports B(ESM) imports C(CJS) — C should be wrapped even though A is not.
    let mut graph = ModuleGraph::new();
    graph.symbols.ensure_modules(3);

    let idx_a = ModuleIdx::from_usize(0);
    let idx_b = ModuleIdx::from_usize(1);
    let idx_c = ModuleIdx::from_usize(2);

    let (default_a, ns_a) = make_simple_module(&mut graph, idx_a, "/a.js");
    let (default_b, ns_b) = make_simple_module(&mut graph, idx_b, "/b.js");
    let (default_c, ns_c) = make_simple_module(&mut graph, idx_c, "/c.js");

    let ns_rec_a = graph.add_symbol(idx_a, "__ns__".to_string());
    let ns_rec_b = graph.add_symbol(idx_b, "__ns__".to_string());

    let mut module_a = make_normal_module(
        idx_a,
        "/a.js",
        FxHashMap::default(),
        FxHashMap::default(),
        vec![ResolvedImportRecord {
            specifier: CompactString::new("./b"),
            resolved_module: Some(idx_b),
            kind: ImportKind::Static,
            namespace_ref: ns_rec_a,
            meta: ImportRecordMeta::empty(),
        }],
        default_a,
        ns_a,
    );
    module_a.exports_kind = ExportsKind::Esm;
    graph.add_normal_module(module_a);

    let mut module_b = make_normal_module(
        idx_b,
        "/b.js",
        FxHashMap::default(),
        FxHashMap::default(),
        vec![ResolvedImportRecord {
            specifier: CompactString::new("./c"),
            resolved_module: Some(idx_c),
            kind: ImportKind::Static,
            namespace_ref: ns_rec_b,
            meta: ImportRecordMeta::empty(),
        }],
        default_b,
        ns_b,
    );
    module_b.exports_kind = ExportsKind::Esm;
    graph.add_normal_module(module_b);

    let mut module_c = make_normal_module(
        idx_c,
        "/c.js",
        FxHashMap::default(),
        FxHashMap::default(),
        Vec::new(),
        default_c,
        ns_c,
    );
    module_c.exports_kind = ExportsKind::CommonJs;
    graph.add_normal_module(module_c);

    let config = WrapModulesConfig {
        on_demand_wrapping: false,
        strict_execution_order: false,
        skip_symbol_creation: false,
    };
    let result = wrap_modules(&mut graph, &config);

    assert_eq!(
        result.wrap_kind_updates.get(&idx_c).copied(),
        Some(WrapKind::Cjs),
        "CJS dep should always be wrapped"
    );
    assert!(
        !result.wrap_kind_updates.contains_key(&idx_a),
        "A (non-wrapped ESM) should not gain wrapping"
    );
}

#[test]
fn test_wrap_deep_chain() {
    // A requires B(ESM) → C(ESM) → D(ESM) — all should be wrapped.
    let mut graph = ModuleGraph::new();
    graph.symbols.ensure_modules(4);

    let idx_a = ModuleIdx::from_usize(0);
    let idx_b = ModuleIdx::from_usize(1);
    let idx_c = ModuleIdx::from_usize(2);
    let idx_d = ModuleIdx::from_usize(3);

    let (default_a, ns_a) = make_simple_module(&mut graph, idx_a, "/a.js");
    let (default_b, ns_b) = make_simple_module(&mut graph, idx_b, "/b.js");
    let (default_c, ns_c) = make_simple_module(&mut graph, idx_c, "/c.js");
    let (default_d, ns_d) = make_simple_module(&mut graph, idx_d, "/d.js");

    let ns_a_rec = graph.add_symbol(idx_a, "__ns__".to_string());
    let ns_b_rec = graph.add_symbol(idx_b, "__ns__".to_string());
    let ns_c_rec = graph.add_symbol(idx_c, "__ns__".to_string());

    graph.add_normal_module(make_normal_module(
        idx_a,
        "/a.js",
        FxHashMap::default(),
        FxHashMap::default(),
        vec![ResolvedImportRecord {
            specifier: CompactString::new("./b"),
            resolved_module: Some(idx_b),
            kind: ImportKind::Require,
            namespace_ref: ns_a_rec,
            meta: ImportRecordMeta::empty(),
        }],
        default_a,
        ns_a,
    ));

    let mut module_b = make_normal_module(
        idx_b,
        "/b.js",
        FxHashMap::default(),
        FxHashMap::default(),
        vec![ResolvedImportRecord {
            specifier: CompactString::new("./c"),
            resolved_module: Some(idx_c),
            kind: ImportKind::Static,
            namespace_ref: ns_b_rec,
            meta: ImportRecordMeta::empty(),
        }],
        default_b,
        ns_b,
    );
    module_b.exports_kind = ExportsKind::Esm;
    module_b.wrap_kind = WrapKind::Esm;
    graph.add_normal_module(module_b);

    let mut module_c = make_normal_module(
        idx_c,
        "/c.js",
        FxHashMap::default(),
        FxHashMap::default(),
        vec![ResolvedImportRecord {
            specifier: CompactString::new("./d"),
            resolved_module: Some(idx_d),
            kind: ImportKind::Static,
            namespace_ref: ns_c_rec,
            meta: ImportRecordMeta::empty(),
        }],
        default_c,
        ns_c,
    );
    module_c.exports_kind = ExportsKind::Esm;
    graph.add_normal_module(module_c);

    let mut module_d = make_normal_module(
        idx_d,
        "/d.js",
        FxHashMap::default(),
        FxHashMap::default(),
        Vec::new(),
        default_d,
        ns_d,
    );
    module_d.exports_kind = ExportsKind::Esm;
    graph.add_normal_module(module_d);

    let config = WrapModulesConfig {
        on_demand_wrapping: false,
        strict_execution_order: false,
        skip_symbol_creation: false,
    };
    let result = wrap_modules(&mut graph, &config);

    assert_eq!(result.wrap_kind_updates.get(&idx_c).copied(), Some(WrapKind::Esm));
    assert_eq!(result.wrap_kind_updates.get(&idx_d).copied(), Some(WrapKind::Esm));
}

#[test]
fn test_wrap_creates_wrapper_symbols() {
    // CJS module and ESM module both wrapped → correct wrapper_ref names.
    let mut graph = ModuleGraph::new();
    graph.symbols.ensure_modules(2);

    let idx_cjs = ModuleIdx::from_usize(0);
    let idx_esm = ModuleIdx::from_usize(1);

    let (default_cjs, ns_cjs) = make_simple_module(&mut graph, idx_cjs, "/cjs_mod.js");
    let (default_esm, ns_esm) = make_simple_module(&mut graph, idx_esm, "/esm_mod.js");

    let mut module_cjs = make_normal_module(
        idx_cjs,
        "/cjs_mod.js",
        FxHashMap::default(),
        FxHashMap::default(),
        Vec::new(),
        default_cjs,
        ns_cjs,
    );
    module_cjs.exports_kind = ExportsKind::CommonJs;
    module_cjs.wrap_kind = WrapKind::Cjs;
    graph.add_normal_module(module_cjs);

    let mut module_esm = make_normal_module(
        idx_esm,
        "/esm_mod.js",
        FxHashMap::default(),
        FxHashMap::default(),
        Vec::new(),
        default_esm,
        ns_esm,
    );
    module_esm.exports_kind = ExportsKind::Esm;
    module_esm.wrap_kind = WrapKind::Esm;
    graph.add_normal_module(module_esm);

    let config = WrapModulesConfig {
        on_demand_wrapping: false,
        strict_execution_order: false,
        skip_symbol_creation: false,
    };
    let result = wrap_modules(&mut graph, &config);

    let cjs_wrapper = result.wrapper_refs.get(&idx_cjs).expect("CJS module should have wrapper");
    let esm_wrapper = result.wrapper_refs.get(&idx_esm).expect("ESM module should have wrapper");

    assert_eq!(graph.symbol_name(*cjs_wrapper), "require_cjs_mod");
    assert_eq!(graph.symbol_name(*esm_wrapper), "init_esm_mod");
}

#[test]
fn test_wrap_no_propagation_without_cjs() {
    // A(ESM) → B(ESM) → C(ESM), no require — nothing should be wrapped.
    let mut graph = ModuleGraph::new();
    graph.symbols.ensure_modules(3);

    let idx_a = ModuleIdx::from_usize(0);
    let idx_b = ModuleIdx::from_usize(1);
    let idx_c = ModuleIdx::from_usize(2);

    let (default_a, ns_a) = make_simple_module(&mut graph, idx_a, "/a.js");
    let (default_b, ns_b) = make_simple_module(&mut graph, idx_b, "/b.js");
    let (default_c, ns_c) = make_simple_module(&mut graph, idx_c, "/c.js");

    let ns_rec_a = graph.add_symbol(idx_a, "__ns__".to_string());
    let ns_rec_b = graph.add_symbol(idx_b, "__ns__".to_string());

    let mut module_a = make_normal_module(
        idx_a,
        "/a.js",
        FxHashMap::default(),
        FxHashMap::default(),
        vec![ResolvedImportRecord {
            specifier: CompactString::new("./b"),
            resolved_module: Some(idx_b),
            kind: ImportKind::Static,
            namespace_ref: ns_rec_a,
            meta: ImportRecordMeta::empty(),
        }],
        default_a,
        ns_a,
    );
    module_a.exports_kind = ExportsKind::Esm;
    graph.add_normal_module(module_a);

    let mut module_b = make_normal_module(
        idx_b,
        "/b.js",
        FxHashMap::default(),
        FxHashMap::default(),
        vec![ResolvedImportRecord {
            specifier: CompactString::new("./c"),
            resolved_module: Some(idx_c),
            kind: ImportKind::Static,
            namespace_ref: ns_rec_b,
            meta: ImportRecordMeta::empty(),
        }],
        default_b,
        ns_b,
    );
    module_b.exports_kind = ExportsKind::Esm;
    graph.add_normal_module(module_b);

    let mut module_c = make_normal_module(
        idx_c,
        "/c.js",
        FxHashMap::default(),
        FxHashMap::default(),
        Vec::new(),
        default_c,
        ns_c,
    );
    module_c.exports_kind = ExportsKind::Esm;
    graph.add_normal_module(module_c);

    let config = WrapModulesConfig {
        on_demand_wrapping: false,
        strict_execution_order: false,
        skip_symbol_creation: false,
    };
    let result = wrap_modules(&mut graph, &config);

    assert!(result.wrap_kind_updates.is_empty(), "Pure ESM graph should have no wrapping");
    assert!(result.wrapper_refs.is_empty(), "No wrapper symbols should be created");
}

#[test]
fn test_wrap_skips_external() {
    // A requires B (external) — B should not appear in wrap results.
    let mut graph = ModuleGraph::new();
    graph.symbols.ensure_modules(2);

    let idx_a = ModuleIdx::from_usize(0);
    let idx_ext = ModuleIdx::from_usize(1);

    let (default_a, ns_a) = make_simple_module(&mut graph, idx_a, "/a.js");
    let ns_ext = graph.add_symbol(idx_ext, "__namespace__".to_string());

    let ns_rec = graph.add_symbol(idx_a, "__ns__".to_string());

    graph.add_normal_module(make_normal_module(
        idx_a,
        "/a.js",
        FxHashMap::default(),
        FxHashMap::default(),
        vec![ResolvedImportRecord {
            specifier: CompactString::new("external"),
            resolved_module: Some(idx_ext),
            kind: ImportKind::Require,
            namespace_ref: ns_rec,
            meta: ImportRecordMeta::empty(),
        }],
        default_a,
        ns_a,
    ));

    graph.add_external_module(ExternalModule {
        idx: idx_ext,
        specifier: CompactString::new("external"),
        side_effects: SideEffects::True,
        namespace_ref: ns_ext,
        exec_order: u32::MAX,
    });

    let config = WrapModulesConfig {
        on_demand_wrapping: false,
        strict_execution_order: false,
        skip_symbol_creation: false,
    };
    let result = wrap_modules(&mut graph, &config);

    assert!(
        !result.wrap_kind_updates.contains_key(&idx_ext),
        "External module should not appear in wrap results"
    );
}

#[test]
fn test_wrap_cyclic_deps() {
    // A requires B, B imports A → should terminate, both wrapped.
    let mut graph = ModuleGraph::new();
    graph.symbols.ensure_modules(2);

    let idx_a = ModuleIdx::from_usize(0);
    let idx_b = ModuleIdx::from_usize(1);

    let (default_a, ns_a) = make_simple_module(&mut graph, idx_a, "/a.js");
    let (default_b, ns_b) = make_simple_module(&mut graph, idx_b, "/b.js");

    let ns_rec_a = graph.add_symbol(idx_a, "__ns__".to_string());
    let ns_rec_b = graph.add_symbol(idx_b, "__ns__".to_string());

    let mut module_a = make_normal_module(
        idx_a,
        "/a.js",
        FxHashMap::default(),
        FxHashMap::default(),
        vec![ResolvedImportRecord {
            specifier: CompactString::new("./b"),
            resolved_module: Some(idx_b),
            kind: ImportKind::Require,
            namespace_ref: ns_rec_a,
            meta: ImportRecordMeta::empty(),
        }],
        default_a,
        ns_a,
    );
    module_a.exports_kind = ExportsKind::Esm;
    graph.add_normal_module(module_a);

    let mut module_b = make_normal_module(
        idx_b,
        "/b.js",
        FxHashMap::default(),
        FxHashMap::default(),
        vec![ResolvedImportRecord {
            specifier: CompactString::new("./a"),
            resolved_module: Some(idx_a),
            kind: ImportKind::Static,
            namespace_ref: ns_rec_b,
            meta: ImportRecordMeta::empty(),
        }],
        default_b,
        ns_b,
    );
    module_b.exports_kind = ExportsKind::Esm;
    module_b.wrap_kind = WrapKind::Esm; // B was marked wrapping by determine_module_exports_kind
    graph.add_normal_module(module_b);

    let config = WrapModulesConfig {
        on_demand_wrapping: false,
        strict_execution_order: false,
        skip_symbol_creation: false,
    };
    let result = wrap_modules(&mut graph, &config);

    // Should terminate (no infinite loop) and A should get wrapped via propagation.
    assert!(
        result.wrap_kind_updates.contains_key(&idx_a),
        "A should be wrapped via propagation from B"
    );
}

#[test]
fn test_wrap_runtime_not_wrapped() {
    // Runtime module should not be wrapped even if it's a dependency of a wrapped module.
    let mut graph = ModuleGraph::new();
    graph.symbols.ensure_modules(2);

    let idx_runtime = ModuleIdx::from_usize(0);
    let idx_a = ModuleIdx::from_usize(1);

    let (default_rt, ns_rt) = make_simple_module(&mut graph, idx_runtime, "/runtime.js");
    let (default_a, ns_a) = make_simple_module(&mut graph, idx_a, "/a.js");

    let ns_rec = graph.add_symbol(idx_a, "__ns__".to_string());

    graph.add_normal_module(make_normal_module(
        idx_runtime,
        "/runtime.js",
        FxHashMap::default(),
        FxHashMap::default(),
        Vec::new(),
        default_rt,
        ns_rt,
    ));

    let mut module_a = make_normal_module(
        idx_a,
        "/a.js",
        FxHashMap::default(),
        FxHashMap::default(),
        vec![ResolvedImportRecord {
            specifier: CompactString::new("./runtime"),
            resolved_module: Some(idx_runtime),
            kind: ImportKind::Static,
            namespace_ref: ns_rec,
            meta: ImportRecordMeta::empty(),
        }],
        default_a,
        ns_a,
    );
    module_a.exports_kind = ExportsKind::Esm;
    module_a.wrap_kind = WrapKind::Esm;
    graph.add_normal_module(module_a);

    graph.set_runtime(idx_runtime);

    let config = WrapModulesConfig {
        on_demand_wrapping: false,
        strict_execution_order: false,
        skip_symbol_creation: false,
    };
    let result = wrap_modules(&mut graph, &config);

    assert!(
        !result.wrap_kind_updates.contains_key(&idx_runtime),
        "Runtime module should not be wrapped"
    );
}

// --- Full pipeline test ---

#[test]
fn test_link_with_cjs_interop() {
    // A(ESM) --static--> B(None) --require--> C(None)
    // After link: B=ESM, C=CJS+wrapped, dynamic_exports on C, wrapper_ref on C.
    let mut graph = ModuleGraph::new();
    graph.symbols.ensure_modules(3);

    let idx_a = ModuleIdx::from_usize(0);
    let idx_b = ModuleIdx::from_usize(1);
    let idx_c = ModuleIdx::from_usize(2);

    let (default_a, ns_a) = make_simple_module(&mut graph, idx_a, "/a.js");
    let (default_b, ns_b) = make_simple_module(&mut graph, idx_b, "/b.js");
    let (default_c, ns_c) = make_simple_module(&mut graph, idx_c, "/c.js");

    let ns_rec_a = graph.add_symbol(idx_a, "__ns__".to_string());
    let ns_rec_b = graph.add_symbol(idx_b, "__ns__".to_string());

    graph.add_normal_module(make_normal_module(
        idx_a,
        "/a.js",
        FxHashMap::default(),
        FxHashMap::default(),
        vec![ResolvedImportRecord {
            specifier: CompactString::new("./b"),
            resolved_module: Some(idx_b),
            kind: ImportKind::Static,
            namespace_ref: ns_rec_a,
            meta: ImportRecordMeta::empty(),
        }],
        default_a,
        ns_a,
    ));

    graph.add_normal_module(make_normal_module(
        idx_b,
        "/b.js",
        FxHashMap::default(),
        FxHashMap::default(),
        vec![ResolvedImportRecord {
            specifier: CompactString::new("./c"),
            resolved_module: Some(idx_c),
            kind: ImportKind::Require,
            namespace_ref: ns_rec_b,
            meta: ImportRecordMeta::empty(),
        }],
        default_b,
        ns_b,
    ));

    graph.add_normal_module(make_normal_module(
        idx_c,
        "/c.js",
        FxHashMap::default(),
        FxHashMap::default(),
        Vec::new(),
        default_c,
        ns_c,
    ));

    graph.set_entries(vec![idx_a]);
    graph.link(&mut LinkConfig::default());

    // B should be ESM (static import from A turns None → ESM).
    let b = graph.normal_module(idx_b).unwrap();
    assert_eq!(b.exports_kind, ExportsKind::Esm, "B should be ESM after link");

    // C should be CJS (require from B turns None → CJS).
    let c = graph.normal_module(idx_c).unwrap();
    assert_eq!(c.exports_kind, ExportsKind::CommonJs, "C should be CJS after link");
    assert_eq!(c.wrap_kind, WrapKind::Cjs, "C should be wrapped CJS");
    assert!(c.wrapper_ref.is_some(), "C should have a wrapper_ref");
    assert!(c.has_dynamic_exports, "CJS C should have dynamic exports");

    // Wrapper name should be "require_c".
    let wrapper = c.wrapper_ref.unwrap();
    assert_eq!(graph.symbol_name(wrapper), "require_c");
}

// --- Phase C: has_lazy_export tests ---

#[test]
fn test_exports_kind_lazy_export_stays_none() {
    // A static-imports B (None, has_lazy_export=true) → B stays None
    let mut graph = ModuleGraph::new();
    graph.symbols.ensure_modules(2);

    let idx_a = ModuleIdx::from_usize(0);
    let idx_b = ModuleIdx::from_usize(1);

    let (default_a, ns_a) = make_simple_module(&mut graph, idx_a, "/a.js");
    let (default_b, ns_b) = make_simple_module(&mut graph, idx_b, "/b.js");

    let ns_rec = graph.add_symbol(idx_a, "__import_ns__".to_string());

    graph.add_normal_module(make_normal_module(
        idx_a,
        "/a.js",
        FxHashMap::default(),
        FxHashMap::default(),
        vec![ResolvedImportRecord {
            specifier: CompactString::new("./b"),
            resolved_module: Some(idx_b),
            kind: ImportKind::Static,
            namespace_ref: ns_rec,
            meta: ImportRecordMeta::empty(),
        }],
        default_a,
        ns_a,
    ));

    let mut module_b = make_normal_module(
        idx_b,
        "/b.js",
        FxHashMap::default(),
        FxHashMap::default(),
        Vec::new(),
        default_b,
        ns_b,
    );
    module_b.has_lazy_export = true;
    graph.add_normal_module(module_b);

    let config = ExportsKindConfig { dynamic_imports_as_require: false, wrap_cjs_entries: false };
    let result = determine_module_exports_kind(&graph, &config);

    assert!(
        !result.exports_kind_updates.contains_key(&idx_b),
        "Module with has_lazy_export should NOT be classified as ESM by static import"
    );
}

#[test]
fn test_exports_kind_lazy_export_require_still_cjs() {
    // A requires B (None, has_lazy_export=true) → B → CJS + wrap
    let mut graph = ModuleGraph::new();
    graph.symbols.ensure_modules(2);

    let idx_a = ModuleIdx::from_usize(0);
    let idx_b = ModuleIdx::from_usize(1);

    let (default_a, ns_a) = make_simple_module(&mut graph, idx_a, "/a.js");
    let (default_b, ns_b) = make_simple_module(&mut graph, idx_b, "/b.js");

    let ns_rec = graph.add_symbol(idx_a, "__import_ns__".to_string());

    graph.add_normal_module(make_normal_module(
        idx_a,
        "/a.js",
        FxHashMap::default(),
        FxHashMap::default(),
        vec![ResolvedImportRecord {
            specifier: CompactString::new("./b"),
            resolved_module: Some(idx_b),
            kind: ImportKind::Require,
            namespace_ref: ns_rec,
            meta: ImportRecordMeta::empty(),
        }],
        default_a,
        ns_a,
    ));

    let mut module_b = make_normal_module(
        idx_b,
        "/b.js",
        FxHashMap::default(),
        FxHashMap::default(),
        Vec::new(),
        default_b,
        ns_b,
    );
    module_b.has_lazy_export = true;
    graph.add_normal_module(module_b);

    let config = ExportsKindConfig { dynamic_imports_as_require: false, wrap_cjs_entries: false };
    let result = determine_module_exports_kind(&graph, &config);

    assert_eq!(
        result.exports_kind_updates.get(&idx_b).copied(),
        Some(ExportsKind::CommonJs),
        "Require of lazy-export module should still become CJS"
    );
    assert_eq!(
        result.wrap_kind_updates.get(&idx_b).copied(),
        Some(WrapKind::Cjs),
        "Require of lazy-export module should get CJS wrap"
    );
}

// --- Phase C: wrap_modules new feature tests ---

#[test]
fn test_wrap_required_by_other_module() {
    // A requires B, B imports C → B in required_by_other_module
    let mut graph = ModuleGraph::new();
    graph.symbols.ensure_modules(3);

    let idx_a = ModuleIdx::from_usize(0);
    let idx_b = ModuleIdx::from_usize(1);
    let idx_c = ModuleIdx::from_usize(2);

    let (default_a, ns_a) = make_simple_module(&mut graph, idx_a, "/a.js");
    let (default_b, ns_b) = make_simple_module(&mut graph, idx_b, "/b.js");
    let (default_c, ns_c) = make_simple_module(&mut graph, idx_c, "/c.js");

    let ns_rec_a = graph.add_symbol(idx_a, "__ns_rec__".to_string());
    let ns_rec_b = graph.add_symbol(idx_b, "__ns_rec__".to_string());

    // A requires B
    graph.add_normal_module(make_normal_module(
        idx_a,
        "/a.js",
        FxHashMap::default(),
        FxHashMap::default(),
        vec![ResolvedImportRecord {
            specifier: CompactString::new("./b"),
            resolved_module: Some(idx_b),
            kind: ImportKind::Require,
            namespace_ref: ns_rec_a,
            meta: ImportRecordMeta::empty(),
        }],
        default_a,
        ns_a,
    ));

    // B imports C (static)
    let mut module_b = make_normal_module(
        idx_b,
        "/b.js",
        FxHashMap::default(),
        FxHashMap::default(),
        vec![ResolvedImportRecord {
            specifier: CompactString::new("./c"),
            resolved_module: Some(idx_c),
            kind: ImportKind::Static,
            namespace_ref: ns_rec_b,
            meta: ImportRecordMeta::empty(),
        }],
        default_b,
        ns_b,
    );
    // B is ESM and gets required → will be wrap ESM from exports_kind
    module_b.exports_kind = ExportsKind::Esm;
    module_b.wrap_kind = WrapKind::Esm;
    graph.add_normal_module(module_b);

    // C is a leaf
    graph.add_normal_module(make_normal_module(
        idx_c,
        "/c.js",
        FxHashMap::default(),
        FxHashMap::default(),
        Vec::new(),
        default_c,
        ns_c,
    ));

    let config = WrapModulesConfig {
        on_demand_wrapping: false,
        strict_execution_order: false,
        skip_symbol_creation: false,
    };
    let result = wrap_modules(&mut graph, &config);

    assert!(
        result.required_by_other_module.contains(&idx_b),
        "B should be in required_by_other_module since A requires B"
    );
}

#[test]
fn test_wrap_original_wrap_kind_preserved() {
    // A requires B(ESM), propagates to C → original_wrap_kind for C is set
    let mut graph = ModuleGraph::new();
    graph.symbols.ensure_modules(3);

    let idx_a = ModuleIdx::from_usize(0);
    let idx_b = ModuleIdx::from_usize(1);
    let idx_c = ModuleIdx::from_usize(2);

    let (default_a, ns_a) = make_simple_module(&mut graph, idx_a, "/a.js");
    let (default_b, ns_b) = make_simple_module(&mut graph, idx_b, "/b.js");
    let (default_c, ns_c) = make_simple_module(&mut graph, idx_c, "/c.js");

    let ns_rec_a = graph.add_symbol(idx_a, "__ns_rec__".to_string());
    let ns_rec_b = graph.add_symbol(idx_b, "__ns_rec__".to_string());

    graph.add_normal_module(make_normal_module(
        idx_a,
        "/a.js",
        FxHashMap::default(),
        FxHashMap::default(),
        vec![ResolvedImportRecord {
            specifier: CompactString::new("./b"),
            resolved_module: Some(idx_b),
            kind: ImportKind::Require,
            namespace_ref: ns_rec_a,
            meta: ImportRecordMeta::empty(),
        }],
        default_a,
        ns_a,
    ));

    // B is ESM with wrap from exports_kind
    let mut module_b = make_normal_module(
        idx_b,
        "/b.js",
        FxHashMap::default(),
        FxHashMap::default(),
        vec![ResolvedImportRecord {
            specifier: CompactString::new("./c"),
            resolved_module: Some(idx_c),
            kind: ImportKind::Static,
            namespace_ref: ns_rec_b,
            meta: ImportRecordMeta::empty(),
        }],
        default_b,
        ns_b,
    );
    module_b.exports_kind = ExportsKind::Esm;
    module_b.wrap_kind = WrapKind::Esm;
    graph.add_normal_module(module_b);

    // C has imports (so on-demand won't skip it)
    graph.add_normal_module(make_normal_module(
        idx_c,
        "/c.js",
        FxHashMap::default(),
        FxHashMap::default(),
        Vec::new(),
        default_c,
        ns_c,
    ));

    let config = WrapModulesConfig {
        on_demand_wrapping: false,
        strict_execution_order: false,
        skip_symbol_creation: false,
    };
    let result = wrap_modules(&mut graph, &config);

    // C should get wrapped via propagation, and its original_wrap_kind should be recorded
    assert!(
        result.original_wrap_kinds.contains_key(&idx_c),
        "C's original_wrap_kind should be recorded during propagation"
    );
    assert_eq!(
        result.original_wrap_kinds.get(&idx_c).copied(),
        Some(WrapKind::Esm),
        "C's original_wrap_kind should be Esm"
    );
}

#[test]
fn test_wrap_strict_execution_order() {
    // strict=true, mixed graph → All CJS wrapped, ESM wrapped
    let mut graph = ModuleGraph::new();
    graph.symbols.ensure_modules(3);

    let idx_a = ModuleIdx::from_usize(0);
    let idx_b = ModuleIdx::from_usize(1);
    let idx_c = ModuleIdx::from_usize(2);

    let (default_a, ns_a) = make_simple_module(&mut graph, idx_a, "/a.js");
    let (default_b, ns_b) = make_simple_module(&mut graph, idx_b, "/b.js");
    let (default_c, ns_c) = make_simple_module(&mut graph, idx_c, "/c.js");

    let ns_rec_a = graph.add_symbol(idx_a, "__ns_rec__".to_string());

    // A imports B (static)
    graph.add_normal_module(make_normal_module(
        idx_a,
        "/a.js",
        FxHashMap::default(),
        FxHashMap::default(),
        vec![ResolvedImportRecord {
            specifier: CompactString::new("./b"),
            resolved_module: Some(idx_b),
            kind: ImportKind::Static,
            namespace_ref: ns_rec_a,
            meta: ImportRecordMeta::empty(),
        }],
        default_a,
        ns_a,
    ));

    // B is CJS
    let mut module_b = make_normal_module(
        idx_b,
        "/b.js",
        FxHashMap::default(),
        FxHashMap::default(),
        Vec::new(),
        default_b,
        ns_b,
    );
    module_b.exports_kind = ExportsKind::CommonJs;
    graph.add_normal_module(module_b);

    // C is ESM, no imports
    let mut module_c = make_normal_module(
        idx_c,
        "/c.js",
        FxHashMap::default(),
        FxHashMap::default(),
        Vec::new(),
        default_c,
        ns_c,
    );
    module_c.exports_kind = ExportsKind::Esm;
    graph.add_normal_module(module_c);

    let config = WrapModulesConfig {
        on_demand_wrapping: false,
        strict_execution_order: true,
        skip_symbol_creation: false,
    };
    let result = wrap_modules(&mut graph, &config);

    // A should be wrapped ESM (strict forces wrapping)
    assert_eq!(
        result.wrap_kind_updates.get(&idx_a).copied(),
        Some(WrapKind::Esm),
        "A should be wrapped ESM in strict mode"
    );
    // B should be wrapped CJS
    assert_eq!(
        result.wrap_kind_updates.get(&idx_b).copied(),
        Some(WrapKind::Cjs),
        "B(CJS) should be wrapped CJS in strict mode"
    );
    // C should be wrapped ESM
    assert_eq!(
        result.wrap_kind_updates.get(&idx_c).copied(),
        Some(WrapKind::Esm),
        "C(ESM) should be wrapped ESM in strict mode"
    );
}

#[test]
fn test_wrap_strict_with_on_demand() {
    // strict+on_demand, pure ESM leaf with no imports → Leaf skips wrapping
    let mut graph = ModuleGraph::new();
    graph.symbols.ensure_modules(2);

    let idx_a = ModuleIdx::from_usize(0);
    let idx_b = ModuleIdx::from_usize(1);

    let (default_a, ns_a) = make_simple_module(&mut graph, idx_a, "/a.js");
    let (default_b, ns_b) = make_simple_module(&mut graph, idx_b, "/b.js");

    let ns_rec_a = graph.add_symbol(idx_a, "__ns_rec__".to_string());

    // A imports B (static)
    graph.add_normal_module(make_normal_module(
        idx_a,
        "/a.js",
        FxHashMap::default(),
        FxHashMap::default(),
        vec![ResolvedImportRecord {
            specifier: CompactString::new("./b"),
            resolved_module: Some(idx_b),
            kind: ImportKind::Static,
            namespace_ref: ns_rec_a,
            meta: ImportRecordMeta::empty(),
        }],
        default_a,
        ns_a,
    ));

    // B is ESM leaf with no imports
    let mut module_b = make_normal_module(
        idx_b,
        "/b.js",
        FxHashMap::default(),
        FxHashMap::default(),
        Vec::new(),
        default_b,
        ns_b,
    );
    module_b.exports_kind = ExportsKind::Esm;
    graph.add_normal_module(module_b);

    let config = WrapModulesConfig {
        on_demand_wrapping: true,
        strict_execution_order: true,
        skip_symbol_creation: false,
    };
    let result = wrap_modules(&mut graph, &config);

    // B should skip wrapping (on-demand: pure ESM, no imports, not sensitive)
    assert!(
        !result.wrap_kind_updates.contains_key(&idx_b),
        "Pure ESM leaf B should skip wrapping with strict+on_demand"
    );
}

#[test]
fn test_wrap_execution_order_sensitive() {
    // on_demand, sensitive module with no imports → Still wrapped
    let mut graph = ModuleGraph::new();
    graph.symbols.ensure_modules(2);

    let idx_a = ModuleIdx::from_usize(0);
    let idx_b = ModuleIdx::from_usize(1);

    let (default_a, ns_a) = make_simple_module(&mut graph, idx_a, "/a.js");
    let (default_b, ns_b) = make_simple_module(&mut graph, idx_b, "/b.js");

    let ns_rec_a = graph.add_symbol(idx_a, "__ns_rec__".to_string());

    // A imports B, A is wrapped
    let mut module_a = make_normal_module(
        idx_a,
        "/a.js",
        FxHashMap::default(),
        FxHashMap::default(),
        vec![ResolvedImportRecord {
            specifier: CompactString::new("./b"),
            resolved_module: Some(idx_b),
            kind: ImportKind::Static,
            namespace_ref: ns_rec_a,
            meta: ImportRecordMeta::empty(),
        }],
        default_a,
        ns_a,
    );
    module_a.wrap_kind = WrapKind::Esm;
    graph.add_normal_module(module_a);

    // B is ESM leaf with no imports but execution_order_sensitive
    let mut module_b = make_normal_module(
        idx_b,
        "/b.js",
        FxHashMap::default(),
        FxHashMap::default(),
        Vec::new(),
        default_b,
        ns_b,
    );
    module_b.exports_kind = ExportsKind::Esm;
    module_b.execution_order_sensitive = true;
    graph.add_normal_module(module_b);

    let config = WrapModulesConfig {
        on_demand_wrapping: true,
        strict_execution_order: false,
        skip_symbol_creation: false,
    };
    let result = wrap_modules(&mut graph, &config);

    // B should still be wrapped because it's execution_order_sensitive
    assert_eq!(
        result.wrap_kind_updates.get(&idx_b).copied(),
        Some(WrapKind::Esm),
        "Execution-order-sensitive module should be wrapped even with on-demand wrapping"
    );
}

// --- Phase C: determine_safely_merge_cjs_ns tests ---

#[test]
fn test_safely_merge_basic() {
    // A(ESM) imports B(CJS) → B in map, namespace_ref from A
    let mut graph = ModuleGraph::new();
    graph.symbols.ensure_modules(2);

    let idx_a = ModuleIdx::from_usize(0);
    let idx_b = ModuleIdx::from_usize(1);

    let (default_a, ns_a) = make_simple_module(&mut graph, idx_a, "/a.js");
    let (default_b, ns_b) = make_simple_module(&mut graph, idx_b, "/b.js");

    let ns_rec = graph.add_symbol(idx_a, "__import_ns__".to_string());

    let mut module_a = make_normal_module(
        idx_a,
        "/a.js",
        FxHashMap::default(),
        FxHashMap::default(),
        vec![ResolvedImportRecord {
            specifier: CompactString::new("./b"),
            resolved_module: Some(idx_b),
            kind: ImportKind::Static,
            namespace_ref: ns_rec,
            meta: ImportRecordMeta::empty(),
        }],
        default_a,
        ns_a,
    );
    module_a.exports_kind = ExportsKind::Esm;
    graph.add_normal_module(module_a);

    let mut module_b = make_normal_module(
        idx_b,
        "/b.js",
        FxHashMap::default(),
        FxHashMap::default(),
        Vec::new(),
        default_b,
        ns_b,
    );
    module_b.exports_kind = ExportsKind::CommonJs;
    graph.add_normal_module(module_b);

    let result = determine_safely_merge_cjs_ns(&graph);

    assert!(result.contains_key(&idx_b), "B(CJS) should be in the merge map");
    let info = &result[&idx_b];
    assert_eq!(info.namespace_refs.len(), 1, "Should have 1 namespace ref from A");
    assert_eq!(info.namespace_refs[0], ns_rec);
    assert!(info.needs_interop, "CJS target should need interop");
}

#[test]
fn test_safely_merge_excludes_star_export() {
    // A has `export * from B(CJS)` → B NOT in map
    let mut graph = ModuleGraph::new();
    graph.symbols.ensure_modules(2);

    let idx_a = ModuleIdx::from_usize(0);
    let idx_b = ModuleIdx::from_usize(1);

    let (default_a, ns_a) = make_simple_module(&mut graph, idx_a, "/a.js");
    let (default_b, ns_b) = make_simple_module(&mut graph, idx_b, "/b.js");

    let ns_rec = graph.add_symbol(idx_a, "__import_ns__".to_string());

    let mut module_a = make_normal_module(
        idx_a,
        "/a.js",
        FxHashMap::default(),
        FxHashMap::default(),
        vec![ResolvedImportRecord {
            specifier: CompactString::new("./b"),
            resolved_module: Some(idx_b),
            kind: ImportKind::Static,
            namespace_ref: ns_rec,
            meta: ImportRecordMeta::IS_EXPORT_STAR, // star export
        }],
        default_a,
        ns_a,
    );
    module_a.exports_kind = ExportsKind::Esm;
    graph.add_normal_module(module_a);

    let mut module_b = make_normal_module(
        idx_b,
        "/b.js",
        FxHashMap::default(),
        FxHashMap::default(),
        Vec::new(),
        default_b,
        ns_b,
    );
    module_b.exports_kind = ExportsKind::CommonJs;
    graph.add_normal_module(module_b);

    let result = determine_safely_merge_cjs_ns(&graph);

    assert!(
        !result.contains_key(&idx_b),
        "Star export to CJS module should NOT be in the merge map"
    );
}

#[test]
fn test_safely_merge_multiple_importers() {
    // A and C both import B(CJS) → B has 2 namespace_refs
    let mut graph = ModuleGraph::new();
    graph.symbols.ensure_modules(3);

    let idx_a = ModuleIdx::from_usize(0);
    let idx_b = ModuleIdx::from_usize(1);
    let idx_c = ModuleIdx::from_usize(2);

    let (default_a, ns_a) = make_simple_module(&mut graph, idx_a, "/a.js");
    let (default_b, ns_b) = make_simple_module(&mut graph, idx_b, "/b.js");
    let (default_c, ns_c) = make_simple_module(&mut graph, idx_c, "/c.js");

    let ns_rec_a = graph.add_symbol(idx_a, "__import_ns__".to_string());
    let ns_rec_c = graph.add_symbol(idx_c, "__import_ns__".to_string());

    // A imports B
    let mut module_a = make_normal_module(
        idx_a,
        "/a.js",
        FxHashMap::default(),
        FxHashMap::default(),
        vec![ResolvedImportRecord {
            specifier: CompactString::new("./b"),
            resolved_module: Some(idx_b),
            kind: ImportKind::Static,
            namespace_ref: ns_rec_a,
            meta: ImportRecordMeta::empty(),
        }],
        default_a,
        ns_a,
    );
    module_a.exports_kind = ExportsKind::Esm;
    graph.add_normal_module(module_a);

    // B is CJS
    let mut module_b = make_normal_module(
        idx_b,
        "/b.js",
        FxHashMap::default(),
        FxHashMap::default(),
        Vec::new(),
        default_b,
        ns_b,
    );
    module_b.exports_kind = ExportsKind::CommonJs;
    graph.add_normal_module(module_b);

    // C imports B
    let mut module_c = make_normal_module(
        idx_c,
        "/c.js",
        FxHashMap::default(),
        FxHashMap::default(),
        vec![ResolvedImportRecord {
            specifier: CompactString::new("./b"),
            resolved_module: Some(idx_b),
            kind: ImportKind::Static,
            namespace_ref: ns_rec_c,
            meta: ImportRecordMeta::empty(),
        }],
        default_c,
        ns_c,
    );
    module_c.exports_kind = ExportsKind::Esm;
    graph.add_normal_module(module_c);

    let result = determine_safely_merge_cjs_ns(&graph);

    assert!(result.contains_key(&idx_b), "B(CJS) should be in the merge map");
    let info = &result[&idx_b];
    assert_eq!(info.namespace_refs.len(), 2, "Should have 2 namespace refs from A and C");
    assert!(info.namespace_refs.contains(&ns_rec_a), "Should contain A's namespace ref");
    assert!(info.namespace_refs.contains(&ns_rec_c), "Should contain C's namespace ref");
}

// ============================================================================
// reserve_modules tests
// ============================================================================

#[test]
fn reserve_modules_preallocates_without_affecting_correctness() {
    let mut graph = ModuleGraph::new();

    // Pre-allocate space for 100 modules.
    graph.reserve_modules(100);

    // Allocate and add a few modules — correctness must be preserved.
    let idx_a = graph.alloc_module_idx();
    let idx_b = graph.alloc_module_idx();

    let sym_a = graph.add_symbol(idx_a, "a".into());
    let sym_b = graph.add_symbol(idx_b, "b".into());

    let module_a = make_normal_module(
        idx_a,
        "/a.js",
        FxHashMap::default(),
        FxHashMap::default(),
        vec![],
        sym_a,
        sym_a,
    );
    graph.add_normal_module(module_a);

    let module_b = make_normal_module(
        idx_b,
        "/b.js",
        FxHashMap::default(),
        FxHashMap::default(),
        vec![],
        sym_b,
        sym_b,
    );
    graph.add_normal_module(module_b);

    // Verify modules are accessible.
    assert_eq!(graph.modules_len(), 2);
    assert!(graph.normal_module(idx_a).is_some());
    assert!(graph.normal_module(idx_b).is_some());
    assert_eq!(graph.symbol_name(sym_a), "a");
    assert_eq!(graph.symbol_name(sym_b), "b");
}

// ============================================================================
// Path halving correctness tests
// ============================================================================

#[test]
fn canonical_ref_mut_applies_path_halving() {
    let mut graph = ModuleGraph::new();

    // Create 5 modules (A→B→C→D→E chain via symbol links).
    let idxs: Vec<ModuleIdx> = (0..5).map(|_| graph.alloc_module_idx()).collect();
    let syms: Vec<SymbolRef> =
        idxs.iter().map(|&idx| graph.add_symbol(idx, format!("s{}", idx.index()))).collect();

    for &idx in &idxs {
        let sym = syms[idx.index()];
        let module = make_normal_module(
            idx,
            &format!("/{}.js", idx.index()),
            FxHashMap::default(),
            FxHashMap::default(),
            vec![],
            sym,
            sym,
        );
        graph.add_normal_module(module);
    }

    // Build chain: A→B→C→D→E using link_symbols to set each hop.
    // link_symbols(from, to) makes `root(from)` resolve to `root(to)`.
    // Since each symbol is its own root initially, linking in forward order
    // produces the chain: sym[0]→sym[1]→sym[2]→sym[3]→sym[4].
    for i in 0..4 {
        graph.link_symbols(syms[i], syms[i + 1]);
    }

    // Before path halving, canonical_ref (immutable) should find E as root.
    let root = graph.canonical_ref(syms[0]);
    assert_eq!(root, syms[4], "canonical root should be E (sym[4])");

    // Call canonical_ref_mut which applies path halving.
    let root_mut = graph.canonical_ref_mut(syms[0]);
    assert_eq!(root_mut, syms[4], "canonical_ref_mut should still find E");

    // After path halving, a second call should still find E.
    // If path halving is working, the internal chain is shortened.
    let root_again = graph.canonical_ref_mut(syms[0]);
    assert_eq!(root_again, syms[4], "canonical root should still be E after repeated calls");

    // Verify all intermediate symbols also resolve to E.
    for i in 1..4 {
        let root_i = graph.canonical_ref_mut(syms[i]);
        assert_eq!(root_i, syms[4], "sym[{}] should resolve to E", i);
    }
}
