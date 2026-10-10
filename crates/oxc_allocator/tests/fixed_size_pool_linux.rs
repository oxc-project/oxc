#![cfg(all(
    feature = "fixed_size",
    not(miri),
    target_os = "linux",
    target_pointer_width = "64",
    target_endian = "little"
))]

use std::{alloc::Layout, process::Command};

use oxc_allocator::AllocatorPool;

const CHILD_MARKER: &str = "OXC_ALLOCATOR_FIXED_SIZE_POOL_RLIMIT_DATA_CHILD";
const DATA_LIMIT: libc::rlim_t = 256 * 1024 * 1024;
const ALLOCATION_SIZE: usize = 64 * 1024;

#[test]
fn fixed_size_pool_with_limited_committed_memory() {
    if std::env::var_os(CHILD_MARKER).is_some() {
        // Limit the child process only: the test runner may execute other tests in parallel.
        let mut limit = libc::rlimit { rlim_cur: 0, rlim_max: 0 };
        // SAFETY: `limit` points to a valid writable `rlimit` structure.
        assert_eq!(unsafe { libc::getrlimit(libc::RLIMIT_DATA, &raw mut limit) }, 0);
        limit.rlim_cur = limit.rlim_cur.min(DATA_LIMIT);
        // SAFETY: Lowering the soft limit while preserving the hard limit is permitted.
        assert_eq!(unsafe { libc::setrlimit(libc::RLIMIT_DATA, &raw const limit) }, 0);

        let pool = AllocatorPool::new_fixed_size(4);
        let allocators: [_; 4] = std::array::from_fn(|_| pool.get());
        for allocator in &allocators {
            let ptr = allocator.alloc_layout(Layout::from_size_align(ALLOCATION_SIZE, 1).unwrap());
            // SAFETY: The first and last bytes lie within the allocation just returned.
            unsafe {
                ptr.as_ptr().write(1);
                ptr.as_ptr().add(ALLOCATION_SIZE - 1).write(2);
            }
        }
        return;
    }

    let status = Command::new(std::env::current_exe().unwrap())
        .arg("--exact")
        .arg("fixed_size_pool_with_limited_committed_memory")
        .arg("--nocapture")
        .env(CHILD_MARKER, "1")
        .status()
        .unwrap();
    assert!(status.success(), "fixed-size allocator pool failed under RLIMIT_DATA");
}
