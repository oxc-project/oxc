//! Common test runner patterns
//!
//! This module provides standardized interfaces and patterns for test runners
//! used across different conformance and testing tasks.

/// Common trait for test runners across different tasks
pub trait TestRunner {
    type Options;

    /// Create a new test runner with the given options
    fn new(options: Self::Options) -> Self;

    /// Run the tests/checks
    fn run(&self);
}

/// Standard options commonly used across test runners
pub trait TestRunnerOptions: Clone + Default {
    /// Whether to run in debug mode (usually single-threaded)
    fn debug(&self) -> bool;

    /// Optional filter for which tests to run
    fn filter(&self) -> Option<&str>;
}

/// Common implementation for test runner options
#[derive(Debug, Clone)]
pub struct StandardTestOptions {
    pub debug: bool,
    pub filter: Option<String>,
}

impl Default for StandardTestOptions {
    fn default() -> Self {
        Self {
            debug: false,
            filter: None,
        }
    }
}

impl TestRunnerOptions for StandardTestOptions {
    fn debug(&self) -> bool {
        self.debug
    }

    fn filter(&self) -> Option<&str> {
        self.filter.as_deref()
    }
}

/// Utility to configure thread pool based on debug option
pub fn configure_thread_pool(debug: bool) {
    if debug {
        rayon::ThreadPoolBuilder::new()
            .num_threads(1)
            .build_global()
            .expect("Failed to set rayon thread pool to single thread");
    }
}