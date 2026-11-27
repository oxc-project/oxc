//! Whether formatting a file called into Node (Prettier, or the Tailwind sorter).
//!
//! Such a result depends on files besides the one formatted (plugins, the Tailwind config),
//! so `--cache` doesn't keep it.
//! The external services call JS on the thread that formats the file, so a thread-local suffices.

use std::cell::Cell;

thread_local! {
    static CALLED: Cell<bool> = const { Cell::new(false) };
}

pub struct ExternalCalls;

impl ExternalCalls {
    /// Called by every external service, before calling into JS.
    #[cfg_attr(not(feature = "napi"), expect(dead_code))]
    pub fn mark() {
        CALLED.set(true);
    }

    /// Runs `f`, and returns its result with whether it called into Node.
    pub fn track<T>(f: impl FnOnce() -> T) -> (T, bool) {
        CALLED.set(false);
        let result = f();
        (result, CALLED.take())
    }
}
