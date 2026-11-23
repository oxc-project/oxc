//! Whether formatting a file called into Node (Prettier, or the Tailwind sorter).
//!
//! Such a result depends on files besides the one formatted (Prettier, its plugins, the
//! project's Tailwind configuration), so `--cache` doesn't keep it.
//! The external services call JS on the thread that formats the file, so a thread-local suffices.

use std::cell::Cell;

thread_local! {
    static CALLED: Cell<bool> = const { Cell::new(false) };
}

/// Tracks, per thread, whether formatting called into Node.
pub struct ExternalCalls;

impl ExternalCalls {
    /// Called by every external service, before calling into JS.
    #[cfg_attr(not(feature = "napi"), expect(dead_code))]
    pub fn mark() {
        CALLED.with(|called| called.set(true));
    }

    /// Run `f`, and return its result with whether it called into Node.
    pub fn track<T>(f: impl FnOnce() -> T) -> (T, bool) {
        let before = CALLED.with(Cell::take);
        let result = f();
        let called = CALLED.with(|called| called.replace(before));
        (result, called)
    }
}
