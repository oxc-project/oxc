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
use oxc_tasks_common::{TestFiles, project_root};

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

fn reset_global_allocs() {
    NUM_ALLOC.reset();
    NUM_REALLOC.reset();
}

// SAFETY: Methods simply delegate to `MiMalloc` allocator to ensure that the allocator
// is the same across different platforms for the purposes of tracking allocations.
#[expect(unsafe_code, clippy::undocumented_unsafe_blocks)]
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

#[test]
#[cfg(any(coverage, coverage_nightly))]
fn test() {
    run().unwrap();
}

/// # Panics
/// # Errors
pub fn run() -> Result<(), io::Error> {
    let files = TestFiles::complicated();
    let snap_path = project_root().join("tasks/track_memory_allocations/allocs_parser.snap");

    let mut out = String::new();

    let width = 14;
    let fixture_width = files.files().iter().map(|file| file.file_name.len()).max().unwrap();
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
    )
    .unwrap();
    out.push_str(&str::repeat("-", width * 7 + fixture_width + 15));
    out.push('\n');

    let mut allocator = Allocator::default();

    let options = ParseOptions { parse_regular_expression: true, ..ParseOptions::default() };

    // Warm-up by parsing each file first, and then measuring the actual allocations. This reduces variance
    // in the number of allocations, because we ensure that the bump allocator has already requested all
    // of the space it will need from the system allocator to parse the largest file in the set.
    for file in files.files() {
        Parser::new(&allocator, &file.source_text, file.source_type).with_options(options).parse();
    }

    for file in files.files() {
        allocator.reset();
        reset_global_allocs();

        Parser::new(&allocator, &file.source_text, file.source_type).with_options(options).parse();

        let sys_allocs = NUM_ALLOC.get();
        let sys_reallocs = NUM_REALLOC.get();
        #[cfg(not(feature = "is_all_features"))]
        let (arena_allocs, arena_reallocs) = allocator.get_allocation_stats();
        #[cfg(feature = "is_all_features")]
        let (arena_allocs, arena_reallocs) = (0, 0);
        let arena_bytes = allocator.used_bytes();

        let s = format!(
            // Using two newlines at the end makes it easier to diff
            "{:fixture_width$} | {:width$} || {:width$} | {:width$} || {:width$} | {:width$} | {:width$}\n\n",
            file.file_name.as_str(),
            format_size(file.source_text.len(), DECIMAL),
            sys_allocs,
            sys_reallocs,
            arena_allocs,
            arena_reallocs,
            format_size(arena_bytes, DECIMAL.decimal_places(3)),
            width = width
        );
        out.push_str(&s);
    }

    println!("{out}");

    let mut snapshot = File::create(snap_path)?;
    snapshot.write_all(out.as_bytes())?;
    snapshot.flush()?;

    Ok(())
}
