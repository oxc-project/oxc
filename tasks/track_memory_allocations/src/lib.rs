use std::{fmt::Write as _, fs, io};

use humansize::{DECIMAL, format_size};
use mimalloc_safe::MiMalloc;
use saphyr::{LoadableYamlNode, SafelyIndex, Yaml};

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

    let mut parser_entries = Vec::new();
    let mut semantic_entries = Vec::new();
    let mut transformer_entries = Vec::new();
    let mut minifier_entries = Vec::new();
    let mut formatter_entries = Vec::new();

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

        parser_entries.push((file, parser_stats));

        let ((), semantic_stats) = record_stats_in(&allocator, || {
            let _ = SemanticBuilder::new().with_enum_eval(true).build(&parsed.program);
        });

        semantic_entries.push((file, semantic_stats));

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

        transformer_entries.push((file, transformer_stats));

        let ((), minifier_stats) = record_stats_in(&allocator, || {
            Minifier::new(minifier_options).minify(&allocator, &mut parsed.program);
        });

        minifier_entries.push((file, minifier_stats));

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

        formatter_entries.push((file, formatter_stats));
    }

    write_snapshot("tasks/track_memory_allocations/allocs_parser.yaml", &parser_entries)?;
    write_snapshot("tasks/track_memory_allocations/allocs_semantic.yaml", &semantic_entries)?;
    write_snapshot("tasks/track_memory_allocations/allocs_transformer.yaml", &transformer_entries)?;
    write_snapshot("tasks/track_memory_allocations/allocs_minifier.yaml", &minifier_entries)?;
    write_snapshot("tasks/track_memory_allocations/allocs_formatter.yaml", &formatter_entries)?;

    Ok(())
}

/// Record current allocation stats from both system allocator and arena allocator.
#[cfg_attr(feature = "is_all_features", expect(unused))]
fn record_stats(allocator: &Allocator) -> AllocatorStats {
    let sys_allocs = NUM_ALLOC.get();
    let sys_reallocs = NUM_REALLOC.get();
    #[cfg(not(feature = "is_all_features"))]
    let ((arena_allocs, arena_reallocs), arena_chunk_allocs) =
        (allocator.get_allocation_stats(), Allocator::global_chunk_allocation_count());
    #[cfg(feature = "is_all_features")]
    let ((arena_allocs, arena_reallocs), arena_chunk_allocs) = ((0, 0), 0);

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

/// Tolerance in bytes when comparing a measured `arena size` against the value already
/// in the committed snapshot.
///
/// Arena byte totals are not exactly reproducible across architectures: some type layouts
/// depend on the target (e.g. hashbrown's table control groups are 16 bytes on x86_64 but
/// 8 bytes on aarch64), so each live `HashMap` in the arena shifts `used_bytes` by a few
/// bytes per platform. The observed drift between x86_64 and aarch64 is at most 128 bytes
/// per file. Keeping the committed value when the difference is within this tolerance makes
/// snapshots generated on one platform pass the `git diff` check on the others.
/// Real regressions (e.g. growing an AST node type) change arena sizes by orders of
/// magnitude more than this; changes below the tolerance are treated as noise.
const ARENA_SIZE_TOLERANCE: usize = 1024;

/// Writes one snapshot file, formatted as a YAML mapping per test file.
///
/// The snapshot is YAML so that the committed `arena size` values can be read back with a
/// YAML parser and compared against the measured ones (see [`snapshot_arena_size`]). It is
/// still emitted by hand to keep full control over the layout for git diffs.
fn write_snapshot(file_path: &str, entries: &[(&TestFile, StageStats)]) -> Result<(), io::Error> {
    let path = project_root().join(file_path);
    let committed = fs::read_to_string(&path).unwrap_or_default();
    let committed_docs = Yaml::load_from_str(&committed).unwrap_or_default();

    let mut out = String::new();
    for (file, stats) in entries {
        let committed_arena_size = committed_docs
            .first()
            .get(file.file_name.as_str())
            .get("arena size")
            .and_then(Yaml::as_integer);
        let file_size = file.source_text.len();
        render_file_stats(&mut out, &file.file_name, file_size, stats, committed_arena_size);
    }
    fs::write(path, out)
}

/// Formats the allocator stats for one file as a YAML mapping keyed by file name.
///
/// One value per line, with no column alignment, so that a change to one value produces
/// a one-line diff, and adding a new value later doesn't reformat existing lines.
/// File names stay at column 0 so they appear in git hunk headers. Byte sizes are exact
/// numbers (the value that diffs) with the human-readable form in a trailing comment.
fn render_file_stats(
    out: &mut String,
    file_name: &str,
    file_size: usize,
    stats: &StageStats,
    committed_arena_size: Option<i64>,
) {
    let arena_size = snapshot_arena_size(stats.arena_used_bytes, committed_arena_size);
    let counters = &stats.counters;
    writeln!(out, "{file_name}:").unwrap();
    writeln!(out, "  file size: {file_size} # {}", format_size(file_size, DECIMAL)).unwrap();
    writeln!(out, "  sys allocs: {}", counters.sys_allocs).unwrap();
    writeln!(out, "  sys reallocs: {}", counters.sys_reallocs).unwrap();
    writeln!(out, "  arena allocs: {}", counters.arena_allocs).unwrap();
    writeln!(out, "  arena reallocs: {}", counters.arena_reallocs).unwrap();
    writeln!(out, "  arena size: {arena_size} # {}", format_size(arena_size, DECIMAL)).unwrap();
    out.push('\n');
}

/// Chooses the `arena size` value to record: the committed value if the measured one is
/// within [`ARENA_SIZE_TOLERANCE`] of it, the measured value otherwise.
fn snapshot_arena_size(measured: usize, committed: Option<i64>) -> usize {
    let Some(committed) = committed.and_then(|value| usize::try_from(value).ok()) else {
        return measured;
    };
    if measured.abs_diff(committed) <= ARENA_SIZE_TOLERANCE { committed } else { measured }
}

#[cfg(test)]
mod tests {
    use saphyr::{LoadableYamlNode, SafelyIndex, Yaml};

    use super::{
        ARENA_SIZE_TOLERANCE, AllocatorStats, StageStats, render_file_stats, snapshot_arena_size,
    };

    fn stage_stats(arena_used_bytes: usize) -> StageStats {
        StageStats {
            counters: AllocatorStats {
                sys_allocs: 1,
                sys_reallocs: 2,
                arena_chunk_allocs: 0,
                arena_allocs: 3,
                arena_reallocs: 4,
            },
            arena_used_bytes,
        }
    }

    #[test]
    fn rendered_snapshot_is_valid_yaml() {
        let mut out = String::new();
        render_file_stats(&mut out, "foo.js", 1000, &stage_stats(12345), None);
        render_file_stats(&mut out, "bar.ts", 2_000_000, &stage_stats(99_999), None);

        let docs = Yaml::load_from_str(&out).unwrap();
        let doc = docs.first();
        assert_eq!(doc.get("foo.js").get("arena size").and_then(Yaml::as_integer), Some(12345));
        assert_eq!(doc.get("foo.js").get("sys allocs").and_then(Yaml::as_integer), Some(1));
        assert_eq!(doc.get("bar.ts").get("file size").and_then(Yaml::as_integer), Some(2_000_000));
        assert_eq!(doc.get("bar.ts").get("arena reallocs").and_then(Yaml::as_integer), Some(4));
    }

    #[test]
    fn keeps_committed_arena_size_within_tolerance() {
        assert_eq!(snapshot_arena_size(10_000, Some(10_128)), 10_128);
        assert_eq!(snapshot_arena_size(10_128, Some(10_000)), 10_000);
        let committed = i64::try_from(10_000 + ARENA_SIZE_TOLERANCE).unwrap();
        assert_eq!(snapshot_arena_size(10_000, Some(committed)), 10_000 + ARENA_SIZE_TOLERANCE);
    }

    #[test]
    fn takes_measured_arena_size_beyond_tolerance() {
        assert_eq!(snapshot_arena_size(10_000, Some(20_000)), 10_000);
        assert_eq!(snapshot_arena_size(20_000, Some(10_000)), 20_000);
    }

    #[test]
    fn takes_measured_arena_size_without_valid_committed_value() {
        assert_eq!(snapshot_arena_size(10_000, None), 10_000);
        assert_eq!(snapshot_arena_size(10_000, Some(-1)), 10_000);
    }
}
