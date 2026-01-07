use std::{
    fs::File,
    io::{self, Write},
};

use humansize::{DECIMAL, format_size};
use mimalloc_safe::MiMalloc;

use oxc_allocator::Allocator;
use oxc_minifier::{CompressOptions, MangleOptions, Minifier, MinifierOptions};
use oxc_parser::{ParseOptions, Parser};
use oxc_semantic::SemanticBuilder;
use oxc_tasks_common::{TestFiles, project_root};
use oxc_transformer::{TransformOptions, Transformer};

use std::alloc::{GlobalAlloc, Layout};
use std::sync::atomic::{AtomicUsize, Ordering::SeqCst};

#[cfg(feature = "track_callsites")]
use std::sync::atomic::AtomicBool;

#[cfg(feature = "track_callsites")]
use std::{backtrace::Backtrace, cell::Cell, sync::Mutex};

#[cfg(feature = "track_callsites")]
use rustc_hash::FxHashMap;

// Thread-local guard to prevent recursive callsite tracking.
// Capturing a backtrace may itself cause allocations, which would lead to infinite recursion.
#[cfg(feature = "track_callsites")]
thread_local! {
    static IN_CALLSITE_RECORDING: Cell<bool> = const { Cell::new(false) };
}

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
        // Result of `fetch_update` cannot be `Err` as closure always returns `Some`
        let _ = self.0.fetch_update(SeqCst, SeqCst, |count| Some(count.saturating_add(1)));
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

/// When `track_callsites` feature is enabled, this tracks allocation callsites.
/// Each unique callsite (identified by a normalized backtrace string) maps to its allocation count.
#[cfg(feature = "track_callsites")]
static CALLSITE_COUNTS: Mutex<Option<FxHashMap<String, usize>>> = Mutex::new(None);

/// Flag to enable/disable callsite tracking at runtime. This allows us to only track
/// callsites during the actual measurement phase, not during warmup.
#[cfg(feature = "track_callsites")]
static TRACK_CALLSITES_ENABLED: AtomicBool = AtomicBool::new(false);

#[cfg(feature = "track_callsites")]
fn enable_callsite_tracking() {
    // Initialize the hashmap if not already done
    {
        let mut guard = CALLSITE_COUNTS.lock().unwrap();
        if guard.is_none() {
            *guard = Some(FxHashMap::default());
        }
    }
    TRACK_CALLSITES_ENABLED.store(true, SeqCst);
}

#[cfg(feature = "track_callsites")]
fn disable_callsite_tracking() {
    TRACK_CALLSITES_ENABLED.store(false, SeqCst);
}

#[cfg(feature = "track_callsites")]
fn reset_callsite_counts() {
    let mut guard = CALLSITE_COUNTS.lock().unwrap();
    if let Some(map) = guard.as_mut() {
        map.clear();
    }
}

#[cfg(feature = "track_callsites")]
fn record_callsite() {
    if !TRACK_CALLSITES_ENABLED.load(SeqCst) {
        return;
    }

    // Guard against recursive callsite recording - capturing backtraces can allocate
    let already_recording = IN_CALLSITE_RECORDING.with(|flag| {
        if flag.get() {
            true
        } else {
            flag.set(true);
            false
        }
    });

    if already_recording {
        return;
    }

    let bt = Backtrace::capture();
    let bt_str = bt.to_string();

    // Extract the relevant part of the backtrace (skip allocator frames)
    let callsite = normalize_backtrace(&bt_str);

    let mut guard = CALLSITE_COUNTS.lock().unwrap();
    if let Some(map) = guard.as_mut() {
        *map.entry(callsite).or_insert(0) += 1;
    }

    // Reset the guard
    IN_CALLSITE_RECORDING.with(|flag| flag.set(false));
}

/// Normalize backtrace to extract a meaningful callsite identifier.
/// This skips the allocator-related frames and captures the actual caller.
#[cfg(feature = "track_callsites")]
fn normalize_backtrace(bt: &str) -> String {
    let lines: Vec<&str> = bt.lines().collect();

    // Find frames that are from the oxc crates (skip allocator frames)
    let mut relevant_frames = Vec::new();
    let mut skip = true;

    for line in &lines {
        let trimmed = line.trim();

        // Skip frames until we get past the allocator internals
        if skip {
            if trimmed.contains("track_memory_allocations")
                || trimmed.contains("GlobalAlloc")
                || trimmed.contains("__rust_alloc")
                || trimmed.contains("alloc::") && !trimmed.contains("oxc_allocator")
            {
                continue;
            }
            skip = false;
        }

        // Capture frames from oxc crates (but not from this tracking crate itself)
        if trimmed.contains("oxc_") && !trimmed.contains("track_memory_allocations") {
            relevant_frames.push(trimmed.to_string());
            // Only keep top few frames to reduce noise
            if relevant_frames.len() >= 5 {
                break;
            }
        }
    }

    if relevant_frames.is_empty() {
        // Fall back to first non-allocator frame
        for line in &lines {
            let trimmed = line.trim();
            if !trimmed.contains("track_memory_allocations")
                && !trimmed.contains("GlobalAlloc")
                && !trimmed.contains("__rust_alloc")
                && !trimmed.is_empty()
                && trimmed.chars().next().is_some_and(char::is_numeric)
            {
                return trimmed.to_string();
            }
        }
        return "unknown".to_string();
    }

    relevant_frames.join("\n")
}

#[cfg(feature = "track_callsites")]
#[expect(clippy::print_stdout)]
fn print_callsite_stats(file: &str) {
    let guard = CALLSITE_COUNTS.lock().unwrap();
    if let Some(map) = guard.as_ref() {
        let mut entries: Vec<_> = map.iter().collect();
        entries.sort_by(|a, b| b.1.cmp(a.1)); // Sort by count descending

        println!("\n=== Allocation callers for {} ===", file);

        // Show top 100
        for (callsite, count) in entries.iter().take(100) {
            println!("--- {count} allocations ---");
            println!("{callsite}\n");
        }
    }
}

fn reset_global_allocs() {
    NUM_ALLOC.reset();
    NUM_REALLOC.reset();
    #[cfg(feature = "track_callsites")]
    reset_callsite_counts();
}

// SAFETY: Methods simply delegate to `MiMalloc` allocator to ensure that the allocator
// is the same across different platforms for the purposes of tracking allocations.
#[expect(clippy::undocumented_unsafe_blocks)]
unsafe impl GlobalAlloc for TrackedAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ret = unsafe { MiMalloc.alloc(layout) };
        if !ret.is_null() {
            NUM_ALLOC.increment();
            #[cfg(feature = "track_callsites")]
            record_callsite();
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
            #[cfg(feature = "track_callsites")]
            record_callsite();
        }
        ret
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let ret = unsafe { MiMalloc.realloc(ptr, layout, new_size) };
        if !ret.is_null() {
            NUM_REALLOC.increment();
            #[cfg(feature = "track_callsites")]
            record_callsite();
        }
        ret
    }
}

/// Stores all of the memory allocation stats that will be printed for each file.
#[derive(Debug)]
struct AllocatorStats {
    /// Number of allocations made by system allocator
    sys_allocs: usize,
    /// Number of reallocations made by system allocator
    sys_reallocs: usize,
    /// Number of allocations made by arena/bump allocator
    arena_allocs: usize,
    /// Number of reallocations made by arena/bump allocator
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
    // Width of each column in the output table
    let width = 14;
    // Width of the longest file name, used for formatting the first column
    let fixture_width = files.files().iter().map(|file| file.file_name.len()).max().unwrap();

    // Table header, which should be same for each file
    let table_header = format_table_header(fixture_width, width);

    let mut parser_out = table_header.clone();
    let mut minifier_out = table_header;

    let mut allocator = Allocator::default();

    let parse_options = ParseOptions { parse_regular_expression: true, ..ParseOptions::default() };
    let minifier_options = MinifierOptions {
        mangle: Some(MangleOptions::default()),
        compress: Some(CompressOptions::smallest()),
    };

    // Warm-up by parsing each file first, and then measuring the actual allocations. This reduces variance
    // in the number of allocations, because we ensure that the bump allocator has already requested all
    // of the space it will need from the system allocator to parse the largest file in the set.
    for file in files.files() {
        let mut parsed = Parser::new(&allocator, &file.source_text, file.source_type)
            .with_options(parse_options)
            .parse();
        assert!(parsed.errors.is_empty());

        // Transform TypeScript to ESNext before minifying (minifier only works on esnext)
        let scoping = SemanticBuilder::new().build(&parsed.program).semantic.into_scoping();
        let transform_options = TransformOptions::from_target("esnext").unwrap();
        let _ =
            Transformer::new(&allocator, std::path::Path::new(&file.file_name), &transform_options)
                .build_with_scoping(scoping, &mut parsed.program);

        Minifier::new(minifier_options.clone()).minify(&allocator, &mut parsed.program);
    }



    for file in files.files() {
        // Enable callsite tracking for the actual measurement phase
        #[cfg(feature = "track_callsites")]
        enable_callsite_tracking();

        let minifier_options = minifier_options.clone();

        allocator.reset();
        reset_global_allocs();

        let mut parsed = Parser::new(&allocator, &file.source_text, file.source_type)
            .with_options(parse_options)
            .parse();
        assert!(parsed.errors.is_empty());

        let parser_stats = record_stats(&allocator);

        parser_out.push_str(&format_table_row(
            file.file_name.as_str(),
            file.source_text.len(),
            &parser_stats,
            fixture_width,
            width,
        ));

        // Transform TypeScript to ESNext before minifying (minifier only works on esnext)
        let scoping = SemanticBuilder::new().build(&parsed.program).semantic.into_scoping();
        let transform_options = TransformOptions::from_target("esnext").unwrap();
        let _ =
            Transformer::new(&allocator, std::path::Path::new(&file.file_name), &transform_options)
                .build_with_scoping(scoping, &mut parsed.program);

        let before_minify_stats = record_stats(&allocator);

        Minifier::new(minifier_options).minify(&allocator, &mut parsed.program);

        let minifier_stats = record_stats_diff(&allocator, &before_minify_stats);

        minifier_out.push_str(&format_table_row(
            file.file_name.as_str(),
            file.source_text.len(),
            &minifier_stats,
            fixture_width,
            width,
        ));

        // Disable callsite tracking and print stats
        #[cfg(feature = "track_callsites")]
        {
            disable_callsite_tracking();
            print_callsite_stats(file.file_name.as_str());
        }
    }

    write_snapshot("tasks/track_memory_allocations/allocs_parser.snap", &parser_out)?;
    write_snapshot("tasks/track_memory_allocations/allocs_minifier.snap", &minifier_out)?;

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

    AllocatorStats { sys_allocs, sys_reallocs, arena_allocs, arena_reallocs }
}

/// Record current allocation stats since the last recorded stats in `prev`. This is useful
/// for measuring allocations made during a specific operation without needing to reset the stats.
fn record_stats_diff(allocator: &Allocator, prev: &AllocatorStats) -> AllocatorStats {
    let stats = record_stats(allocator);
    AllocatorStats {
        sys_allocs: stats.sys_allocs.saturating_sub(prev.sys_allocs),
        sys_reallocs: stats.sys_reallocs.saturating_sub(prev.sys_reallocs),
        arena_allocs: stats.arena_allocs.saturating_sub(prev.arena_allocs),
        arena_reallocs: stats.arena_reallocs.saturating_sub(prev.arena_reallocs),
    }
}

/// Formats a single row of the allocator stats table
fn format_table_row(
    file_name: &str,
    file_size: usize,
    stats: &AllocatorStats,
    fixture_width: usize,
    width: usize,
) -> String {
    format!(
        "{:fixture_width$} | {:width$} || {:width$} | {:width$} || {:width$} | {:width$}\n\n",
        file_name,
        format_size(file_size, DECIMAL),
        stats.sys_allocs,
        stats.sys_reallocs,
        stats.arena_allocs,
        stats.arena_reallocs,
        fixture_width = fixture_width,
        width = width
    )
}

fn format_table_header(fixture_width: usize, width: usize) -> String {
    let mut out = format!(
        "{:fixture_width$} | {:width$} || {:width$} | {:width$} || {:width$} | {:width$} | {:width$} \n",
        "File",
        "File size",
        "Sys allocs",
        "Sys reallocs",
        "Arena allocs",
        "Arena reallocs",
        "Arena bytes",
        fixture_width = fixture_width,
        width = width
    );
    out.push_str(&str::repeat("-", width * 7 + fixture_width + 15));
    out.push('\n');
    out
}

fn write_snapshot(file_path: &str, contents: &str) -> Result<(), io::Error> {
    let mut snapshot = File::create(project_root().join(file_path))?;
    snapshot.write_all(contents.as_bytes())?;
    snapshot.flush()?;
    Ok(())
}
