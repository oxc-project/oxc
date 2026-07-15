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
use oxc_tasks_common::{TestFiles, project_root};
use oxc_transformer::{TransformOptions, Transformer};

use std::alloc::{GlobalAlloc, Layout};
use std::sync::atomic::{AtomicUsize, Ordering::SeqCst};

#[global_allocator]
static GLOBAL: TrackedAllocator = TrackedAllocator;

struct TrackedAllocator;

/// Atomic counter.
///
/// Mainly just a wrapper around `AtomicUsize`, but `increment` method ensures that counter value
/// doesn't wrap around if counter reaches `usize::MAX`.
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
// NOTE: We are only tracking the number of system allocations here, and not the number of bytes that are allocated.
// The original version of this tool did track the number of bytes, but there was some variance between platforms that
// made it less reliable as a measurement. In general, the number of allocations is closely correlated with the size of
// allocations, so just tracking the number of allocations is sufficient for our purposes.
static NUM_ALLOC: AtomicCounter = AtomicCounter::new();
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

        parser_out.push_str(&format_stats(
            file.file_name.as_str(),
            file.source_text.len(),
            &parser_stats,
        ));

        let ((), semantic_stats) = record_stats_in(&allocator, || {
            let _ = SemanticBuilder::new().with_enum_eval(true).build(&parsed.program);
        });

        semantic_out.push_str(&format_stats(
            file.file_name.as_str(),
            file.source_text.len(),
            &semantic_stats,
        ));

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

        transformer_out.push_str(&format_stats(
            file.file_name.as_str(),
            file.source_text.len(),
            &transformer_stats,
        ));

        let ((), minifier_stats) = record_stats_in(&allocator, || {
            Minifier::new(minifier_options).minify(&allocator, &mut parsed.program);
        });

        minifier_out.push_str(&format_stats(
            file.file_name.as_str(),
            file.source_text.len(),
            &minifier_stats,
        ));

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

        formatter_out.push_str(&format_stats(
            file.file_name.as_str(),
            file.source_text.len(),
            &formatter_stats,
        ));
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
    let (arena_allocs, arena_reallocs) = allocator.get_allocation_stats();
    #[cfg(feature = "is_all_features")]
    let (arena_allocs, arena_reallocs) = (0, 0);
    #[cfg(not(feature = "is_all_features"))]
    let arena_chunk_allocs = Allocator::global_chunk_allocation_count();
    #[cfg(feature = "is_all_features")]
    let arena_chunk_allocs = 0;

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
fn record_stats_in<F, R>(allocator: &Allocator, f: F) -> (R, AllocatorStats)
where
    F: FnOnce() -> R,
{
    let before_stats = record_stats(allocator);
    let result = f();
    let diff_stats = record_stats_diff(allocator, &before_stats);

    (result, diff_stats)
}

/// Formats the allocator stats for one file as a block of `label: value` lines.
///
/// One value per line, with no column alignment, so that a change to one value produces
/// a one-line diff, and adding a new value later doesn't reformat existing lines.
/// File names stay at column 0 so they appear in git hunk headers.
fn format_stats(file_name: &str, file_size: usize, stats: &AllocatorStats) -> String {
    let values = [
        ("file size", format_size(file_size, DECIMAL)),
        ("sys allocs", stats.sys_allocs.to_string()),
        ("sys reallocs", stats.sys_reallocs.to_string()),
        ("arena allocs", stats.arena_allocs.to_string()),
        ("arena reallocs", stats.arena_reallocs.to_string()),
    ];

    let mut out = String::new();
    out.push_str(file_name);
    out.push('\n');
    for (label, value) in values {
        writeln!(out, "  {label}: {value}").unwrap();
    }
    out.push('\n');
    out
}

fn write_snapshot(file_path: &str, contents: &str) -> Result<(), io::Error> {
    let mut snapshot = File::create(project_root().join(file_path))?;
    snapshot.write_all(contents.as_bytes())?;
    snapshot.flush()?;
    Ok(())
}
