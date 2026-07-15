use std::{
    fmt::Write as _,
    fs::File,
    io::{self, Write},
};

use humansize::{DECIMAL, format_size};
use mimalloc_safe::MiMalloc;

use oxc_allocator::Allocator;
use oxc_formatter::{JsFormatOptions, format_program, parse_for_format};
use oxc_minifier::{CompressOptions, MangleOptions, Minifier, MinifierOptions};
use oxc_parser::{ParseOptions, Parser};
use oxc_semantic::SemanticBuilder;
use oxc_tasks_common::{TestFile, TestFiles, project_root};
use oxc_transformer::{TransformOptions, Transformer};

use std::alloc::{GlobalAlloc, Layout};
use std::sync::atomic::{AtomicUsize, Ordering::SeqCst};

#[global_allocator]
static GLOBAL: TrackedAllocator = TrackedAllocator;

struct TrackedAllocator;

/// Atomic counter.
///
/// Mainly just a wrapper around `AtomicUsize`, but its methods saturate at `usize::MAX`
/// instead of wrapping around.
/// This is practically infeasible on 64-bit systems, but might just be possible on 32-bit.
///
/// Note: `SeqCst` ordering may be stronger than required, but performance is not the primary concern here,
/// so play it safe.
struct AtomicCounter(AtomicUsize);

impl AtomicCounter {
    const fn new() -> Self {
        Self(AtomicUsize::new(0))
    }

    fn get(&self) -> usize {
        self.0.load(SeqCst)
    }

    fn increment(&self) {
        self.0.update(SeqCst, SeqCst, |count| count.saturating_add(1));
    }

    fn reset(&self) {
        self.0.store(0, SeqCst);
    }
}

/// Number of system allocations
static NUM_ALLOC: AtomicCounter = AtomicCounter::new();
/// Number of system reallocations
static NUM_REALLOC: AtomicCounter = AtomicCounter::new();

fn reset_global_allocs() {
    NUM_ALLOC.reset();
    NUM_REALLOC.reset();
}

// SAFETY: Methods simply delegate to `MiMalloc` allocator to ensure that the allocator
// is the same across different platforms for the purposes of tracking allocations.
#[expect(clippy::undocumented_unsafe_blocks)]
unsafe impl GlobalAlloc for TrackedAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ret = unsafe { MiMalloc.alloc(layout) };
        if !ret.is_null() {
            NUM_ALLOC.increment();
        }
        ret
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { MiMalloc.dealloc(ptr, layout) };
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let ret = unsafe { MiMalloc.alloc_zeroed(layout) };
        if !ret.is_null() {
            NUM_ALLOC.increment();
        }
        ret
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let ret = unsafe { MiMalloc.realloc(ptr, layout, new_size) };
        if !ret.is_null() {
            NUM_REALLOC.increment();
        }
        ret
    }
}

/// Stores all of the memory allocation stats that will be printed for each file.
#[derive(Debug)]
struct StageStats {
    /// Deltas of the allocation counters across the measured operation
    counters: AllocatorStats,
    /// Bytes used in the measured `Allocator`'s arena at the end of the measured operation.
    /// A point-in-time gauge read after the operation completes, not a diffed counter.
    /// The arena is not reset between stages, so this includes content from earlier stages
    /// (e.g. the AST the parser built).
    arena_used_bytes: usize,
}

/// Counters of allocations, captured from the global and arena allocators.
/// Used both as a raw snapshot and as per-operation deltas (see `record_stats_diff`).
#[derive(Debug)]
struct AllocatorStats {
    /// Number of allocations made by system allocator, excluding arena chunk allocations
    sys_allocs: usize,
    /// Number of reallocations made by system allocator
    sys_reallocs: usize,
    /// Number of chunks arenas have requested from the system allocator.
    /// Tracked separately so chunk allocations can be excluded from `sys_allocs`
    /// (see `record_stats_diff`). Not printed in snapshots because the count is platform-dependent.
    arena_chunk_allocs: usize,
    /// Number of allocations made by arena allocator
    arena_allocs: usize,
    /// Number of reallocations made by arena allocator
    arena_reallocs: usize,
}

#[test]
#[cfg(any(coverage, coverage_nightly))]
fn test() {
    run().unwrap();
}

/// # Panics
/// # Errors
pub fn run() -> Result<(), io::Error> {
    let files = TestFiles::complicated();

    let mut parser_out = String::new();
    let mut semantic_out = String::new();
    let mut transformer_out = String::new();
    let mut minifier_out = String::new();
    let mut formatter_out = String::new();

    let mut allocator = Allocator::default();

    let parse_options = ParseOptions { parse_regular_expression: true, ..ParseOptions::default() };
    let minifier_options = MinifierOptions {
        mangle: Some(MangleOptions::default()),
        compress: Some(CompressOptions::smallest()),
    };

    // Warm-up by parsing each file first, and then measuring the actual allocations. This reduces variance
    // in the number of allocations, because we ensure that the arena allocator has already requested all
    // of the space it will need from the system allocator to parse the largest file in the set.
    for file in files.files() {
        let mut parsed = Parser::new(&allocator, &file.source_text, file.source_type)
            .with_options(parse_options)
            .parse();
        assert!(parsed.diagnostics.is_empty());

        // Transform TypeScript to ESNext before minifying (minifier only works on esnext)
        let scoping = SemanticBuilder::new()
            .with_excess_capacity(2.0)
            .with_enum_eval(true)
            .build(&parsed.program)
            .semantic
            .into_scoping();
        let transform_options = TransformOptions::from_target("esnext").unwrap();
        let _ =
            Transformer::new(&allocator, std::path::Path::new(&file.file_name), &transform_options)
                .build_with_scoping(scoping, &mut parsed.program);

        Minifier::new(minifier_options.clone()).minify(&allocator, &mut parsed.program);

        // Formatter runs on a freshly-parsed AST (not after transformer/minifier),
        // so re-parse with the formatter's parse options before formatting
        allocator.reset();
        let parsed = parse_for_format(&allocator, &file.source_text, file.source_type);
        assert!(parsed.diagnostics.is_empty());
        let _ = format_program(&allocator, &parsed.program, JsFormatOptions::default(), None)
            .print()
            .unwrap()
            .into_code();
    }

    for file in files.files() {
        let minifier_options = minifier_options.clone();

        allocator.reset();
        reset_global_allocs();

        let (mut parsed, parser_stats) = record_stats_in(&allocator, || {
            let parsed = Parser::new(&allocator, &file.source_text, file.source_type)
                .with_options(parse_options)
                .parse();
            assert!(parsed.diagnostics.is_empty());
            parsed
        });

        parser_out.push_str(&format_stats(file, &parser_stats));

        let ((), semantic_stats) = record_stats_in(&allocator, || {
            let _ = SemanticBuilder::new().with_enum_eval(true).build(&parsed.program);
        });

        semantic_out.push_str(&format_stats(file, &semantic_stats));

        // Match the production compiler path for transforms: transformers add scopes, symbols, and
        // references, so semantic analysis reserves excess capacity up front.
        let scoping = SemanticBuilder::new()
            .with_excess_capacity(2.0)
            .with_enum_eval(true)
            .build(&parsed.program)
            .semantic
            .into_scoping();

        // Transform TypeScript to ESNext before minifying (minifier only works on esnext)
        let transform_options = TransformOptions::from_target("esnext").unwrap();
        let ((), transformer_stats) = record_stats_in(&allocator, || {
            let _ = Transformer::new(
                &allocator,
                std::path::Path::new(&file.file_name),
                &transform_options,
            )
            .build_with_scoping(scoping, &mut parsed.program);
        });

        transformer_out.push_str(&format_stats(file, &transformer_stats));

        let ((), minifier_stats) = record_stats_in(&allocator, || {
            Minifier::new(minifier_options).minify(&allocator, &mut parsed.program);
        });

        minifier_out.push_str(&format_stats(file, &minifier_stats));

        // Formatter runs on a freshly-parsed AST (not after transformer/minifier),
        // so re-parse with the formatter's parse options before measuring the formatter
        allocator.reset();
        reset_global_allocs();

        let parsed = parse_for_format(&allocator, &file.source_text, file.source_type);
        assert!(parsed.diagnostics.is_empty());

        let (_, formatter_stats) = record_stats_in(&allocator, || {
            format_program(&allocator, &parsed.program, JsFormatOptions::default(), None)
                .print()
                .unwrap()
                .into_code()
        });

        formatter_out.push_str(&format_stats(file, &formatter_stats));
    }

    write_snapshot("tasks/track_memory_allocations/allocs_parser.snap", &parser_out)?;
    write_snapshot("tasks/track_memory_allocations/allocs_semantic.snap", &semantic_out)?;
    write_snapshot("tasks/track_memory_allocations/allocs_transformer.snap", &transformer_out)?;
    write_snapshot("tasks/track_memory_allocations/allocs_minifier.snap", &minifier_out)?;
    write_snapshot("tasks/track_memory_allocations/allocs_formatter.snap", &formatter_out)?;

    Ok(())
}

/// Record current allocation stats from both system allocator and arena allocator.
#[cfg_attr(feature = "is_all_features", expect(unused))]
fn record_stats(allocator: &Allocator) -> AllocatorStats {
    let sys_allocs = NUM_ALLOC.get();
    let sys_reallocs = NUM_REALLOC.get();
    #[cfg(not(feature = "is_all_features"))]
    let ((arena_allocs, arena_reallocs), (arena_chunk_allocs, _arena_chunk_alloc_bytes)) =
        (allocator.get_allocation_stats(), Allocator::global_chunk_allocation_stats());
    #[cfg(feature = "is_all_features")]
    let ((arena_allocs, arena_reallocs), (arena_chunk_allocs, _arena_chunk_alloc_bytes)) =
        ((0, 0), (0, 0));

    AllocatorStats { sys_allocs, sys_reallocs, arena_chunk_allocs, arena_allocs, arena_reallocs }
}

/// Record current allocation stats since the last recorded stats in `prev`. This is useful
/// for measuring allocations made during a specific operation without needing to reset the stats.
fn record_stats_diff(allocator: &Allocator, prev: &AllocatorStats) -> AllocatorStats {
    let stats = record_stats(allocator);
    // Arena chunk allocations go through the system allocator, so they are included in `sys_allocs`.
    // Exclude them. Whether an arena needs one more chunk depends on the total number of bytes
    // allocated in the arena, and that byte total varies across platforms with target type layout
    // and alignment (e.g. hashbrown tables are wider on x86_64 than aarch64). Counting chunk
    // requests therefore made snapshots platform-dependent whenever an arena's content size sat
    // close to a chunk boundary on one platform (see #22621 for a previous instance).
    // All other allocation classes grow on element counts, which are identical on all platforms.
    let arena_chunk_allocs = stats.arena_chunk_allocs.saturating_sub(prev.arena_chunk_allocs);
    AllocatorStats {
        sys_allocs: stats
            .sys_allocs
            .saturating_sub(prev.sys_allocs)
            .saturating_sub(arena_chunk_allocs),
        sys_reallocs: stats.sys_reallocs.saturating_sub(prev.sys_reallocs),
        arena_chunk_allocs,
        arena_allocs: stats.arena_allocs.saturating_sub(prev.arena_allocs),
        arena_reallocs: stats.arena_reallocs.saturating_sub(prev.arena_reallocs),
    }
}

/// Records the allocations stats before and after the given closure is executed.
fn record_stats_in<F, R>(allocator: &Allocator, f: F) -> (R, StageStats)
where
    F: FnOnce() -> R,
{
    let before_stats = record_stats(allocator);
    let result = f();
    let counters = record_stats_diff(allocator, &before_stats);
    let stats = StageStats { counters, arena_used_bytes: allocator.used_bytes() };

    (result, stats)
}

/// Formats the allocator stats for one file as a block of `label: value` lines.
///
/// One value per line, with no column alignment, so that a change to one value produces
/// a one-line diff, and adding a new value later doesn't reformat existing lines.
/// File names stay at column 0 so they appear in git hunk headers.
fn format_stats(file: &TestFile, stats: &StageStats) -> String {
    let counters = &stats.counters;
    let values = [
        ("file size", format_size(file.source_text.len(), DECIMAL)),
        ("sys allocs", counters.sys_allocs.to_string()),
        ("sys reallocs", counters.sys_reallocs.to_string()),
        ("arena allocs", counters.arena_allocs.to_string()),
        ("arena reallocs", counters.arena_reallocs.to_string()),
        ("arena size", format_bytes(stats.arena_used_bytes)),
    ];

    let mut out = String::new();
    out.push_str(&file.file_name);
    out.push('\n');
    for (label, value) in values {
        writeln!(out, "  {label}: {value}").unwrap();
    }
    out.push('\n');
    out
}

/// Formats a byte count as the exact number followed by a human-readable size,
/// e.g. `12582912 (12.58 MB)`. The exact number is what diffs; the human-readable
/// form is only there for scanning.
fn format_bytes(bytes: usize) -> String {
    format!("{bytes} ({})", format_size(bytes, DECIMAL))
}

fn write_snapshot(file_path: &str, contents: &str) -> Result<(), io::Error> {
    let mut snapshot = File::create(project_root().join(file_path))?;
    snapshot.write_all(contents.as_bytes())?;
    snapshot.flush()?;
    Ok(())
}
