use std::path::PathBuf;

use compact_str::CompactString;
use oxc_module_graph::default::{DefaultModuleGraph, Module, SymbolRefDb};
use oxc_module_graph::traits::{ModuleInfo, ModuleStore, SymbolGraph};
use oxc_module_graph::types::{
    ImportEdge, ImportKind, ImportRecordIdx, LocalExport, MatchImportKind, ModuleIdx, NamedImport,
    ResolvedImportRecord, SymbolRef,
};
use rustc_hash::FxHashMap;

fn dummy_symbol_ref(module: ModuleIdx, id: u32) -> SymbolRef {
    use oxc_syntax::symbol::SymbolId;
    SymbolRef::new(module, SymbolId::from_raw_unchecked(id))
}

#[test]
fn test_module_graph_basic() {
    let mut graph = DefaultModuleGraph::new();

    let idx_a = graph.next_idx();
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

    let module_a = Module {
        idx: idx_a,
        path: PathBuf::from("/a.js"),
        has_module_syntax: true,
        is_commonjs: false,
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
        dependencies: vec![ImportEdge {
            specifier: CompactString::new("./b"),
            target: idx_b,
            is_type: false,
        }],
    };

    graph.add_module(module_a);

    // Module B: exports `foo`
    let sym_foo_b = dummy_symbol_ref(idx_b, 0);
    let default_ref_b = dummy_symbol_ref(idx_b, 1);
    let ns_ref_b = dummy_symbol_ref(idx_b, 2);

    let mut named_exports_b = FxHashMap::default();
    named_exports_b.insert(
        CompactString::new("foo"),
        LocalExport { exported_name: CompactString::new("foo"), local_symbol: sym_foo_b },
    );

    let module_b = Module {
        idx: idx_b,
        path: PathBuf::from("/b.js"),
        has_module_syntax: true,
        is_commonjs: false,
        named_exports: named_exports_b,
        named_imports: FxHashMap::default(),
        import_records: vec![],
        default_export_ref: default_ref_b,
        namespace_object_ref: ns_ref_b,
        star_export_entries: vec![],
        indirect_export_entries: vec![],
        dependencies: vec![],
    };

    graph.add_module(module_b);

    // Verify ModuleStore trait
    assert_eq!(graph.modules_len(), 2);
    let module_a_ref = graph.module(idx_a).unwrap();
    assert!(module_a_ref.has_module_syntax());
    assert_eq!(module_a_ref.module_idx(), idx_a);

    // Verify dependencies via for_each_dependency
    let mut deps_a = Vec::new();
    graph.for_each_dependency(idx_a, &mut |dep| deps_a.push(dep));
    assert_eq!(deps_a.len(), 1);
    assert_eq!(deps_a[0], idx_b);

    let mut deps_b = Vec::new();
    graph.for_each_dependency(idx_b, &mut |dep| deps_b.push(dep));
    assert_eq!(deps_b.len(), 0);

    // Verify exports via for_each_named_export
    let mut a_exports = Vec::new();
    module_a_ref.for_each_named_export(&mut |name, _, _| a_exports.push(name.to_string()));
    assert!(a_exports.contains(&"bar".to_string()));

    let module_b_ref = graph.module(idx_b).unwrap();
    let mut b_exports = Vec::new();
    module_b_ref.for_each_named_export(&mut |name, _, _| b_exports.push(name.to_string()));
    assert!(b_exports.contains(&"foo".to_string()));

    // Verify for_each_module
    let mut module_count = 0;
    graph.for_each_module(&mut |_, _| module_count += 1);
    assert_eq!(module_count, 2);
}

#[test]
fn test_symbol_ref_db() {
    let mut db = SymbolRefDb::new();
    db.ensure_modules(2);

    let module_a = ModuleIdx::from_usize(0);
    let module_b = ModuleIdx::from_usize(1);

    let sym_a = db.add_symbol(module_a, "foo".to_string());
    let sym_b = db.add_symbol(module_b, "bar".to_string());

    // Initially, each symbol is canonical
    assert_eq!(db.canonical_ref_for(sym_a), sym_a);
    assert_eq!(db.canonical_ref_for(sym_b), sym_b);

    // Link sym_a -> sym_b
    db.link(sym_a, sym_b);
    assert_eq!(db.canonical_ref_for(sym_a), sym_b);
    assert_eq!(db.canonical_ref_for(sym_b), sym_b);

    // Names
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

    // Chain: s0 -> s1 -> s2
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
    let source = r#"
        import { foo } from './bar';
        export const baz = foo + 1;
        export default 42;
    "#;

    let source_type = SourceType::mjs();
    let ret = Parser::new(&allocator, source, source_type).parse();
    let record = ModuleRecord::from_syntax(&ret.module_record);

    assert!(record.has_module_syntax);
    assert_eq!(record.import_entries.len(), 1);
    assert_eq!(record.import_entries[0].module_request.name(), "./bar");
    assert!(record.local_export_entries.len() >= 1);
    assert!(record.export_default.is_some());
}

// --- Stage 3: Builder tests ---

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

#[test]
fn test_builder_basic() {
    use oxc_module_graph::default::ModuleGraphBuilder;
    use oxc_module_graph::traits::ModuleStore;

    let entry = fixtures_dir().join("entry.js");
    let result = ModuleGraphBuilder::new().build(&[entry]);

    // entry.js imports dep.js and dep2.js; dep2.js imports dep.js
    // Total: 3 modules
    assert_eq!(result.graph.modules_len(), 3);
    assert!(result.errors.is_empty(), "Unexpected errors: {:?}", result.errors);
}

#[test]
fn test_builder_circular() {
    use oxc_module_graph::default::ModuleGraphBuilder;
    use oxc_module_graph::traits::ModuleStore;

    let entry = fixtures_dir().join("circular_a.js");
    let result = ModuleGraphBuilder::new().build(&[entry]);

    // circular_a.js <-> circular_b.js
    assert_eq!(result.graph.modules_len(), 2);
    assert!(result.errors.is_empty());

    // Both should have dependencies
    let mut deps_a = Vec::new();
    result.graph.for_each_dependency(ModuleIdx::from_usize(0), &mut |dep| deps_a.push(dep));
    let mut deps_b = Vec::new();
    result.graph.for_each_dependency(ModuleIdx::from_usize(1), &mut |dep| deps_b.push(dep));
    assert_eq!(deps_a.len(), 1);
    assert_eq!(deps_b.len(), 1);
}

#[test]
fn test_builder_reexport() {
    use oxc_module_graph::default::ModuleGraphBuilder;
    use oxc_module_graph::traits::ModuleStore;

    let entry = fixtures_dir().join("reexport.js");
    let result = ModuleGraphBuilder::new().build(&[entry]);

    // reexport.js imports dep.js and dep2.js; dep2.js imports dep.js
    assert_eq!(result.graph.modules_len(), 3);

    // reexport.js should have star_export_entries and indirect_export_entries
    // Access them directly on the Module struct (not via trait).
    let reexport_module = result.graph.module(ModuleIdx::from_usize(0)).unwrap();
    assert!(!reexport_module.star_export_entries.is_empty());
    assert!(!reexport_module.indirect_export_entries.is_empty());
}

// --- Stage 4: Binding algorithm tests ---

/// Helper to build a two-module graph manually for binding tests.
/// Module A imports `import_name` from Module B which exports it.
fn two_module_graph_with_binding(
    export_name: &str,
    import_name: &str,
) -> (DefaultModuleGraph, SymbolRefDb) {
    let mut graph = DefaultModuleGraph::new();
    let mut db = SymbolRefDb::new();
    db.ensure_modules(2);

    let idx_a = ModuleIdx::from_usize(0);
    let idx_b = ModuleIdx::from_usize(1);

    // Module B: exports `export_name`
    let sym_b_export = db.add_symbol(idx_b, export_name.to_string());
    let sym_b_default = db.add_symbol(idx_b, "__default__".to_string());
    let sym_b_ns = db.add_symbol(idx_b, "__namespace__".to_string());

    let mut named_exports_b = FxHashMap::default();
    named_exports_b.insert(
        CompactString::new(export_name),
        LocalExport { exported_name: CompactString::new(export_name), local_symbol: sym_b_export },
    );

    let module_b = Module {
        idx: idx_b,
        path: PathBuf::from("/b.js"),
        has_module_syntax: true,
        is_commonjs: false,
        named_exports: named_exports_b,
        named_imports: FxHashMap::default(),
        import_records: Vec::new(),
        default_export_ref: sym_b_default,
        namespace_object_ref: sym_b_ns,
        star_export_entries: Vec::new(),
        indirect_export_entries: Vec::new(),
        dependencies: Vec::new(),
    };

    // Module A: imports `import_name` from B
    let sym_a_local = db.add_symbol(idx_a, import_name.to_string());
    let sym_a_default = db.add_symbol(idx_a, "__default__".to_string());
    let sym_a_ns = db.add_symbol(idx_a, "__namespace__".to_string());

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

    let module_a = Module {
        idx: idx_a,
        path: PathBuf::from("/a.js"),
        has_module_syntax: true,
        is_commonjs: false,
        named_exports: FxHashMap::default(),
        named_imports: named_imports_a,
        import_records: vec![ResolvedImportRecord {
            specifier: CompactString::new("./b"),
            resolved_module: Some(idx_b),
            kind: ImportKind::Static,
        }],
        default_export_ref: sym_a_default,
        namespace_object_ref: sym_a_ns,
        star_export_entries: Vec::new(),
        indirect_export_entries: Vec::new(),
        dependencies: vec![ImportEdge {
            specifier: CompactString::new("./b"),
            target: idx_b,
            is_type: false,
        }],
    };

    // Add modules in order
    graph.add_module(module_a);
    graph.add_module(module_b);

    (graph, db)
}

#[test]
fn test_bind_named_import() {
    use oxc_module_graph::bind_imports_and_exports;
    use oxc_module_graph::traits::SymbolGraph;

    let (graph, mut db) = two_module_graph_with_binding("foo", "foo");

    let _result = bind_imports_and_exports(&graph, &mut db);

    // After binding, A's local "foo" should be linked to B's "foo"
    let idx_a = ModuleIdx::from_usize(0);
    let idx_b = ModuleIdx::from_usize(1);

    // A's first symbol (local "foo") should canonicalize to B's first symbol
    let a_foo = dummy_symbol_ref(idx_a, 0);
    let b_foo = dummy_symbol_ref(idx_b, 0);

    let canonical = db.canonical_ref_for(a_foo);
    assert_eq!(canonical, b_foo, "A's foo should link to B's foo");
}

#[test]
fn test_bind_unresolved_import() {
    use oxc_module_graph::bind_imports_and_exports;
    use oxc_module_graph::traits::SymbolGraph;

    // A imports "bar" but B only exports "foo"
    let (graph, mut db) = two_module_graph_with_binding("foo", "bar");

    let _result = bind_imports_and_exports(&graph, &mut db);

    // A's local "bar" should NOT be linked (no match)
    let idx_a = ModuleIdx::from_usize(0);
    let a_bar = dummy_symbol_ref(idx_a, 0);
    let canonical = db.canonical_ref_for(a_bar);
    assert_eq!(canonical, a_bar, "A's bar should remain unlinked");
}

#[test]
fn test_bind_star_reexport() {
    use oxc_module_graph::bind_imports_and_exports;
    use oxc_module_graph::default::Module;
    use oxc_module_graph::traits::SymbolGraph;
    use oxc_module_graph::types::StarExportEntry;

    let mut graph = DefaultModuleGraph::new();
    let mut db = SymbolRefDb::new();
    db.ensure_modules(3);

    let idx_a = ModuleIdx::from_usize(0);
    let idx_b = ModuleIdx::from_usize(1);
    let idx_c = ModuleIdx::from_usize(2);

    // Module C: exports "foo"
    let sym_c_foo = db.add_symbol(idx_c, "foo".to_string());
    let sym_c_default = db.add_symbol(idx_c, "__default__".to_string());
    let sym_c_ns = db.add_symbol(idx_c, "__namespace__".to_string());

    let mut exports_c = FxHashMap::default();
    exports_c.insert(
        CompactString::new("foo"),
        LocalExport { exported_name: CompactString::new("foo"), local_symbol: sym_c_foo },
    );

    // Module B: `export * from './c'` (star re-export)
    let sym_b_default = db.add_symbol(idx_b, "__default__".to_string());
    let sym_b_ns = db.add_symbol(idx_b, "__namespace__".to_string());

    // Module A: `import { foo } from './b'`
    let sym_a_foo = db.add_symbol(idx_a, "foo".to_string());
    let sym_a_default = db.add_symbol(idx_a, "__default__".to_string());
    let sym_a_ns = db.add_symbol(idx_a, "__namespace__".to_string());

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

    // Add modules in order
    graph.add_module(Module {
        idx: idx_a,
        path: PathBuf::from("/a.js"),
        has_module_syntax: true,
        is_commonjs: false,
        named_exports: FxHashMap::default(),
        named_imports: named_imports_a,
        import_records: vec![ResolvedImportRecord {
            specifier: CompactString::new("./b"),
            resolved_module: Some(idx_b),
            kind: ImportKind::Static,
        }],
        default_export_ref: sym_a_default,
        namespace_object_ref: sym_a_ns,
        star_export_entries: Vec::new(),
        indirect_export_entries: Vec::new(),
        dependencies: vec![ImportEdge {
            specifier: CompactString::new("./b"),
            target: idx_b,
            is_type: false,
        }],
    });

    graph.add_module(Module {
        idx: idx_b,
        path: PathBuf::from("/b.js"),
        has_module_syntax: true,
        is_commonjs: false,
        named_exports: FxHashMap::default(),
        named_imports: FxHashMap::default(),
        import_records: vec![ResolvedImportRecord {
            specifier: CompactString::new("./c"),
            resolved_module: Some(idx_c),
            kind: ImportKind::Static,
        }],
        default_export_ref: sym_b_default,
        namespace_object_ref: sym_b_ns,
        star_export_entries: vec![StarExportEntry {
            module_request: CompactString::new("./c"),
            resolved_module: Some(idx_c),
            span: oxc_span::Span::new(0, 0),
        }],
        indirect_export_entries: Vec::new(),
        dependencies: vec![ImportEdge {
            specifier: CompactString::new("./c"),
            target: idx_c,
            is_type: false,
        }],
    });

    graph.add_module(Module {
        idx: idx_c,
        path: PathBuf::from("/c.js"),
        has_module_syntax: true,
        is_commonjs: false,
        named_exports: exports_c,
        named_imports: FxHashMap::default(),
        import_records: Vec::new(),
        default_export_ref: sym_c_default,
        namespace_object_ref: sym_c_ns,
        star_export_entries: Vec::new(),
        indirect_export_entries: Vec::new(),
        dependencies: Vec::new(),
    });

    let _result = bind_imports_and_exports(&graph, &mut db);

    // A's "foo" should link through B's star re-export to C's "foo"
    let canonical = db.canonical_ref_for(sym_a_foo);
    assert_eq!(canonical, sym_c_foo, "A's foo should link through B's star export to C's foo");
}

// --- Stage 5: Graph algorithm tests ---

/// Helper to build a simple graph with no imports/symbols for graph algorithm tests.
fn simple_graph(edges: &[(usize, usize)]) -> DefaultModuleGraph {
    use oxc_module_graph::default::Module;

    // Find max module index.
    let max_idx = edges.iter().flat_map(|&(a, b)| [a, b]).max().map_or(0, |m| m + 1);

    let mut graph = DefaultModuleGraph::new();
    let mut db = SymbolRefDb::new();
    db.ensure_modules(max_idx);

    for i in 0..max_idx {
        let idx = ModuleIdx::from_usize(i);
        let default_ref = db.add_symbol(idx, "__default__".to_string());
        let ns_ref = db.add_symbol(idx, "__namespace__".to_string());

        let deps: Vec<ImportEdge> = edges
            .iter()
            .filter(|&&(from, _)| from == i)
            .map(|&(_, to)| ImportEdge {
                specifier: CompactString::new(format!("./mod{to}")),
                target: ModuleIdx::from_usize(to),
                is_type: false,
            })
            .collect();

        graph.add_module(Module {
            idx,
            path: PathBuf::from(format!("/mod{i}.js")),
            has_module_syntax: true,
            is_commonjs: false,
            named_exports: FxHashMap::default(),
            named_imports: FxHashMap::default(),
            import_records: Vec::new(),
            default_export_ref: default_ref,
            namespace_object_ref: ns_ref,
            star_export_entries: Vec::new(),
            indirect_export_entries: Vec::new(),
            dependencies: deps,
        });
    }

    graph
}

#[test]
fn test_topo_sort_dag() {
    use oxc_module_graph::topological_sort;

    // A(0) -> B(1) -> C(2)
    //    \--> C(2)
    let graph = simple_graph(&[(0, 1), (0, 2), (1, 2)]);
    let entry = ModuleIdx::from_usize(0);

    let result = topological_sort(&graph, &[entry]);
    assert!(result.is_some(), "DAG should produce valid topo sort");

    let order = result.unwrap();
    assert_eq!(order.len(), 3);

    // C(2) must come after A(0) and B(1) in a valid topo order
    // because A->C and B->C
    let pos = |idx: usize| order.iter().position(|&m| m == ModuleIdx::from_usize(idx)).unwrap();
    assert!(pos(0) < pos(2), "A before C");
    assert!(pos(1) < pos(2), "B before C");
    assert!(pos(0) < pos(1), "A before B");
}

#[test]
fn test_topo_sort_with_cycle() {
    use oxc_module_graph::topological_sort;

    // A(0) -> B(1) -> A(0) (cycle)
    let graph = simple_graph(&[(0, 1), (1, 0)]);
    let entry = ModuleIdx::from_usize(0);

    let result = topological_sort(&graph, &[entry]);
    assert!(result.is_none(), "Cyclic graph should return None");
}

#[test]
fn test_topo_sort_empty() {
    use oxc_module_graph::topological_sort;

    let graph = DefaultModuleGraph::new();
    let result = topological_sort(&graph, &[]);
    assert_eq!(result, Some(Vec::new()));
}

#[test]
fn test_find_cycles_none() {
    use oxc_module_graph::find_cycles;

    // A(0) -> B(1) -> C(2) — no cycles
    let graph = simple_graph(&[(0, 1), (1, 2)]);
    let cycles = find_cycles(&graph);
    assert!(cycles.is_empty(), "DAG should have no cycles");
}

#[test]
fn test_find_cycles_simple() {
    use oxc_module_graph::find_cycles;

    // A(0) -> B(1) -> A(0)
    let graph = simple_graph(&[(0, 1), (1, 0)]);
    let cycles = find_cycles(&graph);
    assert_eq!(cycles.len(), 1, "Should find one cycle");
    assert_eq!(cycles[0].len(), 2, "Cycle should have 2 modules");
}

#[test]
fn test_find_cycles_multiple() {
    use oxc_module_graph::find_cycles;

    // Two independent cycles:
    // A(0) -> B(1) -> A(0)
    // C(2) -> D(3) -> C(2)
    let graph = simple_graph(&[(0, 1), (1, 0), (2, 3), (3, 2)]);
    let cycles = find_cycles(&graph);
    assert_eq!(cycles.len(), 2, "Should find two cycles");
}

#[test]
fn test_find_cycles_self_loop() {
    use oxc_module_graph::find_cycles;

    // A(0) -> A(0) (self-referencing)
    let graph = simple_graph(&[(0, 0)]);
    let cycles = find_cycles(&graph);
    assert_eq!(cycles.len(), 1, "Should find self-loop");
    assert_eq!(cycles[0].len(), 1, "Self-loop is a single-module cycle");
}

// --- Stage 6: match_imports with re-export chain following ---

/// Test: A imports "foo" from B, B re-exports "foo" from C via `export { foo } from './c'`.
/// The chain: A.foo → B.foo → C.foo. match_imports should follow the chain.
#[test]
fn test_match_imports_reexport_chain() {
    use oxc_module_graph::traits::SymbolGraph;
    use oxc_module_graph::{DefaultImportMatcher, build_resolved_exports, match_imports};

    let mut graph = DefaultModuleGraph::new();
    let mut db = SymbolRefDb::new();
    db.ensure_modules(3);

    let idx_a = ModuleIdx::from_usize(0);
    let idx_b = ModuleIdx::from_usize(1);
    let idx_c = ModuleIdx::from_usize(2);

    // Module C: local export "foo"
    let sym_c_foo = db.add_symbol(idx_c, "foo".to_string());
    let sym_c_default = db.add_symbol(idx_c, "__default__".to_string());
    let sym_c_ns = db.add_symbol(idx_c, "__namespace__".to_string());

    let mut exports_c = FxHashMap::default();
    exports_c.insert(
        CompactString::new("foo"),
        LocalExport { exported_name: CompactString::new("foo"), local_symbol: sym_c_foo },
    );

    // Module B: imports "foo" from C and re-exports it via `export { foo } from './c'`
    // To model this: B has a named import for "foo" from C, and a named export "foo" pointing
    // to the same local symbol (which is also a named import).
    let sym_b_foo = db.add_symbol(idx_b, "foo".to_string());
    let sym_b_default = db.add_symbol(idx_b, "__default__".to_string());
    let sym_b_ns = db.add_symbol(idx_b, "__namespace__".to_string());

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
    let sym_a_foo = db.add_symbol(idx_a, "foo".to_string());
    let sym_a_default = db.add_symbol(idx_a, "__default__".to_string());
    let sym_a_ns = db.add_symbol(idx_a, "__namespace__".to_string());

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

    // Add modules
    graph.add_module(Module {
        idx: idx_a,
        path: PathBuf::from("/a.js"),
        has_module_syntax: true,
        is_commonjs: false,
        named_exports: FxHashMap::default(),
        named_imports: named_imports_a,
        import_records: vec![ResolvedImportRecord {
            specifier: CompactString::new("./b"),
            resolved_module: Some(idx_b),
            kind: ImportKind::Static,
        }],
        default_export_ref: sym_a_default,
        namespace_object_ref: sym_a_ns,
        star_export_entries: Vec::new(),
        indirect_export_entries: Vec::new(),
        dependencies: vec![ImportEdge {
            specifier: CompactString::new("./b"),
            target: idx_b,
            is_type: false,
        }],
    });

    graph.add_module(Module {
        idx: idx_b,
        path: PathBuf::from("/b.js"),
        has_module_syntax: true,
        is_commonjs: false,
        named_exports: exports_b,
        named_imports: named_imports_b,
        import_records: vec![ResolvedImportRecord {
            specifier: CompactString::new("./c"),
            resolved_module: Some(idx_c),
            kind: ImportKind::Static,
        }],
        default_export_ref: sym_b_default,
        namespace_object_ref: sym_b_ns,
        star_export_entries: Vec::new(),
        indirect_export_entries: Vec::new(),
        dependencies: vec![ImportEdge {
            specifier: CompactString::new("./c"),
            target: idx_c,
            is_type: false,
        }],
    });

    graph.add_module(Module {
        idx: idx_c,
        path: PathBuf::from("/c.js"),
        has_module_syntax: true,
        is_commonjs: false,
        named_exports: exports_c,
        named_imports: FxHashMap::default(),
        import_records: Vec::new(),
        default_export_ref: sym_c_default,
        namespace_object_ref: sym_c_ns,
        star_export_entries: Vec::new(),
        indirect_export_entries: Vec::new(),
        dependencies: Vec::new(),
    });

    let resolved_exports = build_resolved_exports(&graph);
    let mut matcher = DefaultImportMatcher::default();
    let errors = match_imports(&graph, &mut db, &resolved_exports, &mut matcher);

    assert!(errors.is_empty(), "Expected no errors, got: {errors:?}");

    // A's "foo" should follow the chain through B to C's "foo"
    let canonical = db.canonical_ref_for(sym_a_foo);
    assert_eq!(canonical, sym_c_foo, "A's foo should follow re-export chain to C's foo");
}

/// Test: A imports "bar" from B, B has no export "bar". match_imports returns UnresolvedImport.
#[test]
fn test_match_imports_unresolved() {
    use oxc_module_graph::{DefaultImportMatcher, build_resolved_exports, match_imports};

    let (graph, mut db) = two_module_graph_with_binding("foo", "bar");

    let resolved_exports = build_resolved_exports(&graph);
    let mut matcher = DefaultImportMatcher::default();
    let errors = match_imports(&graph, &mut db, &resolved_exports, &mut matcher);

    assert_eq!(errors.len(), 1, "Expected 1 unresolved import error");
}

/// Test: 3-level re-export chain: A → B → C → D. A imports "x", B re-exports from C,
/// C re-exports from D, D has the local export.
#[test]
fn test_match_imports_deep_chain() {
    use oxc_module_graph::traits::SymbolGraph;
    use oxc_module_graph::{DefaultImportMatcher, build_resolved_exports, match_imports};

    let mut graph = DefaultModuleGraph::new();
    let mut db = SymbolRefDb::new();
    db.ensure_modules(4);

    let idx_a = ModuleIdx::from_usize(0);
    let idx_b = ModuleIdx::from_usize(1);
    let idx_c = ModuleIdx::from_usize(2);
    let idx_d = ModuleIdx::from_usize(3);

    // D: local export "x"
    let sym_d_x = db.add_symbol(idx_d, "x".to_string());
    let sym_d_default = db.add_symbol(idx_d, "__default__".to_string());
    let sym_d_ns = db.add_symbol(idx_d, "__namespace__".to_string());

    let mut exports_d = FxHashMap::default();
    exports_d.insert(
        CompactString::new("x"),
        LocalExport { exported_name: CompactString::new("x"), local_symbol: sym_d_x },
    );

    // C: import "x" from D, re-export as "x"
    let sym_c_x = db.add_symbol(idx_c, "x".to_string());
    let sym_c_default = db.add_symbol(idx_c, "__default__".to_string());
    let sym_c_ns = db.add_symbol(idx_c, "__namespace__".to_string());

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
    let sym_b_x = db.add_symbol(idx_b, "x".to_string());
    let sym_b_default = db.add_symbol(idx_b, "__default__".to_string());
    let sym_b_ns = db.add_symbol(idx_b, "__namespace__".to_string());

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
    let sym_a_x = db.add_symbol(idx_a, "x".to_string());
    let sym_a_default = db.add_symbol(idx_a, "__default__".to_string());
    let sym_a_ns = db.add_symbol(idx_a, "__namespace__".to_string());

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

    // Add all modules
    graph.add_module(Module {
        idx: idx_a,
        path: PathBuf::from("/a.js"),
        has_module_syntax: true,
        is_commonjs: false,
        named_exports: FxHashMap::default(),
        named_imports: named_imports_a,
        import_records: vec![ResolvedImportRecord {
            specifier: CompactString::new("./b"),
            resolved_module: Some(idx_b),
            kind: ImportKind::Static,
        }],
        default_export_ref: sym_a_default,
        namespace_object_ref: sym_a_ns,
        star_export_entries: Vec::new(),
        indirect_export_entries: Vec::new(),
        dependencies: vec![ImportEdge {
            specifier: CompactString::new("./b"),
            target: idx_b,
            is_type: false,
        }],
    });

    graph.add_module(Module {
        idx: idx_b,
        path: PathBuf::from("/b.js"),
        has_module_syntax: true,
        is_commonjs: false,
        named_exports: exports_b,
        named_imports: named_imports_b,
        import_records: vec![ResolvedImportRecord {
            specifier: CompactString::new("./c"),
            resolved_module: Some(idx_c),
            kind: ImportKind::Static,
        }],
        default_export_ref: sym_b_default,
        namespace_object_ref: sym_b_ns,
        star_export_entries: Vec::new(),
        indirect_export_entries: Vec::new(),
        dependencies: vec![ImportEdge {
            specifier: CompactString::new("./c"),
            target: idx_c,
            is_type: false,
        }],
    });

    graph.add_module(Module {
        idx: idx_c,
        path: PathBuf::from("/c.js"),
        has_module_syntax: true,
        is_commonjs: false,
        named_exports: exports_c,
        named_imports: named_imports_c,
        import_records: vec![ResolvedImportRecord {
            specifier: CompactString::new("./d"),
            resolved_module: Some(idx_d),
            kind: ImportKind::Static,
        }],
        default_export_ref: sym_c_default,
        namespace_object_ref: sym_c_ns,
        star_export_entries: Vec::new(),
        indirect_export_entries: Vec::new(),
        dependencies: vec![ImportEdge {
            specifier: CompactString::new("./d"),
            target: idx_d,
            is_type: false,
        }],
    });

    graph.add_module(Module {
        idx: idx_d,
        path: PathBuf::from("/d.js"),
        has_module_syntax: true,
        is_commonjs: false,
        named_exports: exports_d,
        named_imports: FxHashMap::default(),
        import_records: Vec::new(),
        default_export_ref: sym_d_default,
        namespace_object_ref: sym_d_ns,
        star_export_entries: Vec::new(),
        indirect_export_entries: Vec::new(),
        dependencies: Vec::new(),
    });

    let resolved_exports = build_resolved_exports(&graph);
    let mut matcher = DefaultImportMatcher::default();
    let errors = match_imports(&graph, &mut db, &resolved_exports, &mut matcher);

    assert!(errors.is_empty(), "Expected no errors, got: {errors:?}");

    // A's "x" should follow the chain A → B → C → D to D's "x"
    let canonical = db.canonical_ref_for(sym_a_x);
    assert_eq!(canonical, sym_d_x, "A's x should follow 3-level chain to D's x");
}

/// Test: Circular re-export chain. A imports "x" from B, B re-exports "x" from C,
/// C re-exports "x" from B. Should detect cycle and not hang.
#[test]
fn test_match_imports_circular_reexport() {
    use oxc_module_graph::{DefaultImportMatcher, build_resolved_exports, match_imports};

    let mut graph = DefaultModuleGraph::new();
    let mut db = SymbolRefDb::new();
    db.ensure_modules(3);

    let idx_a = ModuleIdx::from_usize(0);
    let idx_b = ModuleIdx::from_usize(1);
    let idx_c = ModuleIdx::from_usize(2);

    // B: import "x" from C, export "x" (which is the import)
    let sym_b_x = db.add_symbol(idx_b, "x".to_string());
    let sym_b_default = db.add_symbol(idx_b, "__default__".to_string());
    let sym_b_ns = db.add_symbol(idx_b, "__namespace__".to_string());

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

    // C: import "x" from B, export "x" (which is the import)
    let sym_c_x = db.add_symbol(idx_c, "x".to_string());
    let sym_c_default = db.add_symbol(idx_c, "__default__".to_string());
    let sym_c_ns = db.add_symbol(idx_c, "__namespace__".to_string());

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
    let sym_a_x = db.add_symbol(idx_a, "x".to_string());
    let sym_a_default = db.add_symbol(idx_a, "__default__".to_string());
    let sym_a_ns = db.add_symbol(idx_a, "__namespace__".to_string());

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

    graph.add_module(Module {
        idx: idx_a,
        path: PathBuf::from("/a.js"),
        has_module_syntax: true,
        is_commonjs: false,
        named_exports: FxHashMap::default(),
        named_imports: named_imports_a,
        import_records: vec![ResolvedImportRecord {
            specifier: CompactString::new("./b"),
            resolved_module: Some(idx_b),
            kind: ImportKind::Static,
        }],
        default_export_ref: sym_a_default,
        namespace_object_ref: sym_a_ns,
        star_export_entries: Vec::new(),
        indirect_export_entries: Vec::new(),
        dependencies: vec![ImportEdge {
            specifier: CompactString::new("./b"),
            target: idx_b,
            is_type: false,
        }],
    });

    graph.add_module(Module {
        idx: idx_b,
        path: PathBuf::from("/b.js"),
        has_module_syntax: true,
        is_commonjs: false,
        named_exports: exports_b,
        named_imports: named_imports_b,
        import_records: vec![ResolvedImportRecord {
            specifier: CompactString::new("./c"),
            resolved_module: Some(idx_c),
            kind: ImportKind::Static,
        }],
        default_export_ref: sym_b_default,
        namespace_object_ref: sym_b_ns,
        star_export_entries: Vec::new(),
        indirect_export_entries: Vec::new(),
        dependencies: vec![ImportEdge {
            specifier: CompactString::new("./c"),
            target: idx_c,
            is_type: false,
        }],
    });

    graph.add_module(Module {
        idx: idx_c,
        path: PathBuf::from("/c.js"),
        has_module_syntax: true,
        is_commonjs: false,
        named_exports: exports_c,
        named_imports: named_imports_c,
        import_records: vec![ResolvedImportRecord {
            specifier: CompactString::new("./b"),
            resolved_module: Some(idx_b),
            kind: ImportKind::Static,
        }],
        default_export_ref: sym_c_default,
        namespace_object_ref: sym_c_ns,
        star_export_entries: Vec::new(),
        indirect_export_entries: Vec::new(),
        dependencies: vec![ImportEdge {
            specifier: CompactString::new("./b"),
            target: idx_b,
            is_type: false,
        }],
    });

    let resolved_exports = build_resolved_exports(&graph);
    let mut matcher = DefaultImportMatcher::default();
    // This should not hang — cycle detection kicks in.
    let _errors = match_imports(&graph, &mut db, &resolved_exports, &mut matcher);

    // A's "x" should remain unlinked (cycle detected).
    let canonical = db.canonical_ref_for(sym_a_x);
    assert_eq!(canonical, sym_a_x, "A's x should remain unlinked due to cycle");
}

/// Test: Custom ImportMatcher that short-circuits for a "cjs" module.
/// NormalAndNamespace results are NOT linked by match_imports — the consumer
/// handles them in on_resolved (e.g., setting namespace_alias in Rolldown).
#[test]
fn test_match_imports_custom_matcher() {
    use std::cell::RefCell;
    use std::rc::Rc;

    use oxc_module_graph::traits::{ImportMatcher, SymbolGraph};
    use oxc_module_graph::{build_resolved_exports, match_imports};

    struct CjsMatcher {
        cjs_module: ModuleIdx,
        cjs_ns_ref: SymbolRef,
        /// Collected NormalAndNamespace results: (local_symbol, namespace_ref, alias)
        ns_alias_results: Rc<RefCell<Vec<(SymbolRef, SymbolRef, CompactString)>>>,
    }

    impl ImportMatcher for CjsMatcher {
        type ModuleIdx = ModuleIdx;
        type SymbolRef = SymbolRef;

        fn on_before_match(
            &mut self,
            _importer_idx: ModuleIdx,
            _record_idx: usize,
            target_idx: ModuleIdx,
            import_name: &str,
            _is_namespace: bool,
        ) -> Option<MatchImportKind<SymbolRef>> {
            if target_idx == self.cjs_module {
                Some(MatchImportKind::NormalAndNamespace {
                    namespace_ref: self.cjs_ns_ref,
                    alias: CompactString::new(import_name),
                })
            } else {
                None
            }
        }

        fn on_resolved(
            &mut self,
            _importer_idx: ModuleIdx,
            local_symbol: SymbolRef,
            resolved: &MatchImportKind<SymbolRef>,
            _reexport_chain: &[SymbolRef],
        ) {
            if let MatchImportKind::NormalAndNamespace { namespace_ref, alias } = resolved {
                self.ns_alias_results.borrow_mut().push((
                    local_symbol,
                    *namespace_ref,
                    alias.clone(),
                ));
            }
        }
    }

    let (graph, mut db) = two_module_graph_with_binding("foo", "foo");
    let idx_b = ModuleIdx::from_usize(1);

    let b_ns_ref = dummy_symbol_ref(idx_b, 2);

    let resolved_exports = build_resolved_exports(&graph);
    let ns_alias_results = Rc::new(RefCell::new(Vec::new()));
    let mut matcher = CjsMatcher {
        cjs_module: idx_b,
        cjs_ns_ref: b_ns_ref,
        ns_alias_results: Rc::clone(&ns_alias_results),
    };
    let errors = match_imports(&graph, &mut db, &resolved_exports, &mut matcher);

    assert!(errors.is_empty(), "Expected no errors, got: {errors:?}");

    // NormalAndNamespace is collected by on_resolved, not linked.
    let results = ns_alias_results.borrow();
    assert_eq!(results.len(), 1, "Expected 1 NormalAndNamespace result");
    let (local, ns_ref, alias) = &results[0];
    let idx_a = ModuleIdx::from_usize(0);
    let a_foo = dummy_symbol_ref(idx_a, 0);
    assert_eq!(*local, a_foo, "Local symbol should be A's foo");
    assert_eq!(*ns_ref, b_ns_ref, "Namespace ref should be B's namespace");
    assert_eq!(alias.as_str(), "foo", "Alias should be 'foo'");

    // A's "foo" should NOT be linked (NormalAndNamespace doesn't link).
    let canonical = db.canonical_ref_for(a_foo);
    assert_eq!(canonical, a_foo, "A's foo should remain unlinked for NormalAndNamespace");
}

/// Test: Namespace import (`import * as ns from './b'`) links to namespace_object_ref.
#[test]
fn test_match_imports_namespace() {
    use oxc_module_graph::traits::SymbolGraph;
    use oxc_module_graph::{DefaultImportMatcher, build_resolved_exports, match_imports};

    let mut graph = DefaultModuleGraph::new();
    let mut db = SymbolRefDb::new();
    db.ensure_modules(2);

    let idx_a = ModuleIdx::from_usize(0);
    let idx_b = ModuleIdx::from_usize(1);

    // Module B: exports "foo"
    let sym_b_foo = db.add_symbol(idx_b, "foo".to_string());
    let sym_b_default = db.add_symbol(idx_b, "__default__".to_string());
    let sym_b_ns = db.add_symbol(idx_b, "__namespace__".to_string());

    let mut exports_b = FxHashMap::default();
    exports_b.insert(
        CompactString::new("foo"),
        LocalExport { exported_name: CompactString::new("foo"), local_symbol: sym_b_foo },
    );

    // Module A: `import * as ns from './b'`
    let sym_a_ns_local = db.add_symbol(idx_a, "ns".to_string());
    let sym_a_default = db.add_symbol(idx_a, "__default__".to_string());
    let sym_a_ns = db.add_symbol(idx_a, "__namespace__".to_string());

    let mut named_imports_a = FxHashMap::default();
    named_imports_a.insert(
        sym_a_ns_local,
        NamedImport {
            imported_name: CompactString::new("*"),
            local_symbol: sym_a_ns_local,
            record_idx: ImportRecordIdx::from_usize(0),
            is_type: false,
        },
    );

    graph.add_module(Module {
        idx: idx_a,
        path: PathBuf::from("/a.js"),
        has_module_syntax: true,
        is_commonjs: false,
        named_exports: FxHashMap::default(),
        named_imports: named_imports_a,
        import_records: vec![ResolvedImportRecord {
            specifier: CompactString::new("./b"),
            resolved_module: Some(idx_b),
            kind: ImportKind::Static,
        }],
        default_export_ref: sym_a_default,
        namespace_object_ref: sym_a_ns,
        star_export_entries: Vec::new(),
        indirect_export_entries: Vec::new(),
        dependencies: vec![ImportEdge {
            specifier: CompactString::new("./b"),
            target: idx_b,
            is_type: false,
        }],
    });

    graph.add_module(Module {
        idx: idx_b,
        path: PathBuf::from("/b.js"),
        has_module_syntax: true,
        is_commonjs: false,
        named_exports: exports_b,
        named_imports: FxHashMap::default(),
        import_records: Vec::new(),
        default_export_ref: sym_b_default,
        namespace_object_ref: sym_b_ns,
        star_export_entries: Vec::new(),
        indirect_export_entries: Vec::new(),
        dependencies: Vec::new(),
    });

    let resolved_exports = build_resolved_exports(&graph);
    let mut matcher = DefaultImportMatcher::default();
    let errors = match_imports(&graph, &mut db, &resolved_exports, &mut matcher);

    assert!(errors.is_empty(), "Expected no errors, got: {errors:?}");

    // A's namespace import should link to B's namespace object ref.
    let canonical = db.canonical_ref_for(sym_a_ns_local);
    assert_eq!(canonical, sym_b_ns, "Namespace import should link to B's namespace_object_ref");
}

/// Test: on_resolved callback is called with the re-export chain.
#[test]
fn test_match_imports_on_resolved_chain() {
    use std::cell::RefCell;
    use std::rc::Rc;

    use oxc_module_graph::traits::ImportMatcher;
    use oxc_module_graph::{build_resolved_exports, match_imports};

    struct ChainTracker {
        chains: Rc<RefCell<Vec<Vec<SymbolRef>>>>,
    }

    impl ImportMatcher for ChainTracker {
        type ModuleIdx = ModuleIdx;
        type SymbolRef = SymbolRef;

        fn on_resolved(
            &mut self,
            _importer_idx: ModuleIdx,
            _local_symbol: SymbolRef,
            _resolved: &MatchImportKind<SymbolRef>,
            reexport_chain: &[SymbolRef],
        ) {
            if !reexport_chain.is_empty() {
                self.chains.borrow_mut().push(reexport_chain.to_vec());
            }
        }
    }

    let mut graph = DefaultModuleGraph::new();
    let mut db = SymbolRefDb::new();
    db.ensure_modules(3);

    let idx_a = ModuleIdx::from_usize(0);
    let idx_b = ModuleIdx::from_usize(1);
    let idx_c = ModuleIdx::from_usize(2);

    // C: local export "val"
    let sym_c_val = db.add_symbol(idx_c, "val".to_string());
    let sym_c_default = db.add_symbol(idx_c, "__default__".to_string());
    let sym_c_ns = db.add_symbol(idx_c, "__namespace__".to_string());

    let mut exports_c = FxHashMap::default();
    exports_c.insert(
        CompactString::new("val"),
        LocalExport { exported_name: CompactString::new("val"), local_symbol: sym_c_val },
    );

    // B: import "val" from C, re-export "val"
    let sym_b_val = db.add_symbol(idx_b, "val".to_string());
    let sym_b_default = db.add_symbol(idx_b, "__default__".to_string());
    let sym_b_ns = db.add_symbol(idx_b, "__namespace__".to_string());

    let mut named_imports_b = FxHashMap::default();
    named_imports_b.insert(
        sym_b_val,
        NamedImport {
            imported_name: CompactString::new("val"),
            local_symbol: sym_b_val,
            record_idx: ImportRecordIdx::from_usize(0),
            is_type: false,
        },
    );
    let mut exports_b = FxHashMap::default();
    exports_b.insert(
        CompactString::new("val"),
        LocalExport { exported_name: CompactString::new("val"), local_symbol: sym_b_val },
    );

    // A: import "val" from B
    let sym_a_val = db.add_symbol(idx_a, "val".to_string());
    let sym_a_default = db.add_symbol(idx_a, "__default__".to_string());
    let sym_a_ns = db.add_symbol(idx_a, "__namespace__".to_string());

    let mut named_imports_a = FxHashMap::default();
    named_imports_a.insert(
        sym_a_val,
        NamedImport {
            imported_name: CompactString::new("val"),
            local_symbol: sym_a_val,
            record_idx: ImportRecordIdx::from_usize(0),
            is_type: false,
        },
    );

    graph.add_module(Module {
        idx: idx_a,
        path: PathBuf::from("/a.js"),
        has_module_syntax: true,
        is_commonjs: false,
        named_exports: FxHashMap::default(),
        named_imports: named_imports_a,
        import_records: vec![ResolvedImportRecord {
            specifier: CompactString::new("./b"),
            resolved_module: Some(idx_b),
            kind: ImportKind::Static,
        }],
        default_export_ref: sym_a_default,
        namespace_object_ref: sym_a_ns,
        star_export_entries: Vec::new(),
        indirect_export_entries: Vec::new(),
        dependencies: vec![ImportEdge {
            specifier: CompactString::new("./b"),
            target: idx_b,
            is_type: false,
        }],
    });

    graph.add_module(Module {
        idx: idx_b,
        path: PathBuf::from("/b.js"),
        has_module_syntax: true,
        is_commonjs: false,
        named_exports: exports_b,
        named_imports: named_imports_b,
        import_records: vec![ResolvedImportRecord {
            specifier: CompactString::new("./c"),
            resolved_module: Some(idx_c),
            kind: ImportKind::Static,
        }],
        default_export_ref: sym_b_default,
        namespace_object_ref: sym_b_ns,
        star_export_entries: Vec::new(),
        indirect_export_entries: Vec::new(),
        dependencies: vec![ImportEdge {
            specifier: CompactString::new("./c"),
            target: idx_c,
            is_type: false,
        }],
    });

    graph.add_module(Module {
        idx: idx_c,
        path: PathBuf::from("/c.js"),
        has_module_syntax: true,
        is_commonjs: false,
        named_exports: exports_c,
        named_imports: FxHashMap::default(),
        import_records: Vec::new(),
        default_export_ref: sym_c_default,
        namespace_object_ref: sym_c_ns,
        star_export_entries: Vec::new(),
        indirect_export_entries: Vec::new(),
        dependencies: Vec::new(),
    });

    let resolved_exports = build_resolved_exports(&graph);
    let chains = Rc::new(RefCell::new(Vec::new()));
    let mut tracker = ChainTracker { chains: Rc::clone(&chains) };
    let errors = match_imports(&graph, &mut db, &resolved_exports, &mut tracker);

    assert!(errors.is_empty(), "Expected no errors, got: {errors:?}");

    let chains = chains.borrow();
    assert_eq!(chains.len(), 1, "Expected 1 re-export chain (A→B→C)");
    assert_eq!(chains[0], vec![sym_b_val], "Chain should contain B's val symbol");
}

// ── dynamic exports tests ──────────────────────────────────────────────────

#[test]
fn test_dynamic_exports_cjs_module() {
    // CJS module → dynamic
    use oxc_module_graph::compute_has_dynamic_exports;

    let mut graph = DefaultModuleGraph::new();
    let idx = ModuleIdx::from_usize(0);

    graph.add_module(Module {
        idx,
        path: PathBuf::from("/cjs.js"),
        has_module_syntax: false,
        is_commonjs: true,
        named_exports: FxHashMap::default(),
        named_imports: FxHashMap::default(),
        import_records: Vec::new(),
        default_export_ref: dummy_symbol_ref(idx, 0),
        namespace_object_ref: dummy_symbol_ref(idx, 1),
        star_export_entries: Vec::new(),
        indirect_export_entries: Vec::new(),
        dependencies: Vec::new(),
    });

    let dynamic = compute_has_dynamic_exports(&graph);
    assert!(dynamic.contains(&idx), "CJS module should have dynamic exports");
}

#[test]
fn test_dynamic_exports_external_star_target() {
    // ESM with export * from external (not in store) → dynamic
    use oxc_module_graph::compute_has_dynamic_exports;
    use oxc_module_graph::types::StarExportEntry;

    let mut graph = DefaultModuleGraph::new();
    let idx_a = ModuleIdx::from_usize(0);
    let idx_external = ModuleIdx::from_usize(1); // not added to graph

    graph.add_module(Module {
        idx: idx_a,
        path: PathBuf::from("/a.js"),
        has_module_syntax: true,
        is_commonjs: false,
        named_exports: FxHashMap::default(),
        named_imports: FxHashMap::default(),
        import_records: Vec::new(),
        default_export_ref: dummy_symbol_ref(idx_a, 0),
        namespace_object_ref: dummy_symbol_ref(idx_a, 1),
        star_export_entries: vec![StarExportEntry {
            module_request: CompactString::new("external"),
            resolved_module: Some(idx_external),
            span: oxc_span::Span::default(),
        }],
        indirect_export_entries: Vec::new(),
        dependencies: Vec::new(),
    });

    let dynamic = compute_has_dynamic_exports(&graph);
    assert!(
        dynamic.contains(&idx_a),
        "Module with export * from external should have dynamic exports"
    );
}

#[test]
fn test_dynamic_exports_transitive_from_cjs() {
    // ESM export * from CJS → dynamic (transitive)
    use oxc_module_graph::compute_has_dynamic_exports;
    use oxc_module_graph::types::StarExportEntry;

    let mut graph = DefaultModuleGraph::new();
    let idx_esm = ModuleIdx::from_usize(0);
    let idx_cjs = ModuleIdx::from_usize(1);

    graph.add_module(Module {
        idx: idx_esm,
        path: PathBuf::from("/esm.js"),
        has_module_syntax: true,
        is_commonjs: false,
        named_exports: FxHashMap::default(),
        named_imports: FxHashMap::default(),
        import_records: Vec::new(),
        default_export_ref: dummy_symbol_ref(idx_esm, 0),
        namespace_object_ref: dummy_symbol_ref(idx_esm, 1),
        star_export_entries: vec![StarExportEntry {
            module_request: CompactString::new("./cjs"),
            resolved_module: Some(idx_cjs),
            span: oxc_span::Span::default(),
        }],
        indirect_export_entries: Vec::new(),
        dependencies: vec![ImportEdge {
            specifier: CompactString::new("./cjs"),
            target: idx_cjs,
            is_type: false,
        }],
    });

    graph.add_module(Module {
        idx: idx_cjs,
        path: PathBuf::from("/cjs.js"),
        has_module_syntax: false,
        is_commonjs: true,
        named_exports: FxHashMap::default(),
        named_imports: FxHashMap::default(),
        import_records: Vec::new(),
        default_export_ref: dummy_symbol_ref(idx_cjs, 2),
        namespace_object_ref: dummy_symbol_ref(idx_cjs, 3),
        star_export_entries: Vec::new(),
        indirect_export_entries: Vec::new(),
        dependencies: Vec::new(),
    });

    let dynamic = compute_has_dynamic_exports(&graph);
    assert!(dynamic.contains(&idx_esm), "ESM with export * from CJS should have dynamic exports");
    assert!(dynamic.contains(&idx_cjs), "CJS module itself should have dynamic exports");
}

#[test]
fn test_dynamic_exports_pure_esm_not_dynamic() {
    // Pure ESM → not dynamic
    use oxc_module_graph::compute_has_dynamic_exports;
    use oxc_module_graph::types::StarExportEntry;

    let mut graph = DefaultModuleGraph::new();
    let idx_a = ModuleIdx::from_usize(0);
    let idx_b = ModuleIdx::from_usize(1);

    let mut exports_b = FxHashMap::default();
    exports_b.insert(
        CompactString::new("foo"),
        LocalExport {
            exported_name: CompactString::new("foo"),
            local_symbol: dummy_symbol_ref(idx_b, 0),
        },
    );

    graph.add_module(Module {
        idx: idx_a,
        path: PathBuf::from("/a.js"),
        has_module_syntax: true,
        is_commonjs: false,
        named_exports: FxHashMap::default(),
        named_imports: FxHashMap::default(),
        import_records: Vec::new(),
        default_export_ref: dummy_symbol_ref(idx_a, 0),
        namespace_object_ref: dummy_symbol_ref(idx_a, 1),
        star_export_entries: vec![StarExportEntry {
            module_request: CompactString::new("./b"),
            resolved_module: Some(idx_b),
            span: oxc_span::Span::default(),
        }],
        indirect_export_entries: Vec::new(),
        dependencies: vec![ImportEdge {
            specifier: CompactString::new("./b"),
            target: idx_b,
            is_type: false,
        }],
    });

    graph.add_module(Module {
        idx: idx_b,
        path: PathBuf::from("/b.js"),
        has_module_syntax: true,
        is_commonjs: false,
        named_exports: exports_b,
        named_imports: FxHashMap::default(),
        import_records: Vec::new(),
        default_export_ref: dummy_symbol_ref(idx_b, 1),
        namespace_object_ref: dummy_symbol_ref(idx_b, 2),
        star_export_entries: Vec::new(),
        indirect_export_entries: Vec::new(),
        dependencies: Vec::new(),
    });

    let dynamic = compute_has_dynamic_exports(&graph);
    assert!(dynamic.is_empty(), "Pure ESM modules should not have dynamic exports");
}

#[test]
fn test_dynamic_exports_cycle_no_infinite_loop() {
    // Cycle: A export* from B, B export* from A — should not infinite loop
    use oxc_module_graph::compute_has_dynamic_exports;
    use oxc_module_graph::types::StarExportEntry;

    let mut graph = DefaultModuleGraph::new();
    let idx_a = ModuleIdx::from_usize(0);
    let idx_b = ModuleIdx::from_usize(1);

    graph.add_module(Module {
        idx: idx_a,
        path: PathBuf::from("/a.js"),
        has_module_syntax: true,
        is_commonjs: false,
        named_exports: FxHashMap::default(),
        named_imports: FxHashMap::default(),
        import_records: Vec::new(),
        default_export_ref: dummy_symbol_ref(idx_a, 0),
        namespace_object_ref: dummy_symbol_ref(idx_a, 1),
        star_export_entries: vec![StarExportEntry {
            module_request: CompactString::new("./b"),
            resolved_module: Some(idx_b),
            span: oxc_span::Span::default(),
        }],
        indirect_export_entries: Vec::new(),
        dependencies: vec![ImportEdge {
            specifier: CompactString::new("./b"),
            target: idx_b,
            is_type: false,
        }],
    });

    graph.add_module(Module {
        idx: idx_b,
        path: PathBuf::from("/b.js"),
        has_module_syntax: true,
        is_commonjs: false,
        named_exports: FxHashMap::default(),
        named_imports: FxHashMap::default(),
        import_records: Vec::new(),
        default_export_ref: dummy_symbol_ref(idx_b, 2),
        namespace_object_ref: dummy_symbol_ref(idx_b, 3),
        star_export_entries: vec![StarExportEntry {
            module_request: CompactString::new("./a"),
            resolved_module: Some(idx_a),
            span: oxc_span::Span::default(),
        }],
        indirect_export_entries: Vec::new(),
        dependencies: vec![ImportEdge {
            specifier: CompactString::new("./a"),
            target: idx_a,
            is_type: false,
        }],
    });

    // Should complete without hanging; pure ESM cycle → not dynamic
    let dynamic = compute_has_dynamic_exports(&graph);
    assert!(dynamic.is_empty(), "Pure ESM cycle should not have dynamic exports");
}
