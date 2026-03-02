use compact_str::CompactString;
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use oxc_module_graph::default::SymbolRefDb;
use oxc_module_graph::types::{ModuleIdx, SymbolRef};
use oxc_syntax::symbol::SymbolId;

fn setup_chain(db: &mut SymbolRefDb, module: ModuleIdx, depth: usize) -> SymbolRef {
    db.ensure_module_symbol_capacity(module, depth);
    // Create a chain: sym[0] → sym[1] → ... → sym[depth-1] (root)
    for i in (0..depth - 1).rev() {
        let from = SymbolRef::new(module, SymbolId::from_usize(i));
        let to = SymbolRef::new(module, SymbolId::from_usize(i + 1));
        db.link(from, to);
    }
    SymbolRef::new(module, SymbolId::from_usize(0))
}

fn bench_canonical_ref(c: &mut Criterion) {
    let mut group = c.benchmark_group("canonical_ref");

    for depth in [1, 5, 10, 50] {
        group.bench_with_input(BenchmarkId::new("immutable", depth), &depth, |b, &depth| {
            let mut db = SymbolRefDb::new();
            let module = ModuleIdx::from_usize(0);
            let start = setup_chain(&mut db, module, depth);
            b.iter(|| db.canonical_ref_for(start));
        });

        group.bench_with_input(BenchmarkId::new("mut_path_halving", depth), &depth, |b, &depth| {
            let mut db = SymbolRefDb::new();
            let module = ModuleIdx::from_usize(0);
            let start = setup_chain(&mut db, module, depth);
            b.iter(|| db.canonical_ref_for_mut(start));
        });
    }

    group.finish();
}

fn bench_link(c: &mut Criterion) {
    c.bench_function("link_500k", |b| {
        b.iter_with_setup(
            || {
                let mut db = SymbolRefDb::new();
                let module_count = 1000;
                let symbols_per_module = 500;
                db.ensure_modules(module_count);
                for m in 0..module_count {
                    let idx = ModuleIdx::from_usize(m);
                    for s in 0..symbols_per_module {
                        db.add_symbol(idx, format!("sym_{m}_{s}"));
                    }
                }
                db
            },
            |mut db| {
                // Link each symbol to the next within same module
                for m in 0..1000 {
                    let idx = ModuleIdx::from_usize(m);
                    for s in 0..499 {
                        let from = SymbolRef::new(idx, SymbolId::from_usize(s));
                        let to = SymbolRef::new(idx, SymbolId::from_usize(s + 1));
                        db.link(from, to);
                    }
                }
            },
        );
    });
}

fn bench_flatten(c: &mut Criterion) {
    c.bench_function("flatten_500k", |b| {
        b.iter_with_setup(
            || {
                let mut db = SymbolRefDb::new();
                let module_count = 1000;
                let symbols_per_module = 500;
                db.ensure_modules(module_count);
                for m in 0..module_count {
                    let idx = ModuleIdx::from_usize(m);
                    for s in 0..symbols_per_module {
                        db.add_symbol(idx, format!("sym_{m}_{s}"));
                    }
                }
                // Create chains within each module
                for m in 0..module_count {
                    let idx = ModuleIdx::from_usize(m);
                    for s in 0..symbols_per_module - 1 {
                        let from = SymbolRef::new(idx, SymbolId::from_usize(s));
                        let to = SymbolRef::new(idx, SymbolId::from_usize(s + 1));
                        db.link(from, to);
                    }
                }
                db
            },
            |mut db| {
                db.flatten_all_chains();
            },
        );
    });
}

fn bench_init_module_symbols(c: &mut Criterion) {
    c.bench_function("init_module_symbols_1000", |b| {
        b.iter_with_setup(
            || {
                let names: Vec<CompactString> =
                    (0..1000).map(|i| CompactString::new(format!("sym_{i}"))).collect();
                (SymbolRefDb::new(), names)
            },
            |(mut db, names)| {
                let idx = ModuleIdx::from_usize(0);
                db.init_module_symbols(idx, &names);
            },
        );
    });
}

criterion_group!(
    benches,
    bench_canonical_ref,
    bench_link,
    bench_flatten,
    bench_init_module_symbols,
);
criterion_main!(benches);
