#![expect(
    unsafe_op_in_unsafe_fn,
    clippy::allow_attributes,
    clippy::explicit_iter_loop,
    clippy::print_stderr,
    clippy::undocumented_unsafe_blocks,
    clippy::uninlined_format_args
)]

use std::{
    alloc::{GlobalAlloc, Layout, System},
    sync::atomic::{AtomicBool, Ordering},
};

use rand::RngExt as _;

use oxc_allocator::arena::Arena;

/// A custom allocator that wraps the system allocator, but lets us force
/// allocation failures for testing.
struct Allocator(AtomicBool);

impl Allocator {
    fn is_returning_null(&self) -> bool {
        self.0.load(Ordering::SeqCst)
    }

    fn set_returning_null(&self, returning_null: bool) {
        self.0.store(returning_null, Ordering::SeqCst);
    }

    fn toggle_returning_null(&self) {
        self.set_returning_null(!self.is_returning_null());
    }

    #[allow(dead_code)] // Silence warnings for non-"collections" builds.
    fn with_successful_allocs<F, T>(&self, callback: F) -> T
    where
        F: FnOnce() -> T,
    {
        let old_returning_null = self.is_returning_null();
        self.set_returning_null(false);
        let result = callback();
        self.set_returning_null(old_returning_null);
        result
    }

    fn with_alloc_failures<F, T>(&self, callback: F) -> T
    where
        F: FnOnce() -> T,
    {
        let old_returning_null = self.is_returning_null();
        self.set_returning_null(true);
        let result = callback();
        self.set_returning_null(old_returning_null);
        result
    }
}

unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if self.is_returning_null() { core::ptr::null_mut() } else { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout);
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        if self.is_returning_null() {
            core::ptr::null_mut()
        } else {
            System.realloc(ptr, layout, new_size)
        }
    }
}

#[global_allocator]
static GLOBAL_ALLOCATOR: Allocator = Allocator(AtomicBool::new(false));

/// `assert!` may allocate on failure (e.g. for string formatting and boxing
/// panic info), so we must re-enable allocations during assertions.
macro_rules! assert {
    ($cond:expr $(, $args:tt)*) => {
        if !$cond {
            GLOBAL_ALLOCATOR.set_returning_null(false);
            panic!(concat!("Assertion failed: ", stringify!($cond)));
        }
    };
}

/// NB: We provide our own `main` rather than using the default test harness's
/// so that we can ensure that tests are executed serially, and no background
/// threads get tripped up by us disabling the global allocator, or anything
/// like that.
fn main() {
    macro_rules! test {
        ($name:expr, $test:expr $(,)*) => {
            ($name, $test as fn())
        };
    }

    fn test_static_size_alloc(
        assert_alloc_ok: fn(arena: &Arena),
        assert_alloc_err: fn(arena: &Arena),
    ) {
        // Unlike with `try_alloc_layout`, it's not that easy to test a variety
        // of size/capacity combinations here.
        // Since nothing in Arena is really random, and we have to start fresh
        // each time, just checking each case once is enough.
        for &fail_alloc in &[false, true] {
            let arena = GLOBAL_ALLOCATOR.with_successful_allocs(|| {
                // We can't query the remaining free space in the current chunk,
                // so we have to create a new Arena for each test and fill it to
                // the brink of a new allocation.
                let arena = Arena::new();

                // Arena preallocates space in the initial chunk, so we need to
                // use up this block prior to the actual test
                let layout = Layout::from_size_align(arena.chunk_capacity(), 1).unwrap();
                assert!(arena.try_alloc_layout(layout).is_ok());

                arena
            });

            GLOBAL_ALLOCATOR.set_returning_null(fail_alloc);

            if fail_alloc {
                assert_alloc_err(&arena);
            } else {
                assert_alloc_ok(&arena);
            }
        }
    }

    let tests = [
        test!("Arena::try_new fails when global allocator fails", || {
            GLOBAL_ALLOCATOR.with_alloc_failures(|| {
                assert!(Arena::try_with_capacity(1).is_err());
            });
        }),
        test!("test try_alloc_layout with and without global allocation failures", || {
            const NUM_TESTS: usize = 5000;
            const MAX_BYTES_ALLOCATED: usize = 65536;

            let mut arena = Arena::new();
            let mut bytes_allocated = arena.chunk_capacity();

            // Arena preallocates space in the initial chunk, so we need to
            // use up this block prior to the actual test
            let layout = Layout::from_size_align(arena.chunk_capacity(), 1).unwrap();
            assert!(arena.try_alloc_layout(layout).is_ok());

            let mut rng = rand::rng();

            for _ in 0..NUM_TESTS {
                if rng.random() {
                    GLOBAL_ALLOCATOR.toggle_returning_null();
                }

                let layout = Layout::from_size_align(arena.chunk_capacity() + 1, 1).unwrap();
                if GLOBAL_ALLOCATOR.is_returning_null() {
                    assert!(arena.try_alloc_layout(layout).is_err());
                } else {
                    assert!(arena.try_alloc_layout(layout).is_ok());
                    bytes_allocated += arena.chunk_capacity();
                }

                if bytes_allocated >= MAX_BYTES_ALLOCATED {
                    arena = GLOBAL_ALLOCATOR.with_successful_allocs(Arena::new);
                    bytes_allocated = arena.chunk_capacity();
                }
            }
        },),
        test!("test try_alloc with and without global allocation failures", || {
            test_static_size_alloc(
                |arena| assert!(arena.try_alloc(1u8).is_ok()),
                |arena| assert!(arena.try_alloc(1u8).is_err()),
            );
        },),
        test!("test try_alloc_with with and without global allocation failures", || {
            test_static_size_alloc(
                |arena| assert!(arena.try_alloc_with(|| 1u8).is_ok()),
                |arena| assert!(arena.try_alloc_with(|| 1u8).is_err()),
            );
        },),
    ];

    for (name, test) in tests.iter() {
        assert!(!GLOBAL_ALLOCATOR.is_returning_null());

        eprintln!("=== {} ===", name);
        test();

        GLOBAL_ALLOCATOR.set_returning_null(false);
    }
}
