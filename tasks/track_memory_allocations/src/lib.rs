#![expect(clippy::print_stdout)]

use std::{
    fmt::Write as _,
    fs::File,
    io::{self, Write},
};

use humansize::{DECIMAL, format_size};
use mimalloc_safe::MiMalloc;

use oxc_allocator::Allocator;
use oxc_parser::{ParseOptions, Parser};
use oxc_semantic::SemanticBuilder;
use oxc_tasks_common::{TestFile, TestFiles, project_root};

use std::alloc::{GlobalAlloc, Layout};
use std::sync::atomic::{AtomicUsize, Ordering::SeqCst};

#[global_allocator]
static GLOBAL: TrackedAllocator = TrackedAllocator;

struct TrackedAllocator;

/// Number of system allocations
// NOTE: We are only tracking the number of system allocations here, and not the number of bytes that are allocated.
// The original version of this tool did track the number of bytes, but there was some variance between platforms that
// made it less reliable as a measurement. In general, the number of allocations is closely correlated with the size of
// allocations, so just tracking the number of allocations is sufficient for our purposes.
static NUM_ALLOC: AtomicUsize = AtomicUsize::new(0);
static NUM_REALLOC: AtomicUsize = AtomicUsize::new(0);

fn reset_global_allocs() {
    NUM_ALLOC.store(0, SeqCst);
    NUM_REALLOC.store(0, SeqCst);
}

// SAFETY: Methods simply delegate to `MiMalloc` allocator to ensure that the allocator
// is the same across different platforms for the purposes of tracking allocations.
#[expect(unsafe_code, clippy::undocumented_unsafe_blocks)]
unsafe impl GlobalAlloc for TrackedAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ret = unsafe { MiMalloc.alloc(layout) };
        if !ret.is_null() {
            NUM_ALLOC.fetch_add(1, SeqCst);
        }
        ret
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { MiMalloc.dealloc(ptr, layout) };
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let ret = unsafe { MiMalloc.alloc_zeroed(layout) };
        if !ret.is_null() {
            NUM_ALLOC.fetch_add(1, SeqCst);
        }
        ret
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let ret = unsafe { MiMalloc.realloc(ptr, layout, new_size) };
        if !ret.is_null() {
            NUM_REALLOC.fetch_add(1, SeqCst);
        }
        ret
    }
}

#[test]
#[cfg(any(coverage, coverage_nightly))]
fn test() {
    run().unwrap();
}

struct Stats<'a> {
    file: &'a TestFile,
    sys_allocs: usize,
    sys_reallocs: usize,
    arena_allocs: usize,
    arena_reallocs: usize,
    arena_bytes: usize,
}

fn record_stats<'a>(file: &'a TestFile, allocator: &Allocator) -> Stats<'a> {
    let sys_allocs = NUM_ALLOC.load(SeqCst);
    let sys_reallocs = NUM_REALLOC.load(SeqCst);
    #[cfg(not(feature = "is_all_features"))]
    let (arena_allocs, arena_reallocs) = allocator.get_allocation_stats();
    #[cfg(feature = "is_all_features")]
    let (arena_allocs, arena_reallocs) = (0, 0);
    let arena_bytes = allocator.used_bytes();

    Stats { file, sys_allocs, sys_reallocs, arena_allocs, arena_reallocs, arena_bytes }
}

/// Computes the difference between two `Stats` instances, by subtracting the before values from the after values.
/// This is useful for computing the allocations that occurred without needing to reset all the stats in between.
fn diff_stats<'a>(before: &Stats<'a>, mut after: Stats<'a>) -> Stats<'a> {
    after.sys_allocs -= before.sys_allocs;
    after.sys_reallocs -= before.sys_reallocs;
    after.arena_allocs -= before.arena_allocs;
    after.arena_reallocs -= before.arena_reallocs;
    after.arena_bytes -= before.arena_bytes;
    after
}

/// # Panics
/// # Errors
pub fn run() -> Result<(), io::Error> {
    let files = TestFiles::complicated();

    let mut allocator = Allocator::default();

    let options = ParseOptions { parse_regular_expression: true, ..ParseOptions::default() };

    let mut parser_stats = Vec::new();
    let mut semantic_stats = Vec::new();

    // Warm-up by parsing each file first, and then measuring the actual allocations. This reduces variance
    // in the number of allocations, because we ensure that the bump allocator has already requested all
    // of the space it will need from the system allocator to parse the largest file in the set.
    for file in files.files() {
        let ret = Parser::new(&allocator, &file.source_text, file.source_type)
            .with_options(options)
            .parse();
        SemanticBuilder::new()
            .with_check_syntax_error(false)
            .with_scope_tree_child_ids(true)
            .with_cfg(true)
            .with_build_jsdoc(true)
            .build(&ret.program);
    }

    for file in files.files() {
        allocator.reset();
        reset_global_allocs();

        let ret = Parser::new(&allocator, &file.source_text, file.source_type)
            .with_options(options)
            .parse();

        let parser_stat = record_stats(file, &allocator);

        // TODO: Get the stats out of the `Scoping` internal allocator somehow. For now, we just
        // record the system allocations and leave the arena allocations as zero.
        SemanticBuilder::new()
            .with_check_syntax_error(false)
            .with_scope_tree_child_ids(true)
            .with_cfg(true)
            .with_build_jsdoc(true)
            .build(&ret.program);

        let semantic_stat = diff_stats(&parser_stat, record_stats(file, &allocator));

        parser_stats.push(parser_stat);
        semantic_stats.push(semantic_stat);
    }

    // Print parser stats
    let parser_out = print_stats_table(&parser_stats);
    println!("{parser_out}");

    let semantic_out = print_stats_table(&semantic_stats);
    println!("{semantic_out}");

    let parser_snap_path = project_root().join("tasks/track_memory_allocations/allocs_parser.snap");
    let mut snapshot = File::create(parser_snap_path)?;
    snapshot.write_all(parser_out.as_bytes())?;
    snapshot.flush()?;

    let semantic_snap_path =
        project_root().join("tasks/track_memory_allocations/allocs_semantic.snap");
    let mut snapshot = File::create(semantic_snap_path)?;
    snapshot.write_all(semantic_out.as_bytes())?;
    snapshot.flush()?;

    Ok(())
}

fn print_stats_table(stats: &[Stats]) -> String {
    let mut out = String::new();

    let width = 14;
    let fixture_width = stats.iter().map(|s| s.file.file_name.len()).max().unwrap();
    writeln!(
        out,
        "{:fixture_width$} | {:width$} || {:width$} | {:width$} || {:width$} | {:width$} | {:width$} ",
        "File",
        "File size",
        "Sys allocs",
        "Sys reallocs",
        "Arena allocs",
        "Arena reallocs",
        "Arena bytes",
        width = width,
    ).unwrap();
    out.push_str(&str::repeat("-", width * 7 + fixture_width + 15));
    out.push('\n');

    for stats in stats {
        let s = format!(
            // Using two newlines at the end makes it easier to diff
            "{:fixture_width$} | {:width$} || {:width$} | {:width$} || {:width$} | {:width$} | {:width$}\n\n",
            stats.file.file_name,
            format_size(stats.file.source_text.len(), DECIMAL),
            stats.sys_allocs,
            stats.sys_reallocs,
            stats.arena_allocs,
            stats.arena_reallocs,
            format_size(stats.arena_bytes, DECIMAL.decimal_places(3)),
            width = width
        );
        out.push_str(&s);
    }

    out
}
