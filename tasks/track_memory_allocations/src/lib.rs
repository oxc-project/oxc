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
use std::cell::Cell;
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering::SeqCst};
use std::sync::{Mutex, OnceLock};

use backtrace::Backtrace;
use std::env;

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
static ALLOC_SEQ: AtomicUsize = AtomicUsize::new(0);

// Re-entrancy guard to avoid tracking allocations that occur while we're
// doing the tracking work itself (e.g., backtrace symbolization, map updates).
thread_local! {
    static IN_TRACKING: Cell<bool> = Cell::new(false);
}

// Global aggregation of allocation sites: "path:line" -> count
#[derive(Default, Clone, Copy)]
struct SiteCounts { allocs: usize, reallocs: usize }

static ALLOC_SITES: OnceLock<Mutex<HashMap<String, SiteCounts>>> = OnceLock::new();
fn alloc_sites() -> &'static Mutex<HashMap<String, SiteCounts>> {
    ALLOC_SITES.get_or_init(|| Mutex::new(HashMap::new()))
}

static PROJECT_ROOT: OnceLock<PathBuf> = OnceLock::new();
fn get_project_root() -> &'static Path {
    PROJECT_ROOT.get_or_init(|| project_root()).as_path()
}

// Whether we should collect callsite info; off by default to avoid massive slowdowns.
static TRACK_SITES: OnceLock<bool> = OnceLock::new();
fn should_track_sites() -> bool {
    *TRACK_SITES.get_or_init(|| match env::var("OXC_ALLOC_SITES") {
        Ok(v) => matches!(v.as_str(), "1" | "true" | "yes" | "on"),
        Err(_) => false,
    })
}

// Sampling interval: record 1 in N allocations to reduce overhead. Default 1000.
static SAMPLE_N: OnceLock<usize> = OnceLock::new();
fn sample_interval() -> usize {
    *SAMPLE_N.get_or_init(|| match env::var("OXC_ALLOC_SAMPLE") {
        Ok(v) => v.parse().ok().filter(|n: &usize| *n > 0).unwrap_or(1000),
        Err(_) => 1000,
    })
}

fn reset_site_counts() {
    if let Ok(mut m) = alloc_sites().lock() {
        m.clear();
    }
}

enum AllocKind { Alloc, Realloc }

fn record_allocation_site(kind: AllocKind) {
    // Note: this function must be called with IN_TRACKING already set to true
    // to ensure any allocations here don't get double-counted. We also keep
    // the work minimal.
    let mut bt = Backtrace::new_unresolved();
    bt.resolve();
    let mut key: Option<String> = None;
    let root = get_project_root();

    // Resolve just enough frames to find the first oxc-repo file with a line.
    'outer: for frame in bt.frames() {
        for symbol in frame.symbols() {
            if let (Some(path), Some(lineno)) = (symbol.filename(), symbol.lineno()) {
                // Prefer frames within the repository, and skip this task's own files.
                if path.starts_with(root)
                    && !path.components().any(|c| c.as_os_str() == "track_memory_allocations")
                {
                    // Make path relative to project root for readability.
                    let rel = path.strip_prefix(root).unwrap_or(path);
                    key = Some(format!("{}:{}", rel.display(), lineno));
                    break 'outer;
                }
            }
        }
    }

    // Fallback: if we didn't find a repo frame, try any frame with file:line.
    if key.is_none() {
        'outer2: for frame in bt.frames() {
            for symbol in frame.symbols() {
                if let (Some(path), Some(lineno)) = (symbol.filename(), symbol.lineno()) {
                    key = Some(format!("{}:{}", path.display(), lineno));
                    break 'outer2;
                }
            }
        }
    }

    // Fallback: use function names when file:line isn't available (e.g., debug info stripped).
    if key.is_none() {
        // Prefer frames whose demangled name mentions oxc crates/modules.
        let mut best_oxc: Option<String> = None;
        let mut best_nonstd: Option<String> = None;
        let is_skip = |s: &str| {
            s.contains("track_memory_allocations")
                || s.contains("record_allocation_site")
                || s.contains("TrackedAllocator")
                || s.contains("backtrace::")
                || s.contains("::resolve")
                || s.starts_with("std::")
                || s.starts_with("core::")
                || s.starts_with("alloc::")
                || s.contains("mimalloc")
        };

        for frame in bt.frames() {
            for symbol in frame.symbols() {
                if let Some(name) = symbol.name() {
                    let s = format!("{:#}", name);
                    if is_skip(&s) {
                        continue;
                    }
                    if s.contains("oxc_") || s.contains("oxlint") || s.contains("oxc::") {
                        best_oxc.get_or_insert_with(|| s.clone());
                        // Keep searching in case there is a better, deeper frame, but first match is fine.
                    } else {
                        best_nonstd.get_or_insert(s);
                    }
                }
            }
        }
        key = best_oxc.or(best_nonstd);
    }

    let k = key.unwrap_or_else(|| "<unknown>".to_string());
    if let Ok(mut m) = alloc_sites().lock() {
        let entry = m.entry(k).or_default();
        match kind {
            AllocKind::Alloc => entry.allocs = entry.allocs.saturating_add(1),
            AllocKind::Realloc => entry.reallocs = entry.reallocs.saturating_add(1),
        }
    }
}

fn reset_global_allocs() {
    NUM_ALLOC.store(0, SeqCst);
    NUM_REALLOC.store(0, SeqCst);
    ALLOC_SEQ.store(0, SeqCst);
}

// SAFETY: Methods simply delegate to `MiMalloc` allocator to ensure that the allocator
// is the same across different platforms for the purposes of tracking allocations.
#[expect(unsafe_code, clippy::undocumented_unsafe_blocks)]
unsafe impl GlobalAlloc for TrackedAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ret = unsafe { MiMalloc.alloc(layout) };
        if !ret.is_null() {
            IN_TRACKING.with(|f| {
                if f.get() {
                    // Skip counting and callsite attribution for re-entrant allocations
                    // triggered by tracking itself.
                } else {
                    f.set(true);
                    NUM_ALLOC.fetch_add(1, SeqCst);
            if should_track_sites() {
                        let n = sample_interval();
                        let seq = ALLOC_SEQ.fetch_add(1, SeqCst);
                        if seq % n == 0 {
                record_allocation_site(AllocKind::Alloc);
                        }
                    }
                    f.set(false);
                }
            });
        }
        ret
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { MiMalloc.dealloc(ptr, layout) };
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let ret = unsafe { MiMalloc.alloc_zeroed(layout) };
        if !ret.is_null() {
            IN_TRACKING.with(|f| {
                if f.get() {
                    // see alloc()
                } else {
                    f.set(true);
                    NUM_ALLOC.fetch_add(1, SeqCst);
            if should_track_sites() {
                        let n = sample_interval();
                        let seq = ALLOC_SEQ.fetch_add(1, SeqCst);
                        if seq % n == 0 {
                record_allocation_site(AllocKind::Alloc);
                        }
                    }
                    f.set(false);
                }
            });
        }
        ret
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let ret = unsafe { MiMalloc.realloc(ptr, layout, new_size) };
        if !ret.is_null() {
            IN_TRACKING.with(|f| {
                if f.get() {
                    // see alloc()
                } else {
                    f.set(true);
                    NUM_REALLOC.fetch_add(1, SeqCst);
            if should_track_sites() {
                        let n = sample_interval();
                        let seq = ALLOC_SEQ.fetch_add(1, SeqCst);
                        if seq % n == 0 {
                record_allocation_site(AllocKind::Realloc);
                        }
                    }
                    f.set(false);
                }
            });
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

    // Reset counts post warm-up so only measured work below is captured.
    reset_global_allocs();
    reset_site_counts();

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

    if should_track_sites() {
        // Print and snapshot top allocation sites
        let sites_out = print_top_sites(1000);
        println!("{sites_out}");
    }

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

fn print_top_sites(limit: usize) -> String {
    // Build a top-k selection using a min-heap to avoid sorting the full map.
    #[derive(Eq, PartialEq)]
    struct KeyRef<'a> { total: usize, allocs: usize, reallocs: usize, key: &'a str }
    impl<'a> Ord for KeyRef<'a> {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            // Min-heap via Reverse: smaller total first; for ties, larger key first
            self.total.cmp(&other.total).then_with(|| other.key.cmp(self.key))
        }
    }
    impl<'a> PartialOrd for KeyRef<'a> {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            Some(self.cmp(other))
        }
    }

    let mut top: Vec<(String, SiteCounts)> = Vec::new();

    IN_TRACKING.with(|flag| {
        let prev = flag.replace(true);
        if let Ok(m) = alloc_sites().lock() {
            let mut heap: BinaryHeap<Reverse<KeyRef<'_>>> = BinaryHeap::with_capacity(limit);
            for (ks, counts) in m.iter() {
                let total = counts.allocs.saturating_add(counts.reallocs);
                if total == 0 { continue; }
                let entry = KeyRef { total, allocs: counts.allocs, reallocs: counts.reallocs, key: ks.as_str() };
                if heap.len() < limit {
                    heap.push(Reverse(entry));
                } else if let Some(Reverse(worst)) = heap.peek() {
                    if entry.total > worst.total
                        || (entry.total == worst.total && entry.key < worst.key)
                    {
                        let _ = heap.pop();
                        heap.push(Reverse(entry));
                    }
                }
            }
            top.reserve(heap.len());
            while let Some(Reverse(e)) = heap.pop() {
                top.push((e.key.to_owned(), SiteCounts { allocs: e.allocs, reallocs: e.reallocs }));
            }
        }
        flag.set(prev);
    });

    // Sort selection for display: total desc, key asc.
    top.sort_unstable_by(|a, b| {
        let at = a.1.allocs + a.1.reallocs;
        let bt = b.1.allocs + b.1.reallocs;
        bt.cmp(&at).then_with(|| a.0.cmp(&b.0))
    });

    let mut out = String::new();
    let width_loc = top.iter().map(|(k, _)| k.len()).max().unwrap_or(10).max("Location".len());
    let width_cnt = 14;
    let n = sample_interval();
    let _ = writeln!(out, "Top allocation sites (sampled 1 in {n})");
    writeln!(
        out,
        "{:width_loc$} | {:width_cnt$} | {:width_cnt$} | {:width_cnt$}",
        "Location",
        "Allocations",
        "Reallocations",
        "Total",
        width_loc = width_loc,
        width_cnt = width_cnt
    ).unwrap();
    let dash_len = width_loc + 3 + (width_cnt + 3) * 3 - 3;
    out.push_str(&"-".repeat(dash_len));
    out.push('\n');
    for (loc, counts) in top {
        let total = counts.allocs + counts.reallocs;
        let _ = writeln!(out,
            "{:width_loc$} | {:width_cnt$} | {:width_cnt$} | {:width_cnt$}",
            loc,
            counts.allocs,
            counts.reallocs,
            total,
            width_loc = width_loc,
            width_cnt = width_cnt);
    }
    out
}
