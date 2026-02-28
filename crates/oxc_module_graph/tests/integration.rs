use std::path::PathBuf;

use compact_str::CompactString;
use oxc_module_graph::default::SymbolRefDb;
use oxc_module_graph::types::{
    ImportKind, ImportRecordIdx, LocalExport, ModuleIdx, NamedImport, ResolvedImportRecord,
    StarExportEntry, SymbolRef,
};
use oxc_module_graph::{
    ModuleGraph, NormalModule, SideEffects, bind_imports_and_exports, compute_exec_order,
    compute_has_dynamic_exports, compute_tla, determine_side_effects, find_cycles,
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
        is_commonjs: false,
        has_top_level_await: false,
        side_effects: SideEffects::True,
        named_exports,
        named_imports,
        import_records,
        star_export_entries: Vec::new(),
        indirect_export_entries: Vec::new(),
        default_export_ref,
        namespace_object_ref,
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
        is_commonjs: false,
        has_top_level_await: false,
        side_effects: SideEffects::True,
        named_exports,
        named_imports,
        import_records: vec![ResolvedImportRecord {
            specifier: CompactString::new("./b"),
            resolved_module: Some(idx_b),
            kind: ImportKind::Static,
        }],
        default_export_ref: default_ref,
        namespace_object_ref: ns_ref,
        star_export_entries: vec![],
        indirect_export_entries: vec![],
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
    module.is_commonjs = true;
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
    module_b.is_commonjs = true;
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
