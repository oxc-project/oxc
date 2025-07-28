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
static NUM_ALLOC: AtomicUsize = AtomicUsize::new(0);
/// Number of bytes allocated in system allocator
static BYTES_ALLOC: AtomicUsize = AtomicUsize::new(0);

fn reset_global_allocs() {
    NUM_ALLOC.store(0, SeqCst);
    BYTES_ALLOC.store(0, SeqCst);
}

// SAFETY: Methods simply delegate to `MiMalloc` allocator to ensure that the allocator
// is the same across different platforms for the purposes of tracking allocations.
#[expect(unsafe_code, clippy::undocumented_unsafe_blocks)]
unsafe impl GlobalAlloc for TrackedAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ret = unsafe { mimalloc_safe::MiMalloc.alloc(layout) };
        if !ret.is_null() {
            BYTES_ALLOC.fetch_add(layout.size(), SeqCst);
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
    let fixture_width = files
        .files()
        .iter()
        .max_by(|x, y| x.file_name.len().cmp(&y.file_name.len()))
        .unwrap()
        .file_name
        .len();
    writeln!(
        out,
        "{:fixture_width$} | {:width$} | {:width$} | {:width$} | {:width$} | {:width$} | {:width$} | {:width$} |",
        "File",
        "File size",
        "Sys allocs",
        "Sys bytes",
        "Sys ratio",
        "Arena allocs",
        "Arena bytes",
        "Arena ratio",
        width = width,
    )
    .unwrap();
    out.push_str(&str::repeat("-", width * 8 + fixture_width + 11));
    out.push('\n');

    let mut allocator = Allocator::default();
    for file in files.files() {
        // TODO: Reset num allocs!
        allocator.reset();
        reset_global_allocs();

        let ret = Parser::new(&allocator, &file.source_text, file.source_type)
            .with_options(ParseOptions {
                parse_regular_expression: true,
                ..ParseOptions::default()
            })
            .parse();

        assert!(ret.errors.is_empty());

        let file_size = file.source_text.len();
        let sys_allocs = NUM_ALLOC.load(SeqCst);
        let sys_bytes = BYTES_ALLOC.load(SeqCst);
        #[expect(clippy::cast_precision_loss)]
        let sys_ratio = if file_size > 0 { sys_bytes as f64 / file_size as f64 } else { 0.0 };
        let sys_ratio = format!("{sys_ratio:.2}");
        let arena_allocs = Allocator::num_alloc();
        let arena_bytes = allocator.used_bytes();
        #[expect(clippy::cast_precision_loss)]
        let arena_ratio = if file_size > 0 { arena_bytes as f64 / file_size as f64 } else { 0.0 };
        let arena_ratio = format!("{arena_ratio:.2}");

        let s = format!(
            "{:fixture_width$} | {:width$} | {:width$} | {:width$} | {:width$} | {:width$} | {:width$} | {:width$}\n",
            file.file_name.as_str(),
            format_size(file_size, DECIMAL),
            sys_allocs,
            format_size(sys_bytes, DECIMAL),
            sys_ratio,
            arena_allocs,
            format_size(arena_bytes, DECIMAL),
            arena_ratio,
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
