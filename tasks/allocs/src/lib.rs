#![expect(clippy::print_stdout)]

use std::{
    fmt::Write as _,
    fs::File,
    io::{self, Write},
};

use humansize::{DECIMAL, format_size};

use oxc_allocator::Allocator;
use oxc_parser::{ParseOptions, Parser};
use oxc_tasks_common::{TestFiles, project_root};

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

fn reset_global_allocs() {
    NUM_ALLOC.store(0, SeqCst);
}

// SAFETY: Methods simply delegate to `MiMalloc` allocator to ensure that the allocator
// is the same across different platforms for the purposes of tracking allocations.
#[expect(unsafe_code, clippy::undocumented_unsafe_blocks)]
unsafe impl GlobalAlloc for TrackedAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ret = unsafe { mimalloc_safe::MiMalloc.alloc(layout) };
        if !ret.is_null() {
            NUM_ALLOC.fetch_add(1, SeqCst);
        }
        ret
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe {
            mimalloc_safe::MiMalloc.dealloc(ptr, layout);
        }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        unsafe { mimalloc_safe::MiMalloc.alloc_zeroed(layout) }
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        unsafe { mimalloc_safe::MiMalloc.realloc(ptr, layout, new_size) }
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
    let snap_path = project_root().join("tasks/allocs/allocs_parser.snap");

    let mut out = String::new();

    let width = 12;
    let fixture_width = files.files().iter().map(|file| file.file_name.len()).max().unwrap();
    writeln!(
        out,
        "{:fixture_width$} | {:width$} | {:width$} | {:width$} | {:width$} ",
        "File",
        "File size",
        "Sys allocs",
        "Arena allocs",
        "Arena bytes",
        width = width,
    )
    .unwrap();
    out.push_str(&str::repeat("-", width * 4 + fixture_width + 15));
    out.push('\n');

    let mut allocator = Allocator::default();

    let options = ParseOptions { parse_regular_expression: true, ..ParseOptions::default() };

    // Warm-up by parsing each file first, and then measuring the actual allocations.
    // TODO: For some reason, this causes some variance in the total system allocations, specifically
    // for checker.ts and antd.js. I tried to track this down but wasn't able to figure out why.
    for file in files.files() {
        Parser::new(&allocator, &file.source_text, file.source_type).with_options(options).parse();
    }

    for file in files.files() {
        allocator.reset();
        reset_global_allocs();

        Parser::new(&allocator, &file.source_text, file.source_type).with_options(options).parse();

        let sys_allocs = NUM_ALLOC.load(SeqCst);
        let arena_allocs = allocator.num_alloc();
        let arena_bytes = allocator.used_bytes();

        let s = format!(
            // Using two newlines at the end makes it easier to diff
            "{:fixture_width$} | {:width$} | {:width$} | {:width$} | {:width$}\n\n",
            file.file_name.as_str(),
            format_size(file.source_text.len(), DECIMAL),
            sys_allocs,
            arena_allocs,
            format_size(arena_bytes, DECIMAL),
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
