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

    /// Add `n` to the counter, returning the new value.
    fn add(&self, n: usize) -> usize {
        self.0.update(SeqCst, SeqCst, |count| count.saturating_add(n)).saturating_add(n)
    }

    fn sub(&self, n: usize) {
        self.0.update(SeqCst, SeqCst, |count| count.saturating_sub(n));
    }

    /// Raise the counter to `value` if it is currently lower.
    fn update_max(&self, value: usize) {
        self.0.update(SeqCst, SeqCst, |count| count.max(value));
    }

    fn set(&self, value: usize) {
        self.0.store(value, SeqCst);
    }

    fn reset(&self) {
        self.0.store(0, SeqCst);
    }
}

/// Live counters for the system (heap) allocator, updated by [`TrackedAllocator`].
///
/// To track a new heap metric: add an [`AtomicCounter`] here, update it in the `record_*`
/// methods called from the [`GlobalAlloc`] impl, then read it out in [`HeapTracker::read`]
/// (for counters diffed per stage) or directly off the field (for gauges read in
/// [`record_stats_in`]).
struct HeapTracker {
    /// Number of system allocations
    allocs: AtomicCounter,
    /// Number of system reallocations
    reallocs: AtomicCounter,
    /// Number of system deallocations
    deallocs: AtomicCounter,
    /// Total bytes requested from the system allocator: allocation sizes, plus growth
    /// from reallocations. A monotonic counter.
    alloc_bytes: AtomicCounter,
    /// Bytes currently allocated from the system allocator. A live gauge, never reset.
    live_size: AtomicCounter,
    /// High-water mark of [`live_size`](Self::live_size) since the last
    /// [`reset_peak`](Self::reset_peak).
    peak_size: AtomicCounter,
}

static HEAP: HeapTracker = HeapTracker {
    allocs: AtomicCounter::new(),
    reallocs: AtomicCounter::new(),
    deallocs: AtomicCounter::new(),
    alloc_bytes: AtomicCounter::new(),
    live_size: AtomicCounter::new(),
    peak_size: AtomicCounter::new(),
};

impl HeapTracker {
    fn record_alloc(&self, size: usize) {
        self.allocs.increment();
        self.grow(size);
    }

    fn record_dealloc(&self, size: usize) {
        self.deallocs.increment();
        self.live_size.sub(size);
    }

    fn record_realloc(&self, old_size: usize, new_size: usize) {
        self.reallocs.increment();
        if new_size >= old_size {
            self.grow(new_size - old_size);
        } else {
            self.live_size.sub(old_size - new_size);
        }
    }

    /// Record `delta` new bytes obtained from the system allocator.
    fn grow(&self, delta: usize) {
        self.alloc_bytes.add(delta);
        let live_size = self.live_size.add(delta);
        self.peak_size.update_max(live_size);
    }

    fn read(&self) -> HeapCounters {
        HeapCounters {
            allocs: self.allocs.get(),
            reallocs: self.reallocs.get(),
            deallocs: self.deallocs.get(),
            alloc_bytes: self.alloc_bytes.get(),
        }
    }

    /// Start a new peak measurement window: the high-water mark restarts from the
    /// current live size.
    fn reset_peak(&self) {
        self.peak_size.set(self.live_size.get());
    }

    fn reset(&self) {
        self.allocs.reset();
        self.reallocs.reset();
        self.deallocs.reset();
        self.alloc_bytes.reset();
        // `live_size` and `peak_size` are gauges of real live memory, not counters;
        // they are never reset to zero.
    }
}

/// Whether the system allocator call currently on the stack is an arena chunk operation.
///
/// Chunk operations are excluded from all heap metrics: whether an arena needs one more
/// chunk — and how big it is — depends on byte totals that vary across platforms with
/// target type layout (e.g. hashbrown tables are wider on x86_64 than aarch64; see
/// #22621), and for long-lived arenas chunk growth reflects the arena's history rather
/// than the measured operation's own behavior. Arenas mark their chunk operations
/// immediately before calling the system allocator (see `oxc_allocator::tracking`), and
/// this consumes the marker.
fn is_chunk_operation() -> bool {
    #[cfg(not(feature = "is_all_features"))]
    return Allocator::take_pending_chunk_operation();
    #[cfg(feature = "is_all_features")]
    false
}

// SAFETY: Methods simply delegate to `MiMalloc` allocator to ensure that the allocator
// is the same across different platforms for the purposes of tracking allocations.
//
// Note: `is_chunk_operation` must be consumed unconditionally (even when the allocation
// fails), so it comes first in the `&&` chains below.
#[expect(clippy::undocumented_unsafe_blocks)]
unsafe impl GlobalAlloc for TrackedAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ret = unsafe { MiMalloc.alloc(layout) };
        if !is_chunk_operation() && !ret.is_null() {
            HEAP.record_alloc(layout.size());
        }
        ret
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { MiMalloc.dealloc(ptr, layout) };
        if !is_chunk_operation() {
            HEAP.record_dealloc(layout.size());
        }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let ret = unsafe { MiMalloc.alloc_zeroed(layout) };
        if !is_chunk_operation() && !ret.is_null() {
            HEAP.record_alloc(layout.size());
        }
        ret
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let ret = unsafe { MiMalloc.realloc(ptr, layout, new_size) };
        if !is_chunk_operation() && !ret.is_null() {
            HEAP.record_realloc(layout.size(), new_size);
        }
        ret
    }
}

/// Stores all of the memory allocation stats that will be printed for each file.
#[derive(Debug)]
struct StageStats {
    /// Deltas of the heap and arena counters across the measured operation
    counters: Counters,
    /// Bytes used in the measured `Allocator`'s arena at the end of the measured operation.
    /// A point-in-time gauge read after the operation completes, not a diffed counter.
    /// The arena is not reset between stages, so this includes content from earlier stages
    /// (e.g. the AST the parser built).
    arena_used_bytes: usize,
    /// Peak growth of live heap memory during the measured operation: the high-water mark
    /// of bytes allocated from the system allocator, minus the live bytes when the
    /// operation started. Captures transient heap the operation needs even when it frees
    /// it again before finishing. Arena chunk memory is excluded (see
    /// [`is_chunk_operation`]): chunk sizes are quantized and platform-dependent, and for
    /// long-lived arenas they reflect the arena's growth history rather than the measured
    /// operation's own heap use.
    sys_peak_growth_bytes: usize,
}

impl StageStats {
    /// Every metric that appears in a snapshot, in output order.
    ///
    /// This is the single place that decides what gets snapshotted. To add a metric:
    /// measure it — a counter in [`HeapCounters`] / [`ArenaCounters`], or a gauge read in
    /// [`record_stats_in`] — and list it here with a label and a [`MetricKind`].
    fn metrics(&self, file_size: usize) -> Vec<Metric> {
        let Counters { heap, arena } = &self.counters;
        vec![
            Metric::bytes("file size", file_size, 0),
            Metric::count("sys allocs", heap.allocs),
            Metric::count("sys reallocs", heap.reallocs),
            Metric::count("sys deallocs", heap.deallocs),
            Metric::bytes(
                "sys alloc bytes",
                heap.alloc_bytes,
                relative_tolerance(heap.alloc_bytes),
            ),
            Metric::bytes(
                "sys peak growth",
                self.sys_peak_growth_bytes,
                relative_tolerance(self.sys_peak_growth_bytes),
            ),
            Metric::count("arena allocs", arena.allocs),
            Metric::count("arena reallocs", arena.reallocs),
            Metric::bytes("arena size", self.arena_used_bytes, ARENA_SIZE_TOLERANCE),
        ]
    }
}

/// Counters captured from the heap and arena allocators.
/// Used both as a raw snapshot and as per-operation deltas (see [`Counters::diff_since`]).
#[derive(Debug)]
struct Counters {
    heap: HeapCounters,
    arena: ArenaCounters,
}

/// Counters of the system (heap) allocator.
///
/// Arena chunk operations are excluded from all of these — [`TrackedAllocator`] skips
/// system allocator calls marked as chunk operations (see [`is_chunk_operation`]).
#[derive(Debug)]
struct HeapCounters {
    /// Number of allocations
    allocs: usize,
    /// Number of reallocations
    reallocs: usize,
    /// Number of deallocations
    deallocs: usize,
    /// Total bytes requested from the system allocator: allocation sizes, plus growth
    /// from reallocations
    alloc_bytes: usize,
}

/// Counters of an arena [`Allocator`].
#[derive(Debug)]
struct ArenaCounters {
    /// Number of allocations
    allocs: usize,
    /// Number of reallocations
    reallocs: usize,
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
        HEAP.reset();

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
        HEAP.reset();

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

impl Counters {
    /// Record current counters from both the heap allocator and the arena allocator.
    #[cfg_attr(feature = "is_all_features", expect(unused))]
    fn record(allocator: &Allocator) -> Self {
        let heap = HEAP.read();
        #[cfg(not(feature = "is_all_features"))]
        let (allocs, reallocs) = allocator.get_allocation_stats();
        #[cfg(feature = "is_all_features")]
        let (allocs, reallocs) = (0, 0);

        Self { heap, arena: ArenaCounters { allocs, reallocs } }
    }

    /// Counters accumulated since `prev` was recorded. This is useful for measuring
    /// allocations made during a specific operation without needing to reset the counters.
    ///
    /// Arena chunk operations need no correction here: they are already excluded at the
    /// source by [`TrackedAllocator`] (see [`is_chunk_operation`]). All remaining counters
    /// grow on element counts, which are identical on all platforms.
    fn diff_since(&self, prev: &Self) -> Self {
        Self {
            heap: HeapCounters {
                allocs: self.heap.allocs.saturating_sub(prev.heap.allocs),
                reallocs: self.heap.reallocs.saturating_sub(prev.heap.reallocs),
                deallocs: self.heap.deallocs.saturating_sub(prev.heap.deallocs),
                alloc_bytes: self.heap.alloc_bytes.saturating_sub(prev.heap.alloc_bytes),
            },
            arena: ArenaCounters {
                allocs: self.arena.allocs.saturating_sub(prev.arena.allocs),
                reallocs: self.arena.reallocs.saturating_sub(prev.arena.reallocs),
            },
        }
    }
}

/// Records the allocation stats before and after the given closure is executed.
fn record_stats_in<F, R>(allocator: &Allocator, f: F) -> (R, StageStats)
where
    F: FnOnce() -> R,
{
    let before = Counters::record(allocator);
    let live_before = HEAP.live_size.get();
    HEAP.reset_peak();
    let result = f();
    let counters = Counters::record(allocator).diff_since(&before);
    let stats = StageStats {
        counters,
        arena_used_bytes: allocator.used_bytes(),
        sys_peak_growth_bytes: HEAP.peak_size.get().saturating_sub(live_before),
    };

    (result, stats)
}

/// Tolerance in bytes when comparing a measured `arena size` against the value already
/// in the committed snapshot.
///
/// The observed cross-architecture drift of arena content is at most 128 bytes per file
/// (see [`MetricKind::Bytes`] for why byte totals drift at all); 1024 gives an 8x
/// margin while still catching any real change (e.g. growing an AST node type changes
/// arena sizes by orders of magnitude more).
const ARENA_SIZE_TOLERANCE: usize = 1024;

/// Tolerance for byte metrics whose cross-architecture drift scales with the amount of
/// work a stage does — every heap hashbrown table allocated during the stage (and every
/// one live at the peak) contributes a few bytes of drift (see
/// [`MetricKind::Bytes`]) — so a flat tolerance can't fit both small and large
/// values: 1%, with a floor for small values.
fn relative_tolerance(value: usize) -> usize {
    (value / 100).max(4096)
}

/// One row of a snapshot: a labelled value plus how it should be snapshotted and rendered.
struct Metric {
    /// YAML key the metric is written under, and its committed value is looked up under
    label: &'static str,
    /// Measured value
    value: usize,
    kind: MetricKind,
}

/// How a metric's value behaves across platforms, which decides how it is snapshotted
/// and rendered.
#[derive(Clone, Copy)]
enum MetricKind {
    /// A count, identical on every platform. Snapshotted exactly.
    Count,
    /// A byte total, rendered with a human-readable size comment.
    ///
    /// Byte totals can vary slightly across architectures: some type layouts depend on
    /// the target (e.g. hashbrown's table control groups are 16 bytes on x86_64 but
    /// 8 bytes on aarch64), so each live `HashMap` shifts byte totals by a few bytes per
    /// platform. Keeping the committed value when the measured one is within `tolerance`
    /// of it makes snapshots generated on one platform pass the `git diff` check on the
    /// others; real regressions are far larger and rewrite the value.
    Bytes {
        /// Maximum difference from the committed value that is treated as cross-platform
        /// noise rather than a real change. `0` for byte totals that are identical on
        /// every platform.
        tolerance: usize,
    },
}

impl Metric {
    fn count(label: &'static str, value: usize) -> Self {
        Self { label, value, kind: MetricKind::Count }
    }

    fn bytes(label: &'static str, value: usize, tolerance: usize) -> Self {
        Self { label, value, kind: MetricKind::Bytes { tolerance } }
    }

    /// The value to record in the snapshot, given the committed value (if any).
    ///
    /// [`MetricKind::Bytes`] metrics keep the committed value when the measured one is
    /// within their tolerance of it; everything else records the measured value exactly.
    fn snapshot_value(&self, committed: Option<i64>) -> usize {
        let MetricKind::Bytes { tolerance } = self.kind else {
            return self.value;
        };
        let Some(committed) = committed.and_then(|value| usize::try_from(value).ok()) else {
            return self.value;
        };
        if self.value.abs_diff(committed) <= tolerance { committed } else { self.value }
    }
}

/// Writes one snapshot file, formatted as a YAML mapping per test file.
///
/// The snapshot is YAML so that committed values can be read back with a YAML parser and
/// compared against the measured ones (see [`Metric::snapshot_value`]). It is still emitted
/// by hand to keep full control over the layout for git diffs.
fn write_snapshot(file_path: &str, entries: &[(&TestFile, StageStats)]) -> Result<(), io::Error> {
    let path = project_root().join(file_path);
    let committed = fs::read_to_string(&path).unwrap_or_default();
    let committed_docs = Yaml::load_from_str(&committed).unwrap_or_default();

    let mut out = String::new();
    let committed_doc = committed_docs.first();
    for (file, stats) in entries {
        let committed_file = committed_doc.get(file.file_name.as_str());
        let metrics = stats.metrics(file.source_text.len());
        render_file_stats(&mut out, &file.file_name, &metrics, committed_file);
    }
    fs::write(path, out)
}

/// Formats one file's metrics as a YAML mapping keyed by file name.
///
/// One metric per line, with no column alignment, so that a change to one value produces
/// a one-line diff, and adding a new metric later doesn't reformat existing lines.
/// File names stay at column 0 so they appear in git hunk headers. Byte sizes are exact
/// numbers (the value that diffs) with the human-readable form in a trailing comment.
fn render_file_stats(
    out: &mut String,
    file_name: &str,
    metrics: &[Metric],
    committed_file: Option<&Yaml<'_>>,
) {
    writeln!(out, "{file_name}:").unwrap();
    for metric in metrics {
        let committed = committed_file.get(metric.label).and_then(Yaml::as_integer);
        let value = metric.snapshot_value(committed);
        match metric.kind {
            MetricKind::Count => writeln!(out, "  {}: {value}", metric.label),
            MetricKind::Bytes { .. } => {
                writeln!(out, "  {}: {value} # {}", metric.label, format_size(value, DECIMAL))
            }
        }
        .unwrap();
    }
    out.push('\n');
}
