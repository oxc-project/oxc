//! Criterion benchmarks for the `oxc_react_compiler` crate.
//!
//! Run with:
//! ```bash
//! cargo bench -p oxc_react_compiler
//! ```
//!
//! These benches measure aggregate performance of the compilation pipeline.
//! They are NOT correctness tests; the fixture suite remains the source of
//! truth for behavior. Treat regressions in the absolute numbers as
//! investigation triggers, not as test failures.
//!
//! Benchmarks:
//!   * `Environment::clone` — clone overhead for the shared `Environment` struct.
//!   * `compile_50_components` — end-to-end lower + pipeline over a synthetic
//!     50-component source file. Captures aggregate wins from earlier phases.
//!   * `compile_big_component` — lower + pipeline over a single synthetic
//!     function with ~100 statements. Stresses the validation/analysis walks.

#![allow(clippy::print_stdout, clippy::print_stderr, clippy::disallowed_methods)]

use std::fmt::Write;

use criterion::{Criterion, black_box, criterion_group, criterion_main};

use oxc_react_compiler::entrypoint::imports::ProgramContext;
use oxc_react_compiler::entrypoint::pipeline::run_pipeline;
use oxc_react_compiler::hir::ReactFunctionType;
use oxc_react_compiler::hir::build_hir::{LowerableFunction, collect_import_bindings, lower};
use oxc_react_compiler::hir::environment::{CompilerOutputMode, Environment, EnvironmentConfig};

/// Build the source text for a synthetic file containing `n` simple React-style
/// function components. Each component has slightly different body content to
/// avoid any accidental cache-friendliness.
fn make_multi_component_source(n: usize) -> String {
    let mut out = String::with_capacity(n * 64);
    for i in 0..n {
        let _ = writeln!(
            out,
            "function Component{i}(props) {{ return <div className={{props.cls}}>{{props.x + {i}}}</div>; }}",
        );
    }
    out
}

/// Build a single function with many local-variable statements followed by a JSX
/// return. Designed to stretch HIR-build, type inference and the validation
/// walks without exercising more exotic JS features.
fn make_big_component_source(n: usize) -> String {
    let mut body = String::with_capacity(n * 32);
    body.push_str("function BigComponent(props) {\n");
    body.push_str("  let x0 = props.start | 0;\n");
    for i in 1..n {
        let _ = writeln!(body, "  let x{i} = x{prev} + {i};", prev = i - 1);
    }
    let _ = writeln!(body, "  return <div>{{x{last}}}</div>;", last = n - 1);
    body.push_str("}\n");
    body
}

/// Parse `source` and run the React-compiler analysis pipeline over every
/// top-level function declaration. Mirrors `tests/fixtures.rs::run_pipeline_on_source`
/// but iterates over every function and is panic-on-error so failures surface
/// loudly in benches.
fn compile_all_functions(source: &str) {
    let allocator = oxc_allocator::Allocator::default();
    let source_type = oxc_span::SourceType::jsx();
    let parser_result = oxc_parser::Parser::new(&allocator, source, source_type).parse();
    assert!(parser_result.errors.is_empty(), "parse failed: {:?}", parser_result.errors);

    let outer_bindings = collect_import_bindings(&parser_result.program.body);

    let env = Environment::new(
        ReactFunctionType::Component,
        CompilerOutputMode::Client,
        EnvironmentConfig::default(),
    )
    .unwrap();

    for stmt in &parser_result.program.body {
        let oxc_ast::ast::Statement::FunctionDeclaration(func) = stmt else { continue };
        let lowerable = LowerableFunction::Function(func);
        let mut hir_func =
            lower(&env, ReactFunctionType::Component, &lowerable, outer_bindings.clone())
                .expect("lower failed");
        // Seed a fresh `ProgramContext` per function — matches the fixture
        // harness pattern (`tests/fixtures.rs::run_pipeline_on_source`) and
        // is required since Phase 8 because pipeline passes (notably
        // `LowerContextAccess`) register imports through this context.
        // The bench source is plain JSX with no `useContext`, so the
        // context stays empty in practice; we still construct it so the
        // bench reflects the real pipeline cost (incl. context plumbing)
        // and so the call typechecks.
        let mut program_context = ProgramContext::new();
        run_pipeline(&mut hir_func, &env, &mut program_context).expect("pipeline failed");
    }
}

fn bench_environment_clone(c: &mut Criterion) {
    let env = Environment::new(
        ReactFunctionType::Component,
        CompilerOutputMode::Client,
        EnvironmentConfig::default(),
    )
    .unwrap();

    c.bench_function("Environment::clone", |b| {
        b.iter(|| black_box(env.clone()));
    });
}

fn bench_compile_50_components(c: &mut Criterion) {
    let source = make_multi_component_source(50);
    c.bench_function("compile_50_components", |b| {
        b.iter(|| compile_all_functions(black_box(&source)));
    });
}

fn bench_validation_dispatcher(c: &mut Criterion) {
    let source = make_big_component_source(100);
    c.bench_function("compile_big_component", |b| {
        b.iter(|| compile_all_functions(black_box(&source)));
    });
}

criterion_group!(
    benches,
    bench_environment_clone,
    bench_compile_50_components,
    bench_validation_dispatcher,
);
criterion_main!(benches);
